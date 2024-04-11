use poise::serenity_prelude::GuildChannel;

use crate::{owner::owner, Context, Error};

/// I connect
#[poise::command(
    aliases("conn"),
    prefix_command,
    category = "Owner - Utility",
    channel_cooldown = "5",
    check = "owner",
    guild_only,
    hide_in_help
)]
pub async fn connect(ctx: Context<'_>, channel: GuildChannel) -> Result<(), Error> {
    ctx.data()
        .songbird
        .join(ctx.guild_id().unwrap(), channel.id)
        .await?;

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [connect()]
}
