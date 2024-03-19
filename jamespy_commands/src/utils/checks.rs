use crate::{Command, Data};
use jamespy_config::Checks;

use std::collections::HashSet;
use std::sync::Arc;

use poise::serenity_prelude::User;

pub enum CommandRestrictErr {
    CommandNotFound,
    AlreadyExists,
    DoesntExist,
    FrameworkOwner,
    NotOwnerCommand,
}

pub fn handle_allow_cmd(
    commands: &[Command],
    data: &Arc<Data>,
    cmd_name: String,
    user: &User,
) -> Result<String, CommandRestrictErr> {
    let mut config = data.config.write();

    // Check if the command or its aliases match a real command.
    let command_name = get_cmd_name(commands, &cmd_name)?;

    if let Some(checks) = &mut config.command_checks {
        let set = checks.owners_single.entry(cmd_name.clone()).or_default();
        let inserted = set.insert(user.id);

        if !inserted {
            return Err(CommandRestrictErr::AlreadyExists);
        };
    } else {
        // Set checks to the new setup.
        let mut checks = Checks::new();
        let mut set = HashSet::new();
        set.insert(user.id);
        checks.owners_single.insert(cmd_name, set);
        config.command_checks = Some(checks);
    }

    config.write_config();
    Ok(command_name)
}

pub fn get_cmd_name(
    commands: &[crate::Command],
    cmd_name: &str,
) -> Result<String, CommandRestrictErr> {
    let mut command_name = String::new();
    for command in commands {
        // If the command isn't an owner command, skip.
        if !command
            .category
            .as_deref()
            .map_or(false, |c| c.to_lowercase().starts_with("owner"))
        {
            continue;
        }

        if command.name == cmd_name.to_lowercase()
            || command
                .aliases
                .iter()
                .any(|alias| alias == &cmd_name.to_lowercase())
        {
            // Check if the command is an owner command.
            if !command
                .category
                .as_deref()
                .map_or(false, |c| c.to_lowercase().starts_with("owner"))
            {
                return Err(CommandRestrictErr::NotOwnerCommand);
            }

            // Commands that require you to be a framework owner are not supported.
            if command.owners_only {
                return Err(CommandRestrictErr::FrameworkOwner);
            }

            command_name.clone_from(&command.name);
            break;
        }
    }

    if command_name.is_empty() {
        return Err(CommandRestrictErr::CommandNotFound);
    };

    Ok(command_name)
}

pub fn handle_deny_cmd(
    commands: &[crate::Command],
    data: &Arc<Data>,
    cmd_name: &str,
    user: &User,
) -> Result<String, CommandRestrictErr> {
    let mut config = data.config.write();

    // Check if the command or its aliases match a real command.
    let command_name = get_cmd_name(commands, cmd_name)?;

    if let Some(checks) = &mut config.command_checks {
        let map = &mut checks.owners_single;

        let set = map.entry(command_name.clone()).or_default();
        let stored = set.remove(&user.id);

        if set.is_empty() {
            map.remove(&command_name);
        }

        if !stored {
            return Err(CommandRestrictErr::DoesntExist);
        }
    }

    config.write_config();
    Ok(command_name)
}
