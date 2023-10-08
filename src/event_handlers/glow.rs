use crate::utils::misc::get_guild_name;
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
    let (action, channel_id) = {
        let config = CONFIG.read().unwrap();
        (config.action, config.channel_id)
    };

    if action {
        if let Some(channel_id) = channel_id {
            let message_content = match (old.user.avatar_url(), new.user.avatar_url()) {
                (Some(old_avatar), Some(new_avatar)) => {
                    format!(
                        "{} changed their avatar: {} -> {}",
                        new.user.name, old_avatar, new_avatar
                    )
                }
                (Some(old_avatar), None) => {
                    format!(
                        "{} removed their avatar. Avatar Previously was: {}",
                        new.user.name, old_avatar
                    )
                }
                (None, Some(new_avatar)) => {
                    format!("{} set a new avatar: {}", new.user.name, new_avatar)
                }
                _ => String::from("Huh."),
            };

            channel_id
                .send_message(
                    ctx,
                    serenity::CreateMessage::default().content(message_content),
                )
                .await?;
        }
    }
    Ok(())
}

pub async fn guild_avatar_change(
    ctx: &serenity::Context,
    old: &Member,
    new: &Member,
) -> Result<(), Error> {
    let (action, channel_id) = {
        let config = CONFIG.read().unwrap();
        (config.action, config.channel_id)
    };
    if action {
        if let Some(channel_id) = channel_id {
            let guild_name = get_guild_name(ctx, new.guild_id);
            let message_content = match (old.avatar_url(), new.avatar_url()) {
                (Some(old_avatar), Some(new_avatar)) => {
                    format!(
                        "{} changed their server avatar in {}: {} -> {}",
                        new.user.name, guild_name, old_avatar, new_avatar
                    )
                }
                (Some(old_avatar), None) => {
                    format!(
                        "{} removed their avatar in {}. Avatar Previously was: {}",
                        new.user.name, guild_name, old_avatar
                    )
                }
                (None, Some(new_avatar)) => {
                    format!(
                        "{} set a new avatar in {}: {}",
                        new.user.name, guild_name, new_avatar
                    )
                }
                _ => String::from("Huh."),
            };

            channel_id
                .send_message(
                    ctx,
                    serenity::CreateMessage::default().content(message_content),
                )
                .await?;
        }
    }
    Ok(())
}

pub async fn banner_change(
    ctx: &serenity::Context,
    old: &Member,
    new: &Member,
) -> Result<(), Error> {
    let (action, channel_id) = {
        let config = CONFIG.read().unwrap();
        (config.action, config.channel_id)
    };
    if action {
        if let Some(channel_id) = channel_id {
            let message_content = match (old.user.banner_url(), new.user.banner_url()) {
                (Some(old_banner), Some(new_banner)) => {
                    format!(
                        "{} changed their banner: {} -> {}",
                        new.user.name, old_banner, new_banner
                    )
                }
                (Some(old_banner), None) => {
                    format!(
                        "{} removed their banner. Banner Previously was: {}",
                        new.user.name, old_banner
                    )
                }
                (None, Some(new_banner)) => {
                    format!("{} set a new banner: {}", new.user.name, new_banner)
                }
                _ => String::from("Huh."),
            };

            channel_id
                .send_message(
                    ctx,
                    serenity::CreateMessage::default().content(message_content),
                )
                .await?;
        }
    }
    Ok(())
}

pub async fn username_change(
    ctx: &serenity::Context,
    old: &Member,
    new: &Member,
) -> Result<(), Error> {
    let (action, channel_id) = {
        let config = CONFIG.read().unwrap();
        (config.action, config.channel_id)
    };
    if action {
        if let Some(channel_id) = channel_id {
            let message_content = format!(
                "<@{}> changed their username: {} -> {}",
                old.user.id, old.user.name, new.user.name
            );

            channel_id
                .send_message(
                    ctx,
                    serenity::CreateMessage::default().content(message_content),
                )
                .await?;
        }
    }
    Ok(())
}

pub async fn globalname_change(
    ctx: &serenity::Context,
    old: &Member,
    new: &Member,
) -> Result<(), Error> {
    let (action, channel_id) = {
        let config = CONFIG.read().unwrap();
        (config.action, config.channel_id)
    };
    if action {
        if let Some(channel_id) = channel_id {
            let message_content = match (&old.user.global_name, &new.user.global_name) {
                (Some(old_name), Some(new_name)) => {
                    format!(
                        "<@{}> changed their display name: {} -> {}",
                        old.user.id, old_name, new_name
                    )
                }
                (Some(old_name), None) => {
                    format!(
                        "<@{}> removed their display name of: {}",
                        old.user.id, old_name
                    )
                }
                (None, Some(new_name)) => {
                    format!("<@{}> set their display name to: {}", new.user.id, new_name)
                }
                _ => String::from("Huh."),
            };

            channel_id
                .send_message(
                    ctx,
                    serenity::CreateMessage::default().content(message_content),
                )
                .await?;
        }
    }
    Ok(())
}

pub async fn nickname_change(
    ctx: &serenity::Context,
    old: &Member,
    new: &Member,
) -> Result<(), Error> {
    let (action, channel_id) = {
        let config = CONFIG.read().unwrap();
        (config.action, config.channel_id)
    };
    if action {
        if let Some(channel_id) = channel_id {
            let guild_name = get_guild_name(ctx, old.guild_id);
            let message_content = match (&old.nick, &new.nick) {
                (Some(old_name), Some(new_name)) => {
                    format!(
                        "<@{}> changed their nickname in {}: {} -> {}",
                        old.user.id, guild_name, old_name, new_name
                    )
                }
                (Some(old_name), None) => {
                    format!("<@{}> removed their nickname of: {}", old.user.id, old_name)
                }
                (None, Some(new_name)) => {
                    format!("<@{}> set their nickname to: {}", new.user.id, new_name)
                }
                _ => String::from("Huh."),
            };

            channel_id
                .send_message(
                    ctx,
                    serenity::CreateMessage::default().content(message_content),
                )
                .await?;
        }
    }
    Ok(())
}
