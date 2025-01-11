use crate::{Data, Error};
use moth_data::database::StarboardMessage;
use poise::serenity_prelude::{self as serenity, ChannelId, MessageId, Reaction, UserId};
use small_fixed_array::FixedString;
use std::collections::hash_map::Entry;
use std::{str::FromStr, sync::Arc};

/// Get the reaction count from the cache or fetch it from http if its not available
/// using the reaction_msg for getting the unique total.
///
/// Returns the count, optionally incrementing or decreasing reaction value internally if cached.
/// Panics if `Reaction` is not from the gateway.
pub(crate) async fn get_unique_reaction_count(
    ctx: &serenity::Context,
    data: &Arc<Data>,
    starboard_msg: &StarboardMessage,
    reaction: &Reaction,
    state: Option<bool>,
) -> Result<i16, Error> {
    let author_id = *starboard_msg.user_id;

    let (origin_reactions, starboard_reactions) = {
        let mut guard = data.database.starboard.lock();

        maybe_mutate(&mut guard.reactions_cache, reaction, state);

        // Always get the map entries for both message IDs
        let origin_reactions = guard
            .reactions_cache
            .get(&starboard_msg.message_id)
            .cloned();
        let starboard_reactions = guard
            .reactions_cache
            .get(&starboard_msg.starboard_message_id)
            .cloned();

        (origin_reactions, starboard_reactions)
    };

    let origin_reactions = if let Some(origin_reactions) = origin_reactions {
        origin_reactions
    } else {
        fetch_and_store_uncached(
            ctx,
            data,
            *starboard_msg.channel_id,
            *starboard_msg.message_id,
            author_id,
        )
        .await?
    };

    let starboard_reactions = if let Some(starboard_reactions) = starboard_reactions {
        starboard_reactions
    } else {
        fetch_and_store_uncached(
            ctx,
            data,
            *starboard_msg.starboard_message_channel,
            *starboard_msg.starboard_message_id,
            author_id,
        )
        .await?
    };

    let mut unique_values = std::collections::HashSet::new();
    unique_values.extend(origin_reactions.1);
    unique_values.extend(starboard_reactions.1);

    Ok(unique_values.len() as i16)
}

fn maybe_mutate(
    map: &mut std::collections::HashMap<MessageId, (UserId, Vec<UserId>)>,
    reaction: &Reaction,
    state: Option<bool>,
) {
    let message_id = reaction.message_id;
    let user = reaction.user_id.unwrap();

    map.entry(message_id).and_modify(|(_, v)| {
        if let Some(true) = state {
            if !v.contains(&user) {
                v.push(user);
            }
        } else if let Some(false) = state {
            v.retain(|&user_id| user_id != user);
        }
    });
}

async fn fetch_and_store_uncached(
    ctx: &serenity::Context,
    data: &Arc<Data>,
    channel_id: ChannelId,
    message_id: MessageId,
    author_id: UserId,
) -> Result<(UserId, Vec<UserId>), Error> {
    let users = ctx
        .http
        .get_reaction_users(
            channel_id,
            message_id,
            &serenity::ReactionType::Unicode(
                FixedString::from_str(&data.starboard_config.star_emoji).unwrap(),
            ),
            100,
            None,
        )
        .await?;

    let bot_id = ctx.cache.current_user().id;
    let filtered = users
        .into_iter()
        .filter(|user| user.id != author_id && user.id != bot_id)
        .map(|u| u.id)
        .collect::<Vec<_>>();

    data.database
        .starboard
        .lock()
        .reactions_cache
        .insert(message_id, (author_id, filtered.clone()));

    Ok((author_id, filtered))
}

/// Get the reaction count from the cache or fetch it from http if its not available.
///
/// Returns the count, optionally incrementing or decreasing reaction value internally if cached.
/// Panics if `Reaction` is not from the gateway.
pub(crate) async fn get_reaction_count(
    ctx: &serenity::Context,
    data: &Arc<Data>,
    reaction: &Reaction,
    author_id: UserId,
    state: Option<bool>,
) -> Result<i16, Error> {
    let reaction_user = reaction.user_id.unwrap();

    // If Some(true), add reaction_user, if Some(false), remove.
    let reactions = {
        let mut guard = data.database.starboard.lock();
        guard
            .reactions_cache
            .entry(reaction.message_id)
            .and_modify(|(_, vec)| {
                if let Some(true) = state {
                    if !vec.contains(&reaction_user) {
                        vec.push(reaction_user);
                    }
                } else if let Some(false) = state {
                    vec.retain(|&user_id| user_id != reaction_user);
                }
            });
        guard.reactions_cache.get(&reaction.message_id).cloned()
    };

    if let Some((_, reactors)) = reactions {
        return Ok(reactors.len() as i16);
    }

    // TODO: paginate this.
    let users = ctx
        .http
        .get_reaction_users(
            reaction.channel_id,
            reaction.message_id,
            &serenity::ReactionType::Unicode(
                FixedString::from_str(&data.starboard_config.star_emoji).unwrap(),
            ),
            100,
            None,
        )
        .await?;

    let filtered = users
        .into_iter()
        .filter(|user| user.id != author_id)
        .map(|u| u.id)
        .collect::<Vec<_>>();

    let count = filtered.len();

    let mut guard = data.database.starboard.lock();
    match guard.reactions_cache.entry(reaction.message_id) {
        Entry::Occupied(mut entry) => {
            *entry.get_mut() = (author_id, filtered);
        }
        Entry::Vacant(entry) => {
            entry.insert((author_id, filtered));
        }
    }

    Ok(count as i16)
}
