use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    extractors::authentication_token::AuthenticationToken,
    models::{
        recommendation::CustomerRecommendation,
        user::{User, UserRole},
    },
    utils::error::ApiError,
    web_data::WebData,
};

pub fn recommendation_scope() -> Scope {
    web::scope("/recommendation")
        .route("/create/{user_uuid}", web::post().to(create_recommendation))
        .route(
            "/modify/{recommendation_uuid}",
            web::put().to(modify_recommendation),
        )
        .route(
            "/get-all/{user_uuid}",
            web::get().to(get_recommendations_by_user_uuid),
        )
        .route(
            "/{recommendation_uuid}",
            web::get().to(get_recommendation_by_uuid),
        )
        .route("/change/user", web::put().to(change_recommendation_handler))
        .route("/delete", web::delete().to(delete_recommendations))
}

#[derive(Deserialize, Clone)]
struct CreateRecommendationJson {
    full_name: String,
    phone_number: String,
    city: String,
    referral_name: String,
    created_by: String,
}
async fn create_recommendation(
    web_data: web::Data<WebData>,
    data: web::Json<CreateRecommendationJson>,
    user_uuid: web::Path<Uuid>,
) -> impl Responder {
    let rec = CustomerRecommendation {
        full_name: Some(data.full_name.clone()),
        phone_number: Some(data.phone_number.clone()),
        city: Some(data.city.clone()),
        referral_name: Some(data.referral_name.clone()),
        created_by: Some(data.created_by.clone()),
        ..Default::default()
    };

    match CustomerRecommendation::create(
        &web_data.db,
        &web_data.key,
        &web_data.hmac_secret,
        user_uuid.into_inner(),
        rec,
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Ajánlás sikeresen létrehozva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize, Clone)]
struct ModifyRecommendationJson {
    full_name: Option<String>,
    phone_number: Option<String>,
    city: Option<String>,
    referral_name: Option<String>,
    created_by: Option<String>,
}
async fn modify_recommendation(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    data: web::Json<ModifyRecommendationJson>,
    recommendation_uuid: web::Path<Uuid>,
) -> impl Responder {
    let rec = CustomerRecommendation {
        full_name: data.full_name.clone(),
        phone_number: data.phone_number.clone(),
        city: data.city.clone(),
        referral_name: data.referral_name.clone(),
        created_by: data.created_by.clone(),
        ..Default::default()
    };

    match CustomerRecommendation::modify(
        &web_data.db,
        &web_data.key,
        &web_data.hmac_secret,
        recommendation_uuid.into_inner(),
        rec,
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Ajánlás sikeresen módosítva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_recommendations_by_user_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    user_uuid: web::Path<Uuid>,
) -> impl Responder {
    match CustomerRecommendation::get_all(&web_data.db, &web_data.key, user_uuid.into_inner()).await
    {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_recommendation_by_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    recommendation_uuid: web::Path<Uuid>,
) -> impl Responder {
    match CustomerRecommendation::get_by_uuid(
        &web_data.db,
        &web_data.key,
        recommendation_uuid.into_inner(),
    )
    .await
    {
        Ok(rec) => HttpResponse::Ok().json(rec),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize)]
struct ChangeRecommendationsHandlerJson {
    user_full_name: String,
    recommendation_uuids: Vec<Uuid>,
}
async fn change_recommendation_handler(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<ChangeRecommendationsHandlerJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Leader, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match CustomerRecommendation::change_handler(
        &web_data.db,
        data.user_full_name.clone(),
        data.recommendation_uuids.clone(),
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Ajánlás(ok)ért felelős üzletkötő megváltoztatva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn delete_recommendations(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<Vec<Uuid>>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Agent, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match CustomerRecommendation::delete(&web_data.db, data.0).await {
        Ok(_) => HttpResponse::Created().json("Ajánlás(ok) sikeresen törölve!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}
