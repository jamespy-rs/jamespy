use crate::Data;
use jamespy_data::structs::{Decay, InnerCache};
use poise::serenity_prelude as serenity;
use serenity::{ChannelId, GetMessages, GuildId, MessageId, UserId};
use std::{collections::HashMap, sync::Arc, time::Instant};

async fn fetch(
    ctx: &serenity::Context,
    channel_id: &ChannelId,
    guild_id: &GuildId,
    deleted_message_id: &MessageId,
    data: &Arc<Data>,
    fetch_newer: bool,
) {
    let builder = if fetch_newer {
        GetMessages::new().after(*deleted_message_id).limit(100)
    } else {
        GetMessages::new().before(*deleted_message_id).limit(100)
    };
    println!("Fetching from http.");
    let Ok(msgs) = channel_id.messages(&ctx, builder).await else {
        return;
    };
    if let Some(mut value) = data.anti_delete_cache.map.get_mut(guild_id) {
        for msg in msgs {
            value.msg_user_cache.insert(msg.id, msg.author.id);
        }
    } else {
        let mut map = HashMap::new();
        for msg in msgs {
            map.insert(msg.id, msg.author.id);
        }
        data.anti_delete_cache.map.insert(
            *guild_id,
            InnerCache {
                last_deleted_msg: *deleted_message_id,
                msg_user_cache: map,
            },
        );
    }
}
pub async fn anti_delete(
    ctx: &serenity::Context,
    data: &Arc<Data>,
    channel_id: &ChannelId,
    guild_id: &GuildId,
    deleted_message_id: &MessageId,
) -> Option<UserId> {
    // increase value.
    {
        let Some(mut value) = data.anti_delete_cache.val.get_mut(guild_id) else {
            data.anti_delete_cache.val.insert(
                *guild_id,
                Decay {
                    val: 1,
                    last_update: Instant::now(),
                },
            );
            return None;
        };
        if value.val > 0 && value.val < 5 {
            value.val += 1;
            // low heat = no check.
            if value.val < 3 {
                return None;
            }
        }
    }
    let last_deleted = {
        let Some(mut value) = data.anti_delete_cache.map.get_mut(guild_id) else {
            fetch(ctx, channel_id, guild_id, deleted_message_id, data, false).await;
            return None;
        };
        let last_deleted = value.last_deleted_msg;
        value.last_deleted_msg = *deleted_message_id;
        for (m, u) in &value.value().msg_user_cache {
            if m == deleted_message_id {
                return Some(*u);
            }
        }
        last_deleted
    };
    if last_deleted < *deleted_message_id {
        fetch(ctx, channel_id, guild_id, deleted_message_id, data, true).await;
    } else {
        fetch(ctx, channel_id, guild_id, deleted_message_id, data, false).await;
    }
    None
}
