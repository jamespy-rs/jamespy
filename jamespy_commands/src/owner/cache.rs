use crate::{Context, Error};

/// View/set max messages cached per channel.
#[poise::command(
    rename = "max-messages",
    prefix_command,
    category = "Cache",
    owners_only,
    hide_in_help
)]
pub async fn max_messages(
    ctx: Context<'_>,
    #[description = "Set this value to change the cache limit."] value: Option<u16>,
) -> Result<(), Error> {
    if let Some(val) = value {
        ctx.say(format!(
            "Max messages cached per channel set: **{}** -> **{}**",
            ctx.cache().settings().max_messages,
            val
        ))
        .await?;
        ctx.cache().set_max_messages(val.into());
    } else {
        ctx.say(format!(
            "Max messages cached per channel is set to: **{}**",
            ctx.cache().settings().max_messages
        ))
        .await?;
    }
    Ok(())
}

pub fn commands() -> [crate::Command; 1] {
    [max_messages()]
}
