use jamespy_utils::lob::*;

use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// i lob
#[poise::command(
    slash_command,
    prefix_command,
    category = "Utility",
    channel_cooldown = "5"
)]
pub async fn lob(ctx: Context<'_>) -> Result<(), Error> {
    let option = get_random_lob();
    if let Some(lob) = option {
        ctx.say(lob).await?;
    }

    Ok(())
}

/// reload lob
#[poise::command(
    rename = "reload-lob",
    aliases(
        "reloadlob",
        "reload_lob",
        "update-lob",
        "updatelob",
        "update_lob",
        "reload-lobs"
    ),
    prefix_command,
    category = "Utility",
    global_cooldown = "5",
    check = "trontin",
    hide_in_help
)]
pub async fn reload_lob(ctx: Context<'_>) -> Result<(), Error> {
    let (old_count, new_count) = update_lob().await?;
    ctx.say(format!(
        "Reloaded lobs.\nOld Count: {old_count}\nNew Count: {new_count}"
    ))
    .await?;
    Ok(())
}

/// no lob
#[poise::command(
    rename = "unload-lob",
    aliases("unloadlob", "unload_lob"),
    prefix_command,
    category = "Utility",
    global_cooldown = "5",
    check = "trontin",
    hide_in_help
)]
pub async fn no_lob(ctx: Context<'_>) -> Result<(), Error> {
    unload_lob().await?;
    ctx.say("Unloaded lob!".to_string()).await?;
    Ok(())
}

#[poise::command(
    rename = "add-lob",
    aliases("addlob", "add_lob"),
    prefix_command,
    category = "Utility",
    global_cooldown = "5",
    hide_in_help,
    check = "trontin"
)]
pub async fn new_lob(
    ctx: Context<'_>,
    #[description = "new lob"]
    #[rest]
    item: String,
) -> Result<(), Error> {
    add_lob(&item).await?;
    ctx.send(poise::CreateReply::default().content(format!(
        "Added `{item}` to loblist!\nChanges will not be applied until bot restart or until \
         reload-lob is called!"
    )))
    .await?;
    Ok(())
}

#[poise::command(
    rename = "remove-lob",
    aliases("removelob", "remove_lob"),
    prefix_command,
    category = "Utility",
    global_cooldown = "5",
    hide_in_help,
    check = "trontin"
)]
pub async fn delete_lob(
    ctx: Context<'_>,
    #[description = "Lob to remove"]
    #[rest]
    target: String,
) -> Result<(), Error> {
    if remove_lob(&target).await? {
        ctx.send(poise::CreateReply::default().content(format!(
            "Removed `{target}` from loblist!\nChanges will not be applied until bot restart or \
             until reload-lob is called!"
        )))
        .await?;
    } else {
        ctx.send(
            poise::CreateReply::default()
                .content(format!("`{target}` was not found in the loblist.")),
        )
        .await?;
    }
    Ok(())
}

#[poise::command(
    rename = "total-lobs",
    aliases(
        "totallobs",
        "total_lobs",
        "totallob",
        "total-lob",
        "total_lob",
        "count-lobs"
    ),
    prefix_command,
    category = "Utility",
    user_cooldown = "5",
    hide_in_help,
    check = "trontin"
)]
pub async fn total_lobs(ctx: Context<'_>) -> Result<(), Error> {
    let count = count_lob()?;
    ctx.send(
        poise::CreateReply::default().content(format!("Currently, `{count}` lobs are stored.")),
    )
    .await?;
    Ok(())
}

#[poise::command(
    rename = "send-lobs",
    aliases("sendlobs", "send_lobs", "upload-lobs", "uploadlobs", "upload_lobs"),
    prefix_command,
    category = "Utility",
    user_cooldown = "5",
    hide_in_help,
    check = "trontin"
)]
pub async fn send_lobs(ctx: Context<'_>) -> Result<(), Error> {
    let attachment = serenity::CreateAttachment::path("config/lists/loblist.txt").await?;
    ctx.send(poise::CreateReply::default().attachment(attachment))
        .await?;
    Ok(())
}

pub fn commands() -> [crate::Command; 7] {
    [
        lob(),
        reload_lob(),
        no_lob(),
        new_lob(),
        delete_lob(),
        total_lobs(),
        send_lobs(),
    ]
}
