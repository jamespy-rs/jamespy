use std::collections::HashMap;

#[cfg(feature = "websocket")]
use crate::event_handlers::broadcast_message;
#[cfg(feature = "websocket")]
use crate::websocket::PEER_MAP;

#[cfg(feature = "websocket")]
use crate::event_handlers::WebSocketEvent;
#[cfg(feature = "websocket")]
use tokio_tungstenite::tungstenite;

use crate::utils::misc::get_guild_name;
use crate::Error;
use poise::serenity_prelude::{self as serenity, Guild, GuildId, Member, User, AuditLogEntry, ChannelId, CreateEmbedAuthor};
use serenity::model::guild::audit_log::Action;

pub async fn guild_create(
    ctx: &serenity::Context,
    guild: Guild,
    is_new: Option<bool>,
    //data: &Data,
) -> Result<(), Error> {
    #[cfg(feature = "websocket")]
    {
        let new_message_event = WebSocketEvent::GuildCreate {
            guild: guild.clone(),
            is_new,
        };
        let message = serde_json::to_string(&new_message_event).unwrap();
        let peers = { PEER_MAP.lock().unwrap().clone() };

        let message = tungstenite::protocol::Message::Text(message);
        broadcast_message(peers, message).await;
    }

    if let Some(true) = is_new {
        println!(
            "\x1B[33mJoined {} (ID:{})!\nNow in {} guild(s)\x1B[0m",
            guild.name,
            guild.id,
            ctx.cache.guilds().len()
        );
    }
    Ok(())
}

pub async fn guild_member_addition(
    ctx: &serenity::Context,
    new_member: Member,
) -> Result<(), Error> {
    let guild_id = new_member.guild_id;
    let joined_user_id = new_member.user.id;

    let guild_name = get_guild_name(ctx, guild_id);

    #[cfg(feature = "websocket")]
    {
        let new_message_event = WebSocketEvent::GuildMemberAddition {
            new_member: new_member.clone(),
            guild_name: guild_name.clone(),
        };
        let message = serde_json::to_string(&new_message_event).unwrap();
        let peers = { PEER_MAP.lock().unwrap().clone() };

        let message = tungstenite::protocol::Message::Text(message);
        broadcast_message(peers, message).await;
    }

    println!(
        "\x1B[33m[{}] {} (ID:{}) has joined!\x1B[0m",
        guild_name, new_member.user.name, joined_user_id
    );
    Ok(())
}

pub async fn guild_member_removal(
    ctx: &serenity::Context,
    guild_id: GuildId,
    user: User,
) -> Result<(), Error> {
    let guild_name = get_guild_name(ctx, guild_id);

    #[cfg(feature = "websocket")]
    {
        let new_message_event = WebSocketEvent::GuildMemberRemoval {
            guild_id,
            user: user.clone(),
            guild_name: guild_name.clone(),
        };
        let message = serde_json::to_string(&new_message_event).unwrap();
        let peers = { PEER_MAP.lock().unwrap().clone() };

        let message = tungstenite::protocol::Message::Text(message);
        broadcast_message(peers, message).await;
    }

    println!(
        "\x1B[33m[{}] {} (ID:{}) has left!\x1B[0m",
        guild_name, user.name, user.id
    );
    Ok(())
}

pub async fn guild_audit_log_entry_create(
    ctx: &serenity::Context,
    entry: AuditLogEntry,
    guild_id: GuildId
) -> Result<(), Error> {
    if guild_id == 98226572468690944 {
        if let Action::AutoMod(serenity::AutoModAction::FlagToChannel) = &entry.action {
            if let Some(reason) = entry.reason {
                if reason.starts_with("Voice Channel Status") {
                    let (user_name, avatar_url) = match entry.user_id.to_user(&ctx).await {
                        Ok(user) => {
                            (user.name.clone(), user.avatar_url().unwrap_or_default())
                        }
                        Err(_) => (
                            "Unknown User".to_string(),
                            String::from("https://cdn.discordapp.com/embed/avatars/0.png"),
                        ),
                    };

                    let mut cloned_messages = HashMap::new();

                    let channel_id: Option<u64> = if let Some(options) = entry.options {
                        match options.auto_moderation_rule_name {
                            Some(rule_name) => match rule_name.as_str() {
                                "Bad Words ❌ [BLOCKED]" => Some(697738506944118814),
                                "Bad Words 📫 [ALERT]" => Some(158484765136125952),
                                _ => None,
                            },
                            None => None,
                        }
                    } else {
                        None
                    };

                    if let Some(id) = channel_id {
                        tokio::time::sleep(
                            std::time::Duration::from_secs(1)
                                + std::time::Duration::from_millis(500),
                        )
                        .await;
                        if let Some(channel_messages) = ctx.cache.channel_messages(id) {
                            cloned_messages = channel_messages.clone();
                        }
                        let mut messages: Vec<_> = cloned_messages.values().collect();
                        messages.reverse();
                        let last_8_messages = messages.iter().take(8);
                        let messages = last_8_messages;

                        let mut status = format!(
                            "Unknown (check #{})",
                            ChannelId::new(id)
                                .name(&ctx)
                                .await
                                .unwrap_or("Unknown".to_string())
                        )
                        .to_string();

                        for message in messages {
                            if message.author.id == entry.user_id {
                                if let Some(kind) =
                                    &message.embeds.get(0).and_then(|e| e.kind.clone())
                                {
                                    if kind == "auto_moderation_message" {
                                        if let Some(description) =
                                            &message.embeds[0].description
                                        {
                                            status = description.to_string();
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        // i'll merge this logic with the above but i'll just do something basic now.
                        // blocked = text logs
                        // alert = war room
                        let author_title = match id {
                            697738506944118814 => format!(
                                "{} tried to set an inappropriate status",
                                user_name
                            ),
                            158484765136125952 => {
                                format!("{} set a possibly inappropriate status", user_name)
                            }
                            _ => {
                                format!("{} set a possibly inappropriate status", user_name)
                            }
                        };

                        let footer = serenity::CreateEmbedFooter::new(format!(
                            "User ID: {} • Please check status manually in #{}",
                            entry.user_id,
                            ChannelId::new(id)
                                .name(&ctx)
                                .await
                                .unwrap_or("Unknown".to_string())
                        ));
                        let embed = serenity::CreateEmbed::default()
                            .author(
                                CreateEmbedAuthor::new(author_title).icon_url(avatar_url),
                            )
                            .field("Status", status, true)
                            .footer(footer);
                        let builder = serenity::CreateMessage::default()
                            .embed(embed)
                            .content(format!("<@{}>", entry.user_id));
                        // this is gg/osu only, so i won't enable configurable stuff for this.
                        ChannelId::new(158484765136125952)
                            .send_message(&ctx, builder.clone())
                            .await?;
                        ChannelId::new(1163544192866336808)
                            .send_message(ctx, builder)
                            .await?;
                    }
                }
            }
        }
    }
    Ok(())
}
