use std::collections::HashSet;
use std::fmt::Write;
use std::sync::Arc;

use crate::helper::{get_channel_name, get_guild_name, get_guild_name_override};
use crate::{Data, Error};

use chrono::NaiveDateTime;
use poise::serenity_prelude::{
    self as serenity, ChannelId, Colour, CreateEmbedFooter, GuildId, Message, MessageId,
    MessageUpdateEvent, UserId,
};
use sqlx::query;

pub async fn message(ctx: &serenity::Context, msg: &Message, data: Arc<Data>) -> Result<(), Error> {
    let config = { data.config.read().events.clone() };

    if should_skip_msg(config.no_log_users, config.no_log_channels, msg) {
        return Ok(());
    }

    data.check_or_insert_user(msg.author.id, msg.author.tag())
        .await;

    let guild_id = msg.guild_id;
    let guild_name = get_guild_name_override(ctx, &data, guild_id);
    let channel_name = get_channel_name(ctx, guild_id, msg.channel_id).await;

    if let Some(patterns) = config.regex {
        check_event_dm_regex(ctx, msg, &get_guild_name(ctx, guild_id), &patterns).await?;
    };

    handle_dm(ctx, msg).await?;

    let (attachments, embeds) = attachments_embed_fmt(msg);

    let author_string = author_string(ctx, msg);

    // TODO: fix this before working on the bot after rewrite.
    let flagged_words = get_blacklisted_words(
        msg,
        &config.badlist.unwrap_or_default(),
        &config.fixlist.unwrap_or_default(),
    );

    let flagged_str = if flagged_words.is_empty() {
        ""
    } else {
        println!(
            "Flagged for bad word(s): \x1B[1m\x1B[31m{}\x1B[0m",
            flagged_words.join(", ")
        );
        "\x1B[1m\x1B[31m"
    };

    println!(
        "\x1B[90m[{guild_name}] [#{channel_name}]\x1B[0m {author_string}: \
         {flagged_str}{}\x1B[36m{}{}\x1B[0m",
        msg.content,
        attachments.as_deref().unwrap_or(""),
        embeds.as_deref().unwrap_or("")
    );

    let timestamp: NaiveDateTime =
        NaiveDateTime::from_timestamp_opt(msg.timestamp.timestamp(), 0).unwrap();

    let _ = query!(
        "INSERT INTO msgs (guild_id, channel_id, message_id, user_id, content, attachments, \
         embeds, timestamp)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        guild_id.map(|g| i64::from(g)),
        msg.channel_id.get() as i64,
        msg.id.get() as i64,
        msg.author.id.get() as i64,
        msg.content.to_string(),
        attachments,
        embeds,
        timestamp
    )
    .execute(&data.db)
    .await;

    Ok(())
}

pub async fn message_edit(
    ctx: &serenity::Context,
    old_if_available: &Option<Message>,
    new: &Option<Message>,
    event: &MessageUpdateEvent,
    data: Arc<Data>,
) -> Result<(), Error> {
    let db_pool = &data.db;

    let guild_id = event.guild_id;
    let guild_name = get_guild_name_override(ctx, &data, guild_id);
    let channel_name = get_channel_name(ctx, guild_id, event.channel_id).await;

    // I can probably just check event instead, it probably has what i need.
    match (old_if_available, new) {
        (Some(old_message), Some(new_message)) => {
            if new_message.author.bot() {
                return Ok(());
            }

            if old_message.content != new_message.content {
                let (attachments, embeds) = attachments_embed_fmt(new_message);

                println!(
                    "\x1B[36m[{}] [#{}] A message by \x1B[0m{}\x1B[36m was edited:",
                    guild_name,
                    channel_name,
                    new_message.author.tag()
                );
                println!(
                    "BEFORE: {}: {}",
                    new_message.author.tag(),
                    old_message.content
                ); // potentially check old attachments in the future.
                println!(
                    "AFTER: {}: {}{}{}\x1B[0m",
                    new_message.author.tag(),
                    new_message.content,
                    attachments.as_deref().unwrap_or(""),
                    embeds.as_deref().unwrap_or("")
                );

                let timestamp: NaiveDateTime = NaiveDateTime::from_timestamp_opt(
                    new_message.edited_timestamp.unwrap().timestamp(),
                    0,
                )
                .unwrap();

                let _ = query!(
                    "INSERT INTO msgs_edits (guild_id, channel_id, message_id, user_id, \
                     old_content, new_content, attachments, embeds, timestamp)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                    guild_id.map(|g| i64::from(g)),
                    new_message.channel_id.get() as i64,
                    new_message.id.get() as i64,
                    new_message.author.id.get() as i64,
                    old_message.content.to_string(),
                    new_message.content.to_string(),
                    attachments,
                    embeds,
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
    channel_id: &ChannelId,
    deleted_message_id: &MessageId,
    guild_id: &Option<GuildId>,
    data: Arc<Data>,
) -> Result<(), Error> {
    let db_pool = &data.db;

    let guild_name = get_guild_name_override(ctx, &data, *guild_id);

    let channel_name = get_channel_name(ctx, *guild_id, *channel_id).await;

    // This works but might not be optimal.
    let message = ctx
        .cache
        .message(*channel_id, *deleted_message_id)
        .map(|message_ref| message_ref.clone());

    if let Some(message) = message {
        let user_name = message.author.tag();
        let content = message.content.clone();

        let (attachments_fmt, embeds_fmt) = attachments_embed_fmt(&message);

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
            guild_id.map(|g| i64::from(g)),
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
        crate::attachments::download_attachments(ctx, message, &data).await?;
    } else {
        println!(
            "\x1B[91m\x1B[2mA message (ID:{deleted_message_id}) was deleted but was not in \
             cache\x1B[0m"
        );
    }
    Ok(())
}

fn should_skip_msg(
    no_log_users: Option<Vec<u64>>,
    no_log_channels: Option<Vec<u64>>,
    message: &Message,
) -> bool {
    let user_condition = no_log_users.is_some_and(|vec| vec.contains(&message.author.id.get()));

    let channel_condition =
        no_log_channels.is_some_and(|vec| vec.contains(&message.channel_id.get()));

    // ignore commands in mudae channel.
    let mudae_cmd = message.content.starts_with('$') && message.channel_id == 850342078034870302;

    user_condition || channel_condition || mudae_cmd
}

async fn check_event_dm_regex(
    ctx: &serenity::Context,
    msg: &Message,
    guild_name: &str,
    patterns: &[regex::Regex],
) -> Result<(), Error> {
    if patterns.iter().any(|pattern| {
        pattern.is_match(&msg.content) && msg.author.id != 158567567487795200 && !msg.author.bot()
    }) {
        pattern_matched(ctx, msg, guild_name).await?;
        return Ok(());
    }

    Ok(())
}

async fn pattern_matched(ctx: &serenity::Context, msg: &Message, guild: &str) -> Result<(), Error> {
    // TODO: use fw owner's or make configurable.
    let user_id = UserId::from(158567567487795200);
    let user = user_id.to_user(ctx.clone()).await?;

    let embed = serenity::CreateEmbed::default()
        .title("A pattern was matched!")
        .description(format!(
            "<#{}> by **{}** {}\n\n [Jump to message!]({})",
            msg.channel_id,
            msg.author.tag(),
            msg.content,
            msg.link()
        ))
        .color(Colour::from_rgb(0, 255, 0));

    let msg = serenity::CreateMessage::default()
        .content(format!(
            "In {} <#{}> you were mentioned by {} (ID:{})",
            guild,
            msg.channel_id,
            msg.author.tag(),
            msg.author.id
        ))
        .embed(embed);

    user.dm(&ctx, msg).await?;

    Ok(())
}

async fn handle_dm(ctx: &serenity::Context, msg: &Message) -> Result<(), Error> {
    // TODO: use fw owner's or make configurable.
    if msg.guild_id.is_some()
        || [158567567487795200, ctx.cache.current_user().id.get()].contains(&msg.author.id.get())
    {
        return Ok(());
    }

    let user_id = UserId::from(158567567487795200);
    let user = user_id.to_user(ctx.clone()).await?;

    let embed = serenity::CreateEmbed::default()
        .title("I was messaged!")
        .description(format!("**{}**: {}", msg.author.tag(), msg.content))
        .color(Colour::from_rgb(0, 255, 0))
        .footer(CreateEmbedFooter::new(format!("{}", msg.channel_id)));

    let msg = serenity::CreateMessage::default()
        .content(format!(
            "{} (ID:{}) messaged me",
            msg.author.tag(),
            msg.author.id
        ))
        .embed(embed);

    user.dm(&ctx.clone(), msg).await?;
    Ok(())
}

fn attachments_embed_fmt(new_message: &Message) -> (Option<String>, Option<String>) {
    let attachments = &new_message.attachments;
    let attachments_fmt: Option<String> = if attachments.is_empty() {
        None
    } else {
        let attachment_names: Vec<String> = attachments
            .iter()
            .map(|attachment| attachment.filename.to_string())
            .collect();
        Some(format!(" <{}>", attachment_names.join(", ")))
    };

    let embeds = &new_message.embeds;
    let embeds_fmt: Option<String> = if embeds.is_empty() {
        None
    } else {
        let embed_types: Vec<String> = embeds
            .iter()
            .map(|embed| embed.kind.clone().unwrap_or_default().into_string())
            .collect();

        Some(format!(" {{{}}}", embed_types.join(", ")))
    };

    (attachments_fmt, embeds_fmt)
}

fn get_blacklisted_words(
    new_message: &Message,
    badlist: &HashSet<String>,
    fixlist: &HashSet<String>,
) -> Vec<String> {
    let messagewords: Vec<String> = new_message
        .content
        .to_lowercase()
        .split_whitespace()
        .map(String::from)
        .collect();

    let blacklisted_words: Vec<String> = messagewords
        .into_iter()
        .filter(|word| {
            // Check if the word is in the badlist and not in the fixlist
            let is_blacklisted = badlist.iter().any(|badword| {
                word.contains(badword) && !fixlist.iter().any(|fixword| word.contains(fixword))
            });

            is_blacklisted
        })
        .collect();

    let flagged_words: Vec<String> = blacklisted_words
        .iter()
        .map(|word| (*word).clone())
        .collect();

    flagged_words
}

fn author_string(ctx: &serenity::Context, msg: &Message) -> String {
    // No member meaning no roles.
    if msg.member.is_none() {
        return msg.author.tag();
    }
    let member = msg.member.as_ref().unwrap();
    let username = msg.author.tag();

    let guild = msg.guild(&ctx.cache).unwrap();

    let mut highest: Option<&serenity::Role> = None;

    // TODO: possibly try and optimise this.
    for role_id in &member.roles {
        if let Some(role) = guild.roles.get(role_id) {
            // Skip this role if this role in iteration has:
            // - a position less than the recorded highest
            // - a position equal to the recorded, but a higher ID
            if let Some(r) = highest {
                if role.position < r.position || (role.position == r.position && role.id > r.id) {
                    continue;
                }
            }

            highest = Some(role);
        }
    }

    let mut prefix = String::new();
    if let Some(hr) = highest {
        let c = hr.colour;
        if hr.colour.0 != 0 {
            write!(prefix, "\x1B[38;2;{};{};{}m", c.r(), c.g(), c.b()).unwrap();
        }
    }

    let reset = "\x1B[0m";
    format!("{prefix}{username}{reset}")
}
