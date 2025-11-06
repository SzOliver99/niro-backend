use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    extractors::authentication_token::AuthenticationToken,
    models::recruitment::Recruitment,
    models::user::{User, UserRole},
    utils::error::ApiError,
    web_data::WebData,
};

pub fn recruitment_scope() -> Scope {
    web::scope("/recruitment")
        .route("/create", web::post().to(create_recruitment))
        .route("/modify", web::put().to(modify_recruitment))
        .route("/get-all", web::get().to(get_recruitments))
        .route(
            "/{recruitment_uuid}",
            web::get().to(get_recruitment_by_uuid),
        )
        .route("/{recruitment_uuid}", web::delete().to(delete_recruitments))
}

#[derive(Deserialize, Clone)]
struct CreateRecruitmentJson {
    full_name: String,
    email: String,
    phone_number: String,
    description: String,
    created_by: String,
}
async fn create_recruitment(
    web_data: web::Data<WebData>,
    data: web::Json<CreateRecruitmentJson>,
) -> impl Responder {
    let r = Recruitment {
        full_name: Some(data.full_name.clone()),
        email: Some(data.email.clone()),
        phone_number: Some(data.phone_number.clone()),
        description: Some(data.description.clone()),
        created_by: Some(data.created_by.clone()),
        ..Default::default()
    };

    match Recruitment::create(&web_data.db, &web_data.key, &web_data.hmac_secret, r).await {
        Ok(_) => HttpResponse::Created().json("Jelentkező sikeresen létrehozva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize, Clone)]
struct ModifyRecruitmentJson {
    recruitment_uuid: Uuid,
    full_name: Option<String>,
    email: Option<String>,
    phone_number: Option<String>,
    description: Option<String>,
    created_by: Option<String>,
}
async fn modify_recruitment(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    data: web::Json<ModifyRecruitmentJson>,
) -> impl Responder {
    let r = Recruitment {
        full_name: data.full_name.clone(),
        email: data.email.clone(),
        phone_number: data.phone_number.clone(),
        description: data.description.clone(),
        created_by: data.created_by.clone(),
        ..Default::default()
    };

    match Recruitment::modify(
        &web_data.db,
        &web_data.key,
        &web_data.hmac_secret,
        data.recruitment_uuid,
        r,
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Jelentkező sikeresen módosítva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_recruitments(web_data: web::Data<WebData>, _: AuthenticationToken) -> impl Responder {
    match Recruitment::get_all(&web_data.db, &web_data.key).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_recruitment_by_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    recruitment_uuid: web::Path<Uuid>,
) -> impl Responder {
    match Recruitment::get_by_uuid(&web_data.db, &web_data.key, recruitment_uuid.into_inner()).await
    {
        Ok(rec) => HttpResponse::Ok().json(rec),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn delete_recruitments(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    recruitment_uuid: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Agent, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match Recruitment::delete(&web_data.db, recruitment_uuid.into_inner()).await {
        Ok(_) => HttpResponse::Created().json("Jelentkező sikeresen törölve!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}
