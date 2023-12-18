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
    #[serde(rename = "base64_regexes")]
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
    #[serde(rename = "base64_regexes")]
    pub regex_patterns: Option<Vec<Regex>>,
    pub guilds: Option<Vec<GuildId>>,
}

use base64::{engine::general_purpose, Engine as _};

fn deserialize_regex_patterns<'de, D>(deserializer: D) -> Result<Option<Vec<Regex>>, D::Error>
where
    D: Deserializer<'de>,
{
    let patterns: Option<Vec<String>> = Option::deserialize(deserializer)?;

    let regex_patterns = patterns.map(|patterns| {
        patterns
            .into_iter()
            .filter_map(|pattern| {
                let bytes = general_purpose::STANDARD.decode(pattern).unwrap();
                let pattern = String::from_utf8(bytes).unwrap();
                Regex::new(&pattern).ok()
            })
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

    let config_result = fs::read_to_string("config/config.json");
    if let Ok(config_file) = config_result {
        if let Ok(config) = serde_json::from_str::<JamespyConfig>(&config_file) {
            println!("config: {config:?}");
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
