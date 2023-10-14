use lazy_static::lazy_static;
use poise::serenity_prelude::ChannelId;
use serde::Deserialize;
use std::fs;
use std::sync::RwLock;

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct JamespyConfig {
    pub glow: GlowConfig,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct GlowConfig {
    pub action: bool,
    pub channel_id: Option<ChannelId>,
}

impl JamespyConfig {
    pub fn new() -> Self {
        JamespyConfig {
            glow: GlowConfig {
                action: false,
                channel_id: None,
            },
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

    let config_result = fs::read_to_string("config.toml");
    match config_result {
        Ok(config_file) => {
            if let Ok(config) = toml::from_str::<JamespyConfig>(&config_file) {
                *CONFIG.write().unwrap() = config;
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
