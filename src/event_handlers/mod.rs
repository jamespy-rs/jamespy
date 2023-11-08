#[cfg(feature = "websocket")]
use futures_util::SinkExt;
#[cfg(feature = "websocket")]
use poise::serenity_prelude::{Message, MessageUpdateEvent};
#[cfg(feature = "websocket")]
use serde::Serialize;

pub mod channels;
pub mod glow;
pub mod guilds;
pub mod messages;
pub mod misc;
pub mod reactions;
pub mod users;
pub mod voice;


#[cfg(feature = "websocket")]
#[derive(Serialize, Debug)]
enum WebSocketEvent {
    NewMessage {
        message: Message,
        guild_name: String,
        channel_name: String,
    },
    MessageEdit {
        old_if_available: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
        guild_name : Option<String>,
        channel_name: Option<String>
    },
}

#[cfg(feature = "websocket")]
use std::{net::SocketAddr, collections::HashMap};
#[cfg(feature = "websocket")]
use futures_channel::mpsc::UnboundedSender;


#[cfg(feature = "websocket")]
pub async fn broadcast_message(
    peers: HashMap<SocketAddr, UnboundedSender<tokio_tungstenite::tungstenite::Message>>,
    message: tokio_tungstenite::tungstenite::Message,
) {

    for (_, mut ws_sink) in peers.iter() {
        let cloned_msg: tokio_tungstenite::tungstenite::Message = message.clone();

        if let Err(err) = ws_sink.send(cloned_msg).await {
            println!("Error sending message to peer: {:?}", err);
        }
    }
}
