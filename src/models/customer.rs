use anyhow::{Ok, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::database::Database;

#[derive(Debug, Serialize, Deserialize)]
pub struct Customer {
    pub id: Option<i32>,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub user_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerHistory {
    pub id: Option<i32>,
    pub p_type: String,
    pub time: NaiveDateTime,
}

impl Customer {
    pub async fn new(db: &Database, new_customer: Customer) -> Result<()> {
        if Self::is_customer_exists(db, &new_customer).await? {
            return Err(anyhow::anyhow!("Customer already in the database"));
        }

        let _customer_id = sqlx::query!(
            "INSERT INTO customers(email, phone_number, user_id)
             VALUES($1, $2, $3)
             RETURNING id",
            new_customer.email,
            new_customer.phone_number,
            new_customer.user_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get(db: &Database, user_id: i32) -> Result<Self> {
        let customer = sqlx::query_as!(
            Self,
            "SELECT * FROM customers
            WHERE id = $1",
            user_id
        )
        .fetch_one(&db.pool)
        .await?;
        Ok(customer)
    }

    pub async fn create_history(
        db: &Database,
        customer_id: i32,
        history: CustomerHistory,
    ) -> Result<()> {
        if !Self::is_customer_exists_by_id(db, customer_id).await? {
            return Err(anyhow::anyhow!("Customer is not in the database!"));
        }

        let _history_id = sqlx::query!(
            "INSERT INTO customer_history(p_type, time, customer_id)
             VALUES($1, $2, $3)
             RETURNING id",
            history.p_type,
            history.time,
            customer_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get_history(db: &Database, customer_id: i32) -> Result<Vec<CustomerHistory>> {
        if !Self::is_customer_exists_by_id(db, customer_id).await? {
            return Err(anyhow::anyhow!("Customer is not in the database!"));
        }

        let customer_history = sqlx::query_as!(
            CustomerHistory,
            "SELECT id, p_type, time FROM customer_history
             WHERE customer_id = $1",
            customer_id
        )
        .fetch_all(&db.pool)
        .await?;

        Ok(customer_history)
    }
}

impl Customer {
    async fn is_customer_exists(db: &Database, customer: &Customer) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM customers 
             WHERE email = $1 OR phone_number = $2",
            customer.email,
            customer.phone_number
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }

    async fn is_customer_exists_by_id(db: &Database, customer_id: i32) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id FROM customers
             WHERE id = $1",
            customer_id
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }
}
