use crate::{Data, Error};
use jamespy_data::database::{
    ChannelIdWrapper, MessageIdWrapper, StarboardMessage, StarboardStatus, UserIdWrapper,
};
use poise::serenity_prelude as serenity;
use small_fixed_array::FixedString;
use std::{collections::hash_map::Entry, str::FromStr, sync::Arc};

use super::components::STARBOARD_CHANNEL;

const STARBOARD_QUEUE: serenity::ChannelId = serenity::ChannelId::new(1324543000600383549);

pub async fn starboard_add_handler(
    ctx: &serenity::Context,
    reaction: &serenity::Reaction,
    data: &Arc<Data>,
) -> Result<(), Error> {
    if !std::env::var("STARBOARD_ACTIVE").map(|e| e.parse::<bool>())?? {
        return Ok(());
    };

    if reaction.channel_id == STARBOARD_CHANNEL {
        return Ok(());
    }

    if let Ok(starboard) = data.database.get_starboard_msg(reaction.message_id).await {
        existing(ctx, data, reaction, starboard).await?;
    } else if !data.database.handle_starboard(reaction.message_id) {
        let _ = new(ctx, data, reaction).await;
        data.database.stop_handle_starboard(&reaction.message_id);
    }

    Ok(())
}

pub async fn starboard_remove_handler(
    ctx: &serenity::Context,
    reaction: &serenity::Reaction,
    data: &Arc<Data>,
) -> Result<(), Error> {
    if !std::env::var("STARBOARD_ACTIVE").map(|e| e.parse::<bool>())?? {
        return Ok(());
    };

    if let Ok(mut starboard) = data.database.get_starboard_msg(reaction.message_id).await {
        if *starboard.user_id == reaction.user_id.unwrap() {
            return Ok(());
        }

        starboard.star_count =
            get_reaction_count(ctx, data, reaction, *starboard.user_id, Some(false)).await?;

        let message = starboard_edit_message(ctx, &starboard);

        starboard
            .starboard_message_channel
            .edit_message(&ctx.http, *starboard.starboard_message_id, message)
            .await?;

        data.database
            .update_star_count(starboard.id, starboard.star_count)
            .await?;
    } else {
        let msg = reaction.message(ctx).await?;
        let _ = get_reaction_count(ctx, data, reaction, msg.author.id, Some(false)).await?;
    }

    Ok(())
}

/// Get the reaction count from the cache or fetch it from http if its not available.
///
/// Returns the count, optionally incrementing or decreasing reaction value internally if cached.
/// Panics if `Reaction` is not from the gateway.
async fn get_reaction_count(
    ctx: &serenity::Context,
    data: &Arc<Data>,
    reaction: &serenity::Reaction,
    author_id: serenity::UserId,
    state: Option<bool>,
) -> Result<i16, Error> {
    let reaction_user = reaction.user_id.unwrap();

    // If Some(true), add reaction_user, if Some(false), remove.
    let reactions = {
        let mut guard = data.database.starboard.lock();
        guard
            .reactions_cache
            .entry(reaction.message_id)
            .and_modify(|(_, vec)| {
                if let Some(true) = state {
                    if !vec.contains(&reaction_user) {
                        vec.push(reaction_user);
                    }
                } else if let Some(false) = state {
                    vec.retain(|&user_id| user_id != reaction_user);
                }
            });
        guard.reactions_cache.get(&reaction.message_id).cloned()
    };

    if let Some((_, reactors)) = reactions {
        return Ok(reactors.len() as i16);
    }

    // TODO: paginate this.
    let users = ctx
        .http
        .get_reaction_users(
            reaction.channel_id,
            reaction.message_id,
            &serenity::ReactionType::Unicode(FixedString::from_str("⭐").unwrap()),
            100,
            None,
        )
        .await?;

    let filtered = users
        .into_iter()
        .filter(|user| user.id != author_id)
        .map(|u| u.id)
        .collect::<Vec<_>>();

    let count = filtered.len();

    let mut guard = data.database.starboard.lock();
    match guard.reactions_cache.entry(reaction.message_id) {
        Entry::Occupied(mut entry) => {
            *entry.get_mut() = (author_id, filtered);
        }
        Entry::Vacant(entry) => {
            entry.insert((author_id, filtered));
        }
    }

    Ok(count as i16)
}

async fn existing(
    ctx: &serenity::Context,
    data: &Arc<Data>,
    reaction: &serenity::Reaction,
    mut starboard_msg: StarboardMessage,
) -> Result<(), Error> {
    if *starboard_msg.user_id == reaction.user_id.unwrap() {
        return Ok(());
    }

    starboard_msg.star_count =
        get_reaction_count(ctx, data, reaction, *starboard_msg.user_id, Some(true)).await?;

    let message = starboard_edit_message(ctx, &starboard_msg);

    starboard_msg
        .starboard_message_channel
        .edit_message(&ctx.http, *starboard_msg.starboard_message_id, message)
        .await?;

    data.database
        .update_star_count(starboard_msg.id, starboard_msg.star_count)
        .await?;

    Ok(())
}

async fn new(
    ctx: &serenity::Context,
    data: &Arc<Data>,
    reaction: &serenity::Reaction,
) -> Result<(), Error> {
    let msg = reaction.message(ctx).await?;

    if msg.author.id == reaction.user_id.unwrap() {
        return Ok(());
    }

    let star_count = get_reaction_count(ctx, data, reaction, msg.author.id, Some(true)).await?;

    if star_count < 3 {
        return Ok(());
    }

    let mut starboard_msg = StarboardMessage {
        // gets corrected on insert.
        id: 0,
        user_id: UserIdWrapper(msg.author.id),
        username: msg.author.name.to_string(),
        avatar_url: msg.author.avatar_url(),
        content: msg.content.to_string(),
        channel_id: ChannelIdWrapper(msg.channel_id),
        message_id: MessageIdWrapper(msg.id),
        attachment_urls: msg
            .attachments
            .iter()
            .map(|a| {
                a.url
                    .split_once('?')
                    .map_or_else(|| a.url.to_string(), |a| a.0.to_string())
            })
            .collect(),
        star_count,
        starboard_status: StarboardStatus::InReview,
        // gets corrected on insert.
        starboard_message_id: MessageIdWrapper(0.into()),
        starboard_message_channel: ChannelIdWrapper(STARBOARD_QUEUE),
    };

    let message = starboard_message(ctx, &starboard_msg);

    let msg = STARBOARD_QUEUE.send_message(&ctx.http, message).await?;

    starboard_msg.starboard_message_id = MessageIdWrapper(msg.id);

    // woo hardcoding
    data.database
        .insert_starboard_msg(
            starboard_msg,
            Some(serenity::GuildId::new(98226572468690944)),
        )
        .await?;

    Ok(())
}

macro_rules! starboard_message_macro {
    ($ctx:expr, $msg_type:ty, $new_fn:expr, $starboard_msg:expr) => {{
        let guild = $ctx.cache.guild(98226572468690944.into());

        let name = if let Some(guild) = guild {
            guild
                .channels
                .iter()
                .find(|c| c.id == *$starboard_msg.channel_id)
                .map(|c| c.name.to_string())
                .unwrap_or_else(|| {
                    guild
                        .threads
                        .iter()
                        .find(|t| t.id == *$starboard_msg.channel_id)
                        .map(|t| t.name.to_string())
                        .unwrap_or_else(|| format!("<#{}>", *$starboard_msg.channel_id))
                })
        } else {
            format!("<#{}>", *$starboard_msg.channel_id)
        };

        let mut message = $new_fn()
            .content(format!(
                ":star: **{} | #{name}**",
                $starboard_msg.star_count
            ))
            .embeds(starboard_embeds($starboard_msg));

        if $starboard_msg.starboard_status == StarboardStatus::InReview {
            let components = serenity::CreateActionRow::Buttons(std::borrow::Cow::Owned(vec![
                serenity::CreateButton::new("starboard_accept")
                    .label("Accept")
                    .style(serenity::ButtonStyle::Primary),
                serenity::CreateButton::new("starboard_deny")
                    .label("Deny")
                    .style(serenity::ButtonStyle::Danger),
            ]));
            message = message.components(vec![components]);

            message = message.content(format!(
                ":star: **{} |** <#{}> <@101090238067113984> <@291089948709486593> \
                 <@158567567487795200>",
                $starboard_msg.star_count, *$starboard_msg.channel_id
            ));
        }

        message
    }};
}

pub(super) fn starboard_message<'a>(
    ctx: &'a serenity::Context,
    starboard_msg: &'a StarboardMessage,
) -> serenity::CreateMessage<'a> {
    starboard_message_macro!(
        ctx,
        serenity::CreateMessage<'_>,
        serenity::CreateMessage::new,
        starboard_msg
    )
}

fn starboard_edit_message<'a>(
    ctx: &'a serenity::Context,
    starboard_msg: &'a StarboardMessage,
) -> serenity::EditMessage<'a> {
    starboard_message_macro!(
        ctx,
        serenity::EditMessage<'_>,
        serenity::EditMessage::new,
        starboard_msg
    )
}

/// This is a regex that will extract the file extension, requires query params to be removed.
pub static LINK_REGEX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"\.([a-zA-Z0-9]+)$").unwrap());

fn starboard_embeds(starboard_msg: &StarboardMessage) -> Vec<serenity::CreateEmbed<'_>> {
    let mut author = serenity::CreateEmbedAuthor::new(&starboard_msg.username);
    if let Some(url) = &starboard_msg.avatar_url {
        author = author.icon_url(url);
    }

    let mut embed = serenity::CreateEmbed::new()
        .author(author.clone())
        .description(&starboard_msg.content)
        .color(serenity::Colour::BLUE)
        // deduplication of embeds.
        .url("https://osucord.moe")
        .timestamp(starboard_msg.message_id.created_at());

    if !starboard_msg.attachment_urls.is_empty() {
        embed = embed.field(
            "Attachments",
            starboard_msg.attachment_urls.join("\n"),
            false,
        );
    }

    // hardcoding wooooooo
    embed = embed.field(
        "Original",
        starboard_msg.message_id.link(
            *starboard_msg.channel_id,
            Some(serenity::GuildId::new(98226572468690944)),
        ),
        false,
    );

    let mut embeds = Vec::new();
    for attachment_url in &starboard_msg.attachment_urls {
        if let Some(captures) = LINK_REGEX.captures(attachment_url) {
            if let Some(extension) = captures.get(1) {
                if matches!(extension.as_str(), "jpeg" | "jpg" | "png" | "webp" | "gif") {
                    if embeds.len() == 4 {
                        break;
                    }

                    if embeds.is_empty() {
                        embeds.push(embed.clone().image(attachment_url));
                        continue;
                    }

                    let embed = serenity::CreateEmbed::new()
                        .url("https://osucord.moe")
                        .image(attachment_url);

                    embeds.push(embed);
                }
            }
        }
    }

    if embeds.is_empty() {
        embeds.push(embed);
    }

    embeds
}
