use crate::{Context, Error};
use std::fmt;
use std::fmt::Write;

use aformat::{aformat, ToArrayString};

use jamespy_events::handlers::messages::EMOJI_REGEX;
use poise::{CreateReply, PrefixContext};
use serenity::all::{
    ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage, EmojiId, Permissions,
};
use sqlx::query_as;

enum Expression<'a> {
    Emote((u64, String)),
    Id(u64),
    Name(&'a str),
}

impl fmt::Display for Expression<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Id(id) => write!(f, "{id}"),
            Expression::Name(name) => write!(f, "{name}"),
            Expression::Emote((_, name)) => write!(f, "{name}"),
        }
    }
}

struct ExpressionCounts {
    user_id: i64,
    reaction_count: Option<i64>,
}

#[poise::command(
    slash_command,
    prefix_command,
    rename = "who-reacted",
    category = "Utility",
    guild_only
)]
pub async fn who_reacted(ctx: Context<'_>, emoji: String) -> Result<(), Error> {
    let expression = if let Some(capture) = EMOJI_REGEX.captures(&emoji) {
        let Ok(id) = capture[3].parse::<u64>() else {
            ctx.say("Failed to handle emoji.").await?;
            return Ok(());
        };

        Expression::Emote((id, capture[2].to_string()))
    } else if let Ok(emoji_id) = emoji.parse::<u64>() {
        Expression::Id(emoji_id)
    } else {
        Expression::Name(&emoji)
    };

    let in_guild = check_in_guild(ctx, &expression).await?;
    if !in_guild {
        ctx.say("You are not authorised to check for expressions outside of the guild.")
            .await?;
        return Ok(());
    };

    match expression {
        Expression::Id(id) | Expression::Emote((id, _)) => {
            // TODO: stop duplicating the below path.
            let results = query_as!(
                ExpressionCounts,
                "SELECT eu.user_id, COUNT(eu.id) AS reaction_count FROM emote_usage eu JOIN \
                 emotes e ON eu.emote_id = e.discord_id WHERE eu.usage_type = 'ReactionAdd' AND \
                 eu.emote_id = $1 GROUP BY eu.user_id ORDER BY reaction_count DESC",
                id as i64
            )
            .fetch_all(&ctx.data().db)
            .await?;

            display_expressions(ctx, &results, &expression, in_guild).await?;
        }
        Expression::Name(string) => {
            let results = query_as!(
                ExpressionCounts,
                "SELECT eu.user_id, COUNT(eu.id) AS reaction_count FROM emote_usage eu JOIN \
                 emotes e ON eu.emote_id = e.discord_id WHERE eu.usage_type = 'ReactionAdd' AND \
                 e.emote_name = $1 GROUP BY eu.user_id ORDER BY reaction_count DESC",
                string
            )
            .fetch_all(&ctx.data().db)
            .await?;

            display_expressions(ctx, &results, &expression, in_guild).await?;
        }
    }

    Ok(())
}

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

async fn display_expressions(
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

async fn check_in_guild(ctx: Context<'_>, expression: &Expression<'_>) -> Result<bool, Error> {
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

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [who_reacted()]
}
