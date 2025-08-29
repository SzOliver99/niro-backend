use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use serde::{Deserialize, Serialize};

use crate::{
    database::Database,
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
        .route("/list", web::get().to(get_users))
        .route("/list/by-id", web::post().to(get_users_by_id))
        .route("/sub-users", web::post().to(get_user_sub_users))
        .route("/manager", web::put().to(modify_user_manager))
        .route("/delete", web::delete().to(delete_user))
        .route(
            "/first-login/complete",
            web::post().to(finish_user_first_login),
        )
        .route("/info", web::get().to(get_user_informations_by_id))
        .route("/info", web::put().to(modify_user_info))
        .route("/managers/list", web::post().to(get_manager_names))
        .route("/protected", web::get().to(protected_route))
}

#[derive(Deserialize, Debug)]
struct UserJson {
    email: Option<String>,
    username: Option<String>,
    password: Option<String>,
    info: UserInfo,
    manager_id: Option<i32>,
}

#[derive(Deserialize, Debug)]
struct SignInJson {
    username: String,
    password: String,
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
        manager_id: data.manager_id.clone(),
        ..Default::default()
    };

    match User::create(&web_data.db, new_user).await {
        Ok(_) => HttpResponse::Created().json("Registration successful!"),
        Err(e) => ApiError::from(e).error_response(),
    }
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

async fn get_users_by_id(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<Option<i32>>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    let user_id = match data.0 {
        Some(id) => id,
        None => auth_token.id as i32,
    };

    match User::get_users_by_id(&web_data.db, user_id).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize, Clone, Debug)]
struct ModifyUserInfoJson {
    id: Option<i32>,
    email: String,
    info: UserInfo,
}
async fn modify_user_info(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<ModifyUserInfoJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    let user = User {
        id: if data.id.is_some() {
            data.id
        } else {
            Some(auth_token.id as i32)
        },
        email: Some(data.email.clone()),
        info: data.info.clone(),
        ..Default::default()
    };

    match User::modify_info(&web_data.db, user).await {
        Ok(_) => HttpResponse::Ok().json({}),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize, Clone, Debug)]
struct ModifyUserManagerJson {
    id: i32,
    manager_id: Option<i32>,
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
        id: Some(data.id),
        manager_id: data.manager_id,
        ..Default::default()
    };

    match User::modify_manager(&web_data.db, user).await {
        Ok(_) => HttpResponse::Ok().json({}),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn delete_user(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<i32>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Leader, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    let user = User {
        id: Some(data.0),
        ..Default::default()
    };

    match User::delete(&web_data.db, user).await {
        Ok(_) => HttpResponse::Ok().json({}),
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

async fn get_manager_names(
    web_data: web::Data<WebData>,
    data: web::Json<Option<i32>>,
) -> impl Responder {
    let user = User {
        id: data.0,
        ..Default::default()
    };

    match UserRole::get_managers(&web_data.db, user).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_user_sub_users(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<String>,
) -> impl Responder {
    match User::get_sub_users(&web_data.db, auth_token.id as i32, data.0).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize, Clone, Debug)]
struct FirstLoginJson {
    new_password: String,
    token: String,
}
async fn finish_user_first_login(
    web_data: web::Data<WebData>,
    data: web::Json<FirstLoginJson>,
) -> impl Responder {
    match User::complete_first_login(&web_data.db, data.new_password.clone(), data.token.clone())
        .await
    {
        Ok(token) => HttpResponse::Ok().json(token),
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
