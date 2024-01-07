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
use tracing::warn;

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
    pub dm_activity: DashMap<UserId, (i64, Option<i64>, i16)>,
}

#[allow(clippy::missing_panics_doc)]
impl Data {
    pub async fn get_activity_check(&self, user_id: UserId) -> Option<(i64, Option<i64>, i16)> {
        let cached = self.dm_activity.get(&user_id);

        if let Some(cached) = cached {
            Some(*cached)
        } else {
            self._get_activity_check_psql(user_id).await
        }
    }

    async fn _get_activity_check_psql(&self, user_id: UserId) -> Option<(i64, Option<i64>, i16)> {
        let result = sqlx::query!(
            "SELECT last_announced, until, count FROM dm_activity WHERE user_id = $1",
            i64::from(user_id)
        )
        .fetch_one(&self.db)
        .await;

        match result {
            Ok(record) => Some((
                record.last_announced.unwrap(),
                record.until,
                record.count.unwrap(),
            )),
            Err(err) => {
                if let sqlx::Error::RowNotFound = err {
                    None
                } else {
                    warn!("Error when attempting to find row: {err}");
                    None
                }
            }
        }
    }

    pub async fn updated_no_announce(
        &self,
        user_id: UserId,
        announced: i64,
        until: i64,
        count: i16,
    ) {
        // count will have already been incremented.
        let _ = sqlx::query!(
            "UPDATE dm_activity SET until = $1, count = $2 WHERE user_id = $3",
            until,
            count,
            i64::from(user_id)
        )
        .execute(&self.db)
        .await;

        self.update_user_cache(user_id, announced, until, count);
    }

    pub async fn new_or_announced(
        &self,
        user_id: UserId,
        announced: i64,
        until: i64,
        count: Option<i16>,
    ) {
        // If this is an update, count will have already been supplied and incremented.
        let _ = sqlx::query!(
            "INSERT INTO dm_activity (user_id, last_announced, until, count)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id) DO UPDATE
            SET last_announced = $2, until = $3, count = $4",
            i64::from(user_id),
            announced,
            until,
            count.unwrap_or(0)
        )
        .execute(&self.db)
        .await;

        self.update_user_cache(user_id, announced, until, count.unwrap_or(0));
    }

    pub fn remove_dm_activity_cache(&self, user_id: UserId) {
        self.dm_activity.remove(&user_id);
    }

    fn update_user_cache(&self, user_id: UserId, announced: i64, until: i64, count: i16) {
        self.dm_activity
            .insert(user_id, (announced, Some(until), count));
    }

    pub async fn remove_until(&self, user_id: UserId) {
        self.remove_dm_activity_cache(user_id);
        let _ = sqlx::query!(
            "UPDATE dm_activity SET until = NULL WHERE user_id = $1",
            i64::from(user_id)
        )
        .execute(&self.db)
        .await;
    }
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
        let ctx_clone = ctx.clone();
        let data_clone = data.clone();


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
