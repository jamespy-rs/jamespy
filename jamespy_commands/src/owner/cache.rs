use crate::{Context, Error};

use poise::serenity_prelude as serenity;

/// View/set max messages cached per channel.
#[poise::command(
    rename = "max-messages",
    prefix_command,
    category = "Cache",
    owners_only,
    hide_in_help
)]
pub async fn max_messages(
    ctx: Context<'_>,
    #[description = "Set this value to change the cache limit."] value: Option<u16>,
) -> Result<(), Error> {
    if let Some(val) = value {
        ctx.say(format!(
            "Max messages cached per channel set: **{}** -> **{}**",
            ctx.cache().settings().max_messages,
            val
        ))
        .await?;
        ctx.cache().set_max_messages(val.into());
    } else {
        ctx.say(format!(
            "Max messages cached per channel is set to: **{}**",
            ctx.cache().settings().max_messages
        ))
        .await?;
    }
    Ok(())
}

#[poise::command(rename = "guild-cache-stats", prefix_command, category = "Cache", hide_in_help, owners_only, guild_only)]
pub async fn guild_cache_stats(ctx: Context<'_>) -> Result<(), Error> {
    let (channel_count, thread_count, members_count, cached_members) = {
        let guild = ctx.guild().unwrap();

        let channel_count = guild.channels.len();
        let thread_count = guild.threads.len();
        let members_count = guild.member_count;
        let cached_members = guild.members.len();

        (channel_count, thread_count, members_count, cached_members)
    };

    let stats = format!("Channel Count: {}\n Thread count: {}\nUser count: {}\nCached Users: {}", channel_count, thread_count, members_count, cached_members);

    let embed = serenity::CreateEmbed::default()
        .title("Guild Cache Stats")
        .field("Stats", stats, true);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}


pub fn commands() -> [crate::Command; 2] {
    [max_messages(), guild_cache_stats()]
}
