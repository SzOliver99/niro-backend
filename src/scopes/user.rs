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
};

pub fn user_scope() -> Scope {
    web::scope("/user")
        .route("/sign-up", web::post().to(create_user))
        .route("/sign-in/username", web::post().to(sign_in_via_username))
        .route("/role/get", web::get().to(get_user_role))
        .route("/get-all", web::get().to(get_users))
        .route("/manager/get-all", web::get().to(get_manager_group))
        .route("/manager/modify", web::put().to(modify_user_manager))
        .route("/terminate", web::delete().to(delete_user))
        .route(
            "/first-login/finish",
            web::post().to(finish_user_first_login),
        )
        .route("/info/get", web::get().to(get_user_informations_by_id))
        .route("/info/modify", web::put().to(modify_user_info))
        .route("/managers", web::post().to(get_manager_names))
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
    db: web::Data<Database>,
    auth_token: AuthenticationToken,
    data: web::Json<UserJson>,
) -> impl Responder {
    println!("{data:?}");
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

    match User::create(&db, auth_token.id as i32, new_user).await {
        Ok(_) => HttpResponse::Created().json("Registration successful!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn sign_in_via_username(
    db: web::Data<Database>,
    data: web::Json<SignInJson>,
) -> impl Responder {
    if data.username.trim().is_empty() || data.password.trim().is_empty() {
        return ApiError::Validation("username and password are required".into()).error_response();
    }
    let user = User {
        username: Some(data.username.clone()),
        password: Some(data.password.clone()),
        ..Default::default()
    };

    match User::sign_in_with_username(&db, user).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_users(db: web::Data<Database>, auth_token: AuthenticationToken) -> impl Responder {
    match User::get_all(&db, auth_token.id as i32).await {
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
    db: web::Data<Database>,
    auth_token: AuthenticationToken,
    data: web::Json<ModifyUserInfoJson>,
) -> impl Responder {
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

    match User::modify_info(&db, user).await {
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
    db: web::Data<Database>,
    _: AuthenticationToken,
    data: web::Json<ModifyUserManagerJson>,
) -> impl Responder {
    let user = User {
        id: Some(data.id),
        manager_id: data.manager_id,
        ..Default::default()
    };

    match User::modify_manager(&db, user).await {
        Ok(_) => HttpResponse::Ok().json({}),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn delete_user(
    db: web::Data<Database>,
    auth_token: AuthenticationToken,
    data: web::Json<i32>,
) -> impl Responder {
    let user = User {
        id: Some(data.0),
        ..Default::default()
    };

    match User::terminate_contact(&db, user).await {
        Ok(_) => HttpResponse::Ok().json({}),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_manager_group(
    db: web::Data<Database>,
    auth_token: AuthenticationToken,
) -> impl Responder {
    match User::get_manager_group(&db, auth_token.id as i32).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_user_informations_by_id(
    db: web::Data<Database>,
    auth_token: AuthenticationToken,
) -> impl Responder {
    match User::get_info_by_id(&db, auth_token.id as i32).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_user_role(db: web::Data<Database>, auth_token: AuthenticationToken) -> impl Responder {
    match User::get_role(&db, auth_token.id as i32).await {
        Ok(role) => HttpResponse::Ok().json(role),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_manager_names(
    db: web::Data<Database>,
    data: web::Json<Option<i32>>,
) -> impl Responder {
    let user = User {
        id: data.0,
        ..Default::default()
    };

    match UserRole::get_managers(&db, user).await {
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
    db: web::Data<Database>,
    data: web::Json<FirstLoginJson>,
) -> impl Responder {
    println!("{:?}", data);
    match User::complete_first_login(&db, data.new_password.clone(), data.token.clone()).await {
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
