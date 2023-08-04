use bb8_redis::redis::{self, AsyncCommands};
use poise::serenity_prelude as serenity;
use sqlx::query;

use crate::Data;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn event_handler(
    _ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _ctx_poise: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::Message { new_message } => {
            let db_pool = &data.db;
            let redis_pool = &data.redis;
            let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");
            let result: redis::RedisResult<()> = redis_conn.set("test", "test").await;

            let guild_id = new_message.guild_id.map(|id| id.0 as i64);
            let _ = query!(
                "INSERT INTO msgs (guild_id, channel_id, message_id, user_id, content, attachments, timestamp)
                 VALUES ($1, $2, $3, $4, $5, $6, now())",
                guild_id,
                new_message.channel_id.0 as i64,
                new_message.id.0 as i64,
                new_message.author.id.0 as i64,
                &new_message.content,
                "future me problem"
            )
            .execute(&*db_pool)
            .await;
        }


        _ => (),
    }

    Ok(())
}

