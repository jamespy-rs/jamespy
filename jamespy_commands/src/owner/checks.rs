use std::collections::HashSet;

use crate::{Context, Error};

use jamespy_config::Checks;
use poise::serenity_prelude::User;

// TODO: verify if the command is real.
// TODO: check if they already had access.


#[poise::command(
    rename = "allow-owner-cmd",
    aliases("aoc"),
    prefix_command,
    hide_in_help,
    owners_only
)]
pub async fn allow_owner_cmd(ctx: Context<'_>, cmd_name: String, user: User) -> Result<(), Error> {
    {
        let data = &ctx.data();
        let mut config = data.config.write().unwrap();

        // TODO: possibly make this actually good code.
        if let Some(checks) = &mut config.command_checks {
            let set = checks.owners_single.entry(cmd_name.clone()).or_default();
            set.insert(user.id);
        } else {
            // Set checks to the new setup.
            let mut checks = Checks::new();
            let mut set = HashSet::new();
            set.insert(user.id);
            checks.owners_single.insert(cmd_name.clone(), set);
            config.command_checks = Some(checks);
        }

        config.write_config();
    };

    ctx.say(format!(
        "Successfully allowed <@{}> to use `{}`!",
        user.id, cmd_name
    ))
    .await?;

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
    {
        let data = &ctx.data();
        let mut config = data.config.write().unwrap();

        // TODO: possibly make this actually good code.
        if let Some(checks) = &mut config.command_checks {
            let set = checks.owners_single.entry(cmd_name.clone()).or_default();
            set.remove(&user.id);
        }

        config.write_config();
    };

    ctx.say(format!(
        "Successfully denied <@{}> to use `{}`!",
        user.id, cmd_name
    ))
    .await?;

    Ok(())
}

pub fn commands() -> [crate::Command; 2] {
    [allow_owner_cmd(), deny_owner_cmd()]
}
