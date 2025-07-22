use std::env;

use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, prelude::Type};

use crate::{
    database::Database,
    models::customer::Customer,
    utils::{jwt::generate_jwt_token, password_hashing},
};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Option<i32>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub group: Option<UserGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[sqlx(type_name = "text")]
pub enum UserGroup {
    User,
    Admin,
}

impl From<&str> for UserGroup {
    fn from(value: &str) -> Self {
        match value {
            "Admin" => UserGroup::Admin,
            _ => UserGroup::User,
        }
    }
}

impl User {
    pub async fn new(db: &Database, new_user: User) -> Result<()> {
        // Check for required fields
        if new_user.email.is_none() || new_user.username.is_none() || new_user.password.is_none() {
            return Err(anyhow::anyhow!(
                "All fields (email, username, password) are required"
            ));
        }

        if Self::is_user_exists(db, &new_user).await? {
            return Err(anyhow::anyhow!(
                "User with this email or username already exists"
            ));
        }

        let hashed_password = password_hashing::hash_password(&new_user.password.unwrap());

        let _user_id = sqlx::query!(
            "INSERT INTO users(email, username, password) VALUES($1, $2, $3) RETURNING id",
            new_user.email,
            new_user.username,
            hashed_password
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn sign_in_via_username(db: &Database, user: User) -> Result<String> {
        let user_data = sqlx::query!(
            r#"SELECT id, password FROM users WHERE username = $1"#,
            user.username
        )
        .fetch_optional(&db.pool)
        .await?;

        let Some(hashed_user) = &user_data else {
            return Err(anyhow::anyhow!("User not found"));
        };

        if password_hashing::verify_password(&user.password.unwrap(), &hashed_user.password) {
            Ok(generate_jwt_token(
                user_data.unwrap().id as usize,
                env::var("AUTH_SECRET").unwrap(),
            )
            .await)
        } else {
            Err(anyhow::anyhow!("Incorrect password"))
        }
    }

    pub async fn get_customers_by_id(db: &Database, user_id: i32) -> Result<Vec<Customer>> {
        if !Self::is_user_exists_by_id(db, user_id).await? {
            return Err(anyhow::anyhow!("User not exists"));
        }

        let customers = sqlx::query_as!(
            Customer,
            "SELECT * FROM customers WHERE user_id = $1",
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        Ok(customers)
    }
}

impl User {
    async fn is_user_exists(db: &Database, user: &User) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id from users
             WHERE email = $1 OR username = $2",
            user.email,
            user.username
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }

    async fn is_user_exists_by_id(db: &Database, user_id: i32) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id from users
             WHERE id = $1",
            user_id
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }
}
