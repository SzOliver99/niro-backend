use anyhow::{Ok, Result, anyhow};
use chacha20poly1305::Key;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::Type;
use strum::{AsRefStr, Display, EnumString};
use uuid::Uuid;

use crate::{
    database::Database,
    models::{customer::Customer, dto::ContractDto, user::User},
    utils::encrypt::{self, HmacSecret},
};

#[skip_serializing_none]
#[derive(Debug, Serialize, Default, Clone)]
pub struct Contract {
    pub id: Option<i32>,
    pub uuid: Option<Uuid>,
    pub contract_number: Option<String>,
    pub contract_type: Option<ContractType>,
    pub annual_fee: Option<i32>,
    pub payment_frequency: Option<PaymentFrequency>,
    pub payment_method: Option<PaymentMethod>,
    pub customer_id: Option<i32>,
    pub user_id: Option<i32>,
    pub created_by: Option<String>,
    pub handle_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumString, Display, Type, AsRefStr)]
pub enum ContractType {
    BonusLifeProgram,
    LifeProgram,
    AllianzCareNow,
    HealthProgram,
    MyhomeHomeInsurance,
    MfoHomeInsurance,
    CorporatePropertyInsurance,
    Kgfb,
    Casco,
    TravelInsurance,
    CondominiumInsurance,
    AgriculturalInsurance,
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumString, Display, Type, AsRefStr)]
pub enum PaymentFrequency {
    Monthly,
    Quarterly,
    Semiannual,
    Annual,
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumString, Display, Type, AsRefStr)]
pub enum PaymentMethod {
    CreditCard,
    Transfer,
    DirectDebit,
    Check,
}

// CONTRACT UTILS //
impl Contract {
    pub async fn get_id_by_uuid(db: &Database, contract_uuid: Option<Uuid>) -> Result<Option<i32>> {
        let user = sqlx::query_scalar!(
            "SELECT id FROM customer_contracts WHERE uuid = $1",
            contract_uuid
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_uuid_by_id(db: &Database, contract_id: i32) -> Result<Option<Uuid>> {
        let user = sqlx::query!(
            "SELECT uuid FROM customer_contracts WHERE id = $1",
            contract_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(user.uuid)
    }

    pub(super) async fn is_exists(db: &Database, contract: &Contract) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM customer_contracts
             WHERE contract_number = $1",
            contract.contract_number
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }

    pub(super) async fn is_exists_by_id(db: &Database, contract_id: i32) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM customer_contracts
             WHERE id = $1",
            contract_id
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }
}

// CONTRACT CALLBACKS //
impl Contract {
    pub async fn create(
        db: &Database,
        key: &Key,
        hmac_secret: &HmacSecret,
        user_uuid: Uuid,
        customer: Customer,
        contract: Contract,
    ) -> Result<i32> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;
        let row = sqlx::query!(
            "SELECT id FROM customers
             WHERE email_hash = $1 OR phone_number_hash = $2",
            encrypt::hash_value(hmac_secret, &customer.email.as_ref().unwrap()),
            encrypt::hash_value(hmac_secret, &customer.phone_number.as_ref().unwrap()),
        )
        .fetch_optional(&db.pool)
        .await?;

        // Determine customer_id: create customer if not exists, otherwise use existing id
        let customer_id = if let Some(existing) = row {
            existing.id
        } else {
            Customer::create(db, key, hmac_secret, user_uuid, customer.clone()).await?
        };

        let row = sqlx::query!(
            "INSERT INTO customer_contracts(contract_number, contract_type, annual_fee, payment_frequency, payment_method, customer_id, user_id, created_by)
             VALUES($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id",
            contract.contract_number,
            contract.contract_type.map(|c| c.to_string()),
            contract.annual_fee,
            contract.payment_frequency.map(|c| c.to_string()),
            contract.payment_method.map(|c| c.to_string()),
            customer_id,
            user_id,
            contract.created_by
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(row.id)
    }

    pub async fn modify(
        db: &Database,
        contract_uuid: Uuid,
        updated_contract: Contract,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE customer_contracts
             SET contract_number = $1,
                 contract_type = $2,
                 annual_fee = $3,
                 payment_frequency = $4,
                 payment_method = $5,
                 handle_at = NOW()
             WHERE uuid = $6",
            updated_contract.contract_number,
            updated_contract.contract_type.map(|c| c.to_string()),
            updated_contract.annual_fee,
            updated_contract.payment_frequency.map(|c| c.to_string()),
            updated_contract.payment_method.map(|c| c.to_string()),
            contract_uuid
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get_all(db: &Database, key: &Key, user_uuid: Uuid) -> Result<Vec<ContractDto>> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let rows = sqlx::query!(
            r#"
            SELECT
                c.full_name,
                c.phone_number_enc,
                c.phone_number_nonce,
                c.email_enc,
                c.email_nonce,
                c.address_enc,
                c.address_nonce,
                cc.uuid,
                cc.contract_number,
                cc.contract_type,
                cc.annual_fee,
                cc.payment_frequency,
                cc.payment_method,
                cc.handle_at,
                cc.created_by
            FROM
                customers c
                JOIN customer_contracts cc ON cc.customer_id = c.id
            WHERE
                cc.user_id = $1
            ORDER BY cc.handle_at DESC
            "#,
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        let contracts: Vec<ContractDto> = rows
            .into_iter()
            .map(|row| ContractDto {
                uuid: row.uuid,
                full_name: row.full_name,
                phone_number: encrypt::decrypt_value(
                    key,
                    &row.phone_number_enc,
                    &row.phone_number_nonce,
                )
                .unwrap_or_default(),
                email: encrypt::decrypt_value(key, &row.email_enc, &row.email_nonce)
                    .unwrap_or_default(),
                address: encrypt::decrypt_value(key, &row.address_enc, &row.address_nonce)
                    .unwrap_or_default(),
                contract_number: row.contract_number,
                contract_type: row.contract_type.parse().unwrap(),
                annual_fee: row.annual_fee,
                payment_frequency: row.payment_frequency.parse().unwrap(),
                payment_method: row.payment_method.parse().unwrap(),
                created_by: row.created_by,
                handle_at: row.handle_at,
            })
            .collect();

        Ok(contracts)
    }

    pub async fn get_by_customer_uuid(db: &Database, customer_uuid: Uuid) -> Result<Vec<Contract>> {
        let customer_id = Customer::get_id_by_uuid(db, Some(customer_uuid))
            .await?
            .unwrap();
        println!("{customer_id}");
        let rows = sqlx::query!(
            "SELECT
                uuid,
                contract_number,
                contract_type,
                annual_fee,
                payment_frequency,
                payment_method,
                handle_at,
                created_by
            FROM
                customer_contracts
            WHERE
	            customer_id = $1",
            customer_id
        )
        .fetch_all(&db.pool)
        .await?;

        let items: Vec<Contract> = rows
            .into_iter()
            .map(|row| Contract {
                uuid: row.uuid,
                contract_number: Some(row.contract_number),
                contract_type: Some(row.contract_type.parse().unwrap()),
                annual_fee: Some(row.annual_fee),
                payment_frequency: Some(row.payment_frequency.parse().unwrap()),
                payment_method: Some(row.payment_method.parse().unwrap()),
                handle_at: Some(row.handle_at),
                created_by: Some(row.created_by),
                ..Default::default()
            })
            .collect();

        Ok(items)
    }

    pub async fn get_by_uuid(db: &Database, contract_uuid: Uuid) -> Result<Contract> {
        let row = sqlx::query!(
            "SELECT
                uuid,
                contract_number,
                contract_type,
                annual_fee,
                payment_frequency,
                payment_method,
                handle_at,
                created_by
            FROM
                customer_contracts
            WHERE
	            uuid = $1",
            contract_uuid
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(Contract {
            uuid: row.uuid,
            contract_number: Some(row.contract_number),
            contract_type: Some(row.contract_type.parse().unwrap()),
            annual_fee: Some(row.annual_fee),
            payment_frequency: Some(row.payment_frequency.parse().unwrap()),
            payment_method: Some(row.payment_method.parse().unwrap()),
            handle_at: Some(row.handle_at),
            created_by: Some(row.created_by),
            ..Default::default()
        })
    }

    pub async fn get_customer_uuid(db: &Database, contract_uuid: Uuid) -> Result<Option<Uuid>> {
        let customer = sqlx::query!(
            "SELECT
                c.uuid
            FROM
                customers c
                JOIN customer_contracts cc ON c.id = cc.customer_id
            WHERE 
                cc.uuid = $1",
            contract_uuid
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(customer.uuid)
    }

    pub async fn change_handler(
        db: &Database,
        user_full_name: String,
        contract_uuids: Vec<Uuid>,
    ) -> Result<()> {
        let user = sqlx::query!(
            "SELECT user_id as id FROM user_info WHERE full_name = $1",
            user_full_name
        )
        .fetch_one(&db.pool)
        .await?;

        sqlx::query!(
            "UPDATE customer_contracts
             SET user_id = $2
             WHERE uuid = ANY($1)",
            &contract_uuids,
            user.id
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(db: &Database, contract_uuids: Vec<Uuid>) -> Result<()> {
        sqlx::query!(
            "DELETE FROM customer_contracts
             WHERE uuid = ANY($1)",
            &contract_uuids
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }
}
