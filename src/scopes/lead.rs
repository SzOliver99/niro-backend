use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use serde::Deserialize;

use crate::{database::Database, models::lead::Lead, utils::error::ApiError, web_data::WebData};

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
async fn create_lead(web_data: web::Data<WebData>, data: web::Json<LeadJson>) -> impl Responder {
    let lead = Lead {
        lead_type: todo!(),
        inquiry_type: todo!(),
        lead_status: todo!(),
        handle_at: todo!(),
        ..Default::default()
    };

    match Lead::create(&web_data.db, data.user_id, lead).await {
        Ok(_) => HttpResponse::Created().json("Registration successful!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}
