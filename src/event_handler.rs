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

            let guild_id = new_message.guild_id.map(|id| id.0 as i64).unwrap_or_default();

            // Handle dms vs invalid guild and add the guild if it isn't added for some reason.

            let guild_key = format!("guild:{}", &guild_id);

            let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");
            let guild_name: Option<String> = redis_conn.hget(&guild_key, "name").await.expect("Failed to fetch guild name from Redis");

            let guild_name = match guild_name {
                Some(name) => name,
                None => "None".to_owned(),
            };
            println!("[{}] [#{}] {}: {}", guild_name, new_message.channel_id, new_message.author.tag(), new_message.content);
            // Replace with channel name, but I need to cache this first meaning I need to improve code first.

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
        poise::Event::GuildCreate { guild, is_new } => {
            let redis_pool = &data.redis;
            let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");

            let guild_id = guild.id.0.to_string();
            let redis_key = format!("guild:{}", guild_id);

            let result: redis::RedisResult<()> = redis_conn
                .hset(&redis_key, "name", guild.name.clone())
                .await;

            match result {
                Ok(_) => {
                    println!("Added guild '{}' to Redis with key '{}'", guild.name, redis_key);
                }
                Err(err) => {
                    eprintln!("Failed to add guild to Redis: {:?}", err);
                }
            }
        }
        // Remove on guild remove
        // Track channel, thread deletion/creation/edits
        // Track edits/deletion of messages & cache them properly with a limit of like 1000?
        // user join/leave tracking
        // reaction add/remove/remove all
        // thread member updates?
        // user updates
        // Implement anti 32 Bit Link measures
        _ => (),
    }

    Ok(())
}

