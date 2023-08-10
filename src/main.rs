mod commands;
use commands::*;
mod database;
mod event_handler;
//use sqlx::query;

use poise::serenity_prelude as serenity;
use std::{env::var, time::Duration};
use database::init_data;
use database::init_redis_pool;

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
        poise::FrameworkError::Command { error, ctx } => {
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

    let options = poise::FrameworkOptions {
        commands: vec![
            register(),
            meta::shutdown(),
            meta::source(),
            meta::about(),
            meta::help(),
            meta::uptime(),
            general::lob::lob(),
            utility::snippets::setsnippet(),
            utility::snippets::snippet(),
            utility::snippets::listsnippets(),
            utility::snippets::removesnippet(),
            utility::tracking::trackuser(),
            utility::tracking::untrackuser(),
            utility::tracking::trackedusers(),
            utility::random::choose(),
            ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("-".into()),
            edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
            ..Default::default()
        },

        on_error: |error| Box::pin(on_error(error)),

        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },

        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },

        skip_checks_for_owners: false,
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler::event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    poise::Framework::builder()
        .token(var("JAMESPY_TOKEN").expect("JAMESPY_TOKEN is not set. aborting..."))
        .setup(move |_ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    db: db_pool.clone(),
                    redis: redis_pool.clone(),
                    time_started: std::time::Instant::now(),
                })
            })
        })
        .options(options)
        .intents(
            serenity::GatewayIntents::non_privileged() |
            serenity::GatewayIntents::MESSAGE_CONTENT |
            serenity::GatewayIntents::GUILD_MEMBERS |
            serenity::GatewayIntents::GUILD_PRESENCES
        )
        .run()
        .await
        .unwrap();
}
