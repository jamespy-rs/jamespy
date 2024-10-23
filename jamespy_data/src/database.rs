use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

pub async fn init_data() -> PgPool {
    let database_url =
        env::var("DATABASE_URL").expect("No database url found in environment variables!");

    let database = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database!");

    sqlx::migrate!("../migrations")
        .run(&database)
        .await
        .expect("Unable to apply migrations!");

    database
}

/// Custom type.
#[derive(Debug, Clone, sqlx::Type)]
pub enum EmoteUsageType {
    Message,
    ReactionAdd,
    ReactionRemove,
}
