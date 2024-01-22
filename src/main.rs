#![warn(clippy::pedantic)]
// clippy warns for u64 -> i64 conversions despite this being totally okay in this scenario.
#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::unused_async, // poise checks must be async.
    clippy::unreadable_literal,
    clippy::wildcard_imports,
    clippy::too_many_lines,
)]

use jamespy_data::{
    database::{init_data, init_redis_pool},
    structs::{Data, Error},
};

use poise::serenity_prelude::{self as serenity};
use std::{
    env::var,
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
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

    let config = jamespy_config::JamespyConfig::load_config();

    config.write_config();

    let db_pool = init_data().await;
    let redis_pool = init_redis_pool().await;

    let config = jamespy_config::JamespyConfig::load_config();
    let data = Arc::new(Data {
        has_started: AtomicBool::new(false),
        db: db_pool,
        redis: redis_pool,
        time_started: std::time::Instant::now(),
        reqwest: reqwest::Client::new(),
        config: config.into(),
        dm_activity: dashmap::DashMap::new(),
    });

    let options = poise::FrameworkOptions {
        commands: jamespy_commands::commands(),
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("-".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(600),
            ))),
            ..Default::default()
        },

        on_error: |error| Box::pin(on_error(error)),

        skip_checks_for_owners: false,
        event_handler: |framework, event| Box::pin(jamespy_events::event_handler(framework, event)),
        ..Default::default()
    };

    let framework = poise::Framework::new(options);

    let token = var("JAMESPY_TOKEN").expect("Missing `JAMESPY_TOKEN` env var. Aborting...");
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_PRESENCES;

    let mut settings = serenity::Settings::default();
    settings.max_messages = 350;

    let mut client = serenity::Client::builder(&token, intents)
        .framework(framework)
        .data(data)
        .cache_settings(settings)
        .await
        .unwrap();

    client.start().await.unwrap();
}
