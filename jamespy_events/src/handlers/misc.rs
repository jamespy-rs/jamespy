use crate::{Data, Error};
use poise::serenity_prelude::{self as serenity, Ready};
use serde::{Deserialize, Serialize};

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

pub async fn ready(ctx: &serenity::Context, ready: &Ready, data: Arc<Data>) -> Result<(), Error> {
    let shard_count = ctx.cache.shard_count();
    let is_last_shard = (ctx.shard_id.0 + 1) == shard_count.get();

    if is_last_shard && !data.has_started.swap(true, Ordering::SeqCst) {
        finalize_start(ctx.clone(), &data);
        println!("Logged in as {}", ready.user.tag());
    }

    Ok(())
}

fn finalize_start(ctx: serenity::Context, data: &Arc<Data>) {
    let data_clone = data.clone();

    tokio::spawn(async move {
        let mut interval: tokio::time::Interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            data_clone.anti_delete_cache.decay_proc();
        }
    });

    if let Ok(key) = std::env::var("PERSPECTIVE_KEY") {
        let data_clone = data.clone();
        tokio::spawn(async move {
            let mut interval: tokio::time::Interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                handle_next_perspective(&ctx, &data_clone, &key).await;
            }
        });
    }
}

async fn handle_next_perspective(ctx: &serenity::Context, data: &Arc<Data>, key: &str) {
    let Some((user_id, user_name, content)) = data.perspective_queue.lock().first().cloned() else {
        println!("no entry.");
        return;
    };


    use std::collections::HashMap;
let mut query_params = HashMap::new();
query_params.insert("key", key);
.query(&query_params)


    let Ok(result) = dbg!(
        data.reqwest
            .post("https://commentanalyzer.googleapis.com/v1alpha1/comments:analyze")
            .query(&["key", key])
            .json(&AnalyzeRequest {
                comment: Comment { text: &content },
                do_not_store: true,
                requested_attributes: RequestedAttributes {
                    toxicity: (),
                    inflammatory: (),
                    sexually_explicit: (),
                },
                languages: "en",
            })
            .send()
            .await
    ) else {
        println!("failed to send request");
        data.perspective_queue.lock().remove(0);
        return;
    };

    let Ok(json) = result.json::<AttributeScores>().await else {
        println!("Failed to deserialize.");
        return;
    };

    println!("{json:?}");
}

#[derive(Serialize)]
struct AnalyzeRequest<'a> {
    comment: Comment<'a>,
    #[serde(rename = "camelCase")]
    requested_attributes: RequestedAttributes,
    languages: &'a str,
    #[serde(rename = "camelCase")]
    do_not_store: bool,
}

#[derive(Serialize)]
struct Comment<'a> {
    text: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct RequestedAttributes {
    toxicity: (),
    inflammatory: (),
    sexually_explicit: (),
}

#[derive(Deserialize, Serialize, Debug)]
struct AttributeScores {
    #[serde(rename = "SEXUALLY_EXPLICIT")]
    sexually_explicit: ScoreDetail,

    #[serde(rename = "INFLAMMATORY")]
    inflammatory: ScoreDetail,

    #[serde(rename = "TOXICITY")]
    toxicity: ScoreDetail,

    languages: Vec<String>,

    detected_languages: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct ScoreDetail {
    span_scores: Vec<SpanScore>,
    summary_score: SummaryScore,
}

#[derive(Deserialize, Serialize, Debug)]
struct SpanScore {
    begin: usize,
    end: usize,
    score: Score,
}

#[derive(Deserialize, Serialize, Debug)]
struct Score {
    value: f64,

    #[serde(rename = "type")]
    score_type: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct SummaryScore {
    value: f64,

    #[serde(rename = "type")]
    score_type: String,
}
