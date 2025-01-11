use aformat::{aformat, aformat_into, ArrayString, CapStr, ToArrayString};
use std::{borrow::Cow, fmt::Write};

use crate::{Context, Error};
use poise::CreateReply;
use serenity::all::{
    ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage, EmojiId,
};

use super::{last_reactions::LastReactionEntry, utils::get_paginated_records};

const RECORDS_PER_PAGE: usize = 20;

fn generate_embed<'a>(
    ctx: &Context<'_>,
    expressions: &'a [LastReactionEntry],
    page_info: Option<(usize, usize)>,
) -> CreateEmbed<'a> {
    let mut string = String::new();

    let guild = ctx.guild();

    for expression in expressions {
        if let Some(discord_id) = expression.discord_id {
            if let Some(guild) = &guild {
                if let Some(emoji) = guild.emojis.get(&EmojiId::new(discord_id as u64)) {
                    writeln!(
                        string,
                        "<@{}>: {} ({})",
                        expression.user_id as u64,
                        get_emoji_markdown(emoji),
                        if expression.is_added.unwrap() {
                            "Added"
                        } else {
                            "Removed"
                        }
                    )
                    .unwrap();

                    continue;
                }
            }
        }

        writeln!(
            string,
            "<@{}>: {} ({})",
            expression.user_id as u64,
            expression.emote_name,
            if expression.is_added.unwrap() {
                "Added"
            } else {
                "Removed"
            }
        )
        .unwrap();
    }

    let mut embed = CreateEmbed::new()
        .title("Last reactions")
        .description(string);

    if let Some((current_page, max_pages)) = page_info {
        let footer = CreateEmbedFooter::new(format!("Page {}/{}", current_page + 1, max_pages));
        embed = embed.footer(footer);
    };

    embed
}

fn get_emoji_markdown(emoji: &serenity::all::Emoji) -> ArrayString<57> {
    let mut buf = ArrayString::<57>::new();

    if emoji.animated() {
        aformat_into!(buf, "<a:{}:{}>", CapStr::<32>(&emoji.name), emoji.id);
    } else {
        aformat_into!(buf, "<:{}:{}>", CapStr::<32>(&emoji.name), emoji.id);
    }

    buf
}

pub(super) async fn display_expressions(
    ctx: Context<'_>,
    all_records: &[LastReactionEntry],
) -> Result<(), Error> {
    if all_records.is_empty() {
        ctx.say("No expressions").await?;
        return Ok(());
    };

    let paginate = all_records.len() > RECORDS_PER_PAGE;
    let total_pages = all_records.len().div_ceil(RECORDS_PER_PAGE);
    let mut page = 0_usize;
    let records = get_paginated_records(all_records, page);

    let page_info = if paginate {
        Some((page, total_pages))
    } else {
        None
    };

    let embed = generate_embed(&ctx, records, page_info);
    let builder = CreateReply::new().embed(embed);

    if !paginate {
        ctx.send(builder).await?;
        return Ok(());
    };

    let ctx_id = ctx.id();
    let previous_id = aformat!("{ctx_id}previous");
    let next_id = aformat!("{ctx_id}next");

    let components = [CreateActionRow::Buttons(Cow::Owned(vec![
        CreateButton::new(previous_id.as_str()).emoji('◀'),
        CreateButton::new(next_id.as_str()).emoji('▶'),
    ]))];

    let builder = builder.components(&components);

    let msg = ctx.send(builder).await?;

    while let Some(press) = ComponentInteractionCollector::new(ctx.serenity_context().shard.clone())
        .filter(move |press| {
            press
                .data
                .custom_id
                .starts_with(ctx_id.to_arraystring().as_str())
        })
        .timeout(std::time::Duration::from_secs(180))
        .await
    {
        if *press.data.custom_id == *next_id {
            page += 1;
            if page >= total_pages {
                page = 0;
            }
        } else if *press.data.custom_id == *previous_id {
            page = page.checked_sub(1).unwrap_or(total_pages - 1);
        } else {
            continue;
        }

        let records = get_paginated_records(all_records, page);
        let embed = generate_embed(&ctx, records, Some((page, total_pages)));

        let _ = press
            .create_response(
                ctx.http(),
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::default().embed(embed),
                ),
            )
            .await;
    }

    let records = get_paginated_records(all_records, page);
    let embed = generate_embed(&ctx, records, Some((page, total_pages)));

    msg.edit(ctx, CreateReply::new().embed(embed).components(vec![]))
        .await?;

    Ok(())
}
