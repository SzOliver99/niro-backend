use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    extractors::authentication_token::AuthenticationToken,
    models::{
        user::{User, UserRole},
        user_date::{MeetType, UserMeetDate},
    },
    utils::error::ApiError,
    web_data::WebData,
};

pub fn dates_scope() -> Scope {
    web::scope("/dates")
        .route("/create", web::post().to(create_date))
        .route("/modify", web::put().to(modify_date))
        .route(
            "/{user_uuid}/{selected_month}",
            web::get().to(get_all_by_dates),
        )
        .route("/{date_uuid}", web::get().to(get_date_by_uuid))
        .route("/{date_uuid}/state", web::put().to(change_date_state))
        .route("/change/user", web::put().to(change_dates_handler))
        .route("/delete", web::delete().to(delete_dates))
        .route(
            "/chart/is-completed/get-all",
            web::get().to(get_is_completed_chart),
        )
        .route(
            "/chart/is-completed/{user_uuid}",
            web::get().to(get_is_completed_chart_by_user_uuid),
        )
        .route(
            "/chart/meet-type/get-all",
            web::get().to(get_meet_type_chart),
        )
        .route(
            "/chart/meet-type/{user_uuid}",
            web::get().to(get_meet_type_chart_by_user_uuid),
        )
        .route(
            "/chart/weekly/get-all",
            web::post().to(get_dates_weekly_chart),
        )
        .route(
            "/chart/weekly/{user_uuid}",
            web::post().to(get_dates_weekly_chart_by_user_uuid),
        )
        .route(
            "/chart/monthly/get-all",
            web::post().to(get_dates_monthly_chart),
        )
        .route(
            "/chart/monthly/{user_uuid}",
            web::post().to(get_dates_monthly_chart_by_user_uuid),
        )
}

#[derive(Deserialize, Clone)]
struct CreateDateJson {
    meet_date: String,
    full_name: String,
    phone_number: String,
    meet_location: String,
    meet_type: MeetType,
    created_by: String,
    user_uuid: Uuid,
}
async fn create_date(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    data: web::Json<CreateDateJson>,
) -> impl Responder {
    let parsed_date = chrono::NaiveDateTime::parse_from_str(&data.meet_date, "%Y-%m-%dT%H:%M")
        .or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(&data.meet_date).map(|dt| dt.naive_utc())
        });

    let meet_date = match parsed_date {
        Ok(d) => d,
        Err(e) => return ApiError::from(anyhow!(e)).error_response(),
    };

    let user_date = UserMeetDate {
        meet_date: Some(meet_date),
        full_name: Some(data.full_name.clone()),
        phone_number: Some(data.phone_number.clone()),
        meet_location: Some(data.meet_location.clone()),
        meet_type: Some(data.meet_type.clone()),
        is_completed: Some(false),
        created_by: Some(data.created_by.clone()),
        ..Default::default()
    };

    match UserMeetDate::create(
        &web_data.db,
        &web_data.key,
        &web_data.hmac_secret,
        data.user_uuid,
        user_date,
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Időpont sikeresen létrehozva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize, Clone)]
struct ModifyDateJson {
    date_uuid: Uuid,
    meet_date: String,
    full_name: String,
    phone_number: String,
    meet_location: String,
    meet_type: MeetType,
}

async fn modify_date(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    data: web::Json<ModifyDateJson>,
) -> impl Responder {
    let parsed_date = chrono::NaiveDateTime::parse_from_str(&data.meet_date, "%Y-%m-%dT%H:%M")
        .or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(&data.meet_date).map(|dt| dt.naive_utc())
        });

    let meet_date = match parsed_date {
        Ok(d) => d,
        Err(e) => return ApiError::from(anyhow!(e)).error_response(),
    };

    let user_date = UserMeetDate {
        meet_date: Some(meet_date),
        full_name: Some(data.full_name.clone()),
        phone_number: Some(data.phone_number.clone()),
        meet_location: Some(data.meet_location.clone()),
        meet_type: Some(data.meet_type.clone()),
        ..Default::default()
    };

    match UserMeetDate::modify(
        &web_data.db,
        &web_data.key,
        &web_data.hmac_secret,
        data.date_uuid,
        user_date,
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Időpont sikeresen módosítva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_all_by_dates(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    path: web::Path<(Uuid, String)>,
) -> impl Responder {
    match UserMeetDate::get_all(&web_data.db, &web_data.key, path.clone().0, path.clone().1).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_date_by_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    date_uuid: web::Path<Uuid>,
) -> impl Responder {
    match UserMeetDate::get_by_uuid(&web_data.db, &web_data.key, date_uuid.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn change_date_state(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    date_uuid: web::Path<Uuid>,
    data: web::Json<bool>,
) -> impl Responder {
    match UserMeetDate::change_date_state(&web_data.db, date_uuid.into_inner(), data.0).await {
        Ok(_) => HttpResponse::Ok().json("Időpont státusza megváltoztatva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize)]
struct ChangeDatesHandlerJson {
    user_full_name: String,
    date_uuids: Vec<Uuid>,
}
async fn change_dates_handler(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<ChangeDatesHandlerJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Leader, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match UserMeetDate::change_handler(
        &web_data.db,
        data.user_full_name.clone(),
        data.date_uuids.clone(),
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().json("Időpontért felelős üzletkötő megváltoztatva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn delete_dates(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<Vec<Uuid>>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Agent, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match UserMeetDate::delete(&web_data.db, data.0).await {
        Ok(_) => HttpResponse::Ok().json("Időpont(ok) sikeresen tölölve!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

// USER DATE CHART API's
async fn get_is_completed_chart(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match UserMeetDate::get_is_completed_chart(&web_data.db).await {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_is_completed_chart_by_user_uuid(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    user_uuid: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match UserMeetDate::get_is_completed_chart_by_user_uuid(&web_data.db, user_uuid.into_inner())
        .await
    {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_meet_type_chart(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match UserMeetDate::get_meet_type_chart(&web_data.db).await {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_meet_type_chart_by_user_uuid(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    user_uuid: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match UserMeetDate::get_meet_type_chart_by_user_uuid(&web_data.db, user_uuid.into_inner()).await
    {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize)]
struct DateChartJson {
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
}
async fn get_dates_weekly_chart(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<DateChartJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match UserMeetDate::get_dates_weekly_chart(&web_data.db, data.start_date, data.end_date).await {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_dates_weekly_chart_by_user_uuid(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    user_uuid: web::Path<Uuid>,
    data: web::Json<DateChartJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match UserMeetDate::get_dates_weekly_chart_by_user_uuid(
        &web_data.db,
        user_uuid.into_inner(),
        data.start_date,
        data.end_date,
    )
    .await
    {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_dates_monthly_chart(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<DateChartJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match UserMeetDate::get_dates_monthly_chart(&web_data.db, data.start_date, data.end_date).await
    {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_dates_monthly_chart_by_user_uuid(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    user_uuid: web::Path<Uuid>,
    data: web::Json<DateChartJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match UserMeetDate::get_dates_monthly_chart_by_user_uuid(
        &web_data.db,
        user_uuid.into_inner(),
        data.start_date,
        data.end_date,
    )
    .await
    {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}
