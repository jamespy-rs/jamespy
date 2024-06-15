use crate::{Context, Error};
use ::serenity::all::{ChannelId, Colour, CreateEmbed, CreateEmbedFooter, UserId};
use humantime::parse_duration;
use poise::{
    serenity_prelude::{
        self as serenity, ComponentInteractionCollector, CreateActionRow,
        CreateInteractionResponse, EmojiId, StickerFormatType,
    },
    CreateReply,
};
use sqlx::{query, types::chrono::Utc};
use std::collections::HashMap;
use std::fmt::Write;

#[poise::command(
    slash_command,
    prefix_command,
    install_context = "Guild|User",
    interaction_context = "Guild",
    category = "Utility",
    guild_only
)]
pub async fn stickers(ctx: Context<'_>) -> Result<(), Error> {
    let stickers = {
        let guild = ctx.guild().unwrap();
        guild.stickers.clone()
    };

    let mut pages = vec![];
    for sticker in stickers {
        let mut embed =
            serenity::CreateEmbed::new().title(format!("{} (ID:{})", sticker.name, sticker.id));

        let mut description = String::new();
        if let Some(desc) = sticker.description.clone() {
            println!("{}: {}", sticker.name, desc.len());
            writeln!(description, "**Description:** {desc}").unwrap();
        };

        // if it can be parsed its just numbers and therefore a guild emote.
        // or it was custom set outside the discord client and is just random numbers.
        let related_emoji = if let Ok(id) = sticker.tags[0].parse::<u64>() {
            if let Some(emoji) = ctx.guild().unwrap().emojis.get(&EmojiId::from(id)) {
                format!("{emoji}")
            } else {
                id.to_string()
            }
        } else {
            let emoji_regex = regex::Regex::new(r"[\p{Emoji}]+").unwrap();

            // technically this isn't flawless given discord lets you put random text
            // if you just use the api directly.
            // at least that is what i think.
            if emoji_regex.is_match(&sticker.tags[0]) {
                sticker.tags[0].to_string()
            } else {
                format!(":{}:", sticker.tags[0])
            }
        };

        writeln!(description, "**Related Emoji:** {related_emoji}").unwrap();

        writeln!(
            description,
            "**Format Type:** {}",
            sticker_format_type_str(&sticker.format_type)
        )
        .unwrap();
        writeln!(description, "**Available:** {}", sticker.available).unwrap();
        embed = embed.description(description);

        if let Some(url) = sticker.image_url() {
            embed = embed.thumbnail(url);
        }

        pages.push(embed);
    }

    let ctx_id = ctx.id();
    let prev_button_id = format!("{ctx_id}prev");
    let next_button_id = format!("{ctx_id}next");

    let mut current_page = 0;

    let msg = ctx
        .send(
            poise::CreateReply::default()
                .embed(pages[0].clone())
                .components(vec![CreateActionRow::Buttons(vec![
                    serenity::CreateButton::new(&prev_button_id).emoji('◀'),
                    serenity::CreateButton::new(&next_button_id).emoji('▶'),
                ])]),
        )
        .await?;

    while let Some(press) = ComponentInteractionCollector::new(ctx.serenity_context().shard.clone())
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(180))
        .await
    {
        if press.data.custom_id == next_button_id {
            current_page += 1;
            if current_page >= pages.len() {
                current_page = 0;
            }
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(pages.len() - 1);
        } else {
            continue;
        }

        press
            .create_response(
                ctx.http(),
                CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::default()
                        .embed(pages[current_page].clone()),
                ),
            )
            .await?;
    }
    // clear components.
    msg.edit(
        ctx,
        poise::CreateReply::new()
            .embed(pages[current_page].clone())
            .components(vec![]),
    )
    .await?;

    Ok(())
}

fn sticker_format_type_str(sticker_fmt: &StickerFormatType) -> &str {
    match *sticker_fmt {
        StickerFormatType::Png => "PNG",
        StickerFormatType::Lottie => "LOTTIE",
        StickerFormatType::Apng => "APNG",
        StickerFormatType::Gif => "GIF",
        _ => "Unknown",
    }
}

/// msglb, improved
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    user_cooldown = "5",
    required_permissions = "MANAGE_MESSAGES"
)]
pub async fn msglb(ctx: Context<'_>, #[rest] timestamp: Option<String>) -> Result<(), Error> {
    // This entire command is a try not to allocate challenge where I have completely given up
    // before it even started lol, will revisit when not HUNGRY.

    ctx.defer().await?;

    let time_filter = if let Some(timestamp) = timestamp {
        let Ok(duration) = parse_duration(&timestamp) else {
            ctx.say("Could not parse duration!").await?;
            return Ok(());
        };

        let now = Utc::now().naive_utc();
        Some(now - duration)
    } else {
        None
    };

    let rows = query!(
        "SELECT user_id, channel_id, timestamp FROM msgs WHERE guild_id = $1",
        ctx.guild_id().unwrap().get() as i64
    )
    .fetch_all(&ctx.data().db)
    .await?;

    // I have to filter the timestamp after because the type is "different" between
    // a query that filters the timestamp and not.

    let mut users: HashMap<UserId, HashMap<ChannelId, u32>> = HashMap::new();

    for row in rows {
        // checking this x amount of times is 100% slower but eh, i don't wanna fight the type system.
        if let Some(time_filter) = time_filter {
            if row.timestamp.unwrap() < time_filter {
                continue;
            }
        }

        let Some(user_id) = row.user_id.map(|v| UserId::new(v as u64)) else {
            continue;
        };

        let Some(channel_id) = row.channel_id.map(|v| ChannelId::new(v as u64)) else {
            continue;
        };

        let user_entry = users.entry(user_id).or_default();

        let channel_entry = user_entry.entry(channel_id).or_insert(0);

        *channel_entry += 1;
    }

    let mut user_results = Vec::new();
    for (user_id, channels) in &users {
        let total_messages: u32 = channels.values().sum();
        let mut top_3_keys: [Option<(ChannelId, u32)>; 3] = [None, None, None];

        for (key, &value) in channels {
            for i in 0..top_3_keys.len() {
                if top_3_keys[i].is_none() || top_3_keys[i].unwrap().1 < value {
                    for j in (i..top_3_keys.len() - 1).rev() {
                        top_3_keys[j + 1] = top_3_keys[j];
                    }
                    top_3_keys[i] = Some((*key, value));
                    break;
                }
            }
        }

        let mut first = true;
        let mut result = String::new();

        for &entry in &top_3_keys {
            if let Some((channel_id, value)) = entry {
                if !first {
                    result.push_str(", ");
                }

                // Format and append "<#channel_id> (value)"
                write!(result, "<#{channel_id}> (**{value}**)").unwrap();
                first = false;
            }
        }

        user_results.push((user_id, total_messages, result));
    }

    // sort users by total messages.
    user_results.sort_by(|a, b| b.1.cmp(&a.1));

    let mut paginated_responses: Vec<String> = Vec::new();
    let mut current_response = String::new();

    for (index, (user_id, total_messages, top_channels)) in user_results.iter().enumerate() {
        if index > 0 && index % 15 == 0 {
            paginated_responses.push(current_response.clone());
            current_response.clear();
        }

        if index % 15 != 0 {
            current_response.push('\n');
        }

        write!(
            &mut current_response,
            "<@{user_id}>: **{total_messages}**: {top_channels}"
        )
        .unwrap();
    }

    if !current_response.is_empty() {
        paginated_responses.push(current_response);
    }

    msglb_paginator(ctx, &paginated_responses).await?;
    Ok(())
}

pub async fn msglb_paginator(ctx: Context<'_>, pages: &[String]) -> Result<(), serenity::Error> {
    let ctx_id = ctx.id();
    let prev_button_id = format!("{ctx_id}prev");
    let next_button_id = format!("{ctx_id}next");

    let mut current_page = 0;
    let max_pages = pages.len();

    let initial_embed = build_msglb_embed(&pages[current_page], current_page, max_pages);
    let msg =
        ctx.send(CreateReply::new().embed(initial_embed).components(vec![
            CreateActionRow::Buttons(vec![
                serenity::CreateButton::new(&prev_button_id).emoji('◀'),
                serenity::CreateButton::new(&next_button_id).emoji('▶'),
            ]),
        ]))
        .await?;

    while let Some(press) = ComponentInteractionCollector::new(ctx.serenity_context().shard.clone())
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(300))
        .await
    {
        if press.data.custom_id == next_button_id {
            current_page += 1;
            if current_page >= max_pages {
                current_page = 0;
            }
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(max_pages - 1);
        } else {
            continue;
        }

        let embed = build_msglb_embed(&pages[current_page], current_page, max_pages);
        press
            .create_response(
                ctx.http(),
                CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::default().embed(embed),
                ),
            )
            .await?;
    }
    // post timeout.
    let embed = build_msglb_embed(&pages[current_page], current_page, max_pages);
    msg.edit(
        ctx,
        poise::CreateReply::default()
            .embed(embed)
            .components(vec![]),
    )
    .await?;

    Ok(())
}

fn build_msglb_embed(page: &str, page_num: usize, max_pages: usize) -> CreateEmbed<'_> {
    let footer = CreateEmbedFooter::new(format!("Page {}/{max_pages}", page_num + 1));
    serenity::CreateEmbed::new()
        .description(page)
        .title("Top 15 users sorted by messages")
        .footer(footer)
        .colour(Colour::BLUE)
}

#[must_use]
pub fn commands() -> [crate::Command; 2] {
    [stickers(), msglb()]
}
