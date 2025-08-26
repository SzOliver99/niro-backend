use serde::Serialize;

use super::{lead::Lead, user::UserRole};

#[derive(Serialize)]
pub struct UserSummaryDto {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub role: UserRole,
}

#[derive(Serialize)]
pub struct LeadDto {
    pub id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
}

impl From<Lead> for LeadDto {
    fn from(value: Lead) -> Self {
        Self {
            id: value.id.unwrap_or_default(),
            email: value.email.unwrap_or_default(),
            first_name: value.first_name.unwrap_or_default(),
            last_name: value.last_name.unwrap_or_default(),
            phone_number: value.phone_number.unwrap_or_default(),
        }
    }
}

#[derive(Serialize)]
pub struct ManagerNameDto {
    pub id: i32,
    pub full_name: String,
    pub user_role: String,
}
