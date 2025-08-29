use anyhow::{Ok, Result};
use chrono::NaiveDateTime;
use serde::Serialize;
use serde_with::skip_serializing_none;
use sqlx::prelude::Type;

use crate::{database::Database, models::customer::Customer};

#[skip_serializing_none]
#[derive(Debug, Serialize, Default)]
pub struct Lead {
    pub id: Option<i32>,
    pub lead_type: Option<String>,
    pub inquiry_type: Option<String>,
    pub lead_status: Option<LeadStatus>,
    pub handle_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Type)]
pub enum LeadStatus {
    Opened,
    InProgress,
    Closed,
}

impl std::fmt::Display for LeadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LeadStatus::Opened => "Opened",
            LeadStatus::InProgress => "InProgress",
            LeadStatus::Closed => "Closed",
        };
        write!(f, "{}", s)
    }
}

impl From<String> for LeadStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "InProgress" => LeadStatus::InProgress,
            "Closed" => LeadStatus::Closed,
            _ => LeadStatus::Opened,
        }
    }
}

impl Lead {
    async fn is_customer_exists(db: &Database, lead: &Lead) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM customer_leads
             WHERE inquiry_type = $1",
            lead.inquiry_type,
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }

    async fn is_lead_exists_by_id(db: &Database, lead_id: i32) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM customer_leads
             WHERE id = $1",
            lead_id
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }
}

impl Lead {
    pub async fn create(db: &Database, customer_id: i32, lead: Lead) -> Result<()> {
        if !Customer::is_exists_by_id(db, customer_id).await? {
            return Err(anyhow::anyhow!("Az ügyfél nincs az adatbázisban."));
        }

        let _row = sqlx::query!(
            "INSERT INTO customer_leads(lead_type, inquiry_type, lead_status, handle_at, customer_id)
             VALUES($2, $3, $4, $5, $1)
             RETURNING id",
            customer_id,
            lead.lead_type,
            lead.inquiry_type,
            lead.lead_status.map(|s| s.to_string()),
            lead.handle_at
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(())
    }
}
