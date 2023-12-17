mod commands;
use commands::*;
mod database;
mod event_handler;
mod event_handlers;
mod utils;

#[cfg(feature = "websocket")]
mod websocket;

mod config;

use database::init_data;
use database::init_redis_pool;
use poise::serenity_prelude as serenity;
use serde::Deserialize;
use std::sync::Arc;
use std::sync::RwLock;
use std::{env::var, time::Duration};

#[cfg(feature = "websocket")]
use std::env;
#[cfg(feature = "websocket")]
use tokio::net::TcpListener;
#[cfg(feature = "websocket")]
use websocket::{get_peers, handle_connection};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Clone)]
pub struct Data(pub Arc<DataInner>);

impl std::ops::Deref for Data {
    type Target = DataInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct DataInner {
    pub db: database::DbPool,
    pub redis: database::RedisPool,
    pub time_started: std::time::Instant,
    pub jamespy_config: RwLock<config::JamespyConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildConfig {
    pub prefix: Option<String>,
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

#[tokio::main]
async fn main() {
    let db_pool = init_data().await;
    let redis_pool = init_redis_pool().await;

    #[cfg(feature = "websocket")]
    {
        let addr = env::args()
            .nth(1)
            .unwrap_or_else(|| "0.0.0.0:8080".to_string());

        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.expect("Failed to bind");
        println!("Listening on: {}", addr);

        tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                tokio::spawn(handle_connection(get_peers().clone(), stream, addr));
            }
        });
    }
    let data = Data(Arc::new(DataInner {
        db: db_pool,
        redis: redis_pool,
        time_started: std::time::Instant::now(),
        jamespy_config: config::load_config().into(),
    }));

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
            owner::vcstatus::vcstatus(),
            owner::current_user::jamespy(),
            meta::source(),
            meta::about(),
            meta::help(),
            meta::uptime(),
            meta::ping(),
            general::lob::lob(),
            general::lob::reload_lob(),
            general::lob::no_lob(),
            general::lob::new_lob(),
            general::lob::delete_lob(),
            general::lob::total_lobs(),
            general::lob::send_lobs(),
            utility::random::choose(),
            utility::users::guild_flags(),
            utility::users::last_reactions(),
            utility::users::statuses(),
            utility::users::playing(),
            utility::info::role_info(),
            utility::join_tracks::track_join(),
            utility::join_tracks::list_tracked(),
            utility::join_tracks::remove_track(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("-".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(600),
            ))),
            ..Default::default()
        },

        on_error: |error| Box::pin(on_error(error)),

        skip_checks_for_owners: false,
        event_handler: |ctx: &serenity::Context, event: &serenity::FullEvent, framework, data| {
            Box::pin(event_handler::event_handler(
                ctx,
                event.clone(),
                framework,
                data,
            ))
        },
        ..Default::default()
    };

    let framework = poise::Framework::new(options, move |ctx, ready, framework| {
        Box::pin(async move {
            println!("Logged in as {}", ready.user.name);
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            Ok(data)
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
