use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;

use poise::serenity_prelude::{GuildId, UserId};
use sqlx::{query, types::chrono::NaiveDateTime};

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
    pub redis: crate::database::RedisPool,
    pub time_started: std::time::Instant,
    pub reqwest: reqwest::Client,
    pub config: RwLock<jamespy_config::JamespyConfig>,
    pub dm_activity: DashMap<UserId, DmActivity>,
    pub names: Mutex<Names>,
}

#[derive(Clone, Default, Debug)]
pub struct Names {
    pub usernames: VecDeque<(UserId, String)>,
    pub global_names: VecDeque<(UserId, String)>,
    pub nicknames: HashMap<GuildId, VecDeque<(UserId, String)>>,
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
            time_started: std::time::Instant::now(),
            reqwest: reqwest::Client::new(),
            config: RwLock::new(config),
            dm_activity: DashMap::new(),
            names: Mutex::new(Names::new()),
        })
    }

    pub async fn check_or_insert_user(&self, user_id: UserId, name: String) {
        // this logic is barebones and should probably use drain or something else?
        // i don't plan on changing the limit at runtime so the current implementation should be fine.
        const MAX_LENGTH: usize = 250;

        // Iterate through the cached names and check if the user is present.
        // If the user is present, move them to the back, updating the value if needed.
        let mut update_db = false;
        let mut check_db = false;
        {
            let names = &mut self.names.lock();
            let usernames = &mut names.usernames;

            if let Some(index) = usernames.iter().position(|(id, _)| id.eq(&user_id)) {
                let (_, cached_name) = usernames.remove(index).unwrap();

                // only update the database if its different.
                if !cached_name.eq(&name) {
                    update_db = true;
                }

                usernames.push_back((user_id, name.clone()));

                if usernames.len() > MAX_LENGTH {
                    usernames.pop_front();
                }
            } else {
                check_db = true;
            }
        }

        // update database after lock has been released.
        if update_db {
            self.insert_user_db(user_id, name.clone()).await;
            return;
        }

        if check_db {
            // if the name couldn't be found in the cache, check the database.
            if let Some(db_name) = self.get_latest_username_psql(user_id).await {
                // The name is in the database, but isn't the same.
                if !db_name.eq(&name) {
                    self.insert_user_db(user_id, name.clone()).await;
                }
            } else {
                self.insert_user_db(user_id, name.clone()).await;
            }

            // cache the username.
            let usernames = &mut self.names.lock().usernames;
            usernames.push_back((user_id, name));

            if usernames.len() > MAX_LENGTH {
                usernames.pop_front();
            }
        }
    }

    async fn insert_user_db(&self, user_id: UserId, name: String) {
        let timestamp: NaiveDateTime = sqlx::types::chrono::Utc::now().naive_utc();

        let _ = query!(
            "INSERT INTO usernames (user_id, username, timestamp) VALUES ($1, $2, $3)",
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
