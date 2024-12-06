#![warn(clippy::pedantic)]

use std::collections::{HashMap, HashSet};

use regex::Regex;
use serde::{Deserialize, Serialize};

use poise::serenity_prelude::{ChannelId, GuildId};

mod serialize;
use serialize::{read_words_from_file, regex_patterns};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct JamespyConfig {
    // configuration for the event handler.
    pub events: Events,
    // Tracking for osu!game, harshly hardcoded.
    pub vcstatus: VCStatus,
}

impl JamespyConfig {
    #[must_use]
    pub fn new() -> Self {
        JamespyConfig {
            events: Events::default(),
            vcstatus: VCStatus::default(),
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

    #[must_use]
    pub fn load_config() -> Self {
        let default_config = JamespyConfig::new();

        let config_result = std::fs::read_to_string("config/config.json");
        if let Ok(config_file) = config_result {
            if let Ok(mut config) = serde_json::from_str::<JamespyConfig>(&config_file) {
                // Set value of unconfigurable properties.
                config.events.badlist = read_words_from_file("config/lists/badwords.txt");
                config.events.fixlist = read_words_from_file("config/lists/fixwords.txt");

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

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct VCStatus {
    pub action: bool,
    pub post_channel: Option<ChannelId>,
    pub blacklist_detection: bool,
    pub announce_channel: Option<ChannelId>,
    #[serde(with = "regex_patterns")]
    pub regex: Option<Vec<Regex>>,
    pub guilds: Option<Vec<GuildId>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Events {
    pub no_log_channels: Option<Vec<u64>>,
    pub no_log_users: Option<Vec<u64>>,
    #[serde(with = "regex_patterns")]
    pub regex: Option<Vec<Regex>>,
    #[serde(skip_serializing)]
    pub badlist: HashSet<String>,
    #[serde(skip_serializing)]
    pub fixlist: HashSet<String>,
    pub guild_name_override: Option<HashMap<GuildId, String>>,
}
