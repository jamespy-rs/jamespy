use serenity::all::{EditMember, GuildMemberFlags};

use crate::{Context, Error};

#[poise::command(
    rename = "unverify-all",
    prefix_command,
    required_bot_permissions = "MODERATE_MEMBERS",
    hide_in_help,
    owners_only,
    guild_only
)]
pub async fn unverify_all(ctx: Context<'_>) -> Result<(), Error> {
    let no_nos = [];

    let users = {
        let Some(cache) = ctx.guild() else {
            ctx.say("Cannot find guild in cache.").await?;
            return Ok(());
        };

        cache
            .members
            .iter()
            .filter(|m| m.flags.contains(GuildMemberFlags::BYPASSES_VERIFICATION))
            .map(|m| m.user.id)
            .collect::<Vec<_>>()
    };

    ctx.say("Unverifying all accounts but mod alts...").await?;

    let guild_id = ctx.guild_id().unwrap();
    for user_id in users {
        if !no_nos.contains(&user_id.get()) {
            guild_id
                .edit_member(
                    ctx.http(),
                    user_id,
                    EditMember::new().flags(GuildMemberFlags::empty()),
                )
                .await?;
        }
    }

    ctx.say("complete!").await?;

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [unverify_all()]
}
