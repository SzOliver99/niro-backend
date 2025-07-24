use actix_web::{HttpResponse, Responder, Scope, web};
use jsonwebtoken::TokenData;
use serde::Deserialize;

use crate::{
    database::Database, extractors::authentication_token::AuthenticationToken, models::user::User,
    utils::jwt::generate_jwt_token,
};

pub fn user_scope() -> Scope {
    web::scope("/user")
        .route("/sign-up", web::post().to(create_user))
        .route("/sign-in/username", web::post().to(sign_in_via_username))
        .route("/is-any-permission", web::get().to(is_user_any_permission))
        .route("/test", web::get().to(test))
}

#[derive(Deserialize, Debug)]
struct UserJson {
    email: Option<String>,
    username: Option<String>,
    full_name: Option<String>,
    password: Option<String>,
}

async fn create_user(data: web::Json<UserJson>) -> impl Responder {
    let db = Database::create_connection().await.unwrap();
    let user = User {
        id: None,
        email: data.email.clone(),
        username: data.username.clone(),
        full_name: data.full_name.clone(),
        password: data.password.clone(),
        user_group: None,
    };
    println!("{:?}", data);

    match User::new(&db, user).await {
        Ok(_) => HttpResponse::Created().json("Registration successful!"),
        Err(e) => HttpResponse::InternalServerError().json(format!("An error occurred: {}", e)),
    }
}

async fn sign_in_via_username(data: web::Json<UserJson>) -> impl Responder {
    let db = Database::create_connection().await.unwrap();
    let user = User {
        id: None,
        email: None,
        username: data.username.clone(),
        full_name: None,
        password: data.password.clone(),
        user_group: None,
    };

    match User::sign_in_via_username(&db, user).await {
        Ok(token) => HttpResponse::Created().json(token),
        Err(e) => HttpResponse::InternalServerError().json(format!("An error occurred: {}", e)),
    }
}

async fn is_user_any_permission(auth_token: AuthenticationToken) -> impl Responder {
    let db = Database::create_connection().await.unwrap();

    match User::is_any_permission(&db, auth_token.id as i32).await {
        Ok(token) => HttpResponse::Created().json(token),
        Err(e) => HttpResponse::InternalServerError().json(format!("An error occurred: {}", e)),
    }
}

async fn test() -> impl Responder {
    HttpResponse::Ok().json(generate_jwt_token(1usize, "sanyika".to_string()).await)
}
