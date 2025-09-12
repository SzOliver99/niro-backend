use std::str::FromStr;

use anyhow::{Result, anyhow};
use chacha20poly1305::Key;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::Type;
use strum::{AsRefStr, Display, EnumString};
use uuid::Uuid;

use crate::{
    database::Database,
    models::{customer::Customer, dto::LeadListItemDto, user::User},
    utils::encrypt::{self, HmacSecret},
};

#[skip_serializing_none]
#[derive(Debug, Serialize, Default)]
pub struct Lead {
    pub id: Option<i32>,
    pub uuid: Option<Uuid>,
    pub lead_type: Option<LeadType>,
    pub inquiry_type: Option<String>,
    pub lead_status: Option<LeadStatus>,
    pub handle_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, AsRefStr, EnumString, Display)]
pub enum LeadType {
    Personal,
    Recommendation,
    Salesforce,
    RedLead,
    BlueLead,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, AsRefStr, EnumString, Display)]
pub enum LeadStatus {
    Opened,
    InProgress,
    Closed,
}

impl Lead {
    async fn is_exists(db: &Database, lead: &Lead) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM customer_leads
             WHERE inquiry_type = $1",
            lead.inquiry_type,
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }

    async fn is_exists_by_id(db: &Database, lead_uuid: Uuid) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM customer_leads
             WHERE uuid = $1",
            lead_uuid
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }
}

impl Lead {
    pub async fn create(
        db: &Database,
        key: &Key,
        hmac_secret: &HmacSecret,
        user_uuid: Uuid,
        customer: Customer,
        lead: Lead,
    ) -> Result<()> {
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

        let _row = sqlx::query!(
            "INSERT INTO customer_leads(lead_type, inquiry_type, lead_status, customer_id, user_id, created_by)
             VALUES($1,$2, $3, $4, $5, $6)
             RETURNING id",
            lead.lead_type.map(|t| t.to_string()),
            lead.inquiry_type,
            lead.lead_status.map(|l| l.to_string()),
            customer_id,
            user_id,
            lead.created_by
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn modify(db: &Database, lead_uuid: Uuid, updated_lead: Lead) -> Result<()> {
        sqlx::query!(
            "UPDATE customer_leads
             SET lead_type = $1,
                 inquiry_type = $2,
                 lead_status = $3,
                 handle_at = NOW()
             WHERE uuid = $4",
            updated_lead.lead_type.map(|t| t.to_string()),
            updated_lead.inquiry_type,
            updated_lead.lead_status.map(|s| s.to_string()),
            lead_uuid
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get_all(
        db: &Database,
        key: &Key,
        user_uuid: Uuid,
    ) -> Result<Vec<LeadListItemDto>> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;
        let rows = sqlx::query!(
            "SELECT c.full_name, c.phone_number_enc, c.phone_number_nonce, c.email_enc, c.email_nonce, c.address_enc, c.address_nonce, l.uuid, l.lead_type, l.inquiry_type, l.lead_status, l.handle_at, l.created_by
             FROM customers c
             JOIN customer_leads l ON l.customer_id = c.id
             WHERE l.user_id = $1",
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        let items: Vec<LeadListItemDto> = rows
            .into_iter()
            .map(|row| LeadListItemDto {
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
                lead_type: row.lead_type,
                inquiry_type: row.inquiry_type,
                lead_status: row.lead_status,
                handle_at: row.handle_at,
                created_by: row.created_by,
            })
            .collect();

        Ok(items)
    }

    pub async fn get_by_customer_uuid(db: &Database, customer_uuid: Uuid) -> Result<Vec<Lead>> {
        let customer_id = Customer::get_id_by_uuid(db, Some(customer_uuid))
            .await?
            .unwrap();
        println!("{customer_id}");
        let rows = sqlx::query!(
            "SELECT
                uuid,
                lead_type,
                inquiry_type,
                lead_status,
                handle_at,
                created_by
            FROM
                customer_leads
            WHERE
	            customer_id = $1",
            customer_id
        )
        .fetch_all(&db.pool)
        .await?;

        let items: Vec<Lead> = rows
            .into_iter()
            .map(|row| Lead {
                uuid: row.uuid,
                lead_type: Some(row.lead_type.parse().unwrap()),
                inquiry_type: Some(row.inquiry_type),
                lead_status: LeadStatus::from_str(&row.lead_status).ok(),
                handle_at: Some(row.handle_at),
                created_by: Some(row.created_by),
                ..Default::default()
            })
            .collect();

        Ok(items)
    }

    pub async fn get_by_uuid(db: &Database, lead_uuid: Uuid) -> Result<Lead> {
        let row = sqlx::query!(
            "SELECT
                uuid,
                lead_type,
                inquiry_type,
                lead_status,
                handle_at,
                created_by
            FROM
                customer_leads
            WHERE
	            uuid = $1",
            lead_uuid
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(Lead {
            uuid: row.uuid,
            lead_type: Some(row.lead_type.parse().unwrap()),
            inquiry_type: Some(row.inquiry_type),
            lead_status: LeadStatus::from_str(&row.lead_status).ok(),
            handle_at: Some(row.handle_at),
            created_by: Some(row.created_by),
            ..Default::default()
        })
    }

    pub async fn get_customer_uuid(db: &Database, lead_uuid: Uuid) -> Result<Option<Uuid>> {
        let customer = sqlx::query!(
            "SELECT
                c.uuid
            FROM
                customers c
                JOIN customer_leads l ON c.id = l.customer_id
            WHERE 
                l.uuid = $1",
            lead_uuid
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(customer.uuid)
    }

    pub async fn change_handler(
        db: &Database,
        user_full_name: String,
        lead_uuids: Vec<Uuid>,
    ) -> Result<()> {
        let user = sqlx::query!(
            "SELECT user_id as id FROM user_info WHERE full_name = $1",
            user_full_name
        )
        .fetch_one(&db.pool)
        .await?;

        sqlx::query!(
            "UPDATE customer_leads
             SET user_id = $2
             WHERE uuid = ANY($1)",
            &lead_uuids,
            user.id
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(db: &Database, lead_uuids: Vec<Uuid>) -> Result<()> {
        sqlx::query!(
            "DELETE FROM customer_leads
             WHERE uuid = ANY($1)",
            &lead_uuids
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }
}
