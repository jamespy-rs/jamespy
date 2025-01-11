use ::serenity::all::{GuildId, Reaction, ReactionType, UserId};
use chrono::Utc;
use sqlx::query;

use crate::Error;

use moth_data::database::{Database, EmoteUsageType};

async fn insert_emote_usage(
    database: &Database,
    guild_id: GuildId,
    user_id: UserId,
    reaction: &Reaction,
    usage_type: EmoteUsageType,
) -> Result<(), Error> {
    let (name, id) = match &reaction.emoji {
        ReactionType::Custom {
            animated: _,
            id,
            name,
        } => {
            let Some(name) = name else { return Ok(()) };

            (name, Some(id.get() as i64))
        }
        ReactionType::Unicode(string) => (string, None),
        _ => return Ok(()),
    };

    database
        .insert_channel(reaction.channel_id, Some(guild_id))
        .await?;
    database.insert_user(user_id).await?;

    // This is so fucking dumb.

    let id = if let Some(id) = id {
        let id = query!(
            "INSERT INTO emotes (emote_name, discord_id) VALUES ($1, $2) ON CONFLICT (discord_id) \
             DO UPDATE SET emote_name = EXCLUDED.emote_name RETURNING id",
            &name.as_str(),
            id
        )
        .fetch_one(&database.db)
        .await?;
        id.id
    } else {
        let id = query!(
            "INSERT INTO emotes (emote_name)
                     VALUES ($1)
                     ON CONFLICT (emote_name) WHERE discord_id IS NULL
                     DO UPDATE SET discord_id = emotes.discord_id
                     RETURNING id",
            &name.as_str(),
        )
        .fetch_one(&database.db)
        .await?;
        id.id
    };

    query!(
        "INSERT INTO emote_usage (emote_id, message_id, user_id, channel_id, guild_id,
    used_at, usage_type) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        i64::from(id),
        reaction.message_id.get() as i64,
        user_id.get() as i64,
        reaction.channel_id.get() as i64,
        guild_id.get() as i64,
        Utc::now().timestamp(),
        usage_type as _,
    )
    .execute(&database.db)
    .await?;

    Ok(())
}

pub(super) async fn insert_addition(
    database: &Database,
    guild_id: GuildId,
    user_id: UserId,
    reaction: &Reaction,
) -> Result<(), Error> {
    insert_emote_usage(
        database,
        guild_id,
        user_id,
        reaction,
        EmoteUsageType::ReactionAdd,
    )
    .await?;
    Ok(())
}

pub(super) async fn insert_removal(
    database: &Database,
    guild_id: GuildId,
    user_id: UserId,
    reaction: &Reaction,
) -> Result<(), Error> {
    insert_emote_usage(
        database,
        guild_id,
        user_id,
        reaction,
        EmoteUsageType::ReactionRemove,
    )
    .await?;

    Ok(())
}
