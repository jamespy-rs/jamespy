use std::collections::HashSet;
use std::fmt::Write;
use std::sync::Arc;

use rustrict::{Censor, Type};

mod database;
pub use database::EMOJI_REGEX;

pub static WHITESPACE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"(\s*)(\S+)").unwrap());

use crate::helper::{get_channel_name, get_guild_name, get_guild_name_override};
use crate::{Data, Error};

use database::{insert_deletion, insert_edit, insert_message};
use poise::serenity_prelude::{
    self as serenity, ChannelId, Colour, CreateEmbedFooter, GuildId, Message, MessageId,
    MessageUpdateEvent, UserId,
};
use small_fixed_array::FixedString;

pub async fn message(ctx: &serenity::Context, msg: &Message, data: Arc<Data>) -> Result<(), Error> {
    let (flagged_words, patterns) = {
        let config = &data.config.read().events;

        if should_skip_msg(
            config.no_log_users.as_ref(),
            config.no_log_channels.as_ref(),
            msg,
        ) {
            return Ok(());
        }

        let flagged_words =
            get_blacklisted_words(msg, config.badlist.as_ref(), config.fixlist.as_ref());

        (flagged_words, config.regex.clone())
    };

    let guild_id = msg.guild_id;
    let guild_name = get_guild_name_override(ctx, &data, guild_id);
    let channel_name = get_channel_name(ctx, guild_id, msg.channel_id).await;

    // check names.
    data.check_or_insert_user(&msg.author).await;

    if let (Some(id), Some(member)) = (guild_id, msg.member.as_ref()) {
        if let Some(nick) = member.nick.as_ref().map(std::string::ToString::to_string) {
            data.check_or_insert_nick(id, msg.author.id, Some(nick))
                .await;
        }
    }

    if let Some(patterns) = patterns {
        check_event_dm_regex(ctx, msg, &get_guild_name(ctx, guild_id), &patterns).await?;
    };

    handle_dm(ctx, msg).await?;

    let (attachments, embeds) = attachments_embed_fmt(msg);

    let author_string = author_string(ctx, msg);

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
         {flagged_str}{}\x1B[0m\x1B[36m{}{}\x1B[0m",
        msg.content,
        attachments.as_deref().unwrap_or(""),
        embeds.as_deref().unwrap_or("")
    );

    insert_message(&data.database, msg).await?;

    Ok(())
}

pub async fn message_edit(
    ctx: &serenity::Context,
    old_if_available: &Option<Message>,
    new: &Option<Message>,
    event: &MessageUpdateEvent,
    data: Arc<Data>,
) -> Result<(), Error> {
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

                let _ = insert_edit(&data.database, new_message).await;
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

        let _ = insert_deletion(&data.database, &message).await;

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
    no_log_users: Option<&Vec<u64>>,
    no_log_channels: Option<&Vec<u64>>,
    message: &Message,
) -> bool {
    let user_condition = no_log_users
        .as_ref()
        .is_some_and(|vec| vec.contains(&message.author.id.get()));

    let channel_condition = no_log_channels
        .as_ref()
        .is_some_and(|vec| vec.contains(&message.channel_id.get()));

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

    // TODO: use fw owner's or make configurable.
    UserId::from(158567567487795200).dm(&ctx.http, msg).await?;

    Ok(())
}

async fn handle_dm(ctx: &serenity::Context, msg: &Message) -> Result<(), Error> {
    // TODO: use fw owner's or make configurable.
    if msg.guild_id.is_some()
        || [158567567487795200, ctx.cache.current_user().id.get()].contains(&msg.author.id.get())
    {
        return Ok(());
    }

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

    // dm me about the mention of me.
    UserId::from(158567567487795200).dm(&ctx.http, msg).await?;
    Ok(())
}

#[must_use]
pub fn attachments_embed_fmt(new_message: &Message) -> (Option<String>, Option<String>) {
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
    badlist: Option<&HashSet<String>>,
    fixlist: Option<&HashSet<String>>,
) -> Vec<FixedString<u16>> {
    let message_lowercase = new_message.content.to_lowercase();

    let content = &new_message.content;
    let mut censor = Censor::from_str(content);
    if censor.analyze() != Type::NONE {
        // scuffed stuff.
        censor.reset(content.chars());
        // it doesn't return the difference, there's no other way than to do some weird comparison.
        let censored = censor.censor();

        let mut orig = content.split_whitespace();
        let mut censored = censored.split_whitespace();

        let mut changed_words = Vec::new();

        loop {
            match (orig.next(), censored.next()) {
                (Some(w1), Some(w2)) if w1 != w2 => {
                    changed_words.push(w1);
                }
                (Some(_) | None, Some(_)) => continue,
                (Some(w1), None) => changed_words.push(w1),
                (None, None) => break,
            }
        }

        println!("{changed_words:?}");

        let mut result = String::new();

        for cap in WHITESPACE.captures_iter(content) {
            // leading whitespace
            let leading_whitespace = &cap[1];
            // The word
            let word = &cap[2];

            result.push_str(leading_whitespace);

            if changed_words.contains(&word) {
                write!(result, "\x1B[1m\x1B[31m{word}\x1B[0m").unwrap();
            } else {
                result.push_str(word);
            }
        }

        println!("{result}");
    }

    message_lowercase
        .split_whitespace()
        .filter_map(|word| {
            let is_blacklisted = match (badlist, fixlist) {
                (Some(bad_set), Some(fix_set)) => {
                    bad_set.iter().any(|badword| word.contains(badword))
                        && !fix_set.iter().any(|fixword| word.contains(fixword))
                }
                (Some(bad_set), None) => bad_set.iter().any(|badword| word.contains(badword)),
                _ => false,
            };

            if is_blacklisted {
                Some(FixedString::<u16>::from_str_trunc(word))
            } else {
                None
            }
        })
        .collect()
}

#[must_use]
pub fn author_string(ctx: &serenity::Context, msg: &Message) -> String {
    // No member meaning no roles.
    let Some(member) = &msg.member else {
        return msg.author.tag();
    };

    let username = msg.author.tag();

    let guild = msg.guild(&ctx.cache).unwrap();

    let mut highest: Option<&serenity::Role> = None;

    // TODO: possibly try and optimise this.
    for role_id in &member.roles {
        if let Some(role) = guild.roles.get(role_id) {
            if role.colour.0 == 000000 {
                continue;
            }

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
