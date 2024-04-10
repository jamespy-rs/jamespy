use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;

use chrono::{NaiveDateTime, Utc};
use poise::serenity_prelude::{GuildId, User, UserId};
use sqlx::query;

use std::{
    collections::VecDeque,
    sync::{atomic::AtomicBool, Arc},
};

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type Command = poise::Command<Data, Error>;

pub struct Data {
    pub has_started: AtomicBool,
    pub db: sqlx::PgPool,
    pub songbird: Arc<songbird::Songbird>,
    pub redis: crate::database::RedisPool,
    pub time_started: std::time::Instant,
    pub reqwest: reqwest::Client,
    pub config: RwLock<jamespy_config::JamespyConfig>,
    pub dm_activity: DashMap<UserId, DmActivity>,
    pub names: Mutex<Names>,
}

#[derive(Clone, Default, Debug)]
pub struct Names {
    pub usernames: VecDeque<(UserId, UserNames)>,
    pub nicknames: HashMap<GuildId, VecDeque<(UserId, Option<String>)>>,
}

// I feel like doing it this way instead of a tuple has better representation.
#[derive(Clone, Default, Debug)]
pub struct UserNames {
    pub username: String,
    pub global_name: Option<String>,
}

impl UserNames {
    #[must_use]
    pub fn new(username: String, global_name: Option<String>) -> Self {
        UserNames {
            username,
            global_name,
        }
    }
}

impl Names {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DmActivity {
    pub last_announced: i64,
    pub until: Option<i64>,
    pub count: i16,
}

impl DmActivity {
    #[must_use]
    pub fn new(last_announced: i64, until: Option<i64>, count: i16) -> Self {
        DmActivity {
            last_announced,
            until,
            count,
        }
    }
}

#[allow(clippy::missing_panics_doc)]
impl Data {
    pub async fn new() -> Arc<Self> {
        let db_pool = crate::database::init_data().await;
        let redis_pool = crate::database::init_redis_pool().await;

        let config = jamespy_config::JamespyConfig::load_config();
        Arc::new(Data {
            has_started: AtomicBool::new(false),
            db: db_pool,
            redis: redis_pool,
            songbird: songbird::Songbird::serenity(),
            time_started: std::time::Instant::now(),
            reqwest: reqwest::Client::new(),
            config: RwLock::new(config),
            dm_activity: DashMap::new(),
            names: Mutex::new(Names::new()),
        })
    }

    pub async fn check_or_insert_user(&self, user: &User) {
        // this logic is barebones and should probably use drain or something else?
        // i don't plan on changing the limit at runtime so the current implementation should be fine.
        const MAX_LENGTH: usize = 250;

        // Iterate through the cached names and check if the user is present.
        // If the user is present, move them to the back, updating the value if needed.

        let mut update_user = false;
        let mut update_display = false;
        let mut check_db = false;

        {
            let names = &mut self.names.lock();
            let usernames = &mut names.usernames;

            if let Some(index) = usernames.iter().position(|(id, _)| id.eq(&user.id)) {
                // remove so the user can be moved later.
                let (_, cached_name) = usernames.remove(index).unwrap();

                // Update the user in the database if the username is different.
                update_user = !cached_name.username.eq(&user.tag());

                let global_name = user
                    .global_name
                    .as_ref()
                    .map(std::string::ToString::to_string);

                update_display = cached_name.global_name.eq(&global_name);

                if let Some(global) = &user.global_name {
                    // only update this if they have a new display name, also use old name if new is none.
                    update_display = cached_name.global_name.eq(&Some(global.to_string()));
                };

                usernames.push_back((user.id, UserNames::new(user.tag(), global_name)));

                // Length will not be configurable at this time so this doesn't need to do anything fancy.
                if usernames.len() > MAX_LENGTH {
                    usernames.pop_front();
                }
            } else {
                check_db = true;
            }
        }

        match (update_user, update_display) {
            (true, true) => {
                self.insert_user_db(user.id, user.tag()).await;
                self.insert_display_db(user.id, user.global_name.clone().map(|s| s.to_string()))
                    .await;
                return;
            }
            (true, false) => {
                self.insert_user_db(user.id, user.tag()).await;
                return;
            }
            (false, true) => {
                self.insert_display_db(user.id, user.global_name.clone().map(|s| s.to_string()))
                    .await;
                return;
            }
            (false, false) => (),
        }

        if check_db {
            if let Some(db_name) = self.get_latest_username_psql(user.id).await {
                if !db_name.eq(&user.tag()) {
                    self.insert_user_db(user.id, user.tag()).await;
                }
            } else {
                self.insert_user_db(user.id, user.tag()).await;
            }

            if let Some(db_name) = self.get_latest_global_name_psql(user.id).await {
                // never insert if no name.
                if let Some(user_global_name) = &user.global_name {
                    if !db_name.eq(user_global_name) {
                        self.insert_display_db(
                            user.id,
                            user.global_name.clone().map(|s| s.to_string()),
                        )
                        .await;
                    }
                }
            } else {
                // optional values are handled internally on this function.
                self.insert_display_db(user.id, user.global_name.clone().map(|s| s.to_string()))
                    .await;
            }

            // cache the names.
            let usernames = &mut self.names.lock().usernames;
            usernames.push_back((
                user.id,
                UserNames::new(user.tag(), user.global_name.clone().map(|s| s.to_string())),
            ));

            if usernames.len() > MAX_LENGTH {
                usernames.pop_front();
            }
        }
    }

    pub async fn check_or_insert_nick(
        &self,
        guild_id: GuildId,
        user_id: UserId,
        nick: Option<String>,
    ) {
        const MAX_LENGTH: usize = 250;

        let mut update_user = false;
        let mut check_db = false;

        {
            let names = &mut self.names.lock();
            let nicknames = names.nicknames.entry(guild_id).or_default();

            if let Some(index) = nicknames.iter().position(|(id, _)| id.eq(&user_id)) {
                let (_, cached_name) = nicknames.remove(index).unwrap();

                // Update the nickname in database if different.
                update_user = !cached_name.eq(&nick);

                nicknames.push_back((user_id, nick.clone()));

                // Length will not be configurable at this time so this doesn't need to do anything fancy.
                if nicknames.len() > MAX_LENGTH {
                    nicknames.pop_front();
                }
            } else {
                check_db = true;
            }
        }

        if update_user {
            self.insert_nick_db(guild_id, user_id, nick).await;
            return;
        }

        if check_db {
            if let Some(db_name) = self.get_latest_nickname_psql(guild_id, user_id).await {
                // never insert if no name.
                if let Some(nick) = nick.clone() {
                    if !db_name.eq(&nick) {
                        // optional stuff is handled internally.
                        self.insert_nick_db(guild_id, user_id, Some(nick)).await;
                    }
                }
            } else {
                // optional values are handled internally on this function.
                self.insert_nick_db(guild_id, user_id, nick.clone()).await;
            }

            let names = &mut self.names.lock();
            let nicknames = names.nicknames.entry(guild_id).or_default();

            nicknames.push_back((user_id, nick));

            if nicknames.len() > MAX_LENGTH {
                nicknames.pop_front();
            }
        }
    }

    async fn insert_user_db(&self, user_id: UserId, name: String) {
        let timestamp: NaiveDateTime = Utc::now().naive_utc();

        let _ = query!(
            "INSERT INTO usernames (user_id, username, timestamp) VALUES ($1, $2, $3)",
            i64::from(user_id),
            name,
            timestamp
        )
        .execute(&self.db)
        .await;
    }

    async fn insert_display_db(&self, user_id: UserId, name: Option<String>) {
        if name.is_none() {
            return;
        }
        let name = name.unwrap();

        let timestamp: NaiveDateTime = chrono::Utc::now().naive_utc();

        let _ = query!(
            "INSERT INTO global_names (user_id, global_name, timestamp) VALUES ($1, $2, $3)",
            i64::from(user_id),
            name,
            timestamp
        )
        .execute(&self.db)
        .await;
    }

    async fn insert_nick_db(&self, guild_id: GuildId, user_id: UserId, name: Option<String>) {
        if name.is_none() {
            return;
        }
        let name = name.unwrap();

        let timestamp: NaiveDateTime = chrono::Utc::now().naive_utc();

        let _ = query!(
            "INSERT INTO nicknames (guild_id, user_id, nickname, timestamp) VALUES ($1, $2, $3, \
             $4)",
            i64::from(guild_id),
            i64::from(user_id),
            name,
            timestamp
        )
        .execute(&self.db)
        .await;
    }

    async fn get_latest_username_psql(&self, user_id: UserId) -> Option<String> {
        let result = query!(
            "SELECT * FROM usernames WHERE user_id = $1 ORDER BY timestamp DESC LIMIT 1",
            i64::from(user_id)
        )
        .fetch_one(&self.db)
        .await;

        match result {
            Ok(record) => Some(record.username),
            Err(_) => None,
        }
    }

    async fn get_latest_global_name_psql(&self, user_id: UserId) -> Option<String> {
        let result = query!(
            "SELECT * FROM global_names WHERE user_id = $1 ORDER BY timestamp DESC LIMIT 1",
            i64::from(user_id)
        )
        .fetch_one(&self.db)
        .await;

        match result {
            Ok(record) => Some(record.global_name),
            Err(_) => None,
        }
    }

    async fn get_latest_nickname_psql(&self, guild_id: GuildId, user_id: UserId) -> Option<String> {
        let result = query!(
            "SELECT * FROM nicknames WHERE guild_id = $1 AND user_id = $2 ORDER BY timestamp DESC \
             LIMIT 1",
            i64::from(guild_id),
            i64::from(user_id)
        )
        .fetch_one(&self.db)
        .await;

        match result {
            Ok(record) => Some(record.nickname),
            Err(_) => None,
        }
    }

    pub async fn get_activity_check(&self, user_id: UserId) -> Option<DmActivity> {
        let cached = self.dm_activity.get(&user_id);

        if let Some(cached) = cached {
            Some(*cached)
        } else {
            self._get_activity_check_psql(user_id).await
        }
    }

    async fn _get_activity_check_psql(&self, user_id: UserId) -> Option<DmActivity> {
        let result = sqlx::query!(
            "SELECT last_announced, until, count FROM dm_activity WHERE user_id = $1",
            i64::from(user_id)
        )
        .fetch_one(&self.db)
        .await;

        match result {
            Ok(record) => Some(DmActivity::new(
                record.last_announced.unwrap(),
                record.until,
                record.count.unwrap(),
            )),
            Err(err) => {
                if let sqlx::Error::RowNotFound = err {
                    None
                } else {
                    tracing::warn!("Error when attempting to find row: {err}");
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
            .insert(user_id, DmActivity::new(announced, Some(until), count));
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
