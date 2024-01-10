use std::collections::HashSet;

use regex::Regex;
use serde::{Deserialize, Serialize};

use poise::serenity_prelude::{ChannelId, GuildId};

mod serialize;
use serialize::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JamespyConfig {
    pub events: Events,     // configuration for the event handler.
    pub vcstatus: VCStatus, // Tracking for osu!game.
    pub spy_guild: Option<SpyGuild>,
    pub attachment_store: Option<Attachments>,
}

impl JamespyConfig {
    pub fn new() -> Self {
        JamespyConfig {
            events: Events {
                no_log_channels: None,
                no_log_users: None,
                regex: None,
                badlist: None,
                fixlist: None,
            },
            vcstatus: VCStatus {
                action: false,
                post_channel: None,
                blacklist_detection: false,
                announce_channel: None,
                regex: None,
                guilds: None,
            },
            spy_guild: Some(SpyGuild { event: 1 }),
            attachment_store: None,
        }
    }

    pub fn write_config(&self) {
        let config_result = std::fs::read_to_string("config/config.json");

        if let Ok(_config_file) = config_result {
            let writer = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(false)
                .open("config/config.json");

            match writer {
                Ok(writer) => match serde_json::to_writer_pretty(writer, &self) {
                    Ok(()) => println!("Successfully saved config"),
                    Err(e) => println!("Failed to save config: {e}"),
                },
                Err(e) => println!("Unable to write config: {e}"),
            };
        }
    }

    pub fn load_config() -> Self {
        let default_config = JamespyConfig::new();

        let config_result = std::fs::read_to_string("config/config.json");
        if let Ok(config_file) = config_result {
            if let Ok(mut config) = serde_json::from_str::<JamespyConfig>(&config_file) {
                // Set value of unconfigurable properties.
                config.events.badlist = Some(read_words_from_file("config/lists/badwords.txt"));
                config.events.fixlist = Some(read_words_from_file("config/lists/fixwords.txt"));

                config
            } else {
                eprintln!("Error: Failed to parse config.json. Using default configuration.");
                if let Err(err) = serde_json::from_str::<JamespyConfig>(&config_file) {
                    eprintln!("Parse error details: {err}");
                }
                default_config
            }
        } else {
            eprintln!("Error: Failed to read config.json. Using default configuration.");
            default_config
        }
    }
}

impl Default for JamespyConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VCStatus {
    pub action: bool,
    pub post_channel: Option<ChannelId>,
    pub blacklist_detection: bool,
    pub announce_channel: Option<ChannelId>,
    #[serde(with = "regex_patterns")]
    pub regex: Option<Vec<Regex>>,
    pub guilds: Option<Vec<GuildId>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpyGuild {
    pub event: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Events {
    pub no_log_channels: Option<Vec<u64>>,
    pub no_log_users: Option<Vec<u64>>,
    #[serde(with = "regex_patterns")]
    pub regex: Option<Vec<Regex>>,
    #[serde(skip_serializing)]
    pub badlist: Option<HashSet<String>>,
    #[serde(skip_serializing)]
    pub fixlist: Option<HashSet<String>>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Attachments {
    pub enabled: bool,
    pub single_limit: Option<u64>,
    pub soft_limit: Option<u64>,
    pub hard_limit: Option<u64>,
}
