use anyhow::{Ok, Result};
use chacha20poly1305::Key;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{database::Database, utils::encrypt};

#[skip_serializing_none]
#[derive(Debug, Serialize, Default, Clone)]
pub struct UserMeetDate {
    pub id: Option<i32>,
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
        hmac_secret: &Vec<u8>,
        new_meet_date: UserMeetDate,
    ) -> Result<i32> {
        let (phone_hash, phone_enc, phone_nonce) = {
            let phone = new_meet_date.phone_number.as_deref().unwrap();
            let phone_hash = encrypt::hash_value(&hmac_secret, phone);

            let (phone_enc, phone_nonce) = encrypt::encrypt_value(&key, phone);

            (phone_hash, phone_enc, phone_nonce)
        };

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
            new_meet_date.user_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(row.id)
    }

    pub async fn get_all(db: &Database, key: &Key, user_id: i32) -> Result<Vec<UserMeetDate>> {
        let rows = sqlx::query!(
            "SELECT id, meet_date, full_name, phone_number_enc, phone_number_nonce, phone_number_hash, meet_location, meet_type,is_completed, created_by, created_at, user_id
             FROM user_dates
             WHERE user_id = $1
             ORDER BY meet_date DESC
            ",
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| UserMeetDate {
                id: Some(row.id),
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
                user_id: row.user_id,
            })
            .collect())
    }

    pub async fn change_handler(
        db: &Database,
        user_full_name: String,
        date_ids: Vec<i32>,
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
             WHERE id = ANY($1)",
            &date_ids,
            user.id
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn change_date_state(db: &Database, date_id: i32, is_completed: bool) -> Result<()> {
        sqlx::query!(
            "UPDATE user_dates
             SET is_completed = $2
             WHERE id = $1",
            &date_id,
            is_completed
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(db: &Database, date_ids: Vec<i32>) -> Result<()> {
        sqlx::query!("DELETE FROM user_dates WHERE id = ANY($1)", &date_ids)
            .execute(&db.pool)
            .await?;
        Ok(())
    }
}
