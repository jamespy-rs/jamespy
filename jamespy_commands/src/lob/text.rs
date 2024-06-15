use std::time::Duration;

use jamespy_data::lob::*;

use crate::{Context, Error, InvocationData};
use poise::serenity_prelude::{self as serenity, UserId};

/// i lob
#[poise::command(
    slash_command,
    prefix_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel",
    category = "Utility",
    manual_cooldowns
)]
pub async fn lob(ctx: Context<'_>) -> Result<(), Error> {
    lob_cooldown(ctx).await?;
    let option = get_random_lob();
    if let Some(lob) = option {
        ctx.say(lob).await?;
    }

    Ok(())
}

/// manual cooldown function for lob command.
async fn lob_cooldown(ctx: Context<'_>) -> Result<(), Error> {
    let duration = {
        let mut cooldown_tracker = ctx.command().cooldowns.lock().unwrap();
        let mut cooldown_durations = poise::CooldownConfig::default();

        let osu_game_allowed_users = [
            UserId::from(101090238067113984), // Phil
            UserId::from(158567567487795200), // me
        ];

        // osugame
        if ctx.guild_id() == Some(98226572468690944.into()) {
            cooldown_durations.channel = Some(Duration::from_secs(15));

            // Cooldowns do not apply to these people.
            if osu_game_allowed_users.contains(&ctx.author().id) {
                return Ok(());
            }
        }

        if let Some(remaining) =
            cooldown_tracker.remaining_cooldown(ctx.cooldown_context(), &cooldown_durations)
        {
            Some(remaining)
        } else {
            cooldown_tracker.start_cooldown(ctx.cooldown_context());
            None
        }
    };

    // This will be used in the error handling to determine the response.
    // the error we return does not matter because it will not be used.
    if let Some(duration) = duration {
        // handle error differently down the line.
        ctx.set_invocation_data(InvocationData {
            cooldown_remaining: Some(duration),
        })
        .await;

        return Err("".into());
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
    check = "trontin",
    hide_in_help
)]
pub async fn reload_lob(ctx: Context<'_>) -> Result<(), Error> {
    let (old_count, new_count) = update_lob()?;
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
    check = "trontin",
    hide_in_help
)]
pub async fn no_lob(ctx: Context<'_>) -> Result<(), Error> {
    unload_lob()?;
    ctx.say("Unloaded lob!".to_string()).await?;
    Ok(())
}

use small_fixed_array::FixedString;

#[poise::command(
    rename = "add-lob",
    aliases("addlob", "add_lob"),
    prefix_command,
    category = "Utility",
    hide_in_help,
    check = "trontin"
)]
pub async fn new_lob(
    ctx: Context<'_>,
    #[description = "new lob"]
    #[rest]
    item: FixedString<u16>,
) -> Result<(), Error> {
    let lines = item.lines();
    let count = item.lines().count();
    let msg = if count > 1 {
        // terrible code.
        let lobs: String = lines
            .map(|line| format!("`{line}`"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("Added {count} lobs:\n{lobs}")
    } else {
        format!("Added `{item}` to loblist!\n")
    };

    ctx.send(poise::CreateReply::default().content(msg)).await?;

    add_lob(&item)?;

    Ok(())
}

#[poise::command(
    rename = "remove-lob",
    aliases("removelob", "remove_lob"),
    prefix_command,
    category = "Utility",
    hide_in_help,
    check = "trontin"
)]
pub async fn delete_lob(
    ctx: Context<'_>,
    #[description = "Lob to remove"]
    #[rest]
    target: String,
) -> Result<(), Error> {
    if remove_lob(&target)? {
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
    hide_in_help,
    check = "trontin"
)]
pub async fn send_lobs(ctx: Context<'_>) -> Result<(), Error> {
    let attachment = serenity::CreateAttachment::path("config/lists/loblist.txt").await?;
    ctx.send(poise::CreateReply::default().attachment(attachment))
        .await?;
    Ok(())
}

#[must_use]
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
