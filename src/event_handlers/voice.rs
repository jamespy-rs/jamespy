#[cfg(feature = "websocket")]
use crate::{
    event_handlers::{broadcast_message, WebSocketEvent},
    websocket::PEER_MAP,
};
use poise::serenity_prelude::{self as serenity, VoiceState};
#[cfg(feature = "websocket")]
use tokio_tungstenite::tungstenite;

use crate::Error;

pub async fn voice_state_update(
    ctx: &serenity::Context,
    old: Option<VoiceState>,
    new: VoiceState,
) -> Result<(), Error> {
    #[cfg(feature = "websocket")]
    {
        let mut old_guild_name = None;
        let mut old_channel_name = None;
        let mut new_guild_name = None;
        let mut new_channel_name = None;
        let mut user_name = None;

        if let Some(old) = &old {
            if let Some(old_channel_id) = &old.channel_id {
                old_channel_name = Some(old_channel_id.name(ctx).await?);
            }

            if let Some(old_guild_id) = &old.guild_id {
                old_guild_name = Some(match old_guild_id.name(ctx) {
                    Some(name) => name,
                    None => "Unknown".to_string(),
                });
            }
        }

        if let Some(new_channel_id) = &new.channel_id {
            new_channel_name = Some(new_channel_id.name(ctx).await?);
        }

        if let Some(new_guild_id) = &new.guild_id {
            new_guild_name = Some(match new_guild_id.name(ctx) {
                Some(name) => name,
                None => "Unknown".to_string(),
            });
        }

        if let Some(member) = &new.member {
            user_name = Some(member.user.name.clone());
        }

        let new_message_event = WebSocketEvent::VoiceStateUpdate {
            old: old.clone(),
            new: new.clone(),
            old_guild_name,
            old_channel_name,
            new_guild_name,
            new_channel_name,
            user_name,
        };
        let message = serde_json::to_string(&new_message_event).unwrap();
        let peers = { PEER_MAP.lock().unwrap().clone() };

        let message = tungstenite::protocol::Message::Text(message);
        broadcast_message(peers, message).await;
    }

    if let Some(old) = old {
        if old.channel_id != new.channel_id && new.channel_id.is_some() {
            let mut guild_name = String::from("Unknown");
            let mut user_name = String::from("Unknown User");
            let mut old_channel = String::from("Unknown");
            let mut old_channel_id_ = String::from("Unknown");
            let mut new_channel = String::from("Unknown");
            let mut new_channel_id_ = String::from("Unknown");

            if let Some(guild_id) = old.guild_id {
                guild_name = guild_id
                    .name(ctx.clone())
                    .unwrap_or_else(|| guild_name.clone());
            }
            if let Some(member) = new.member {
                user_name = member.user.name;
            }

            if let Some(old_channel_id) = old.channel_id {
                old_channel_id_ = old_channel_id.get().to_string();
                if let Ok(channel_name) = old_channel_id.name(ctx.clone()).await {
                    old_channel = channel_name;
                } else {
                    old_channel = "Unknown".to_owned();
                }
            }

            if let Some(new_channel_id) = new.channel_id {
                new_channel_id_ = new_channel_id.get().to_string();
                if let Ok(channel_name) = new_channel_id.name(ctx.clone()).await {
                    new_channel = channel_name;
                } else {
                    new_channel = "Unknown".to_owned();
                }
            }
            println!(
                "\x1B[32m[{}] {}: {} (ID:{}) -> {} (ID:{})\x1B[0m",
                guild_name, user_name, old_channel, old_channel_id_, new_channel, new_channel_id_
            )
        } else if new.channel_id.is_none() {
            let mut guild_name = String::from("Unknown");
            let mut user_name = String::from("Unknown User");
            let mut old_channel = String::from("Unknown");
            let mut old_channel_id_ = String::from("Unknown");

            if let Some(guild_id) = old.guild_id {
                guild_name = guild_id
                    .name(ctx.clone())
                    .unwrap_or_else(|| guild_name.clone());
            }
            if let Some(member) = new.member {
                user_name = member.user.name;
            }
            if let Some(old_channel_id) = old.channel_id {
                old_channel_id_ = old_channel_id.get().to_string();
                if let Ok(channel_name) = old_channel_id.name(ctx.clone()).await {
                    old_channel = channel_name;
                } else {
                    old_channel = "Unknown".to_owned();
                }
            }
            println!(
                "\x1B[32m[{}] {} left {} (ID:{})\x1B[0m",
                guild_name, user_name, old_channel, old_channel_id_
            )
        } else {
            // mutes, unmutes, deafens, etc are here.
        }
    } else {
        let mut guild_name = String::from("Unknown");
        let mut user_name = String::from("Unknown User");
        let mut new_channel = String::from("Unknown");
        let mut new_channel_id_ = String::from("Unknown");

        if let Some(guild_id) = new.guild_id {
            guild_name = guild_id
                .name(ctx.clone())
                .unwrap_or_else(|| guild_name.clone());
        }
        if let Some(member) = new.member {
            user_name = member.user.name;
        }
        if let Some(new_channel_id) = new.channel_id {
            new_channel_id_ = new_channel_id.get().to_string();
            if let Ok(channel_name) = new_channel_id.name(ctx.clone()).await {
                new_channel = channel_name;
            } else {
                new_channel = "Unknown".to_owned();
            }
        }

        println!(
            "\x1B[32m[{}] {} joined {} (ID:{})\x1B[0m",
            guild_name, user_name, new_channel, new_channel_id_
        );
    }

    Ok(())
}
