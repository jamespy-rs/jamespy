use crate::utils::misc::{get_channel_name, get_guild_name, read_words_from_file};
use poise::serenity_prelude::{
    self as serenity, ChannelId, Colour, CreateEmbedFooter, GetMessages, GuildId, Message,
    MessageId, MessageUpdateEvent, UserId,
};

use crate::{Data, Error};

use chrono::NaiveDateTime;
use std::{
    collections::HashSet,
    num::NonZeroU64,
    sync::{Arc, RwLock},
};

use lazy_static::lazy_static;
use regex::Regex;
use sqlx::query;

static REGEX_PATTERNS: [&str; 2] = [
    r"(?i)\b\w*j\s*\d*\W*?a+\W*\d*\W*?m+\W*\d*\W*?e+\W*\d*\W*?s+\W*\d*\W*?\w*\b",
    r"(?i)\b\w*b\s*\d*\W*?t+\W*\d*\W*?3+\W*\d*\W*?6+\W*\d*\W*?5+\W*\d*\W*?\w*\b",
];

lazy_static! {
    // These are the channels in gg/osu specified for logging, I don't want to show these.
    static ref NO_LOG_CHANNEL: Vec<u64> = vec![
        572899947226333254,
        787623037834100737,
        697738506944118814,
        787389586665504778,
    ];
    // mudae and rin bot, these bots are typically spammy and serve no purpose being seen directly.
    static ref NO_LOG_USER: Vec<u64> = vec![432610292342587392, 429656936435286016];

    static ref COMPILED_PATTERNS: Vec<Regex> = REGEX_PATTERNS
    .iter()
    .map(|pattern| Regex::new(pattern).unwrap())
    .collect();

    pub static ref BADLIST: Arc<RwLock<HashSet<String>>> = {
        let words = read_words_from_file("badwords.txt");
        Arc::new(RwLock::new(words))
    };
    pub static ref FIXLIST: Arc<RwLock<HashSet<String>>> = {
        let words = read_words_from_file("fixwords.txt");
        Arc::new(RwLock::new(words))
    };

    pub static ref TRACK: RwLock<bool> = RwLock::new(true);
}

pub async fn message(
    ctx: &serenity::Context,
    new_message: Message,
    data: &Data,
) -> Result<(), Error> {
    if NO_LOG_USER.contains(&new_message.author.id.get())
        || NO_LOG_CHANNEL.contains(&new_message.channel_id.get())
        || new_message.content.starts_with('$') && new_message.channel_id == 850342078034870302
    {
        return Ok(());
    }

    let db_pool = &data.db;
    let guild_id = new_message.guild_id.unwrap_or_default();

    let guild_name = get_guild_name(ctx, guild_id);

    let channel_name = get_channel_name(&ctx.clone(), guild_id, new_message.channel_id).await;

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
        let user = user_id.to_user(ctx.clone()).await?;
        user.dm(
            &ctx.clone(),
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
    if guild_id == 1
        && ![
            158567567487795200,
            ctx.clone().cache.current_user().id.get(),
        ]
        .contains(&new_message.author.id.get())
    {
        let user_id = UserId::from(NonZeroU64::new(158567567487795200).unwrap());
        let user = user_id.to_user(ctx.clone()).await?;
        user.dm(
            &ctx.clone(),
            serenity::CreateMessage::default()
                .content(format!(
                    "{} (ID:{}) messaged me",
                    new_message.author.name, new_message.author.id
                ))
                .embed(
                    serenity::CreateEmbed::default()
                        .title("I was messaged!")
                        .description(format!(
                            "**{}**: {}",
                            new_message.author.name, new_message.content
                        ))
                        .color(Colour::from_rgb(0, 255, 0))
                        .footer(CreateEmbedFooter::new(format!(
                            "{}",
                            new_message.channel_id
                        ))),
                ),
        )
        .await?;
    }

    // For gavin.
    if *TRACK.read().unwrap() && new_message.author.id.get() == 221026934287499264 {
        let builder = GetMessages::new()
            .before(MessageId::new(new_message.id.get()))
            .limit(100);
        let messages = new_message
            .channel_id
            .messages(ctx.clone(), builder)
            .await?;

        let user_has_spoken = messages
            .iter()
            .any(|msg| msg.author.id == 221026934287499264);
        if !user_has_spoken {
            let user_id = UserId::from(NonZeroU64::new(646202688148865024).unwrap());
            let user = user_id.to_user(ctx.clone()).await?;
            user.dm(
                ctx.clone(),
                serenity::CreateMessage::default().content(format!(
                    "<@221026934287499264> (ID:221026934287499264> was spotted in **#{}** {}",
                    new_message
                        .channel_id
                        .name(ctx.clone())
                        .await
                        .unwrap_or("I broke".to_owned()),
                    new_message.link()
                )),
            )
            .await?;
        }
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
            let badlist = BADLIST.read().unwrap();
            let fixlist = FIXLIST.read().unwrap();

            // Check if the word is in the BADLIST and not in the FIXLIST
            let is_blacklisted = badlist.iter().any(|badword| {
                word.contains(badword) && !fixlist.iter().any(|fixword| word.contains(fixword))
            });

            is_blacklisted
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
    let timestamp: NaiveDateTime =
        NaiveDateTime::from_timestamp_opt(new_message.timestamp.timestamp(), 0).unwrap();

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
            .execute(db_pool)
            .await;

    Ok(())
}

pub async fn message_edit(
    ctx: &serenity::Context,
    old_if_available: Option<Message>,
    new: Option<Message>,
    event: MessageUpdateEvent,
    data: &Data,
) -> Result<(), Error> {
    let db_pool = &data.db;
    match (old_if_available, new) {
        (Some(old_message), Some(new_message)) => {
            if new_message.author.bot {
                return Ok(());
            }
            if old_message.content != new_message.content {
                let guild_id = new_message.guild_id.unwrap_or_default();
                let guild_name = get_guild_name(ctx, guild_id);

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

                let channel_name = get_channel_name(ctx, guild_id, new_message.channel_id).await;
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
                let timestamp: NaiveDateTime = NaiveDateTime::from_timestamp_opt(
                    new_message.edited_timestamp.unwrap().timestamp(),
                    0,
                )
                .unwrap();

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
                .execute(db_pool)
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
    Ok(())
}

pub async fn message_delete(
    ctx: &serenity::Context,
    channel_id: ChannelId,
    deleted_message_id: MessageId,
    guild_id: Option<GuildId>,
    data: &Data,
) -> Result<(), Error> {
    let db_pool = &data.db;
    let guild_id = guild_id.unwrap_or_default();

    let guild_name = get_guild_name(ctx, guild_id);

    let channel_name = get_channel_name(ctx, guild_id, channel_id).await;

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
            .execute(db_pool)
            .await;
    } else {
        println!(
            "\x1B[91m\x1B[2mA message (ID:{}) was deleted but was not in cache\x1B[0m",
            deleted_message_id
        );
    }
    Ok(())
}
