use ::serenity::all::{ChannelId, MessageId};
use dashmap::{DashMap, DashSet};
use parking_lot::Mutex;
use serenity::all::UserId;
use sqlx::{
    postgres::{PgHasArrayType, PgPoolOptions, PgTypeInfo},
    query, Executor, PgPool,
};
use std::{
    collections::{HashMap, HashSet},
    env,
};

use crate::structs::{DmActivity, Error, Names};

use poise::serenity_prelude as serenity;

use std::ops::Deref;

macro_rules! id_wrapper {
    ($wrapper_name:ident, $inner_name:ident) => {
        #[derive(Clone, Copy, PartialEq, Debug)]
        pub struct $wrapper_name(pub $inner_name);

        impl Deref for $wrapper_name {
            type Target = $inner_name;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        /// Convert from i64
        impl From<i64> for $wrapper_name {
            fn from(item: i64) -> Self {
                $wrapper_name($inner_name::new(item as u64))
            }
        }
    };
}

pub async fn init_data() -> Database {
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

    Database {
        db: database,
        owner_overwrites: checks,
        banned_users,
        starboard: Mutex::new(StarboardHandler::default()),
        dm_activity: DashMap::new(),
        names: parking_lot::Mutex::new(Names::new()),
    }
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
    // TODO: return privacy
    pub starboard: Mutex<StarboardHandler>,

    /// Runtime caches for dm activity.
    pub(crate) dm_activity: DashMap<UserId, DmActivity>,
    pub(crate) names: parking_lot::Mutex<Names>,
}

#[derive(Default, Debug)]
pub struct StarboardHandler {
    messages: Vec<StarboardMessage>,
    being_handled: HashSet<MessageId>,
    // message id is the appropriate in messages, the first userid is the author
    // the collection is the reaction users.
    pub reactions_cache: HashMap<MessageId, (UserId, Vec<UserId>)>,
}

#[derive(Clone, Debug, Default)]
pub struct Checks {
    // Users under this will have access to all owner commands.
    pub owners_all: DashSet<UserId>,
    pub owners_single: DashMap<String, HashSet<UserId>>,
}

id_wrapper!(UserIdWrapper, UserId);
id_wrapper!(ChannelIdWrapper, ChannelId);
id_wrapper!(MessageIdWrapper, MessageId);

#[derive(Clone, Debug)]
pub struct StarboardMessage {
    pub id: i32,
    pub user_id: UserIdWrapper,
    pub username: String,
    pub avatar_url: Option<String>,
    pub content: String,
    pub channel_id: ChannelIdWrapper,
    pub message_id: MessageIdWrapper,
    pub attachment_urls: Vec<String>,
    pub star_count: i16,
    pub starboard_status: StarboardStatus,
    pub starboard_message_id: MessageIdWrapper,
    pub starboard_message_channel: ChannelIdWrapper,
}

#[derive(Debug, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "starboard_status")]
pub enum StarboardStatus {
    InReview,
    Accepted,
    Denied,
}

impl PgHasArrayType for StarboardStatus {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("starboard_status[]")
    }
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

    pub async fn get_starboard_msg(&self, msg_id: MessageId) -> Result<StarboardMessage, Error> {
        if let Some(starboard) = self
            .starboard
            .lock()
            .messages
            .iter()
            .find(|s| *s.message_id == msg_id)
            .cloned()
        {
            return Ok(starboard);
        }

        let starboard = self.get_starboard_msg_(msg_id).await?;

        self.starboard.lock().messages.push(starboard.clone());

        Ok(starboard)
    }

    async fn get_starboard_msg_(&self, msg_id: MessageId) -> Result<StarboardMessage, sqlx::Error> {
        sqlx::query_as!(StarboardMessage,
        r#"
        SELECT id, user_id, username, avatar_url, content, channel_id, message_id, attachment_urls, star_count, starboard_message_id, starboard_message_channel, starboard_status as "starboard_status: StarboardStatus"
        FROM starboard
        WHERE message_id = $1
        "#, msg_id.get() as i64)
            .fetch_one(&self.db)
            .await
    }

    pub async fn update_star_count(&self, id: i32, count: i16) -> Result<(), sqlx::Error> {
        {
            let mut starboard = self.starboard.lock();
            let entry = starboard.messages.iter_mut().find(|s| s.id == id);
            if let Some(entry) = entry {
                entry.star_count = count;
            }
        };

        query!(
            "UPDATE starboard SET star_count = $1 WHERE id = $2",
            count,
            id,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Check if a starboard is being handled, and if its not, handle it.
    ///
    /// returns if its already being handled.
    pub fn handle_starboard(&self, message_id: MessageId) -> bool {
        !self.starboard.lock().being_handled.insert(message_id)
    }

    /// Remove the safety check for a starboard being handled.
    pub fn stop_handle_starboard(&self, message_id: &MessageId) {
        self.starboard.lock().being_handled.remove(message_id);
    }

    pub async fn insert_starboard_msg(
        &self,
        m: StarboardMessage,
        guild_id: Option<serenity::GuildId>,
    ) -> Result<(), sqlx::Error> {
        let m_id = *m.message_id;
        let _ = self.insert_starboard_msg_(m, guild_id).await;
        self.stop_handle_starboard(&m_id);

        Ok(())
    }

    async fn insert_starboard_msg_(
        &self,
        mut m: StarboardMessage,
        guild_id: Option<serenity::GuildId>,
    ) -> Result<(), Error> {
        if let Some(g_id) = guild_id {
            self.insert_guild(g_id).await?;
        }

        self.insert_channel(*m.channel_id, guild_id).await?;
        self.insert_channel(*m.starboard_message_channel, guild_id)
            .await?;
        self.insert_user(*m.user_id).await?;

        let val = match sqlx::query!(
            r#"
                INSERT INTO starboard (
                    user_id, username, avatar_url, content, channel_id, message_id,
                    attachment_urls, star_count, starboard_status,
                    starboard_message_id, starboard_message_channel
                )
                VALUES (
                    $1, $2, $3, $4, $5, $6,
                    $7, $8, $9,
                    $10, $11
                ) RETURNING id
                "#,
            m.user_id.get() as i64,
            m.username,
            m.avatar_url,
            m.content,
            m.channel_id.get() as i64,
            m.message_id.get() as i64,
            &m.attachment_urls,
            m.star_count,
            m.starboard_status as _,
            m.starboard_message_id.get() as i64,
            m.starboard_message_channel.get() as i64,
        )
        .fetch_one(&self.db)
        .await
        {
            Ok(result) => result,
            Err(e) => {
                println!("SQL query failed: {e:?}");
                return Err(e.into());
            }
        };

        m.id = val.id;

        let mut lock = self.starboard.lock();
        let m_id = *m.message_id;

        lock.messages.push(m);
        lock.being_handled.remove(&m_id);

        Ok(())
    }

    pub async fn get_starboard_msg_by_starboard_id(
        &self,
        starboard_msg_id: MessageId,
    ) -> Result<StarboardMessage, Error> {
        if let Some(starboard) = self
            .starboard
            .lock()
            .messages
            .iter()
            .find(|s| *s.starboard_message_id == starboard_msg_id)
            .cloned()
        {
            return Ok(starboard);
        }

        let starboard = self
            .get_starboard_msg_by_starboard_id_(starboard_msg_id)
            .await?;

        self.starboard.lock().messages.push(starboard.clone());

        Ok(starboard)
    }

    async fn get_starboard_msg_by_starboard_id_(
        &self,
        starboard_msg_id: MessageId,
    ) -> Result<StarboardMessage, sqlx::Error> {
        sqlx::query_as!(StarboardMessage,
        r#"
        SELECT id, user_id, username, avatar_url, content, channel_id, message_id, attachment_urls, star_count, starboard_message_id, starboard_message_channel, starboard_status as "starboard_status: StarboardStatus"
        FROM starboard
        WHERE starboard_message_id = $1
        "#, starboard_msg_id.get() as i64)
            .fetch_one(&self.db)
            .await
    }

    pub async fn approve_starboard(
        &self,
        starboard_message_id: MessageId,
        new_message_id: MessageId,
        new_channel_id: ChannelId,
    ) -> Result<(), Error> {
        let status = StarboardStatus::Accepted;

        query!(
            "UPDATE starboard SET starboard_status = $1, starboard_message_id = $2, \
             starboard_message_channel = $3 WHERE starboard_message_id = $4",
            status as _,
            new_message_id.get() as i64,
            new_channel_id.get() as i64,
            starboard_message_id.get() as i64,
        )
        .execute(&self.db)
        .await?;

        let mut lock = self.starboard.lock();
        let m = lock
            .messages
            .iter_mut()
            .find(|m| *m.starboard_message_id == starboard_message_id);

        if let Some(m) = m {
            m.starboard_message_channel = ChannelIdWrapper(new_channel_id);
            m.starboard_message_id = MessageIdWrapper(new_message_id);
        }

        Ok(())
    }

    pub async fn deny_starboard(&self, starboard_message_id: MessageId) -> Result<(), Error> {
        let status = StarboardStatus::Denied;

        query!(
            "UPDATE starboard SET starboard_status = $1 WHERE starboard_message_id = $2",
            status as _,
            starboard_message_id.get() as i64,
        )
        .execute(&self.db)
        .await?;

        let mut lock = self.starboard.lock();
        if let Some(index) = lock
            .messages
            .iter()
            .position(|m| *m.starboard_message_id == starboard_message_id)
        {
            lock.messages.remove(index);
        }

        Ok(())
    }

    // temporary function to give access to the inner command overwrites while i figure something out.
    #[must_use]
    pub fn inner_overwrites(&self) -> &Checks {
        &self.owner_overwrites
    }
}
