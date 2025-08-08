use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use serde::Deserialize;

use crate::{
    database::Database,
    models::{contact::Contact, user::User},
    utils::error::ApiError,
};

pub fn contact_scope() -> Scope {
    web::scope("/contact")
        .route("/create", web::post().to(create_contact))
        .route("/list", web::get().to(list_contacts))
}

#[derive(Deserialize)]
struct ContactJson {
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    phone_number: Option<String>,
    user_id: Option<i32>,
}
async fn create_contact(db: web::Data<Database>, data: web::Json<ContactJson>) -> impl Responder {
    let contact = Contact {
        id: None,
        email: data.email.clone(),
        first_name: data.first_name.clone(),
        last_name: data.last_name.clone(),
        phone_number: data.phone_number.clone(),
        user_id: data.user_id,
    };

    match Contact::new(&db, contact).await {
        Ok(_) => HttpResponse::Created().json("Registration successful!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize)]
struct PaginationQuery {
    user_id: i32,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn list_contacts(
    db: web::Data<Database>,
    query: web::Query<PaginationQuery>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    match User::list_contacts_paginated(&db, query.user_id, limit, offset).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}
