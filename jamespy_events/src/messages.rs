use jamespy_utils::misc::{get_channel_name, get_guild_name};

use poise::serenity_prelude::{
    self as serenity, ChannelId, Colour, CreateEmbedFooter, CreateMessage, GuildId, Message,
    MessageId, MessageUpdateEvent, Timestamp, UserId,
};

use crate::{Data, Error};

use chrono::NaiveDateTime;
use std::collections::HashMap;

use sqlx::query;

pub async fn message(
    ctx: &serenity::Context,
    new_message: Message,
    data: &Data,
) -> Result<(), Error> {
    let (no_log_user, no_log_channel, badlist, fixlist, patterns) = {
        let config = data.jamespy_config.read().unwrap().events_config.clone();

        (
            config.no_log_users.unwrap_or_default(),
            config.no_log_channels.unwrap_or_default(),
            config.badlist.unwrap_or_default(),
            config.fixlist.unwrap_or_default(),
            config.regex_patterns.unwrap_or_default(),
        )
    };

    if no_log_user.contains(&new_message.author.id.get())
        || no_log_channel.contains(&new_message.channel_id.get())
        || new_message.content.starts_with('$') && new_message.channel_id == 850342078034870302
    {
        return Ok(());
    }
    // mania
    if new_message.channel_id == 426392414429773835 {
        let mut cloned_messages = HashMap::new();
        if let Some(channel_messages) = ctx.cache.channel_messages(new_message.channel_id) {
            cloned_messages = channel_messages.clone();
        }

        let user_id = new_message.author.id;
        let mut found_match = false;
        let mut iter = cloned_messages.values().peekable();

        while let Some(message) = iter.next() {
            if iter.peek().is_none() {
                break;
            }
            if message.author.id == user_id {
                found_match = true;
                break;
            }
        }

        if !found_match
            && (Timestamp::now().timestamp() - new_message.author.created_at().timestamp())
                <= 604800
        {
            ChannelId::new(1164619284518010890)
                .send_message(
                    ctx,
                    CreateMessage::default().content(format!(
                        "New user <@{}> spotted talking in <#426392414429773835>! {}",
                        new_message.author.id,
                        new_message.link()
                    )),
                )
                .await?;
        }
    }

    let db_pool = &data.db;
    let guild_id = new_message.guild_id.unwrap_or_default();

    let guild_name = get_guild_name(ctx, guild_id);

    let channel_name = get_channel_name(&ctx.clone(), guild_id, new_message.channel_id).await;

    let mut any_pattern_matched = false;
    for pattern in patterns {
        if pattern.is_match(&new_message.content)
            && new_message.author.id != 158567567487795200
            && !new_message.author.bot()
        {
            any_pattern_matched = true;
            break;
        }
    }

    if any_pattern_matched {
        let user_id = UserId::from(158567567487795200);
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
        let user_id = UserId::from(158567567487795200);
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

    let attachments = new_message.attachments.clone();
    let attachments_fmt: Option<String> = if attachments.is_empty() {
        None
    } else {
        let attachment_names: Vec<String> = attachments
            .iter()
            .map(|attachment| attachment.filename.to_string())
            .collect();
        Some(format!(" <{}>", attachment_names.join(", ")))
    };

    let embeds = new_message.embeds.clone();
    let embeds_fmt: Option<String> = if embeds.is_empty() {
        None
    } else {
        let embed_types: Vec<String> = embeds
            .iter()
            .map(|embed| embed.kind.clone().unwrap_or_default().into_string())
            .collect();

        Some(format!(" {{{}}}", embed_types.join(", ")))
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
            // Check if the word is in the badlist and not in the fixlist
            let is_blacklisted = badlist.iter().any(|badword| {
                word.contains(badword) && !fixlist.iter().any(|fixword| word.contains(fixword))
            });

            is_blacklisted
        })
        .collect();

    if blacklisted_words.is_empty() {
        println!(
            "\x1B[90m[{}] [#{}]\x1B[0m {}: {}\x1B[36m{}{}\x1B[0m",
            guild_name,
            channel_name,
            new_message.author.name,
            new_message.content,
            attachments_fmt.as_deref().unwrap_or(""),
            embeds_fmt.as_deref().unwrap_or("")
        );
    } else {
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
    }
    let timestamp: NaiveDateTime =
        NaiveDateTime::from_timestamp_opt(new_message.timestamp.timestamp(), 0).unwrap();

    let _ = query!(
        "INSERT INTO msgs (guild_id, channel_id, message_id, user_id, content, attachments, \
         embeds, timestamp)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        i64::from(guild_id),
        new_message.channel_id.get() as i64,
        new_message.id.get() as i64,
        new_message.author.id.get() as i64,
        new_message.content.to_string(),
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

    let guild_id = event.guild_id.unwrap_or_default();
    let guild_name = get_guild_name(ctx, guild_id);
    let channel_name = get_channel_name(ctx, guild_id, event.channel_id).await;

    match (old_if_available, new) {
        (Some(old_message), Some(new_message)) => {
            if new_message.author.bot() {
                return Ok(());
            }

            if old_message.content != new_message.content {
                let attachments = new_message.attachments.clone();
                let attachments_fmt: Option<String> = if attachments.is_empty() {
                    None
                } else {
                    let attachment_names: Vec<String> = attachments
                        .iter()
                        .map(|attachment| attachment.filename.to_string())
                        .collect();
                    Some(format!(" <{}>", attachment_names.join(", ")))
                };

                let embeds = new_message.embeds.clone();
                let embeds_fmt: Option<String> = if embeds.is_empty() {
                    None
                } else {
                    let embed_types: Vec<String> = embeds
                        .iter()
                        .map(|embed| embed.kind.clone().unwrap_or_default().into_string())
                        .collect();

                    Some(format!(" {{{}}}", embed_types.join(", ")))
                };

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
                    "INSERT INTO msgs_edits (guild_id, channel_id, message_id, user_id, \
                     old_content, new_content, attachments, embeds, timestamp)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                    i64::from(guild_id),
                    new_message.channel_id.get() as i64,
                    new_message.id.get() as i64,
                    new_message.author.id.get() as i64,
                    old_message.content.to_string(),
                    new_message.content.to_string(),
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
    let guild_id_default = guild_id.unwrap_or_default();

    let guild_name = get_guild_name(ctx, guild_id_default);

    let channel_name = get_channel_name(ctx, guild_id_default, channel_id).await;

    // This works but might not be optimal.
    let message = ctx
        .cache
        .message(channel_id, deleted_message_id)
        .map(|message_ref| message_ref.clone());

    if let Some(message) = message {
        let user_name = message.author.name.clone();
        let content = message.content.clone();

        let attachments = message.attachments.clone();
        let attachments_fmt: Option<String> = if attachments.is_empty() {
            None
        } else {
            let attachment_names: Vec<String> = attachments
                .iter()
                .map(|attachment| attachment.filename.to_string())
                .collect();
            Some(format!(" <{}>", attachment_names.join(", ")))
        };

        let embeds = message.embeds.clone();
        let embeds_fmt: Option<String> = if embeds.is_empty() {
            None
        } else {
            let embed_types: Vec<String> = embeds
                .iter()
                .map(|embed| embed.kind.clone().unwrap_or_default().into_string())
                .collect();

            Some(format!(" {{{}}}", embed_types.join(", ")))
        };

        println!(
            "\x1B[91m\x1B[2m[{}] [#{}] A message from \x1B[0m{}\x1B[91m\x1B[2m was deleted: \
             {}{}{}\x1B[0m",
            guild_name,
            channel_name,
            user_name,
            content,
            attachments_fmt.as_deref().unwrap_or(""),
            embeds_fmt.as_deref().unwrap_or("")
        );

        let timestamp: NaiveDateTime = chrono::Utc::now().naive_utc();

        let _ = query!(
            "INSERT INTO msgs_deletions (guild_id, channel_id, message_id, user_id, content, \
             attachments, embeds, timestamp)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            i64::from(guild_id_default),
            message.channel_id.get() as i64,
            message.id.get() as i64,
            message.author.id.get() as i64,
            message.content.to_string(),
            attachments_fmt,
            embeds_fmt,
            timestamp
        )
        .execute(db_pool)
        .await;
        jamespy_utils::misc::download_attachments(message, data).await?;
    } else {
        println!(
            "\x1B[91m\x1B[2mA message (ID:{deleted_message_id}) was deleted but was not in \
             cache\x1B[0m"
        );
    }
    Ok(())
}
