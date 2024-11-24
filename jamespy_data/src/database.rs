use dashmap::{DashMap, DashSet};
use serenity::all::UserId;
use sqlx::{
    postgres::{PgHasArrayType, PgPoolOptions, PgTypeInfo},
    query, Executor, PgPool,
};
use std::{collections::HashSet, env};

use crate::structs::{DmActivity, Error, Names};

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

    let user_ids = query!("SELECT user_id FROM banned_users")
        .fetch_all(&database)
        .await
        .unwrap();

    let banned_users = user_ids
        .iter()
        .map(|r| UserId::new(r.user_id as u64))
        .collect::<DashSet<UserId>>();

    let db_checks = query!("SELECT * FROM owner_access")
        .fetch_all(&database)
        .await
        .unwrap();

    let checks = Checks::default();

    for check in db_checks {
        if let Some(command_name) = check.command_name {
            let mut entry = checks
                .owners_single
                .entry(command_name)
                .or_insert_with(HashSet::new);
            entry.insert(UserId::new(check.user_id as u64));
        } else {
            checks.owners_all.insert(UserId::new(check.user_id as u64));
        }
    }

    (
        Database {
            db: database.clone(),
            owner_overwrites: checks,
            banned_users,
            dm_activity: DashMap::new(),
            names: parking_lot::Mutex::new(Names::new()),
        },
        database,
    )
}

/// Custom type.
#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "emoteusagetype")]
pub enum EmoteUsageType {
    Message,
    ReactionAdd,
    ReactionRemove,
}

impl PgHasArrayType for EmoteUsageType {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("emoteusagetype[]")
    }
}

pub struct Database {
    pub db: PgPool,
    banned_users: DashSet<UserId>,
    owner_overwrites: Checks,
    /// Runtime caches for dm activity.
    pub(crate) dm_activity: DashMap<UserId, DmActivity>,
    pub(crate) names: parking_lot::Mutex<Names>,
}

#[derive(Clone, Debug, Default)]
pub struct Checks {
    // Users under this will have access to all owner commands.
    pub owners_all: DashSet<UserId>,
    pub owners_single: DashMap<String, HashSet<UserId>>,
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

    /// Checks if a user is banned from using commands.
    #[must_use]
    pub fn is_banned(&self, user_id: &UserId) -> bool {
        self.banned_users.contains(user_id)
    }

    /// Sets the user banned/unbanned from the bot, returning the old status.
    pub async fn set_banned(&self, user_id: UserId, banned: bool) -> Result<bool, Error> {
        if banned == self.banned_users.contains(&user_id) {
            return Ok(banned);
        }

        let old_status = self.banned_users.contains(&user_id);

        if banned {
            self.banned_users.insert(user_id);
            self.insert_user(user_id).await?;
            query!(
                "INSERT INTO banned_users (user_id) VALUES ($1)",
                user_id.get() as i64
            )
            .execute(&self.db)
            .await?;
        } else {
            self.banned_users.remove(&user_id);
            query!(
                "DELETE FROM banned_users WHERE user_id = $1",
                user_id.get() as i64
            )
            .execute(&self.db)
            .await?;
        }

        Ok(old_status)
    }

    /// To be called in a function that uses the owner check.
    #[must_use]
    pub fn check_owner(&self, user_id: UserId, command: &str) -> bool {
        if self.owner_overwrites.owners_all.get(&user_id).is_some() {
            return true;
        }

        if let Some(data) = self.owner_overwrites.owners_single.get(command) {
            if data.value().contains(&user_id) {
                return true;
            }
        }

        false
    }

    /// Sets the user as an owner for every owner command or specifically one owner command
    /// returning the old value.
    pub async fn set_owner(&self, user_id: UserId, command: Option<&str>) -> Result<bool, Error> {
        let Some(command) = command else {
            if self.owner_overwrites.owners_all.contains(&user_id) {
                return Ok(true);
            };

            self.insert_user(user_id).await?;
            query!(
                "INSERT INTO owner_access (user_id) VALUES ($1)",
                user_id.get() as i64
            )
            .execute(&self.db)
            .await?;

            self.owner_overwrites.owners_all.insert(user_id);
            return Ok(false);
        };

        {
            if let Some(cmd_cache) = self.owner_overwrites.owners_single.get(command) {
                if cmd_cache.contains(&user_id) {
                    return Ok(true);
                }
            }
        }

        self.insert_user(user_id).await?;
        query!(
            "INSERT INTO owner_access (user_id, command_name) VALUES ($1, $2)",
            user_id.get() as i64,
            command
        )
        .execute(&self.db)
        .await?;

        self.owner_overwrites
            .owners_single
            .entry(command.to_string())
            .or_default()
            .insert(user_id);
        Ok(false)
    }

    /// Removes the user as an owner for every owner command or specifically one owner command
    /// returning the old value.
    pub async fn remove_owner(
        &self,
        user_id: UserId,
        command: Option<&str>,
    ) -> Result<bool, Error> {
        let Some(command) = command else {
            if !self.owner_overwrites.owners_all.contains(&user_id) {
                return Ok(false);
            }
            query!(
                "DELETE FROM owner_access WHERE user_id = $1",
                user_id.get() as i64
            )
            .execute(&self.db)
            .await?;
            self.owner_overwrites.owners_all.remove(&user_id);

            return Ok(true);
        };

        let is_owner = {
            if let Some(cmd_cache) = self.owner_overwrites.owners_single.get(command) {
                cmd_cache.contains(&user_id)
            } else {
                false
            }
        };

        if !is_owner {
            return Ok(false);
        }

        query!(
            "DELETE FROM owner_access WHERE user_id = $1 AND command_name = $2",
            user_id.get() as i64,
            command
        )
        .execute(&self.db)
        .await?;

        let mut should_remove_entry = false;
        if let Some(mut cmd_cache) = self.owner_overwrites.owners_single.get_mut(command) {
            cmd_cache.remove(&user_id);
            if cmd_cache.is_empty() {
                should_remove_entry = true;
            }
        }

        // Remove the entry if it is now empty
        if should_remove_entry {
            self.owner_overwrites.owners_single.remove(command);
        }

        Ok(true)
    }

    // temporary function to give access to the inner command overwrites while i figure something out.
    #[must_use]
    pub fn inner_overwrites(&self) -> &Checks {
        &self.owner_overwrites
    }
}
