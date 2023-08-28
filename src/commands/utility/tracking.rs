use crate::{Context, Error, Data};
use poise::serenity_prelude::{self as serenity, GuildId, UserId};
use sqlx;

#[poise::command(rename = "track-user", slash_command, prefix_command, aliases("track_user", "trackuser"), guild_only, category = "Utility")]
pub async fn track_user(
    ctx: Context<'_>,
    #[description = "The user that you want to track!"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let author_id = ctx.author().id;

    if let Some(user) = user {
        let tracked_user_id = user.id;

        if let Some(member) = guild_id.member(ctx.http(), tracked_user_id).await.ok() {
            ctx.say(format!("{} is already a member of this guild and cannot be tracked.", member.display_name())).await?;
        } else if is_user_already_tracking(&ctx, guild_id, tracked_user_id, author_id, ctx.data()).await {
            ctx.say("You are already tracking this user.").await?;
        } else {
            add_tracked_user(&ctx, guild_id, tracked_user_id, author_id, ctx.data()).await;
        }
    }

    Ok(())
}


async fn fetch_user_name(ctx: &Context<'_>, user_id: UserId) -> Option<String> {
    match user_id.to_user(&ctx.http()).await {
        Ok(user) => Some(user.name),
        Err(_) => None,
    }
}

async fn add_tracked_user(ctx: &Context<'_>, guild_id: GuildId, tracked_user_id: UserId, author_id: UserId, data: &Data) {
    let db_pool = &data.db;

    let tracked_user_name = match fetch_user_name(&ctx, tracked_user_id).await {
        Some(name) => name,
        None => {
            ctx.say("Failed to fetch tracked user's name from the API.")
                .await
                .expect("Failed to send message");
            return;
        }
    };

    let query_result = sqlx::query!(
        "INSERT INTO join_tracks (guild_id, author_id, user_id) VALUES ($1, $2, $3)",
        guild_id.0 as i64,
        author_id.0 as i64,
        tracked_user_id.0 as i64
    )
    .execute(&*db_pool)
    .await;

    match query_result {
        Ok(_) => {
            ctx.say(format!(
                "Added {} (ID:{}) to your track list for this guild!",
                tracked_user_name, tracked_user_id
            ))
            .await
            .expect("Failed to send message");
        }
        Err(err) => {
            eprintln!("Failed to add tracked user: {:?}", err);
        }
    }
}

async fn is_user_already_tracking(_ctx: &Context<'_>, guild_id: GuildId, tracked_user_id: UserId, author_id: UserId, data: &Data) -> bool {
    let db_pool = &data.db;

    let query_result = sqlx::query!(
        "SELECT COUNT(*) AS count FROM join_tracks WHERE guild_id = $1 AND author_id = $2 AND user_id = $3",
        guild_id.0 as i64,
        author_id.0 as i64,
        tracked_user_id.0 as i64
    )
    .fetch_one(db_pool)
    .await;

    match query_result {
        Ok(row) => row.count.map_or(false, |count| count > 0),
        Err(err) => {
            eprintln!("Failed to check tracking status: {:?}", err);
            false
        }
    }
}


#[poise::command(rename = "untrack-user", slash_command, prefix_command, aliases("untrackuser", "untrack_user", "remove-track", "delete-track", "del-track", "untrack", "un-track"), guild_only, category = "Utility")]
pub async fn untrack_user(
    ctx: Context<'_>,
    #[description = "The user that you want to untrack!"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let author_id = ctx.author().id;

    if let Some(user) = user {
        let tracked_user_id = user.id;

        if is_user_already_tracking(&ctx, guild_id, tracked_user_id, author_id, ctx.data()).await {
            remove_tracked_user(&ctx, guild_id, tracked_user_id, author_id, ctx.data()).await;
        } else {
            ctx.say("You are not tracking this user.").await?;
        }
    }

    Ok(())
}

async fn remove_tracked_user(ctx: &Context<'_>, guild_id: GuildId, tracked_user_id: UserId, author_id: UserId, data: &Data) {
    let db_pool = &data.db;

    let query_result = sqlx::query!(
        "DELETE FROM join_tracks WHERE guild_id = $1 AND author_id = $2 AND user_id = $3",
        guild_id.0 as i64,
        author_id.0 as i64,
        tracked_user_id.0 as i64
    )
    .execute(db_pool)
    .await;

    match query_result {
        Ok(_) => {
            ctx.say(format!(
                "Removed user {} (ID:{}) from your track list for this guild!",
                tracked_user_id, tracked_user_id
            ))
            .await
            .expect("Failed to send message");
        }
        Err(err) => {
            eprintln!("Failed to remove tracked user: {:?}", err);
        }
    }
}

#[poise::command(rename = "tracked-users", slash_command, prefix_command, aliases("tracked_users", "listtracked", "list-tracked"), guild_only, category = "Utility")]
pub async fn tracked_users(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let author_id = ctx.author().id;

    let tracked_users = get_tracked_users(&ctx, guild_id, author_id, ctx.data()).await;

    if !tracked_users.is_empty() {
        let mut message = "You are tracking the following users in this guild:\n".to_string();
        for user in tracked_users {
            message.push_str(&format!("{} (ID:{}), ", user.0, user.1));
        }
        ctx.say(message).await?;
    } else {
        ctx.say("You are not tracking any users in this guild.").await?;
    }

    Ok(())
}

async fn get_tracked_users(ctx: &Context<'_>, guild_id: GuildId, author_id: UserId, data: &Data) -> Vec<(String, UserId)> {
    let db_pool = &data.db;

    let query_result = sqlx::query!(
        "SELECT user_id FROM join_tracks WHERE guild_id = $1 AND author_id = $2",
        guild_id.0 as i64,
        author_id.0 as i64
    )
    .fetch_all(db_pool)
    .await;

    match query_result {
        Ok(rows) => {
            let mut tracked_users = Vec::new();
            for row in rows {
                if let Some(user_id) = row.user_id {
                    if let Some(user_name) = fetch_user_name(&ctx, UserId(user_id as u64)).await {
                        tracked_users.push((user_name, UserId(user_id as u64)));
                    }
                }
            }

            tracked_users
        }
        Err(err) => {
            eprintln!("Failed to retrieve tracked users: {:?}", err);
            Vec::new()
        }
    }
}
