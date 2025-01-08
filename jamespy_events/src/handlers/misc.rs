use crate::helper::get_channel_name;
use crate::{Data, Error};
use ::serenity::all::{ChannelId, CreateEmbed, CreateEmbedAuthor, CreateMessage};
use perspective_api::client::{AnalyzeCommentRequest, Comment, RequestedAttributes, ScoreOptions};
use poise::serenity_prelude::{self as serenity, Ready};

use std::fmt::Write;

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

    if data.perspective.is_some() {
        let data_clone = data.clone();

        tokio::spawn(async move {
            let mut interval: tokio::time::Interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                handle_next_perspective(&ctx, &data_clone).await;
            }
        });
    }
}

async fn handle_next_perspective(ctx: &serenity::Context, data: &Arc<Data>) {
    // Kinda scuff, but i'm never gonna set this to none at runtime soooooo...
    let perspective = data.perspective.as_ref().unwrap();

    let Some(partial_message) = data.perspective_queue.lock().first().cloned() else {
        return;
    };

    let default_option = Some(ScoreOptions {
        score_type: None,
        score_threshold: Some(0.8),
    });

    let request = AnalyzeCommentRequest {
        comment: Comment {
            text: partial_message.content.to_string(),
            comment_type: None,
        },
        requested_attributes: RequestedAttributes {
            toxicity: default_option.clone(),
            severe_toxicity: default_option.clone(),
            identity_attack: Some(ScoreOptions {
                score_type: None,
                score_threshold: Some(0.65),
            }),
            insult: default_option.clone(),
            profanity: Some(ScoreOptions {
                score_type: None,
                score_threshold: Some(0.9),
            }),
            threat: default_option.clone(),
            sexually_explicit: default_option.clone(),
            flirtation: default_option.clone(),
            toxicity_experimental: None,
            severe_toxicity_experimental: None,
            identity_attack_experimental: None,
            insult_experimental: None,
            profanity_experimental: None,
            threat_experimental: None,
        },
        languages: Some(vec!["en".to_string()]),
        do_not_store: Some(true),
        ..Default::default()
    };

    let Ok(result) = perspective.analyze(request).await else {
        data.perspective_queue.lock().remove(0);
        return;
    };

    let mut msg_response = String::new();

    if let Some(scores) = &result.attribute_scores {
        let attributes = [
            ("TOXICITY", &scores.toxicity),
            ("IDENTITY_ATTACK", &scores.identity_attack),
            ("SEXUALLY_EXPLICIT", &scores.sexually_explicit),
            ("SEVERE_TOXICITY", &scores.severe_toxicity),
            ("PROFANITY", &scores.profanity),
            ("THREAT", &scores.threat),
            ("FLIRTATION", &scores.flirtation),
            ("INSULT", &scores.insult),
            ("TOXICITY_EXPERIMENTAL", &scores.toxicity_experimental),
            (
                "SEVERE_TOXICITY_EXPERIMENTAL",
                &scores.severe_toxicity_experimental,
            ),
            (
                "IDENTITY_ATTACK_EXPERIMENTAL",
                &scores.identity_attack_experimental,
            ),
            ("INSULT_EXPERIMENTAL", &scores.insult_experimental),
            ("PROFANITY_EXPERIMENTAL", &scores.profanity_experimental),
            ("THREAT_EXPERIMENTAL", &scores.threat_experimental),
        ];

        for (label, score_option) in attributes {
            if let Some(score) = score_option {
                let score = score
                    .summary_score
                    .as_ref()
                    .map_or(0.0, |s| s.value.unwrap_or_default());

                writeln!(msg_response, "{label}: {score:.2}").unwrap();
            }
        }

        let name =
            get_channel_name(ctx, partial_message.guild_id, partial_message.channel_id).await;

        if !msg_response.is_empty() {
            let mut author = CreateEmbedAuthor::new(&partial_message.user_name);

            if let Some(url) = partial_message.avatar_url() {
                author = author.icon_url(url);
            }

            let embed = CreateEmbed::new()
                .author(author)
                .title(format!("Message in #{name}"))
                .description(&partial_message.content)
                .fields([
                    ("Link", partial_message.message_link(), false),
                    ("Score", msg_response, false),
                ]);

            let _ = ChannelId::new(1325934163014058004)
                .send_message(&ctx.http, CreateMessage::new().embed(embed))
                .await;
        }
    }

    data.perspective_queue.lock().remove(0);
}
