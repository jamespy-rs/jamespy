use crate::Data;

use std::fmt::Write;
use std::sync::Arc;

use poise::serenity_prelude::{
    self as serenity, AutoArchiveDuration, ChannelId, ChannelType, Context, ForumLayoutType,
    GuildId, PermissionOverwrite, PermissionOverwriteType, Permissions, SortOrder,
};

// Helper function for getting the guild name override or guild name even if None.
pub fn get_guild_name_override(
    ctx: &serenity::Context,
    data: &Arc<Data>,
    guild_id: Option<GuildId>,
) -> String {
    if guild_id.is_none() {
        return get_guild_name(ctx, guild_id);
    }

    if let Some(overrides) = &data.config.read().events.guild_name_override {
        overrides
            .get(&guild_id.unwrap())
            .unwrap_or(&get_guild_name(ctx, guild_id))
            .to_string()
    } else {
        get_guild_name(ctx, guild_id)
    }
}

// Helper function for getting the guild name even if ID is a None variant.
#[must_use]
pub fn get_guild_name(ctx: &serenity::Context, guild_id: Option<GuildId>) -> String {
    if let Some(id) = guild_id {
        match id.name(&ctx.cache) {
            Some(name) => name,
            None => "Unknown".to_owned(),
        }
    } else {
        "None".to_string()
    }
}

// Helper function for getting the channel name.
pub async fn get_channel_name(
    ctx: &serenity::Context,
    guild_id: Option<GuildId>,
    channel_id: ChannelId,
) -> String {

    // Right now all threads make a HTTP request even if they are cached because
    // they go through the channel logic first, and fail to get the channel.
    // This is not efficient at all and eats requests.

    // to get around this you should probably check if the channel is cached.

    // if not cached, check if its a thread, and then check over http.

    match channel_id.name(ctx).await {
        Ok(name) => name,
        Err(_) => get_channel_name_thread(ctx, guild_id, channel_id).await,
    }
}

// Helper function for getting the channel name if its a thread.
async fn get_channel_name_thread(
    ctx: &serenity::Context,
    guild_id: Option<GuildId>,
    channel_id: ChannelId,
) -> String {
    if guild_id.is_none() {
        return "Unknown Channel".to_string();
    }

    let id = guild_id.unwrap();
    let Some(guild_cache) = ctx.cache.guild(id) else {
        return "Unknown Channel".to_string();
    };

    let threads = &guild_cache.threads;

    for thread in threads {
        if thread.id == channel_id.get() {
            return thread.name.to_string();
        }
    }

    "Unknown Channel".to_string()
}

#[must_use]
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
        _ => format!("Unknown({}", channel_type.0),
    }
}

#[must_use]
pub fn overwrite_to_string(overwrite: PermissionOverwriteType) -> String {
    match overwrite {
        PermissionOverwriteType::Member(_) => String::from("Member"),
        PermissionOverwriteType::Role(_) => String::from("Role"),
        _ => String::from("?"),
    }
}

#[must_use]
pub fn auto_archive_duration_to_string(duration: AutoArchiveDuration) -> String {
    match duration {
        AutoArchiveDuration::None => String::from("None"),
        AutoArchiveDuration::OneHour => String::from("1 hour"),
        AutoArchiveDuration::OneDay => String::from("1 day"),
        AutoArchiveDuration::ThreeDays => String::from("3 days"),
        AutoArchiveDuration::OneWeek => String::from("1 week"),
        _ => format!("Unknown({}", duration.0),
    }
}

#[must_use]
pub fn forum_layout_to_string(layout_type: ForumLayoutType) -> String {
    match layout_type {
        ForumLayoutType::NotSet => String::from("Not Set"),
        ForumLayoutType::ListView => String::from("List View"),
        ForumLayoutType::GalleryView => String::from("Gallery View"),
        _ => format!("Unknown({}", layout_type.0),
    }
}

#[must_use]
pub fn sort_order_to_string(sort_order: SortOrder) -> String {
    match sort_order {
        SortOrder::LatestActivity => String::from("Latest Activity"),
        SortOrder::CreationDate => String::from("Creation Date"),
        _ => format!("Unknown({}", sort_order.0),
    }
}

pub async fn get_permission_changes(
    ctx: &Context,
    guild_id: GuildId,
    old_allow: Permissions,
    new_allow: Permissions,
    old_deny: Permissions,
    new_deny: Permissions,
    kind: PermissionOverwriteType,
) -> String {
    let name = match kind {
        PermissionOverwriteType::Member(user_id) => match user_id.to_user(ctx).await {
            Ok(user) => user.tag(),
            Err(_) => String::from("Unknown User"),
        },
        PermissionOverwriteType::Role(role_id) => ctx
            .cache
            .guild(guild_id)
            .unwrap()
            .roles
            .get(&role_id)
            .map_or_else(|| "Unknown Role".to_string(), |role| role.name.to_string()),
        _ => String::from("Unknown"),
    };

    let mut changes_str = String::new();
    let kind_string = overwrite_to_string(kind);
    if old_allow != new_allow || old_deny != new_deny {
        writeln!(
            changes_str,
            "Permission override for {name} ({kind_string}) changed!"
        )
        .unwrap();

        let allow_changes_detail = get_permission_changes_detail(old_allow, new_allow, true);
        let deny_changes_detail = get_permission_changes_detail(old_deny, new_deny, false);

        if !allow_changes_detail.is_empty() {
            writeln!(changes_str, "allow:").unwrap();
            write!(changes_str, "{}", &allow_changes_detail).unwrap();
        }

        if !deny_changes_detail.is_empty() {
            writeln!(changes_str, "deny:").unwrap();
            write!(changes_str, "{}", &deny_changes_detail).unwrap();
        }
    }

    changes_str
}

#[must_use]
pub fn get_permission_changes_detail(old: Permissions, new: Permissions, allow: bool) -> String {
    let mut changes_str = String::new();
    let added_color = if allow { "\x1B[92m" } else { "\x1B[31m" };
    let removed_color = if allow { "\x1B[31m" } else { "\x1B[92m" };

    let added_perms: Vec<String> = {
        let mut added = Vec::new();
        for permission in Permissions::all().iter() {
            let permission_name = permission.to_string();
            if new.contains(permission) && !old.contains(permission) {
                added.push(permission_name);
            }
        }
        added
    };

    let removed_perms: Vec<String> = {
        let mut removed = Vec::new();
        for permission in Permissions::all().iter() {
            let permission_name = permission.to_string();
            if !new.contains(permission) && old.contains(permission) {
                removed.push(permission_name);
            }
        }
        removed
    };

    if !added_perms.is_empty() {
        for perm in &added_perms {
            writeln!(changes_str, "{added_color}+ {perm}\x1B[0m").unwrap();
        }
    }

    if !removed_perms.is_empty() {
        for perm in &removed_perms {
            writeln!(changes_str, "{removed_color}- {perm}\x1B[0m").unwrap();
        }
    }

    changes_str
}

pub async fn overwrite_removal(
    ctx: &Context,
    guild_id: GuildId,
    overwrite: &PermissionOverwrite,
) -> String {
    let name = match overwrite.kind {
        PermissionOverwriteType::Member(user_id) => match user_id.to_user(ctx).await {
            Ok(user) => user.tag(),
            Err(_) => String::from("Unknown User"),
        },
        PermissionOverwriteType::Role(role_id) => ctx
            .cache
            .guild(guild_id)
            .unwrap()
            .roles
            .get(&role_id)
            .map_or_else(|| "Unknown Role".to_string(), |role| role.name.to_string()),
        _ => String::from("Unknown"),
    };

    let mut changes_str = String::new();
    let kind_string = overwrite_to_string(overwrite.kind);
    writeln!(
        changes_str,
        "Permission override for {name} ({kind_string}) was removed!"
    )
    .unwrap();

    let added_color = "\x1B[92m";
    let removed_color = "\x1B[31m";

    let mut allowed_str = String::new();
    let mut denied_str = String::new();
    for allowed in overwrite.allow {
        writeln!(allowed_str, "{added_color}+ {allowed}\x1B[0m").unwrap();
    }

    for denied in overwrite.deny {
        writeln!(denied_str, "{removed_color}+ {denied}\x1B[0m").unwrap();
    }

    if !allowed_str.is_empty() {
        changes_str.push_str("allowed:\n");
        changes_str.push_str(&allowed_str);
    }

    if !denied_str.is_empty() {
        changes_str.push_str("denied:\n");
        changes_str.push_str(&denied_str);
    }

    changes_str
}
