use poise::serenity_prelude::{self as serenity, ChannelId, GuildId};
use std::collections::HashSet;

pub fn read_words_from_file(filename: &str) -> HashSet<String> {
    std::fs::read_to_string(filename)
        .expect("Failed to read the file")
        .lines()
        .map(|line| line.trim().to_lowercase())
        .collect()
}

pub async fn get_channel_name(
    ctx: &serenity::Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> String {
    let mut channel_name = channel_id
        .name(ctx)
        .await
        .unwrap_or("Unknown Channel".to_owned());

    if guild_id.get() != 0 && channel_name == "Unknown Channel" {
        let guild_cache = ctx.cache.guild(guild_id).unwrap();
        let threads = &guild_cache.threads;
        for thread in threads {
            if thread.id == channel_id.get() {
                channel_name = thread.name.clone();
                break;
            }
        }
    }

    channel_name
}
