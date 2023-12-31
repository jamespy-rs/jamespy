use chrono::Utc;
use poise::serenity_prelude::{
    self as serenity, ChannelId, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter,
    GuildMemberUpdateEvent, Member,
};

use crate::{Data, Error};

pub async fn guild_member_update(
    ctx: &serenity::Context,
    old_if_available: Option<Member>,
    new: Option<Member>,
    event: GuildMemberUpdateEvent,
    data: &Data,
) -> Result<(), Error> {
    let guild_id = event.guild_id;
    let guild_name = if guild_id == 1 {
        "None".to_owned()
    } else {
        match guild_id.name(ctx.clone()) {
            Some(name) => name,
            None => "Unknown".to_owned(),
        }
    };

    if let Some(old_member) = old_if_available {
        if let Some(new_member) = new {
            let old_nickname = old_member.nick.as_deref().unwrap_or("None");
            let new_nickname = new_member.nick.as_deref().unwrap_or("None");

            if old_nickname != new_nickname {
                println!(
                    "\x1B[92m[{}] Nickname change: {}: {} -> {} (ID:{})\x1B[0m",
                    guild_name,
                    new_member.user.name,
                    old_nickname,
                    new_nickname,
                    new_member.user.id
                );
            };

            if old_member.user.name != new_member.user.name {
                println!(
                    "\x1B[92mUsername change: {} -> {} (ID:{})\x1B[0m",
                    old_member.user.name, new_member.user.name, new_member.user.id
                );
            }
            if old_member.user.global_name != new_member.user.global_name {
                println!(
                    "\x1B[92mDisplay name change: {}: {} -> {} (ID:{})\x1B[0m",
                    old_member.user.name,
                    old_member
                        .clone()
                        .user
                        .global_name
                        .unwrap_or("None".to_owned().into()),
                    new_member
                        .clone()
                        .user
                        .global_name
                        .unwrap_or("None".to_owned().into()),
                    new_member.user.id
                );
            }
        }

        if let Some(timestamp) = event.unusual_dm_activity_until {
            let timestamp = timestamp.timestamp();
            if guild_id != 98226572468690944 {
                return Ok(());
            }

            let now = Utc::now().timestamp();

            // last_announced, activity until, times updated.
            // they have a flag now, check if they had one before.
            if let Some(old_stamp) = data.get_activity_check(event.user.id).await {
                // they had a flag before.
                if let Some(until) = old_stamp.1 {
                    if until < now {
                        data.remove_until(event.user.id).await;
                        return Ok(());
                    }

                    // If its newer by an hour, announce.
                    if timestamp >= (old_stamp.0 + 3600) {
                        dm_activity_updated(ctx, &event, old_stamp.2).await?;
                        data.new_or_announced(event.user.id, now, timestamp, Some(old_stamp.2 + 1))
                            .await;
                        return Ok(()); // its okay to return here because it'll be updated.
                    }

                    // If its newer than a minute, update.
                    if timestamp >= (until + 60) {
                        data.updated_no_announce(event.user.id, now, timestamp, old_stamp.2 + 1)
                            .await;
                    }
                } else {
                    dm_activity_new(ctx, &event, old_stamp.2).await?;
                    data.new_or_announced(event.user.id, now, timestamp, Some(old_stamp.2 + 1))
                        .await;
                }
            } else if timestamp >= Utc::now().timestamp() {
                // add, no previous match but in future.
                dm_activity_new(ctx, &event, 0).await?;
                data.new_or_announced(event.user.id, now, timestamp, Some(1))
                    .await;
            }
        }
    }

    Ok(())
}

async fn dm_activity_new(
    ctx: &serenity::Context,
    event: &GuildMemberUpdateEvent,
    count: i16,
) -> Result<(), Error> {
    let user_ping = format!("<@{}>", event.user.id);
    let joined_at = event.joined_at.unix_timestamp();
    let created_at = event.user.created_at().unix_timestamp();
    let online_status = {
        let guild = ctx.cache.guild(event.guild_id).unwrap();

        guild
            .presences
            .get(&event.user.id)
            .map(|p| p.client_status.clone())
    };

    let mut client_stat = vec![];
    if let Some(Some(client)) = online_status {
        if client.desktop.is_some() {
            client_stat.push("Desktop");
        }
        if client.mobile.is_some() {
            client_stat.push("Mobile");
        }
        if client.web.is_some() {
            client_stat.push("Web");
        }
    }
    let stats = client_stat.join(", ");

    let mut embed = CreateEmbed::new()
        .author(
            CreateEmbedAuthor::new(format!(
                "{} is flagged with unusual dm activity",
                event.user.name
            ))
            .icon_url(event.user.face()),
        )
        .field("User", user_ping, true)
        .field("Joined at", format!("<t:{joined_at}:R>"), true)
        .field("Creation Date", format!("<t:{created_at}:R>"), true)
        .footer(CreateEmbedFooter::new(format!(
            "User ID: {}",
            event.user.id
        )));

    if count != 0 {
        embed = embed.footer(CreateEmbedFooter::new(format!(
            "User ID: {} • Previous hits: {}",
            event.user.id, count
        )));
    }

    if !stats.is_empty() {
        embed = embed.description(format!("**Online on**:\n{stats}"));
    }

    ChannelId::new(158484765136125952)
        .send_message(ctx, serenity::CreateMessage::default().embed(embed))
        .await?;

    Ok(())
}

async fn dm_activity_updated(
    ctx: &serenity::Context,
    event: &GuildMemberUpdateEvent,
    count: i16,
) -> Result<(), Error> {
    let user_ping = format!("<@{}>", event.user.id);
    let joined_at = event.joined_at.unix_timestamp();
    let created_at = event.user.created_at().unix_timestamp();

    let online_status = {
        let guild = ctx.cache.guild(event.guild_id).unwrap();

        guild
            .presences
            .get(&event.user.id)
            .map(|p| p.client_status.clone())
    };

    let mut client_stat = vec![];
    if let Some(Some(client)) = online_status {
        if client.desktop.is_some() {
            client_stat.push("Desktop");
        }
        if client.mobile.is_some() {
            client_stat.push("Mobile");
        }
        if client.web.is_some() {
            client_stat.push("Web");
        }
    }
    let stats = client_stat.join(", ");

    let mut embed = CreateEmbed::new()
        .author(
            CreateEmbedAuthor::new(format!("{} dm activity flag updated!", event.user.name))
                .icon_url(event.user.face()),
        )
        .field("User", user_ping, true)
        .field("Joined at", format!("<t:{joined_at}:R>"), true)
        .field("Creation Date", format!("<t:{created_at}:R>"), true)
        .footer(CreateEmbedFooter::new(format!(
            "User ID: {}",
            event.user.id
        )));

    if count != 0 {
        embed = embed.footer(CreateEmbedFooter::new(format!(
            "User ID: {} • Previous hits: {}",
            event.user.id, count
        )));
    }

    if !stats.is_empty() {
        embed = embed.description(format!("**Online on**:\n{stats}"));
    }

    ChannelId::new(158484765136125952)
        .send_message(ctx, serenity::CreateMessage::default().embed(embed))
        .await?;

    Ok(())
}
