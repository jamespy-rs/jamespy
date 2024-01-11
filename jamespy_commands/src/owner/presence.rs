use poise::serenity_prelude::{ActivityData, ActivityType};

use crate::{Context, Error};

#[derive(Debug, poise::ChoiceParameter)]
pub enum OnlineStatus {
    Online,
    Idle,
    #[name = "Do Not Disturb"]
    #[name = "dnd"]
    DoNotDisturb,
    Invisible,
}

#[poise::command(
    prefix_command,
    category = "Management",
    owners_only,
    track_edits,
    hide_in_help
)]
pub async fn status(
    ctx: Context<'_>,
    #[description = "What online status should I have?"]
    status_type: OnlineStatus,
) -> Result<(), Error> {

    let s = ctx.serenity_context();
    match status_type {
        OnlineStatus::Online => {
            s.online();
        }
        OnlineStatus::Idle => {
            s.idle();
        }
        OnlineStatus::DoNotDisturb => {
            s.dnd();
        }
        OnlineStatus::Invisible => {
            s.invisible();
        }
    };

    ctx.say(format!(
        "Updating status to: **{status_type:?}**. (this could take a moment)"
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
        "streaming" => ActivityType::Streaming,
        "listening" => ActivityType::Listening,
        "watching" => ActivityType::Watching,
        "custom" => ActivityType::Custom,
        "competing" => ActivityType::Competing,
        _ => ActivityType::Playing,
    };

    let status = custom_status.map(std::convert::Into::into);

    let activity_data = ActivityData {
        name: name.into(),
        kind: activity_type_enum,
        state: status,
        url: None,
    };

    ctx.serenity_context().set_activity(Some(activity_data));

    Ok(())
}




pub fn commands() -> [crate::Command; 2] {
    [status(), reset_presence()]
}
