use crate::Data;
use poise::serenity_prelude::{
    self as serenity, AutoArchiveDuration, ChannelId, ChannelType, ForumLayoutType, GuildId,
    PermissionOverwriteType, SortOrder, Message
};



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
                channel_name = thread.name.clone().to_string();
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

pub async fn download_attachments(message: Message, data: &Data) -> Result<(), std::io::Error> {
    use std::io::Write;

    let castle_conf = {
        let data = data.jamespy_config.read().unwrap();

        data.castle_conf.clone()
    };

    if let Some(castle) = &castle_conf {
        if castle.base.as_ref().unwrap().setup_complete
            && castle.media.as_ref().unwrap().media_stashing_post
        {
            let folder_location = "config/attachments";
            for (index, attachment) in message.attachments.clone().into_iter().enumerate() {
                if let Some(single_limit) = castle.media.as_ref().unwrap().single_limit {
                    if (attachment.size / 1_000_000) as u64 > single_limit {
                        println!(
                            "Cannot download attachment '{}' as it exceeds the single file limit! \
                             ({}MB/{}MB)",
                            attachment.filename,
                            attachment.size / 1_000_000,
                            single_limit
                        );
                        return Ok(());
                    }
                }

                println!(
                    "Downloading file: {} ({}kb)",
                    &attachment.filename,
                    &attachment.size / 1000
                );
                let attach = attachment.download().await;

                let guild_folder = if let Some(guild_id) = &message.guild_id {
                    guild_id.get()
                } else {
                    0
                };

                let path = format!("{folder_location}/{guild_folder}",);
                std::fs::DirBuilder::new().recursive(true).create(path)?;

                let file_loc = format!(
                    "{}/{}/{}_{}_{}",
                    folder_location, guild_folder, message.id, index, attachment.filename
                );
                let mut file = match std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(file_loc)
                {
                    Ok(file) => file,
                    Err(err) => {
                        eprintln!("Error opening or creating file: {err}");
                        return Err(err);
                    }
                };

                file.write_all(&attach.unwrap())?;
            }
        };
    };

    Ok(())
}
