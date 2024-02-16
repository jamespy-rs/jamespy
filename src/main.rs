#![warn(clippy::pedantic)]
use jamespy_data::structs::{Data, Error};

use poise::serenity_prelude::{self as serenity};
use std::{env::var, sync::Arc, time::Duration};

// TODO: move error handler.
async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        poise::FrameworkError::NotAnOwner { ctx, .. } => {
            let owner_bypass = {
                let data = ctx.data();
                let config = data.config.read();

                if let Some(check) = &config.command_checks {
                    check.owners_all.contains(&ctx.author().id)
                } else {
                    false
                }
            };

            let msg = if owner_bypass {
                "You may have access to most owner commands, but not this one <3"
            } else {
                "Only bot owners can call this command"
            };

            let _ = ctx.say(msg).await;
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

    let mut client = serenity::Client::builder(&token, intents)
        .framework(framework)
        .data(Data::new().await)
        .cache_settings(settings)
        .await
        .unwrap();

    client.start().await.unwrap();
}
