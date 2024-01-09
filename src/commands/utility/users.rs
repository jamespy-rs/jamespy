use crate::{Context, Error};
use ::serenity::all::{User, UserId};
use bb8_redis::redis::AsyncCommands;
use poise::serenity_prelude::{
    self as serenity, ActivityType, Colour, GuildMemberFlags, Member, OnlineStatus,
};
use std::collections::HashMap;

#[poise::command(
    rename = "guild-flags",
    aliases("guild_flags", "guildflags"),
    slash_command,
    prefix_command,
    guild_only,
    category = "Utility",
    required_permissions = "MANAGE_MESSAGES",
    default_member_permissions = "MANAGE_MESSAGES",
    track_edits,
    user_cooldown = 4
)]
pub async fn guild_flags(
    ctx: Context<'_>,
    #[description = "The member whose flags are to be checked."] member: Member,
) -> Result<(), Error> {
    let member_flags = member.flags;

    let mut outcome = String::new();

    if member_flags.contains(GuildMemberFlags::DID_REJOIN) {
        outcome.push_str("Member has left and rejoined the guild.\n");
    }

    if member_flags.contains(GuildMemberFlags::COMPLETED_ONBOARDING) {
        outcome.push_str("Member has completed onboarding.\n");
    }

    if member_flags.contains(GuildMemberFlags::BYPASSES_VERIFICATION) {
        outcome.push_str("Member is exempt from guild verification requirements.\n");
    }

    if member_flags.contains(GuildMemberFlags::STARTED_ONBOARDING) {
        outcome.push_str("Member has started onboarding.\n");
    }

    if outcome.is_empty() {
        ctx.say("Member has no special flags.").await?;
    } else {
        ctx.say(outcome).await?;
    }

    Ok(())
}

#[poise::command(
    rename = "last-reactions",
    aliases("lastreactions", "last_reactions"),
    slash_command,
    prefix_command,
    category = "Utility",
    required_permissions = "MANAGE_MESSAGES",
    default_member_permissions = "MANAGE_MESSAGES",
    guild_only,
    user_cooldown = 3
)]
pub async fn last_reactions(ctx: Context<'_>) -> Result<(), Error> {
    let redis_pool = &ctx.data().redis;
    let mut redis_conn = redis_pool.get().await?;
    let reaction_key = format!("reactions:{}", ctx.guild_id().unwrap());

    let reactions: Vec<String> = redis_conn.lrange(reaction_key, 0, 24).await?;

    let mut formatted: Vec<String> = vec![];
    for reaction in reactions {
        let components: Vec<&str> = reaction
            .trim_matches(|c| c == '[' || c == ']' || c == '"')
            .split(',')
            .map(|component| component.trim_matches(|c| c == '"'))
            .collect();

        if components.len() == 4 {
            let (emoji, user_id, reaction_id, state) = (
                components[0],
                components[1].parse::<u64>().unwrap(),
                components[2].parse::<u64>().unwrap(),
                components[3].parse::<u32>().unwrap(),
            );

            let username = UserId::new(user_id)
                .to_user(ctx)
                .await
                .map_or_else(|_| "Unknown User".to_string(), |user| user.name.to_string());
            formatted.push(format!(
                "**{}** {} {} Message ID: {}",
                username,
                if state == 1 { "added" } else { "removed" },
                emoji,
                reaction_id
            ));
        }
    }

    ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title("Last reaction events")
                .description(formatted.join("\n"))
                .color(Colour::from_rgb(0, 255, 0)),
        ),
    )
    .await?;

    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    category = "Utility",
    guild_only,
    user_cooldown = 15
)]
pub async fn statuses(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let cache = &ctx.cache();
    let guild = cache.guild(guild_id).unwrap().clone(); // I don't know how to use new stuff.

    let mut status_counts = HashMap::new();
    let mut message = String::new();
    for presence in &guild.presences {
        let status = presence.1.status;

        let count = status_counts.entry(status).or_insert(0);
        *count += 1;
    }

    for (status, count) in &status_counts {
        let status_message = match status {
            OnlineStatus::DoNotDisturb => format!("Do Not Disturb: {count}"),
            OnlineStatus::Idle => format!("Idle: {count}"),
            OnlineStatus::Invisible => format!("Invisible: {count}"),
            OnlineStatus::Offline => format!("Offline: {count}"),
            OnlineStatus::Online => format!("Online: {count}"),
            _ => String::new(),
        };

        message.push_str(&status_message);
        message.push('\n');
    }
    message.push_str(&guild.presences.len().to_string());
    ctx.send(poise::CreateReply::default().content(message))
        .await?;

    Ok(())
}

/// See what games people are playing!
#[poise::command(
    slash_command,
    prefix_command,
    category = "Utility",
    guild_only,
    user_cooldown = 15
)]
pub async fn playing(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let cache = &ctx.cache();
    let guild = cache.guild(guild_id).unwrap().clone();

    let total_members = guild
        .presences
        .values()
        .filter(|presence| {
            presence
                .activities
                .iter()
                .any(|activity| activity.kind == ActivityType::Playing)
        })
        .count();

    let mut activity_counts: HashMap<&str, u32> = HashMap::new();
    for presence in guild.presences.values() {
        for activity in &presence.activities {
            if activity.kind == ActivityType::Playing {
                let name = &activity.name;
                let count = activity_counts.entry(name.as_str()).or_insert(0);
                *count += 1;
            }
        }
    }

    let total_games: usize = activity_counts.values().len();

    let mut vec: Vec<(&&str, &u32)> = activity_counts.iter().collect();
    vec.sort_by(|a, b| b.1.cmp(a.1));

    let pages: Vec<Vec<(&str, u32)>> = vec
        .iter()
        .map(|&(name, count)| (*name, *count))
        .collect::<Vec<(&str, u32)>>()
        .chunks(15)
        .map(<[(&str, u32)]>::to_vec)
        .collect();

    jamespy_utils::cache::presence_builder(ctx, pages, total_members, total_games).await?;

    Ok(())
}

/// See what games people are playing!
#[poise::command(
    rename = "dm-activity-check",
    aliases("dm-activity"),
    prefix_command,
    category = "Utility",
    guild_only,
    required_permissions = "MANAGE_MESSAGES"
)]
pub async fn dm_activity_check(ctx: Context<'_>, user: User) -> Result<(), Error> {
    if ctx.guild_id().unwrap() != 98226572468690944 {
        return Ok(());
    }

    let author =
        serenity::CreateEmbedAuthor::new(format!("{}'s unusual dm activity info", user.tag()))
            .icon_url(user.avatar_url().unwrap_or_default());

    let mut embed = serenity::CreateEmbed::default().author(author);

    let result = ctx.data().get_activity_check(user.id).await;

    if let Some(result) = result {
        let until = if let Some(u) = result.1 {
            format!("<t:{u}>")
        } else {
            String::from("None")
        };

        embed = embed
            .field("Announced last", format!("<t:{}>", result.0), true)
            .field("Until", until, true)
            .field("Count", result.2.to_string(), true);
    }

    if let Ok(member) = ctx.guild_id().unwrap().member(ctx, user.id).await {
        let until = if let Some(activity) = member.unusual_dm_activity_until {
            format!("<t:{}>", activity.unix_timestamp())
        } else {
            String::from("None")
        };
        embed = embed.field("Currently flagged until?", until, false);
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// See what games people are playing!
#[poise::command(
    rename = "flag-lb",
    aliases("flagged-lb", "dm-activity-lb"),
    prefix_command,
    category = "Utility",
    guild_only,
    required_permissions = "MANAGE_MESSAGES"
)]
pub async fn flag_lb(ctx: Context<'_>) -> Result<(), Error> {
    if ctx.guild_id().unwrap() != 98226572468690944 {
        return Ok(());
    }

    // poise will display an error if this goes wrong, though at the same time it'll show an error if nobody is on the list.
    let result = sqlx::query!("SELECT user_id, count FROM dm_activity ORDER BY count DESC LIMIT 20")
        .fetch_all(&ctx.data().db)
        .await
        .unwrap();

    let mut description = String::new();
    for (index, record) in result.into_iter().enumerate() {
        description.push_str(&format!("**{}**. <@{}>: **{}**\n", index + 1, record.user_id, record.count.unwrap()));
    }

    let embed = serenity::CreateEmbed::default().title("Top 20 users flagged with dm-activity").description(description);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

pub fn commands() -> [crate::Command; 6] {
    [
        guild_flags(),
        last_reactions(),
        statuses(),
        playing(),
        dm_activity_check(),
        flag_lb(),
    ]
}
