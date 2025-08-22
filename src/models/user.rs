use std::env;

use anyhow::{Ok, Result};
use redis::Commands;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, prelude::Type};

use crate::{
    database::Database,
    models::{
        contact::Contact,
        dto::{ContactDto, ManagerNameDto, UserInfoDto, UserWithInfoDto},
        user_info::UserInfo,
    },
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
    pub info: UserInfo,
    pub password: Option<String>,
    pub first_login: Option<bool>,
    pub user_role: Option<UserRole>,
    pub manager_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub enum UserRole {
    Agent,   // Üzletkötő
    Manager, // Menedzser
    Leader,  // Hálózati igazgató
}

impl From<String> for UserRole {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Leader" => UserRole::Leader,
            "Manager" => UserRole::Manager,
            _ => UserRole::Agent,
        }
    }
}

#[derive(Serialize)]
pub enum SignInResult {
    UserToken(String),
    FirstLoginToken(String),
}

impl User {
    pub async fn create(db: &Database, user_id: i32, new_user: User) -> Result<()> {
        if new_user.email.is_none()
            || new_user.username.is_none()
            || new_user.password.is_none()
            || new_user.info.full_name.is_none()
            || new_user.info.phone_number.is_none()
            || new_user.info.hufa_code.is_none()
            || new_user.info.agent_code.is_none()
        {
            return Err(anyhow::anyhow!(
                "All fields (email, username, password, full_name, phone_number, hufa_code, agent_code) are required"
            ));
        }

        let user_role = User::get_role(db, user_id).await?;
        if !matches!(user_role, UserRole::Leader) {
            return Err(anyhow::anyhow!("User no permission to create agent"));
        }

        if User::is_exists(db, &new_user).await? {
            return Err(anyhow::anyhow!(
                "User with this email or username already exists"
            ));
        }

        let hashed_password = password_hashing::hash_password(&new_user.password.unwrap());

        let mut tx = db.pool.begin().await?;
        let user_id = sqlx::query!(
            "INSERT INTO users(email, username, password, manager_id) VALUES($1, $2, $3, $4) RETURNING id",
            new_user.email,
            new_user.username,
            hashed_password,
            new_user.manager_id
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO user_info(user_id, full_name, phone_number, hufa_code, agent_code)
             VALUES($1, $2, $3, $4, $5)",
            user_id.id,
            new_user.info.full_name,
            new_user.info.phone_number,
            new_user.info.hufa_code,
            new_user.info.agent_code
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn sign_in_with_username(db: &Database, user: User) -> Result<SignInResult> {
        let user_data = sqlx::query!(
            "SELECT id as \"id!\", username, password, first_login FROM users WHERE username = $1",
            user.username
        )
        .fetch_optional(&db.pool)
        .await?;

        let Some(hashed_user) = &user_data else {
            return Err(anyhow::anyhow!("User not found"));
        };

        if password_hashing::verify_password(&user.password.unwrap(), &hashed_user.password) {
            if hashed_user.first_login {
                let token = User::create_first_login_token(db, hashed_user.id).await?;

                return Ok(SignInResult::FirstLoginToken(token));
            }

            Ok(SignInResult::UserToken(
                generate_jwt_token(hashed_user.id as usize, env::var("AUTH_SECRET").unwrap()).await,
            ))
        } else {
            Err(anyhow::anyhow!("Incorrect password"))
        }
    }

    pub async fn get_all(db: &Database, user_id: i32) -> Result<Vec<UserWithInfoDto>> {
        if !User::is_exists_by_id(db, user_id).await? {
            return Err(anyhow::anyhow!("Invalid user_id"));
        }

        let user = sqlx::query!("SELECT user_role FROM users WHERE id = $1", user_id)
            .fetch_one(&db.pool)
            .await?;

        if let UserRole::Leader = UserRole::from(user.user_role) {
            let rows = sqlx::query!(
                r#"
                SELECT u.id               AS user_id,
                       u.email            AS user_email,
                       u.username         AS user_username,
                       u.user_role        AS user_user_role,
                       u.manager_id       AS user_manager_id,
                       ui.id              AS ui_id,
                       ui.full_name       AS ui_full_name,
                       ui.phone_number    AS ui_phone_number,
                       ui.hufa_code       AS ui_hufa_code,
                       ui.agent_code      AS ui_agent_code
                FROM users u
                JOIN user_info ui ON ui.user_id = u.id
                ORDER BY CASE u.user_role 
                    WHEN 'Leader' THEN 1
                    WHEN 'Manager' THEN 2
                    WHEN 'Agent' THEN 3
                    ELSE 4
                END;
                "#
            )
            .fetch_all(&db.pool)
            .await?;

            let users: Vec<UserWithInfoDto> = rows
                .into_iter()
                .map(|row| UserWithInfoDto {
                    id: row.user_id,
                    email: row.user_email,
                    username: row.user_username,
                    role: UserRole::from(row.user_user_role),
                    info: UserInfoDto {
                        id: row.ui_id,
                        full_name: row.ui_full_name,
                        phone_number: row.ui_phone_number,
                        hufa_code: row.ui_hufa_code,
                        agent_code: row.ui_agent_code,
                    },
                    manager_id: row.user_manager_id,
                })
                .collect();

            return Ok(users);
        }

        Err(anyhow::anyhow!("User has no permission for that!"))
    }

    pub async fn get_info_by_id(db: &Database, user_id: i32) -> Result<UserInfoDto> {
        if !User::is_exists_by_id(db, user_id).await? {
            return Err(anyhow::anyhow!("Invalid user_id"));
        }

        let user_info = sqlx::query!(
            "SELECT id, full_name, phone_number, hufa_code, agent_code FROM user_info WHERE user_id = $1",
            user_id
        )
            .fetch_one(&db.pool)
            .await?;

        Ok(UserInfoDto {
            id: user_info.id,
            full_name: user_info.full_name,
            phone_number: user_info.phone_number,
            hufa_code: user_info.hufa_code,
            agent_code: user_info.agent_code,
        })
    }

    pub async fn modify_info(db: &Database, user: User) -> Result<()> {
        if !User::is_exists_by_id(db, user.id.unwrap()).await? {
            return Err(anyhow::anyhow!("Invalid user_id"));
        }

        let mut tx = db.pool.begin().await?;
        sqlx::query!(
            "UPDATE users
             SET email = $2
             WHERE id = $1",
            user.id,
            user.email
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE user_info
             SET full_name = $2, phone_number = $3, hufa_code = $4, agent_code = $5
             WHERE user_id = $1",
            user.id,
            user.info.full_name,
            user.info.phone_number,
            user.info.hufa_code,
            user.info.agent_code
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn modify_manager(db: &Database, user: User) -> Result<()> {
        if !User::is_exists_by_id(db, user.id.unwrap()).await? {
            return Err(anyhow::anyhow!("Invalid user_id"));
        }

        if let Some(manager_id) = user.manager_id {
            sqlx::query!(
                "UPDATE users
                 SET manager_id = $2, user_role = DEFAULT
                 WHERE id = $1",
                user.id,
                manager_id
            )
            .execute(&db.pool)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE users
                 SET manager_id = NULL, user_role = 'Manager'
                 WHERE id = $1",
                user.id
            )
            .execute(&db.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_contacts_by_id(db: &Database, user_id: i32) -> Result<Vec<Contact>> {
        if !User::is_exists_by_id(db, user_id).await? {
            return Err(anyhow::anyhow!("Invalid user_id"));
        }

        let contacts = sqlx::query_as!(
            Contact,
            "SELECT id, email, first_name, last_name, phone_number, user_id FROM contacts WHERE user_id = $1",
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        Ok(contacts)
    }

    pub async fn list_contacts_paginated(
        db: &Database,
        user_id: i32,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ContactDto>> {
        if !User::is_exists_by_id(db, user_id).await? {
            return Err(anyhow::anyhow!("Invalid user_id"));
        }

        let rows = sqlx::query!(
            r#"
            SELECT c.id, c.email, c.first_name, c.last_name, c.phone_number
            FROM contacts c
            JOIN users u ON u.id = c.user_id
            WHERE c.user_id = $1
            ORDER BY c.id
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&db.pool)
        .await?;

        let result: Vec<ContactDto> = rows
            .into_iter()
            .map(|r| ContactDto {
                id: r.id,
                email: r.email,
                first_name: r.first_name,
                last_name: r.last_name,
                phone_number: r.phone_number,
            })
            .collect();

        Ok(result)
    }

    pub async fn complete_first_login(
        db: &Database,
        new_password: String,
        token: String,
    ) -> Result<String> {
        let mut redis_con = db.redis.get_connection().unwrap();
        let user_id = Redis::get_user_id_by_token(&mut redis_con, &token)?;

        if !User::is_exists_by_id(db, user_id).await? {
            return Err(anyhow::anyhow!("Invalid user_id"));
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

    pub async fn get_manager_group(db: &Database, user_id: i32) -> Result<Vec<UserWithInfoDto>> {
        let rows = sqlx::query!(
            r#"
            SELECT  u.id               AS user_id,
                    u.email            AS user_email,
                    u.username         AS user_username,
                    u.user_role        AS user_user_role,
                    u.manager_id       AS user_manager_id,
                    ui.id              AS ui_id,
                    ui.full_name       AS ui_full_name,
                    ui.phone_number    AS ui_phone_number,
                    ui.hufa_code       AS ui_hufa_code,
                    ui.agent_code      AS ui_agent_code
            FROM users u
            JOIN user_info ui ON ui.user_id = u.id
            WHERE u.manager_id = $1
            "#,
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        let users: Vec<UserWithInfoDto> = rows
            .into_iter()
            .map(|row| UserWithInfoDto {
                id: row.user_id,
                email: row.user_email,
                username: row.user_username,
                role: UserRole::from(row.user_user_role),
                info: UserInfoDto {
                    id: row.ui_id,
                    full_name: row.ui_full_name,
                    phone_number: row.ui_phone_number,
                    hufa_code: row.ui_hufa_code,
                    agent_code: row.ui_agent_code,
                },
                manager_id: row.user_manager_id,
            })
            .collect();

        Ok(users)
    }
}

impl User {
    pub async fn get_role(db: &Database, user_id: i32) -> Result<UserRole> {
        let user = sqlx::query!("SELECT user_role FROM users WHERE id = $1", user_id)
            .fetch_one(&db.pool)
            .await?;

        Ok(UserRole::from(user.user_role))
    }

    async fn is_exists(db: &Database, user: &User) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM users
             WHERE email = $1 OR username = $2",
            user.email,
            user.username
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }

    async fn is_exists_by_id(db: &Database, user_id: i32) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM users
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

impl UserRole {
    pub async fn get_manager(db: &Database) -> Result<Vec<ManagerNameDto>> {
        let rows = sqlx::query!(
            r#"
            SELECT u.id AS user_id, ui.full_name AS full_name, user_role
            FROM users u
            JOIN user_info ui ON ui.user_id = u.id
            WHERE u.user_role = 'Manager' OR u.user_role = 'Leader'
            ORDER BY u.user_role ASC, ui.full_name
            "#
        )
        .fetch_all(&db.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ManagerNameDto {
                id: r.user_id,
                full_name: r.full_name,
                user_role: r.user_role,
            })
            .collect())
    }
}
