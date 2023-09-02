use poise::serenity_prelude as serenity;
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
    #[description = "What to say"] string: String,
) -> Result<(), Error> {
    let target_channel = channel.unwrap_or(ctx.channel_id());

    target_channel.say(&ctx.http(), string).await?;

    Ok(())
}

/// prints all the cached users!
#[poise::command(rename = "cached-users-raw", prefix_command, category = "Debug", owners_only, hide_in_help)]
pub async fn cached_users_raw(ctx: Context<'_>) -> Result<(), Error> {
    let users = ctx.serenity_context().cache.users();
    let user_count = ctx.serenity_context().cache.user_count();
    ctx.send(poise::CreateReply::default().content(format!("The cache contains **{}** users", user_count)).attachment(serenity::CreateAttachment::bytes(format!("{:?}", users), format!("filename.txt")))).await?;
    Ok(())
}

/// View/set max messages cached per channel
#[poise::command(rename = "max-messages", prefix_command, category = "Management", owners_only, hide_in_help)]
pub async fn max_messages(
    ctx: Context<'_>,
    #[description = "What to say"] value: Option<u16>,
) -> Result<(), Error> {
    if let Some(val) = value {
        ctx.say(format!("Max messages cached per channel set: **{}** -> **{}**", ctx.serenity_context().cache.settings().max_messages, val)).await?;
        ctx.serenity_context().cache.set_max_messages(val.into())
    } else {
        ctx.say(format!("Max messages cached per channel is set to: **{}**", ctx.serenity_context().cache.settings().max_messages)).await?;

    }
    Ok(())
}
