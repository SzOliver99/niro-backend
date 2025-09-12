use anyhow::{Ok, Result, anyhow};
use chacha20poly1305::Key;
use serde::Serialize;
use serde_with::skip_serializing_none;
use sqlx::types::Uuid;

use crate::{
    database::Database,
    models::user::User,
    utils::encrypt::{self, HmacSecret},
};

#[skip_serializing_none]
#[derive(Debug, Serialize, Default, Clone)]
pub struct Customer {
    pub id: Option<i32>,
    pub uuid: Option<Uuid>,
    pub full_name: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub comment: Option<String>,
    pub user_id: Option<i32>,
    pub created_by: Option<String>,
}

impl Customer {
    pub async fn get_id_by_uuid(db: &Database, customer_uuid: Option<Uuid>) -> Result<Option<i32>> {
        let user = sqlx::query_scalar!("SELECT id FROM customers WHERE uuid = $1", customer_uuid)
            .fetch_optional(&db.pool)
            .await?;

        Ok(user)
    }

    pub async fn get_uuid_by_id(db: &Database, customer_id: i32) -> Result<Option<Uuid>> {
        let user = sqlx::query!("SELECT uuid FROM customers WHERE id = $1", customer_id)
            .fetch_one(&db.pool)
            .await?;

        Ok(user.uuid)
    }

    pub(super) async fn is_exists(
        db: &Database,
        hmac_secret: &HmacSecret,
        customer: &Customer,
    ) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM customers
             WHERE email_hash = $1 OR phone_number_hash = $2",
            encrypt::hash_value(hmac_secret, &customer.email.as_ref().unwrap()),
            encrypt::hash_value(hmac_secret, &customer.phone_number.as_ref().unwrap()),
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }

    pub(super) async fn is_exists_by_id(db: &Database, customer_id: i32) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM customers
             WHERE id = $1",
            customer_id
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }
}

impl Customer {
    pub async fn create(
        db: &Database,
        key: &Key,
        hmac_secret: &HmacSecret,
        user_uuid: Uuid,
        new_customer: Customer,
    ) -> Result<i32> {
        if Self::is_exists(db, &hmac_secret, &new_customer).await? {
            return Err(anyhow!("Az ügyfél már szerepel az adatbázisban."));
        }

        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let email = new_customer.email.as_deref().unwrap();
        let phone = new_customer.phone_number.as_deref().unwrap();
        let address = new_customer.address.as_deref().unwrap();

        let email_hash = encrypt::hash_value(&hmac_secret, email);
        let phone_hash = encrypt::hash_value(&hmac_secret, phone);

        let (email_enc, email_nonce) = encrypt::encrypt_value(&key, email);
        let (phone_enc, phone_nonce) = encrypt::encrypt_value(&key, phone);
        let (address_enc, address_nonce) = encrypt::encrypt_value(&key, address);

        let row = sqlx::query!(
            "INSERT INTO customers(full_name, phone_number_enc, phone_number_nonce, phone_number_hash, email_enc, email_nonce, email_hash, address_enc, address_nonce, user_id, created_by)
             VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING id",
            new_customer.full_name,
            phone_enc,
            phone_nonce,
            phone_hash,
            email_enc,
            email_nonce,
            email_hash,
            address_enc,
            address_nonce,
            user_id,
            new_customer.created_by
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(row.id)
    }

    pub async fn modify(
        db: &Database,
        key: &Key,
        hmac_secret: &HmacSecret,
        customer_uuid: Uuid,
        updated_customer: Customer,
    ) -> Result<()> {
        let email = updated_customer.email.as_deref().unwrap_or_default();
        let phone = updated_customer.phone_number.as_deref().unwrap_or_default();
        let address = updated_customer.address.as_deref().unwrap_or_default();

        let email_hash = encrypt::hash_value(hmac_secret, email);
        let phone_hash = encrypt::hash_value(hmac_secret, phone);

        let (email_enc, email_nonce) = encrypt::encrypt_value(key, email);
        let (phone_enc, phone_nonce) = encrypt::encrypt_value(key, phone);
        let (address_enc, address_nonce) = encrypt::encrypt_value(key, address);

        sqlx::query!(
            "UPDATE customers
             SET full_name = $1,
                 phone_number_enc = $2,
                 phone_number_nonce = $3,
                 phone_number_hash = $4,
                 email_enc = $5,
                 email_nonce = $6,
                 email_hash = $7,
                 address_enc = $8,
                 address_nonce = $9
             WHERE uuid = $10",
            updated_customer.full_name,
            phone_enc,
            phone_nonce,
            phone_hash,
            email_enc,
            email_nonce,
            email_hash,
            address_enc,
            address_nonce,
            customer_uuid
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn save_comment(db: &Database, customer_uuid: Uuid, comment: String) -> Result<()> {
        sqlx::query!(
            "UPDATE customers
             SET comment = $1
             WHERE uuid = $2",
            comment,
            customer_uuid
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get_by_uuid(db: &Database, key: &Key, customer_uuid: Uuid) -> Result<Self> {
        let row = sqlx::query!(
            "SELECT uuid, full_name, phone_number_enc, phone_number_nonce, email_enc, email_nonce, address_enc, address_nonce, comment, user_id
             FROM customers
             WHERE uuid = $1",
             customer_uuid
        )
        .fetch_one(&db.pool)
        .await?;
        Ok(Customer {
            uuid: row.uuid,
            full_name: Some(row.full_name),
            phone_number: encrypt::decrypt_value(
                key,
                &row.phone_number_enc,
                &row.phone_number_nonce,
            ),
            email: encrypt::decrypt_value(key, &row.email_enc, &row.email_nonce),
            address: encrypt::decrypt_value(key, &row.address_enc, &row.address_nonce),
            comment: row.comment,
            user_id: row.user_id,
            ..Default::default()
        })
    }

    pub async fn get_all(db: &Database, key: &Key, user_uuid: Uuid) -> Result<Vec<Self>> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;
        let row = sqlx::query!(
            "SELECT uuid, full_name, phone_number_enc, phone_number_nonce, email_enc, email_nonce, address_enc, address_nonce, user_id, created_by
             FROM customers
             WHERE user_id = $1",
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        let customers: Vec<Customer> = row
            .into_iter()
            .map(|customer| Customer {
                uuid: customer.uuid,
                full_name: Some(customer.full_name),
                phone_number: encrypt::decrypt_value(
                    key,
                    &customer.phone_number_enc,
                    &customer.phone_number_nonce,
                ),
                email: encrypt::decrypt_value(key, &customer.email_enc, &customer.email_nonce),
                address: encrypt::decrypt_value(
                    key,
                    &customer.address_enc,
                    &customer.address_nonce,
                ),
                user_id: customer.user_id,
                created_by: Some(customer.created_by),
                ..Default::default()
            })
            .collect();
        Ok(customers)
    }

    pub async fn change_handler(
        db: &Database,
        user_full_name: String,
        customer_ids: Vec<Uuid>,
    ) -> Result<()> {
        let user = sqlx::query!(
            "SELECT user_id as id FROM user_info WHERE full_name = $1",
            user_full_name
        )
        .fetch_one(&db.pool)
        .await?;

        sqlx::query!(
            "UPDATE customers
             SET user_id = $2
             WHERE uuid = ANY($1)",
            &customer_ids,
            user.id
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(db: &Database, customer_ids: Vec<Uuid>) -> Result<()> {
        for customer_uuid in customer_ids {
            let customer_id = Self::get_id_by_uuid(db, Some(customer_uuid))
                .await?
                .unwrap();
            if !Customer::is_exists_by_id(db, customer_id).await? {
                return Err(anyhow!("Nem létező ügyfél"));
            }

            sqlx::query!(
                "DELETE FROM customers
                 WHERE id = $1",
                customer_id
            )
            .execute(&db.pool)
            .await?;
        }

        Ok(())
    }
}
