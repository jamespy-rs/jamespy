use dashmap::{DashMap, DashSet};
use jamespy_data::structs::{Data, Names};

use parking_lot::{Mutex, RwLock};
use std::sync::{atomic::AtomicBool, Arc};

pub async fn setup() -> Arc<Data> {
    let db_pool = jamespy_data::database::init_data().await;
    let redis_pool = jamespy_data::database::init_redis_pool().await;

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
        mod_mode: DashSet::new(),
        names: Mutex::new(Names::new()),
    })
}
