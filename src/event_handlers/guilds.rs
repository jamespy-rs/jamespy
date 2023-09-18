use poise::serenity_prelude::{self as serenity, Guild, GuildId, Member, User};

use crate::Error;

pub async fn guild_create(
    ctx: &serenity::Context,
    guild: Guild,
    is_new: Option<bool>,
    //data: &Data,
) -> Result<(), Error> {
    if let Some(true) = is_new {
        println!(
            "\x1B[33mJoined {} (ID:{})!\nNow in {} guild(s)\x1B[0m",
            guild.name,
            guild.id,
            ctx.cache.guilds().len()
        );
    }
    Ok(())
}

pub async fn guild_member_addition(
    ctx: &serenity::Context,
    new_member: Member,
) -> Result<(), Error> {
    let guild_id = new_member.guild_id;
    let joined_user_id = new_member.user.id;

    let guild_name = if guild_id == 1 {
        "None".to_owned()
    } else {
        match guild_id.name(ctx.clone()) {
            Some(name) => name,
            None => "Unknown".to_owned(),
        }
    };
    println!(
        "\x1B[33m[{}] {} (ID:{}) has joined!\x1B[0m",
        guild_name, new_member.user.name, joined_user_id
    );
    Ok(())
}

pub async fn guild_member_removal(
    ctx: &serenity::Context,
    guild_id: GuildId,
    user: User,
) -> Result<(), Error> {
    let guild_id = guild_id;
    let guild_name = if guild_id == 1 {
        "None".to_owned()
    } else {
        match guild_id.name(ctx.clone()) {
            Some(name) => name,
            None => "Unknown".to_owned(),
        }
    };

    println!(
        "\x1B[33m[{}] {} (ID:{}) has left!\x1B[0m",
        guild_name, user.name, user.id
    );
    Ok(())
}
