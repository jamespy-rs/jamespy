use std::collections::{HashMap, HashSet};

use regex::Regex;
use serde::{Deserialize, Serialize};

use poise::serenity_prelude::{ChannelId, GuildId};

mod serialize;
use serialize::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JamespyConfig {
    // configuration for the event handler.
    pub events: Events,
    // Tracking for osu!game, harshly hardcoded.
    pub vcstatus: VCStatus,
    // Having a dedicated guild for managing the deployment of jamespy.
    pub spy_guild: Option<SpyGuild>,
    // Control for saving attachments from deleted messages.
    pub attachment_store: Option<Attachments>,
}

impl JamespyConfig {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(GuildId::new(1), "Yes".to_string());

        JamespyConfig {
            events: Events {
                no_log_channels: None,
                no_log_users: None,
                regex: None,
                badlist: None,
                fixlist: None,
                guild_name_override: Some(map),
            },
            vcstatus: VCStatus {
                action: false,
                post_channel: None,
                blacklist_detection: false,
                announce_channel: None,
                regex: None,
                guilds: None,
            },
            spy_guild: None,
            attachment_store: Some(Attachments {
                enabled: true,
                single_limit: Some(200),
                soft_limit: Some(9000),
                hard_limit: Some(10000),
            }),
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

// TODO: categories should probably be stored too.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpyGuild {
    pub status: Init,
    pub self_regex: Option<SelfRegex>,
    pub patterns: Option<PatternAnnounce>,
    pub attachment_hook: Option<AttachmentHook>,
}

impl SpyGuild {
    pub fn new(guild_id: GuildId) -> Self {
        SpyGuild {
            status: Init {
                enabled: true,
                guild_id: Some(guild_id),
            },
            self_regex: None,
            patterns: None,
            attachment_hook: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Init {
    pub enabled: bool,
    pub guild_id: Option<GuildId>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelfRegex {
    pub enabled: bool,
    pub channel_id: Option<ChannelId>,
    pub use_events_regex: bool,
    #[serde(with = "regex_patterns")]
    pub extra_regex: Option<Vec<Regex>>,
    pub context_info: bool,
    pub mention: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PatternAnnounce {
    pub enabled: bool,
    pub default_channel_id: Option<ChannelId>,
    pub list: Vec<Pattern>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pattern {
    pub enabled: bool,
    pub name: Option<String>,
    pub ping: bool,
    pub traverse_embeds: bool,
    pub channel_id: ChannelId,
    #[serde(with = "regex_patterns")]
    pub patterns: Option<Vec<Regex>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AttachmentHook {
    pub enabled: bool,
    pub channel_id: Option<ChannelId>,
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
    pub guild_name_override: Option<HashMap<GuildId, String>>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Attachments {
    pub enabled: bool,
    pub single_limit: Option<u64>,
    pub soft_limit: Option<u64>,
    pub hard_limit: Option<u64>,
}
