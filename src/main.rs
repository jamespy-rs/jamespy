mod commands;
use commands::*;


use poise::serenity_prelude as serenity;
use std::{env::var, time::Duration};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn event_handler(
    _ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _ctx_poise: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        _ => (),
    }

    Ok(())
}

pub struct Data {}

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
    let options = poise::FrameworkOptions {
        commands: vec![
            register(),
            meta::shutdown(),
            meta::source(),
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
            Box::pin(event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    poise::Framework::builder()
        .token(var("JAMESPY_TOKEN").expect("JAMESPY_TOKEN is not set. aborting..."))
        .setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }))
        .options(options)
        .intents(
            serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
        )
        .run()
        .await
        .unwrap();
}
