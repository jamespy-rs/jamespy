use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use bb8_redis::{
    bb8::Pool,
    RedisConnectionManager,
};



pub type DbPool = PgPool;
pub type RedisPool = Pool<RedisConnectionManager>;


pub async fn init_data() -> DbPool {
    let database_url =
        env::var("DATABASE_URL").expect("No database url found in environment variables!");

    let database = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database!");

    sqlx::migrate!().run(&database).await.expect("Unable to apply migrations!");

    database
}

pub async fn init_redis_pool() -> RedisPool {
    let redis_url = env::var("REDIS_URL").expect("No Redis URL found in environment variables!");

    let manager = RedisConnectionManager::new(redis_url)
        .expect("Failed to create Redis connection manager!");

    let pool = Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create Redis connection pool!");

    pool
}
