mod commands;
use commands::*;
mod database;
mod event_handler;
mod event_handlers;
mod utils;
mod websocket;

mod config;

use ::serenity::futures::channel::mpsc::UnboundedSender;
use database::init_data;
use database::init_redis_pool;
use poise::serenity_prelude as serenity;
use std::net::SocketAddr;
use tokio::net::TcpListener;
//use websocket::handle_connection;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::{env::var, time::Duration};

use std::env;

use futures_channel::mpsc::unbounded;
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

use lazy_static::lazy_static;
lazy_static! {
    pub static ref PEER_MAP: PeerMap = Arc::new(Mutex::new(HashMap::new()));
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    pub db: database::DbPool,
    pub redis: database::RedisPool,
    time_started: std::time::Instant,
}

#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;

    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        println!(
            "Received a message from {}: {}",
            addr,
            msg.to_text().unwrap()
        );
        let peers = peer_map.lock().unwrap();

        // We want to broadcast the message to everyone except ourselves.
        let broadcast_recipients = peers
            .iter()
            .filter(|(peer_addr, _)| peer_addr != &&addr)
            .map(|(_, ws_sink)| ws_sink);

        for recp in broadcast_recipients {
            recp.unbounded_send(msg.clone()).unwrap();
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
}

#[tokio::main]
async fn main() {
    let db_pool = init_data().await;
    let redis_pool = init_redis_pool().await;
    config::load_config();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    tokio::spawn(async move {
        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(handle_connection(PEER_MAP.clone(), stream, addr));
        }
    });

    let options = poise::FrameworkOptions {
        commands: vec![
            register(),
            owner::other::shutdown(),
            owner::other::say(),
            owner::other::dm(),
            owner::other::react(),
            owner::cache::cached_users_raw(),
            owner::cache::cached_users(),
            owner::cache::max_messages(),
            owner::cache::cache_stats(),
            owner::presence::status(),
            owner::presence::reset_presence(),
            owner::presence::set_activity(),
            owner::database::dbstats(),
            owner::database::sql(),
            owner::cache::guild_message_cache(),
            owner::lists::update_lists(),
            owner::lists::unload_lists(),
            owner::glow::glow(),
            owner::vcstatus::vcstatus(),
            owner::current_user::jamespy(),
            meta::source(),
            meta::about(),
            meta::help(),
            meta::uptime(),
            meta::ping(),
            meta::send(),
            general::lob::lob(),
            general::lob::reload_lob(),
            general::lob::no_lob(),
            general::lob::new_lob(),
            general::lob::delete_lob(),
            general::lob::total_lobs(),
            general::lob::send_lobs(),
            utility::snippets::set_snippet(),
            utility::snippets::snippet(),
            utility::snippets::list_snippets(),
            utility::snippets::remove_snippet(),
            utility::random::choose(),
            utility::users::guild_flags(),
            utility::users::last_reactions(),
            utility::users::statuses(),
            utility::users::playing(),
            utility::info::role_info(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("-".into()),
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(600))),
            ..Default::default()
        },

        on_error: |error| Box::pin(on_error(error)),

        skip_checks_for_owners: false,
        event_handler: |event: &serenity::FullEvent, framework, data| {
            Box::pin(event_handler::event_handler(event.clone(), framework, data))
        },
        ..Default::default()
    };

    let framework = poise::Framework::new(options, move |ctx, ready, framework| {
        Box::pin(async move {
            println!("Logged in as {}", ready.user.name);
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            Ok(Data {
                db: db_pool.clone(),
                redis: redis_pool.clone(),
                time_started: std::time::Instant::now(),
            })
        })
    });

    let token = var("JAMESPY_TOKEN").expect("Missing `JAMESPY_TOKEN` env var. Aborting...");
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_PRESENCES;

    let mut client = serenity::Client::builder(token, intents)
        .framework(framework)
        .await
        .unwrap();

    client.start().await.unwrap();
}
