use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct ManagerNameDto {
    pub uuid: Option<Uuid>,
    pub full_name: String,
    pub user_role: String,
}

#[derive(Serialize)]
pub struct LeadListItemDto {
    pub uuid: Option<Uuid>,
    pub name: String,
    pub phone: String,
    pub email: String,
    pub address: String,
    pub lead_type: String,
    pub inquiry_type: String,
    pub lead_status: String,
    pub handle_at: DateTime<Utc>,
    pub created_by: String,
}
