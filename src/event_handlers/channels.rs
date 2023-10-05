use crate::utils::misc::get_guild_name;
use crate::Error;
use poise::serenity_prelude::{self as serenity, GuildChannel, PartialGuildChannel};

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
    let guild_name = get_guild_name(ctx, guild_id);

    let parent_channel_name = if let Some(parent_id) = thread.parent_id {
        parent_id.name(ctx).await?
    } else {
        "Unknown Channel".to_string()
    };

    println!(
        "\x1B[94m[{}] Thread #{} was created in #{}!\x1B[0m",
        guild_name, thread.name, parent_channel_name
    );
    Ok(())
}

pub async fn thread_delete(
    ctx: &serenity::Context,
    thread: PartialGuildChannel,
    full_thread_data: Option<GuildChannel>,
) -> Result<(), Error> {
    let guild_id = thread.guild_id;
    let mut channel_name = String::new();
    let mut parent_channel_name: String = String::new();

    if let Some(full_thread) = full_thread_data {
        channel_name = full_thread.name;

        if let Some(parent_id) = full_thread.parent_id {
            parent_channel_name = parent_id.name(ctx).await?;
        } else {
            parent_channel_name = "Unknown Channel".to_string();
        }
    }
    let guild_name = get_guild_name(ctx, guild_id);

    if channel_name.is_empty() {
        println!(
            "\x1B[94m[{}] An unknown thread was deleted!\x1B[0m",
            guild_name
        )
    } else {
        println!(
            "\x1B[94m[{}] Thread #{} was deleted from #{}!\x1B[0m",
            guild_name, channel_name, parent_channel_name
        )
    }
    Ok(())
}
