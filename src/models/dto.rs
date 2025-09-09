use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::models::contract::{ContractType, PaymentFrequency, PaymentMethod};

#[derive(Serialize)]
pub struct ManagerNameDto {
    pub uuid: Option<Uuid>,
    pub full_name: String,
    pub user_role: String,
}

#[derive(Serialize)]
pub struct LeadListItemDto {
    pub uuid: Option<Uuid>,
    pub full_name: String,
    pub phone_number: String,
    pub email: String,
    pub address: String,
    pub lead_type: String,
    pub inquiry_type: String,
    pub lead_status: String,
    pub handle_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Serialize)]
pub struct ContractDto {
    pub uuid: Option<Uuid>,
    pub full_name: String,
    pub phone_number: String,
    pub email: String,
    pub address: String,
    pub contract_number: String,
    pub contract_type: ContractType,
    pub annual_fee: i32,
    pub payment_frequency: PaymentFrequency,
    pub payment_method: PaymentMethod,
    pub created_by: String,
    pub handle_at: DateTime<Utc>,
}
