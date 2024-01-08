use std::collections::HashMap;

use sqlx::query;

use crate::{Data, Error};
use jamespy_utils::misc::get_guild_name;
use poise::serenity_prelude::{
    self as serenity, AuditLogEntry, ChannelId, CreateEmbedAuthor, Guild, GuildId, Member, User,
    UserId,
};
use serenity::model::guild::audit_log::Action;

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
    data: &Data,
) -> Result<(), Error> {
    let db_pool = &data.db;
    let guild_id = new_member.guild_id;
    let joined_user_id = new_member.user.id;

    let guild_name = get_guild_name(ctx, guild_id);

    println!(
        "\x1B[33m[{}] {} (ID:{}) has joined!\x1B[0m",
        guild_name, new_member.user.name, joined_user_id
    );

    // Check join tracks
    let result = query!(
        "SELECT * FROM join_tracks WHERE user_id = $1 AND guild_id = $2",
        i64::from(joined_user_id),
        i64::from(new_member.guild_id)
    )
    .fetch_all(db_pool)
    .await;
    let reply_content: &str = &format!(
        "{} (<@{}>) joined {}!",
        new_member.user.name, new_member.user.id, guild_name
    );
    if let Ok(records) = result {
        for record in records {
            let reply_builder = serenity::CreateMessage::default().content(reply_content);
            // record contain the user_id.

            // check author is in guild still, check if author can be dmed.
            // Remove if author can't be dmed.
            let authorid = UserId::new(record.author_id.unwrap() as u64);

            match guild_id.member(ctx, authorid).await {
                Ok(member) => {
                    member.user.dm(ctx, reply_builder).await?;
                    // in the future i should check for if this fails and why, and remove depending on the situation.
                    let _ = query!(
                        "DELETE FROM join_tracks WHERE guild_id = $1 AND author_id = $2 AND \
                         user_id = $3",
                        i64::from(guild_id),
                        i64::from(authorid),
                        i64::from(joined_user_id)
                    )
                    .execute(db_pool)
                    .await;
                }
                Err(_err) => {
                    // In the future the user should be removed if the user isn't valid, but checking that is a bit of a pain.
                }
            }
        }
    };

    Ok(())
}

pub async fn guild_member_removal(
    ctx: &serenity::Context,
    guild_id: GuildId,
    user: User,
    data: &Data,
) -> Result<(), Error> {
    let db_pool = &data.db;
    let guild_name = get_guild_name(ctx, guild_id);

    println!(
        "\x1B[33m[{}] {} (ID:{}) has left!\x1B[0m",
        guild_name, user.name, user.id
    );

    // Author left guild, these are no longer important.
    let _ = query!(
        "DELETE FROM join_tracks WHERE author_id = $1 AND guild_id = $2",
        i64::from(user.id),
        i64::from(guild_id)
    )
    .execute(db_pool)
    .await;

    Ok(())
}

pub async fn guild_audit_log_entry_create(
    ctx: &serenity::Context,
    entry: AuditLogEntry,
    guild_id: GuildId,
) -> Result<(), Error> {
    if guild_id == 98226572468690944 {
        if let Action::AutoMod(serenity::AutoModAction::FlagToChannel) = &entry.action {
            if let Some(reason) = entry.reason {
                if reason.starts_with("Voice Channel Status") {
                    let (user_name, avatar_url) = {
                        let user = entry.user_id.to_user(&ctx).await.unwrap();
                        (user.name.clone(), user.face())
                    };

                    let mut cloned_messages = HashMap::new();

                    let channel_id: Option<u64> = if let Some(options) = entry.options {
                        match options.auto_moderation_rule_name {
                            Some(rule_name) => match rule_name.as_str() {
                                "Bad Words ❌ [BLOCKED]" => Some(697738506944118814),
                                _ => None,
                            },
                            None => None,
                        }
                    } else {
                        None
                    };

                    if let Some(id) = channel_id {
                        tokio::time::sleep(
                            std::time::Duration::from_secs(1)
                                + std::time::Duration::from_millis(500),
                        )
                        .await;
                        if let Some(channel_messages) = ctx.cache.channel_messages(id) {
                            cloned_messages = channel_messages.clone();
                        }
                        let messages: Vec<_> = cloned_messages.values().collect();
                        let last_8_messages = messages.iter().take(8);
                        let messages = last_8_messages;

                        let mut status = format!(
                            "Unknown (check #{})",
                            ChannelId::new(id)
                                .name(&ctx)
                                .await
                                .unwrap_or("Unknown".to_string())
                        )
                        .to_string();

                        for message in messages {
                            if message.author.id == entry.user_id {
                                if let Some(kind) =
                                    &message.embeds.first().and_then(|e| e.kind.clone())
                                {
                                    if kind == "auto_moderation_message" {
                                        if let Some(description) = &message.embeds[0].description {
                                            status = description.to_string();
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        // i'll merge this logic with the above but i'll just do something basic now.
                        // blocked = text logs
                        // alert = war room
                        let author_title = match id {
                            697738506944118814 => {
                                format!("{user_name} tried to set an inappropriate status")
                            }
                            _ => {
                                format!("{user_name} set a possibly inappropriate status")
                            }
                        };

                        let footer = serenity::CreateEmbedFooter::new(format!(
                            "User ID: {} • Please check status manually in #{}",
                            entry.user_id,
                            ChannelId::new(id)
                                .name(&ctx)
                                .await
                                .unwrap_or("Unknown".to_string())
                        ));
                        let embed = serenity::CreateEmbed::default()
                            .author(CreateEmbedAuthor::new(author_title).icon_url(avatar_url))
                            .field("Status", status, true)
                            .footer(footer);
                        let builder = serenity::CreateMessage::default()
                            .embed(embed)
                            .content(format!("<@{}>", entry.user_id));
                        // this is gg/osu only, so i won't enable configurable stuff for this.
                        ChannelId::new(158484765136125952)
                            .send_message(&ctx, builder.clone())
                            .await?;
                        ChannelId::new(1163544192866336808)
                            .send_message(ctx, builder)
                            .await?;
                    }
                }
            }
        }
    }
    Ok(())
}
