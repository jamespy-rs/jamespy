use bb8_redis::redis::{self, AsyncCommands};
use poise::serenity_prelude::{self as serenity, Channel};
use sqlx::query;

use crate::Data;
use crate::Error;

const MAX_CACHED_MESSAGES: usize = 250; // Max number of messages cached per channel

pub async fn recieve_or_cache_guild(ctx: &serenity::Context, guild_id: i64, data: &Data) -> Result<String, serenity::Error> {
    let redis_pool = &data.redis;
    let guild_key = format!("guild:{}", &guild_id);
    let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");
    let guild_name: Option<String> = redis_conn.hget(&guild_key, "name").await.expect("Failed to fetch guild from cache.");

    let guild_name = match guild_name {
        Some(name) => name,
        None => {
            let guild = ctx.http.get_guild(guild_id.try_into().unwrap()).await?;
            let guild_name = guild.name.clone();

            redis_conn.hset::<_, _, _, ()>(&guild_key, "name", &guild_name.as_str()).await.expect("Failed to cache guild.");
            guild_name
        }
    };

    Ok(guild_name)
}

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _ctx_poise: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::Message { new_message } => {
            let db_pool = &data.db;
            let redis_pool = &data.redis;

            let guild_id = new_message.guild_id.map(|id| id.0 as i64).unwrap_or_default(); // This makes it 0 if no guild is present

            let guild_name = if guild_id == 0 {
                "None".to_string()
            } else {
                recieve_or_cache_guild(ctx, guild_id, data).await?
            };

            let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");
            let message_cache_key = format!("channel:{}:messages", new_message.channel_id.0);

            // I should cache everything about a message thats important!

            let _result: Result<(), _> = redis_conn
                .lpush(&message_cache_key, format!("{}:{}", new_message.id.0, new_message.content))
                .await;

            let _trim_result: Result<(), _> = redis_conn
                .ltrim(&message_cache_key, 0, (MAX_CACHED_MESSAGES as isize) - 1)
                .await;

            let channel_name: String;
            let channel_key = format!("channel:{}", new_message.channel_id.0);

            let channel_name_redis: Option<String> = redis_conn.hget(&channel_key, "name").await.expect("Failed to fetch channel from cache.");
            if let Some(name) = channel_name_redis {
                channel_name = name;
            } else {
                channel_name = format!("{}", new_message.channel_id.0);
            }
            // Print the message
            // TODO: colouring!
            println!("[{}] [#{}] {}: {}", guild_name, channel_name, new_message.author.name, new_message.content);


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

        poise::Event::GuildCreate { guild, is_new: _ } => {
            let redis_pool = &data.redis;
            let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");

            let guild_id = guild.id.0.to_string();
            let guild_redis_key = format!("guild:{}", guild_id);

            let _: redis::RedisResult<()> = redis_conn
                .hset(&guild_redis_key, "name", guild.name.clone())
                .await;

            for (channel_id, channel) in &guild.channels {
                let channel_redis_key = format!("channel:{}", channel_id.0);

                let channel_name = match channel {
                    Channel::Guild(guild_channel) => guild_channel.name.clone(),
                    Channel::Category(category_channel) => category_channel.name.clone(),
                    _ => todo!(),
                };

                let _: redis::RedisResult<()> = redis_conn
                    .hset(&channel_redis_key, "name", channel_name)
                    .await;

                let _: redis::RedisResult<()> = redis_conn
                    .sadd(&guild_redis_key, channel_id.0.to_string())
                    .await;
            }
            // Need to cache threads!
        }

        poise::Event::GuildDelete { incomplete, full: _ } => {
            let redis_pool = &data.redis;
            let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");


            let guild_id = incomplete.id.0.to_string();
            let guild_key: String = format!("guild:{}", &guild_id);
            let guild_name: Option<String> = redis_conn.hget(&guild_key, "name").await.expect("Failed to fetch guild from cache.");
            println!("{:?}", guild_name) // This is a some() so fix it then remove guild and channels


        }
        poise::Event::ReactionAdd { add_reaction } => {
            println!("[{:?}] [#{}] {:?} added a reaction: {}", add_reaction.guild_id, add_reaction.channel_id, add_reaction.user_id, add_reaction.emoji)
            // Need to just cache and recieve almost everything!
        }
        poise::Event::ReactionRemove { removed_reaction } => {
            println!("[{:?}] [#{}] {:?} removed a reaction: {}", removed_reaction.guild_id, removed_reaction.channel_id, removed_reaction.user_id, removed_reaction.emoji)
            // Need to just cache and recieve almost everything!
        }
        poise::Event::ReactionRemoveAll { channel_id: _, removed_from_message_id: _ } => {
            // Need to do the funny here.
            // Will leave it untouched until I have a better codebase.
        }
        poise::Event::ChannelCreate { channel } => {
            let redis_pool = &data.redis;
            let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");

            let guild_id = channel.guild_id.0.to_string();
            let guild_redis_key = format!("guild:{}", guild_id);

            let guild_name: Option<String> = redis_conn.hget(&guild_redis_key, "name").await.expect("Failed to fetch guild from cache.");

            let guild_name = match guild_name {
                Some(name) => name,
                None => "Unknown Guild".to_owned(),
            };

            println!("[{}] #{} was created!", guild_name, channel.name);

            let channel_redis_key = format!("channel:{}", channel.id.0);
            let _channel_cache_result: Result<(), _> = redis_conn
                .hset(&channel_redis_key, "name", channel.name.clone())
                .await;

            let _guild_channels_result: Result<(), _> = redis_conn
                .sadd(&guild_redis_key, channel.id.0.to_string())
                .await;
        }

        poise::Event::ChannelDelete { channel } => {
            let redis_pool = &data.redis;
            let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");

            let channel_redis_key = format!("channel:{}", channel.id.0);
            let _delete_channel_result: Result<(), _> = redis_conn
                .del(&channel_redis_key)
                .await;

            let guild_id = channel.guild_id.0.to_string();
            let guild_redis_key = format!("guild:{}", guild_id);
            let _remove_channel_result: Result<(), _> = redis_conn
                .srem(&guild_redis_key, channel.id.0.to_string())
                .await;

            let message_cache_key = format!("channel:{}:messages", channel.id.0);
            let _delete_messages_result: Result<(), _> = redis_conn
                .del(&message_cache_key)
                .await;
            // This will also need to delete messages from all threads if the channel has them.
        }
        poise::Event::ChannelUpdate { old: _, new } => {
            let redis_pool = &data.redis;
            let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");

            let channel_name = match &new {
                Channel::Guild(guild_channel) => guild_channel.name.clone(),
                Channel::Category(category_channel) => category_channel.name.clone(),
                _ => todo!(),
            };

            let channel_redis_key = format!("channel:{}", new.id().0);

            let _channel_cache_result: Result<(), _> = redis_conn
                .hset(&channel_redis_key, "name", channel_name.clone())
                .await;

            println!("#{}'s name updated to {}!", channel_name, channel_name);
        }

        // Will come back for threads when I cache them
        poise::Event::ThreadCreate { thread } => {

        }
        poise::Event::ThreadDelete { thread } => {

        }
        poise::Event::VoiceStateUpdate { old, new } => {
            // Oh this one will be fun..
            // Later me problem!
        }

        // Remove on guild remove (technically done, just need to do it on threads as well)
        // Thread deletion/creation/edits
        // Track edits/deletion of messages & cache them properly with a limit
        // user join/leave tracking
        // user updates
        // voice events
        // Implement anti 32 Bit Link measures
        _ => (),
    }

    Ok(())
}
