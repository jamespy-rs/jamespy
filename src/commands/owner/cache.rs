use std::borrow::Cow;

use poise::serenity_prelude::{self as serenity, ChannelType, GuildId};

use crate::{
    utils::{self, misc::channel_type_to_string},
    Context, Error,
};
use typesize::TypeSize;

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

struct CacheData {
    name: String,
    size: usize,
    is_collection: bool,
    value: String,
}

#[poise::command(
    rename = "cache-stats",
    aliases("cache_stats", "cache_status", "cache-status"),
    prefix_command,
    category = "Cache",
    check = "cachestats",
    hide_in_help,
    subcommands("guild", "settings", "legacy")
)]
pub async fn cache_stats(ctx: Context<'_>) -> Result<(), Error> {
    let cache = &ctx.cache();

    let mut sum_size: usize = 0;
    let mut fields = Vec::new();
    for field in cache.get_size_details() {
        let name = format!("`{}`", field.name);
        let size = field.size;
        sum_size += size;
        if let Some(count) = field.collection_items {
            let (count, size_per): (Cow<'_, str>, Cow<'_, str>) = if count == 0 {
                (Cow::Owned("0".to_string()), Cow::Owned("N/A".to_string()))
            } else {
                let size_of = format!("{}", (field.size / count));
                let count_fmt = count.to_string();
                (Cow::Owned(count_fmt), Cow::Owned(size_of))
            };

            fields.push(CacheData {
                name,
                size,
                is_collection: true,
                value: format!("Count: `{count}`\nSize: `{size}b`\nAverage size: `{size_per}b`"),
            });
        } else {
            fields.push(CacheData {
                name,
                size,
                is_collection: false,
                value: format!("Size: `{size}b`"),
            });
        }
    }

    fields.sort_by_key(|field| field.size);
    fields.sort_by_key(|field| field.is_collection);
    fields.reverse();

    let embed = serenity::CreateEmbed::default()
        .title("Cache Statistics")
        .fields(fields.into_iter().map(|f| (f.name, f.value, true)))
        .footer(serenity::CreateEmbedFooter::new(format!(
            "Total size: {}b",
            sum_size
        )));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(prefix_command, category = "Cache", hide_in_help, owners_only)]
pub async fn guild(ctx: Context<'_>, guild_id: GuildId) -> Result<(), Error> {
    let cache = &ctx.cache();

    let mut sum_size: usize = 0;
    let mut fields = Vec::new();
    if cache.guild(guild_id).is_some() {
        for field in cache.guild(guild_id).unwrap().get_size_details() {
            let name = format!("`{}`", field.name);
            let size = field.size;
            sum_size += size;
            if let Some(count) = field.collection_items {
                let (count, size_per): (Cow<'_, str>, Cow<'_, str>) = if count == 0 {
                    (Cow::Owned("0".to_string()), Cow::Owned("N/A".to_string()))
                } else {
                    let size_of = format!("{}b", (field.size / count));
                    let count_fmt = count.to_string();
                    (Cow::Owned(count_fmt), Cow::Owned(size_of))
                };

                fields.push(CacheData {
                    name,
                    size,
                    is_collection: true,
                    value: format!("Count: `{count}`\nSize: `{size}`\nAverage size: `{size_per}b`"),
                });
            } else {
                fields.push(CacheData {
                    name,
                    size,
                    is_collection: false,
                    value: format!("Size: `{size}b`"),
                });
            }
        }

        fields.sort_by_key(|field| field.size);
        fields.sort_by_key(|field| field.is_collection);
        fields.reverse();

        let embed = serenity::CreateEmbed::default()
            .title("Cache Statistics")
            .fields(fields.into_iter().map(|f| (f.name, f.value, true)))
            .footer(serenity::CreateEmbedFooter::new(format!(
                "Total size: {}b",
                sum_size
            )));

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    }

    Ok(())
}

#[poise::command(prefix_command, category = "Cache", hide_in_help, owners_only)]
pub async fn legacy(ctx: Context<'_>) -> Result<(), Error> {
    let cache = &ctx.serenity_context().cache;

    let guilds = cache.guild_count();
    let users = cache.user_count();
    let channels = cache.guild_channel_count();
    let shards = cache.shard_count();

    let unknown_members = cache.unknown_members();
    let unavailable_guilds_len = cache.unavailable_guilds().len();

    let settings = cache.settings().clone();
    let max_messages = settings.max_messages;
    let cache_guilds = settings.cache_guilds;
    let cache_channels = settings.cache_channels;
    let cache_users = settings.cache_users;
    let time_to_live = settings.time_to_live;

    let normal_stats = format!(
        "Guilds: **{}**\nUsers: **{}**\nChannels: **{}**\nShards: **{}**",
        guilds, users, channels, shards
    );
    let unknown_stats = format!(
        "Unknown Members: **{}**\nUnavailable Guilds: **{}**",
        unknown_members, unavailable_guilds_len
    );
    let settings_value = format!("Max Messages: **{}**\nCache Guilds?: **{}**\nCache Channels?: **{}**\nCache Users?: **{}**\nTime to Live: **{:?}**", max_messages, cache_guilds, cache_channels, cache_users, time_to_live);

    let embed = serenity::CreateEmbed::default()
        .title("Cache Statistics")
        .field("Normal Stats", normal_stats, true)
        .field("Unknown Cache Stats", unknown_stats, true)
        .field("Cache Settings", settings_value, true);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(prefix_command, category = "Cache", hide_in_help, owners_only)]
pub async fn settings(ctx: Context<'_>) -> Result<(), Error> {
    let cache = &ctx.cache();

    let settings = cache.settings().clone();
    let max_messages = settings.max_messages;
    let cache_guilds = settings.cache_guilds;
    let cache_channels = settings.cache_channels;
    let cache_users = settings.cache_users;
    let time_to_live = settings.time_to_live;

    let settings_value = format!("Max Messages: **{}**\nCache Guilds?: **{}**\nCache Channels?: **{}**\nCache Users?: **{}**\nTime to Live: **{:?}**", max_messages, cache_guilds, cache_channels, cache_users, time_to_live);

    let embed = serenity::CreateEmbed::default()
        .title("Cache settings")
        .description(settings_value);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

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
    let users = ctx.cache().users();
    let user_count = ctx.cache().user_count();
    ctx.send(
        poise::CreateReply::default()
            .content(format!("The cache contains **{}** users", user_count))
            .attachment(serenity::CreateAttachment::bytes(
                format!("{:?}", users),
                "raw_users.txt".to_string(),
            )),
    )
    .await?;
    Ok(())
}

/// Prints a formatted list of cached users.
#[poise::command(
    rename = "cached-users",
    prefix_command,
    category = "Cache",
    check = "cachestats",
    hide_in_help
)]
pub async fn cached_users(ctx: Context<'_>) -> Result<(), Error> {
    let cache = ctx.cache();
    let user_count = cache.user_count();

    let mut user_info = String::new();

    let user_ids = cache.users();

    for entry in user_ids.iter() {
        let user_id = entry.key();
        let user = entry.value().clone();

        let user_name = &user.name;
        let discriminator = user
            .discriminator
            .map_or_else(|| "0000".to_owned(), |d| d.to_string());
        let global_name = user.global_name.as_deref().unwrap_or("None");
        let avatar_url = &user.avatar_url().unwrap_or("None".to_owned());
        let bot = &user.bot;
        let public_flags = &user.public_flags.unwrap_or_default();

        user_info.push_str(&format!(
            "ID: {}\nNAME: {}\nDISCRIMINATOR: {}\nDISPLAY NAME: {}\nAVATAR_URL: {}\nBOT: {}\nFLAGS: {:?}\n----------\n",
            user_id, user_name, discriminator, global_name, avatar_url, bot, public_flags
        ));
    }
    let attachment =
        serenity::CreateAttachment::bytes(user_info.to_string(), "users.txt".to_string());
    ctx.send(
        poise::CreateReply::default()
            .content(format!("The cache contains **{}** users", user_count))
            .attachment(attachment),
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

pub fn commands() -> [crate::Command; 6] {
    [
        max_messages(),
        cache_stats(),
        guild_message_cache(),
        cached_users_raw(),
        cached_users(),
        cache_stats(),
    ]
}
