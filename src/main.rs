#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::unused_async,
    clippy::unreadable_literal,
    clippy::wildcard_imports,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::module_name_repetitions
)]

mod commands;
mod database;
mod event_handler;
mod event_handlers;
mod utils;

mod config;

use dashmap::DashMap;
use database::init_data;
use database::init_redis_pool;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::UserId;
use serde::Deserialize;
use std::sync::Arc;
use std::sync::RwLock;
use std::{env::var, time::Duration};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
pub type Command = poise::Command<Data, Error>;

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
    pub dm_activity: DashMap<UserId, serenity::Timestamp>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GuildConfig {
    pub prefix: Option<String>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {error:?}"),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {e}");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db_pool = init_data().await;
    let redis_pool = init_redis_pool().await;

    let data = Data(Arc::new(DataInner {
        db: db_pool,
        redis: redis_pool,
        time_started: std::time::Instant::now(),
        jamespy_config: config::load_config().into(),
        dm_activity: DashMap::new(),
    }));

    let options = poise::FrameworkOptions {
        commands: commands::commands(),
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
        #[cfg(feature = "castle")]
        let ctx_clone = ctx.clone();
        #[cfg(feature = "castle")]
        let data_clone = data.clone();

        #[cfg(feature = "castle")]
        tokio::spawn(async move {
            let mut interval: tokio::time::Interval =
                tokio::time::interval(std::time::Duration::from_secs(60 * 60));
            loop {
                // TODO: eventually move this to its own function.
                interval.tick().await;
                let _ = crate::utils::tasks::check_space(&ctx_clone, &data_clone).await;
                let _ = crate::utils::tasks::update_stats(&ctx_clone, &data_clone).await;
            }
        });

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
