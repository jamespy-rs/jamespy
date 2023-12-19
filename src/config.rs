use poise::serenity_prelude::{ChannelId, GuildId, MessageId};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::HashSet, fs};

use crate::utils::misc::read_words_from_file;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JamespyConfig {
    pub events_config: EventsConfig,
    pub vcstatus: VCStatus,
    #[cfg(feature = "castle")]
    pub castle_conf: Option<Castle>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventsConfig {
    pub no_log_channels: Option<Vec<u64>>,
    pub no_log_users: Option<Vec<u64>>,
    #[serde(deserialize_with = "deserialize_regex_patterns")]
    #[serde(serialize_with = "serialize_regex")]
    #[serde(rename = "base64_regexes")]
    pub regex_patterns: Option<Vec<Regex>>,
    #[serde(deserialize_with = "deserialize_words_file_to_words")]
    #[serde(skip_serializing)]
    #[serde(default)]
    pub badlist: Option<HashSet<String>>,
    #[serde(deserialize_with = "deserialize_words_file_to_words")]
    #[serde(skip_serializing)]
    #[serde(default)]
    pub fixlist: Option<HashSet<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VCStatus {
    pub action: bool,
    pub post_channel: Option<ChannelId>,
    pub blacklist_detection: bool,
    pub announce_channel: Option<ChannelId>,
    #[serde(deserialize_with = "deserialize_regex_patterns")]
    #[serde(serialize_with = "serialize_regex")]
    #[serde(rename = "base64_regexes")]
    pub regex_patterns: Option<Vec<Regex>>,
    pub guilds: Option<Vec<GuildId>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Castle {
    pub base: Option<InitStatus>,
    pub stats: Option<StatsStatus>,
    pub self_regex: Option<SelfRegex>,
    pub media: Option<MediaStashingStatus>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InitStatus {
    pub setup_complete: bool,
    pub guild_id: Option<GuildId>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StatsStatus {
    pub stats_enabled: bool,
    pub stats_channel: Option<ChannelId>,
    pub stats_message: Option<MessageId>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelfRegex {
    pub regex_self: bool,
    pub regex_channel: Option<ChannelId>,
    pub regex_self_ping: bool,
    #[serde(deserialize_with = "deserialize_regex_patterns")]
    #[serde(serialize_with = "serialize_regex")]
    pub self_regexes: Option<Vec<Regex>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MediaStashingStatus {
    pub media_stashing_post: bool,
    pub media_stash_channel: Option<ChannelId>,
    pub single_limit: Option<u64>,
    pub soft_limit: Option<u64>,
    pub hard_limit: Option<u64>,
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
pub fn serialize_regex<S: Serializer>(
    patterns: &Option<Vec<Regex>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let unwrapped = patterns.as_ref().cloned().unwrap_or_default();
    let mut new: Vec<String> = Vec::new();

    for pattern in unwrapped {
        new.push(general_purpose::STANDARD.encode(pattern.as_str()));
    }

    serializer.collect_seq(new)
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
            #[cfg(feature = "castle")]
            castle_conf: Some(Castle {
                base: Some(InitStatus {
                    setup_complete: false,
                    guild_id: None,
                }),
                stats: None,
                self_regex: None,
                media: None,
            }),
        }
    }

    pub fn write_config(&self) {
        let config_result = fs::read_to_string("config/config.json");

        if let Ok(_config_file) = config_result {
            let writer = fs::OpenOptions::new()
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
        if let Ok(mut config) = serde_json::from_str::<JamespyConfig>(&config_file) {
            // stupid hardcode until i can be bothered to fix and rewrite this.
            config.events_config.badlist = Some(read_words_from_file("data/badwords.txt"));
            config.events_config.fixlist = Some(read_words_from_file("data/fixwords.txt"));
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
