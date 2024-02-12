
use crate::{Context, Error};

use crate::utils::{handle_allow_cmd, handle_deny_cmd, CommandRestrictErr};
use poise::serenity_prelude::User;

#[poise::command(
    rename = "allow-owner-cmd",
    aliases("aoc"),
    prefix_command,
    hide_in_help,
    owners_only
)]
pub async fn allow_owner_cmd(ctx: Context<'_>, user: User, cmd_name: String) -> Result<(), Error> {
    let statement = match handle_allow_cmd(
        &ctx.framework().options.commands,
        ctx.data(),
        cmd_name,
        &user,
    ) {
        Ok(cmd) => format!("Successfully allowed {} to use `{}`!", user, cmd),
        Err(err) => match err {
            CommandRestrictErr::CommandNotFound => "Could not find command!",
            CommandRestrictErr::AlreadyExists => "The user is already allowed to use this!",
            _ => "", // This error is not used at this point.
        }
        .to_string(),
    };

    ctx.say(statement).await?;

    Ok(())
}

#[poise::command(
    rename = "deny-owner-cmd",
    aliases("doc"),
    prefix_command,
    hide_in_help,
    owners_only
)]
pub async fn deny_owner_cmd(ctx: Context<'_>, cmd_name: String, user: User) -> Result<(), Error> {
    let statement = match handle_deny_cmd(
        &ctx.framework().options.commands,
        ctx.data(),
        cmd_name,
        &user,
    ) {
        Ok(cmd) => format!("Successfully denied {} to use `{}`!", user, cmd),
        Err(err) => match err {
            CommandRestrictErr::CommandNotFound => "Could not find command!",
            CommandRestrictErr::DoesntExist => "Cannot remove
            permissions they don't have!",
            _ => "", // This error is not used at this point.
        }
        .to_string(),
    };

    ctx.say(statement).await?;

    Ok(())
}

pub fn commands() -> [crate::Command; 2] {
    [allow_owner_cmd(), deny_owner_cmd()]
}
