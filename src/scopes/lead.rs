use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    extractors::authentication_token::AuthenticationToken,
    models::{
        customer::Customer,
        lead::{Lead, LeadStatus, LeadType},
        user::{User, UserRole},
    },
    utils::error::ApiError,
    web_data::WebData,
};

pub fn lead_scope() -> Scope {
    web::scope("/lead")
        .route("/create", web::post().to(create_lead))
        .route("/modify", web::put().to(modify_lead))
        .route(
            "/get-all/{user_uuid}",
            web::get().to(get_leads_by_user_uuid),
        )
        .route("/{lead_uuid}", web::get().to(get_lead_by_uuid))
        .route("/{lead_uuid}/customer", web::get().to(get_customer_uuid))
        .route("/change/user", web::put().to(change_lead_handler))
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
    lead_type: LeadType,
    inquiry_type: String,
    lead_status: LeadStatus,
    user_uuid: Uuid,
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
        created_by: Some(data.created_by.clone()),
        ..Default::default()
    };
    let lead = Lead {
        lead_type: Some(data.lead_type.clone()),
        inquiry_type: Some(data.inquiry_type.clone()),
        lead_status: Some(data.lead_status.clone()),
        created_by: Some(data.created_by.clone()),
        ..Default::default()
    };

    match Lead::create(
        &web_data.db,
        &web_data.key,
        &web_data.hmac_secret,
        data.user_uuid,
        customer,
        lead,
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Tevékenység sikeresen létrehozva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize, Clone)]
struct ModifyLeadJson {
    lead_uuid: Uuid,
    lead_type: LeadType,
    inquiry_type: String,
    lead_status: LeadStatus,
}
async fn modify_lead(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    data: web::Json<ModifyLeadJson>,
) -> impl Responder {
    let lead = Lead {
        lead_type: Some(data.lead_type.clone()),
        inquiry_type: Some(data.inquiry_type.clone()),
        lead_status: Some(data.lead_status.clone()),
        ..Default::default()
    };

    match Lead::modify(&web_data.db, data.lead_uuid, lead).await {
        Ok(_) => HttpResponse::Created().json("Sikeresen megváltoztattad a tevékenységet!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_leads_by_user_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    user_uuid: web::Path<Uuid>,
) -> impl Responder {
    match Lead::get_all(&web_data.db, &web_data.key, user_uuid.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_lead_by_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    lead_uuid: web::Path<Uuid>,
) -> impl Responder {
    match Lead::get_by_uuid(&web_data.db, lead_uuid.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_customer_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    lead_uuid: web::Path<Uuid>,
) -> impl Responder {
    match Lead::get_customer_uuid(&web_data.db, lead_uuid.into_inner()).await {
        Ok(customer_uuid) => HttpResponse::Ok().json(customer_uuid),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize)]
struct ChangeLeadsHandlerJson {
    user_full_name: String,
    lead_uuids: Vec<Uuid>,
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
        data.lead_uuids.clone(),
    )
    .await
    {
        Ok(_) => {
            HttpResponse::Created().json("Tevékenység(ek)ért felelős üzletkötő megváltoztatva!")
        }
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn delete_lead(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<Vec<Uuid>>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Agent, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match Lead::delete(&web_data.db, data.0).await {
        Ok(_) => HttpResponse::Created().json("Tevékenység(ek) sikeresen törölve!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}
