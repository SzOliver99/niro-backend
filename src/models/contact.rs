use anyhow::{Ok, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::database::Database;

#[derive(Debug, Serialize, Deserialize)]
pub struct Contact {
    pub id: Option<i32>,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone_number: Option<String>,
    pub user_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactHistory {
    pub id: Option<i32>,
    pub p_type: String,
    pub time: NaiveDateTime,
}

impl Contact {
    pub async fn new(db: &Database, new_contact: Contact) -> Result<()> {
        if Self::is_contact_exists(db, &new_contact).await? {
            return Err(anyhow::anyhow!("contact already in the database"));
        }

        let _contact_id = sqlx::query!(
            "INSERT INTO contacts(email, first_name, last_name, phone_number, user_id)
             VALUES($1, $2, $3, $4, $5)
             RETURNING id",
            new_contact.email,
            new_contact.first_name,
            new_contact.last_name,
            new_contact.phone_number,
            new_contact.user_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get(db: &Database, user_id: i32) -> Result<Self> {
        let contact = sqlx::query_as!(
            Self,
            "SELECT * FROM contacts
            WHERE id = $1",
            user_id
        )
        .fetch_one(&db.pool)
        .await?;
        Ok(contact)
    }

    pub async fn create_history(
        db: &Database,
        contact_id: i32,
        history: ContactHistory,
    ) -> Result<()> {
        if !Self::is_contact_exists_by_id(db, contact_id).await? {
            return Err(anyhow::anyhow!("contact is not in the database!"));
        }

        let _history_id = sqlx::query!(
            "INSERT INTO contact_history(p_type, time, contact_id)
             VALUES($1, $2, $3)
             RETURNING id",
            history.p_type,
            history.time,
            contact_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get_history(db: &Database, contact_id: i32) -> Result<Vec<ContactHistory>> {
        if !Self::is_contact_exists_by_id(db, contact_id).await? {
            return Err(anyhow::anyhow!("contact is not in the database!"));
        }

        let contact_history = sqlx::query_as!(
            ContactHistory,
            "SELECT id, p_type, time FROM contact_history
             WHERE contact_id = $1",
            contact_id
        )
        .fetch_all(&db.pool)
        .await?;

        Ok(contact_history)
    }
}

impl Contact {
    async fn is_contact_exists(db: &Database, contact: &Contact) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM contacts 
             WHERE email = $1 OR phone_number = $2",
            contact.email,
            contact.phone_number
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }

    async fn is_contact_exists_by_id(db: &Database, contact_id: i32) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM contacts
             WHERE id = $1",
            contact_id
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }
}
