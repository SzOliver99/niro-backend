use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, FromRow, Default, Clone)]
pub struct UserInfo {
    pub id: Option<i32>,
    pub full_name: Option<String>,
    pub phone_number: Option<String>,
    pub hufa_code: Option<String>,
    pub agent_code: Option<String>,
}
