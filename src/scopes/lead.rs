use actix_web::{web, HttpResponse, Responder, ResponseError, Scope};
use serde::Deserialize;

use crate::{database::Database, models::lead::Lead, utils::error::ApiError};

pub fn lead_scope() -> Scope {
    web::scope("/customer").route("/create", web::post().to(create_lead))
}

#[derive(Deserialize, Clone)]
struct LeadJson {
    full_name: String,
    phone_number: String,
    address: String,
    email: String,
    user_id: i32,
}
async fn create_lead(db: web::Data<Database>, data: web::Json<LeadJson>) -> impl Responder {
    let lead = Lead {
        lead_type: todo!(),
        inquiry_type: todo!(),
        lead_status: todo!(),
        handle_at: todo!(),
        ..Default::default()
    };

    match Lead::create(&db, lead).await {
        Ok(_) => HttpResponse::Created().json("Registration successful!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}
