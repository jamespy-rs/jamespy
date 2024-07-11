use serenity::all::CreateAllowedMentions;

use crate::{Context, Error};
use std::fmt::Write;

/// Get the info of all characters in a message.
#[poise::command(
    slash_command,
    prefix_command,
    category = "Utility",
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn charinfo(
    ctx: Context<'_>,
    #[description = "String containing characters, limited to 25."] string: String,
) -> Result<(), Error> {
    if string.len() > 25 {
        ctx.say("This command is limited to 25 characters!").await?;
        return Ok(());
    }

    let mut result = String::new();
    for c in string.chars() {
        if let Some(name) = unicode_names2::name(c) {
            let digit = c as u32;
            writeln!(
                result,
                "[`\\U{digit:08x}`](http://www.fileformat.info/info/unicode/char/{digit:08x}): {name} — {c}",
            )
            .unwrap();
        } else {
            writeln!(result, "Name not found.").unwrap();
        }
    }

    if result.len() > 2000 {
        ctx.say("Message too long.").await?;
        return Ok(());
    }

    ctx.send(
        poise::CreateReply::new().content(result).allowed_mentions(
            CreateAllowedMentions::new()
                .everyone(false)
                .all_roles(false)
                .all_users(false),
        ),
    )
    .await?;

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [charinfo()]
}
