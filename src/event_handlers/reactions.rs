use crate::{
    utils::misc::{get_channel_name, get_guild_name},
    Data, Error,
};
use bb8_redis::redis::AsyncCommands;
use poise::serenity_prelude::{self as serenity, Reaction};

pub async fn reaction_add(
    ctx: &serenity::Context,
    add_reaction: Reaction,
    data: &Data,
) -> Result<(), Error> {
    let user_id = add_reaction.user_id.unwrap();
    if ctx.cache.user(user_id).map_or(false, |user| user.bot) {
        return Ok(());
        // May merge with the one below.
    }
    let guild_id = add_reaction.guild_id.unwrap_or_default();
    let guild_name = get_guild_name(ctx, guild_id);

    let channel_name = get_channel_name(ctx, guild_id, add_reaction.channel_id).await;

    let redis_pool = &data.redis;
    let mut redis_conn = redis_pool.get().await?;

    let user_name = match user_id.to_user(ctx).await {
        Ok(user) => user.name,
        Err(_) => "Unknown User".to_string(),
    };

    println!(
        "\x1B[95m[{}] [#{}] {} added a reaction: {}\x1B[0m",
        guild_name, channel_name, user_name, add_reaction.emoji
    );

    let reaction_key = format!("reactions:{}", guild_id);
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
    removed_reaction: Reaction,
    data: &Data,
) -> Result<(), Error> {
    let user_id = removed_reaction.user_id.unwrap();
    if ctx.cache.user(user_id).map_or(false, |user| user.bot) {
        return Ok(());
        // May merge with the one below.
    }
    let guild_id = removed_reaction.guild_id.unwrap_or_default();
    let guild_name = get_guild_name(ctx, guild_id);
    let channel_name = get_channel_name(ctx, guild_id, removed_reaction.channel_id).await;

    let redis_pool = &data.redis;
    let mut redis_conn = redis_pool.get().await?;

    let user_name = match user_id.to_user(&ctx.http).await {
        Ok(user) => user.name,
        Err(_) => "Unknown User".to_string(),
    };

    println!(
        "\x1B[95m[{}] [#{}] {} removed a reaction: {}\x1B[0m",
        guild_name, channel_name, user_name, removed_reaction.emoji
    );

    let reaction_key = format!("reactions:{}", guild_id);
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
