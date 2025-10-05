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
        dto::{DatesMonthlyChartDto, DatesWeeklyChartDto, IsCompletedChartDto, MeetTypeChartDto},
        user::User,
    },
    utils::encrypt::{self, HmacSecret},
};

#[skip_serializing_none]
#[derive(Debug, Serialize, Default, Clone)]
pub struct UserMeetDate {
    pub id: Option<i32>,
    pub uuid: Option<Uuid>,
    pub meet_date: Option<NaiveDateTime>,
    pub full_name: Option<String>,
    pub phone_number: Option<String>,
    pub meet_location: Option<String>,
    pub meet_type: Option<MeetType>,
    pub is_completed: Option<bool>,
    pub created_by: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub user_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumString, Display, Type, AsRefStr)]
pub enum MeetType {
    NeedsAssessment,
    Consultation,
    Service,
    AnnualReview,
}

impl UserMeetDate {
    pub async fn create(
        db: &Database,
        key: &Key,
        hmac_secret: &HmacSecret,
        user_uuid: Uuid,
        new_meet_date: UserMeetDate,
    ) -> Result<i32> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let phone = new_meet_date.phone_number.as_deref().unwrap();
        let phone_hash = encrypt::hash_value(&hmac_secret, phone);
        let (phone_enc, phone_nonce) = encrypt::encrypt_value(&key, phone);

        let row = sqlx::query!(
            "INSERT INTO user_dates(meet_date, full_name, phone_number_enc, phone_number_nonce, phone_number_hash, meet_location, meet_type, created_by, user_id)
             VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id",
            new_meet_date.meet_date,
            new_meet_date.full_name,
            phone_enc,
            phone_nonce,
            phone_hash,
            new_meet_date.meet_location,
            new_meet_date.meet_type.map(|t| t.to_string()),
            new_meet_date.created_by,
            user_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(row.id)
    }

    pub async fn modify(
        db: &Database,
        key: &Key,
        hmac_secret: &HmacSecret,
        date_uuid: Uuid,
        updated_user_date: UserMeetDate,
    ) -> Result<()> {
        let phone = updated_user_date
            .phone_number
            .as_deref()
            .unwrap_or_default();
        let phone_hash = encrypt::hash_value(hmac_secret, phone);
        let (phone_enc, phone_nonce) = encrypt::encrypt_value(key, phone);

        sqlx::query!(
            "UPDATE user_dates
             SET meet_date = $1,
                 full_name = $2,
                 phone_number_enc = $3,
                 phone_number_nonce = $4,
                 phone_number_hash = $5,
                 meet_location = $6,
                 meet_type = $7
             WHERE uuid = $8",
            updated_user_date.meet_date,
            updated_user_date.full_name,
            phone_enc,
            phone_nonce,
            phone_hash,
            updated_user_date.meet_location,
            updated_user_date.meet_type.map(|t| t.to_string()),
            date_uuid
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get_all(
        db: &Database,
        key: &Key,
        user_uuid: Uuid,
        selected_month: String,
    ) -> Result<Vec<UserMeetDate>> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let rows = sqlx::query!(
            "SELECT uuid, meet_date, full_name, phone_number_enc, phone_number_nonce, phone_number_hash, meet_location, meet_type, is_completed, created_by, created_at
             FROM user_dates
             WHERE user_id = $1 AND TRIM(TO_CHAR(meet_date, 'Month')) = $2
             ORDER BY meet_date DESC",
            user_id,
            selected_month
        )
        .fetch_all(&db.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| UserMeetDate {
                uuid: row.uuid,
                meet_date: Some(row.meet_date),
                full_name: Some(row.full_name),
                phone_number: encrypt::decrypt_value(
                    key,
                    &row.phone_number_enc,
                    &row.phone_number_nonce,
                ),
                meet_location: Some(row.meet_location),
                meet_type: Some(row.meet_type.parse().unwrap()),
                is_completed: Some(row.is_completed),
                created_by: Some(row.created_by),
                created_at: Some(row.created_at),
                ..Default::default()
            })
            .collect())
    }

    pub async fn get_by_uuid(db: &Database, key: &Key, date_uuid: Uuid) -> Result<UserMeetDate> {
        let row = sqlx::query!(
            "SELECT
                uuid,
                meet_date,
                full_name,
                phone_number_enc,
                phone_number_nonce,
                phone_number_hash,
                meet_location,
                meet_type,
                is_completed,
                created_by,
                created_at
            FROM
                user_dates
            WHERE
	            uuid = $1",
            date_uuid
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(UserMeetDate {
            uuid: row.uuid,
            meet_date: Some(row.meet_date),
            full_name: Some(row.full_name),
            phone_number: encrypt::decrypt_value(
                key,
                &row.phone_number_enc,
                &row.phone_number_nonce,
            ),
            meet_location: Some(row.meet_location),
            meet_type: Some(row.meet_type.parse().unwrap()),
            is_completed: Some(row.is_completed),
            created_by: Some(row.created_by),
            created_at: Some(row.created_at),
            ..Default::default()
        })
    }

    pub async fn change_date_state(
        db: &Database,
        date_uuid: Uuid,
        is_completed: bool,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE user_dates
             SET is_completed = $2
             WHERE uuid = $1",
            &date_uuid,
            is_completed
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn change_handler(
        db: &Database,
        user_full_name: String,
        date_uuids: Vec<Uuid>,
    ) -> Result<()> {
        let user = sqlx::query!(
            "SELECT user_id as id FROM user_info WHERE full_name = $1",
            user_full_name
        )
        .fetch_one(&db.pool)
        .await?;

        sqlx::query!(
            "UPDATE user_dates
             SET user_id = $2
             WHERE uuid = ANY($1)",
            &date_uuids,
            user.id
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(db: &Database, date_uuids: Vec<Uuid>) -> Result<()> {
        sqlx::query!("DELETE FROM user_dates WHERE uuid = ANY($1)", &date_uuids)
            .execute(&db.pool)
            .await?;
        Ok(())
    }

    // CHART FUNCTIONS
    pub async fn get_is_completed_chart(db: &Database) -> Result<IsCompletedChartDto> {
        let chart = sqlx::query!(
            "SELECT
                COUNT(*) FILTER (WHERE is_completed = TRUE)  AS yes,
                COUNT(*) FILTER (WHERE is_completed = FALSE) AS no
            FROM user_dates;"
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(IsCompletedChartDto {
            yes: chart.yes.unwrap(),
            no: chart.no.unwrap(),
        })
    }

    pub async fn get_is_completed_chart_by_user_uuid(
        db: &Database,
        user_uuid: Uuid,
    ) -> Result<IsCompletedChartDto> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let chart = sqlx::query!(
            "SELECT
                COUNT(*) FILTER (WHERE is_completed = TRUE AND user_id = $1)  AS yes,
                COUNT(*) FILTER (WHERE is_completed = FALSE AND user_id = $1) AS no
            FROM user_dates;",
            user_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(IsCompletedChartDto {
            yes: chart.yes.unwrap(),
            no: chart.no.unwrap(),
        })
    }

    pub async fn get_meet_type_chart(db: &Database) -> Result<MeetTypeChartDto> {
        let chart = sqlx::query!(
            "SELECT
                COUNT(*) FILTER (WHERE meet_type = 'NeedsAssessment') AS needs_assessment,
                COUNT(*) FILTER (WHERE meet_type = 'Consultation') AS consultation,
                COUNT(*) FILTER (WHERE meet_type = 'Service') AS service,
                COUNT(*) FILTER (WHERE meet_type = 'AnnualReview') AS annual_review
            FROM user_dates;"
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(MeetTypeChartDto {
            needs_assessment: chart.needs_assessment.unwrap(),
            consultation: chart.consultation.unwrap(),
            service: chart.service.unwrap(),
            annual_review: chart.annual_review.unwrap(),
        })
    }

    pub async fn get_meet_type_chart_by_user_uuid(
        db: &Database,
        user_uuid: Uuid,
    ) -> Result<MeetTypeChartDto> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let chart = sqlx::query!(
            "SELECT
                COUNT(*) FILTER (WHERE meet_type = 'NeedsAssessment' AND user_id = $1) AS needs_assessment,
                COUNT(*) FILTER (WHERE meet_type = 'Consultation' AND user_id = $1) AS consultation,
                COUNT(*) FILTER (WHERE meet_type = 'Service' AND user_id = $1) AS service,
                COUNT(*) FILTER (WHERE meet_type = 'AnnualReview' AND user_id = $1) AS annual_review
            FROM user_dates;",
            user_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(MeetTypeChartDto {
            needs_assessment: chart.needs_assessment.unwrap(),
            consultation: chart.consultation.unwrap(),
            service: chart.service.unwrap(),
            annual_review: chart.annual_review.unwrap(),
        })
    }

    pub async fn get_dates_weekly_chart(
        db: &Database,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
    ) -> Result<DatesWeeklyChartDto> {
        let chart = sqlx::query!(
            "SELECT
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM meet_date) = 1) AS monday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM meet_date) = 2) AS tuesday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM meet_date) = 3) AS wednesday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM meet_date) = 4) AS thursday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM meet_date) = 5) AS friday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM meet_date) = 6) AS saturday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM meet_date) = 0) AS sunday
            FROM user_dates
            WHERE meet_date BETWEEN $1 AND $2",
            start_date,
            end_date
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(DatesWeeklyChartDto {
            monday: chart.monday.unwrap(),
            tuesday: chart.tuesday.unwrap(),
            wednesday: chart.wednesday.unwrap(),
            thursday: chart.thursday.unwrap(),
            friday: chart.friday.unwrap(),
            saturday: chart.saturday.unwrap(),
            sunday: chart.sunday.unwrap(),
        })
    }

    pub async fn get_dates_weekly_chart_by_user_uuid(
        db: &Database,
        user_uuid: Uuid,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
    ) -> Result<DatesWeeklyChartDto> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let chart = sqlx::query!(
            "SELECT
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM meet_date) = 1) AS monday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM meet_date) = 2) AS tuesday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM meet_date) = 3) AS wednesday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM meet_date) = 4) AS thursday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM meet_date) = 5) AS friday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM meet_date) = 6) AS saturday,
                COUNT(*) FILTER (WHERE EXTRACT(DOW FROM meet_date) = 0) AS sunday
            FROM user_dates
            WHERE meet_date BETWEEN $2 AND $3 AND user_id = $1",
            user_id,
            start_date,
            end_date
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(DatesWeeklyChartDto {
            monday: chart.monday.unwrap(),
            tuesday: chart.tuesday.unwrap(),
            wednesday: chart.wednesday.unwrap(),
            thursday: chart.thursday.unwrap(),
            friday: chart.friday.unwrap(),
            saturday: chart.saturday.unwrap(),
            sunday: chart.sunday.unwrap(),
        })
    }

    pub async fn get_dates_monthly_chart(
        db: &Database,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
    ) -> Result<DatesMonthlyChartDto> {
        let chart = sqlx::query!(
            "SELECT
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 1) AS january,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 2) AS february,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 3) AS march,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 4) AS april,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 5) AS may,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 6) AS june,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 7) AS july,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 8) AS august,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 9) AS september,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 10) AS october,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 11) AS november,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 12) AS december
            FROM user_dates
            WHERE meet_date BETWEEN $1 AND $2",
            start_date,
            end_date
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(DatesMonthlyChartDto {
            january: chart.january.unwrap(),
            february: chart.february.unwrap(),
            march: chart.march.unwrap(),
            april: chart.april.unwrap(),
            may: chart.may.unwrap(),
            june: chart.june.unwrap(),
            july: chart.july.unwrap(),
            august: chart.august.unwrap(),
            september: chart.september.unwrap(),
            october: chart.october.unwrap(),
            november: chart.november.unwrap(),
            december: chart.december.unwrap(),
        })
    }

    pub async fn get_dates_monthly_chart_by_user_uuid(
        db: &Database,
        user_uuid: Uuid,
        start_date: NaiveDateTime,
        end_date: NaiveDateTime,
    ) -> Result<DatesMonthlyChartDto> {
        let user_id = User::get_id_by_uuid(db, Some(user_uuid))
            .await?
            .ok_or_else(|| anyhow!("Felhasználó nem található!"))?;

        let chart = sqlx::query!(
            "SELECT
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 1) AS january,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 2) AS february,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 3) AS march,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 4) AS april,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 5) AS may,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 6) AS june,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 7) AS july,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 8) AS august,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 9) AS september,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 10) AS october,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 11) AS november,
                COUNT(*) FILTER (WHERE EXTRACT(MONTH FROM meet_date) = 12) AS december
            FROM user_dates
            WHERE meet_date BETWEEN $2 AND $3 AND user_id = $1",
            user_id,
            start_date,
            end_date
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(DatesMonthlyChartDto {
            january: chart.january.unwrap(),
            february: chart.february.unwrap(),
            march: chart.march.unwrap(),
            april: chart.april.unwrap(),
            may: chart.may.unwrap(),
            june: chart.june.unwrap(),
            july: chart.july.unwrap(),
            august: chart.august.unwrap(),
            september: chart.september.unwrap(),
            october: chart.october.unwrap(),
            november: chart.november.unwrap(),
            december: chart.december.unwrap(),
        })
    }
}
