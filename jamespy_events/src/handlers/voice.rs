use std::borrow::Cow;

use crate::{
    helper::{get_guild_name_override, get_user},
    Error,
};
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
    // unwrapping this is probably fine considering i already handle this before?
    // I don't think i've seen a panic here?
    let old_id = old.channel_id.unwrap();

    // Ditto.
    let new_id = new.channel_id.unwrap();

    let user_name = match get_user(ctx, new.guild_id.unwrap(), new.user_id).await {
        Some(user) => user.tag(),
        None => return Ok(()),
    };

    let guild_cache = ctx.cache.guild(new.guild_id.unwrap());
    // will fire real error in the future.
    if guild_cache.is_none() {
        return Ok(());
    }
    // will clean up the "manual" unwrap later, this is slower, probably but looks nicer.
    let guild_cache = guild_cache.unwrap();

    let channel_old_name = guild_cache.channels.get(&old_id).map(|c| &c.name);
    let channel_new_name = guild_cache.channels.get(&new_id).map(|c| &c.name);

    // maybe i should use fixedstring directly?
    let old_name: Cow<str> = if let Some(channel_name) = channel_old_name {
        Cow::Borrowed(channel_name)
    } else {
        Cow::Borrowed("None")
    };

    // ditto
    let new_name: Cow<str> = if let Some(channel_name) = channel_new_name {
        Cow::Borrowed(channel_name)
    } else {
        Cow::Borrowed("None")
    };

    let guild_name = get_guild_name_override(ctx, &ctx.data(), new.guild_id);

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
    // they are leaving so old should hold the guild_id, see handle_joins for justification.
    let user_name = match get_user(ctx, new.guild_id.unwrap(), new.user_id).await {
        Some(user) => user.tag(),
        None => return Ok(()),
    };

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

    // unwrapping the guild should be fine here unless the discord api is being funky
    // they are joining, so a guild_id is present.
    let user_name = match get_user(ctx, new.guild_id.unwrap(), new.user_id).await {
        Some(user) => user.tag(),
        None => return Ok(()),
    };

    let guild_cache = ctx.cache.guild(new.guild_id.unwrap()).unwrap();
    let channel = guild_cache.channels.get(&channel_id).unwrap();

    let channel_name = &channel.name;

    let guild_name = get_guild_name_override(ctx, &ctx.data(), Some(channel.guild_id));

    println!("\x1B[32m[{guild_name}] {user_name} joined {channel_name} (ID:{channel_id})\x1B[0m");
    Ok(())
}
