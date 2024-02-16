pub mod cache;
pub mod checks;
pub mod database;
pub mod other;
pub mod presence;
pub mod spy_guild;

use crate::{Context, Error};

/// ALL owner commands should have a category that starts with owner.
/// Well, not all, only ones that are intended to be given out to trusted users.

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    {
        cache::commands()
            .into_iter()
            .chain(checks::commands())
            .chain(database::commands())
            .chain(presence::commands())
            .chain(other::commands())
            .chain(spy_guild::commands())
            .collect()
    }
}

/// I use this check instead of the default `owners_only` check
/// When i want to be able to temporarily give access to specific owner commands
/// This executes after `command_check` is executed, so this works.
pub async fn owner(ctx: Context<'_>) -> Result<bool, Error> {
    let user_id = &ctx.author().id;

    // Owners will always be able to execute.
    if ctx.framework().options.owners.get(user_id).is_some() {
        return Ok(true);
    };

    if let Some(checks) = &ctx.data().config.read().unwrap().command_checks {
        if checks.owners_all.contains(user_id) {
            return Ok(true);
        }

        let cmd_name = &ctx.command().name;
        if let Some(cmd_override) = checks.owners_single.get(cmd_name) {
            return Ok(cmd_override
                .iter()
                .find(|&user| user == user_id)
                .is_some_and(|_| true));
        }
    };

    Ok(false)
}
