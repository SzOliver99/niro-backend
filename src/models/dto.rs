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
    pub month: i16,
    pub week1: i64,
    pub week2: i64,
    pub week3: i64,
    pub week4: i64,
    pub week5: i64,
}

// CONTRACTS CHART
#[derive(Serialize)]
pub struct PortfolioDto {
    pub bonus_life_program: i64,
    pub life_program: i64,
    pub allianz_care_now: i64,
    pub health_program: i64,
    pub myhome_home_insurance: i64,
    pub mfo_home_insurance: i64,
    pub corporate_property_insurance: i64,
    pub kgfb: i64,
    pub casco: i64,
    pub travel_insurance: i64,
    pub condominium_insurance: i64,
    pub agricultural_insurance: i64,
}

#[derive(Serialize)]
pub struct WeeklyProductionChartDto {
    pub monday: i64,
    pub tuesday: i64,
    pub wednesday: i64,
    pub thursday: i64,
    pub friday: i64,
    pub saturday: i64,
    pub sunday: i64,
}

#[derive(Serialize)]
pub struct MonthlyProductionChartDto {
    pub month: i16,
    pub week1: i64,
    pub week2: i64,
    pub week3: i64,
    pub week4: i64,
    pub week5: i64,
}