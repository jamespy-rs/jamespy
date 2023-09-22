use crate::utils::lob::*;

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
        "Reloaded lobs.\nOld Count: {}\nNew Count: {}",
        old_count, new_count
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
    ctx.say(format!("Unloaded lob!")).await?;
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
    ctx.send(poise::CreateReply::default().content(format!("Added `{}` to loblist!\nChanges will not be applied until bot restart or until reload-lob is called!", item))).await?;
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
    remove_lob(&target).await?;
    ctx.send(poise::CreateReply::default().content(format!("Removed `{}` from loblist!\nChanges will not be applied until bot restart or until reload-lob is called!", target))).await?;
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
        poise::CreateReply::default().content(format!("Currently, `{}` lobs are stored.", count)),
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
    let attachment = serenity::CreateAttachment::path("loblist.txt").await?;
    ctx.send(poise::CreateReply::default().attachment(attachment))
        .await?;
    Ok(())
}
