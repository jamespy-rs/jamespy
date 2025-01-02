use std::sync::Arc;

use crate::helper::{get_channel_name, get_guild_name_override, get_user};
use crate::{Data, Error};

mod database;
use database::*;

use jamespy_ansi::{HI_MAGENTA, RESET};

use poise::serenity_prelude::{self as serenity, Reaction};

pub async fn reaction_add(
    ctx: &serenity::Context,
    add_reaction: &Reaction,
    data: Arc<Data>,
) -> Result<(), Error> {
    // I'm not bothered here anymore.
    // will need to to_user when guild_id is none and i'm not adding complexity
    // for reactions that don't matter.
    if add_reaction.guild_id.is_none() {
        return Ok(());
    };

    // recieved over gateway, so a user is present.
    let user_id = add_reaction.user_id.unwrap();
    let user_name = match get_user(ctx, add_reaction.guild_id.unwrap(), user_id).await {
        Some(user) => {
            if user.bot() {
                return Ok(());
            }
            user.tag()
        }
        None => String::from("Unknown User"),
    };

    let guild_id = add_reaction.guild_id;
    let guild_name = get_guild_name_override(ctx, &data, guild_id);

    let channel_name = get_channel_name(ctx, guild_id, add_reaction.channel_id).await;

    println!(
        "{HI_MAGENTA}[{}] [#{}] {} added a reaction: {}{RESET}",
        guild_name, channel_name, user_name, add_reaction.emoji
    );

    insert_addition(&data.database, guild_id.unwrap(), user_id, add_reaction).await?;

    Ok(())
}

pub async fn reaction_remove(
    ctx: &serenity::Context,
    removed_reaction: &Reaction,
    data: Arc<Data>,
) -> Result<(), Error> {
    // ditto.
    if removed_reaction.guild_id.is_none() {
        return Ok(());
    };

    // ditto.
    let user_id = removed_reaction.user_id.unwrap();
    let user_name = match get_user(ctx, removed_reaction.guild_id.unwrap(), user_id).await {
        Some(user) => {
            if user.bot() {
                return Ok(());
            }
            user.tag()
        }
        None => String::from("Unknown User"),
    };
    let guild_id = removed_reaction.guild_id;
    let guild_name = get_guild_name_override(ctx, &data, guild_id);
    let channel_name = get_channel_name(ctx, guild_id, removed_reaction.channel_id).await;

    println!(
        "{HI_MAGENTA}[{}] [#{}] {} removed a reaction: {}{RESET}",
        guild_name, channel_name, user_name, removed_reaction.emoji
    );

    insert_removal(&data.database, guild_id.unwrap(), user_id, removed_reaction).await?;

    Ok(())
}
