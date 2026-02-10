pub mod contracts;
pub mod gigs;
pub mod messages;
pub mod portfolio;
pub mod users;

use sea_orm::{Database, DatabaseConnection};
use std::env;

/// Create a SeaORM database connection pool from the `DATABASE_URL` env var.
pub async fn create_pool() -> DatabaseConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Database::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}
