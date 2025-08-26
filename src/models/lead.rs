use anyhow::{Ok, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;

use crate::database::Database;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Lead {
    pub id: Option<i32>,
    pub lead_type: Option<String>,
    pub inquiry_type: Option<String>,
    pub lead_status: Option<LeadStatus>,
    pub handle_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
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
    pub async fn create(db: &Database, lead: Lead) -> Result<()> {
        Ok(())
    }
}
