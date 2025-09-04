use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    extractors::authentication_token::AuthenticationToken,
    models::{
        customer::Customer,
        lead::Lead,
        user::{User, UserRole},
    },
    utils::error::ApiError,
    web_data::WebData,
};

pub fn customer_scope() -> Scope {
    web::scope("/customer")
        .route("/create", web::post().to(create_customer))
        .route("/modify", web::put().to(modify_customer))
        .route("/leads", web::post().to(get_leads_by_customer_uuid))
        .route("/get-all", web::post().to(get_customers_by_uuid))
        .route("/get", web::post().to(get_customer_by_uuid))
        .route("/change/user", web::post().to(change_customer_handler))
        .route("/delete", web::delete().to(delete_customer))
}

#[derive(Deserialize, Clone)]
struct CreateCustomerJson {
    user_uuid: Uuid,
    full_name: String,
    phone_number: String,
    address: String,
    email: String,
    created_by: String,
}
async fn create_customer(
    web_data: web::Data<WebData>,
    data: web::Json<CreateCustomerJson>,
) -> impl Responder {
    let customer = Customer {
        uuid: Some(data.user_uuid),
        full_name: Some(data.full_name.clone()),
        phone_number: Some(data.phone_number.clone()),
        address: Some(data.address.clone()),
        email: Some(data.email.clone()),
        created_by: Some(data.created_by.clone()),
        ..Default::default()
    };

    match Customer::create(&web_data.db, &web_data.key, &web_data.hmac_secret, customer).await {
        Ok(_) => HttpResponse::Created().json("Sikeresen létre lett hozva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize, Clone)]
struct ModifyCustomerJson {
    customer_uuid: Uuid,
    full_name: String,
    phone_number: String,
    address: String,
    email: String,
}
async fn modify_customer(
    web_data: web::Data<WebData>,
    data: web::Json<ModifyCustomerJson>,
) -> impl Responder {
    let customer = Customer {
        full_name: Some(data.full_name.clone()),
        phone_number: Some(data.phone_number.clone()),
        address: Some(data.address.clone()),
        email: Some(data.email.clone()),
        ..Default::default()
    };

    match Customer::modify(
        &web_data.db,
        &web_data.key,
        &web_data.hmac_secret,
        data.customer_uuid,
        customer,
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Sikeresen módosítottad az ügyfelet!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_customers_by_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    data: web::Json<Uuid>,
) -> impl Responder {
    match Customer::get_all(&web_data.db, &web_data.key, data.0).await {
        Ok(customers) => HttpResponse::Created().json(customers),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_leads_by_customer_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    data: web::Json<Uuid>,
) -> impl Responder {
    match Lead::get_by_customer_uuid(&web_data.db, data.0).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_customer_by_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    data: web::Json<Uuid>,
) -> impl Responder {
    match Customer::get_by_uuid(&web_data.db, &web_data.key, data.0).await {
        Ok(customers) => HttpResponse::Created().json(customers),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize)]
struct ChangeCustomersHandlerJson {
    user_full_name: String,
    customer_uuids: Vec<Uuid>,
}
async fn change_customer_handler(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<ChangeCustomersHandlerJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Leader, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match Customer::change_handler(
        &web_data.db,
        data.user_full_name.clone(),
        data.customer_uuids.clone(),
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Ügyfélt kezelő üzletkötő sikeresen megváltoztatva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn delete_customer(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<Vec<Uuid>>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Agent, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match Customer::delete(&web_data.db, data.0).await {
        Ok(_) => HttpResponse::Created().json("Ügyfél sikeresen létrehozva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

// #[derive(Deserialize)]
// struct PaginationQuery {
//     user_id: i32,
//     limit: Option<i64>,
//     offset: Option<i64>,
// }

// async fn list_contacts(
//     web_data: web::Data<WebData>,
//     query: web::Query<PaginationQuery>,
// ) -> impl Responder {
//     let limit = query.limit.unwrap_or(20).clamp(1, 100);
//     let offset = query.offset.unwrap_or(0).max(0);

//     match User::list_contacts_paginated(&db, query.user_id, limit, offset).await {
//         Ok(list) => HttpResponse::Ok().json(list),
//         Err(e) => ApiError::from(e).error_response(),
//     }
// }
