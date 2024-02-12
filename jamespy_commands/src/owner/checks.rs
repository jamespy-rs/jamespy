use crate::{Context, Error};

use crate::utils::{get_cmd_name, handle_allow_cmd, handle_deny_cmd, CommandRestrictErr};
use jamespy_config::Checks;
use poise::serenity_prelude::{self as serenity, User};

// This entire module needs new command/function names.

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
            CommandRestrictErr::FrameworkOwner => {
                "This command requires you to be an owner in the framework!"
            }
            CommandRestrictErr::NotOwnerCommand => "This command is not an owner command!",
            _ => "", // No other errors should fire in this code.
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
pub async fn deny_owner_cmd(ctx: Context<'_>, user: User, cmd_name: String) -> Result<(), Error> {
    let statement = match handle_deny_cmd(
        &ctx.framework().options.commands,
        ctx.data(),
        cmd_name,
        &user,
    ) {
        Ok(cmd) => format!("Successfully denied {} to use `{}`!", user, cmd),
        Err(err) => match err {
            CommandRestrictErr::CommandNotFound => "Could not find command!",
            CommandRestrictErr::FrameworkOwner => {
                "This command requires you to be an owner in the framework!"
            }
            CommandRestrictErr::NotOwnerCommand => "This command is not an owner command!",
            CommandRestrictErr::DoesntExist => {
                "Cannot remove
            permissions they don't have!"
            }
            _ => "", // No other errors should fire in this code.
        }
        .to_string(),
    };

    ctx.say(statement).await?;

    Ok(())
}

#[poise::command(
    rename = "owner-overrides",
    aliases("oo"),
    prefix_command,
    hide_in_help,
    owners_only,
    subcommands("user", "cmd"),
    subcommand_required
)]
pub async fn owner_overrides(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(prefix_command, hide_in_help, owners_only)]
pub async fn user(ctx: Context<'_>, user: User) -> Result<(), Error> {
    let overrides = {
        let data = ctx.data();
        let config = data.config.read().unwrap();

        if let Some(checks) = &config.command_checks {
            let mut single_overrides = Vec::new();
            for single_check in checks.owners_single.iter() {
                if single_check.1.contains(&user.id) {
                    single_overrides.push(single_check.0.clone())
                }
            }

            (
                checks.owners_all.get(&user.id).cloned(),
                Some(single_overrides),
            )
        } else {
            (None, None)
        }
    };

    // TODO: fix this mess, and paginate.
    match overrides {
        (Some(_), Some(single_overrides)) => {
            let embed = if !single_overrides.is_empty() {
                let mut description = String::new();
                for over in single_overrides {
                    description.push_str(&format!("**{over}**\n"))
                }

                let embed = serenity::CreateEmbed::new()
                    .title("Extra Owner Overrides")
                    .description(description);
                Some(embed)
            } else {
                None
            };

            if let Some(embed) = embed {
                let msg = poise::CreateReply::new()
                    .content("This user has overrides for all owner commands!")
                    .embed(embed);
                ctx.send(msg).await?;
            } else {
                let msg = poise::CreateReply::new()
                    .content("This user has overrides for all owner commands!");
                ctx.send(msg).await?;
            }
        }
        (None, Some(single_overrides)) => {
            let embed = if !single_overrides.is_empty() {
                let mut description = String::new();
                for over in single_overrides {
                    description.push_str(&format!("**{over}**\n"))
                }

                let embed = serenity::CreateEmbed::new()
                    .title("Owner Overrides")
                    .description(description);
                Some(embed)
            } else {
                None
            };

            if let Some(embed) = embed {
                let msg = poise::CreateReply::new().embed(embed);
                ctx.send(msg).await?;
            } else {
                ctx.say("This user doesn't have any overrides!").await?;
            }
        }
        _ => {
            // This should be the only other case that can happen.
            // The vec will always exist (except for when no config) but will just be empty.
            ctx.say("No overrides exist!").await?;
        }
    }

    Ok(())
}

#[poise::command(prefix_command, hide_in_help, owners_only)]
pub async fn cmd(ctx: Context<'_>, cmd_name: String) -> Result<(), Error> {
    let res = get_cmd_name(&ctx.framework().options.commands, &cmd_name);

    match res {
        Ok(name) => {
            cmd_overrides(ctx, &name).await?;
        }
        Err(err) => {
            let _ = match err {
                CommandRestrictErr::CommandNotFound => ctx.say("No command was found!").await?,
                CommandRestrictErr::FrameworkOwner => {
                    ctx.say("This command requires framework owner!").await?
                }
                _ => ctx.say("Unknown Error type!").await?, // This shouldn't fire.
            };
        }
    };

    Ok(())
}

// TODO: paginate.
pub async fn cmd_overrides(ctx: Context<'_>, cmd_name: &str) -> Result<(), Error> {
    let overrides = {
        let data = ctx.data();
        let config = data.config.read().unwrap();

        if let Some(checks) = &config.command_checks {
            checks.owners_single.get(cmd_name).cloned()
        } else {
            None
        }
    };

    if let Some(over) = overrides {
        if over.is_empty() {
            ctx.say("No overrrides for this command!").await?;
            return Ok(());
        }

        let mut description = String::new();
        for u in over {
            description.push_str(&format!("<@{u}>\n"))
        }
        let embed = serenity::CreateEmbed::new()
            .title(format!("Overrides for {cmd_name}"))
            .description(description);

        let msg = poise::CreateReply::new().embed(embed);
        ctx.send(msg).await?;
    }

    Ok(())
}

#[poise::command(aliases("ao"), prefix_command, hide_in_help, owners_only)]
pub async fn allow_owner(ctx: Context<'_>, user: User) -> Result<(), Error> {
    let statement = match handle_allow_owner(ctx, user.clone()) {
        Ok(_) => format!("Successfully allowed {user} to use owner commands!"),
        Err(err) => match err {
            CommandRestrictErr::AlreadyExists => format!("{user} already has a bypass!"),
            _ => String::from("Error while handling error: Unexpected Error!"),
        },
    };

    ctx.say(statement).await?;
    Ok(())
}

fn handle_allow_owner(ctx: Context<'_>, user: User) -> Result<(), CommandRestrictErr> {
    let data = ctx.data();
    let mut config = data.config.write().unwrap();

    if let Some(checks) = &mut config.command_checks {
        let newly_added = &checks.owners_all.insert(user.id);
        if !newly_added {
            return Err(CommandRestrictErr::AlreadyExists);
        }
    } else {
        let mut checks = Checks::new();
        checks.owners_all.insert(user.id);
        config.command_checks = Some(checks);
    }

    config.write_config();

    Ok(())
}

#[poise::command(aliases("do"), prefix_command, hide_in_help, owners_only)]
pub async fn deny_owner(ctx: Context<'_>, user: User) -> Result<(), Error> {
    let statement = match handle_deny_owner(ctx, user.clone()) {
        Ok(_) => format!("Successfully allowed {user} to use owner commands!"),
        Err(err) => match err {
            CommandRestrictErr::DoesntExist => format!("{user} doesn't have a bypass!"),
            _ => String::from("Error while handling error: Unexpected Error!"), // No other errors should fire in this code.
        },
    };

    ctx.say(statement).await?;
    Ok(())
}

fn handle_deny_owner(ctx: Context<'_>, user: User) -> Result<(), CommandRestrictErr> {
    let data = ctx.data();
    let mut config = data.config.write().unwrap();

    if let Some(checks) = &mut config.command_checks {
        let present = &checks.owners_all.remove(&user.id);
        if !present {
            return Err(CommandRestrictErr::DoesntExist);
        }
    } else {
        return Err(CommandRestrictErr::DoesntExist);
    }

    config.write_config();

    Ok(())
}

pub fn commands() -> [crate::Command; 5] {
    [
        allow_owner_cmd(),
        deny_owner_cmd(),
        owner_overrides(),
        allow_owner(),
        deny_owner(),
    ]
}
