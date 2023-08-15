use bb8_redis::redis::{self, AsyncCommands};
use poise::futures_util::future::join_all;
use poise::serenity_prelude::UserId;
use poise::serenity_prelude::{self as serenity, Channel};
use sqlx::query;

use crate::Data;
use crate::Error;
use crate::utils;

use utils::snippets::*;

const MAX_CACHED_MESSAGES: usize = 350; // Max number of messages cached per channel

pub async fn receive_or_cache_guild(ctx: &serenity::Context, guild_id: i64, data: &Data) -> Result<String, serenity::Error> {
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
// I could probably merge this with threads or whatever else? I need to cache users regardless anyway so I might make a combo version.
// (I have no good developer practices)
pub async fn receive_or_cache_channel(ctx: &serenity::Context, guild_id: i64, channel_id: i64, data: &Data) -> Result<String, serenity::Error> {
    let redis_pool = &data.redis;
    let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");

    let channel_key = format!("channel:{}", channel_id);

    let channel_name: Option<String> = redis_conn.hget(&channel_key, "name").await.expect("Failed to fetch channel from cache.");

    let channel_name = match channel_name {
        Some(name) => name,
        None => {
            let channel = match ctx.http.get_channel(channel_id.try_into().unwrap()).await {
                Ok(channel) => channel,
                Err(_) => return Err(serenity::Error::Other("Failed to receive the channel!")),
            };

            let fetched_channel_name = match &channel {
                serenity::model::channel::Channel::Guild(text_channel) => text_channel.name.clone(),
                serenity::model::channel::Channel::Private(private_channel) => private_channel.name().clone(),
                _ => "Unknown Channel Name".to_string(),
            };

            redis_conn.hset::<_, _, _, ()>(&channel_key, "name", &fetched_channel_name).await.expect("Failed to cache channel.");

            let channel_set_key = format!("channel_set:{}", guild_id);
            redis_conn.sadd::<_, _, ()>(&channel_set_key, channel_id.to_string()).await.expect("Failed to add channel_id to guild set.");

            fetched_channel_name
        }
    };

    Ok(channel_name)
}

pub async fn receive_or_cache_user(ctx: &serenity::Context, user_id: i64, data: &Data) -> Result<String, serenity::Error> {
    let redis_pool = &data.redis;
    let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");

    let user_key = format!("user:{}", user_id);

    let user_name: Option<String> = redis_conn.hget(&user_key, "name").await.expect("Failed to fetch user from cache.");

    let user_name = match user_name {
        Some(name) => name,
        None => {
            let user = match ctx.http.get_user(user_id.try_into().unwrap()).await {
                Ok(user) => user,
                Err(_) => return Err(serenity::Error::Other("Failed to receive the user!")),
            };

            let fetched_user_name = user.name.clone();

            redis_conn.hset::<_, _, _, ()>(&user_key, "name", &fetched_user_name).await.expect("Failed to cache user.");

            fetched_user_name
        }
    };

    Ok(user_name)
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

            let guild_id = new_message.guild_id.unwrap_or_default();

            let guild_name = if guild_id == 0 {
                "None".to_string()
            } else {
                if let Some(guild) = ctx.cache.guild(guild_id) {
                    guild.name.to_string()
                } else {
                    "Unknown".to_string()
                }
            };


            let channel_name = if let Some(channel) = ctx.cache.channel(new_message.channel_id) {
                match &channel {
                    serenity::model::channel::Channel::Guild(guild_channel) => {
                        guild_channel.name.clone()
                    },
                    _ => "Unknown Channel".to_string(),
                }
            } else {
                "Unknown Channel".to_string()
            };


            // I 100% should handle threads at some point, but I may have to merge the channels & threads set for this to happen without an extra request?
            // I'm not actually sure if this is the case.

            let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");
            let message_cache_key = format!("channel:{}:messages", new_message.channel_id.0);

            // I should cache everything about a message thats important!

            let _result: Result<(), _> = redis_conn
                .lpush(&message_cache_key, format!("{}:{}", new_message.id.0, new_message.content))
                .await;

            let _trim_result: Result<(), _> = redis_conn
                .ltrim(&message_cache_key, 0, (MAX_CACHED_MESSAGES as isize) - 1)
                .await;

            // I can get this directly from the message but eh
            let user_name = receive_or_cache_user(ctx, new_message.author.id.0 as i64, data).await?; // Cache or retrieve user name

            // Print the message
            // TODO: colouring!
            println!("[{}] [#{}] {}: {}", guild_name, channel_name, user_name, new_message.content);

            let _ = query!(
                "INSERT INTO msgs (guild_id, channel_id, message_id, user_id, content, attachments, timestamp)
                 VALUES ($1, $2, $3, $4, $5, $6, now())",
                i64::from(guild_id), // It told me to do this.
                new_message.channel_id.0 as i64,
                new_message.id.0 as i64,
                new_message.author.id.0 as i64,
                &new_message.content,
                "future me problem"
            )
            .execute(&*db_pool)
            .await;
        // Need to get my bot to react for join tracking.
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
                if let Channel::Guild(guild_channel) = channel {
                    let channel_redis_key = format!("channel:{}", channel_id.0);
                    let channel_set_key = format!("channel_set:{}", guild_id);

                    let channel_name = guild_channel.name.clone();

                    let _: redis::RedisResult<()> = redis_conn
                        .hset(&channel_redis_key, "name", channel_name)
                        .await;

                    let _: redis::RedisResult<()> = redis_conn
                        .sadd(&channel_set_key, channel_id.0.to_string())
                        .await;
                }
            }
            for thread in &guild.threads {
                let thread_redis_key = format!("thread:{}", thread.id.0);
                let thread_name = thread.name.clone();

                let _: redis::RedisResult<()> = redis_conn
                    .hset(&thread_redis_key, "name", thread_name)
                    .await;
            }
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
            // Need to track reacts on accela messages.

            if let Some(guild_id) = add_reaction.guild_id {
                let guild_name = receive_or_cache_guild(ctx, guild_id.into(), data).await;
                let channel_id = add_reaction.channel_id;
                let channel_name = receive_or_cache_channel(ctx, guild_id.into(), channel_id.into(), data).await;

                let user_id = add_reaction.user_id.unwrap();
                let user_name = match user_id.to_user(&ctx.http).await {
                    // I'm going to phase out the function for caching users probably and use this instead.
                    Ok(user) => user.name,
                    Err(_) => "Unknown User".to_string(),
                };

                if let (Ok(guild_name), Ok(channel_name)) = (guild_name, channel_name) {
                    println!(
                        "[{}] [#{}] {} added a reaction: {}",
                        guild_name, channel_name, user_name, add_reaction.emoji
                    );
                }
            }
        }



        poise::Event::ReactionRemove { removed_reaction } => {
            if let Some(guild_id) = removed_reaction.guild_id {
                let guild_name = receive_or_cache_guild(ctx, guild_id.into(), data).await;
                let channel_id = removed_reaction.channel_id;
                let channel_name = receive_or_cache_channel(ctx, guild_id.into(), channel_id.into(), data).await;

                let user_id = removed_reaction.user_id.unwrap();
                let user_name = match user_id.to_user(&ctx.http).await {
                    // I'm going to phase out the function for caching users probably and use this instead.
                    Ok(user) => user.name,
                    Err(_) => "Unknown User".to_string(),
                };

                if let (Ok(guild_name), Ok(channel_name)) = (guild_name, channel_name) {
                    println!(
                        "[{}] [#{}] {} added a reaction: {}",
                        guild_name, channel_name, user_name, removed_reaction.emoji
                    );
                }
            }
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
            let channel_set_key = format!("channel_set:{}", guild_id);

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

            let _: redis::RedisResult<()> = redis_conn
            .sadd(&channel_set_key, channel.id.0.to_string())
            .await;
        }
        // I need to go back to this.
        poise::Event::ChannelDelete { channel } => {
            let redis_pool = &data.redis;
            let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");

            let channel_redis_key = format!("channel:{}", channel.id.0);

            let _delete_channel_result: Result<(), _> = redis_conn
                .del(&channel_redis_key)
                .await;

            let guild_id = channel.guild_id.0.to_string();
            let channel_set_key = format!("channel_set:{}", guild_id);
            let _remove_channel_result: Result<(), _> = redis_conn
                .srem(&channel_set_key, channel.id.0.to_string())
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

            let channel_redis_key = format!("channel:{}", new.id().0);

            let old_channel_name: String = redis_conn
                .hget(&channel_redis_key, "name")
                .await
                .unwrap_or_else(|_| String::from("Unknown"));

            let new_channel_name = match &new {
                Channel::Guild(new_guild_channel) => new_guild_channel.name.clone(),
                Channel::Category(new_category_channel) => new_category_channel.name.clone(),
                _ => todo!(),
            };

            let _channel_cache_result: Result<(), _> = redis_conn
                .hset(&channel_redis_key, "name", new_channel_name.clone())
                .await;

            println!(
                "#{}'s name updated to #{}!",
                old_channel_name, new_channel_name
            );
        }


        // Will come back for threads when I cache them
        poise::Event::ThreadCreate { thread } => {
            let redis_pool = &data.redis;
            let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");

            let guild_id = thread.guild_id.0.to_string();
            let guild_redis_key = format!("guild:{}", guild_id);
            let thread_set_key = format!("thread_set:{}", guild_id);

            let guild_name: Option<String> = redis_conn.hget(&guild_redis_key, "name").await.expect("Failed to fetch guild from cache.");

            let guild_name = match guild_name {
                Some(name) => name,
                None => "Unknown Guild".to_owned(),
            };

            println!("[{}] Thread #{} was created!", guild_name, thread.name);

            let thread_redis_key = format!("thread:{}", thread.id.0);
            let _thread_cache_result: Result<(), _> = redis_conn
                .hset(&thread_redis_key, "name", thread.name.clone())
                .await;

            let _: redis::RedisResult<()> = redis_conn
                .sadd(&thread_set_key, thread.id.0.to_string())
                .await;
        }

        poise::Event::ThreadDelete { thread } => {
            // TODO: do this after cleanup of the rest of the bot is done. (need to delete cached messages related etc etc)
        }

        poise::Event::ThreadUpdate { thread } => {
            let redis_pool = &data.redis;
            let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");

            let thread_redis_key = format!("thread:{}", thread.id.0);

            let old_thread_name: String = redis_conn
                .hget(&thread_redis_key, "name")
                .await
                .unwrap_or_else(|_| String::from("Unknown"));

            let new_thread_name = thread.name.clone();

            let _thread_cache_result: Result<(), _> = redis_conn
                .hset(&thread_redis_key, "name", new_thread_name.clone())
                .await;

            println!(
                "Thread #{}'s name updated to #{}!",
                old_thread_name, new_thread_name
            );
        }

        poise::Event::VoiceStateUpdate { old, new } => {
            // Oh this one will be fun..
            // Later me problem!
        }

        poise::Event::GuildMemberUpdate { old_if_available: _, new } => {
            let redis_pool = &data.redis;
            let mut redis_conn = redis_pool.get().await.expect("Failed to get Redis connection");

            let user_id = new.user.id.0 as i64;

            let user_key = format!("user:{}", user_id);

            let updated_name = new.user.name.clone();
            // I assume this works, but I need to do the same for nicknames and AAAAAAAAAAA
            redis_conn.hset::<_, _, _, ()>(&user_key, "name", &updated_name).await.expect("Failed to update cached user name.");

        }
        poise::Event::Ready { data_about_bot: _ } => {
            let _ = set_all_snippets(&data).await;
            // Need to check join tracks.
        }
        poise::Event::GuildMemberAddition { new_member } => {
            let guild_id = new_member.guild_id;
            let joined_user_id = new_member.user.id;
            let db_pool = &data.db;

            let query_result = sqlx::query!(
                "SELECT author_id FROM join_tracks WHERE guild_id = $1 AND user_id = $2",
                guild_id.0 as i64,
                UserId(joined_user_id.0 as u64).0 as i64
            )
            .fetch_all(db_pool)
            .await;


            match query_result {
                Ok(rows) => {
                    let mut author_ids = Vec::new();

                    for row in rows {
                        let author_id = match row.author_id {
                            Some(value) => value,
                            None => 0,
                        };
                        author_ids.push(UserId(author_id.try_into().unwrap()));
                    }

                    let author_futures = author_ids.into_iter().filter_map(|author_id| {
                        let cache = ctx.cache.clone();
                        let dm_content = format!(
                            "{} has joined {}!",
                            new_member.user.name,
                            guild_id.name(&ctx.cache).unwrap_or_else(|| "the server".to_string())
                        );

                        Some(async move {
                            if let Some(author) = cache.user(author_id) {
                                if let Err(err) = author.dm(ctx, |m| m.content(dm_content)).await {
                                    eprintln!("Failed to send DM to author {}: {:?}", author_id, err);
                                }
                            }
                        })
                    });

                    let _ = join_all(author_futures).await;
                }
                Err(err) => {
                    eprintln!("Failed to retrieve authors tracking user: {:?}", err);
                }
            }
        }

        // Only say the name changed if the name changed.
        // Remove on guild remove (technically done, just need to do it on threads as well)
        // Thread deletion/creation/edits
        // Track edits/deletion of messages & cache them properly with a limit
        // user join/leave tracking
        // user updates
        // voice events
        _ => (),
    }

    Ok(())
}
