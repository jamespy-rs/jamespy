use crate::{owner::owner, Context, Error};
use poise::serenity_prelude::ChannelId;

use jamespy_events::{handlers::messages::attachments_embed_fmt, helper::get_channel_name};

#[poise::command(
    prefix_command,
    category = "Owner - Messages",
    check = "owner",
    hide_in_help
)]
pub async fn msgs(
    ctx: Context<'_>,
    channel: Option<ChannelId>,
    limit: Option<usize>,
) -> Result<(), Error> {
    if limit == Some(0) {
        ctx.say("silly").await?;
        return Ok(());
    };

    let channel_id = {
        if let Some(channel) = channel {
            channel
        } else {
            ctx.channel_id().to_channel(ctx).await?.id()
        }
    };

    let msgs = {
        let channel_messages = ctx.cache().channel_messages(channel_id);

        if let Some(msgs) = channel_messages {
            let last_10_msgs: Vec<_> = msgs.iter().take(limit.unwrap_or(30)).cloned().collect();
            last_10_msgs
        } else {
            Vec::new()
        }
    };

    if msgs.is_empty() {
        ctx.say("There are no cached messages in this channel!")
            .await?;
        return Ok(());
    }

    let channel_name =
        get_channel_name(ctx.serenity_context(), msgs[0].guild_id, msgs[0].channel_id).await;

    let mut content = String::from("```ansi\n");

    for msg in msgs {
        let (attachments, embeds) = attachments_embed_fmt(&msg);

        let filtered_content = if msg.content.contains("```") {
            "-- codeblock skipped --"
        } else {
            &msg.content
        };

        // if attachments and embeds are empty no point putting the colour change in there.
        let string = format!(
            "\x1B[90m[#{channel_name}]\x1B[0m {}: {}\x1B[36m{}{}\x1B[0m\n",
            msg.author.tag(),
            filtered_content,
            attachments.as_deref().unwrap_or(""),
            embeds.as_deref().unwrap_or("")
        );

        // newline and ``` must be added after.
        if content.len() + string.len() >= 1995 {
            break;
        }

        content.push_str(&string);
    }
    // technically a condition exists where the only thing that exists is the codeblock.
    content.push_str("\n```");
    ctx.say(content).await?;

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [msgs()]
}
