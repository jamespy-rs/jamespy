use std::time::Instant;

use ::serenity::all::{ChannelType, GuildChannel};
use poise::serenity_prelude as serenity;
use sysinfo::{Pid, System};

use crate::{Context, Error};

fn uptime_str(seconds: u64) -> String {
    let calculation = |a, b| (a / b, a % b);
    let (minutes, seconds) = calculation(seconds, 60);
    let (hours, minutes) = calculation(minutes, 60);
    let (days, hours) = calculation(hours, 24);

    format!("`{days}d {hours}h {minutes}m {seconds}s`")
}

/// See how long I've been online for!
#[poise::command(slash_command, prefix_command, category = "Meta", user_cooldown = 3)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let uptime = ctx.data().time_started.elapsed().as_secs();

    let uptime_str = uptime_str(uptime);

    ctx.say(uptime_str).await?;

    Ok(())
}

// Post a link to my source code!
#[poise::command(slash_command, prefix_command, category = "Meta", user_cooldown = 3)]
pub async fn source(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("<https://github.com/jamespy-rs/jamespy>").await?;
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

    ctx.data()
        .reqwest
        .get("https://discordapp.com/api/v6/gateway")
        .send()
        .await?;

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

fn bytes_to_gibibytes(bytes: u64) -> f64 {
    const GIBIBYTE: f64 = 1024.0 * 1024.0 * 1024.0;
    bytes as f64 / GIBIBYTE
}

fn bytes_to_mebibytes(bytes: u64) -> f64 {
    const MEBIBYTE: f64 = 1024.0 * 1024.0;
    bytes as f64 / MEBIBYTE
}

#[poise::command(prefix_command, hide_in_help)]
async fn stats(ctx: Context<'_>) -> Result<(), Error> {
    let pid = std::process::id();

    let s = System::new_all();

    let threads = s.cpus().len();
    let total_mem = bytes_to_gibibytes(s.total_memory());
    let used = bytes_to_gibibytes(s.used_memory());

    let seconds = System::uptime();

    let sys_uptime = uptime_str(seconds);
    let bot_uptime = uptime_str(ctx.data().time_started.elapsed().as_secs());

    let mut embed = serenity::CreateEmbed::new()
        .title("Bot & System Statistics")
        .thumbnail(ctx.cache().current_user().face());

    // TODO: readd guild channel count by iterating.
    embed = embed.field(
        "Bot Info",
        format!(
            "Up: {}\nShards: **{}**\nGuilds: **{}**",
            bot_uptime,
            ctx.cache().shard_count(),
            ctx.cache().guilds().len(),
        ),
        true,
    );

    embed = embed.field(
        "System Info",
        format!(
            "CPU Threads: **{threads}**\nMemory: **{used:.2}/{total_mem:.2}** GiB\nUp: \
             {sys_uptime}"
        ),
        true,
    );

    if let Some(process) = s.process(Pid::from(pid as usize)) {
        let physical = bytes_to_mebibytes(process.memory());
        let virtual_m = bytes_to_mebibytes(process.virtual_memory());
        embed = embed.field(
            "Bot Memory",
            format!("Physical: {physical:.2} MiB\nVirtual: {virtual_m:.2} MiB"),
            true,
        );
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    // This uses an inbuilt function because spy guild commands should only
    // be registered in the spy guild.
    crate::register::register_application_commands_buttons(ctx, ctx.data()).await?;

    Ok(())
}

#[poise::command(prefix_command, hide_in_help, guild_only)]
async fn overwrite(ctx: Context<'_>, category: Option<GuildChannel>) -> Result<(), Error> {
    let mut count = 0;

    if let Some(category) = &category {
        if category.kind != ChannelType::Category {
            ctx.say("Not a category!").await?;
        }
    }

    if let Some(category) = &category {
        let guild = ctx.guild().unwrap();
        for channel in &guild.channels {
            if let Some(parent) = channel.parent_id {
                if parent == category.id {
                    count += channel.permission_overwrites.len();
                }
            };
        }
    } else {
        let guild = ctx.guild().unwrap();
        for channel in &guild.channels {
            count += channel.permission_overwrites.len();
        }
    }

    if let Some(category) = category {
        ctx.say(format!("{count} overwrites for {category}"))
            .await?;
    } else {
        ctx.say(format!("{count} channel overwrites for the whole server"))
            .await?;
    }

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 7] {
    [
        uptime(),
        source(),
        help(),
        ping(),
        register(),
        stats(),
        overwrite(),
    ]
}
