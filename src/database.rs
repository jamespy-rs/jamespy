use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;


// Could you tell I have no idea what i'm doing?
pub type DbPool = PgPool;

pub async fn init_data() -> DbPool {
    let database_url =
        env::var("DATABASE_URL").expect("No database url found in environment variables!");

    let database = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database!");

    // Apply migrations
    sqlx::migrate!().run(&database).await.expect("Unable to apply migrations!");

    database
}

