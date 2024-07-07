use arrayvec::ArrayVec;
use serde::Deserialize;
use serenity::all::{ChannelId, Context, CreateAllowedMentions, CreateMessage, Http, MessageReference};
use futures_util::StreamExt;
use std::{net::SocketAddr, sync::{Arc, Mutex}};
use std::collections::HashMap;


use futures_channel::mpsc::{unbounded, UnboundedSender};

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[derive(Deserialize)]
struct WebSocketMessage {
    #[serde(rename = "type")]
    message_type: String,
    message: String
}


#[derive(Deserialize)]
struct DiscordMessage {
    channel_id: ChannelId,
    content: String,
    reply_options: Option<ReplyOptions>
}

enum MessageType {
    SendDiscordMessage(DiscordMessage)
}

impl MessageType {
    fn from_websocket_message(ws_msg: &WebSocketMessage) -> serde_json::Result<Option<MessageType>> {
        match ws_msg.message_type.as_str() {
            "SendDiscordMessage" => {
                let discord_msg: DiscordMessage = serde_json::from_str(&ws_msg.message)?;
                Ok(Some(MessageType::SendDiscordMessage(discord_msg)))
            }
            _ => Ok(None),
        }
    }
}




/* "messageReference": {
    "channel_id": "927380498215481395",
    "message_id": "1258934898711072809"
},
"allowedMentions": {
    "parse": [
        "users",
        "roles",
        "everyone"
    ],
    "replied_user": false
    }
    use serde::Deserializer;
} */

#[derive(Deserialize, Clone)]
struct ReplyOptions {
    #[serde(rename = "messageReference")]
    message_reference: MessageReference,
    #[serde(rename = "allowedMentions")]
    allowed_mentions: Option<AllowedMentions>
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
enum ParseValue {
    Everyone,
    Users,
    Roles,
}

// a copy of the internal structure, just public.
#[derive(Deserialize, Clone)]
struct AllowedMentions {
    // mutally exclusive to the below.
    parse: ArrayVec<ParseValue, 3>,
    //users: Vec<UserId>,
    //roles: Vec<RoleId>,
    replied_user: Option<bool>,
}

impl<'a> From<AllowedMentions> for CreateAllowedMentions<'a> {
    fn from(val: AllowedMentions) -> Self {
        let mut builder = CreateAllowedMentions::new().replied_user(val.replied_user.unwrap_or(false));

        for p in val.parse {
            match p {
                ParseValue::Everyone => builder = builder.everyone(true),
                ParseValue::Users => builder = builder.all_users(true),
                ParseValue::Roles => builder = builder.all_roles(true),
            }
        }

        // TODO: include users and roles later.

        builder
    }
}

pub async fn start_server(ctx: Context) {
    let addr = "127.0.0.1:8486";

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {addr}");

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, addr, ctx.clone()));
    }

}

async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr, ctx: Context) {
    println!("Incoming TCP connection from: {addr}");

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {addr}");

    // Insert the write part of this peer to the peer map.
    let (tx, _) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx);

    let (_, mut incoming) = ws_stream.split();

    while let Some(message) = incoming.next().await {
        let Ok(message) = message else {
            continue
        };

        let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(message.to_text().unwrap()) else {
            continue
        };

        match MessageType::from_websocket_message(&ws_msg) {
            Err(err) => { println!("Error when parsing websocket message: {err}"); continue }
            Ok(None) => continue,
            Ok(Some(msg_type)) => {
                match msg_type {
                    MessageType::SendDiscordMessage(msg) => handle_send_message(&ctx.http, msg).await,
                }
            }
        }
    }


    println!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
}


async fn handle_send_message(http: &Http, msg: DiscordMessage) {
    // strip allowed prefixes.
    let Some(content) = msg.content.strip_prefix("j!send ") else {
        return
    };

    let mut builder = CreateMessage::new().content(content);

    if let Some(reply_options) = msg.reply_options {
        builder = builder
            .reference_message(reply_options.message_reference);

        if let Some(x) = reply_options.allowed_mentions {
            builder = builder.allowed_mentions(x.into())
        }
    }

    let _ = msg.channel_id.send_message(http, builder).await;
}
