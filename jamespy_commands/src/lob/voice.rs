use crate::{owner::owner, Context, Error};
use jamespy_data::lob::*;

use poise::serenity_prelude::{self as serenity, GuildChannel};
use songbird::{input::YoutubeDl, tracks::Track};
use std::{fmt::Write, sync::Arc};

pub struct TrackData {
    pub name: Option<String>,
    pub url: String,
}

/// I join
#[poise::command(
    prefix_command,
    category = "Utility",
    channel_cooldown = "5",
    check = "trontin",
    guild_only,
    hide_in_help
)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    let maybe_voice = {
        let guild = ctx.guild().unwrap();
        guild.voice_states.get(&ctx.author().id).cloned()
    };

    if let Some(voice) = maybe_voice {
        if let Some(channel_id) = voice.channel_id {
            ctx.data()
                .songbird
                .join(ctx.guild_id().unwrap(), channel_id)
                .await?;
        }
    }

    Ok(())
}

/// I connect
#[poise::command(
    aliases("conn"),
    prefix_command,
    category = "Utility",
    channel_cooldown = "5",
    check = "owner",
    guild_only,
    hide_in_help
)]
pub async fn connect(ctx: Context<'_>, channel: GuildChannel) -> Result<(), Error> {
    ctx.data()
        .songbird
        .join(ctx.guild_id().unwrap(), channel.id)
        .await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    category = "Utility",
    channel_cooldown = "5",
    check = "trontin",
    guild_only,
    hide_in_help
)]
pub async fn fun(ctx: Context<'_>) -> Result<(), Error> {
    let manager = &ctx.data().songbird;
    let guild_id = ctx.guild_id().unwrap();

    if let Some(handler_lock) = manager.get(guild_id) {
        let option = get_random_lob();

        if let Some(lob) = option {
            let mut handler = handler_lock.lock().await;
            let mut ytdl = YoutubeDl::new(ctx.data().reqwest.clone(), lob.clone());
            let mut data = ytdl.search(Some(1)).await?;

            let metadata = TrackData {
                name: data.next().unwrap().title,
                url: lob.clone(),
            };

            let track = Track::new_with_data(ytdl.into(), Arc::new(metadata));

            handler.enqueue(track).await;
            let mentions = serenity::CreateAllowedMentions::new()
                .all_users(false)
                .everyone(false)
                .all_roles(false);
            ctx.send(
                poise::CreateReply::new()
                    .content(format!("Queuing lob: {lob}"))
                    .allowed_mentions(mentions),
            )
            .await?;
        }
    } else {
        ctx.say("Cannot play without being in a voice channel!")
            .await?;
    }

    Ok(())
}

use url::Url;
fn is_youtube_url(url: &str) -> bool {
    if let Ok(parsed_url) = Url::parse(url) {
        matches!(
            parsed_url.domain(),
            Some("www.youtube.com" | "youtube.com" | "www.youtu.be" | "youtu.be")
        )
    } else {
        false
    }
}

#[poise::command(
    prefix_command,
    category = "Utility",
    channel_cooldown = "5",
    check = "trontin",
    guild_only,
    hide_in_help
)]
pub async fn play(ctx: Context<'_>, mut track: String) -> Result<(), Error> {
    let manager = &ctx.data().songbird;
    let guild_id = ctx.guild_id().unwrap();

    // trim start and end.
    track = track.trim_matches(|c| c == '<' || c == '>').to_string();

    if !is_youtube_url(&track) {
        ctx.say("I can only play YouTube.").await?;
        return Ok(());
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let mut ytdl = YoutubeDl::new(ctx.data().reqwest.clone(), track.clone());
        let mut data = ytdl.search(Some(1)).await?;

        let metadata = TrackData {
            name: data.next().unwrap().title,
            url: track.clone(),
        };

        let track = Track::new_with_data(ytdl.into(), Arc::new(metadata));

        handler.enqueue(track).await;

        ctx.say("Queuing track!").await?;
    } else {
        ctx.say("Cannot play without being in a voice channel!")
            .await?;
    }

    Ok(())
}

#[poise::command(
    prefix_command,
    category = "Utility",
    channel_cooldown = "5",
    check = "trontin",
    guild_only,
    hide_in_help
)]
pub async fn queue(ctx: Context<'_>) -> Result<(), Error> {
    let manager = &ctx.data().songbird;
    let guild_id = ctx.guild_id().unwrap();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        let mut description = String::new();

        {
            let queue = handler.queue();
            if let Some(current) = queue.current() {
                let data: Arc<TrackData> = current.data();

                let track_str = if let Some(title) = &data.name {
                    format!("[{}]({})", title, data.url)
                } else {
                    data.url.clone()
                };

                writeln!(description, "__Currently playing__\n{track_str}").unwrap();
            }

            for (index, item) in queue.current_queue().iter().enumerate().skip(1) {
                let data: Arc<TrackData> = item.data();

                let track_str = if let Some(title) = &data.name {
                    format!("[{}]({})", title, data.url)
                } else {
                    data.url.clone()
                };

                writeln!(description, "{index}. {track_str}").unwrap();
            }
        }

        let embed = serenity::CreateEmbed::new()
            .title("Voice Queue")
            .description(description);

        ctx.send(poise::CreateReply::new().embed(embed)).await?;
    } else {
        ctx.say("Cannot get queue!").await?;
    }

    Ok(())
}

#[poise::command(
    prefix_command,
    category = "Utility",
    channel_cooldown = "5",
    check = "trontin",
    guild_only,
    hide_in_help
)]
pub async fn remove(ctx: Context<'_>, keyword: String) -> Result<(), Error> {
    let manager = &ctx.data().songbird;
    let guild_id = ctx.guild_id().unwrap();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        let index = handler.queue().current_queue().iter().position(|t| {
            let data: Arc<TrackData> = t.data();

            if data.url.starts_with(&keyword) {
                return true;
            }

            if let Some(title) = &data.name {
                if title.starts_with(&keyword) {
                    return true;
                }
            }

            false
        });

        if let Some(index) = index {
            let q = handler.queue().modify_queue(|q| q.remove(index));
            if let Some(item) = q {
                let data: Arc<TrackData> = item.data();

                let track_str = if let Some(title) = &data.name {
                    format!("[{}]({})", title, data.url)
                } else {
                    data.url.clone()
                };

                let mentions = serenity::CreateAllowedMentions::new()
                    .all_users(false)
                    .everyone(false)
                    .all_roles(false);
                ctx.send(
                    poise::CreateReply::new()
                        .content(format!("Removed: {track_str}"))
                        .allowed_mentions(mentions),
                )
                .await?;
            } else {
                ctx.say("Track not found!").await?;
            }
        } else {
            ctx.say("Track not found!").await?;
        }
    } else {
        ctx.say("Cannot get queue!").await?;
    }

    Ok(())
}

#[poise::command(
    prefix_command,
    category = "Utility",
    channel_cooldown = "5",
    check = "trontin",
    guild_only,
    hide_in_help
)]
pub async fn skip(ctx: Context<'_>) -> Result<(), Error> {
    let manager = &ctx.data().songbird;
    let guild_id = ctx.guild_id().unwrap();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;

        // will be true if theres tracks to skip to.
        let skip = handler.queue().len() > 1;

        // stops the current song, which will start the next.
        handler.queue().skip()?;

        if skip {
            ctx.say("skipping!").await?;
        } else {
            ctx.say("stopping!").await?;
        }
    } else {
        ctx.say("I am not in a voice channel.").await?;
    }

    Ok(())
}

#[poise::command(
    prefix_command,
    category = "Utility",
    channel_cooldown = "5",
    check = "trontin",
    guild_only,
    hide_in_help
)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    let manager = &ctx.data().songbird;
    let guild_id = ctx.guild_id().unwrap();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        handler.queue().stop();

        ctx.say("Stopping all tracks.").await?;
    } else {
        ctx.say("I am not in a voice channel.").await?;
    }

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 8] {
    [
        connect(),
        fun(),
        play(),
        queue(),
        skip(),
        stop(),
        remove(),
        play(),
    ]
}
