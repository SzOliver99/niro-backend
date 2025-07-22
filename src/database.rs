use sqlx::{Pool, Postgres, prelude::FromRow};
use std::{env, time::Duration};

#[derive(FromRow, Debug, Clone)]
pub struct Database {
    pub pool: Pool<Postgres>,
    // pub redis: redis::Client
}

impl Database {
    pub async fn create_connection() -> Result<Self, sqlx::error::Error> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect(&database_url)
            .await?;

        // let redis = redis::Client::open(redis_url).unwrap();

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }
}
