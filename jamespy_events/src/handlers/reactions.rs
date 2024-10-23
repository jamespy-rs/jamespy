use std::sync::Arc;

use crate::helper::{get_channel_name, get_guild_name_override, get_user};
use crate::messages::FuckRustRules;
use crate::{Data, Error};

use ::serenity::all::{GuildId, UserId};
use chrono::Utc;
use sqlx::{query, Postgres, Transaction};

use jamespy_data::database::EmoteUsageType;

use poise::serenity_prelude::{self as serenity, Reaction};

pub async fn reaction_add(
    ctx: &serenity::Context,
    add_reaction: &Reaction,
    data: Arc<Data>,
) -> Result<(), Error> {
    // I'm not bothered here anymore.
    // will need to to_user when guild_id is none and i'm not adding complexity
    // for reactions that don't matter.
    if add_reaction.guild_id.is_none() {
        return Ok(());
    };

    // recieved over gateway, so a user is present.
    let user_id = add_reaction.user_id.unwrap();
    let user_name = match get_user(ctx, add_reaction.guild_id.unwrap(), user_id).await {
        Some(user) => {
            if user.bot() {
                return Ok(());
            }
            user.tag()
        }
        None => String::from("Unknown User"),
    };

    let guild_id = add_reaction.guild_id;
    let guild_name = get_guild_name_override(ctx, &data, guild_id);

    let channel_name = get_channel_name(ctx, guild_id, add_reaction.channel_id).await;

    println!(
        "\x1B[95m[{}] [#{}] {} added a reaction: {}\x1B[0m",
        guild_name, channel_name, user_name, add_reaction.emoji
    );

    insert_addition(
        data.db.begin().await?,
        guild_id.unwrap(),
        user_id,
        add_reaction,
    )
    .await?;

    Ok(())
}

pub async fn reaction_remove(
    ctx: &serenity::Context,
    removed_reaction: &Reaction,
    data: Arc<Data>,
) -> Result<(), Error> {
    // ditto.
    if removed_reaction.guild_id.is_none() {
        return Ok(());
    };

    // ditto.
    let user_id = removed_reaction.user_id.unwrap();
    let user_name = match get_user(ctx, removed_reaction.guild_id.unwrap(), user_id).await {
        Some(user) => {
            if user.bot() {
                return Ok(());
            }
            user.tag()
        }
        None => String::from("Unknown User"),
    };
    let guild_id = removed_reaction.guild_id;
    let guild_name = get_guild_name_override(ctx, &data, guild_id);
    let channel_name = get_channel_name(ctx, guild_id, removed_reaction.channel_id).await;

    println!(
        "\x1B[95m[{}] [#{}] {} removed a reaction: {}\x1B[0m",
        guild_name, channel_name, user_name, removed_reaction.emoji
    );

    insert_removal(
        data.db.begin().await?,
        guild_id.unwrap(),
        user_id,
        removed_reaction,
    )
    .await?;

    Ok(())
}

async fn insert_emote_usage(
    mut transaction: Transaction<'_, Postgres>,
    guild_id: GuildId,
    user_id: UserId,
    reaction: &Reaction,
    usage_type: EmoteUsageType,
) -> Result<(), Error> {
    let serenity::ReactionType::Custom {
        animated: _,
        id,
        name,
    } = &reaction.emoji
    else {
        return Ok(());
    };

    let Some(name) = name else { return Ok(()) };

    // Insert or ignore the emote
    query!(
        "INSERT INTO emotes (emote_name, discord_id) VALUES ($1, $2) ON CONFLICT (discord_id) DO \
         NOTHING",
        &FuckRustRules(name),
        id.get() as i64
    )
    .execute(&mut *transaction)
    .await?;

    // Insert the emote usage
    query!(
        "INSERT INTO emote_usage (message_id, emote_id, user_id, channel_id, guild_id,
         used_at, usage_type) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        reaction.message_id.get() as i64,
        id.get() as i64,
        user_id.get() as i64,
        reaction.channel_id.get() as i64,
        guild_id.get() as i64,
        Utc::now().timestamp(),
        usage_type as _,
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
}

async fn insert_addition(
    transaction: Transaction<'_, Postgres>,
    guild_id: GuildId,
    user_id: UserId,
    reaction: &Reaction,
) -> Result<(), Error> {
    insert_emote_usage(
        transaction,
        guild_id,
        user_id,
        reaction,
        EmoteUsageType::ReactionAdd,
    )
    .await?;
    Ok(())
}

async fn insert_removal(
    transaction: Transaction<'_, Postgres>,
    guild_id: GuildId,
    user_id: UserId,
    reaction: &Reaction,
) -> Result<(), Error> {
    insert_emote_usage(
        transaction,
        guild_id,
        user_id,
        reaction,
        EmoteUsageType::ReactionRemove,
    )
    .await?;

    Ok(())
}
