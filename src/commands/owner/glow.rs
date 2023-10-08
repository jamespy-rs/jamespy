use poise::serenity_prelude::ChannelId;

use crate::event_handlers::glow::CONFIG;
use crate::{Context, Error};

#[poise::command(
    prefix_command,
    category = "Glow",
    hide_in_help,
    owners_only,
    subcommands("toggle", "set_channel", "reset", "enable_here")
)]
pub async fn glow(ctx: Context<'_>) -> Result<(), Error> {
    let (action, channel_id) = {
        let config = CONFIG.read().unwrap();
        (config.action, config.channel_id)
    };
    let channel_name;
    let channelid;
    if let Some(channel) = channel_id {
        channel_name = channel.name(ctx).await?;
        channelid = channel.get();
    } else {
        channel_name = "None".to_string();
        channelid = 0;
    }
    let message = format!(
        "Enabled:{}\nChannel: {} (ID:{})",
        action, channel_name, channelid
    );
    ctx.say(message).await?;

    Ok(())
}

#[poise::command(prefix_command, category = "Glow", hide_in_help, owners_only)]
pub async fn toggle(ctx: Context<'_>) -> Result<(), Error> {
    let current_value;
    let current_channel;
    let mut glowie = String::new();
    {
        let mut write_lock = CONFIG.write().unwrap();
        write_lock.action = !write_lock.action;
        current_value = write_lock.action;
        current_channel = write_lock.channel_id;
    }

    if let Some(channel) = current_channel {
        let name = channel.name(ctx).await?;
        glowie = format!("#{} (ID:{}).", name, channel.get());
    }

    let glow_channel = if glowie.is_empty() {
        "The channel to glow to is not set!".to_string()
    } else {
        format!("The channel to glow to is set to {}", glowie)
    };

    let content = format!("Glow status is set to {}!\n{}", current_value, glow_channel);
    ctx.say(content).await?;

    Ok(())
}

#[poise::command(
    rename = "set-channel",
    prefix_command,
    category = "Glow",
    hide_in_help,
    owners_only
)]
pub async fn set_channel(
    ctx: Context<'_>,
    #[description = "Pick a channel"] channel: Option<ChannelId>,
) -> Result<(), Error> {
    let new_channel;
    if let Some(ch) = channel {
        new_channel = ch;
    } else {
        new_channel = ctx.channel_id();
    }

    {
        let mut write_lock = CONFIG.write().unwrap();
        write_lock.channel_id = Some(new_channel);
    }

    let content = format!(
        "Set glow channel to: {} (ID:{})",
        new_channel.name(ctx).await?,
        new_channel.get()
    );
    ctx.say(content).await?;

    Ok(())
}

#[poise::command(prefix_command, category = "Glow", hide_in_help, owners_only)]
pub async fn reset(ctx: Context<'_>) -> Result<(), Error> {
    {
        let mut write_lock = CONFIG.write().unwrap();
        write_lock.action = false;
        write_lock.channel_id = None;
    }

    ctx.say("Glow config has been reset!").await?;

    Ok(())
}

#[poise::command(
    rename = "enable-here",
    aliases("enable"),
    prefix_command,
    category = "Glow",
    hide_in_help,
    owners_only
)]
pub async fn enable_here(ctx: Context<'_>) -> Result<(), Error> {
    let new_channel = ctx.channel_id();
    {
        let mut write_lock = CONFIG.write().unwrap();
        write_lock.action = true;
        write_lock.channel_id = Some(new_channel);
    }

    ctx.say("Enabling glow in this channel..").await?;

    Ok(())
}
