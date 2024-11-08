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
    } else if STANDARD_EMOJI_REGEX.captures(&emoji).is_some() {
        Expression::Standard(&emoji)
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
            let results = query_as!(
                ExpressionCounts,
                "SELECT eu.user_id, COUNT(eu.id) AS reaction_count FROM emote_usage eu JOIN \
                 emotes e ON eu.emote_id = e.id WHERE eu.usage_type = 'ReactionAdd' AND \
                 e.discord_id = $1 GROUP BY eu.user_id ORDER BY reaction_count DESC",
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
                 emotes e ON eu.emote_id = e.id WHERE eu.usage_type = 'ReactionAdd' AND \
                 e.emote_name = $1 GROUP BY eu.user_id ORDER BY reaction_count DESC",
                string
            )
            .fetch_all(&ctx.data().db)
            .await?;

            display_expressions(ctx, &results, &expression, in_guild).await?;
        }
        Expression::Standard(string) => {
            let results = query_as!(
                ExpressionCounts,
                "SELECT eu.user_id, COUNT(eu.id) AS reaction_count FROM emote_usage eu JOIN \
                 emotes e ON eu.emote_id = e.id WHERE eu.usage_type = 'ReactionAdd' AND \
                 e.emote_name = $1 AND e.discord_id IS NULL GROUP BY eu.user_id ORDER BY \
                 reaction_count DESC",
                string
            )
            .fetch_all(&ctx.data().db)
            .await?;

            display_expressions(ctx, &results, &expression, true).await?;
        }
    }

    Ok(())
}

// /emote-usage reactions [emote]
// /emote-usage messages [emote]
// /emote-usage all [emote]

// /emote-leaderboard reactions [duration]
// /emote-leaderboard messages [duration]
// /emote-leaderboard all [duration]

// /sticker-usage [sticker]
// /sticker-leaderboard [duration]

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [who_reacted()]
}
