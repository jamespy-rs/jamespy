use crate::{Context, Error};
use std::fmt;
mod utils;

use jamespy_events::handlers::messages::{EMOJI_REGEX, STANDARD_EMOJI_REGEX};
use sqlx::query_as;

use utils::{check_in_guild, display_expressions};

pub enum Expression<'a> {
    Emote((u64, String)),
    Standard(&'a str),
    Id(u64),
    Name(&'a str),
}

impl fmt::Display for Expression<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Id(id) => write!(f, "{id}"),
            Expression::Name(name) | Expression::Standard(name) => write!(f, "{name}"),
            Expression::Emote((_, name)) => write!(f, "{name}"),
        }
    }
}

#[derive(Debug)]
struct ExpressionCounts {
    user_id: i64,
    reaction_count: Option<i64>,
}

/// Display the usage of emoji's in reactions or messages.
#[poise::command(
    slash_command,
    prefix_command,
    rename = "emoji-usage",
    category = "Utility",
    guild_only,
    subcommands("reactions", "messages", "all"),
    subcommand_required
)]
pub async fn emoji_usage(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

pub fn string_to_expression(emoji: &str) -> Option<Expression<'_>> {
    let expression = if let Some(capture) = EMOJI_REGEX.captures(emoji) {
        let Ok(id) = capture[3].parse::<u64>() else {
            return None;
        };

        Expression::Emote((id, capture[2].to_string()))
    } else if let Ok(emoji_id) = emoji.parse::<u64>() {
        Expression::Id(emoji_id)
    } else if STANDARD_EMOJI_REGEX.captures(emoji).is_some() {
        Expression::Standard(emoji)
    } else {
        Expression::Name(emoji)
    };

    Some(expression)
}

/// Display usage of a reaction.
#[poise::command(slash_command, prefix_command, category = "Utility", guild_only)]
pub async fn reactions(ctx: Context<'_>, emoji: String) -> Result<(), Error> {
    let Some(expression) = string_to_expression(&emoji) else {
        ctx.say("I could not parse an expression from this string.")
            .await?;
        return Ok(());
    };

    let in_guild = check_in_guild(ctx, &expression).await?;
    if !in_guild {
        ctx.say("You require Manage Messages to be able to check expressions outside the guild.")
            .await?;
        return Ok(());
    };

    let guild_id = ctx.guild_id().unwrap().get() as i64;
    let results = match expression {
        Expression::Id(id) | Expression::Emote((id, _)) => {
            query_as!(
                ExpressionCounts,
                "SELECT eu.user_id, COUNT(eu.id) AS reaction_count FROM emote_usage eu JOIN \
                 emotes e ON eu.emote_id = e.id WHERE eu.usage_type = 'ReactionAdd' AND \
                 e.discord_id = $1 AND eu.guild_id = $2 GROUP BY eu.user_id ORDER BY \
                 reaction_count DESC",
                id as i64,
                guild_id
            )
            .fetch_all(&ctx.data().db)
            .await?
        }
        Expression::Name(string) => {
            query_as!(
                ExpressionCounts,
                "SELECT eu.user_id, COUNT(eu.id) AS reaction_count FROM emote_usage eu JOIN \
                 emotes e ON eu.emote_id = e.id WHERE eu.usage_type = 'ReactionAdd' AND \
                 e.emote_name = $1 AND eu.guild_id = $2 GROUP BY eu.user_id ORDER BY \
                 reaction_count DESC",
                string,
                guild_id
            )
            .fetch_all(&ctx.data().db)
            .await?
        }
        Expression::Standard(string) => {
            query_as!(
                ExpressionCounts,
                "SELECT eu.user_id, COUNT(eu.id) AS reaction_count FROM emote_usage eu JOIN \
                 emotes e ON eu.emote_id = e.id WHERE eu.usage_type = 'ReactionAdd' AND \
                 e.emote_name = $1 AND eu.guild_id = $2 AND e.discord_id IS NULL GROUP BY \
                 eu.user_id ORDER BY reaction_count DESC",
                string,
                guild_id
            )
            .fetch_all(&ctx.data().db)
            .await?
        }
    };

    display_expressions(ctx, &results, &expression, in_guild, Some(false)).await?;

    Ok(())
}

/// Display usage of emoji's through messages.
#[poise::command(slash_command, prefix_command, category = "Utility", guild_only)]
pub async fn messages(ctx: Context<'_>, emoji: String) -> Result<(), Error> {
    let Some(expression) = string_to_expression(&emoji) else {
        ctx.say("I could not parse an expression from this string.")
            .await?;
        return Ok(());
    };

    let in_guild = check_in_guild(ctx, &expression).await?;
    if !in_guild {
        ctx.say("You require Manage Messages to be able to check expressions outside the guild.")
            .await?;
        return Ok(());
    };

    let guild_id = ctx.guild_id().unwrap().get() as i64;
    let results = match expression {
        Expression::Id(id) | Expression::Emote((id, _)) => {
            query_as!(
                ExpressionCounts,
                "SELECT eu.user_id, COUNT(eu.id) AS reaction_count FROM emote_usage eu JOIN \
                 emotes e ON eu.emote_id = e.id WHERE eu.usage_type = 'Message' AND e.discord_id \
                 = $1 AND eu.guild_id = $2 GROUP BY eu.user_id ORDER BY reaction_count DESC",
                id as i64,
                guild_id,
            )
            .fetch_all(&ctx.data().db)
            .await?
        }
        Expression::Name(string) => {
            query_as!(
                ExpressionCounts,
                "SELECT eu.user_id, COUNT(eu.id) AS reaction_count FROM emote_usage eu JOIN \
                 emotes e ON eu.emote_id = e.id WHERE eu.usage_type = 'Message' AND e.emote_name \
                 = $1 AND eu.guild_id = $2 GROUP BY eu.user_id ORDER BY reaction_count DESC",
                string,
                guild_id
            )
            .fetch_all(&ctx.data().db)
            .await?
        }
        Expression::Standard(string) => {
            query_as!(
                ExpressionCounts,
                "SELECT eu.user_id, COUNT(eu.id) AS reaction_count FROM emote_usage eu JOIN \
                 emotes e ON eu.emote_id = e.id WHERE eu.usage_type = 'Message' AND e.emote_name \
                 = $1 AND eu.guild_id = $2 AND e.discord_id IS NULL GROUP BY eu.user_id ORDER BY \
                 reaction_count DESC",
                string,
                guild_id,
            )
            .fetch_all(&ctx.data().db)
            .await?
        }
    };

    display_expressions(ctx, &results, &expression, in_guild, Some(true)).await?;

    Ok(())
}

/// Display usage of emojis everywhere.
#[poise::command(slash_command, prefix_command, category = "Utility", guild_only)]
pub async fn all(ctx: Context<'_>, emoji: String) -> Result<(), Error> {
    let Some(expression) = string_to_expression(&emoji) else {
        ctx.say("I could not parse an expression from this string.")
            .await?;
        return Ok(());
    };

    let in_guild = check_in_guild(ctx, &expression).await?;
    if !in_guild {
        ctx.say("You require Manage Messages to be able to check expressions outside the guild.")
            .await?;
        return Ok(());
    };

    let guild_id = ctx.guild_id().unwrap().get() as i64;
    let results = match expression {
        Expression::Id(id) | Expression::Emote((id, _)) => {
            query_as!(
                ExpressionCounts,
                "SELECT eu.user_id, COUNT(eu.id) AS reaction_count FROM emote_usage eu JOIN \
                 emotes e ON eu.emote_id = e.id WHERE (eu.usage_type = 'Message' OR eu.usage_type \
                 = 'ReactionAdd') AND e.discord_id = $1 AND eu.guild_id = $2 GROUP BY  eu.user_id \
                 ORDER BY  reaction_count DESC",
                id as i64,
                guild_id
            )
            .fetch_all(&ctx.data().db)
            .await?
        }
        Expression::Name(string) => {
            query_as!(
                ExpressionCounts,
                "SELECT eu.user_id, COUNT(eu.id) AS reaction_count FROM emote_usage eu JOIN \
                 emotes e ON eu.emote_id = e.id WHERE (eu.usage_type = 'Message' OR eu.usage_type \
                 = 'ReactionAdd') AND e.emote_name = $1 AND eu.guild_id = $2 GROUP BY eu.user_id \
                 ORDER BY reaction_count DESC",
                string,
                guild_id
            )
            .fetch_all(&ctx.data().db)
            .await?
        }
        Expression::Standard(string) => {
            query_as!(
                ExpressionCounts,
                "SELECT eu.user_id, COUNT(eu.id) AS reaction_count FROM emote_usage eu JOIN \
                 emotes e ON eu.emote_id = e.id WHERE (eu.usage_type = 'Message' OR eu.usage_type \
                 = 'ReactionAdd') AND e.emote_name = $1 AND eu.guild_id = $2 AND e.discord_id IS \
                 NULL GROUP BY eu.user_id ORDER BY reaction_count DESC",
                string,
                guild_id
            )
            .fetch_all(&ctx.data().db)
            .await?
        }
    };

    display_expressions(ctx, &results, &expression, in_guild, None).await?;

    Ok(())
}

// /emote-leaderboard reactions [duration]
// /emote-leaderboard messages [duration]
// /emote-leaderboard all [duration]

// /sticker-usage [sticker]
// /sticker-leaderboard [duration]

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [emoji_usage()]
}
