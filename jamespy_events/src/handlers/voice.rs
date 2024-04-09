use crate::{helper::get_guild_name_override, Error};
use poise::serenity_prelude::{self as serenity, VoiceState};

pub async fn voice_state_update(
    ctx: &serenity::Context,
    old: &Option<VoiceState>,
    new: &VoiceState,
) -> Result<(), Error> {
    if let Some(old) = old {
        if old.channel_id != new.channel_id && new.channel_id.is_some() {
            handle_switch(ctx, old, new).await?;
        } else if new.channel_id.is_none() {
            handle_leave(ctx, old, new).await?;
        }
        // third case where mutes and other changes happen.
    } else {
        handle_joins(ctx, new).await?;
    }

    Ok(())
}

async fn handle_switch(
    ctx: &serenity::Context,
    old: &VoiceState,
    new: &VoiceState,
) -> Result<(), Error> {
    // unwraping this should be fine because the user should
    // have this when switching a channel, i'll know if this fails.
    // Potentially might die with no cache.
    let old_id = old.channel_id.unwrap();

    // Ditto.
    let new_id = new.channel_id.unwrap();

    // Should be fine given as voice states shouldn't be on private channels.

    let user_name = new.user_id.to_user(ctx).await?.tag();

    let guild_cache = ctx.cache.guild(new.guild_id.unwrap());
    // will fire real error in the future.
    if guild_cache.is_none() {
        return Ok(());
    }
    // will clean up the "manual" unwrap later, this is slower, probably but looks nicer.
    let guild_cache = guild_cache.unwrap();

    let old_channel = guild_cache.channels.get(&old_id).unwrap();
    let new_channel = guild_cache.channels.get(&new_id).unwrap();

    let old_name = &old_channel.name;
    let new_name = &new_channel.name;

    let guild_name = get_guild_name_override(ctx, &ctx.data(), Some(new_channel.guild_id));

    println!(
        "\x1B[32m[{guild_name}] {user_name}: {old_name} (ID:{old_id}) -> {new_name} \
         (ID:{new_id})\x1B[0m"
    );

    Ok(())
}
async fn handle_leave(
    ctx: &serenity::Context,
    old: &VoiceState,
    new: &VoiceState,
) -> Result<(), Error> {
    // There is no new channel ID.
    let channel_id = old.channel_id.unwrap();
    let user_name = new.user_id.to_user(ctx).await?.tag();

    // going to unwrap because i'm lazy and this is fine usually, private bot private issues.
    let guild_cache = ctx.cache.guild(new.guild_id.unwrap()).unwrap();

    let old_channel = guild_cache.channels.get(&channel_id).unwrap();
    let channel_name = &old_channel.name;

    let guild_name = get_guild_name_override(ctx, &ctx.data(), Some(old_channel.guild_id));

    println!("\x1B[32m[{guild_name}] {user_name} left {channel_name} (ID:{channel_id})\x1B[0m");
    Ok(())
}
async fn handle_joins(ctx: &serenity::Context, new: &VoiceState) -> Result<(), Error> {
    let channel_id = new.channel_id.unwrap();

    let user_name = new.user_id.to_user(ctx).await?.tag();

    let guild_cache = ctx.cache.guild(new.guild_id.unwrap()).unwrap();
    let channel = guild_cache.channels.get(&channel_id).unwrap();

    let channel_name = &channel.name;

    let guild_name = get_guild_name_override(ctx, &ctx.data(), Some(channel.guild_id));

    println!("\x1B[32m[{guild_name}] {user_name} joined {channel_name} (ID:{channel_id})\x1B[0m");
    Ok(())
}
