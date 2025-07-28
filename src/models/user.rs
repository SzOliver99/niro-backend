use std::env;

use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, prelude::Type};

use crate::{
    database::Database,
    models::contact::Contact,
    utils::{jwt::generate_jwt_token, password_hashing},
};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Option<i32>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub password: Option<String>,
    pub first_login: Option<bool>,
    pub user_group: Option<UserGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub enum UserGroup {
    Agent,
    Admin,
    Master,
}

impl From<String> for UserGroup {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Master" => UserGroup::Master,
            "Admin" => UserGroup::Admin,
            _ => UserGroup::Agent,
        }
    }
}

impl User {
    pub async fn new(db: &Database, new_user: User) -> Result<()> {
        // Check for required fields
        if new_user.email.is_none() || new_user.full_name.is_none() || new_user.password.is_none() {
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
            "INSERT INTO users(email, username, full_name, password) VALUES($1, $2, $3, $4) RETURNING id",
            new_user.email,
            new_user.username,
            new_user.full_name,
            hashed_password
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn sign_in_via_username(db: &Database, user: User) -> Result<String> {
        let user_data = sqlx::query!(
            "SELECT id, username, password FROM users WHERE username = $1",
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

    pub async fn is_any_permission(db: &Database, user_id: i32) -> Result<bool> {
        if !Self::is_user_exists_by_id(db, user_id).await? {
            return Err(anyhow::anyhow!("User not exists"));
        }

        let user_role = sqlx::query!("SELECT user_group FROM users WHERE id = $1", user_id)
            .fetch_one(&db.pool)
            .await?;

        match UserGroup::from(user_role.user_group) {
            UserGroup::Master => Ok(true),
            UserGroup::Admin => Ok(true),
            _ => Ok(false),
        }
    }

    pub async fn get_contacts_by_id(db: &Database, user_id: i32) -> Result<Vec<Contact>> {
        if !Self::is_user_exists_by_id(db, user_id).await? {
            return Err(anyhow::anyhow!("User not exists"));
        }

        let contacts = sqlx::query_as!(
            Contact,
            "SELECT * FROM contacts WHERE user_id = $1",
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        Ok(contacts)
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
