use poise::serenity_prelude::{
    self as serenity, AutoArchiveDuration, ChannelId, ChannelType, ForumLayoutType, GuildId,
    PermissionOverwriteType, SortOrder,
};
use std::collections::HashSet;

pub fn read_words_from_file(filename: &str) -> HashSet<String> {
    std::fs::read_to_string(filename)
        .expect("Failed to read the file")
        .lines()
        .map(|line| line.trim().to_lowercase())
        .collect()
}

pub async fn get_channel_name(
    ctx: &serenity::Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> String {
    let mut channel_name = channel_id
        .name(ctx)
        .await
        .unwrap_or("Unknown Channel".to_owned());

    if guild_id.get() != 0 && channel_name == "Unknown Channel" {
        let guild_cache = ctx.cache.guild(guild_id).unwrap();
        let threads = &guild_cache.threads;
        for thread in threads {
            if thread.id == channel_id.get() {
                channel_name = thread.name.clone();
                break;
            }
        }
    }

    channel_name
}

pub fn get_guild_name(ctx: &serenity::Context, guild_id: GuildId) -> String {
    if guild_id == 1 {
        "None".to_owned()
    } else {
        match guild_id.name(ctx) {
            Some(name) => name,
            None => "Unknown".to_owned(),
        }
    }
}

pub fn channel_type_to_string(channel_type: ChannelType) -> String {
    match channel_type {
        ChannelType::Text => String::from("Text"),
        ChannelType::Private => String::from("Private"),
        ChannelType::Voice => String::from("Voice"),
        ChannelType::GroupDm => String::from("GroupDm"),
        ChannelType::Category => String::from("Category"),
        ChannelType::News => String::from("News"),
        ChannelType::NewsThread => String::from("NewsThread"),
        ChannelType::PublicThread => String::from("PublicThread"),
        ChannelType::PrivateThread => String::from("PrivateThread"),
        ChannelType::Stage => String::from("Stage"),
        ChannelType::Directory => String::from("Directory"),
        ChannelType::Forum => String::from("Forum"),
        ChannelType::Unknown(u) => format!("Unknown({u})"),
        _ => String::from("?"),
    }
}

pub fn overwrite_to_string(overwrite: PermissionOverwriteType) -> String {
    match overwrite {
        PermissionOverwriteType::Member(_) => String::from("Member"),
        PermissionOverwriteType::Role(_) => String::from("Role"),
        _ => String::from("?"),
    }
}

pub fn auto_archive_duration_to_string(duration: AutoArchiveDuration) -> String {
    match duration {
        AutoArchiveDuration::None => String::from("None"),
        AutoArchiveDuration::OneHour => String::from("1 hour"),
        AutoArchiveDuration::OneDay => String::from("1 day"),
        AutoArchiveDuration::ThreeDays => String::from("3 days"),
        AutoArchiveDuration::OneWeek => String::from("1 week"),
        AutoArchiveDuration::Unknown(u) => format!("Unknown({u})"),
        _ => String::from("?"),
    }
}

pub fn forum_layout_to_string(layout_type: ForumLayoutType) -> String {
    match layout_type {
        ForumLayoutType::NotSet => String::from("Not Set"),
        ForumLayoutType::ListView => String::from("List View"),
        ForumLayoutType::GalleryView => String::from("Gallery View"),
        ForumLayoutType::Unknown(u) => format!("Unknown({u})"),
        _ => String::from("?"),
    }
}

pub fn sort_order_to_string(sort_order: SortOrder) -> String {
    match sort_order {
        SortOrder::LatestActivity => String::from("Latest Activity"),
        SortOrder::CreationDate => String::from("Creation Date"),
        SortOrder::Unknown(u) => format!("Unknown({u})"),
        _ => String::from("?"),
    }
}
