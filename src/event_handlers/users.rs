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
            if guild_id != 98226572468690944 {
                return Ok(());
            }

            if let Some(old_stamp) = data.dm_activity.get(&event.user.id) {
                // if there is an old timestamp in there, but its in the past, remove.
                if old_stamp.unix_timestamp() < Utc::now().timestamp() {
                    data.dm_activity.remove(&event.user.id);
                    return Ok(());
                }

                if timestamp.timestamp() >= (old_stamp.timestamp() + 3600) {
                    dm_activity_updated(ctx, &event, *old_stamp).await?;
                    data.dm_activity.insert(event.user.id, timestamp);
                    return Ok(());
                }
            } else if timestamp.unix_timestamp() >= Utc::now().timestamp() {
                // add, no previous match but in future.
                dm_activity_new(ctx, &event).await?;
                data.dm_activity.insert(event.user.id, timestamp);
            }
        }
    }

    Ok(())
}

async fn dm_activity_new(
    ctx: &serenity::Context,
    event: &GuildMemberUpdateEvent,
) -> Result<(), Error> {
    let user_ping = format!("<@{}>", event.user.id);
    let joined_at = event.joined_at.unix_timestamp();
    let created_at = event.user.created_at().unix_timestamp();
    let embed = CreateEmbed::new()
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
    ChannelId::new(158484765136125952)
        .send_message(ctx, serenity::CreateMessage::default().embed(embed))
        .await?;

    Ok(())
}

async fn dm_activity_updated(
    ctx: &serenity::Context,
    event: &GuildMemberUpdateEvent,
    old_stamp: serenity::Timestamp,
) -> Result<(), Error> {
    let user_ping = format!("<@{}>", event.user.id);
    let joined_at = event.joined_at.unix_timestamp();
    let created_at = event.user.created_at().unix_timestamp();
    let old = old_stamp.unix_timestamp();
    let new = event.unusual_dm_activity_until.unwrap().unix_timestamp();
    let embed = CreateEmbed::new()
        .author(
            CreateEmbedAuthor::new(format!("{} dm activity flag updated!", event.user.name))
                .icon_url(event.user.face()),
        )
        .field("User", user_ping, true)
        .field("Joined at", format!("<t:{joined_at}:R>"), true)
        .field("Creation Date", format!("<t:{created_at}:R>"), true)
        .field("Old", format!("<t:{old}:R>"), true)
        .field("New", format!("<t:{new}:R>"), true)
        .footer(CreateEmbedFooter::new(format!(
            "User ID: {}",
            event.user.id
        )));
    ChannelId::new(158484765136125952)
        .send_message(ctx, serenity::CreateMessage::default().embed(embed))
        .await?;

    Ok(())
}
