use std::time::Instant;

use poise::serenity_prelude as serenity;

use crate::{Context, Error};

/// See how long I've been online for!
#[poise::command(slash_command, prefix_command, category = "Meta", user_cooldown = 3)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let uptime = ctx.data().time_started.elapsed();

    let calculation = |a, b| (a / b, a % b);

    let seconds = uptime.as_secs();
    let (minutes, seconds) = calculation(seconds, 60);
    let (hours, minutes) = calculation(minutes, 60);
    let (days, hours) = calculation(hours, 24);

    ctx.say(format!("`Uptime: {days}d {hours}h {minutes}m {seconds}s`"))
        .await?;

    Ok(())
}

// Post a link to my source code!
#[poise::command(slash_command, prefix_command, category = "Meta", user_cooldown = 3)]
pub async fn source(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(
        "<https://github.com/jamespy-rs/jamespy>\n<https://github.com/jamespy-rs/jamespy-client>",
    )
    .await?;
    Ok(())
}

/// Show general help or help to a specific command!
#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    category = "Miscellaneous",
    user_cooldown = 3
)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            ephemeral: true,
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// pong!
#[poise::command(slash_command, prefix_command, category = "Meta", user_cooldown = 10)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let now = Instant::now();
    reqwest::get("https://discordapp.com/api/v6/gateway").await?;
    let get_latency = now.elapsed().as_millis();
    let now = Instant::now();
    let ping_msg = ctx.say("Calculating...").await?;
    let post_latency = now.elapsed().as_millis();

    ping_msg
        .edit(
            ctx,
            poise::CreateReply::default().content("").embed(
                serenity::CreateEmbed::default()
                    .title("Pong!")
                    .field("GET Latency", format!("{get_latency}ms"), false)
                    .field("POST Latency", format!("{post_latency}ms"), false),
            ),
        )
        .await?;

    Ok(())
}

#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;

    Ok(())
}

pub fn commands() -> [crate::Command; 5] {
    [uptime(), source(), help(), ping(), register()]
}
