use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::models::contract::{ContractType, PaymentFrequency, PaymentMethod};
use crate::models::intervention_task::InterventionTaskStatus;

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
pub struct InterventionTaskDto {
    pub uuid: Option<Uuid>,
    pub full_name: String,
    pub phone_number: String,
    pub email: String,
    pub address: String,
    pub contract_number: String,
    pub product_name: String,
    pub outstanding_days: i32,
    pub balance: i32,
    pub processing_deadline: NaiveDateTime,
    pub comment: Option<String>,
    pub status: InterventionTaskStatus,
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
    pub first_payment: bool,
    pub payment_frequency: PaymentFrequency,
    pub payment_method: PaymentMethod,
    pub created_by: String,
    pub handle_at: DateTime<Utc>,
}

// USER DATE CHART
#[derive(Serialize)]
pub struct IsCompletedChartDto {
    pub yes: i64,
    pub no: i64,
}

#[derive(Serialize)]
pub struct MeetTypeChartDto {
    pub needs_assessment: i64,
    pub consultation: i64,
    pub service: i64,
    pub annual_review: i64,
}

#[derive(Serialize)]
pub struct DatesWeeklyChartDto {
    pub monday: i64,
    pub tuesday: i64,
    pub wednesday: i64,
    pub thursday: i64,
    pub friday: i64,
    pub saturday: i64,
    pub sunday: i64,
}

#[derive(Serialize)]
pub struct DatesMonthlyChartDto {
    pub january: i64,
    pub february: i64,
    pub march: i64,
    pub april: i64,
    pub may: i64,
    pub june: i64,
    pub july: i64,
    pub august: i64,
    pub september: i64,
    pub october: i64,
    pub november: i64,
    pub december: i64,
}
