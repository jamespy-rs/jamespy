use crate::utils::misc::{
    auto_archive_duration_to_string, channel_type_to_string, forum_layout_to_string,
    get_guild_name, sort_order_to_string,
};
use crate::utils::permissions::get_permission_changes;
use crate::Error;
use poise::serenity_prelude::{
    self as serenity, Channel, ChannelFlags, ForumEmoji, GuildChannel, PartialGuildChannel,
};
use serenity::all::ChannelType;

pub async fn channel_create(ctx: &serenity::Context, channel: GuildChannel) -> Result<(), Error> {
    // Show channel kind.
    let guild_name = channel
        .guild_id
        .name(ctx)
        .unwrap_or("Unknown Guild".to_string());
    let kind = channel_type_to_string(channel.kind);
    println!(
        "\x1B[34m[{}] #{} ({}) was created!\x1B[0m",
        guild_name, channel.name, kind
    );
    Ok(())
}

pub async fn channel_update(
    ctx: &serenity::Context,
    old: Option<Channel>,
    new: Channel,
) -> Result<(), Error> {
    let mut guild_name = String::new();
    let mut channel_name = String::new();
    let mut kind = String::new();
    let mut diff = String::new();

    if let Some(old) = old.and_then(|o| o.guild()) {
        if let Some(new) = new.guild() {
            guild_name = get_guild_name(ctx, old.guild_id);
            channel_name = new.name.clone();
            kind = channel_type_to_string(new.kind);

            // Differences
            if old.name != new.name {
                diff.push_str(&format!("Name: {} -> {}\n", old.name, new.name))
            }
            if old.nsfw != new.nsfw {
                diff.push_str(&format!("NSFW: {} -> {}\n", old.nsfw, new.nsfw))
            }
            // Check if the channel is in a category.
            if let (Some(old_parent_id), Some(new_parent_id)) = (old.parent_id, new.parent_id) {
                if old_parent_id != new_parent_id {
                    diff.push_str(&format!(
                        "Parent: {} -> {}\n",
                        old_parent_id.name(ctx.clone()).await?,
                        new_parent_id.name(ctx.clone()).await?
                    ));
                }
            } else if old.parent_id.is_none() && new.parent_id.is_some() {
                if let Some(parent_id) = new.parent_id {
                    diff.push_str(&format!(
                        "Parent: None -> {}\n",
                        parent_id.name(ctx.clone()).await?
                    ));
                }
            } else if old.parent_id.is_some() && new.parent_id.is_none() {
                if let Some(parent_id) = old.parent_id {
                    diff.push_str(&format!(
                        "Parent: {} -> None\n",
                        parent_id.name(ctx.clone()).await?
                    ));
                }
            }
            match (old.bitrate, new.bitrate) {
                (Some(old_value), Some(new_value)) if old_value != new_value => {
                    diff.push_str(&format!(
                        "Bitrate: {}kbps -> {}kbps\n",
                        old_value / 1000,
                        new_value / 1000
                    ));
                }
                _ => {}
            }

            if old.permission_overwrites != new.permission_overwrites {
                for old_overwrite in old.permission_overwrites {
                    for new_overwrite in &new.permission_overwrites {
                        if old_overwrite.kind == new_overwrite.kind {
                            let changes_str = get_permission_changes(
                                ctx.clone(),
                                old_overwrite.allow,
                                new_overwrite.allow,
                                old_overwrite.deny,
                                new_overwrite.deny,
                                new_overwrite.kind,
                            )
                            .await;
                            diff.push_str(&changes_str);
                        }
                    }
                }
            }

            // If both the old and new topic are the same, it shouldn't print.
            match (old.topic, new.topic) {
                (Some(old_value), Some(new_value)) if old_value != new_value => {
                    diff.push_str(&format!("Topic: {} -> {}\n", old_value, new_value));
                }
                (None, Some(new_value)) if !new_value.is_empty() => {
                    diff.push_str(&format!("Topic: None -> {}\n", new_value));
                }
                (Some(old_value), None) if !old_value.is_empty() => {
                    diff.push_str(&format!("Topic: {} -> None\n", old_value));
                }
                (None, None) => {}
                _ => {}
            }

            match (old.user_limit, new.user_limit) {
                (Some(old_value), Some(new_value)) if old_value != new_value => {
                    diff.push_str(&format!("User Limit: {} -> {}\n", old_value, new_value));
                }
                (None, Some(new_value)) => {
                    diff.push_str(&format!("User Limit: None -> {}\n", new_value));
                }
                (Some(old_value), None) => {
                    diff.push_str(&format!("User Limit: {} -> None\n", old_value));
                }
                _ => {}
            }

            match (old.rate_limit_per_user, new.rate_limit_per_user) {
                (Some(old_value), Some(new_value)) if old_value != new_value => {
                    diff.push_str(&format!("Slowmode: {}s -> {}s\n", old_value, new_value));
                }
                _ => {}
            }

            match (
                old.default_thread_rate_limit_per_user,
                new.default_thread_rate_limit_per_user,
            ) {
                (Some(old_value), Some(new_value)) if old_value != new_value => {
                    diff.push_str(&format!(
                        "Default Thread Slowmode: {}s -> {}s\n",
                        old_value, new_value
                    ));
                }
                _ => {}
            }

            match (
                old.default_auto_archive_duration,
                new.default_auto_archive_duration,
            ) {
                (Some(old_value), Some(new_value)) if old_value != new_value => {
                    let old_duration = auto_archive_duration_to_string(old_value);
                    let new_duration = auto_archive_duration_to_string(new_value);
                    diff.push_str(&format!(
                        "Default Archive Duration: {} -> {}\n",
                        old_duration, new_duration
                    ));
                }
                _ => {}
            }

            match (old.default_reaction_emoji, new.default_reaction_emoji) {
                (Some(ForumEmoji::Name(old_name)), Some(ForumEmoji::Name(new_name)))
                    if old_name != new_name =>
                {
                    diff.push_str(&format!(
                        "Default Reaction Emoji: {} -> {}\n",
                        old_name, new_name
                    ));
                }
                (None, Some(ForumEmoji::Name(new_name))) => {
                    diff.push_str(&format!("Default Reaction Emoji: None -> {}\n", new_name));
                }
                (Some(ForumEmoji::Name(old_name)), None) => {
                    diff.push_str(&format!("Default Reaction Emoji: {} -> None\n", old_name));
                }
                _ => {}
            }

            if old.flags.contains(ChannelFlags::REQUIRE_TAG)
                != new.flags.contains(ChannelFlags::REQUIRE_TAG)
            {
                match new.flags.contains(ChannelFlags::REQUIRE_TAG) {
                    true => diff.push_str("REQUIRE_TAG was enabled!"),
                    false => diff.push_str("REQUIRE_TAG was disabled!"),
                }
            }

            match (old.default_forum_layout, new.default_forum_layout) {
                (Some(old_value), Some(new_value)) if old_value != new_value => {
                    diff.push_str(&format!(
                        "Default Forum Layout: {} -> {}\n",
                        forum_layout_to_string(old_value),
                        forum_layout_to_string(new_value)
                    ));
                }
                (None, Some(new_value)) => {
                    diff.push_str(&format!(
                        "Default Forum Layout: None -> {}\n",
                        forum_layout_to_string(new_value)
                    ));
                }
                (Some(old_value), None) => {
                    diff.push_str(&format!(
                        "Default Forum Layout: {} -> None\n",
                        forum_layout_to_string(old_value)
                    ));
                }
                (None, None) => {}
                _ => {}
            }

            match (old.default_sort_order, new.default_sort_order) {
                (Some(old_value), Some(new_value)) if old_value != new_value => {
                    diff.push_str(&format!(
                        "Default Forum Layout: {} -> {}\n",
                        sort_order_to_string(old_value),
                        sort_order_to_string(new_value)
                    ));
                }
                (None, Some(new_value)) => {
                    diff.push_str(&format!(
                        "Default Forum Layout: None -> {}\n",
                        sort_order_to_string(new_value)
                    ));
                }
                (Some(old_value), None) => {
                    diff.push_str(&format!(
                        "Default Forum Layout: {} -> None\n",
                        sort_order_to_string(old_value)
                    ));
                }
                (None, None) => {}
                _ => {}
            }
            // Forum tags doesn't implement what i want, I refuse to do it until this is matched.
        }
    }
    diff = diff.trim_end_matches('\n').to_string();
    if !diff.is_empty() {
        println!(
            "\x1B[34m[{}] #{} was updated! ({})\x1B[0m\n{}",
            guild_name, channel_name, kind, diff
        );
    }
    Ok(())
}

pub async fn channel_delete(ctx: &serenity::Context, channel: GuildChannel) -> Result<(), Error> {
    let kind = channel_type_to_string(channel.kind);
    let guild_name = channel
        .guild_id
        .name(ctx)
        .unwrap_or("Unknown Guild".to_string());
    println!(
        "\x1B[34m[{}] #{} ({}) was deleted!\x1B[0m",
        guild_name, channel.name, kind
    );
    Ok(())
}

pub async fn thread_create(ctx: &serenity::Context, thread: GuildChannel) -> Result<(), Error> {
    let guild_id = thread.guild_id;
    let guild_name = get_guild_name(ctx, guild_id);
    let kind = channel_type_to_string(thread.kind);

    let parent_channel_name = if let Some(parent_id) = thread.parent_id {
        parent_id.name(ctx).await?
    } else {
        "Unknown Channel".to_string()
    };

    println!(
        "\x1B[94m[{}] Thread #{} ({}) was created in #{}!\x1B[0m",
        guild_name, thread.name, kind, parent_channel_name
    );
    Ok(())
}

pub async fn thread_update(
    ctx: &serenity::Context,
    old: Option<GuildChannel>,
    new: GuildChannel,
) -> Result<(), Error> {
    let guild_id = new.guild_id;
    let guild_name = get_guild_name(ctx, guild_id);
    let kind = channel_type_to_string(new.kind);
    let mut diff = String::new();

    let parent_channel_name = if let Some(parent_id) = new.parent_id {
        parent_id.name(ctx).await?
    } else {
        "Unknown Channel".to_string()
    };

    if let Some(old) = old {
        if old.name != new.name {
            diff.push_str(&format!("Name: {} -> {}\n", old.name, new.name))
        }

        match (old.rate_limit_per_user, new.rate_limit_per_user) {
            (Some(old_value), Some(new_value)) if old_value != new_value => {
                diff.push_str(&format!("Slowmode: {}s -> {}s\n", old_value, new_value));
            }
            _ => {}
        }

        if old.flags.contains(ChannelFlags::PINNED) != new.flags.contains(ChannelFlags::PINNED) {
            match new.flags.contains(ChannelFlags::PINNED) {
                true => diff.push_str("Pinned: true"),
                false => diff.push_str("Pinned: false"),
            }
        }

        if let (Some(old_metadata), Some(new_metadata)) = (old.thread_metadata, new.thread_metadata)
        {
            if old.kind == ChannelType::PrivateThread {
                match (old_metadata.invitable, new_metadata.invitable) {
                    (true, false) => diff.push_str("Invitable: false\n"),
                    (false, true) => diff.push_str("Invitable: true\n"),
                    _ => {}
                }
            }

            match (old_metadata.archived, new_metadata.archived) {
                (true, false) => diff.push_str("Archived: false\n"),
                (false, true) => diff.push_str("Archived: true\n"),
                _ => {}
            }
            match (old_metadata.locked, new_metadata.locked) {
                (true, false) => diff.push_str("Locked: false\n"),
                (false, true) => diff.push_str("Locked: true\n"),
                _ => {}
            }

            if old_metadata.auto_archive_duration != new_metadata.auto_archive_duration {
                let old_duration =
                    auto_archive_duration_to_string(old_metadata.auto_archive_duration);
                let new_duration =
                    auto_archive_duration_to_string(new_metadata.auto_archive_duration);
                diff.push_str(&format!(
                    "Archive Duration: {} -> {}\n",
                    old_duration, new_duration
                ));
            }
        }
    }

    diff = diff.trim_end_matches('\n').to_string();
    if !diff.is_empty() {
        println!(
            "\x1B[94m[{}] #{} in {} was updated! ({})\x1B[0m\n{}",
            guild_name, new.name, parent_channel_name, kind, diff
        );
    }

    Ok(())
}

pub async fn thread_delete(
    ctx: &serenity::Context,
    thread: PartialGuildChannel,
    full_thread_data: Option<GuildChannel>,
) -> Result<(), Error> {
    let guild_id = thread.guild_id;
    let mut channel_name = String::new();
    let mut parent_channel_name: String = String::new();
    let mut kind = String::new();

    if let Some(full_thread) = full_thread_data {
        channel_name = full_thread.name;
        kind = channel_type_to_string(full_thread.kind);

        if let Some(parent_id) = full_thread.parent_id {
            parent_channel_name = parent_id.name(ctx).await?;
        } else {
            parent_channel_name = "Unknown Channel".to_string();
        }
    }
    let guild_name = get_guild_name(ctx, guild_id);

    if channel_name.is_empty() {
        println!(
            "\x1B[94m[{}] An unknown thread was deleted!\x1B[0m",
            guild_name
        )
    } else {
        println!(
            "\x1B[94m[{}] Thread #{} ({}) was deleted from #{}!\x1B[0m",
            guild_name, channel_name, kind, parent_channel_name
        )
    }
    Ok(())
}
