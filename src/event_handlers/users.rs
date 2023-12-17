#[cfg(feature = "websocket")]
use crate::{
    event_handlers::{broadcast_message, WebSocketEvent},
    websocket::PEER_MAP,
};
use poise::serenity_prelude::{self as serenity, GuildMemberUpdateEvent, Member};
#[cfg(feature = "websocket")]
use tokio_tungstenite::tungstenite;

use crate::Error;

pub async fn guild_member_update(
    ctx: &serenity::Context,
    old_if_available: Option<Member>,
    new: Option<Member>,
    event: GuildMemberUpdateEvent,
) -> Result<(), Error> {
    let guild_id = event.guild_id;
    let guild_name = if guild_id == 1 {
        "None".to_owned()
    } else {
        match guild_id.name(ctx.clone()) {
            Some(name) => name,
            None => "Unknown".to_owned(),
        }
    };

    #[cfg(feature = "websocket")]
    {
        let new_message_event = WebSocketEvent::GuildMemberUpdate {
            old_if_available: old_if_available.clone(),
            new: new.clone(),
            event: event.clone(),
            guild_name: guild_name.clone(),
        };
        let message = serde_json::to_string(&new_message_event).unwrap();
        let peers = { PEER_MAP.lock().unwrap().clone() };

        let message = tungstenite::protocol::Message::Text(message);
        broadcast_message(peers, message).await;
    }
    if let Some(old_member) = old_if_available {
        if let Some(new_member) = new {
            let old_nickname = old_member.nick.as_deref().unwrap_or("None");
            let new_nickname = new_member.nick.as_deref().unwrap_or("None");

            if old_nickname != new_nickname {
                println!(
                    "\x1B[92m[{}] Nickname change: {}: {} -> {} (ID:{})\x1B[0m",
                    guild_name,
                    new_member.user.name,
                    old_nickname,
                    new_nickname,
                    new_member.user.id
                );
            };

            if old_member.user.name != new_member.user.name {
                println!(
                    "\x1B[92mUsername change: {} -> {} (ID:{})\x1B[0m",
                    old_member.user.name, new_member.user.name, new_member.user.id
                );
            }
            if old_member.user.global_name != new_member.user.global_name {
                println!(
                    "\x1B[92mDisplay name change: {}: {} -> {} (ID:{})\x1B[0m",
                    old_member.user.name,
                    old_member
                        .clone()
                        .user
                        .global_name
                        .unwrap_or("None".to_owned()),
                    new_member
                        .clone()
                        .user
                        .global_name
                        .unwrap_or("None".to_owned()),
                    new_member.user.id
                );
            }
        }
    }

    Ok(())
}
