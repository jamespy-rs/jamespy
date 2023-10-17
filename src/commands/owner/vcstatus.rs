use std::fs;

use crate::config::{JamespyConfig, CONFIG};
use crate::{Context, Error};

#[poise::command(
    prefix_command,
    category = "VCStatus",
    hide_in_help,
    check = "servermanager",
    subcommands("toggle", "reload_regex")
)]
pub async fn vcstatus(ctx: Context<'_>) -> Result<(), Error> {
    let vcstatus = {
        let config = CONFIG.read().unwrap();
        config.vcstatus.clone()
    };
    let action = vcstatus.action;
    let blacklist_detection = vcstatus.blacklist_detection;
    let mut post_channel_id: u64 = 0;
    let mut announce_channel_id: u64 = 0;

    let announce_channel_name: String;
    let post_channel_name: String;

    if let Some(channel) = vcstatus.post_channel {
        post_channel_name = channel.name(ctx).await?;
        post_channel_id = channel.get();
    } else {
        post_channel_name = "None".to_string();
    }

    if let Some(channel) = vcstatus.announce_channel {
        announce_channel_name = channel.name(ctx).await?;
        announce_channel_id = channel.get();
    } else {
        announce_channel_name = "None".to_string();
    }
    // show regex later.
    let message = format!(
        "Enabled:{}\nPost Channel: {} (ID:{})\nBlacklist detection: {}\nBlacklist Channel: {} (ID:{})\n\nTry the arguments \"toggle\" and \"reload-regex\".",
        action, post_channel_name, post_channel_id, blacklist_detection, announce_channel_name, announce_channel_id
    );
    ctx.say(message).await?;

    Ok(())
}

#[poise::command(prefix_command, category = "VCStatus", hide_in_help, owners_only)]
pub async fn toggle(ctx: Context<'_>) -> Result<(), Error> {
    let vcstatus = {
        let mut config = CONFIG.write().unwrap();
        let mut write_lock = config.vcstatus.clone();
        write_lock.action = !write_lock.action;
        config.vcstatus = write_lock.clone();
        write_lock
    };

    let content = format!("Toggling VCStatus tracking (blacklist is not touched):\nEnabled: {}\nBlacklist Enabled: {}", vcstatus.action, vcstatus.blacklist_detection);
    ctx.say(content).await?;

    Ok(())
}

#[poise::command(
    rename = "reload-regex",
    aliases("enable"),
    prefix_command,
    category = "Glow",
    hide_in_help,
    owners_only
)]
pub async fn reload_regex(ctx: Context<'_>) -> Result<(), Error> {
    new_regex(&ctx).await;
    Ok(())
}

pub async fn new_regex(ctx: &Context<'_>) {
    let config_result = fs::read_to_string("config/config.toml");
    match config_result {
        Ok(config_file) => {
            if let Ok(config) = toml::from_str::<JamespyConfig>(&config_file) {
                if let Some(ref regex_patterns) = config.vcstatus.regex_patterns {
                    {
                        let mut write_lock = CONFIG.write().unwrap();
                        let cloned_regex_patterns: Vec<regex::Regex> = regex_patterns.clone();
                        write_lock.vcstatus.regex_patterns = Some(cloned_regex_patterns);
                    }
                    let _ = ctx
                        .say(format!(
                            "Successfully updated {} regex patterns!",
                            regex_patterns.len()
                        ))
                        .await;
                } else {
                    println!("No regex patterns found in config.toml.");
                }
            } else {
                eprintln!("Failed to parse the configuration!");
            }
        }
        Err(_) => {
            eprintln!("Failed to read the configuration!");
        }
    }
}

pub async fn servermanager(ctx: Context<'_>) -> Result<bool, Error> {
    let allowed_users = [158567567487795200, 101090238067113984, 291089948709486593]; // Me, Phil, Ruben
    let user_id = ctx.author().id.get();
    let trontin = allowed_users.contains(&user_id);

    Ok(trontin)
}
