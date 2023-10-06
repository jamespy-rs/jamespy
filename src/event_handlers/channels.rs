use crate::utils::misc::{auto_archive_duration_to_string, channel_type_to_string, get_guild_name};
use crate::utils::permissions::get_permission_changes;
use crate::Error;
use poise::serenity_prelude::{
    self as serenity, Channel, ForumEmoji, GuildChannel, PartialGuildChannel,
};

pub async fn channel_create(ctx: &serenity::Context, channel: GuildChannel) -> Result<(), Error> {
    // Show channel kind.
    let guild_name = channel
        .guild_id
        .name(ctx)
        .unwrap_or("Unknown Guild".to_string());
    println!(
        "\x1B[34m[{}] #{} was created!\x1B[0m",
        guild_name, channel.name
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
            match (old.topic, new.topic) {
                (Some(old_value), Some(new_value)) if old_value != new_value => {
                    diff.push_str(&format!("Topic: {} -> {}\n", old_value, new_value));
                }
                (None, Some(new_value)) => {
                    diff.push_str(&format!("Topic: None -> {}\n", new_value));
                }
                (Some(old_value), None) => {
                    diff.push_str(&format!("Topic: {} -> None\n", old_value));
                }
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

            // available_tags
            // channelflags
            // default_sort_order
            // default_forum_layout
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
    // show channel kind.
    let guild_name = channel
        .guild_id
        .name(ctx)
        .unwrap_or("Unknown Guild".to_string());
    println!(
        "\x1B[34m[{}] #{} was deleted!\x1B[0m",
        guild_name, channel.name
    );
    Ok(())
}

pub async fn thread_create(ctx: &serenity::Context, thread: GuildChannel) -> Result<(), Error> {
    // show thread kind?
    let guild_id = thread.guild_id;
    let guild_name = get_guild_name(ctx, guild_id);

    let parent_channel_name = if let Some(parent_id) = thread.parent_id {
        parent_id.name(ctx).await?
    } else {
        "Unknown Channel".to_string()
    };

    println!(
        "\x1B[94m[{}] Thread #{} was created in #{}!\x1B[0m",
        guild_name, thread.name, parent_channel_name
    );
    Ok(())
}

pub async fn thread_delete(
    ctx: &serenity::Context,
    thread: PartialGuildChannel,
    full_thread_data: Option<GuildChannel>,
) -> Result<(), Error> {
    // show thread kind?
    let guild_id = thread.guild_id;
    let mut channel_name = String::new();
    let mut parent_channel_name: String = String::new();

    if let Some(full_thread) = full_thread_data {
        channel_name = full_thread.name;

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
            "\x1B[94m[{}] Thread #{} was deleted from #{}!\x1B[0m",
            guild_name, channel_name, parent_channel_name
        )
    }
    Ok(())
}
