use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;

use super::user::UserRole;

#[derive(Serialize)]
pub struct ManagerNameDto {
    pub id: i32,
    pub full_name: String,
    pub user_role: String,
}

#[derive(Serialize)]
pub struct LeadListItemDto {
    pub id: i32,
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
