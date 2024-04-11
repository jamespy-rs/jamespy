use poise::serenity_prelude::Channel;

use crate::{owner::owner, Context, Error};

/// Connect to a voice channel.
#[poise::command(
    aliases("conn"),
    prefix_command,
    category = "Owner - Utility",
    channel_cooldown = "5",
    check = "owner",
    guild_only,
    hide_in_help
)]
pub async fn connect(ctx: Context<'_>, c: Channel, lockout: Option<bool>) -> Result<(), Error> {
    // gets around http request error due to serenity@next bug.
    let channel = c.guild().unwrap();

    let guild_id = ctx.guild_id().unwrap();

    if let Some(lockout) = lockout {
        if lockout {
            ctx.data().mod_mode.insert(guild_id);

            // stop anything that is going on when entering this mode.
            if let Some(handler_lock) = ctx.data().songbird.get(guild_id) {
                handler_lock.lock().await.stop();
            }
            ctx.say("Connecting to channel and enabling Modmode.")
                .await?;
        } else {
            ctx.data().mod_mode.remove(&guild_id);
            ctx.say("Connecting to channel and disabling Modmode.")
                .await?;
        }
    }

    ctx.data()
        .songbird
        .join(ctx.guild_id().unwrap(), channel.id)
        .await?;

    Ok(())
}

/// Connect to a voice channel.
#[poise::command(
    prefix_command,
    category = "Owner - Utility",
    channel_cooldown = "5",
    check = "owner",
    guild_only,
    hide_in_help
)]
pub async fn modmode(ctx: Context<'_>, lockout: Option<bool>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    if let Some(lockout) = lockout {
        if lockout {
            ctx.data().mod_mode.insert(guild_id);

            // stop anything that is going on when entering this mode.
            if let Some(handler_lock) = ctx.data().songbird.get(guild_id) {
                handler_lock.lock().await.stop();
            }
            ctx.say("Enabling Modmode.").await?;
        } else {
            ctx.data().mod_mode.remove(&guild_id);
            ctx.say("Disabling Modmode.").await?;
        }
    } else if ctx.data().mod_mode.contains(&guild_id) {
        ctx.say("Modmode is currently enabled.").await?;
    } else {
        ctx.say("Modmode is currently disabled.").await?;
    }

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 2] {
    [connect(), modmode()]
}
