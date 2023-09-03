use poise::serenity_prelude as serenity;
use poise::serenity_prelude::ChannelId;
use ::serenity::{gateway::ActivityData, all::ActivityType};

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

#[poise::command(rename = "online-status", prefix_command, category = "Management", owners_only, hide_in_help)]
pub async fn online_status(
    ctx: Context<'_>,
    #[description = "What to say"] status_type: String,
) -> Result<(), Error> {
    let new_status = match status_type.to_lowercase().as_str() {
        "invisible" => {
            ctx.serenity_context().invisible();
            "Invisible"
        }
        "idle" => {
            ctx.serenity_context().idle();
            "Idle"
        }
        "online" => {
            ctx.serenity_context().online();
            "Online"
        }
        "dnd" => {
            ctx.serenity_context().dnd();
            "Do Not Disturb"
        }
        _ => {
            ctx.say("Invalid status!").await?;
            return Ok(());
        }
    };

    ctx.say(format!("Updating status to: **{}**. (this could take a moment)", new_status)).await?;

    Ok(())
}



#[poise::command(rename = "reset-presence", prefix_command, category = "Management", owners_only, hide_in_help)]
pub async fn reset_presence(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.serenity_context().reset_presence();
    ctx.say("Resetting the current presence...").await?;

    Ok(())
}

#[poise::command(rename = "set-activity", prefix_command, category = "Management", owners_only, hide_in_help)]
pub async fn set_activity(
    ctx: Context<'_>,
    #[description = "The activity name"] name: String,
    #[description = "The activity type"] activity_type: String,
    #[description = "Custom status (optional)"] custom_status: Option<String>,
) -> Result<(), Error> {
    let activity_type_enum = match activity_type.to_lowercase().as_str() {
        "playing" => ActivityType::Playing,
        "streaming" => ActivityType::Streaming,
        "listening" => ActivityType::Listening,
        "watching" => ActivityType::Watching,
        "custom" => ActivityType::Custom,
        "competing" => ActivityType::Competing,
        _ => ActivityType::Playing,
    };

    let activity_data = ActivityData {
        name,
        kind: activity_type_enum,
        state: custom_status,
        url: None,
    };

    ctx.serenity_context().set_activity(Some(activity_data));

    Ok(())
}


#[poise::command(rename = "cache-status", prefix_command, category = "Management", owners_only, hide_in_help)]
pub async fn cache_status(
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

/// prints all the cached users!
#[poise::command(rename = "cached-users-raw", prefix_command, category = "Debug", owners_only, hide_in_help)]
pub async fn cached_users_raw(ctx: Context<'_>) -> Result<(), Error> {
    let users = ctx.serenity_context().cache.users();
    let user_count = ctx.serenity_context().cache.user_count();
    ctx.send(poise::CreateReply::default().content(format!("The cache contains **{}** users", user_count)).attachment(serenity::CreateAttachment::bytes(format!("{:?}", users), format!("raw_users.txt")))).await?;
    Ok(())
}

#[poise::command(rename = "cached-users", prefix_command, category = "Debug", owners_only, hide_in_help)]
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
        let accent_colour = &user.accent_colour.unwrap_or_default();
        let public_flags = &user.public_flags.unwrap_or_default();

        user_info.push_str(&format!(
            "ID: {}\nNAME: {}\nDISCRIMINATOR: {}\nDISPLAY NAME: {}\nAVATAR_URL: {}\nBANNER_URL: {}\nBOT: {}\nACCENT_COLOUR: {:?}\nFLAGS: {:?}\n----------\n",
            user_id, user_name, discriminator, global_name, avatar_url, banner_url, bot, accent_colour, public_flags
        ));
    }
    let attachment = serenity::CreateAttachment::bytes(format!("{}", user_info), format!("users.txt"));
    ctx.send(poise::CreateReply::default().content(format!("The cache contains **{}** users", user_count)).attachment(attachment)).await?;

    Ok(())
}

