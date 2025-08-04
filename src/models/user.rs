use std::env;

use anyhow::{Ok, Result};
use redis::Commands;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, prelude::Type};

use crate::{
    database::Database,
    models::{contact::Contact, user_info::UserInfo},
    utils::{
        jwt::generate_jwt_token,
        password_hashing,
        redis::{Redis, Token},
    },
};

#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
pub struct User {
    pub id: Option<i32>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub user_info: UserInfo,
    pub password: Option<String>,
    pub first_login: Option<bool>,
    pub user_group: Option<UserGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub enum UserGroup {
    Agent,   // Üzletkötő
    Manager, // Menedzser
    Leader,  // Hálózati igazgató
}

impl From<String> for UserGroup {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Leader" => UserGroup::Leader,
            "Manager" => UserGroup::Manager,
            _ => UserGroup::Agent,
        }
    }
}

#[derive(Serialize)]
pub enum SignInResult {
    UserToken(String),
    FirstLoginToken(String),
}

impl User {
    pub async fn new(db: &Database, new_user: User) -> Result<()> {
        // Check for required fields
        if new_user.email.is_none()
            || new_user.user_info.full_name.is_none()
            || new_user.user_info.phone_number.is_none()
            || new_user.user_info.hufa_code.is_none()
            || new_user.user_info.agent_code.is_none()
            || new_user.password.is_none()
        {
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

        let user_id = sqlx::query!(
            "INSERT INTO users(email, username, password) VALUES($1, $2, $3) RETURNING id",
            new_user.email,
            new_user.username,
            hashed_password
        )
        .fetch_one(&db.pool)
        .await?;

        sqlx::query!(
            "INSERT INTO user_info(user_id, full_name, phone_number, hufa_code, agent_code) VALUES($1, $2, $3, $4, $5)",
            user_id.id,
            new_user.user_info.full_name,
            new_user.user_info.phone_number,
            new_user.user_info.hufa_code,
            new_user.user_info.agent_code
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn sign_in_via_username(db: &Database, user: User) -> Result<SignInResult> {
        let user_data = sqlx::query!(
            "SELECT id, username, password, first_login FROM users WHERE username = $1",
            user.username
        )
        .fetch_optional(&db.pool)
        .await?;

        let Some(hashed_user) = &user_data else {
            return Err(anyhow::anyhow!("User not found"));
        };

        if password_hashing::verify_password(&user.password.unwrap(), &hashed_user.password) {
            if hashed_user.first_login {
                let token = Self::create_first_login_token(db, hashed_user.id).await?;

                return Ok(SignInResult::FirstLoginToken(token));
            }

            Ok(SignInResult::UserToken(
                generate_jwt_token(hashed_user.id as usize, env::var("AUTH_SECRET").unwrap()).await,
            ))
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
            UserGroup::Leader => Ok(true),
            UserGroup::Manager => Ok(true),
            _ => Ok(false),
        }
    }

    pub async fn get_all(db: &Database, user_id: i32) -> Result<Vec<User>> {
        if !Self::is_user_exists_by_id(db, user_id).await? {
            return Err(anyhow::anyhow!("User not exists"));
        }

        let user = sqlx::query!("SELECT user_group FROM users WHERE id = $1", user_id)
            .fetch_one(&db.pool)
            .await?;

        if let UserGroup::Leader = UserGroup::from(user.user_group) {
            let user_info = sqlx::query!("SELECT * FROM user_info")
                .fetch_all(&db.pool)
                .await?;

            let users: Vec<User> = sqlx::query!("SELECT * FROM users")
                .fetch_all(&db.pool)
                .await?
                .into_iter()
                .map(|user| {
                    let user_info = user_info.iter().find(|info| info.user_id == user.id);

                    User {
                        id: Some(user.id),
                        email: Some(user.email),
                        username: Some(user.username),
                        user_info: UserInfo {
                            id: Some(user_info.unwrap().id.clone()),
                            full_name: Some(user_info.unwrap().full_name.clone()),
                            phone_number: Some(user_info.unwrap().phone_number.clone()),
                            hufa_code: Some(user_info.unwrap().hufa_code.clone()),
                            agent_code: Some(user_info.unwrap().agent_code.clone()),
                        },
                        user_group: Some(UserGroup::from(user.user_group)),
                        ..Default::default()
                    }
                })
                .collect();

            return Ok(users);
        }

        Err(anyhow::anyhow!("User has no permission for that!"))
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

    pub async fn first_login(db: &Database, new_password: String, token: String) -> Result<String> {
        let mut redis_con = db.redis.get_connection().unwrap();
        let user_id = Redis::get_user_id_by_token(&mut redis_con, &token)?;
        println!("{user_id}");

        if !Self::is_user_exists_by_id(db, user_id).await? {
            return Err(anyhow::anyhow!("User not exists"));
        }
        let hashed_password = password_hashing::hash_password(&new_password);
        let _ = sqlx::query!(
            "UPDATE users
             SET password = $2, first_login = False
             WHERE id = $1",
            user_id,
            hashed_password
        )
        .execute(&db.pool)
        .await?;

        let user_token =
            generate_jwt_token(user_id as usize, env::var("AUTH_SECRET").unwrap()).await;

        redis_con.del::<_, String>(token)?;

        Ok(user_token)
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

    async fn create_first_login_token(db: &Database, user_id: i32) -> Result<String> {
        let mut redis_con = db.redis.get_connection().unwrap();

        let token = Token::generate_token();
        let _ = Redis::set_token_to_user(&mut redis_con, user_id as u32, &token, 120)?;

        Ok(token)
    }
}
