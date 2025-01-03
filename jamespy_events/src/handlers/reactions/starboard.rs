use crate::{Data, Error};
use jamespy_data::database::{
    ChannelIdWrapper, MessageIdWrapper, StarboardMessage, StarboardStatus, UserIdWrapper,
};
use poise::serenity_prelude as serenity;
use std::sync::Arc;

use super::components::STARBOARD_CHANNEL;

const STARBOARD_QUEUE: serenity::ChannelId = serenity::ChannelId::new(1324437869808582656);

pub(super) async fn starboard_add_handler(
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
    } else if !data.database.handle_starboard(reaction.message_id)
        && new(ctx, data, reaction).await.is_err()
    {
        data.database.stop_handle_starboard(&reaction.message_id);
    }

    Ok(())
}

pub(super) async fn starboard_remove_handler(
    ctx: &serenity::Context,
    reaction: &serenity::Reaction,
    data: &Arc<Data>,
) -> Result<(), Error> {
    if !std::env::var("STARBOARD_ACTIVE").map(|e| e.parse::<bool>())?? {
        return Ok(());
    };

    if let Ok(mut starboard) = data.database.get_starboard_msg(reaction.message_id).await {
        let msg = reaction.message(ctx).await?;

        starboard.star_count = count(&msg);

        let message = starboard_edit_message(&starboard);

        starboard
            .starboard_message_channel
            .edit_message(&ctx.http, *starboard.starboard_message_id, message)
            .await?;

        data.database
            .update_star_count(starboard.id, starboard.star_count)
            .await?;
    }

    Ok(())
}

// maybe i should cache this better later.
fn count(msg: &serenity::Message) -> i16 {
    let star_count: u64 = msg
        .reactions
        .iter()
        .filter(|r| {
            if let serenity::ReactionType::Unicode(ref unicode) = r.reaction_type {
                return unicode == "⭐";
            }
            false
        })
        .map(|r| r.count)
        .sum();

    star_count as i16
}

async fn existing(
    ctx: &serenity::Context,
    data: &Arc<Data>,
    reaction: &serenity::Reaction,
    mut starboard_msg: StarboardMessage,
) -> Result<(), Error> {
    let msg = ctx
        .http
        .get_message(reaction.channel_id, reaction.message_id)
        .await?;

    starboard_msg.star_count = count(&msg);
    println!("{}", starboard_msg.star_count);

    let message = starboard_edit_message(&starboard_msg);

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

    let star_count = count(&msg);

    if star_count <= 5 {
        return Ok(());
    }

    let mut starboard_msg = StarboardMessage {
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
        star_count: star_count as i16,
        starboard_status: StarboardStatus::InReview,
        starboard_message_id: MessageIdWrapper(0.into()),
        starboard_message_channel: ChannelIdWrapper(STARBOARD_QUEUE),
    };

    let message = starboard_message(&starboard_msg);

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
    ($msg_type:ty, $new_fn:expr, $starboard_msg:expr) => {{
        let mut message = $new_fn()
            .content(format!(
                ":star: **{} |** <#{}>",
                $starboard_msg.star_count, *$starboard_msg.channel_id
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
        }

        message
    }};
}

pub(super) fn starboard_message(starboard_msg: &StarboardMessage) -> serenity::CreateMessage<'_> {
    starboard_message_macro!(
        serenity::CreateMessage<'_>,
        serenity::CreateMessage::new,
        starboard_msg
    )
}

fn starboard_edit_message(starboard_msg: &StarboardMessage) -> serenity::EditMessage<'_> {
    starboard_message_macro!(
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
        println!("yes the doing");
        author = author.icon_url(url);
    }

    let mut embed = serenity::CreateEmbed::new()
        .author(author.clone())
        .description(&starboard_msg.content)
        .color(serenity::Colour::BLUE)
        // deduplication of embeds.
        .url("https://osucord.moe");

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
                if matches!(extension.as_str(), "jpeg" | "jpg" | "png" | "webp") {
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
