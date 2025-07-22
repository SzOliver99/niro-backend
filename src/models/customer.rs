use anyhow::{Ok, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::database::Database;

#[derive(Debug, Serialize, Deserialize)]
pub struct Customer {
    pub id: Option<i32>,
    pub email: Option<String>,
    pub phonenumber: Option<String>,
    pub history: Vec<CustomerHistory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerHistory {
    pub id: Option<i32>,
    pub ptype: String,
    pub time: NaiveDateTime,
}

impl Customer {
    pub async fn new(db: &Database, user_id: i32, new_customer: Customer) -> Result<()> {
        if Self::is_customer_exists(db, &new_customer).await? {
            return Err(anyhow::anyhow!("Customer already in the database"));
        }

        let _customer_id = sqlx::query!(
            "INSERT INTO customers(email, phonenumber, user_id) VALUES($1, $2, $3) RETURNING id",
            new_customer.email,
            new_customer.phonenumber,
            user_id
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(())
    }
}

impl Customer {
    async fn is_customer_exists(db: &Database, customer: &Customer) -> Result<bool> {
        let is_exists = sqlx::query!(
            "SELECT id from customers WHERE email = $1 OR phonenumber = $2",
            customer.email,
            customer.phonenumber
        )
        .fetch_optional(&db.pool)
        .await?;

        Ok(is_exists.is_some())
    }
}
