use jamespy_data::structs::Data;
use std::sync::{atomic::AtomicBool, Arc};

pub async fn setup() -> Arc<Data> {
    let handler = jamespy_data::database::init_data().await;

    let config = jamespy_config::JamespyConfig::load_config();

    Arc::new(Data {
        has_started: AtomicBool::new(false),
        database: handler,
        time_started: std::time::Instant::now(),
        reqwest: reqwest::Client::new(),
        config: parking_lot::RwLock::new(config),
        anti_delete_cache: jamespy_data::structs::AntiDeleteCache::default(),
    })
}
