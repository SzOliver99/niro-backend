use anyhow::{Result, anyhow};
use chacha20poly1305::Key;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::Type;
use strum::{AsRefStr, Display, EnumString};
use uuid::Uuid;

use crate::models::dto::InterventionTaskDto;
use crate::{
    database::Database,
    models::{customer::Customer, user::User},
    utils::encrypt::{self, HmacSecret},
};

#[skip_serializing_none]
#[derive(Debug, Serialize, Default, Clone)]
pub struct InterventionTask {
    pub id: Option<i32>,
    pub uuid: Option<Uuid>,
    pub contract_number: Option<String>,
    pub product_name: Option<String>,
    pub outstanding_days: Option<i32>,
    pub balance: Option<i32>,
    pub processing_deadline: Option<NaiveDateTime>,
    pub comment: Option<String>,
    pub status: Option<InterventionTaskStatus>,
    pub customer_id: Option<i32>,
    pub user_id: Option<i32>,
    pub created_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumString, Display, Type, AsRefStr)]
pub enum InterventionTaskStatus {
    Pending,
    PaymentPromise,
    Processed,
    Nonpayment,
    PendingDeletion,
}

impl InterventionTask {}

impl InterventionTask {
    pub async fn create(
        db: &Database,
        key: &Key,
        hmac_secret: &HmacSecret,
        user_uuid: Uuid,
        customer: Customer,
        intervention_task: InterventionTask,
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

        let intervention_task_row = sqlx::query!(
            "INSERT INTO customer_intervention_tasks(contract_number, product_name, outstanding_days, balance, processing_deadline, comment, status, customer_id, user_id, created_by)
             VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING id",
            intervention_task.contract_number,
            intervention_task.product_name,
            intervention_task.outstanding_days,
            intervention_task.balance,
            intervention_task.processing_deadline,
            intervention_task.comment,
            intervention_task.status.map(|s| s.to_string()),
            customer_id,
            user_id,
            intervention_task.created_by
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(intervention_task_row.id)
    }

    pub async fn modify(
        db: &Database,
        intervention_task_uuid: Uuid,
        updated_intervention_task: InterventionTask,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE customer_intervention_tasks
             SET contract_number = $1,
                 product_name = $2,
                 outstanding_days = $3,
                 balance = $4,
                 processing_deadline = $5,
                 comment = $6,
                 status = $7
             WHERE uuid = $8",
            updated_intervention_task.contract_number,
            updated_intervention_task.product_name,
            updated_intervention_task.outstanding_days,
            updated_intervention_task.balance,
            updated_intervention_task.processing_deadline,
            updated_intervention_task.comment,
            updated_intervention_task.status.map(|s| s.to_string()),
            intervention_task_uuid
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get_all(
        db: &Database,
        key: &Key,
        user_uuid: Uuid,
    ) -> Result<Vec<InterventionTaskDto>> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;
        let rows = sqlx::query!(
            "SELECT c.full_name, c.phone_number_enc, c.phone_number_nonce, c.email_enc, c.email_nonce, c.address_enc, c.address_nonce, it.uuid, it.contract_number, it.product_name, it.outstanding_days, it.balance, it.processing_deadline, it.comment, it.status, it.created_by
             FROM customers c
             JOIN customer_intervention_tasks it ON it.customer_id = c.id
             WHERE it.user_id = $1",
            user_id
        )
            .fetch_all(&db.pool)
            .await?;

        let items: Vec<InterventionTaskDto> = rows
            .into_iter()
            .map(|row| InterventionTaskDto {
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
                uuid: row.uuid,
                contract_number: row.contract_number,
                product_name: row.product_name,
                outstanding_days: row.outstanding_days,
                balance: row.balance,
                processing_deadline: row.processing_deadline,
                comment: row.comment,
                status: row.status.parse().unwrap(),
                created_by: row.created_by,
            })
            .collect();

        Ok(items)
    }

    pub async fn get_by_customer_uuid(
        db: &Database,
        customer_uuid: Uuid,
    ) -> Result<Vec<InterventionTask>> {
        let customer_id = Customer::get_id_by_uuid(db, Some(customer_uuid))
            .await?
            .unwrap();

        let rows = sqlx::query!(
            "SELECT
                uuid,
                contract_number,
                product_name,
                outstanding_days,
                balance,
                processing_deadline,
                comment,
                status,
                created_by
            FROM
                customer_intervention_tasks
            WHERE
	            customer_id = $1",
            customer_id
        )
        .fetch_all(&db.pool)
        .await?;

        let items: Vec<InterventionTask> = rows
            .into_iter()
            .map(|row| InterventionTask {
                uuid: row.uuid,
                contract_number: Some(row.contract_number),
                product_name: Some(row.product_name),
                outstanding_days: Some(row.outstanding_days),
                balance: Some(row.balance),
                processing_deadline: Some(row.processing_deadline),
                comment: row.comment,
                status: Some(row.status.parse().unwrap()),
                created_by: Some(row.created_by),
                ..Default::default()
            })
            .collect();

        Ok(items)
    }

    pub async fn get_by_uuid(
        db: &Database,
        intervention_task_uuid: Uuid,
    ) -> Result<InterventionTask> {
        let row = sqlx::query!(
            "SELECT
                uuid,
                contract_number,
                product_name,
                outstanding_days,
                balance,
                processing_deadline,
                comment,
                status,
                created_by
            FROM
                customer_intervention_tasks
            WHERE
	            uuid = $1",
            intervention_task_uuid
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(InterventionTask {
            uuid: row.uuid,
            contract_number: Some(row.contract_number),
            product_name: Some(row.product_name),
            outstanding_days: Some(row.outstanding_days),
            balance: Some(row.balance),
            processing_deadline: Some(row.processing_deadline),
            comment: row.comment,
            status: Some(row.status.parse()?),
            created_by: Some(row.created_by),
            ..Default::default()
        })
    }

    pub async fn get_customer_uuid(
        db: &Database,
        intervention_task_uuid: Uuid,
    ) -> Result<Option<Uuid>> {
        let customer = sqlx::query!(
            "SELECT
                c.uuid
            FROM
                customers c
                JOIN customer_intervention_tasks it ON c.id = it.customer_id
            WHERE
                it.uuid = $1",
            intervention_task_uuid
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(customer.uuid)
    }

    pub async fn change_handler(
        db: &Database,
        user_full_name: String,
        intervention_task_uuids: Vec<Uuid>,
    ) -> Result<()> {
        let user = sqlx::query!(
            "SELECT user_id as id FROM user_info WHERE full_name = $1",
            user_full_name
        )
        .fetch_one(&db.pool)
        .await?;

        sqlx::query!(
            "UPDATE customer_intervention_tasks
             SET user_id = $2
             WHERE uuid = ANY($1)",
            &intervention_task_uuids,
            user.id
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(db: &Database, intervention_task_uuids: Vec<Uuid>) -> Result<()> {
        sqlx::query!(
            "DELETE FROM customer_intervention_tasks
             WHERE uuid = ANY($1)",
            &intervention_task_uuids
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }
}
