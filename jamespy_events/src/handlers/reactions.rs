use std::sync::Arc;

use crate::helper::{get_channel_name, get_guild_name_override, get_user};
use crate::{Data, Error};

use bb8_redis::redis::AsyncCommands;
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

    let redis_pool = &data.redis;
    let mut redis_conn = redis_pool.get().await?;

    println!(
        "\x1B[95m[{}] [#{}] {} added a reaction: {}\x1B[0m",
        guild_name, channel_name, user_name, add_reaction.emoji
    );

    let reaction_key = format!("reactions:{}", guild_id.unwrap_or_default());
    let reaction_info = (
        add_reaction.emoji.to_string(),
        user_id,
        add_reaction.message_id.get(),
        1,
    );

    let reaction_info_json = serde_json::to_string(&reaction_info)?;

    redis_conn.lpush(&reaction_key, reaction_info_json).await?;
    redis_conn.ltrim(&reaction_key, 0, 249).await?;
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

    let redis_pool = &data.redis;
    let mut redis_conn = redis_pool.get().await?;

    println!(
        "\x1B[95m[{}] [#{}] {} removed a reaction: {}\x1B[0m",
        guild_name, channel_name, user_name, removed_reaction.emoji
    );

    let reaction_key = format!("reactions:{}", guild_id.unwrap_or_default());
    let reaction_info = (
        removed_reaction.emoji.to_string(),
        user_id,
        removed_reaction.message_id.get(),
        0,
    );

    let reaction_info_json = serde_json::to_string(&reaction_info)?;

    redis_conn.lpush(&reaction_key, reaction_info_json).await?;
    redis_conn.ltrim(&reaction_key, 0, 249).await?;
    Ok(())
}
