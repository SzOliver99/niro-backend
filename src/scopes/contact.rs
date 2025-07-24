use actix_web::{HttpResponse, Responder, Scope, web};
use serde::Deserialize;

use crate::{database::Database, models::contact::Contact};

pub fn contact_scope() -> Scope {
    web::scope("/contact").route("/create", web::post().to(create_contact))
}

#[derive(Deserialize)]
struct ContactJson {
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    phone_number: Option<String>,
    user_id: Option<i32>,
}
async fn create_contact(data: web::Json<ContactJson>) -> impl Responder {
    let db = Database::create_connection().await.unwrap();
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
        Err(e) => HttpResponse::InternalServerError().json(format!("An error occurred: {}", e)),
    }
}
