use crate::utils::lob::*;

use crate::{Context, Error};

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
    aliases("reloadlob", "reload_lob", "update-lob", "updatelob", "update_lob"),
    prefix_command,
    category = "Utility",
    global_cooldown = "5",
    check = "trontin",
    hide_in_help
)]
pub async fn reload_lob(ctx: Context<'_>) -> Result<(), Error> {
    let (old_count, new_count) = update_lob().await?;
    ctx.say(format!("Reloaded lobs.\nOld Count: {}\nNew Count: {}", old_count, new_count)).await?;
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
    slash_command,
    prefix_command,
    category = "Utility",
    channel_cooldown = "5",
    check = "trontin"
)]
pub async fn new_lob(
    ctx: Context<'_>,
    #[description = "new lob"] item: String,
) -> Result<(), Error> {
    add_lob(&item).await?;
    ctx.send(poise::CreateReply::default().content(format!("Added `{}` to loblist!\nChanges will not be applied until bot restart or until reload-lob is called!", item))).await?;
    Ok(())
}

#[poise::command(
    rename = "remove-lob",
    aliases("removelob", "remove_lob"),
    slash_command,
    prefix_command,
    category = "Utility",
    channel_cooldown = "5",
    check = "trontin"
)]
pub async fn delete_lob(
    ctx: Context<'_>,
    #[description = "Lob to remove"] target: String,
) -> Result<(), Error> {
    remove_lob(&target).await?;
    ctx.send(poise::CreateReply::default().content(format!("Removed `{}` from loblist!\nChanges will not be applied until bot restart or until reload-lob is called!", target))).await?;
    Ok(())
}


// A check for Trash, so he can refresh the loblist. Includes me because, well I'm me.
// Also includes a few gg/osu mods because well why not!
async fn trontin(ctx: Context<'_>) -> Result<bool, Error> {
    let allowed_users = vec![158567567487795200, 288054604548276235, 291089948709486593, 718513035555242086]; // me, trontin, ruben, cv
    let user_id = ctx.author().id.get();
    let trontin = allowed_users.contains(&user_id);

    Ok(trontin)
}
