use poise::serenity_prelude::{ChannelId, GuildId};
use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::{collections::HashSet, fs};

use crate::utils::misc::read_words_from_file;

#[derive(Clone, Debug, Deserialize)]
pub struct JamespyConfig {
    pub events_config: EventsConfig,
    pub vcstatus: VCStatus,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EventsConfig {
    pub no_log_channels: Option<Vec<u64>>,
    pub no_log_users: Option<Vec<u64>>,
    #[serde(deserialize_with = "deserialize_regex_patterns")]
    pub regex_patterns: Option<Vec<Regex>>,
    #[serde(deserialize_with = "deserialize_words_file_to_words")]
    pub badlist: Option<HashSet<String>>,
    #[serde(deserialize_with = "deserialize_words_file_to_words")]
    pub fixlist: Option<HashSet<String>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VCStatus {
    pub action: bool,
    pub post_channel: Option<ChannelId>,
    pub blacklist_detection: bool,
    pub announce_channel: Option<ChannelId>,
    #[serde(deserialize_with = "deserialize_regex_patterns")]
    pub regex_patterns: Option<Vec<Regex>>,
    pub guilds: Option<Vec<GuildId>>,
}

fn deserialize_regex_patterns<'de, D>(deserializer: D) -> Result<Option<Vec<Regex>>, D::Error>
where
    D: Deserializer<'de>,
{
    let patterns: Option<Vec<String>> = Option::deserialize(deserializer)?;
    let regex_patterns = patterns.map(|patterns| {
        patterns
            .into_iter()
            .filter_map(|pattern| Regex::new(&pattern).ok())
            .collect()
    });
    Ok(regex_patterns)
}

fn deserialize_words_file_to_words<'de, D>(
    deserializer: D,
) -> Result<Option<HashSet<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<String> = Option::deserialize(deserializer)?;
    let lines = value.map(|value| read_words_from_file(&value));

    Ok(lines)
}

impl JamespyConfig {
    pub fn new() -> Self {
        JamespyConfig {
            vcstatus: VCStatus {
                action: false,
                post_channel: None,
                blacklist_detection: false,
                announce_channel: None,
                regex_patterns: None,
                guilds: None,
            },
            events_config: EventsConfig {
                no_log_channels: None,
                no_log_users: None,
                regex_patterns: None,
                badlist: Some(read_words_from_file("data/badwords.txt")),
                fixlist: Some(read_words_from_file("data/fixwords.txt")),
            },
        }
    }
}

impl Default for JamespyConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub fn load_config() -> JamespyConfig {
    let default_config = JamespyConfig::new();

    let config_result = fs::read_to_string("config/config.toml");
    match config_result {
        Ok(config_file) => {
            if let Ok(config) = toml::from_str::<JamespyConfig>(&config_file) {
                config
            } else {
                eprintln!("Error: Failed to parse config.toml. Using default configuration.");
                if let Err(err) = toml::from_str::<JamespyConfig>(&config_file) {
                    eprintln!("Parse error details: {}", err);
                }
                default_config
            }
        }
        Err(_) => {
            eprintln!("Error: Failed to read config.toml. Using default configuration.");
            default_config
        }
    }
}
