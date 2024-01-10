use crate::Data;
use poise::serenity_prelude::{Attachment, Message};

const BYTES_TO_MB: u32 = 1_000_000;

pub async fn download_attachments(message: Message, data: &Data) -> Result<(), std::io::Error> {
    use std::io::Write;

    // TODO: hit spy guild.
    let (attachments_set, _spy_guild) = {
        let data = data.config.read().unwrap();

        (data.attachment_store, data.spy_guild.clone())
    };

    // cannot download if disabled.
    if attachments_set.is_none() {
        return Ok(());
    }
    let attachments_set = attachments_set.unwrap();

    if !attachments_set.enabled {
        return Ok(());
    }

    // TODO: allow configuration.
    let folder_location = "config/attachments";

    for (index, attachment) in message.attachments.into_iter().enumerate() {
        if !is_below_single_limit(&attachment, attachments_set) {
            continue;
        }

        // Potentially hard limit here?

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

    Ok(())
}

fn is_below_single_limit(
    attachment: &Attachment,
    attachment_settings: jamespy_config::Attachments,
) -> bool {
    if let Some(limit) = attachment_settings.single_limit {
        let size = attachment.size / BYTES_TO_MB;

        if size > limit as u32 {
            println!(
                "Cannot download attachment '{}' as it exceeds the single file limit! \
                 ({}MB/{:.2}MB)",
                attachment.filename, size, limit
            );
            false // above limit.
        } else {
            true // below.
        }
    } else {
        true // no limit.
    }
}
