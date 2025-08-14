use serde::Serialize;

use super::{contact::Contact, user::UserRole, user_info::UserInfo};

#[derive(Serialize)]
pub struct UserSummaryDto {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub role: UserRole,
}

#[derive(Serialize)]
pub struct UserWithInfoDto {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub role: UserRole,
    pub info: UserInfoDto,
    pub manager_id: Option<i32>,
}

#[derive(Serialize)]
pub struct UserInfoDto {
    pub id: i32,
    pub full_name: String,
    pub phone_number: String,
    pub hufa_code: String,
    pub agent_code: String,
}

impl From<UserInfo> for UserInfoDto {
    fn from(value: UserInfo) -> Self {
        Self {
            id: value.id.unwrap_or_default(),
            full_name: value.full_name.unwrap_or_default(),
            phone_number: value.phone_number.unwrap_or_default(),
            hufa_code: value.hufa_code.unwrap_or_default(),
            agent_code: value.agent_code.unwrap_or_default(),
        }
    }
}

#[derive(Serialize)]
pub struct ContactDto {
    pub id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
}

impl From<Contact> for ContactDto {
    fn from(value: Contact) -> Self {
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
