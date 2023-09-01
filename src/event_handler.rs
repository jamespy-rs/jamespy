use std::collections::HashSet;
use std::num::NonZeroU64;

use lazy_static::lazy_static;
use poise::serenity_prelude::{self as serenity, Colour};
use poise::serenity_prelude::{ChannelId, GuildId, UserId};

use regex::Regex;
use sqlx::query;

use crate::utils;
use crate::Data;
use crate::Error;

use chrono::NaiveDateTime;

use utils::snippets::*;

async fn get_channel_name(
    ctx: &serenity::Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> String {
    let mut channel_name = channel_id
        .name(ctx)
        .await
        .unwrap_or("Unknown Channel".to_owned());

    if guild_id.get() != 0 && channel_name == "Unknown Channel" {
        let guild_cache = ctx.cache.guild(guild_id).unwrap();
        let threads = &guild_cache.threads;
        for thread in threads {
            if thread.id == channel_id.get() {
                channel_name = thread.name.clone();
                break;
            }
        }
    }

    channel_name
}

static REGEX_PATTERNS: [&str; 2] = [
    r"(?i)\b\w*j\s*\d*\W*?a+\W*\d*\W*?m+\W*\d*\W*?e+\W*\d*\W*?s+\W*\d*\W*?\w*\b",
    r"(?i)\b\w*b\s*\d*\W*?t+\W*\d*\W*?3+\W*\d*\W*?6+\W*\d*\W*?5+\W*\d*\W*?\w*\b",
];

fn read_words_from_file(filename: &str) -> HashSet<String> {
    std::fs::read_to_string(filename)
        .expect("Failed to read the file")
        .lines()
        .map(|line| line.trim().to_lowercase())
        .collect()
}
lazy_static! {
    static ref BADLIST: HashSet<String> = read_words_from_file("badwords.txt");
    static ref FIXLIST: HashSet<String> = read_words_from_file("fixwords.txt");
}

lazy_static! {
    static ref COMPILED_PATTERNS: Vec<Regex> = REGEX_PATTERNS
        .iter()
        .map(|pattern| Regex::new(pattern).unwrap())
        .collect();
}

pub async fn event_handler(
    event: serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    let no_log_user: Vec<u64> = vec![432610292342587392, 429656936435286016]; // mudae and rin bot
    let no_log_channel: Vec<u64> = vec![
        572899947226333254,
        787623037834100737,
        697738506944118814,
        787389586665504778,
    ]; // log channels in gg/osu
    match event {
        serenity::FullEvent::Message { ctx, new_message } => {
            // Removes mudae commands in the mudae channel in gg/osu, alongside other criteria above.
            if no_log_user.contains(&new_message.author.id.get())
                || no_log_channel.contains(&new_message.channel_id.get())
                || new_message.content.starts_with("$")
                    && new_message.channel_id == 850342078034870302
            {
                return Ok(());
            }

            let db_pool = &data.db;
            let guild_id = new_message.guild_id.unwrap_or_default();

            let ctx_clone = ctx.clone();

            let guild_name = if guild_id == 1 {
                "None".to_owned()
            } else {
                match guild_id.name(ctx.clone()) {
                    Some(name) => name,
                    None => "Unknown".to_owned(),
                }
            };

            let channel_name = get_channel_name(&ctx, guild_id, new_message.channel_id).await;

            let mut any_pattern_matched = false;
            for pattern in &*COMPILED_PATTERNS {
                if pattern.is_match(&new_message.content)
                    && new_message.author.id != 158567567487795200
                    && !new_message.author.bot
                {
                    any_pattern_matched = true;
                    break;
                }
            }

            if any_pattern_matched {
                let user_id = UserId::from(NonZeroU64::new(158567567487795200).unwrap());
                let user = user_id.to_user(ctx).await?;
                user.dm(
                    &ctx_clone,
                    serenity::CreateMessage::default()
                        .content(format!(
                            "In {} <#{}> you were mentioned by {} (ID:{})",
                            guild_name,
                            new_message.channel_id,
                            new_message.author.name,
                            new_message.author.id
                        ))
                        .embed(
                            serenity::CreateEmbed::default()
                                .title("A pattern was matched!")
                                .description(format!(
                                    "<#{}> by **{}** {}\n\n [Jump to message!]({})",
                                    new_message.channel_id,
                                    new_message.author.name,
                                    new_message.content,
                                    new_message.link()
                                ))
                                .color(Colour::from_rgb(0, 255, 0)),
                        ),
                )
                .await?;
            }

            let attachments = new_message.attachments.clone();
            let attachments_fmt: Option<String> = if !attachments.is_empty() {
                let attachment_names: Vec<String> = attachments
                    .iter()
                    .map(|attachment| attachment.filename.clone())
                    .collect();
                Some(format!(" <{}>", attachment_names.join(", ")))
            } else {
                None
            };

            let embeds = new_message.embeds.clone();
            let embeds_fmt: Option<String> = if !embeds.is_empty() {
                let embed_types: Vec<String> = embeds
                    .iter()
                    .map(|embed| embed.kind.clone().unwrap_or("Unknown Type".to_string()))
                    .collect();

                Some(format!(" {{{}}}", embed_types.join(", ")))
            } else {
                None
            };
            let messagewords: Vec<String> = new_message
                .content
                .to_lowercase()
                .split_whitespace()
                .map(String::from)
                .collect();

            let blacklisted_words: Vec<&String> = messagewords
                .iter()
                .filter(|word| {
                    BADLIST
                        .iter()
                        .any(|badword| word.contains(badword) && !FIXLIST.contains(*word))
                })
                .collect();

            if !blacklisted_words.is_empty() {
                let flagged_words: Vec<String> = blacklisted_words
                    .iter()
                    .map(|word| (*word).clone())
                    .collect();
                println!(
                    "Flagged for bad word(s): \x1B[1m\x1B[31m{}\x1B[0m",
                    flagged_words.join(", ")
                );
                println!(
                    "\x1B[90m[{}] [#{}]\x1B[0m {}: \x1B[1m\x1B[31m{}{}{}\x1B[0m",
                    guild_name,
                    channel_name,
                    new_message.author.name,
                    new_message.content,
                    attachments_fmt.as_deref().unwrap_or(""),
                    embeds_fmt.as_deref().unwrap_or("")
                );
            } else {
                println!(
                    "\x1B[90m[{}] [#{}]\x1B[0m {}: {}\x1B[36m{}{}\x1B[0m",
                    guild_name,
                    channel_name,
                    new_message.author.name,
                    new_message.content,
                    attachments_fmt.as_deref().unwrap_or(""),
                    embeds_fmt.as_deref().unwrap_or("")
                );
            }
            let timestamp: NaiveDateTime = NaiveDateTime::from_timestamp_opt(new_message.timestamp.timestamp(), 0).unwrap();

            let _ = query!(
                "INSERT INTO msgs (guild_id, channel_id, message_id, user_id, content, attachments, embeds, timestamp)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                i64::from(guild_id),
                new_message.channel_id.get() as i64,
                new_message.id.get() as i64,
                new_message.author.id.get() as i64,
                new_message.content,
                attachments_fmt,
                embeds_fmt,
                timestamp
            )
            .execute(&*db_pool)
            .await;
        }
        serenity::FullEvent::MessageUpdate {
            ctx,
            old_if_available,
            new,
            event,
        } => {
            let db_pool = &data.db;
            match (old_if_available, new) {
                (Some(old_message), Some(new_message)) => {
                    if new_message.author.bot {
                        return Ok(());
                    }
                    if old_message.content != new_message.content {
                        let guild_id = new_message.guild_id.unwrap_or_default();

                        let guild_name = if guild_id == 1 {
                            "None".to_owned()
                        } else {
                            match guild_id.name(ctx.clone()) {
                                Some(name) => name,
                                None => "Unknown".to_owned(),
                            }
                        };
                        let attachments = new_message.attachments.clone();
                        let attachments_fmt: Option<String> = if !attachments.is_empty() {
                            let attachment_names: Vec<String> = attachments
                                .iter()
                                .map(|attachment| attachment.filename.clone())
                                .collect();
                            Some(format!(" <{}>", attachment_names.join(", ")))
                        } else {
                            None
                        };

                        let embeds = new_message.embeds.clone();
                        let embeds_fmt: Option<String> = if !embeds.is_empty() {
                            let embed_types: Vec<String> = embeds
                                .iter()
                                .map(|embed| {
                                    embed.kind.clone().unwrap_or("Unknown Type".to_string())
                                })
                                .collect();

                            Some(format!(" {{{}}}", embed_types.join(", ")))
                        } else {
                            None
                        };

                        let channel_name =
                            get_channel_name(&ctx, guild_id, new_message.channel_id).await;
                        println!(
                            "\x1B[36m[{}] [#{}] A message by \x1B[0m{}\x1B[36m was edited:",
                            guild_name, channel_name, new_message.author.name
                        );
                        println!(
                            "BEFORE: {}: {}",
                            new_message.author.name, old_message.content
                        ); // potentially check old attachments in the future.
                        println!(
                            "AFTER: {}: {}{}{}\x1B[0m",
                            new_message.author.name,
                            new_message.content,
                            attachments_fmt.as_deref().unwrap_or(""),
                            embeds_fmt.as_deref().unwrap_or("")
                        );
                        // We will see if this ever panics.
                        let timestamp: NaiveDateTime = NaiveDateTime::from_timestamp_opt(new_message.edited_timestamp.unwrap().timestamp(), 0).unwrap();

                        let _ = query!(
                            "INSERT INTO msgs_edits (guild_id, channel_id, message_id, user_id, old_content, new_content, attachments, embeds, timestamp)
                             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                            i64::from(guild_id),
                            new_message.channel_id.get() as i64,
                            new_message.id.get() as i64,
                            new_message.author.id.get() as i64,
                            old_message.content,
                            new_message.content,
                            attachments_fmt,
                            embeds_fmt,
                            timestamp
                        )
                        .execute(&*db_pool)
                        .await;
                    }
                }
                (None, None) => {
                    println!(
                        "\x1B[36mA message (ID:{}) was edited but was not in cache\x1B[0m",
                        event.id
                    );
                }
                _ => {}
            }
        }
        serenity::FullEvent::MessageDelete {
            ctx,
            channel_id,
            deleted_message_id,
            guild_id,
        } => {
            let db_pool = &data.db;
            let guild_id = guild_id.unwrap_or_default();
            let channel_id = channel_id;

            let guild_name = if guild_id == 1 {
                "None".to_owned()
            } else {
                match guild_id.name(ctx.clone()) {
                    Some(name) => name,
                    None => "Unknown".to_owned(),
                }
            };

            let channel_name = get_channel_name(&ctx, guild_id, channel_id).await;

            if let Some(message) = ctx.cache.message(channel_id, deleted_message_id) {
                let user_name = message.author.name.clone();
                let content = message.content.clone();

                let attachments = message.attachments.clone();
                let attachments_fmt: Option<String> = if !attachments.is_empty() {
                    let attachment_names: Vec<String> = attachments
                        .iter()
                        .map(|attachment| attachment.filename.clone())
                        .collect();
                    Some(format!(" <{}>", attachment_names.join(", ")))
                } else {
                    None
                };

                let embeds = message.embeds.clone();
                let embeds_fmt: Option<String> = if !embeds.is_empty() {
                    let embed_types: Vec<String> = embeds
                        .iter()
                        .map(|embed| embed.kind.clone().unwrap_or("Unknown Type".to_string()))
                        .collect();

                    Some(format!(" {{{}}}", embed_types.join(", ")))
                } else {
                    None
                };

                println!("\x1B[91m\x1B[2m[{}] [#{}] A message from \x1B[0m{}\x1B[91m\x1B[2m was deleted: {}{}{}\x1B[0m",
                    guild_name, channel_name, user_name, content, attachments_fmt.as_deref().unwrap_or(""), embeds_fmt.as_deref().unwrap_or(""));

                let timestamp: NaiveDateTime = chrono::Utc::now().naive_utc();

                let _ = query!(
                        "INSERT INTO msgs_deletions (guild_id, channel_id, message_id, user_id, content, attachments, embeds, timestamp)
                         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                        i64::from(guild_id),
                        message.channel_id.get() as i64,
                        message.id.get() as i64,
                        message.author.id.get() as i64,
                        message.content,
                        attachments_fmt,
                        embeds_fmt,
                        timestamp
                    )
                    .execute(&*db_pool)
                    .await;
            } else {
                println!(
                    "\x1B[91m\x1B[2mA message (ID:{}) was deleted but was not in cache\x1B[0m",
                    deleted_message_id
                );
            }
        }
        // need serenity:FullEvent::MessageDeleteBulk
        serenity::FullEvent::GuildCreate { ctx, guild, is_new } => {
            if let Some(true) = is_new {
                println!(
                    "\x1B[33mJoined {}!\nNow in {} guild(s)\x1B[0m",
                    guild.name,
                    ctx.cache.guilds().len()
                );
            }
        }

        serenity::FullEvent::ReactionAdd { ctx, add_reaction } => {
            let user_id = add_reaction.user_id.unwrap();
            if ctx.cache.user(user_id).map_or(false, |user| user.bot) {
                return Ok(());
                // May merge with the one below.
            }
            let guild_id = add_reaction.guild_id.unwrap_or_default();
            let guild_name = if guild_id == 1 {
                "None".to_owned()
            } else {
                match guild_id.name(ctx.clone()) {
                    Some(name) => name,
                    None => "Unknown".to_owned(),
                }
            };
            let channel_name = get_channel_name(&ctx, guild_id, add_reaction.channel_id).await;

            let user_name = match user_id.to_user(ctx).await {
                Ok(user) => user.name,
                Err(_) => "Unknown User".to_string(),
            };

            println!(
                "\x1B[95m[{}] [#{}] {} added a reaction: {}\x1B[0m",
                guild_name, channel_name, user_name, add_reaction.emoji
            );
        }
        serenity::FullEvent::ReactionRemove {
            ctx,
            removed_reaction,
        } => {
            let user_id = removed_reaction.user_id.unwrap();
            if ctx.cache.user(user_id).map_or(false, |user| user.bot) {
                return Ok(());
                // May merge with the one below.
            }
            let guild_id = removed_reaction.guild_id.unwrap_or_default();
            let guild_name = if guild_id == 1 {
                "None".to_owned()
            } else {
                match guild_id.name(ctx.clone()) {
                    Some(name) => name,
                    None => "Unknown".to_owned(),
                }
            };
            let channel_name = get_channel_name(&ctx, guild_id, removed_reaction.channel_id).await;

            let user_name = match user_id.to_user(&ctx.http).await {
                Ok(user) => user.name,
                Err(_) => "Unknown User".to_string(),
            };

            println!(
                "\x1B[95m[{}] [#{}] {} removed a reaction: {}\x1B[0m",
                guild_name, channel_name, user_name, removed_reaction.emoji
            );
        }
        serenity::FullEvent::ReactionRemoveAll {
            ctx: _,
            channel_id: _,
            removed_from_message_id: _,
        } => {
            // Need to do the funny here.
            // Will leave it untouched until I have a better codebase.
        }
        serenity::FullEvent::ChannelCreate { ctx, channel } => {
            let guild_name = channel
                .guild_id
                .name(ctx)
                .unwrap_or("Unknown Guild".to_string());
            println!(
                "\x1B[34m[{}] #{} was created!\x1B[0m",
                guild_name, channel.name
            );
        }
        serenity::FullEvent::ChannelDelete {
            ctx,
            channel,
            messages: _,
        } => {
            let guild_name = channel
                .guild_id
                .name(ctx)
                .unwrap_or("Unknown Guild".to_string());
            println!(
                "\x1B[34m[{}] #{} was deleted!\x1B[0m",
                guild_name, channel.name
            );
        }

        serenity::FullEvent::ThreadCreate { ctx, thread } => {
            let guild_id = thread.guild_id;

            let guild_name = if guild_id == 1 {
                "None".to_owned()
            } else {
                match guild_id.name(ctx.clone()) {
                    Some(name) => name,
                    None => "Unknown".to_owned(),
                }
            };
            // Tell which channel it was created in.
            println!(
                "\x1B[94m[{}] Thread #{} was created!\x1B[0m",
                guild_name, thread.name
            );
        }
        serenity::FullEvent::ThreadDelete { ctx, thread } => {
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
            let guild_name = if guild_id == 1 {
                "None".to_owned()
            } else {
                match guild_id.name(ctx.clone()) {
                    Some(name) => name,
                    None => "Unknown".to_owned(),
                }
            };
            // Currently it won't know which thread was deleted because the method in which it is checked.
            // Tell which channel it was deleted from.
            if let Some(name) = channel_name {
                println!(
                    "\x1B[94m[{}] Thread '{}' was deleted!\x1B[0m",
                    guild_name, name
                );
            } else {
                println!(
                    "\x1B[94m[{}] Thread with unknown name was deleted!\x1B[0m",
                    guild_name
                );
            }
        }
        serenity::FullEvent::VoiceStateUpdate {
            ctx,
            old,
            new,
        } => {
            if let Some(old) = old {
                if old.channel_id != new.channel_id && new.channel_id != None {

                    let mut guild_name = String::from("Unknown");
                    let mut user_name = String::from("Unknown User");
                    let mut old_channel = String::from("Unknown");
                    let mut old_channel_id_ = String::from("Unknown");
                    let mut new_channel = String::from("Unknown");
                    let mut new_channel_id_ = String::from("Unknown");

                    if let Some(guild_id) = old.guild_id {
                        guild_name = guild_id.name(ctx.clone()).unwrap_or_else(|| guild_name.clone());
                    }
                    if let Some(member) = new.member {
                        user_name = member.user.name;
                    }
                    if let Some(old_channel_id) = old.channel_id {
                        old_channel_id_ = old_channel_id.get().to_string();
                        if let Ok(channel_name) = old_channel_id.name(ctx.clone()).await {
                            old_channel = channel_name;

                        } else {
                            old_channel = "Unknown".to_owned();
                        }
                    }
                    if let Some(new_channel_id) = new.channel_id {
                        new_channel_id_ = new_channel_id.get().to_string();
                        if let Ok(channel_name) = new_channel_id.name(ctx.clone()).await {
                            new_channel = channel_name;

                        } else {
                            new_channel = "Unknown".to_owned();
                        }
                    }
                    println!("\x1B[32m[{}] {}: {} (ID:{}) -> {} (ID:{})\x1B[0m", guild_name, user_name, old_channel, old_channel_id_, new_channel, new_channel_id_)
                } else {
                    if new.channel_id == None {
                        let mut guild_name = String::from("Unknown");
                        let mut user_name = String::from("Unknown User");
                        let mut old_channel = String::from("Unknown");
                        let mut old_channel_id_ = String::from("Unknown");

                        if let Some(guild_id) = old.guild_id {
                            guild_name = guild_id.name(ctx.clone()).unwrap_or_else(|| guild_name.clone());
                        }
                        if let Some(member) = new.member {
                            user_name = member.user.name;
                        }
                        if let Some(old_channel_id) = old.channel_id {
                            old_channel_id_ = old_channel_id.get().to_string();
                            if let Ok(channel_name) = old_channel_id.name(ctx.clone()).await {
                                old_channel = channel_name;

                            } else {
                                old_channel = "Unknown".to_owned();
                            }
                        }
                        println!("\x1B[32m[{}] {} left {} (ID:{})\x1B[0m", guild_name, user_name, old_channel, old_channel_id_)
                    } else {
                        // mutes, unmutes, deafens, etc are here.
                    }
                }
            } else {
                let mut guild_name = String::from("Unknown");
                let mut user_name = String::from("Unknown User");
                let mut new_channel = String::from("Unknown");
                let mut new_channel_id_ = String::from("Unknown");

                if let Some(guild_id) = new.guild_id {
                    guild_name = guild_id.name(ctx.clone()).unwrap_or_else(|| guild_name.clone());
                }
                if let Some(member) = new.member {
                    user_name = member.user.name;
                }
                if let Some(new_channel_id) = new.channel_id {
                    new_channel_id_ = new_channel_id.get().to_string();
                    if let Ok(channel_name) = new_channel_id.name(ctx.clone()).await {
                        new_channel = channel_name;

                    } else {
                        new_channel = "Unknown".to_owned();
                    }
                }

                println!("\x1B[32m[{}] {} joined {} (ID:{})\x1B[0m", guild_name, user_name, new_channel, new_channel_id_);
        }
    }

        serenity::FullEvent::Ready {
            ctx,
            data_about_bot: _,
        } => {
            ctx.cache.set_max_messages(350);
            let _ = set_all_snippets(&data).await;
            // Need to check join tracks.
        }
        serenity::FullEvent::GuildMemberAddition { ctx, new_member } => {
            let guild_id = new_member.guild_id;
            let joined_user_id = new_member.user.id;

            let guild_name = if guild_id == 1 {
                "None".to_owned()
            } else {
                match guild_id.name(ctx.clone()) {
                    Some(name) => name,
                    None => "Unknown".to_owned(),
                }
            };
            println!(
                "\x1B[33m[{}] {} (ID:{}) has joined!\x1B[0m",
                guild_name, new_member.user.name, joined_user_id
            );
        }
        serenity::FullEvent::GuildMemberRemoval {
            ctx,
            guild_id,
            user,
            member_data_if_available: _,
        } => {
            let guild_id = guild_id;
            let guild_name = if guild_id == 1 {
                "None".to_owned()
            } else {
                match guild_id.name(ctx.clone()) {
                    Some(name) => name,
                    None => "Unknown".to_owned(),
                }
            };

            println!(
                "\x1B[33m[{}] {} (ID:{}) has left!\x1B[0m",
                guild_name, user.name, user.id
            );
        }
        serenity::FullEvent::GuildMemberUpdate {
            ctx,
            old_if_available,
            new,
            event,
        } => {
            if let Some(old_member) = old_if_available {
                if let Some(new_member) = new {
                    let guild_id = event.guild_id;
                    let guild_name = if guild_id == 1 {
                        "None".to_owned()
                    } else {
                        match guild_id.name(ctx.clone()) {
                            Some(name) => name,
                            None => "Unknown".to_owned(),
                        }
                    };

                    let old_nickname = old_member.nick.as_deref().unwrap_or("None");
                    let new_nickname = new_member.nick.as_deref().unwrap_or("None");

                    if old_nickname != new_nickname {
                        println!(
                            "\x1B[92m[{}] Nickname change: {}: {} -> {} (ID:{})\x1B[0m",
                            guild_name,
                            new_member.user.name,
                            old_nickname,
                            new_nickname,
                            new_member.user.id
                        );
                    };

                    if old_member.user.name != new_member.user.name {
                        println!(
                            "\x1B[92mUsername change: {} -> {} (ID:{})\x1B[0m",
                            old_member.user.name, new_member.user.name, new_member.user.id
                        );
                    }
                    if old_member.user.global_name != new_member.user.global_name {
                        println!("\x1B[92mDisplay name change: {}: {} -> {} (ID:{})\x1B[0m", old_member.user.name, old_member.user.global_name.unwrap_or("None".to_owned()), new_member.user.global_name.unwrap_or("None".to_owned()), new_member.user.id)
                    }
                }
            }
        }
        _ => (),
    }

    Ok(())
}
