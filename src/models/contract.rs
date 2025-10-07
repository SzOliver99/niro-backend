use anyhow::{Ok, Result, anyhow};
use chacha20poly1305::Key;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::Type;
use strum::{AsRefStr, Display, EnumString};
use uuid::Uuid;

use crate::{
    database::Database,
    models::{
        customer::Customer,
        dto::{ContractDto, MonthlyProductionChartDto, PortfolioDto, WeeklyProductionChartDto},
        user::User,
    },
    utils::encrypt::{self, HmacSecret},
};

#[skip_serializing_none]
#[derive(Debug, Serialize, Default, Clone)]
pub struct Contract {
    pub id: Option<i32>,
    pub uuid: Option<Uuid>,
    pub contract_number: Option<String>,
    pub contract_type: Option<ContractType>,
    pub annual_fee: Option<i32>,
    pub first_payment: Option<bool>,
    pub payment_frequency: Option<PaymentFrequency>,
    pub payment_method: Option<PaymentMethod>,
    pub customer_id: Option<i32>,
    pub user_id: Option<i32>,
    pub created_by: Option<String>,
    pub handle_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumString, Display, Type, AsRefStr)]
pub enum ContractType {
    BonusLifeProgram,
    LifeProgram,
    AllianzCareNow,
    HealthProgram,
    MyhomeHomeInsurance,
    MfoHomeInsurance,
    CorporatePropertyInsurance,
    Kgfb,
    Casco,
    TravelInsurance,
    CondominiumInsurance,
    AgriculturalInsurance,
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumString, Display, Type, AsRefStr)]
pub enum PaymentFrequency {
    Monthly,
    Quarterly,
    Semiannual,
    Annual,
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumString, Display, Type, AsRefStr)]
pub enum PaymentMethod {
    CreditCard,
    Transfer,
    DirectDebit,
    Check,
}

// CONTRACT UTILS //
impl Contract {
    pub async fn get_id_by_uuid(db: &Database, contract_uuid: Option<Uuid>) -> Result<Option<i32>> {
        let user = sqlx::query_scalar!(
            "SELECT id FROM customer_contracts WHERE uuid = $1",
            contract_uuid
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_uuid_by_id(db: &Database, contract_id: i32) -> Result<Option<Uuid>> {
        let user = sqlx::query!(
            "SELECT uuid FROM customer_contracts WHERE id = $1",
            contract_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(user.uuid)
    }

    pub(super) async fn is_exists(db: &Database, contract: &Contract) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM customer_contracts
             WHERE contract_number = $1",
            contract.contract_number
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }

    pub(super) async fn is_exists_by_id(db: &Database, contract_id: i32) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM customer_contracts
             WHERE id = $1",
            contract_id
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }
}

// CONTRACT CALLBACKS //
impl Contract {
    pub async fn create(
        db: &Database,
        key: &Key,
        hmac_secret: &HmacSecret,
        user_uuid: Uuid,
        customer: Customer,
        contract: Contract,
    ) -> Result<i32> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;
        let row = sqlx::query!(
            "SELECT id FROM customers
             WHERE email_hash = $1 OR phone_number_hash = $2",
            encrypt::hash_value(hmac_secret, &customer.email.as_ref().unwrap()),
            encrypt::hash_value(hmac_secret, &customer.phone_number.as_ref().unwrap()),
        )
        .fetch_optional(&db.pool)
        .await?;

        // Determine customer_id: create customer if not exists, otherwise use existing id
        let customer_id = if let Some(existing) = row {
            existing.id
        } else {
            Customer::create(db, key, hmac_secret, user_uuid, customer.clone()).await?
        };

        let row = sqlx::query!(
            "INSERT INTO customer_contracts(contract_number, contract_type, annual_fee, payment_frequency, payment_method, customer_id, user_id, created_by)
             VALUES($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id",
            contract.contract_number,
            contract.contract_type.map(|c| c.to_string()),
            contract.annual_fee,
            contract.payment_frequency.map(|c| c.to_string()),
            contract.payment_method.map(|c| c.to_string()),
            customer_id,
            user_id,
            contract.created_by
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(row.id)
    }

    pub async fn modify(
        db: &Database,
        contract_uuid: Uuid,
        updated_contract: Contract,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE customer_contracts
             SET contract_number = $1,
                 contract_type = $2,
                 annual_fee = $3,
                 payment_frequency = $4,
                 payment_method = $5,
                 handle_at = NOW()
             WHERE uuid = $6",
            updated_contract.contract_number,
            updated_contract.contract_type.map(|c| c.to_string()),
            updated_contract.annual_fee,
            updated_contract.payment_frequency.map(|c| c.to_string()),
            updated_contract.payment_method.map(|c| c.to_string()),
            contract_uuid
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get_all(db: &Database, key: &Key, user_uuid: Uuid) -> Result<Vec<ContractDto>> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let rows = sqlx::query!(
            r#"
            SELECT
                c.full_name,
                c.phone_number_enc,
                c.phone_number_nonce,
                c.email_enc,
                c.email_nonce,
                c.address_enc,
                c.address_nonce,
                cc.uuid,
                cc.contract_number,
                cc.contract_type,
                cc.annual_fee,
                cc.first_payment,
                cc.payment_frequency,
                cc.payment_method,
                cc.handle_at,
                cc.created_by
            FROM
                customers c
                JOIN customer_contracts cc ON cc.customer_id = c.id
            WHERE
                cc.user_id = $1
            ORDER BY cc.handle_at DESC
            "#,
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        let contracts: Vec<ContractDto> = rows
            .into_iter()
            .map(|row| ContractDto {
                uuid: row.uuid,
                full_name: row.full_name,
                phone_number: encrypt::decrypt_value(
                    key,
                    &row.phone_number_enc,
                    &row.phone_number_nonce,
                )
                .unwrap_or_default(),
                email: encrypt::decrypt_value(key, &row.email_enc, &row.email_nonce)
                    .unwrap_or_default(),
                address: encrypt::decrypt_value(key, &row.address_enc, &row.address_nonce)
                    .unwrap_or_default(),
                contract_number: row.contract_number,
                contract_type: row.contract_type.parse().unwrap(),
                annual_fee: row.annual_fee,
                first_payment: row.first_payment,
                payment_frequency: row.payment_frequency.parse().unwrap(),
                payment_method: row.payment_method.parse().unwrap(),
                created_by: row.created_by,
                handle_at: row.handle_at,
            })
            .collect();

        Ok(contracts)
    }

    pub async fn get_by_customer_uuid(db: &Database, customer_uuid: Uuid) -> Result<Vec<Contract>> {
        let customer_id = Customer::get_id_by_uuid(db, Some(customer_uuid))
            .await?
            .unwrap();
        println!("{customer_id}");
        let rows = sqlx::query!(
            "SELECT
                uuid,
                contract_number,
                contract_type,
                annual_fee,
                first_payment,
                payment_frequency,
                payment_method,
                handle_at,
                created_by
            FROM
                customer_contracts
            WHERE
	            customer_id = $1",
            customer_id
        )
        .fetch_all(&db.pool)
        .await?;

        let items: Vec<Contract> = rows
            .into_iter()
            .map(|row| Contract {
                uuid: row.uuid,
                contract_number: Some(row.contract_number),
                contract_type: Some(row.contract_type.parse().unwrap()),
                annual_fee: Some(row.annual_fee),
                first_payment: Some(row.first_payment),
                payment_frequency: Some(row.payment_frequency.parse().unwrap()),
                payment_method: Some(row.payment_method.parse().unwrap()),
                handle_at: Some(row.handle_at),
                created_by: Some(row.created_by),
                ..Default::default()
            })
            .collect();

        Ok(items)
    }

    pub async fn get_by_uuid(db: &Database, contract_uuid: Uuid) -> Result<Contract> {
        let row = sqlx::query!(
            "SELECT
                uuid,
                contract_number,
                contract_type,
                annual_fee,
                first_payment,
                payment_frequency,
                payment_method,
                handle_at,
                created_by
            FROM
                customer_contracts
            WHERE
	            uuid = $1",
            contract_uuid
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(Contract {
            uuid: row.uuid,
            contract_number: Some(row.contract_number),
            contract_type: Some(row.contract_type.parse().unwrap()),
            annual_fee: Some(row.annual_fee),
            first_payment: Some(row.first_payment),
            payment_frequency: Some(row.payment_frequency.parse().unwrap()),
            payment_method: Some(row.payment_method.parse().unwrap()),
            handle_at: Some(row.handle_at),
            created_by: Some(row.created_by),
            ..Default::default()
        })
    }

    pub async fn get_customer_uuid(db: &Database, contract_uuid: Uuid) -> Result<Option<Uuid>> {
        let customer = sqlx::query!(
            "SELECT
                c.uuid
            FROM
                customers c
                JOIN customer_contracts cc ON c.id = cc.customer_id
            WHERE 
                cc.uuid = $1",
            contract_uuid
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(customer.uuid)
    }

    pub async fn change_first_payment_state(
        db: &Database,
        contract_uuid: Uuid,
        value: bool,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE customer_contracts
             SET first_payment = $2
             WHERE uuid = $1",
            &contract_uuid,
            value
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn change_handler(
        db: &Database,
        user_full_name: String,
        contract_uuids: Vec<Uuid>,
    ) -> Result<()> {
        let user = sqlx::query!(
            "SELECT user_id as id FROM user_info WHERE full_name = $1",
            user_full_name
        )
        .fetch_one(&db.pool)
        .await?;

        sqlx::query!(
            "UPDATE customer_contracts
             SET user_id = $2
             WHERE uuid = ANY($1)",
            &contract_uuids,
            user.id
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(db: &Database, contract_uuids: Vec<Uuid>) -> Result<()> {
        sqlx::query!(
            "DELETE FROM customer_contracts
             WHERE uuid = ANY($1)",
            &contract_uuids
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }

    // CHART FUNCTIONS
    pub async fn get_production_value(db: &Database) -> Result<i64> {
        let chart = sqlx::query!(
            "SELECT
                COALESCE(SUM(annual_fee), 0) as production_value
            FROM customer_contracts;"
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(chart.production_value.unwrap())
    }

    pub async fn get_production_value_by_user_uuid(db: &Database, user_uuid: Uuid) -> Result<i64> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let chart = sqlx::query!(
            "SELECT
                COALESCE(SUM(annual_fee), 0) as production_value
            FROM customer_contracts
            WHERE user_id = $1",
            user_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(chart.production_value.unwrap())
    }

    pub async fn get_production_count(db: &Database) -> Result<i64> {
        let chart = sqlx::query!(
            "SELECT
                COALESCE(COUNT(*), 0) as production
            FROM customer_contracts;"
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(chart.production.unwrap())
    }

    pub async fn get_production_count_by_user_uuid(db: &Database, user_uuid: Uuid) -> Result<i64> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let chart = sqlx::query!(
            "SELECT
                COALESCE(COUNT(*), 0) as production
            FROM customer_contracts
            WHERE user_id = $1",
            user_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(chart.production.unwrap())
    }

    pub async fn get_portfolio_chart(db: &Database) -> Result<PortfolioDto> {
        let chart = sqlx::query!(
            "SELECT
                COUNT(*) FILTER (WHERE contract_type = 'BonusLifeProgram') AS bonus_life_program,
                COUNT(*) FILTER (WHERE contract_type = 'LifeProgram') AS life_program,
                COUNT(*) FILTER (WHERE contract_type = 'AllianzCareNow') AS allianz_care_now,
                COUNT(*) FILTER (WHERE contract_type = 'HealthProgram') AS health_program,
                COUNT(*) FILTER (WHERE contract_type = 'MyhomeHomeInsurance') AS myhome_home_insurance,
                COUNT(*) FILTER (WHERE contract_type = 'MfoHomeInsurance') AS mfo_home_insurance,
                COUNT(*) FILTER (WHERE contract_type = 'CorporatePropertyInsurance') AS corporate_property_insurance,
                COUNT(*) FILTER (WHERE contract_type = 'Kgfb') AS kgfb,
                COUNT(*) FILTER (WHERE contract_type = 'Casco') AS casco,
                COUNT(*) FILTER (WHERE contract_type = 'TravelInsurance') AS travel_insurance,
                COUNT(*) FILTER (WHERE contract_type = 'CondominiumInsurance') AS condominium_insurance,
                COUNT(*) FILTER (WHERE contract_type = 'AgriculturalInsurance') AS agricultural_insurance
            FROM customer_contracts;"
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(PortfolioDto {
            bonus_life_program: chart.bonus_life_program.unwrap(),
            life_program: chart.life_program.unwrap(),
            allianz_care_now: chart.allianz_care_now.unwrap(),
            health_program: chart.health_program.unwrap(),
            myhome_home_insurance: chart.myhome_home_insurance.unwrap(),
            mfo_home_insurance: chart.mfo_home_insurance.unwrap(),
            corporate_property_insurance: chart.corporate_property_insurance.unwrap(),
            kgfb: chart.kgfb.unwrap(),
            casco: chart.casco.unwrap(),
            travel_insurance: chart.travel_insurance.unwrap(),
            condominium_insurance: chart.condominium_insurance.unwrap(),
            agricultural_insurance: chart.agricultural_insurance.unwrap(),
        })
    }

    pub async fn get_portfolio_chart_by_user_uuid(
        db: &Database,
        user_uuid: Uuid,
    ) -> Result<PortfolioDto> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let chart = sqlx::query!(
            "SELECT
                COUNT(*) FILTER (WHERE contract_type = 'BonusLifeProgram') AS bonus_life_program,
                COUNT(*) FILTER (WHERE contract_type = 'LifeProgram') AS life_program,
                COUNT(*) FILTER (WHERE contract_type = 'AllianzCareNow') AS allianz_care_now,
                COUNT(*) FILTER (WHERE contract_type = 'HealthProgram') AS health_program,
                COUNT(*) FILTER (WHERE contract_type = 'MyhomeHomeInsurance') AS myhome_home_insurance,
                COUNT(*) FILTER (WHERE contract_type = 'MfoHomeInsurance') AS mfo_home_insurance,
                COUNT(*) FILTER (WHERE contract_type = 'CorporatePropertyInsurance') AS corporate_property_insurance,
                COUNT(*) FILTER (WHERE contract_type = 'Kgfb') AS kgfb,
                COUNT(*) FILTER (WHERE contract_type = 'Casco') AS casco,
                COUNT(*) FILTER (WHERE contract_type = 'TravelInsurance') AS travel_insurance,
                COUNT(*) FILTER (WHERE contract_type = 'CondominiumInsurance') AS condominium_insurance,
                COUNT(*) FILTER (WHERE contract_type = 'AgriculturalInsurance') AS agricultural_insurance
            FROM customer_contracts
            WHERE user_id = $1",
            user_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(PortfolioDto {
            bonus_life_program: chart.bonus_life_program.unwrap(),
            life_program: chart.life_program.unwrap(),
            allianz_care_now: chart.allianz_care_now.unwrap(),
            health_program: chart.health_program.unwrap(),
            myhome_home_insurance: chart.myhome_home_insurance.unwrap(),
            mfo_home_insurance: chart.mfo_home_insurance.unwrap(),
            corporate_property_insurance: chart.corporate_property_insurance.unwrap(),
            kgfb: chart.kgfb.unwrap(),
            casco: chart.casco.unwrap(),
            travel_insurance: chart.travel_insurance.unwrap(),
            condominium_insurance: chart.condominium_insurance.unwrap(),
            agricultural_insurance: chart.agricultural_insurance.unwrap(),
        })
    }

    pub async fn get_weekly_production_chart(
        db: &Database,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
    ) -> Result<WeeklyProductionChartDto> {
        let chart = sqlx::query!(
            "SELECT
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM handle_at) = 1) AS monday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM handle_at) = 2) AS tuesday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM handle_at) = 3) AS wednesday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM handle_at) = 4) AS thursday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM handle_at) = 5) AS friday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM handle_at) = 6) AS saturday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM handle_at) = 0) AS sunday
            FROM customer_contracts
            WHERE handle_at BETWEEN $1 AND $2",
            start_date.and_utc(),
            end_date.and_utc()
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(WeeklyProductionChartDto {
            monday: chart.monday.unwrap(),
            tuesday: chart.tuesday.unwrap(),
            wednesday: chart.wednesday.unwrap(),
            thursday: chart.thursday.unwrap(),
            friday: chart.friday.unwrap(),
            saturday: chart.saturday.unwrap(),
            sunday: chart.sunday.unwrap(),
        })
    }

    pub async fn get_weekly_production_chart_by_user_uuid(
        db: &Database,
        user_uuid: Uuid,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
    ) -> Result<WeeklyProductionChartDto> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let chart = sqlx::query!(
            "SELECT
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM handle_at) = 1) AS monday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM handle_at) = 2) AS tuesday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM handle_at) = 3) AS wednesday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM handle_at) = 4) AS thursday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM handle_at) = 5) AS friday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM handle_at) = 6) AS saturday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM handle_at) = 0) AS sunday
            FROM customer_contracts
            WHERE handle_at BETWEEN $2 AND $3 AND user_id = $1",
            user_id,
            start_date.and_utc(),
            end_date.and_utc()
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(WeeklyProductionChartDto {
            monday: chart.monday.unwrap(),
            tuesday: chart.tuesday.unwrap(),
            wednesday: chart.wednesday.unwrap(),
            thursday: chart.thursday.unwrap(),
            friday: chart.friday.unwrap(),
            saturday: chart.saturday.unwrap(),
            sunday: chart.sunday.unwrap(),
        })
    }

    pub async fn get_monthly_production_value_chart(
        db: &Database,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
    ) -> Result<Vec<MonthlyProductionChartDto>> {
        let charts = sqlx::query!(
            "SELECT
                CAST(EXTRACT(MONTH FROM handle_at) AS SMALLINT) AS month,
                COALESCE(SUM(annual_fee) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 1), 0) AS week1,
                COALESCE(SUM(annual_fee) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 2), 0) AS week2,
                COALESCE(SUM(annual_fee) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 3), 0) AS week3,
                COALESCE(SUM(annual_fee) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 4), 0) AS week4,
                COALESCE(SUM(annual_fee) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 5), 0) AS week5
            FROM customer_contracts
            WHERE handle_at BETWEEN $1 AND $2
            GROUP BY month
            ORDER BY month;",
            start_date.and_utc(),
            end_date.and_utc()
        )
        .fetch_all(&db.pool)
        .await?;

        let dates = charts
            .into_iter()
            .map(|chart| MonthlyProductionChartDto {
                month: chart.month.unwrap(),
                week1: chart.week1.unwrap(),
                week2: chart.week2.unwrap(),
                week3: chart.week3.unwrap(),
                week4: chart.week4.unwrap(),
                week5: chart.week5.unwrap(),
            })
            .collect();

        Ok(dates)
    }

    pub async fn get_monthly_production_value_chart_by_user_uuid(
        db: &Database,
        user_uuid: Uuid,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
    ) -> Result<Vec<MonthlyProductionChartDto>> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let charts = sqlx::query!(
            "SELECT
                CAST(EXTRACT(MONTH FROM handle_at) AS SMALLINT) AS month,
                COALESCE(SUM(annual_fee) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 1), 0) AS week1,
                COALESCE(SUM(annual_fee) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 2), 0) AS week2,
                COALESCE(SUM(annual_fee) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 3), 0) AS week3,
                COALESCE(SUM(annual_fee) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 4), 0) AS week4,
                COALESCE(SUM(annual_fee) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 5), 0) AS week5
            FROM customer_contracts
            WHERE handle_at BETWEEN $2 AND $3 AND user_id = $1
            GROUP BY month
            ORDER BY month;",
            user_id,
            start_date.and_utc(),
            end_date.and_utc()
        )
        .fetch_all(&db.pool)
        .await?;

        let dates = charts
            .into_iter()
            .map(|chart| MonthlyProductionChartDto {
                month: chart.month.unwrap(),
                week1: chart.week1.unwrap(),
                week2: chart.week2.unwrap(),
                week3: chart.week3.unwrap(),
                week4: chart.week4.unwrap(),
                week5: chart.week5.unwrap(),
            })
            .collect();

        Ok(dates)
    }

    pub async fn get_monthly_production_chart(
        db: &Database,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
    ) -> Result<Vec<MonthlyProductionChartDto>> {
        let charts = sqlx::query!(
            "SELECT
                CAST(EXTRACT(MONTH FROM handle_at) as SMALLINT) AS month,
                COUNT(*) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 1) AS week1,
                COUNT(*) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 2) AS week2,
                COUNT(*) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 3) AS week3,
                COUNT(*) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 4) AS week4,
                COUNT(*) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 5) AS week5
            FROM customer_contracts
            WHERE handle_at BETWEEN $1 AND $2
            GROUP BY month
            ORDER BY month;",
            start_date.and_utc(),
            end_date.and_utc()
        )
        .fetch_all(&db.pool)
        .await?;

        let dates = charts
            .into_iter()
            .map(|chart| MonthlyProductionChartDto {
                month: chart.month.unwrap(),
                week1: chart.week1.unwrap(),
                week2: chart.week2.unwrap(),
                week3: chart.week3.unwrap(),
                week4: chart.week4.unwrap(),
                week5: chart.week5.unwrap(),
            })
            .collect();

        Ok(dates)
    }

    pub async fn get_monthly_production_chart_by_user_uuid(
        db: &Database,
        user_uuid: Uuid,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
    ) -> Result<Vec<MonthlyProductionChartDto>> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let charts = sqlx::query!(
            "SELECT
                CAST(EXTRACT(MONTH FROM handle_at) as SMALLINT) AS month,
                COUNT(*) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 1) AS week1,
                COUNT(*) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 2) AS week2,
                COUNT(*) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 3) AS week3,
                COUNT(*) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 4) AS week4,
                COUNT(*) FILTER (WHERE EXTRACT(WEEK FROM handle_at) - EXTRACT(WEEK FROM DATE_TRUNC('month', handle_at)) + 1 = 5) AS week5
            FROM customer_contracts
            WHERE handle_at BETWEEN $2 AND $3 AND user_id = $1
            GROUP BY month
            ORDER BY month;",
            user_id,
            start_date.and_utc(),
            end_date.and_utc()
        )
        .fetch_all(&db.pool)
        .await?;

        let dates = charts
            .into_iter()
            .map(|chart| MonthlyProductionChartDto {
                month: chart.month.unwrap(),
                week1: chart.week1.unwrap(),
                week2: chart.week2.unwrap(),
                week3: chart.week3.unwrap(),
                week4: chart.week4.unwrap(),
                week5: chart.week5.unwrap(),
            })
            .collect();

        Ok(dates)
    }
}
