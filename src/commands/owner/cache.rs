use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelType, CreateEmbedFooter};

use crate::{Context, Error};


/// View/set max messages cached per channel
#[poise::command(rename = "max-messages", prefix_command, category = "Cache", owners_only, hide_in_help)]
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

#[poise::command(rename = "cache-stats", aliases("cache_stats", "cache_status", "cache-status"), prefix_command, category = "Cache", owners_only, hide_in_help)]
pub async fn cache_stats(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let cache = &ctx.serenity_context().cache;

    let guilds = cache.guild_count();
    let users = cache.user_count();
    let channels = cache.guild_channel_count();
    let categories = cache.category_count();
    let shards = cache.shard_count();

    let unknown_members = cache.unknown_members();
    let unavailable_guilds_len = cache.unavailable_guilds().len(); // Now either this works right or doesn't.

    let settings = cache.settings();
    let max_messages = settings.max_messages;
    let cache_guilds = settings.cache_guilds;
    let cache_channels = settings.cache_channels;
    let cache_users = settings.cache_users;
    let time_to_live = settings.time_to_live;

    let normal_stats = format!("Guilds: **{}**\nUsers: **{}**\nChannels: **{}**\nCategories: **{}**\nShards: **{}**", guilds, users, channels, categories, shards);
    let unknown_stats = format!("Unknown Members: **{}**\nUnavailable Guilds: **{}**", unknown_members, unavailable_guilds_len);
    let settings_value = format!("Max Messages: **{}**\nCache Guilds?: **{}**\nCache Channels?: **{}**\nCache Users?: **{}**\nTime to Live: **{:?}**", max_messages, cache_guilds, cache_channels, cache_users, time_to_live);

    let embed = serenity::CreateEmbed::default()
    .title("Cache Statistics")
    .field("Normal Stats", normal_stats, true)
    .field("Unknown Cache Stats", unknown_stats, true)
    .field("Cache Settings", settings_value, true);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(rename = "guild-message-cache", prefix_command, category = "Cache", guild_only, owners_only, hide_in_help)]
pub async fn guild_message_cache(
    ctx: Context<'_>,
    #[description = "What to say"] guild_id: Option<u64>,
) -> Result<(), Error> {
    let cache = &ctx.serenity_context().cache;

    let guild_id = guild_id.unwrap_or(ctx.guild_id().unwrap().get());

    let channels = cache.guild_channels(guild_id).unwrap();
    let mut channels_with_counts: Vec<(String, usize)> = Vec::new();
    let mut total_messages_cached = 0;


    for channel in channels {
        if channel.1.kind != ChannelType::Category && channel.1.kind != ChannelType::Voice {
            if let Some(messages) = cache.channel_messages(channel.0) {
                let message_count = messages.len();
                total_messages_cached += message_count;
                channels_with_counts.push((channel.1.name, message_count));
            }
        }
    }

    channels_with_counts.sort_by(|a, b| b.1.cmp(&a.1));

    let mut channel_string = String::new();

    for (channel_name, message_count) in channels_with_counts {
        channel_string.push_str(&format!("**#{}: {}**\n", channel_name, message_count));
    }
    let embed = serenity::CreateEmbed::default().title("Channels with most cached messages.").description(channel_string)
    .footer(CreateEmbedFooter::new(format!("Total messages cached in the guild: {}", total_messages_cached)));
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}




/// prints all the cached users!
#[poise::command(rename = "cached-users-raw", prefix_command, category = "Cache", owners_only, hide_in_help)]
pub async fn cached_users_raw(ctx: Context<'_>) -> Result<(), Error> {
    let users = ctx.serenity_context().cache.users();
    let user_count = ctx.serenity_context().cache.user_count();
    ctx.send(poise::CreateReply::default().content(format!("The cache contains **{}** users", user_count)).attachment(serenity::CreateAttachment::bytes(format!("{:?}", users), format!("raw_users.txt")))).await?;
    Ok(())
}

/// Prints a formatted list of cached users.
#[poise::command(rename = "cached-users", prefix_command, category = "Cache", owners_only, hide_in_help)]
pub async fn cached_users(ctx: Context<'_>) -> Result<(), Error> {
    let cache = ctx.serenity_context().cache.clone();
    let user_count = cache.user_count();

    let mut user_info = String::new();

    let user_ids = cache.users();

    for entry in user_ids.iter() {
        let user_id = entry.key();
        let user = entry.value().clone();

        let user_name = &user.name;
        let discriminator = user.discriminator.map(|d| d.to_string()).unwrap_or_else(|| "0000".to_owned());
        let global_name = user.global_name.as_ref().map(|s| s.as_str()).unwrap_or("None");
        let avatar_url = &user.avatar_url().unwrap_or("None".to_owned());
        let bot = &user.bot;
        let banner_url = &user.banner_url().unwrap_or("None".to_owned());
        let public_flags = &user.public_flags.unwrap_or_default();

        user_info.push_str(&format!(
            "ID: {}\nNAME: {}\nDISCRIMINATOR: {}\nDISPLAY NAME: {}\nAVATAR_URL: {}\nBANNER_URL: {}\nBOT: {}\nFLAGS: {:?}\n----------\n",
            user_id, user_name, discriminator, global_name, avatar_url, banner_url, bot, public_flags
        ));
    }
    let attachment = serenity::CreateAttachment::bytes(format!("{}", user_info), format!("users.txt"));
    ctx.send(poise::CreateReply::default().content(format!("The cache contains **{}** users", user_count)).attachment(attachment)).await?;

    Ok(())
}
