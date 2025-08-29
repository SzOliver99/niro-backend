use anyhow::{Ok, Result};
use chacha20poly1305::Key;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{
    database::Database,
    models::user::User,
    utils::encrypt::{self},
};

#[skip_serializing_none]
#[derive(Debug, Serialize, Default)]
pub struct Customer {
    pub id: Option<i32>,
    pub full_name: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub user_id: Option<i32>,
    pub created_by: Option<String>,
    pub leads: Vec<Customer>,
}

impl Customer {
    pub(super) async fn is_exists(
        db: &Database,
        hmac_secret: &Vec<u8>,
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
        hmac_secret: &Vec<u8>,
        new_customer: Customer,
        user: User,
    ) -> Result<()> {
        if Self::is_exists(db, &hmac_secret, &new_customer).await? {
            return Err(anyhow::anyhow!("Az ügyfél már szerepel az adatbázisban."));
        }

        // Borrow inner strings, hash and encrypt without moving from new_customer
        let (
            email_hash,
            phone_hash,
            email_enc,
            email_nonce,
            phone_enc,
            phone_nonce,
            address_enc,
            address_nonce,
        ) = {
            let email = new_customer.email.as_deref().unwrap();
            let phone = new_customer.phone_number.as_deref().unwrap();
            let address = new_customer.address.as_deref().unwrap();

            let email_hash = encrypt::hash_value(&hmac_secret, email);
            let phone_hash = encrypt::hash_value(&hmac_secret, phone);

            let (email_enc, email_nonce) = encrypt::encrypt_value(&key, email);
            let (phone_enc, phone_nonce) = encrypt::encrypt_value(&key, phone);
            let (address_enc, address_nonce) = encrypt::encrypt_value(&key, address);

            (
                email_hash,
                phone_hash,
                email_enc,
                email_nonce,
                phone_enc,
                phone_nonce,
                address_enc,
                address_nonce,
            )
        };

        let _row = sqlx::query!(
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
            new_customer.user_id,
            user.info.full_name
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get_by_id(db: &Database, key: &Key, user_id: i32) -> Result<Self> {
        let row = sqlx::query!(
            "SELECT full_name, phone_number_enc, phone_number_nonce, email_enc, email_nonce, address_enc, address_nonce, user_id 
             FROM customers
             WHERE id = $1",
            user_id
        )
        .fetch_one(&db.pool)
        .await?;
        Ok(Customer {
            full_name: Some(row.full_name),
            phone_number: encrypt::decrypt_value(
                key,
                &row.phone_number_enc,
                &row.phone_number_nonce,
            ),
            email: encrypt::decrypt_value(key, &row.email_enc, &row.email_nonce),
            address: encrypt::decrypt_value(key, &row.address_enc, &row.address_nonce),
            user_id: row.user_id,
            ..Default::default()
        })
    }

    pub async fn get_all(db: &Database, key: &Key, user_id: i32) -> Result<Vec<Self>> {
        let row = sqlx::query!(
            "SELECT id, full_name, phone_number_enc, phone_number_nonce, email_enc, email_nonce, address_enc, address_nonce, user_id, created_by
             FROM customers
             WHERE user_id = $1",
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        let customers: Vec<Customer> = row
            .into_iter()
            .map(|customer| Customer {
                id: Some(customer.id),
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
        customer_ids: Vec<i32>,
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
             WHERE id = ANY($1)",
            &customer_ids,
            user.id
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(db: &Database, customer_ids: Vec<i32>) -> Result<()> {
        for customer_id in customer_ids {
            if !Customer::is_exists_by_id(db, customer_id).await? {
                return Err(anyhow::anyhow!("Nem létező ügyfél"));
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
