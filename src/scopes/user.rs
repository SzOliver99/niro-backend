use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    extractors::authentication_token::AuthenticationToken,
    models::{
        user::{User, UserRole},
        user_info::UserInfo,
    },
    utils::error::ApiError,
    web_data::WebData,
};

pub fn user_scope() -> Scope {
    web::scope("/user")
        .route("/register", web::post().to(create_user))
        .route("/login/username", web::post().to(sign_in_via_username))
        .route("/role", web::get().to(get_user_role))
        .route("/get-all", web::get().to(get_users))
        .route("/get/{user_uuid}", web::get().to(get_users_by_uuid))
        .route("/sub-users/{min_role}", web::get().to(get_user_sub_users))
        .route("/managers", web::post().to(get_managers))
        .route("/manager", web::put().to(modify_user_manager))
        .route("/info", web::get().to(get_user_informations_by_id))
        .route("/{user_uuid}/info", web::put().to(modify_user_info))
        .route("/delete/{user_uuid}", web::delete().to(delete_user))
        .route("/protected", web::get().to(protected_route))
}

#[derive(Deserialize, Debug)]
struct UserJson {
    email: Option<String>,
    username: Option<String>,
    password: Option<String>,
    info: UserInfo,
    manager_uuid: Option<Uuid>,
}

async fn create_user(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<UserJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Leader, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    let new_user = User {
        email: data.email.clone(),
        username: data.username.clone(),
        password: data.password.clone(),
        info: UserInfo {
            full_name: data.info.full_name.clone(),
            phone_number: data.info.phone_number.clone(),
            hufa_code: data.info.hufa_code.clone(),
            agent_code: data.info.agent_code.clone(),
            ..Default::default()
        },
        manager_uuid: data.manager_uuid,
        ..Default::default()
    };

    match User::create(&web_data.db, new_user).await {
        Ok(_) => HttpResponse::Created().json("Registration successful!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize, Debug)]
struct SignInJson {
    username: String,
    password: String,
}
async fn sign_in_via_username(
    web_data: web::Data<WebData>,
    data: web::Json<SignInJson>,
) -> impl Responder {
    let user = User {
        username: Some(data.username.clone()),
        password: Some(data.password.clone()),
        ..Default::default()
    };

    match User::sign_in_with_username(&web_data.db, user).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_users(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Leader, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match User::get_users(&web_data.db, auth_token.id as i32).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_users_by_uuid(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    user_uuid: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match User::get_users_by_id(&web_data.db, user_uuid.into_inner()).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize, Clone, Debug)]
struct ModifyUserInfoJson {
    email: String,
    info: UserInfo,
}
async fn modify_user_info(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<ModifyUserInfoJson>,
    user_uuid: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    let user = User {
        email: Some(data.email.clone()),
        info: data.info.clone(),
        ..Default::default()
    };

    match User::modify_info(&web_data.db, user_uuid.into_inner(), user).await {
        Ok(_) => HttpResponse::Ok().json("Sikeresen megváltoztattad a felhasználó adatait!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize, Clone, Debug)]
struct ModifyUserManagerJson {
    user_uuid: Uuid,
    manager_uuid: Option<Uuid>,
}
async fn modify_user_manager(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<ModifyUserManagerJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Leader, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    let user = User {
        manager_uuid: data.manager_uuid,
        ..Default::default()
    };

    match User::modify_manager(&web_data.db, data.user_uuid, user).await {
        Ok(_) => HttpResponse::Ok().json("Sikeresen megváltoztattad a felhasználó adatait!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn delete_user(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    user_uuid: web::Json<Uuid>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Leader, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match User::delete(&web_data.db, user_uuid.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json("Sikeresen kitörölted a felhasználót!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_user_informations_by_id(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
) -> impl Responder {
    match User::get_info_by_id(&web_data.db, auth_token.id as i32).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_user_role(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
) -> impl Responder {
    match User::get_role(&web_data.db, auth_token.id as i32).await {
        Ok(role) => HttpResponse::Ok().json(role),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_managers(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    data: web::Json<Option<Uuid>>,
) -> impl Responder {
    let user_id = match data.0 {
        Some(user_uuid) => User::get_id_by_uuid(&web_data.db, Some(user_uuid))
            .await
            .unwrap()
            .unwrap(),
        None => 0,
    };

    match UserRole::get_managers(&web_data.db, user_id).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_user_sub_users(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    min_role: web::Path<String>,
) -> impl Responder {
    match User::get_sub_users(&web_data.db, auth_token.id as i32, min_role.to_string()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Serialize)]
struct ProtectedResponse {
    message: String,
}
async fn protected_route(_auth_token: AuthenticationToken) -> impl Responder {
    HttpResponse::Ok().json(ProtectedResponse {
        message: "Sikeres hitelesítés".to_string(),
    })
}
