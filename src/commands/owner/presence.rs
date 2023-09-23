use poise::serenity_prelude::{all::ActivityType, gateway::ActivityData};

use crate::{Context, Error};

#[poise::command(
    rename = "status",
    prefix_command,
    category = "Management",
    owners_only,
    track_edits,
    hide_in_help
)]
pub async fn status(
    ctx: Context<'_>,
    #[description = "What to say"] status_type: String,
) -> Result<(), Error> {
    let new_status = match status_type.to_lowercase().as_str() {
        "invisible" => {
            ctx.serenity_context().invisible();
            "Invisible"
        }
        "idle" => {
            ctx.serenity_context().idle();
            "Idle"
        }
        "online" => {
            ctx.serenity_context().online();
            "Online"
        }
        "dnd" => {
            ctx.serenity_context().dnd();
            "Do Not Disturb"
        }
        _ => {
            ctx.say("Invalid status!").await?;
            return Ok(());
        }
    };

    ctx.say(format!(
        "Updating status to: **{}**. (this could take a moment)",
        new_status
    ))
    .await?;

    Ok(())
}

#[poise::command(
    rename = "reset-presence",
    prefix_command,
    category = "Management",
    owners_only,
    hide_in_help
)]
pub async fn reset_presence(ctx: Context<'_>) -> Result<(), Error> {
    ctx.serenity_context().reset_presence();
    ctx.say("Resetting the current presence...").await?;

    Ok(())
}

#[poise::command(
    rename = "set-activity",
    prefix_command,
    category = "Management",
    owners_only,
    track_edits,
    hide_in_help
)]
pub async fn set_activity(
    ctx: Context<'_>,
    #[description = "The activity name"] name: String,
    #[description = "The activity type"] activity_type: String,
    #[description = "Custom status (optional)"] custom_status: Option<String>,
) -> Result<(), Error> {
    let activity_type_enum = match activity_type.to_lowercase().as_str() {
        "playing" => ActivityType::Playing,
        "streaming" => ActivityType::Streaming,
        "listening" => ActivityType::Listening,
        "watching" => ActivityType::Watching,
        "custom" => ActivityType::Custom,
        "competing" => ActivityType::Competing,
        _ => ActivityType::Playing,
    };

    let activity_data = ActivityData {
        name,
        kind: activity_type_enum,
        state: custom_status,
        url: None,
    };

    ctx.serenity_context().set_activity(Some(activity_data));

    Ok(())
}
