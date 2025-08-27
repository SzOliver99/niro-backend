use anyhow::{Ok, Result};
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{database::Database, models::user::User};

#[skip_serializing_none]
#[derive(Debug, Serialize, Default)]
pub struct Customer {
    pub id: Option<i32>,
    pub full_name: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub user_id: Option<i32>,
    pub created_by: Option<String>,
    pub leads: Vec<Customer>,
}

impl Customer {
    pub(super) async fn is_exists(db: &Database, customer: &Customer) -> Result<bool> {
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

    pub(super) async fn is_exists_by_id(db: &Database, customer_id: i32) -> Result<bool> {
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

impl Customer {
    pub async fn create(db: &Database, new_customer: Customer, user: User) -> Result<()> {
        if Self::is_exists(db, &new_customer).await? {
            return Err(anyhow::anyhow!("Customer is already in the database"));
        }

        let _row = sqlx::query!(
            "INSERT INTO customers(full_name, phone_number, email, address, user_id, created_by)
             VALUES($1, $2, $3, $4, $5, $6)
             RETURNING id",
            new_customer.full_name,
            new_customer.phone_number,
            new_customer.email,
            new_customer.address,
            new_customer.user_id,
            user.info.full_name
        )
        .fetch_one(&db.pool)
        .await?;

        Ok(())
    }

    pub async fn get(db: &Database, user_id: i32) -> Result<Self> {
        let row = sqlx::query!(
            "SELECT full_name, phone_number, email, address, user_id 
             FROM customers
             WHERE id = $1",
            user_id
        )
        .fetch_one(&db.pool)
        .await?;
        Ok(Customer {
            full_name: Some(row.full_name),
            phone_number: Some(row.phone_number),
            email: Some(row.email),
            address: Some(row.address),
            user_id: row.user_id,
            ..Default::default()
        })
    }

    pub async fn change_handler(
        db: &Database,
        user_full_name: String,
        customer_ids: Vec<i32>,
    ) -> Result<()> {
        let user = sqlx::query!(
            "SELECT user_id as id FROM user_info WHERE full_name = $1",
            user_full_name
        )
        .fetch_one(&db.pool)
        .await?;

        sqlx::query!(
            "UPDATE customers
             SET user_id = $2
             WHERE id = ANY($1)",
            &customer_ids,
            user.id
        )
        .execute(&db.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(db: &Database, customer_ids: Vec<i32>) -> Result<()> {
        for customer_id in customer_ids {
            if !Customer::is_exists_by_id(db, customer_id).await? {
                return Err(anyhow::anyhow!("Invalid customer_id"));
            }

            sqlx::query!(
                "DELETE FROM customers
                 WHERE id = $1",
                customer_id
            )
            .execute(&db.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_all(db: &Database, user_id: i32) -> Result<Vec<Self>> {
        let row = sqlx::query!(
            "SELECT id, full_name, phone_number, email, address, user_id, created_by
             FROM customers
             WHERE user_id = $1",
            user_id
        )
        .fetch_all(&db.pool)
        .await?;

        let customers: Vec<Customer> = row
            .into_iter()
            .map(|customer| Customer {
                id: Some(customer.id),
                full_name: Some(customer.full_name),
                phone_number: Some(customer.phone_number),
                email: Some(customer.email),
                address: Some(customer.address),
                user_id: customer.user_id,
                created_by: Some(customer.created_by),
                ..Default::default()
            })
            .collect();
        Ok(customers)
    }
}
