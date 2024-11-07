use aformat::{aformat, ToArrayString};
use std::fmt::Write;

use crate::{Context, Error};
use poise::{CreateReply, PrefixContext};
use serenity::all::{
    ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage, EmojiId, Permissions,
};

use super::{Expression, ExpressionCounts};

const RECORDS_PER_PAGE: usize = 20;

// does this panic lol?
fn get_paginated_records(records: &[ExpressionCounts], current_page: usize) -> &[ExpressionCounts] {
    let start_index = current_page * RECORDS_PER_PAGE;
    let end_index = start_index + RECORDS_PER_PAGE;

    &records[start_index..end_index.min(records.len())]
}

fn generate_embed<'a>(
    title: &'a str,
    expressions: &'a [ExpressionCounts],
    page_info: Option<(usize, usize)>,
) -> CreateEmbed<'a> {
    let mut string = String::new();

    for expression in expressions {
        let Some(count) = expression.reaction_count else {
            continue;
        };

        writeln!(string, "<@{}>: {count}", expression.user_id as u64).unwrap();
    }

    let mut embed = CreateEmbed::new().title(title).description(string);

    if let Some((current_page, max_pages)) = page_info {
        let footer = CreateEmbedFooter::new(format!("Page {}/{}", current_page + 1, max_pages + 1));
        embed = embed.footer(footer);
    };

    embed
}

pub(super) async fn display_expressions(
    ctx: Context<'_>,
    records: &[ExpressionCounts],
    expression: &Expression<'_>,
    in_guild: bool,
) -> Result<(), Error> {
    if records.is_empty() {
        ctx.say("No expressions").await?;
        return Ok(());
    };

    let paginate = records.len() > 20;
    let total_pages = records.len() / RECORDS_PER_PAGE;
    let mut page = 0_usize;
    let records = get_paginated_records(records, page);

    // I will go back on this at a later date.
    let name = if in_guild {
        if let Some(guild) = ctx.guild() {
            let emote = match expression {
                Expression::Id(id) | Expression::Emote((id, _)) => {
                    guild.emojis.get(&EmojiId::new(*id))
                }
                Expression::Name(string) => {
                    guild.emojis.iter().find(|e| e.name.as_str() == *string)
                }
                Expression::Standard(_) => None,
            };

            emote.map_or_else(|| expression.to_string(), ToString::to_string)
        } else {
            expression.to_string()
        }
    } else {
        expression.to_string()
    };

    let title = format!("Top {name} Reactors");

    let page_info = if paginate {
        Some((page, total_pages))
    } else {
        None
    };

    let embed = generate_embed(&title, records, page_info);
    let builder = CreateReply::new().embed(embed);

    if !paginate {
        ctx.send(builder).await?;
        return Ok(());
    };

    let ctx_id = ctx.id();
    let previous_id = aformat!("{ctx_id}previous");
    let next_id = aformat!("{ctx_id}next");

    let components = [CreateActionRow::Buttons(vec![
        CreateButton::new(previous_id.as_str()).emoji('◀'),
        CreateButton::new(next_id.as_str()).emoji('▶'),
    ])];

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

        let records = get_paginated_records(records, page);
        let embed = generate_embed(&title, records, Some((page, total_pages)));

        let _ = press
            .create_response(
                ctx.http(),
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::default().embed(embed),
                ),
            )
            .await;
    }

    let records = get_paginated_records(records, page);
    let embed = generate_embed(&title, records, Some((page, total_pages)));

    msg.edit(ctx, CreateReply::new().embed(embed)).await?;

    Ok(())
}

pub(super) async fn check_in_guild(
    ctx: Context<'_>,
    expression: &Expression<'_>,
) -> Result<bool, Error> {
    if let Expression::Standard(_) = expression {
        return Ok(true);
    }

    let permissions = match ctx {
        poise::Context::Application(ctx) => ctx
            .interaction
            .member
            .as_ref()
            .unwrap()
            .permissions
            .unwrap(),
        poise::Context::Prefix(ctx) => prefix_member_perms(ctx).await?,
    };

    if permissions.manage_messages() {
        return Ok(true);
    };

    let Some(guild) = ctx.guild() else {
        return Err("Could not retrieve guild from cache.".into());
    };

    let present = match expression {
        Expression::Id(id) | Expression::Emote((id, _)) => {
            guild.emojis.contains_key(&EmojiId::new(*id))
        }
        Expression::Name(string) => guild.emojis.iter().any(|e| e.name.as_str() == *string),
        // This is handled at the start of this check.
        Expression::Standard(_) => unreachable!(),
    };

    Ok(present)
}

// Messy, I know but I also don't care. It also checks guild perms instead of channel perms here but I also don't care.
// I'll clean this up at a later date.
async fn prefix_member_perms(
    ctx: PrefixContext<'_, crate::Data, Error>,
) -> Result<Permissions, Error> {
    let Some(_) = ctx.msg.member.as_ref() else {
        let member = ctx.author_member().await.ok_or("Failed to fetch member.")?;
        let Some(guild) = ctx.guild() else {
            return Err("Could not retrieve guild from cache.".into());
        };

        return Ok(guild.member_permissions(&member));
    };

    let member = ctx.author_member().await.ok_or("Failed to fetch member.")?;
    let Some(guild) = ctx.guild() else {
        return Err("Could not retrieve guild from cache.".into());
    };

    // https://github.com/serenity-rs/serenity/pull/3001
    Ok(guild.member_permissions(&member))
}
