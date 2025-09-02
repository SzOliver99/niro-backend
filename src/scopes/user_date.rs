use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use chrono::{NaiveDateTime, Utc};
use serde::Deserialize;

use crate::{
    extractors::authentication_token::AuthenticationToken,
    models::user::{User, UserRole},
    models::user_date::UserMeetDate,
    utils::error::ApiError,
    web_data::WebData,
};

pub fn dates_scope() -> Scope {
    web::scope("/dates")
        .route("/create", web::post().to(create_date))
        .route("/get-all", web::post().to(get_all_dates))
        .route("/change/user", web::post().to(change_dates_handler))
        .route("/change/state", web::post().to(change_date_state))
        .route("/delete", web::delete().to(delete_dates))
}

#[derive(Deserialize, Clone)]
struct CreateDateJson {
    meet_date: String,
    full_name: String,
    phone_number: String,
    meet_location: String,
    meet_type: String,
    created_by: String,
    user_id: i32,
}

async fn create_date(
    web_data: web::Data<WebData>,
    data: web::Json<CreateDateJson>,
) -> impl Responder {
    let parsed_date = chrono::NaiveDateTime::parse_from_str(&data.meet_date, "%Y-%m-%dT%H:%M")
        .or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(&data.meet_date).map(|dt| dt.naive_utc())
        });

    let meet_date = match parsed_date {
        Ok(d) => d,
        Err(e) => return ApiError::from(anyhow::anyhow!(e)).error_response(),
    };

    let user_date = UserMeetDate {
        meet_date: Some(meet_date),
        full_name: Some(data.full_name.clone()),
        phone_number: Some(data.phone_number.clone()),
        meet_location: Some(data.meet_location.clone()),
        meet_type: Some(data.meet_type.clone()),
        is_completed: Some(false),
        created_by: Some(data.created_by.clone()),
        user_id: Some(data.user_id),
        ..Default::default()
    };
    println!("User_date: {:?}", user_date);

    match UserMeetDate::create(
        &web_data.db,
        &web_data.key,
        &web_data.hmac_secret,
        user_date,
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Időpont sikeresen létrehozva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_all_dates(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<i32>,
) -> impl Responder {
    if data.0 != auth_token.id as i32 {
        if let Err(e) =
            User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
        {
            return ApiError::from(e).error_response();
        }
    }

    match UserMeetDate::get_all(&web_data.db, &web_data.key, data.0).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize)]
struct ChangeDatesHandlerJson {
    user_full_name: String,
    date_ids: Vec<i32>,
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
        data.date_ids.clone(),
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().json("Időpontért felelős üzletkötő megváltoztatva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize)]
struct ChangeUserDateStateJson {
    date_id: i32,
    value: bool,
}
async fn change_date_state(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    data: web::Json<ChangeUserDateStateJson>,
) -> impl Responder {
    match UserMeetDate::change_date_state(&web_data.db, data.date_id, data.value).await {
        Ok(_) => HttpResponse::Ok().json("Időpont státusza megváltoztatva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn delete_dates(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<Vec<i32>>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Agent, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match UserMeetDate::delete(&web_data.db, data.0).await {
        Ok(_) => HttpResponse::Ok().json("Időpont(ok) sikeresen tölölve!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}
