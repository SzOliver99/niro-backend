use actix_web::{HttpResponse, Responder, Scope, web};
use serde::Deserialize;

use crate::{database::Database, models::customer::Customer};

pub fn customer_scope() -> Scope {
    web::scope("/customer").route("/create", web::post().to(create_customer))
}

#[derive(Deserialize)]
struct CustomerJson {
    email: String,
    phone_number: String,
    user_id: i32,
}
async fn create_customer(data: web::Json<CustomerJson>) -> impl Responder {
    let db = Database::create_connection().await.unwrap();
    let customer = Customer {
        id: None,
        email: Some(data.email.clone()),
        phone_number: Some(data.phone_number.clone()),
        user_id: Some(data.user_id),
    };

    match Customer::new(&db, customer).await {
        Ok(_) => HttpResponse::Created().json("Registration successful!"),
        Err(e) => HttpResponse::InternalServerError().json(format!("An error occurred: {}", e)),
    }
}
