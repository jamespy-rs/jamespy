use crate::{
    Data, Error,
    reactions::{get_reaction_count, get_unique_reaction_count},
};
use moth_data::database::{
    ChannelIdWrapper, MessageIdWrapper, StarboardMessage, StarboardStatus, UserIdWrapper,
};
use poise::serenity_prelude as serenity;
use std::sync::Arc;

pub async fn starboard_add_handler(
    ctx: &serenity::Context,
    reaction: &serenity::Reaction,
    data: &Arc<Data>,
) -> Result<(), Error> {
    if !data.starboard_config.active {
        return Ok(());
    }

    if reaction.user_id.unwrap() == ctx.cache.current_user().id {
        return Ok(());
    }

    if let Ok(starboard_msg) = data.database.get_starboard_msg(reaction.message_id).await {
        if starboard_msg.starboard_status == StarboardStatus::Denied {
            return Ok(());
        }

        existing(ctx, data, reaction, starboard_msg).await?;
    } else if let Ok(starboard_msg_by_id) = data
        .database
        .get_starboard_msg_by_starboard_id(reaction.message_id)
        .await
    {
        if starboard_msg_by_id.starboard_status != StarboardStatus::Denied {
            existing(ctx, data, reaction, starboard_msg_by_id).await?;
        }
    } else if !data.database.handle_starboard(reaction.message_id) {
        // If no existing starboard message is found, handle the new starboard message
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
    if !data.starboard_config.active {
        return Ok(());
    }

    if reaction.user_id.unwrap() == ctx.cache.current_user().id {
        return Ok(());
    }

    let mut starboard =
        if let Ok(starboard) = data.database.get_starboard_msg(reaction.message_id).await {
            starboard
        } else if let Ok(starboard) = data
            .database
            .get_starboard_msg_by_starboard_id(reaction.message_id)
            .await
        {
            starboard
        } else {
            return Ok(());
        };

    if *starboard.user_id == reaction.user_id.unwrap() {
        return Ok(());
    }

    starboard.star_count =
        get_unique_reaction_count(ctx, data, &starboard, reaction, Some(false)).await?;

    let message = starboard_edit_message(ctx, data, &starboard);

    starboard
        .starboard_message_channel
        .edit_message(&ctx.http, *starboard.starboard_message_id, message)
        .await?;

    data.database
        .update_star_count(starboard.id, starboard.star_count)
        .await?;

    Ok(())
}

async fn remove_reaction(ctx: &serenity::Context, reaction: &serenity::Reaction) {
    let has_permissions = has_permissions(ctx, reaction);

    if has_permissions {
        let _ = ctx
            .http
            .delete_reaction(
                reaction.channel_id,
                reaction.message_id,
                reaction
                    .user_id
                    .expect("This will only be called with messages from the gateway."),
                &reaction.emoji,
            )
            .await;
    }
}

/// Checks if the bot has manage messages in the channel that the reaction was in.
fn has_permissions(ctx: &serenity::Context, reaction: &serenity::Reaction) -> bool {
    if let Some(guild) = ctx.cache.guild(
        reaction
            .guild_id
            .expect("This will only be called from a guild."),
    ) {
        let channel = guild
            .channels
            .get(&reaction.channel_id)
            .or(guild.threads.iter().find(|t| t.id == reaction.channel_id));

        if let Some(channel) = channel {
            let permissions = guild.user_permissions_in(
                channel,
                guild.members.get(&ctx.cache.current_user().id).unwrap(),
            );

            return permissions.manage_messages();
        }
    }

    false
}

async fn existing(
    ctx: &serenity::Context,
    data: &Arc<Data>,
    reaction: &serenity::Reaction,
    mut starboard_msg: StarboardMessage,
) -> Result<(), Error> {
    if *starboard_msg.user_id == reaction.user_id.unwrap() {
        remove_reaction(ctx, reaction).await;
        return Ok(());
    }

    starboard_msg.star_count =
        get_unique_reaction_count(ctx, data, &starboard_msg, reaction, Some(true)).await?;

    let message = starboard_edit_message(ctx, data, &starboard_msg);

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
        remove_reaction(ctx, reaction).await;
        return Ok(());
    }

    let star_count = get_reaction_count(ctx, data, reaction, msg.author.id, Some(true)).await?;

    if star_count < data.starboard_config.threshold as i16 {
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
        starboard_message_channel: ChannelIdWrapper(data.starboard_config.queue_channel),
    };

    let message = starboard_message(ctx, data, &starboard_msg);

    let msg = data
        .starboard_config
        .queue_channel
        .send_message(&ctx.http, message)
        .await?;

    starboard_msg.starboard_message_id = MessageIdWrapper(msg.id);

    // woo hardcoding
    data.database
        .insert_starboard_msg(starboard_msg, Some(data.starboard_config.guild_id))
        .await?;

    Ok(())
}

macro_rules! starboard_message_macro {
    ($ctx:expr, $data:expr, $msg_type:ty, $new_fn:expr, $starboard_msg:expr) => {{
        let guild = $ctx.cache.guild($data.starboard_config.guild_id);

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
                "{} **{} | #{name}**",
                $data.starboard_config.star_emoji, $starboard_msg.star_count
            ))
            .embeds(starboard_embeds($data, $starboard_msg));

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
                "{} **{} |** <#{}> <@101090238067113984> <@291089948709486593> \
                 <@158567567487795200>",
                $data.starboard_config.star_emoji,
                $starboard_msg.star_count,
                *$starboard_msg.channel_id
            ));
        }

        message
    }};
}

pub(super) fn starboard_message<'a>(
    ctx: &'a serenity::Context,
    data: &Arc<Data>,
    starboard_msg: &'a StarboardMessage,
) -> serenity::CreateMessage<'a> {
    starboard_message_macro!(
        ctx,
        data,
        serenity::CreateMessage<'_>,
        serenity::CreateMessage::new,
        starboard_msg
    )
}

fn starboard_edit_message<'a>(
    ctx: &'a serenity::Context,
    data: &Arc<Data>,
    starboard_msg: &'a StarboardMessage,
) -> serenity::EditMessage<'a> {
    starboard_message_macro!(
        ctx,
        data,
        serenity::EditMessage<'_>,
        serenity::EditMessage::new,
        starboard_msg
    )
}

/// This is a regex that will extract the file extension, requires query params to be removed.
pub static LINK_REGEX: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"\.([a-zA-Z0-9]+)$").unwrap());

fn starboard_embeds<'a>(
    data: &Arc<Data>,
    starboard_msg: &'a StarboardMessage,
) -> Vec<serenity::CreateEmbed<'a>> {
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
            Some(data.starboard_config.guild_id),
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
