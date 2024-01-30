use dashmap::DashMap;
use serenity::all::UserId;
use std::sync::{atomic::AtomicBool, Arc, RwLock};

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
}

#[derive(Clone, Copy, Debug)]
pub struct DmActivity {
    pub last_announced: i64,
    pub until: Option<i64>,
    pub count: i16,
}

impl DmActivity {
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
            config: config.into(),
            dm_activity: dashmap::DashMap::new(),
        })
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
