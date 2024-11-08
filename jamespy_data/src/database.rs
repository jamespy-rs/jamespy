use sqlx::{postgres::PgPoolOptions, query, Executor, PgPool};
use std::env;

use crate::structs::Error;

use poise::serenity_prelude as serenity;

pub async fn init_data() -> (Database, PgPool) {
    let database_url =
        env::var("DATABASE_URL").expect("No database url found in environment variables!");

    let database = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database!");

    database
        .execute("SET client_encoding TO 'UTF8'")
        .await
        .unwrap();

    sqlx::migrate!("../migrations")
        .run(&database)
        .await
        .expect("Unable to apply migrations!");

    (
        Database {
            db: database.clone(),
        },
        database,
    )
}

/// Custom type.
#[derive(Debug, Clone, sqlx::Type)]
pub enum EmoteUsageType {
    Message,
    ReactionAdd,
    ReactionRemove,
}

pub struct Database {
    pub db: PgPool,
}

impl Database {
    pub async fn insert_user(&self, user_id: serenity::UserId) -> Result<(), Error> {
        query!(
            "INSERT INTO users (user_id)
            VALUES ($1)
            ON CONFLICT (user_id) DO NOTHING",
            user_id.get() as i64
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn insert_channel(
        &self,
        channel_id: serenity::ChannelId,
        guild_id: Option<serenity::GuildId>,
    ) -> Result<(), Error> {
        if let Some(guild_id) = guild_id {
            self.insert_guild(guild_id).await?;
        }

        query!(
            "INSERT INTO channels (channel_id, guild_id)
             VALUES ($1, $2)
             ON CONFLICT (channel_id) DO NOTHING",
            channel_id.get() as i64,
            guild_id.map(|g| g.get() as i64),
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn insert_guild(&self, guild_id: serenity::GuildId) -> Result<(), Error> {
        query!(
            "INSERT INTO guilds (guild_id)
             VALUES ($1)
             ON CONFLICT (guild_id) DO NOTHING",
            guild_id.get() as i64
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
