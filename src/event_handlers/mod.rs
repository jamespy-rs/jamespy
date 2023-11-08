#[cfg(feature = "websocket")]
use futures_util::SinkExt;
use poise::serenity_prelude::{MessageId, ChannelId, GuildId, GuildChannel, Channel, PartialGuildChannel, Guild, Member, User, Reaction, GuildMemberUpdateEvent, VoiceState};
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
        guild_name: Option<String>,
        channel_name: Option<String>
    },
    MessageDelete {
        channel_id: ChannelId,
        deleted_message_id: MessageId,
        guild_id: Option<GuildId>,
        message: Option<Message>,
        guild_name: String,
        channel_name: String
    },
    ChannelCreate {
        channel: GuildChannel
    },
    ChannelUpdate {
        old: Option<Channel>,
        new: Channel,
    },
    ChannelDelete {
        channel: GuildChannel,
    },
    ThreadCreate {
        thread: GuildChannel
    },
    ThreadUpdate {
        old: Option<GuildChannel>,
        new: GuildChannel,
    },
    ThreadDelete {
        thread: PartialGuildChannel,
        full_thread_data: Option<GuildChannel>,
    },
    GuildCreate {
        guild: Guild,
        is_new: Option<bool>,
    },
    GuildMemberAddition {
        new_member: Member,
        guild_name: String,
    },
    GuildMemberRemoval {
        guild_id: GuildId,
        user: User,
        guild_name: String,
    },
    ReactionAdd {
        add_reaction: Reaction,
        user_name: String,
        guild_name: String,
        channel_name: String
    },
    ReactionRemove {
        removed_reaction: Reaction,
        user_name: String,
        guild_name: String,
        channel_name: String
    },
    GuildMemberUpdate {
        old_if_available: Option<Member>,
        new: Option<Member>,
        event: GuildMemberUpdateEvent,
        guild_name: String,
    },
    VoiceStateUpdate {
        old: Option<VoiceState>,
        new: VoiceState,
        old_guild_name: String,
        old_channel_name: String,
        new_guild_name: String,
        new_channel_name: String
    }
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
