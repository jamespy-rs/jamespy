use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, ReactionType};

use crate::{Context, Error};

#[poise::command(prefix_command, aliases("kys"), owners_only, hide_in_help)]
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

/// dm a user!
#[poise::command(prefix_command, hide_in_help, owners_only)]
pub async fn dm(
    ctx: Context<'_>,
    #[description = "ID"] user_id: poise::serenity_prelude::UserId,
    #[rest]
    #[description = "Message"]
    message: String,
) -> Result<(), Error> {
    let user = user_id.to_user(&ctx).await?;
    user.direct_message(ctx, serenity::CreateMessage::default().content(message)).await?;
    Ok(())
}

/// React to a message with a specific reaction!
#[poise::command(prefix_command, hide_in_help, owners_only)]
pub async fn react(
    ctx: Context<'_>,
    #[description = "Channel where the message is"] channel_id: ChannelId,
    #[description = "Message to react to"] message_id: u64,
    #[description = "What to React with"] string: String,
) -> Result<(), Error> {
    let message = channel_id.message(&ctx.http(), message_id).await?;

    let trimmed_string = string.trim_matches('`').trim_matches('\\').to_string();

    // React to the message with the specified emoji
    let reaction = trimmed_string.parse::<ReactionType>().unwrap(); // You may want to handle parsing errors
    message.react(&ctx.http(), reaction).await?;

    Ok(())
}

