use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use serde::Deserialize;

use crate::{
    database::Database,
    extractors::authentication_token::AuthenticationToken,
    models::{
        customer::Customer,
        user::{User, UserRole},
    },
    utils::error::ApiError,
};

pub fn customer_scope() -> Scope {
    web::scope("/customer")
        .route("/create", web::post().to(create_customer))
        .route("/get-all", web::post().to(get_user_customers_by_id))
        .route("/change/user", web::post().to(change_customer_handler))
        .route("/delete", web::delete().to(delete_customer))
}

#[derive(Deserialize, Clone)]
struct CustomerJson {
    full_name: String,
    phone_number: String,
    address: String,
    email: String,
    user_id: i32,
}
async fn create_customer(db: web::Data<Database>, data: web::Json<CustomerJson>) -> impl Responder {
    let customer = Customer {
        full_name: Some(data.full_name.clone()),
        phone_number: Some(data.phone_number.clone()),
        address: Some(data.address.clone()),
        email: Some(data.email.clone()),
        user_id: Some(data.user_id),
        ..Default::default()
    };

    match Customer::create(&db, customer).await {
        Ok(_) => HttpResponse::Created().json("Creation was successful!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_user_customers_by_id(
    db: web::Data<Database>,
    auth_token: AuthenticationToken,
    data: web::Json<i32>,
) -> impl Responder {
    if data.0 != auth_token.id as i32 {
        if let Err(e) = User::require_role(&db, UserRole::Manager, auth_token.id as i32).await {
            return ApiError::from(e).error_response();
        }
    }

    match Customer::get_all(&db, data.0).await {
        Ok(customers) => HttpResponse::Created().json(customers),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize)]
struct ChangeCustomersHandlerJson {
    user_full_name: String,
    customer_ids: Vec<i32>,
}
async fn change_customer_handler(
    db: web::Data<Database>,
    auth_token: AuthenticationToken,
    data: web::Json<ChangeCustomersHandlerJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&db, UserRole::Leader, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match Customer::change_handler(&db, data.user_full_name.clone(), data.customer_ids.clone())
        .await
    {
        Ok(_) => HttpResponse::Created().json("Registration successful!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn delete_customer(
    db: web::Data<Database>,
    auth_token: AuthenticationToken,
    data: web::Json<Vec<i32>>,
) -> impl Responder {
    if let Err(e) = User::require_role(&db, UserRole::Agent, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match Customer::delete(&db, data.0).await {
        Ok(_) => HttpResponse::Created().json("Registration successful!"),
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
//     db: web::Data<Database>,
//     query: web::Query<PaginationQuery>,
// ) -> impl Responder {
//     let limit = query.limit.unwrap_or(20).clamp(1, 100);
//     let offset = query.offset.unwrap_or(0).max(0);

//     match User::list_contacts_paginated(&db, query.user_id, limit, offset).await {
//         Ok(list) => HttpResponse::Ok().json(list),
//         Err(e) => ApiError::from(e).error_response(),
//     }
// }
