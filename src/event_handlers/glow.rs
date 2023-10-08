// File to handle glowing into private channels
use crate::Error;
use lazy_static::lazy_static;
use poise::serenity_prelude::{self as serenity, ChannelId};
use serenity::all::Member;
use std::sync::RwLock;

#[derive(Clone)]
pub struct GlowConfig {
    pub action: bool,
    pub channel_id: Option<ChannelId>,
}

impl GlowConfig {
    pub fn new() -> Self {
        GlowConfig {
            action: false,
            channel_id: None,
        }
    }
}

lazy_static! {
    pub static ref CONFIG: RwLock<GlowConfig> = RwLock::new(GlowConfig::new());
}

pub async fn avatar_change(
    ctx: &serenity::Context,
    old: &Member,
    new: &Member,
) -> Result<(), Error> {
    let config = CONFIG.read().unwrap();
    if config.action {
        println!("test");
    }
    Ok(())
}

pub async fn guild_avatar_change(
    ctx: &serenity::Context,
    old: &Member,
    new: &Member,
) -> Result<(), Error> {
    //
    Ok(())
}

pub async fn banner_change(
    ctx: &serenity::Context,
    old: &Member,
    new: &Member,
) -> Result<(), Error> {
    //
    Ok(())
}
