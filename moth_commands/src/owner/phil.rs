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
    let no_nos = [
        442299248172597258_u64,
        306853470638440460,
        685796754939052047,
        314081534824939530,
        871823102068260915,
        198903530394877952,
        575119446134226964,
        838585979610726400,
        724445579811487775,
        668541709017153592,
        793282666367680564,
        906664596679577620,
        208101469365207041,
        815083930591297546,
        1136901818882994207,
    ];

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
                    // if discord ever adds other flags I should store them beforehand.
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
