use dashmap::DashMap;
use jamespy_data::structs::{AntiDeleteCache, Data, Names};

use ocrs::{OcrEngine, OcrEngineParams};
use parking_lot::{Mutex, RwLock};
use rten::Model;
use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};

pub async fn setup() -> Arc<Data> {
    let db_pool = jamespy_data::database::init_data().await;

    let config = jamespy_config::JamespyConfig::load_config();

    Arc::new(Data {
        has_started: AtomicBool::new(false),
        db: db_pool,
        time_started: std::time::Instant::now(),
        reqwest: reqwest::Client::new(),
        config: RwLock::new(config),
        dm_activity: DashMap::new(),
        names: Mutex::new(Names::new()),
        anti_delete_cache: AntiDeleteCache::default(),
        ocr_engine: prep_engine(),
    })
}

fn prep_engine() -> OcrEngine {
    let detection_model_path = PathBuf::from("text_models/text-detection.rten");
    let rec_model_path = PathBuf::from("text_models/text-recognition.rten");

    let detection_model = Model::load_file(detection_model_path).unwrap();
    let recognition_model = Model::load_file(rec_model_path).unwrap();

    OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })
    .unwrap()
}
