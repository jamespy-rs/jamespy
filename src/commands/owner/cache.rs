use poise::serenity_prelude::{self as serenity, ChannelType};
use std::collections::HashMap;

use crate::{
    utils::{self, misc::channel_type_to_string},
    Context, Error,
};

/// View/set max messages cached per channel
#[poise::command(
    rename = "max-messages",
    prefix_command,
    category = "Cache",
    owners_only,
    hide_in_help
)]
pub async fn max_messages(
    ctx: Context<'_>,
    #[description = "What to say"] value: Option<u16>,
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

#[poise::command(
    rename = "guild-message-cache",
    prefix_command,
    category = "Cache",
    guild_only,
    owners_only,
    hide_in_help
)]
pub async fn guild_message_cache(
    ctx: Context<'_>,
    #[description = "Which guild to check"] guild_id: Option<u64>,
) -> Result<(), Error> {
    // This still doesn't include threads.
    let cache = &ctx.cache();
    let guild_id = guild_id.unwrap_or(ctx.guild_id().unwrap().get());

    let channels = cache.guild_channels(guild_id).unwrap().clone(); // future james no clone this.
    let threads = cache.guild(guild_id).unwrap().threads.clone();
    // does guild.channels() work better?

    let mut channel_info_vec: Vec<(String, usize)> = Vec::new();

    // future james handle better.
    for channel in channels {
        if channel.1.kind != ChannelType::Category && channel.1.kind != ChannelType::Voice {
            if let Some(messages) = cache.channel_messages(channel.0) {
                let message_count = messages.len();
                let kind = channel_type_to_string(channel.1.kind);
                let channel_info =
                    format!("**#{} ({}): {}**\n", channel.1.name, kind, message_count);
                channel_info_vec.push((channel_info, message_count));
            }
        }
    }

    for thread in threads {
        if let Some(messages) = cache.channel_messages(thread.id) {
            let message_count = messages.len();
            let kind = channel_type_to_string(thread.kind);
            let channel_info = format!("**#{} ({}): {}**\n", thread.name, kind, message_count);
            channel_info_vec.push((channel_info, message_count));
        }
    }
    channel_info_vec.sort_by(|a, b| b.1.cmp(&a.1));

    let mut current_page = String::new();
    let mut total_messages_cached = 0;
    let mut pages: Vec<String> = Vec::new();

    for (channel_info, message_count) in channel_info_vec {
        total_messages_cached += message_count;

        // Check if adding this channel to the current page exceeds the character limit
        if current_page.len() + channel_info.len() > 2000 {
            pages.push(current_page.clone());
            current_page.clear();
        }

        current_page.push_str(&channel_info);
    }

    if !current_page.is_empty() {
        pages.push(current_page);
    }

    let pages_ref: Vec<&str> = pages.iter().map(String::as_str).collect();
    utils::cache::guild_message_cache_builder(ctx, &pages_ref, total_messages_cached).await?;
    Ok(())
}

/// prints all the cached users!
#[poise::command(
    rename = "cached-users-raw",
    prefix_command,
    category = "Cache",
    check = "cachestats",
    hide_in_help
)]
pub async fn cached_users_raw(ctx: Context<'_>) -> Result<(), Error> {
    let mut users = HashMap::new();
    {
        for guild in ctx.cache().guilds() {
            for member in &ctx.cache().guild(guild).unwrap().members {
                users.insert(*member.0, format!("{:?}", member.1));
            }
        }
    };

    let bytes: Vec<u8> = format!("{users:?}").into();

    ctx.send(
        poise::CreateReply::default()
            .content(format!("The cache contains **{}** users", users.len()))
            .attachment(serenity::CreateAttachment::bytes(
                bytes,
                "raw_users.txt".to_string(),
            )),
    )
    .await?;
    Ok(())
}

// Me and Ruben because idk why he wants this permission.
pub async fn cachestats(ctx: Context<'_>) -> Result<bool, Error> {
    let allowed_users = [158567567487795200, 291089948709486593];
    let user_id = ctx.author().id.get();
    let cachestats = allowed_users.contains(&user_id);

    Ok(cachestats)
}

pub fn commands() -> [crate::Command; 3] {
    [max_messages(), guild_message_cache(), cached_users_raw()]
}
