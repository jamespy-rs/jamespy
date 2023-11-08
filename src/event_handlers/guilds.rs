#[cfg(feature = "websocket")]
use crate::event_handlers::broadcast_message;
#[cfg(feature = "websocket")]
use crate::websocket::PEER_MAP;

#[cfg(feature = "websocket")]
use crate::event_handlers::WebSocketEvent;
#[cfg(feature = "websocket")]
use tokio_tungstenite::tungstenite;

use crate::Error;
use crate::utils::misc::get_guild_name;
use poise::serenity_prelude::{self as serenity, Guild, GuildId, Member, User};


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
