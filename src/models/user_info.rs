use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
pub struct UserInfo {
    pub id: Option<i32>,
    pub full_name: Option<String>,
    pub phone_number: Option<String>,
    pub hufa_code: Option<String>,
    pub agent_code: Option<String>,
}
