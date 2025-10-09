use crate::utils::encrypt::HmacSecret;
use crate::{database::Database, utils::encrypt};
use anyhow::{Ok, Result, anyhow};
use chacha20poly1305::Key;
use uuid::Uuid;

use crate::models::user::User;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CustomerRecommendation {
    pub uuid: Option<Uuid>,
    pub full_name: Option<String>,
    pub phone_number: Option<String>,
    pub city: Option<String>,
    pub referral_name: Option<String>,
    pub created_by: Option<String>,
}

impl CustomerRecommendation {
    async fn is_exists(
        db: &Database,
        hmac_secret: &HmacSecret,
        customer_recommendation: &CustomerRecommendation,
    ) -> Result<bool> {
        let full_name = customer_recommendation.full_name.as_deref().unwrap_or("");
        let phone = customer_recommendation
            .phone_number
            .as_deref()
            .unwrap_or("");
        let is_exists = sqlx::query!(
            "SELECT uuid
             FROM customer_recommendations
             WHERE full_name = $1 OR phone_number_hash = $2",
            full_name,
            encrypt::hash_value(hmac_secret, phone)
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }

    async fn is_exists_by_uuid(db: &Database, customer_recommendation_uuid: Uuid) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT uuid
             FROM customer_recommendations
             WHERE uuid = $1",
            customer_recommendation_uuid
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }

    pub async fn create(
        db: &Database,
        key: &Key,
        hmac_secret: &HmacSecret,
        user_uuid: Uuid,
        customer_recommendation: CustomerRecommendation,
    ) -> Result<Uuid> {
        if CustomerRecommendation::is_exists(db, hmac_secret, &customer_recommendation).await? {
            return Err(anyhow!("Az ügyfél már szerepel az ajánlásban!"));
        }

        let phone = customer_recommendation
            .phone_number
            .as_deref()
            .ok_or_else(|| anyhow!("Telefonszám megadása kötelező!"))?;
        let phone_hash = encrypt::hash_value(hmac_secret, phone);
        let (phone_number_enc, phone_number_nonce) = encrypt::encrypt_value(key, phone);

        let city = customer_recommendation
            .city
            .as_deref()
            .ok_or_else(|| anyhow!("Település megadása kötelező!"))?;
        let (city_enc, city_nonce) = encrypt::encrypt_value(key, city);

        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let row = sqlx::query!(
            "INSERT INTO customer_recommendations(full_name, phone_number_enc, phone_number_nonce, phone_number_hash, city_enc, city_nonce, referral_name, user_id, created_by)
             VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING uuid",
            customer_recommendation.full_name,
            phone_number_enc,
            phone_number_nonce,
            phone_hash,
            city_enc,
            city_nonce,
            customer_recommendation.referral_name,
            user_id,
            customer_recommendation.created_by
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(row.uuid.unwrap())
    }

    pub async fn modify(
        db: &Database,
        key: &Key,
        hmac_secret: &HmacSecret,
        recommendation_uuid: Uuid,
        updated: CustomerRecommendation,
    ) -> Result<()> {
        // Load existing to avoid wiping unspecified fields
        let existing = CustomerRecommendation::get_by_uuid(db, key, recommendation_uuid).await?;

        let full_name = updated.full_name.or(existing.full_name);
        let effective_phone = updated
            .phone_number
            .or(existing.phone_number)
            .unwrap_or_default();
        let effective_city = updated.city.or(existing.city).unwrap_or_default();
        let referral_name = updated.referral_name.or(existing.referral_name);
        let created_by = updated.created_by.or(existing.created_by);

        let (phone_enc, phone_nonce) = encrypt::encrypt_value(key, &effective_phone);
        let phone_hash_opt = if effective_phone.is_empty() {
            None
        } else {
            Some(encrypt::hash_value(hmac_secret, &effective_phone))
        };
        let (city_enc, city_nonce) = encrypt::encrypt_value(key, &effective_city);

        sqlx::query!(
            "UPDATE customer_recommendations
             SET full_name = $1,
                 phone_number_enc = $2,
                 phone_number_nonce = $3,
                 phone_number_hash = $4,
                 city_enc = $5,
                 city_nonce = $6,
                 referral_name = $7,
                 created_by = $8
             WHERE uuid = $9",
            full_name,
            phone_enc,
            phone_nonce,
            phone_hash_opt,
            city_enc,
            city_nonce,
            referral_name,
            created_by,
            recommendation_uuid
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get_all(
        db: &Database,
        key: &Key,
        user_uuid: Uuid,
    ) -> Result<Vec<CustomerRecommendation>> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let rows = sqlx::query!(
            "SELECT uuid, full_name, phone_number_enc, phone_number_nonce, city_enc, city_nonce, referral_name, created_by
             FROM customer_recommendations
             WHERE user_id = $1
             ORDER BY full_name ASC",
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| CustomerRecommendation {
                uuid: row.uuid,
                full_name: Some(row.full_name),
                phone_number: encrypt::decrypt_value(
                    key,
                    &row.phone_number_enc,
                    &row.phone_number_nonce,
                ),
                city: encrypt::decrypt_value(key, &row.city_enc, &row.city_nonce),
                referral_name: Some(row.referral_name),
                created_by: Some(row.created_by),
            })
            .collect())
    }

    pub async fn get_by_uuid(
        db: &Database,
        key: &Key,
        recommendation_uuid: Uuid,
    ) -> Result<CustomerRecommendation> {
        let row = sqlx::query!(
            "SELECT uuid, full_name, phone_number_enc, phone_number_nonce, city_enc, city_nonce, referral_name, created_by
             FROM customer_recommendations
             WHERE uuid = $1",
            recommendation_uuid
        )
            .fetch_one(&db.pool)
            .await?;

        Ok(CustomerRecommendation {
            uuid: row.uuid,
            full_name: Some(row.full_name),
            phone_number: encrypt::decrypt_value(
                key,
                &row.phone_number_enc,
                &row.phone_number_nonce,
            ),
            city: encrypt::decrypt_value(key, &row.city_enc, &row.city_nonce),
            referral_name: Some(row.referral_name),
            created_by: Some(row.created_by),
        })
    }

    pub async fn change_handler(
        db: &Database,
        user_full_name: String,
        recommendation_uuids: Vec<Uuid>,
    ) -> Result<()> {
        let user = sqlx::query!(
            "SELECT user_id as id FROM user_info WHERE full_name = $1",
            user_full_name
        )
        .fetch_one(&db.pool)
        .await?;

        sqlx::query!(
            "UPDATE customer_recommendations
             SET user_id = $2
             WHERE uuid = ANY($1)",
            &recommendation_uuids,
            user.id
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(db: &Database, recommendation_uuids: Vec<Uuid>) -> Result<()> {
        sqlx::query!(
            "DELETE FROM customer_recommendations WHERE uuid = ANY($1)",
            &recommendation_uuids
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }
}
