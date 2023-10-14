use crate::config::CONFIG;
use crate::utils::misc::get_guild_name;
use crate::Error;
use poise::serenity_prelude::{self as serenity, Member};

pub async fn avatar_change(
    ctx: &serenity::Context,
    old: &Member,
    new: &Member,
) -> Result<(), Error> {
    let (action, channel_id) = {
        let config = CONFIG.read().unwrap().glow;
        (config.action, config.channel_id)
    };

    if action {
        if let Some(channel_id) = channel_id {
            let message_content = match (old.user.avatar_url(), new.user.avatar_url()) {
                (Some(old_avatar), Some(new_avatar)) => {
                    format!(
                        "<@{}> (ID:{}) changed their avatar: {} -> {}",
                        new.user.id, new.user.id, old_avatar, new_avatar
                    )
                }
                (Some(old_avatar), None) => {
                    format!(
                        "<@{}> (ID:{}) removed their avatar. Avatar Previously was: {}",
                        new.user.id, new.user.id, old_avatar
                    )
                }
                (None, Some(new_avatar)) => {
                    format!(
                        "<@{}> (ID:{}) set a new avatar: {}",
                        new.user.id, new.user.id, new_avatar
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

pub async fn guild_avatar_change(
    ctx: &serenity::Context,
    old: &Member,
    new: &Member,
) -> Result<(), Error> {
    let (action, channel_id) = {
        let config = CONFIG.read().unwrap().glow;
        (config.action, config.channel_id)
    };
    if action {
        if let Some(channel_id) = channel_id {
            let guild_name = get_guild_name(ctx, new.guild_id);
            let message_content = match (old.avatar_url(), new.avatar_url()) {
                (Some(old_avatar), Some(new_avatar)) => {
                    format!(
                        "<@{}> (ID:{}) changed their server avatar in {}: {} -> {}",
                        new.user.id, new.user.id, guild_name, old_avatar, new_avatar
                    )
                }
                (Some(old_avatar), None) => {
                    format!(
                        "<@{}> (ID:{}) removed their avatar in {}. Avatar Previously was: {}",
                        new.user.id, new.user.id, guild_name, old_avatar
                    )
                }
                (None, Some(new_avatar)) => {
                    format!(
                        "<@{}>  (ID:{}) set a new avatar in {}: {}",
                        new.user.id, new.user.id, guild_name, new_avatar
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
        let config = CONFIG.read().unwrap().glow;
        (config.action, config.channel_id)
    };
    if action {
        if let Some(channel_id) = channel_id {
            let message_content = match (old.user.banner_url(), new.user.banner_url()) {
                (Some(old_banner), Some(new_banner)) => {
                    format!(
                        "<@{}> (ID:{}) changed their banner: {} -> {}",
                        new.user.id, new.user.id, old_banner, new_banner
                    )
                }
                (Some(old_banner), None) => {
                    format!(
                        "<@{}> (ID:{}) removed their banner. Banner Previously was: {}",
                        new.user.id, new.user.id, old_banner
                    )
                }
                (None, Some(new_banner)) => {
                    format!(
                        "<@{}> (ID:{}) set a new banner: {}",
                        new.user.id, new.user.id, new_banner
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

pub async fn username_change(
    ctx: &serenity::Context,
    old: &Member,
    new: &Member,
) -> Result<(), Error> {
    let (action, channel_id) = {
        let config = CONFIG.read().unwrap().glow;
        (config.action, config.channel_id)
    };
    if action {
        if let Some(channel_id) = channel_id {
            let message_content = format!(
                "<@{}> (ID:{}) changed their username: {} -> {}",
                old.user.id, old.user.id, old.user.name, new.user.name
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
        let config = CONFIG.read().unwrap().glow;
        (config.action, config.channel_id)
    };
    if action {
        if let Some(channel_id) = channel_id {
            let message_content = match (&old.user.global_name, &new.user.global_name) {
                (Some(old_name), Some(new_name)) => {
                    format!(
                        "<@{}> (ID:{}) changed their display name: {} -> {}",
                        old.user.id, old.user.id, old_name, new_name
                    )
                }
                (Some(old_name), None) => {
                    format!(
                        "<@{}> (ID:{}) removed their display name of: {}",
                        old.user.id, old.user.id, old_name
                    )
                }
                (None, Some(new_name)) => {
                    format!(
                        "<@{}> (ID:{}) set their display name to: {}",
                        new.user.id, new.user.id, new_name
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

pub async fn nickname_change(
    ctx: &serenity::Context,
    old: &Member,
    new: &Member,
) -> Result<(), Error> {
    let (action, channel_id) = {
        let config = CONFIG.read().unwrap().glow;
        (config.action, config.channel_id)
    };
    if action {
        if let Some(channel_id) = channel_id {
            let guild_name = get_guild_name(ctx, old.guild_id);
            let message_content = match (&old.nick, &new.nick) {
                (Some(old_name), Some(new_name)) => {
                    format!(
                        "<@{}> (ID:{}) changed their nickname in {}: {} -> {}",
                        old.user.id, new.user.id, guild_name, old_name, new_name
                    )
                }
                (Some(old_name), None) => {
                    format!(
                        "<@{}> (ID:{}) removed their nickname of: {}",
                        old.user.id, new.user.id, old_name
                    )
                }
                (None, Some(new_name)) => {
                    format!(
                        "<@{}> (ID:{}) set their nickname to: {}",
                        new.user.id, new.user.id, new_name
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
