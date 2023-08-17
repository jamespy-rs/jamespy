use poise::futures_util::future::join_all;
use poise::serenity_prelude::{UserId, GuildId, ChannelId};
use poise::serenity_prelude::{self as serenity};

use sqlx::query;

use crate::Data;
use crate::Error;
use crate::utils;

use utils::snippets::*;

async fn get_channel_name(ctx: &serenity::Context, guild_id: GuildId, channel_id: ChannelId) -> String {
    let mut channel_name = channel_id.name(ctx).await.unwrap_or("Unknown Channel".to_owned());

    if guild_id.0 != 0 && channel_name == "Unknown Channel" {
        let guild_cache = ctx.cache.guild(guild_id).unwrap();
        let threads = &guild_cache.threads;
        for thread in threads {
            if thread.id == channel_id.0 {
                channel_name = thread.name.clone();
                break;
            }
        }
    }

    channel_name
}



pub async fn event_handler(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _ctx_poise: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    let no_log_user: Vec<u64> = vec![432610292342587392, 429656936435286016]; // mudae and rin bot
    let no_log_channel: Vec<u64> = vec![572899947226333254, 787623037834100737, 697738506944118814, 787389586665504778]; // log channels in gg/osu
    match event {
        poise::Event::Message { new_message } => {
            if no_log_user.contains(&new_message.author.id.0) || no_log_channel.contains(&new_message.channel_id.0) ||
            new_message.content.starts_with("$") && new_message.channel_id == 850342078034870302 {
            return Ok(());
            // Removes mudae commands in the mudae channel in gg/osu, alongside other criteria above.
            }
            let db_pool = &data.db;
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

            let channel_name = get_channel_name(ctx, guild_id, new_message.channel_id).await;

            // TODO: colouring!
            println!("\x1B[90m[{}] [#{}]\x1B[0m {}: {}\x1B[0m", guild_name, channel_name, new_message.author.name, new_message.content);
            let _ = query!(
                "INSERT INTO msgs (guild_id, channel_id, message_id, user_id, content, attachments, timestamp)
                 VALUES ($1, $2, $3, $4, $5, $6, now())",
                i64::from(guild_id),
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
        poise::Event::MessageUpdate { old_if_available, new, event } => {
            match (old_if_available, new) {
                (Some(old_message), Some(new_message)) => {
                    if new_message.author.bot {
                        return Ok(());
                    }
                    if old_message.content != new_message.content {
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
                        let channel_name = get_channel_name(ctx, guild_id, new_message.channel_id).await;
                        println!("\x1B[36m[{}] [#{}] A message by \x1B[0m{}\x1B[36m was edited:", guild_name, channel_name, new_message.author.name);
                        println!("BEFORE: {}: {}", new_message.author.name, old_message.content);
                        println!("AFTER: {}: {}\x1B[0m", new_message.author.name, new_message.content);
                    }
                },
                (None, None) => {
                    println!("\x1B[36mA message (ID:{}) was edited but was not in cache\x1B[0m", event.id);
                },
                _ => {}
            }
        }
        poise::Event::MessageDelete { channel_id, deleted_message_id, guild_id } => {
            let guild_id = guild_id.unwrap_or_default();
            let channel_id = channel_id;

            let guild_name = if guild_id == 0 {
                "None".to_string()
            } else {
                if let Some(guild) = ctx.cache.guild(guild_id) {
                    guild.name.to_string()
                } else {
                    "Unknown".to_string()
                }
            };

            let channel_name = get_channel_name(ctx, guild_id, *channel_id).await;

            if let Some(message) = ctx.cache.message(*channel_id, deleted_message_id) {
                let user_name = message.author.name.clone();
                let content = message.content.clone();
                println!("\x1B[91m\x1B[2m[{}] [#{}] A message from \x1B[0m{}\x1B[91m\x1B[2m was deleted: {}\x1B[0m",
                    guild_name, channel_name, user_name, content);
            } else {
                println!("\x1B[91m\x1B[2mA message (ID:{}) was deleted but was not in cache\x1B[0m", deleted_message_id);
            }
        }
        // need poise::Event::MessageDeleteBulk

        poise::Event::GuildCreate { guild: _, is_new: _ } => {
            // eeee

        }
        poise::Event::GuildDelete { incomplete: _, full: _ } => {
            // eeee
        }
        poise::Event::ReactionAdd { add_reaction } => {
            let user_id = add_reaction.user_id.unwrap();
            if ctx.cache.user(user_id).map_or(false, |user| user.bot) {
                return Ok(());
                // May merge with the one below.
            }
            // Need to track reacts on accela messages.
            let guild_id = add_reaction.guild_id.unwrap_or_default();
            let guild_name = if guild_id == 0 {
                "None".to_string()
            } else {
                if let Some(guild) = ctx.cache.guild(guild_id) {
                    guild.name.to_string()
                } else {
                    "Unknown".to_string()
                }
            };
            let channel_name = get_channel_name(ctx, guild_id, add_reaction.channel_id).await;

            let user_name = match user_id.to_user(ctx).await {
                Ok(user) => user.name,
                Err(_) => "Unknown User".to_string(),
            };

            println!(
                "\x1B[95m[{}] [#{}] {} added a reaction: {}\x1B[0m",
                guild_name, channel_name, user_name, add_reaction.emoji
            );

        }
        poise::Event::ReactionRemove { removed_reaction } => {
            let user_id = removed_reaction.user_id.unwrap();
            if ctx.cache.user(user_id).map_or(false, |user| user.bot) {
                return Ok(());
                // May merge with the one below.
            }
            let guild_id = removed_reaction.guild_id.unwrap_or_default();
            let guild_name = if guild_id == 0 {
                "None".to_string()
            } else {
                if let Some(guild) = ctx.cache.guild(guild_id) {
                    guild.name.to_string()
                } else {
                    "Unknown".to_string()
                }
            };
            let channel_name = get_channel_name(ctx, guild_id, removed_reaction.channel_id).await;

            let user_name = match user_id.to_user(&ctx.http).await {
                Ok(user) => user.name,
                Err(_) => "Unknown User".to_string(),
            };

            println!(
                "\x1B[95m[{}] [#{}] {} removed a reaction: {}\x1B[0m",
                guild_name, channel_name, user_name, removed_reaction.emoji
            );

        }
        poise::Event::ReactionRemoveAll { channel_id: _, removed_from_message_id: _ } => {
            // Need to do the funny here.
            // Will leave it untouched until I have a better codebase.
        }
        poise::Event::ChannelCreate { channel } => {
            let guild_name = channel.guild_id.name(ctx).unwrap_or("Unknown Guild".to_string());
            println!("\x1B[34m[{}] #{} was created!\x1B[0m", guild_name, channel.name);
        }
        // I need to go back to this.
        poise::Event::ChannelDelete { channel } => {
            let guild_name = channel.guild_id.name(ctx).unwrap_or("Unknown Guild".to_string());
            println!("\x1B[34m[{}] #{} was deleted!\x1B[0m", guild_name, channel.name);
        }
        poise::Event::ChannelUpdate { old, new } => {
            // Currently doesn't actually show a change, just announces the new name twice.
            if let Some(old_channel) = old {
                let old_channel_name = old_channel.id().name(ctx).await;

            let new_channel_name = new.id().name(ctx).await;

                println!(
                    "\x1B[34m#{}'s name updated to #{}!\x1B[0m",
                    old_channel_name.unwrap_or("Unknown Name".to_string()), new_channel_name.unwrap()
                );
            } else {
                // Should be unreachable, I just won't "fix" until I actually fix the issue above.
            }
        }

        // Will come back for threads when I cache them
        poise::Event::ThreadCreate { thread } => {
            let guild_id = thread.guild_id;

            let guild_name = if guild_id == 0 {
                "None".to_string()
            } else {
                if let Some(guild) = ctx.cache.guild(guild_id) {
                    guild.name.to_string()
                } else {
                    "Unknown".to_string()
                }
            };
            // Tell which channel it was created in.
            println!("\x1B[94m[{}] Thread #{} was created!\x1B[0m", guild_name, thread.name);

        }
        poise::Event::ThreadDelete { thread } => {
            let guild_id = thread.guild_id;
            let guild_cache = ctx.cache.guild(guild_id).unwrap();

            let threads = &guild_cache.threads;

            let mut channel_name = None;

            for thread_cache in threads {
                if thread_cache.id == thread.id {
                    channel_name = Some(thread_cache.name.clone());
                    break;
                }
            }
            let guild_name = if guild_id == 0 {
                "None".to_string()
            } else {
                if let Some(guild) = ctx.cache.guild(guild_id) {
                    guild.name.to_string()
                } else {
                    "Unknown".to_string()
                }
            };
            // Currently it won't know which thread was deleted because the method in which it is checked.
            // Tell which channel it was deleted from.
            if let Some(name) = channel_name {
                println!("\x1B[94m[{}] Thread '{}' was deleted!\x1B[0m", guild_name, name);
            } else {
                println!("\x1B[94m[{}] Thread with unknown name was deleted!\x1B[0m", guild_name);
            }
        }

        poise::Event::VoiceStateUpdate { old:_ , new: _ } => {
            // Oh this one will be fun..
            // Later me problem!
        }

        poise::Event::Ready { data_about_bot: _ } => {
            ctx.cache.set_max_messages(350);
            let _ = set_all_snippets(&data).await;
            // Need to check join tracks.
        }
        poise::Event::GuildMemberAddition { new_member } => {
            let guild_id = new_member.guild_id;
            let joined_user_id = new_member.user.id;
            let db_pool = &data.db;

            let guild_name = if guild_id == 0 {
                "None".to_string()
            } else {
                if let Some(guild) = ctx.cache.guild(guild_id) {
                    guild.name.to_string()
                } else {
                    "Unknown".to_string()
                }
            };

            println!("\x1B[33m[{}] {} (ID:{}) has joined!\x1B[0m", guild_name, new_member.user.name, joined_user_id);

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
        poise::Event::GuildMemberRemoval { guild_id, user, member_data_if_available: _ } => {
            let guild_id = guild_id;
            let guild_name = if *guild_id == 0 {
                "None".to_string()
            } else {
                if let Some(guild) = ctx.cache.guild(guild_id) {
                    guild.name.to_string()
                } else {
                    "Unknown".to_string()
                }
            };

            println!("\x1B[33m[{}] {} (ID:{}) has left!\x1B[0m", guild_name, user.name, user.id);
            // If the member data is available I guess print some stuff?

        }
        poise::Event::GuildMemberUpdate { old_if_available, new } => {
            if let Some(old_member) = old_if_available {
                let guild_id = new.guild_id;
                let guild_name = if guild_id == 0 {
                    "None".to_string()
                } else {
                    if let Some(guild) = ctx.cache.guild(guild_id) {
                        guild.name.to_string()
                    } else {
                        "Unknown".to_string()
                    }
                };
                let old_nickname = old_member.nick.as_deref().unwrap_or("None");
                let new_nickname = new.nick.as_deref().unwrap_or("None");
                if old_nickname != new_nickname {
                    println!("\x1B[92m[{}] Nickname change: {}: {} -> {}\x1B[0m", guild_name, new.user.name, old_nickname, new_nickname);
                }
                if old_member.user.name != new.user.name {
                    println!("\x1B[92mUsername change: {} -> {}\x1B[0m", old_member.user.name, new.user.name)
                }
            }
            // TODO: bump dependencies when merge happens and show display names.
        }


        // Only say the name changed if the name changed.
        // user updates
        // voice events
        _ => (),
    }

    Ok(())
}
