use lazy_static::lazy_static;
use poise::serenity_prelude::{ChannelId, GuildId};
use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::fs;
use std::sync::RwLock;

#[derive(Clone, Debug, Deserialize)]
pub struct JamespyConfig {
    pub vcstatus: VCStatus,
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
        }
    }
    pub fn compile_regex_patterns(&mut self) {
        if let Some(regex_patterns) = &self.vcstatus.regex_patterns {
            let compiled_regex_patterns: Vec<Regex> = regex_patterns
                .iter()
                .filter_map(|pattern| Regex::new(pattern.as_str()).ok())
                .collect();
            self.vcstatus.regex_patterns = Some(compiled_regex_patterns);
        }
    }
}

impl Default for JamespyConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub fn load_config() {
    let default_config = JamespyConfig::new();

    let config_result = fs::read_to_string("config/config.toml");
    match config_result {
        Ok(config_file) => {
            if let Ok(config) = toml::from_str::<JamespyConfig>(&config_file) {
                let mut config_lock = CONFIG.write().unwrap();
                *config_lock = config;
                // Compile regex patterns if any exist in the loaded config
                if let Some(ref mut _regex_patterns) = &mut config_lock.vcstatus.regex_patterns {
                    config_lock.compile_regex_patterns();
                }
            } else {
                eprintln!("Error: Failed to parse config.toml. Using default configuration.");
                *CONFIG.write().unwrap() = default_config;
            }
        }
        Err(_) => {
            eprintln!("Error: Failed to read config.toml. Using default configuration.");
            *CONFIG.write().unwrap() = default_config;
        }
    }
}

lazy_static! {
    pub static ref CONFIG: RwLock<JamespyConfig> = RwLock::new(JamespyConfig::default());
}
