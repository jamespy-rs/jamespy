use crate::{Context, Error};
use jamespy_data::database::EmoteUsageType;
use poise::serenity_prelude::MessageId;
use sqlx::query_as;

use super::Expression;

enum SearchContext {
    Guild,
    Message(MessageId),
}

/// Get a log of reactions for the guild or a message.
#[poise::command(
    slash_command,
    prefix_command,
    rename = "last-reactions",
    category = "Utility",
    guild_only,
    install_context = "Guild",
    interaction_context = "Guild",
    required_permissions = "MANAGE_MESSAGES",
    subcommands("guild", "message")
)]
pub async fn last_reactions(ctx: Context<'_>, emoji: Option<String>) -> Result<(), Error> {
    shared(ctx, SearchContext::Guild, emoji).await
}

/// Displays last reactions in the Guild.
#[poise::command(slash_command, prefix_command, category = "Utility", guild_only)]
pub async fn guild(ctx: Context<'_>, emoji: Option<String>) -> Result<(), Error> {
    shared(ctx, SearchContext::Guild, emoji).await
}

/// Displays last reactions on a Message.
#[poise::command(slash_command, prefix_command, category = "Utility", guild_only)]
pub async fn message(
    ctx: Context<'_>,
    message: MessageId,
    emoji: Option<String>,
) -> Result<(), Error> {
    shared(ctx, SearchContext::Message(message), emoji).await
}

async fn shared(
    ctx: Context<'_>,
    search_context: SearchContext,
    emoji: Option<String>,
) -> Result<(), Error> {
    let expression = if let Some(ref emoji) = emoji {
        if let Some(expression) = super::string_to_expression(emoji) {
            Some(expression)
        } else {
            ctx.say("I could not parse an expression from this string.")
                .await?;
            return Ok(());
        }
    } else {
        None
    };

    let results =
        query_last_reactions(ctx, &ctx.data().database, search_context, expression).await?;

    super::last_reactions_paginator::display_expressions(ctx, &results).await
}

pub struct LastReactionEntry {
    pub user_id: i64,
    pub emote_name: String,
    pub discord_id: Option<i64>,
    pub is_added: Option<bool>,
}

// TODO: dedupe this shit lmao
async fn query_last_reactions(
    ctx: Context<'_>,
    database: &jamespy_data::database::Database,
    search_context: SearchContext,
    emoji: Option<Expression<'_>>,
) -> Result<Vec<LastReactionEntry>, Error> {
    match (search_context, emoji) {
        (SearchContext::Guild, None) => {
            return Ok(query_as!(
                LastReactionEntry,
                r#"
                SELECT eu.user_id, e.emote_name, e.discord_id,
                    CASE
                        WHEN eu.usage_type = 'ReactionAdd' THEN true
                        WHEN eu.usage_type = 'ReactionRemove' THEN false
                        ELSE false
                    END as is_added
                FROM emote_usage eu
                JOIN emotes e ON eu.emote_id = e.id
                WHERE eu.usage_type = ANY($2)
                AND eu.guild_id = $1
                ORDER BY eu.used_at DESC
                LIMIT 250
                "#,
                ctx.guild_id().unwrap().get() as i64,
                &[EmoteUsageType::ReactionAdd, EmoteUsageType::ReactionRemove]
                    as &[EmoteUsageType],
            )
            .fetch_all(&database.db)
            .await?);
        }
        (SearchContext::Guild, Some(expression)) => match expression {
            Expression::Id(id) | Expression::Emote((id, _)) => {
                return Ok(query_as!(
                    LastReactionEntry,
                    r#"
                        SELECT eu.user_id, e.emote_name, e.discord_id,
                            CASE
                                WHEN eu.usage_type = 'ReactionAdd' THEN true
                                WHEN eu.usage_type = 'ReactionRemove' THEN false
                                ELSE false
                            END as is_added
                        FROM emote_usage eu
                        JOIN emotes e ON eu.emote_id = e.id
                        WHERE eu.usage_type = ANY($2)
                        AND eu.guild_id = $1
                        AND eu.emote_id = $3
                        ORDER BY eu.used_at DESC
                        LIMIT 250
                        "#,
                    ctx.guild_id().unwrap().get() as i64,
                    &[EmoteUsageType::ReactionAdd, EmoteUsageType::ReactionRemove]
                        as &[EmoteUsageType],
                    id as i64
                )
                .fetch_all(&database.db)
                .await?);
            }
            Expression::Standard(name) => {
                return Ok(query_as!(
                    LastReactionEntry,
                    r#"
                        SELECT eu.user_id, e.emote_name, e.discord_id,
                            CASE
                                WHEN eu.usage_type = 'ReactionAdd' THEN true
                                WHEN eu.usage_type = 'ReactionRemove' THEN false
                                ELSE false
                            END as is_added
                        FROM emote_usage eu
                        JOIN emotes e ON eu.emote_id = e.id
                        WHERE eu.usage_type = ANY($2)
                        AND eu.guild_id = $1
                        AND e.emote_name = $3
                        AND e.discord_id IS NOT NULL
                        ORDER BY eu.used_at DESC
                        LIMIT 250
                        "#,
                    ctx.guild_id().unwrap().get() as i64,
                    &[EmoteUsageType::ReactionAdd, EmoteUsageType::ReactionRemove]
                        as &[EmoteUsageType],
                    name
                )
                .fetch_all(&database.db)
                .await?);
            }
            Expression::Name(name) => {
                return Ok(query_as!(
                    LastReactionEntry,
                    r#"
                        SELECT eu.user_id, e.emote_name, e.discord_id,
                            CASE
                                WHEN eu.usage_type = 'ReactionAdd' THEN true
                                WHEN eu.usage_type = 'ReactionRemove' THEN false
                                ELSE false
                            END as is_added
                        FROM emote_usage eu
                        JOIN emotes e ON eu.emote_id = e.id
                        WHERE eu.usage_type = ANY($2)
                        AND eu.guild_id = $1
                        AND e.emote_name = $3
                        AND e.discord_id IS NULL
                        ORDER BY eu.used_at DESC
                        LIMIT 250
                        "#,
                    ctx.guild_id().unwrap().get() as i64,
                    &[EmoteUsageType::ReactionAdd, EmoteUsageType::ReactionRemove]
                        as &[EmoteUsageType],
                    name
                )
                .fetch_all(&database.db)
                .await?);
            }
        },
        (SearchContext::Message(message_id), None) => {
            return Ok(query_as!(
                LastReactionEntry,
                r#"
                SELECT eu.user_id, e.emote_name, e.discord_id,
                    CASE
                        WHEN eu.usage_type = 'ReactionAdd' THEN true
                        WHEN eu.usage_type = 'ReactionRemove' THEN false
                        ELSE false
                    END as is_added
                FROM emote_usage eu
                JOIN emotes e ON eu.emote_id = e.id
                WHERE eu.usage_type = ANY($2)
                AND eu.message_id = $1
                ORDER BY eu.used_at DESC
                LIMIT 250
                "#,
                message_id.get() as i64,
                &[EmoteUsageType::ReactionAdd, EmoteUsageType::ReactionRemove]
                    as &[EmoteUsageType],
            )
            .fetch_all(&database.db)
            .await?);
        }
        (SearchContext::Message(message_id), Some(expression)) => match expression {
            Expression::Id(id) | Expression::Emote((id, _)) => {
                return Ok(query_as!(
                    LastReactionEntry,
                    r#"
                        SELECT eu.user_id, e.emote_name, e.discord_id,
                            CASE
                                WHEN eu.usage_type = 'ReactionAdd' THEN true
                                WHEN eu.usage_type = 'ReactionRemove' THEN false
                                ELSE false
                            END as is_added
                        FROM emote_usage eu
                        JOIN emotes e ON eu.emote_id = e.id
                        WHERE eu.usage_type = ANY($2)
                        AND eu.message_id = $1
                        AND eu.emote_id = $3
                        ORDER BY eu.used_at DESC
                        LIMIT 250
                        "#,
                    message_id.get() as i64,
                    &[EmoteUsageType::ReactionAdd, EmoteUsageType::ReactionRemove]
                        as &[EmoteUsageType],
                    id as i64
                )
                .fetch_all(&database.db)
                .await?);
            }
            Expression::Standard(name) => {
                return Ok(query_as!(
                    LastReactionEntry,
                    r#"
                        SELECT eu.user_id, e.emote_name, e.discord_id,
                            CASE
                                WHEN eu.usage_type = 'ReactionAdd' THEN true
                                WHEN eu.usage_type = 'ReactionRemove' THEN false
                                ELSE false
                            END as is_added
                        FROM emote_usage eu
                        JOIN emotes e ON eu.emote_id = e.id
                        WHERE eu.usage_type = ANY($2)
                        AND eu.message_id = $1
                        AND e.emote_name = $3
                        AND e.discord_id IS NOT NULL
                        ORDER BY eu.used_at DESC
                        LIMIT 250
                        "#,
                    message_id.get() as i64,
                    &[EmoteUsageType::ReactionAdd, EmoteUsageType::ReactionRemove]
                        as &[EmoteUsageType],
                    name
                )
                .fetch_all(&database.db)
                .await?);
            }
            Expression::Name(name) => {
                return Ok(query_as!(
                    LastReactionEntry,
                    r#"
                        SELECT eu.user_id, e.emote_name, e.discord_id,
                            CASE
                                WHEN eu.usage_type = 'ReactionAdd' THEN true
                                WHEN eu.usage_type = 'ReactionRemove' THEN false
                                ELSE false
                            END as is_added
                        FROM emote_usage eu
                        JOIN emotes e ON eu.emote_id = e.id
                        WHERE eu.usage_type = ANY($2)
                        AND eu.message_id = $1
                        AND e.emote_name = $3
                        AND e.discord_id IS NULL
                        ORDER BY eu.used_at DESC
                        LIMIT 250
                        "#,
                    message_id.get() as i64,
                    &[EmoteUsageType::ReactionAdd, EmoteUsageType::ReactionRemove]
                        as &[EmoteUsageType],
                    name
                )
                .fetch_all(&database.db)
                .await?);
            }
        },
    }
}
