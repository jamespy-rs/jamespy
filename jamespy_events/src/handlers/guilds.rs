use sqlx::query;

use crate::{helper::get_guild_name, Data, Error};
use poise::serenity_prelude::{
    self as serenity, AuditLogEntry, AutoModAction, ChannelId, CreateEmbedAuthor, Guild, GuildId,
    Member, User, UserId,
};
use serenity::model::guild::audit_log::Action;

pub async fn guild_create(
    ctx: &serenity::Context,
    guild: &Guild,
    is_new: &Option<bool>,
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
    new_member: &Member,
    data: &Data,
) -> Result<(), Error> {
    let db_pool = &data.db;
    let guild_id = new_member.guild_id;
    let joined_user_id = new_member.user.id;

    let guild_name = get_guild_name(ctx, Some(guild_id));

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
    guild_id: &GuildId,
    user: &User,
    data: &Data,
) -> Result<(), Error> {
    let db_pool = &data.db;
    let guild_name = get_guild_name(ctx, Some(*guild_id));

    println!(
        "\x1B[33m[{}] {} (ID:{}) has left!\x1B[0m",
        guild_name, user.name, user.id
    );

    // Author left guild, these are no longer important.
    let _ = query!(
        "DELETE FROM join_tracks WHERE author_id = $1 AND guild_id = $2",
        i64::from(user.id),
        i64::from(*guild_id)
    )
    .execute(db_pool)
    .await;

    Ok(())
}

pub async fn guild_audit_log_entry_create(
    ctx: &serenity::Context,
    entry: &AuditLogEntry,
    guild_id: &GuildId,
) -> Result<(), Error> {
    if *guild_id != 98226572468690944 {
        return Ok(());
    }

    if !matches!(entry.action, Action::AutoMod(AutoModAction::FlagToChannel)) {
        return Ok(());
    }

    if entry.reason.is_none() {
        return Ok(());
    }
    let reason = entry.reason.as_ref().unwrap();

    if !reason.starts_with("Voice Channel Status") {
        return Ok(());
    }

    let (user_name, avatar_url) = {
        let user = entry.user_id.to_user(&ctx).await.unwrap();
        (user.name.clone(), user.face())
    };

    let (check_contents, culprit_channel_id): (Option<u64>, Option<ChannelId>) =
        if let Some(options) = &entry.options {
            (
                match &options.auto_moderation_rule_name {
                    Some(rule_name) => match rule_name.as_str() {
                        "Bad Words ❌ [BLOCKED]" => Some(697738506944118814),
                        _ => None,
                    },
                    None => None,
                },
                options.channel_id, // culprit.
            )
        } else {
            (None, None)
        };

    // use channel_id instead.
    if let Some(id) = check_contents {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let cloned_messages = ctx.cache.channel_messages(id.into()).map(|c| c.clone());

        let mut status = format!(
            "Unknown (check #{})",
            ChannelId::new(id)
                .name(&ctx)
                .await
                .unwrap_or("Unknown".to_string())
        )
        .to_string();

        // its rather expensive to iterate and sort upwards of 350 messages.
        // so i'll fix it later.
        // alongside the horrible usage of indents.
        if let Some(msg_clones) = cloned_messages {
            let mut sorted_msgs: Vec<_> = msg_clones.into_iter().collect();
            sorted_msgs.sort_by(|a, b| b.0.cmp(&a.0));

            for msg in sorted_msgs {
                if msg.1.author.id == entry.user_id {
                    if let Some(kind) = &msg.1.embeds.first().and_then(|e| e.kind.clone()) {
                        if kind == "auto_moderation_message" {
                            if let Some(description) = &msg.1.embeds[0].description {
                                status = description.to_string();
                                break;
                            }
                        }
                    }
                }
            }
        };

        let author_title = format!("{user_name} tried to set an inappropriate status");
        let footer = serenity::CreateEmbedFooter::new(format!(
            "User ID: {} • Please check status manually in #{}",
            entry.user_id,
            ChannelId::new(id)
                .name(&ctx)
                .await
                .unwrap_or("Unknown".to_string())
        ));
        let mut embed = serenity::CreateEmbed::default()
            .author(CreateEmbedAuthor::new(author_title).icon_url(avatar_url))
            .field("Status", status, true)
            .footer(footer);

        if let Some(channel_id) = culprit_channel_id {
            embed = embed.field("Channel", format!("<#{channel_id}>"), true);
        }

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
    Ok(())
}
