#[cfg(feature = "websocket")]
use futures_util::SinkExt;

#[cfg(feature = "websocket")]
use poise::serenity_prelude::{
    Channel, ChannelId, Guild, GuildChannel, GuildId, GuildMemberUpdateEvent, Member, MessageId,
    PartialGuildChannel, Reaction, User, VoiceState,
};
#[cfg(feature = "websocket")]
use poise::serenity_prelude::{Message, MessageUpdateEvent};
#[cfg(feature = "websocket")]
use serde::Serialize;

pub mod channels;
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
        channel_name: Option<String>,
    },
    MessageDelete {
        channel_id: ChannelId,
        deleted_message_id: MessageId,
        guild_id: Option<GuildId>,
        message: Option<Message>,
        guild_name: String,
        channel_name: String,
    },
    ChannelCreate {
        channel: GuildChannel,
        guild_name: String,
    },
    ChannelUpdate {
        old: Option<GuildChannel>,
        new: GuildChannel,
        guild_name: String,
    },
    ChannelDelete {
        channel: GuildChannel,
        guild_name: String,
    },
    ThreadCreate {
        thread: GuildChannel,
        guild_name: String,
    },
    ThreadUpdate {
        old: Option<GuildChannel>,
        new: GuildChannel,
        parent_channel: Option<Channel>,
        guild_name: String,
    },
    ThreadDelete {
        thread: PartialGuildChannel,
        full_thread_data: Option<GuildChannel>,
        guild_name: String,
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
        channel_name: String,
    },
    ReactionRemove {
        removed_reaction: Reaction,
        user_name: String,
        guild_name: String,
        channel_name: String,
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
        old_guild_name: Option<String>,
        old_channel_name: Option<String>,
        new_guild_name: Option<String>,
        new_channel_name: Option<String>,
        user_name: Option<String>,
    },
}

#[cfg(feature = "websocket")]
use futures_channel::mpsc::UnboundedSender;
#[cfg(feature = "websocket")]
use std::{collections::HashMap, net::SocketAddr};

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
