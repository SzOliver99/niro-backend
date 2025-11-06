use crate::utils::encrypt::HmacSecret;
use crate::{database::Database, utils::encrypt};
use anyhow::{Ok, Result, anyhow};
use chacha20poly1305::Key;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Recruitment {
    pub uuid: Option<Uuid>,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<String>,
}

impl Recruitment {
    async fn is_exists(db: &Database, hmac_secret: &HmacSecret, r: &Recruitment) -> Result<bool> {
        let full_name = r.full_name.as_deref().unwrap_or("");
        let email = r.email.as_deref().unwrap_or("");
        let phone = r.phone_number.as_deref().unwrap_or("");
        let is_exists = sqlx::query!(
            "SELECT uuid FROM recruitment WHERE full_name = $1 OR email_hash = $2 OR phone_number_hash = $3",
            full_name,
            encrypt::hash_value(hmac_secret, email),
            encrypt::hash_value(hmac_secret, phone)
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }

    pub async fn create(
        db: &Database,
        key: &Key,
        hmac_secret: &HmacSecret,
        recruitment: Recruitment,
    ) -> Result<Uuid> {
        if Recruitment::is_exists(db, hmac_secret, &recruitment).await? {
            return Err(anyhow!("A jelölt már szerepel!"));
        }

        let email = recruitment
            .email
            .as_deref()
            .ok_or_else(|| anyhow!("Email megadása kötelező!"))?;
        let email_hash = encrypt::hash_value(hmac_secret, email);
        let (email_enc, email_nonce) = encrypt::encrypt_value(key, email);

        let phone = recruitment
            .phone_number
            .as_deref()
            .ok_or_else(|| anyhow!("Telefonszám megadása kötelező!"))?;
        let phone_hash = encrypt::hash_value(hmac_secret, phone);
        let (phone_enc, phone_nonce) = encrypt::encrypt_value(key, phone);

        let row = sqlx::query!(
            "INSERT INTO recruitment(full_name, email_enc, email_nonce, email_hash, phone_number_enc, phone_number_nonce, phone_number_hash, description, created_by)
             VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING uuid",
            recruitment.full_name,
            email_enc,
            email_nonce,
            email_hash,
            phone_enc,
            phone_nonce,
            phone_hash,
            recruitment.description,
            recruitment.created_by
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(row.uuid.unwrap())
    }

    pub async fn modify(
        db: &Database,
        key: &Key,
        hmac_secret: &HmacSecret,
        recruitment_uuid: Uuid,
        updated: Recruitment,
    ) -> Result<()> {
        let existing = Recruitment::get_by_uuid(db, key, recruitment_uuid).await?;

        let full_name = updated.full_name.or(existing.full_name);
        let created_by = updated.created_by.or(existing.created_by);
        let effective_email = updated.email.or(existing.email).unwrap_or_default();
        let effective_phone = updated
            .phone_number
            .or(existing.phone_number)
            .unwrap_or_default();

        let (email_enc, email_nonce) = encrypt::encrypt_value(key, &effective_email);
        let email_hash_opt = if effective_email.is_empty() {
            None
        } else {
            Some(encrypt::hash_value(hmac_secret, &effective_email))
        };
        let (phone_enc, phone_nonce) = encrypt::encrypt_value(key, &effective_phone);
        let phone_hash_opt = if effective_phone.is_empty() {
            None
        } else {
            Some(encrypt::hash_value(hmac_secret, &effective_phone))
        };

        sqlx::query!(
            "UPDATE recruitment
             SET full_name = $1,
                 email_enc = $2,
                 email_nonce = $3,
                 email_hash = $4,
                 phone_number_enc = $5,
                 phone_number_nonce = $6,
                 phone_number_hash = $7,
                 created_by = $8
             WHERE uuid = $9",
            full_name,
            email_enc,
            email_nonce,
            email_hash_opt,
            phone_enc,
            phone_nonce,
            phone_hash_opt,
            created_by,
            recruitment_uuid
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get_all(db: &Database, key: &Key) -> Result<Vec<Recruitment>> {
        let rows = sqlx::query!(
            "SELECT uuid, full_name, email_enc, email_nonce, phone_number_enc, phone_number_nonce, description, created_by
             FROM recruitment
             ORDER BY full_name ASC"
        )
        .fetch_all(&db.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Recruitment {
                uuid: row.uuid,
                full_name: Some(row.full_name),
                email: encrypt::decrypt_value(key, &row.email_enc, &row.email_nonce),
                phone_number: encrypt::decrypt_value(
                    key,
                    &row.phone_number_enc,
                    &row.phone_number_nonce,
                ),
                description: Some(row.description),
                created_by: Some(row.created_by),
            })
            .collect())
    }

    pub async fn get_by_uuid(
        db: &Database,
        key: &Key,
        recruitment_uuid: Uuid,
    ) -> Result<Recruitment> {
        let row = sqlx::query!(
            "SELECT uuid, full_name, email_enc, email_nonce, phone_number_enc, phone_number_nonce, description, created_by
             FROM recruitment
             WHERE uuid = $1",
            recruitment_uuid
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(Recruitment {
            uuid: row.uuid,
            full_name: Some(row.full_name),
            email: encrypt::decrypt_value(key, &row.email_enc, &row.email_nonce),
            phone_number: encrypt::decrypt_value(
                key,
                &row.phone_number_enc,
                &row.phone_number_nonce,
            ),
            description: Some(row.description),
            created_by: Some(row.created_by),
        })
    }

    pub async fn delete(db: &Database, recruitment_uuid: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM recruitment WHERE uuid = $1", &recruitment_uuid)
            .execute(&db.pool)
            .await?;
        Ok(())
    }
}
