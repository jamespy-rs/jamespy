use crate::{Context, Error};
use std::fmt;
mod query;
mod utils;

use jamespy_data::database::EmoteUsageType;
use jamespy_events::handlers::messages::{EMOJI_REGEX, STANDARD_EMOJI_REGEX};
use query::handle_expression_query;

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
    let types = [EmoteUsageType::ReactionAdd];
    shared(ctx, emoji, &types, Some(false)).await
}

/// Display usage of emoji's through messages.
#[poise::command(slash_command, prefix_command, category = "Utility", guild_only)]
pub async fn messages(ctx: Context<'_>, emoji: String) -> Result<(), Error> {
    let types = [EmoteUsageType::Message];
    shared(ctx, emoji, &types, Some(true)).await
}

/// Display usage of emojis everywhere.
#[poise::command(slash_command, prefix_command, category = "Utility", guild_only)]
pub async fn all(ctx: Context<'_>, emoji: String) -> Result<(), Error> {
    let types = [EmoteUsageType::ReactionAdd, EmoteUsageType::Message];
    shared(ctx, emoji, &types, None).await
}

async fn shared(
    ctx: Context<'_>,
    emoji: String,
    types: &[EmoteUsageType],
    msg_type: Option<bool>,
) -> Result<(), Error> {
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

    let results = handle_expression_query(
        &ctx.data().database,
        &expression,
        ctx.guild_id().unwrap(),
        types,
    )
    .await?;

    display_expressions(ctx, &results, &expression, in_guild, msg_type).await
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
