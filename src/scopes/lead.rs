use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use chrono::Timelike;
use serde::Deserialize;

use crate::{
    extractors::authentication_token::AuthenticationToken,
    models::user::{User, UserRole},
    models::{
        customer::Customer,
        lead::{Lead, LeadStatus},
    },
    utils::error::ApiError,
    web_data::WebData,
};

pub fn lead_scope() -> Scope {
    web::scope("/lead")
        .route("/create", web::post().to(create_lead))
        .route("/get-all", web::post().to(get_leads_by_user_id))
        .route("/change/user", web::post().to(change_lead_handler))
        .route("/delete", web::delete().to(delete_lead))
}

#[derive(Deserialize, Clone)]
struct CustomerJson {
    full_name: String,
    phone_number: String,
    address: String,
    email: String,
}
#[derive(Deserialize, Clone)]
struct CreateLeadJson {
    customer: CustomerJson,
    lead_type: String,
    inquiry_type: String,
    lead_status: LeadStatus,
    user_id: i32,
    created_by: String,
}
async fn create_lead(
    web_data: web::Data<WebData>,
    data: web::Json<CreateLeadJson>,
) -> impl Responder {
    let customer = Customer {
        full_name: Some(data.customer.full_name.clone()),
        phone_number: Some(data.customer.phone_number.clone()),
        email: Some(data.customer.email.clone()),
        address: Some(data.customer.address.clone()),
        user_id: Some(data.user_id),
        created_by: Some(data.created_by.clone()),
        ..Default::default()
    };
    let lead = Lead {
        lead_type: Some(data.lead_type.clone()),
        inquiry_type: Some(data.inquiry_type.clone()),
        lead_status: Some(data.lead_status.clone()),
        handle_at: Some(chrono::Utc::now().naive_utc().with_nanosecond(0).unwrap()),
        ..Default::default()
    };

    match Lead::create(
        &web_data.db,
        &web_data.key,
        &web_data.hmac_secret,
        customer,
        lead,
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Registration successful!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_leads_by_user_id(
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

    match Lead::get_all(&web_data.db, &web_data.key, data.0).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize)]
struct ChangeLeadsHandlerJson {
    user_full_name: String,
    lead_ids: Vec<i32>,
}
async fn change_lead_handler(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<ChangeLeadsHandlerJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Leader, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match Lead::change_handler(
        &web_data.db,
        data.user_full_name.clone(),
        data.lead_ids.clone(),
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Registration successful!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn delete_lead(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<Vec<i32>>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Agent, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match Lead::delete(&web_data.db, data.0).await {
        Ok(_) => HttpResponse::Created().json("Címanyag(ok) sikeresen létrehozva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}
