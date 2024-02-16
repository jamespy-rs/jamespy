use crate::{helper::get_guild_name, Data, Error};
use ::serenity::http::CacheHttp;
use poise::serenity_prelude::{
    self as serenity, Attachment, CreateAttachment, CreateButton,
    CreateInteractionResponseFollowup, Message,
};
use small_fixed_array::FixedString;
use std::fmt::Write;

const BYTES_TO_MB: u32 = 1_000_000;

pub async fn download_attachments(
    ctx: &serenity::Context,
    message: Message,
    data: &Data,
) -> Result<(), Error> {
    use std::io::Write;

    let (attachments_set, spy_guild) = {
        let data = data.config.read();

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

    if let Some(ref spy_guild) = spy_guild {
        if matches!(&spy_guild.attachment_hook, Some(hook) if hook.channel_id == Some(message.channel_id))
        {
            return Ok(()); // do not download files from this channel.
        }
    }

    // TODO: allow configuration.
    let folder_location = "config/attachments";

    let mut files: Vec<File> = vec![];

    for (index, attachment) in message.attachments.clone().into_iter().enumerate() {
        if !is_below_single_limit(&attachment, attachments_set) {
            continue;
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
            .truncate(true)
            .write(true)
            .open(&file_loc)
        {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Error opening or creating file: {err}");
                return Err(Box::new(err));
            }
        };

        file.write_all(&attach.unwrap())?;

        if let Some(spy) = &spy_guild {
            if let Some(hook) = &spy.attachment_hook {
                if !hook.enabled {
                    continue;
                }
                files.push(File {
                    file_name: attachment.filename,
                    location: file_loc,
                });
            }
        }
    }

    if !files.is_empty() {
        announce_deleted_spy(ctx, files, message, spy_guild).await?;
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

async fn announce_deleted_spy(
    ctx: &serenity::Context,
    files: Vec<File>,
    message: Message,
    spy: Option<jamespy_config::SpyGuild>,
) -> Result<(), Error> {
    if spy.is_none() {
        return Ok(());
    }
    let spy = spy.unwrap();

    if spy.attachment_hook.is_none() {
        return Ok(());
    }
    let hook = spy.attachment_hook.unwrap();

    // already checked if enabled by the time this is executed.
    if hook.channel_id.is_none() {
        return Ok(());
    }

    let ctx_id = message.id;
    let mut description = String::new();
    let mut buttons = vec![];
    let mut button_ids: Vec<(String, usize)> = vec![];

    for (index, file) in files.clone().into_iter().enumerate() {
        writeln!(description, "{}: {}", index + 1, file.file_name)?;
        let custom_id = format!("{}_{}", ctx_id, index + 1);

        buttons.push(CreateButton::new(custom_id.clone()).label((index + 1).to_string()));
        button_ids.push((custom_id, index));
    }

    // Split into multiple chunks as the limit per row is 5.
    let mut action_rows = Vec::new();
    for chunk in buttons.chunks(5) {
        action_rows.push(serenity::CreateActionRow::Buttons(chunk.to_vec()));
    }

    let description_fmt = if message.content.is_empty() {
        format!("**Attachments**:\n{description}")
    } else {
        format!(
            "**Message**:\n{}\n**Attachments**:\n{}",
            message.content, description
        )
    };

    let message = serenity::CreateMessage::default()
        .embed(
            serenity::CreateEmbed::default()
                .author(
                    serenity::CreateEmbedAuthor::new(format!(
                        "{}'s deleted message with attachments",
                        message.author.tag()
                    ))
                    .icon_url(message.author.face()),
                )
                .title(format!(
                    "Message deleted in <#{}> in {}",
                    message.channel_id,
                    get_guild_name(ctx, message.guild_id)
                ))
                .description(description_fmt),
        )
        .components(action_rows);

    let mut msg = hook.channel_id.unwrap().send_message(ctx, message).await?;

    while let Some(press) = serenity::ComponentInteractionCollector::new(ctx.shard.clone())
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(3600)) // 1 hour.
        .await
    {
        for id in &button_ids {
            if *id.0 == press.data.custom_id {
                let file = files.get(id.1).unwrap();

                press.defer(ctx.http()).await?;

                let bytes = match std::fs::read(&file.location) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        println!("{err:?}");
                        return Ok(());
                    }
                };

                press
                    .create_followup(
                        ctx.http(),
                        CreateInteractionResponseFollowup::new()
                            .files(vec![CreateAttachment::bytes(bytes, file.file_name.clone())]),
                    )
                    .await?;
            }
        }
    }

    msg.edit(ctx, serenity::EditMessage::new().components(vec![]))
        .await?;

    Ok(())
}

#[derive(Clone, Debug)]
pub struct File {
    pub file_name: FixedString,
    pub location: String,
}
