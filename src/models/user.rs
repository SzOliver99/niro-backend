use std::env;

use anyhow::{Ok, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::{FromRow, prelude::Type};
use uuid::Uuid;

use crate::{
    database::Database,
    models::{dto::ManagerNameDto, user_info::UserInfo},
    utils::{jwt::generate_jwt_token, password_hashing},
};

#[skip_serializing_none]
#[derive(Debug, Serialize, FromRow, Default)]
pub struct User {
    pub id: Option<i32>,
    pub uuid: Option<Uuid>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub info: UserInfo,
    pub password: Option<String>,
    pub user_role: Option<UserRole>,
    pub manager_uuid: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Type, PartialEq, Eq, PartialOrd, Ord)]
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
}

impl User {
    pub async fn get_id_by_uuid(db: &Database, user_uuid: Option<Uuid>) -> Result<Option<i32>> {
        let user = sqlx::query_scalar!("SELECT id FROM users WHERE uuid = $1", user_uuid)
            .fetch_optional(&db.pool)
            .await?;

        Ok(user)
    }

    pub async fn get_uuid_by_id(db: &Database, user_id: i32) -> Result<Option<Uuid>> {
        let user = sqlx::query!("SELECT uuid FROM users WHERE id = $1", user_id)
            .fetch_one(&db.pool)
            .await?;

        Ok(user.uuid)
    }

    pub async fn get_role(db: &Database, user_id: i32) -> Result<UserRole> {
        let user = sqlx::query!("SELECT user_role FROM users WHERE id = $1", user_id)
            .fetch_one(&db.pool)
            .await?;

        Ok(UserRole::from(user.user_role))
    }

    pub async fn require_role(db: &Database, min_role: UserRole, user_id: i32) -> Result<()> {
        let user_role = Self::get_role(db, user_id).await?;

        if user_role >= min_role {
            return Ok(());
        }
        Err(anyhow!("Ehez a folyamathoz nincs jogosultságod!"))
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
}

impl User {
    pub async fn create(db: &Database, new_user: User) -> Result<()> {
        if User::is_exists(db, &new_user).await? {
            return Err(anyhow!("Ez az e-mail cím vagy felhasználónév már létezik."));
        }

        let hashed_password = password_hashing::hash_password(&new_user.password.unwrap());

        println!("Manager UUID: {:?}", new_user.manager_uuid);
        let mut tx = db.pool.begin().await?;
        let user_id = sqlx::query!(
            "INSERT INTO users(email, username, password, user_role, manager_id) VALUES($1, $2, $3, $4, $5) RETURNING id",
            new_user.email,
            new_user.username,
            hashed_password,
            if new_user.manager_uuid.is_some() { "Agent" } else { "Manager" },
            Self::get_id_by_uuid(db, new_user.manager_uuid).await?
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
            "SELECT id as \"id!\", username, password FROM users WHERE username = $1",
            user.username
        )
        .fetch_optional(&db.pool)
        .await?;

        let Some(hashed_user) = &user_data else {
            return Err(anyhow!("Felhasználó nem található"));
        };

        if password_hashing::verify_password(&user.password.unwrap(), &hashed_user.password) {
            Ok(SignInResult::UserToken(
                generate_jwt_token(hashed_user.id as usize, env::var("AUTH_SECRET").unwrap()).await,
            ))
        } else {
            Err(anyhow!("Helytelen jelszó!"))
        }
    }

    pub async fn get_users(db: &Database, user_id: i32) -> Result<Vec<User>> {
        if !User::is_exists_by_id(db, user_id).await? {
            return Err(anyhow!("Felhasználó nem létezik"));
        }

        let rows = sqlx::query!(
            "SELECT u.uuid,
                    u.email,
                    u.username,
                    u.user_role,
                    m.uuid as manager_uuid,
                    ui.id              AS ui_id,
                    ui.full_name       AS ui_full_name,
                    ui.phone_number    AS ui_phone_number,
                    ui.hufa_code       AS ui_hufa_code,
                    ui.agent_code      AS ui_agent_code
              FROM users u
              JOIN user_info ui ON ui.user_id = u.id
              LEFT JOIN users m ON m.id = u.manager_id
              ORDER BY CASE u.user_role
                  WHEN 'Leader' THEN 1
                  WHEN 'Manager' THEN 2
                  WHEN 'Agent' THEN 3
              END;"
        )
        .fetch_all(&db.pool)
        .await?;

        let users = rows
            .into_iter()
            .map(|row| User {
                uuid: row.uuid,
                email: Some(row.email),
                username: Some(row.username),
                user_role: Some(UserRole::from(row.user_role)),
                info: UserInfo {
                    full_name: Some(row.ui_full_name),
                    phone_number: Some(row.ui_phone_number),
                    hufa_code: Some(row.ui_hufa_code),
                    agent_code: Some(row.ui_agent_code),
                    ..Default::default()
                },
                manager_uuid: row.manager_uuid,
                ..Default::default()
            })
            .collect();
        Ok(users)
    }

    pub async fn get_users_by_id(db: &Database, user_uuid: Uuid) -> Result<Vec<User>> {
        let user_id = Self::get_id_by_uuid(db, Some(user_uuid)).await?.unwrap();
        if !User::is_exists_by_id(db, user_id).await? {
            return Err(anyhow!("Felhasználó nem létezik"));
        }

        let rows = sqlx::query!(
            "SELECT u.uuid             AS user_uuid,
                    u.email            AS user_email,
                    u.username         AS user_username,
                    u.user_role        AS user_user_role,
                    m.uuid             AS manager_uuid,
                    ui.id              AS ui_id,
                    ui.full_name       AS ui_full_name,
                    ui.phone_number    AS ui_phone_number,
                    ui.hufa_code       AS ui_hufa_code,
                    ui.agent_code      AS ui_agent_code
              FROM users u
              JOIN user_info ui ON ui.user_id = u.id
              JOIN users m ON m.id = u.manager_id
              WHERE u.manager_id = $1
              ORDER BY CASE u.user_role
                  WHEN 'Leader' THEN 1
                  WHEN 'Manager' THEN 2
                  WHEN 'Agent' THEN 3
              END;",
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        let users = rows
            .into_iter()
            .map(|row| User {
                uuid: row.user_uuid,
                email: Some(row.user_email),
                username: Some(row.user_username),
                user_role: Some(UserRole::from(row.user_user_role)),
                info: UserInfo {
                    full_name: Some(row.ui_full_name),
                    phone_number: Some(row.ui_phone_number),
                    hufa_code: Some(row.ui_hufa_code),
                    agent_code: Some(row.ui_agent_code),
                    ..Default::default()
                },
                manager_uuid: row.manager_uuid,
                ..Default::default()
            })
            .collect();
        Ok(users)
    }

    pub async fn get_info_by_id(db: &Database, user_id: i32) -> Result<User> {
        if !User::is_exists_by_id(db, user_id).await? {
            return Err(anyhow!("Felhasználó nem létezik"));
        }

        let row = sqlx::query!(
            "SELECT u.email            AS user_email,
                    u.user_role        AS user_user_role,
                    ui.full_name       AS ui_full_name,
                    ui.phone_number    AS ui_phone_number,
                    ui.hufa_code       AS ui_hufa_code,
                    ui.agent_code      AS ui_agent_code
             FROM users u
             JOIN user_info ui ON ui.user_id = u.id
             WHERE user_id = $1",
            user_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(User {
            email: Some(row.user_email),
            info: UserInfo {
                full_name: Some(row.ui_full_name),
                phone_number: Some(row.ui_phone_number),
                hufa_code: Some(row.ui_hufa_code),
                agent_code: Some(row.ui_agent_code),
                ..Default::default()
            },
            user_role: Some(UserRole::from(row.user_user_role)),
            ..Default::default()
        })
    }

    pub async fn modify_info(db: &Database, user_uuid: Uuid, user: User) -> Result<()> {
        let user_id = Self::get_id_by_uuid(db, Some(user_uuid)).await?.unwrap();
        if !User::is_exists_by_id(db, user_id).await? {
            return Err(anyhow!("Invalid user_id"));
        }

        let mut tx = db.pool.begin().await?;
        sqlx::query!(
            "UPDATE users
             SET email = $2
             WHERE id = $1",
            user_id,
            user.email
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE user_info
             SET full_name = $2,
                 phone_number = $3,
                 hufa_code = COALESCE($4, hufa_code),
                 agent_code = COALESCE($5, agent_code)
             WHERE user_id = $1",
            user_id,
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

    pub async fn modify_manager(db: &Database, user_uuid: Uuid, user: User) -> Result<()> {
        let user_id = Self::get_id_by_uuid(db, Some(user_uuid)).await?.unwrap();
        if !User::is_exists_by_id(db, user_id).await? {
            return Err(anyhow!("Invalid user_id"));
        }

        let manager_id = Self::get_id_by_uuid(db, user.manager_uuid).await?;
        if let Some(manager_id) = manager_id {
            sqlx::query!(
                "UPDATE users
                 SET manager_id = $2, user_role = DEFAULT
                 WHERE id = $1",
                user_id,
                manager_id
            )
            .execute(&db.pool)
            .await?;
        } else {
            sqlx::query!(
                "UPDATE users
                 SET manager_id = NULL, user_role = 'Manager'
                 WHERE id = $1",
                user_id
            )
            .execute(&db.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn delete(db: &Database, user_uuid: Uuid) -> Result<()> {
        let user_id = Self::get_id_by_uuid(db, Some(user_uuid)).await?.unwrap();
        if !User::is_exists_by_id(db, user_id).await? {
            return Err(anyhow!("Invalid user_id"));
        }

        sqlx::query!(
            "DELETE FROM users
             WHERE id = $1",
            user_id
        )
        .execute(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get_sub_users(db: &Database, user_id: i32, min_role: String) -> Result<Vec<User>> {
        let user_role = Self::get_role(db, user_id).await?;

        let users = match user_role {
            UserRole::Leader => {
                let rows = sqlx::query!(
                    "SELECT u.uuid, ui.full_name, u.user_role
                     FROM users u
                     JOIN user_info ui ON ui.user_id = u.id
                     WHERE (
                        ($2 = 'Leader' AND u.user_role = 'Leader')
                        OR ($2 = 'Manager' AND u.user_role IN ('Manager', 'Leader'))
                        OR ($2 = 'Any')
                     )
                     ORDER BY
                        CASE WHEN u.id = $1 THEN 0 END,
                        CASE u.user_role 
                            WHEN 'Leader' THEN 1
                            WHEN 'Manager' THEN 2
                            WHEN 'Agent' THEN 3
                        END;",
                    user_id,
                    min_role
                )
                .fetch_all(&db.pool)
                .await?;

                rows.into_iter()
                    .map(|user| User {
                        uuid: user.uuid,
                        info: UserInfo {
                            full_name: Some(user.full_name),
                            ..Default::default()
                        },
                        user_role: Some(UserRole::from(user.user_role)),
                        ..Default::default()
                    })
                    .collect()
            }
            UserRole::Manager => {
                let rows = sqlx::query!(
                    "SELECT u.uuid, ui.full_name, u.user_role
                     FROM users u
                     JOIN user_info ui ON ui.user_id = u.id
                     WHERE u.id = $1 OR u.manager_id = $1 AND (
                        ($2 = 'Leader' AND u.user_role = 'Leader')
                        OR ($2 = 'Manager' AND u.user_role IN ('Manager', 'Leader'))
                        OR ($2 = 'Any')
                     )
                     ORDER BY
                        CASE WHEN u.id = $1 THEN 0 END,
                        CASE u.user_role 
                            WHEN 'Leader' THEN 1
                            WHEN 'Manager' THEN 2
                            WHEN 'Agent' THEN 3
                        END;",
                    user_id,
                    min_role
                )
                .fetch_all(&db.pool)
                .await?;

                rows.into_iter()
                    .map(|user| User {
                        uuid: user.uuid,
                        info: UserInfo {
                            full_name: Some(user.full_name),
                            ..Default::default()
                        },
                        user_role: Some(UserRole::from(user.user_role)),
                        ..Default::default()
                    })
                    .collect()
            }
            _ => {
                let rows = sqlx::query!(
                    "SELECT u.uuid, ui.full_name, u.user_role
                     FROM users u
                     JOIN user_info ui ON ui.user_id = u.id
                     WHERE u.id = $1",
                    user_id
                )
                .fetch_all(&db.pool)
                .await?;

                rows.into_iter()
                    .map(|user| User {
                        uuid: user.uuid,
                        info: UserInfo {
                            full_name: Some(user.full_name),
                            ..Default::default()
                        },
                        user_role: Some(UserRole::from(user.user_role)),
                        ..Default::default()
                    })
                    .collect()
            }
        };

        Ok(users)
    }
}

impl UserRole {
    pub async fn get_managers(db: &Database, user_id: i32) -> Result<Vec<ManagerNameDto>> {
        let rows = sqlx::query!(
            r#"
            SELECT u.uuid as user_uuid, ui.full_name, user_role
            FROM users u
            JOIN user_info ui ON ui.user_id = u.id
            WHERE (u.user_role = 'Manager' OR u.user_role = 'Leader') AND u.id != $1
            ORDER BY u.user_role ASC, ui.full_name
            "#,
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ManagerNameDto {
                uuid: r.user_uuid,
                full_name: r.full_name,
                user_role: r.user_role,
            })
            .collect())
    }
}
