use serde::Serialize;

use super::{customer::Customer, user::UserRole};

#[derive(Serialize)]
pub struct UserSummaryDto {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub role: UserRole,
}

#[derive(Serialize)]
pub struct ManagerNameDto {
    pub id: i32,
    pub full_name: String,
    pub user_role: String,
}
