use poise::serenity_prelude::{self as serenity, GuildChannel, PartialGuildChannel};

use crate::Error;

pub async fn channel_create(ctx: &serenity::Context, channel: GuildChannel) -> Result<(), Error> {
    let guild_name = channel
        .guild_id
        .name(ctx)
        .unwrap_or("Unknown Guild".to_string());
    println!(
        "\x1B[34m[{}] #{} was created!\x1B[0m",
        guild_name, channel.name
    );
    Ok(())
}

pub async fn channel_delete(ctx: &serenity::Context, channel: GuildChannel) -> Result<(), Error> {
    let guild_name = channel
        .guild_id
        .name(ctx)
        .unwrap_or("Unknown Guild".to_string());
    println!(
        "\x1B[34m[{}] #{} was deleted!\x1B[0m",
        guild_name, channel.name
    );
    Ok(())
}

pub async fn thread_create(ctx: &serenity::Context, thread: GuildChannel) -> Result<(), Error> {
    let guild_id = thread.guild_id;

    let guild_name = if guild_id == 1 {
        "None".to_owned()
    } else {
        match guild_id.name(ctx.clone()) {
            Some(name) => name,
            None => "Unknown".to_owned(),
        }
    };
    // Tell which channel it was created in.
    println!(
        "\x1B[94m[{}] Thread #{} was created!\x1B[0m",
        guild_name, thread.name
    );
    Ok(())
}

pub async fn thread_delete(
    ctx: &serenity::Context,
    thread: PartialGuildChannel,
) -> Result<(), Error> {
    let guild_id = thread.guild_id;
    let guild_cache = ctx.cache.guild(guild_id).unwrap();

    let threads = &guild_cache.threads;

    let mut channel_name = None;

    for thread_cache in threads {
        if thread_cache.id == thread.id {
            channel_name = Some(thread_cache.name.clone());
            break;
        }
    }
    let guild_name = if guild_id == 1 {
        "None".to_owned()
    } else {
        match guild_id.name(ctx.clone()) {
            Some(name) => name,
            None => "Unknown".to_owned(),
        }
    };
    // Currently it won't know which thread was deleted because the method in which it is checked.
    // Tell which channel it was deleted from.
    if let Some(name) = channel_name {
        println!(
            "\x1B[94m[{}] Thread '{}' was deleted!\x1B[0m",
            guild_name, name
        );
    } else {
        println!(
            "\x1B[94m[{}] Thread with unknown name was deleted!\x1B[0m",
            guild_name
        );
    }
    Ok(())
}
