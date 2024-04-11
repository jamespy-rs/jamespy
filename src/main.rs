#![warn(clippy::pedantic)]

mod data;

use poise::serenity_prelude::{self as serenity};
use std::{env::var, sync::Arc, time::Duration};

mod error;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let options = poise::FrameworkOptions {
        commands: jamespy_commands::commands(),
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("-".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(600),
            ))),
            ..Default::default()
        },

        on_error: |error| Box::pin(error::handler(error)),

        command_check: Some(|ctx| Box::pin(jamespy_commands::command_check(ctx))),

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

    let data = data::setup().await;

    let mut client = serenity::Client::builder(&token, intents)
        .framework(framework)
        .voice_manager::<songbird::Songbird>(data.songbird.clone())
        .data(data)
        .cache_settings(settings)
        .await
        .unwrap();

    client.start().await.unwrap();
}
