use poise::serenity_prelude::ChannelId;

use crate::{Context, Error};

#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("**Bailing out, you are on your own. Good luck.**")
        .await?;
    ctx.framework()
        .shard_manager()
        .lock()
        .await
        .shutdown_all()
        .await;
    Ok(())
}


/// Say something!
#[poise::command(prefix_command, hide_in_help, owners_only)]
pub async fn say(
    ctx: Context<'_>,
    #[description = "Channel where the message will be sent"] channel: Option<ChannelId>,
    #[description = "What to say"] #[rest] string: String,
) -> Result<(), Error> {
    let target_channel = channel.unwrap_or(ctx.channel_id());

    target_channel.say(&ctx.http(), string).await?;

    Ok(())
}
