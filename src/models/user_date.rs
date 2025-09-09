use anyhow::{Ok, Result};
use chacha20poly1305::Key;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::{
    database::Database,
    models::user::User,
    utils::encrypt::{self, HmacSecret},
};

#[skip_serializing_none]
#[derive(Debug, Serialize, Default, Clone)]
pub struct UserMeetDate {
    pub id: Option<i32>,
    pub uuid: Option<Uuid>,
    pub meet_date: Option<NaiveDateTime>,
    pub full_name: Option<String>,
    pub phone_number: Option<String>,
    pub meet_location: Option<String>,
    pub meet_type: Option<String>,
    pub is_completed: Option<bool>,
    pub created_by: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub user_id: Option<i32>,
}

impl UserMeetDate {
    pub async fn create(
        db: &Database,
        key: &Key,
        hmac_secret: &HmacSecret,
        user_uuid: Uuid,
        new_meet_date: UserMeetDate,
    ) -> Result<i32> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid)).await?.unwrap();

        let phone = new_meet_date.phone_number.as_deref().unwrap();
        let phone_hash = encrypt::hash_value(&hmac_secret, phone);
        let (phone_enc, phone_nonce) = encrypt::encrypt_value(&key, phone);

        let row = sqlx::query!(
            "INSERT INTO user_dates(meet_date, full_name, phone_number_enc, phone_number_nonce, phone_number_hash, meet_location, meet_type, created_by, user_id)
             VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id",
            new_meet_date.meet_date,
            new_meet_date.full_name,
            phone_enc,
            phone_nonce,
            phone_hash,
            new_meet_date.meet_location,
            new_meet_date.meet_type,
            new_meet_date.created_by,
            user_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(row.id)
    }

    pub async fn get_all(
        db: &Database,
        key: &Key,
        user_uuid: Uuid,
        selected_month: String,
    ) -> Result<Vec<UserMeetDate>> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid)).await?.unwrap();
        let rows = sqlx::query!(
            "SELECT uuid, meet_date, full_name, phone_number_enc, phone_number_nonce, phone_number_hash, meet_location, meet_type, is_completed, created_by, created_at
             FROM user_dates
             WHERE user_id = $1 AND TRIM(TO_CHAR(meet_date, 'Month')) = $2
             ORDER BY meet_date DESC",
            user_id,
            selected_month
        )
        .fetch_all(&db.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| UserMeetDate {
                uuid: row.uuid,
                meet_date: Some(row.meet_date),
                full_name: Some(row.full_name),
                phone_number: encrypt::decrypt_value(
                    key,
                    &row.phone_number_enc,
                    &row.phone_number_nonce,
                ),
                meet_location: Some(row.meet_location),
                meet_type: Some(row.meet_type),
                is_completed: Some(row.is_completed),
                created_by: Some(row.created_by),
                created_at: Some(row.created_at),
                ..Default::default()
            })
            .collect())
    }

    pub async fn change_handler(
        db: &Database,
        user_full_name: String,
        date_uuids: Vec<Uuid>,
    ) -> Result<()> {
        let user = sqlx::query!(
            "SELECT user_id as id FROM user_info WHERE full_name = $1",
            user_full_name
        )
        .fetch_one(&db.pool)
        .await?;

        sqlx::query!(
            "UPDATE user_dates
             SET user_id = $2
             WHERE uuid = ANY($1)",
            &date_uuids,
            user.id
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn change_date_state(
        db: &Database,
        date_uuid: Uuid,
        is_completed: bool,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE user_dates
             SET is_completed = $2
             WHERE uuid = $1",
            &date_uuid,
            is_completed
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(db: &Database, date_uuids: Vec<Uuid>) -> Result<()> {
        sqlx::query!("DELETE FROM user_dates WHERE uuid = ANY($1)", &date_uuids)
            .execute(&db.pool)
            .await?;
        Ok(())
    }
}
