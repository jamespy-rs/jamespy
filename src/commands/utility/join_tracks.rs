use crate::{Context, Error};
use poise::serenity_prelude::{CreateEmbed, GuildId, User, UserId};
use sqlx::query;

/// Track a user joining!
#[poise::command(
    rename = "track-join",
    aliases("stalk", "trackuser", "user-track", "join-track", "track-user"),
    slash_command,
    prefix_command,
    guild_only,
    category = "Utility",
    user_cooldown = "5"
)]
pub async fn track_join(
    ctx: Context<'_>,
    #[description = "User to track"] user: User,
) -> Result<(), Error> {
    let db_pool = &ctx.data().db;

    let memb = ctx.guild_id().unwrap().member(ctx, user.id).await;
    if memb.is_ok() {
        ctx.say("You can't track users already in the guild!")
            .await?;
        return Ok(());
    };

    let result = query!(
        "SELECT user_id FROM join_tracks WHERE author_id = $1 AND guild_id = $2",
        i64::from(ctx.author().id),
        i64::from(ctx.guild_id().unwrap())
    )
    .fetch_one(db_pool)
    .await;

    println!("{:?}", result);
    if result.is_ok() {
        ctx.say("You are already tracking this user!").await?;
        return Ok(());
    }

    let _ = query!(
        "INSERT INTO join_tracks (guild_id, author_id, user_id)
         VALUES ($1, $2, $3)",
        i64::from(ctx.guild_id().unwrap()),
        i64::from(ctx.author().id),
        i64::from(user.id)
    )
    .execute(db_pool)
    .await;

    ctx.say(format!("Successfully started tracking {}", user.name))
        .await?;

    Ok(())
}

/// List your tracked users for a guild!
#[poise::command(
    rename = "list-tracked",
    aliases("list-stalk", "list-stalking", "tracked-users", "tracked-user"),
    slash_command,
    prefix_command,
    guild_only,
    category = "Utility",
    user_cooldown = "5"
)]
pub async fn list_tracked(
    ctx: Context<'_>,
    #[description = "Guild to check"] guild_id: Option<GuildId>,
) -> Result<(), Error> {
    let db_pool = &ctx.data().db;

    let guild_id = guild_id.unwrap_or(ctx.guild_id().unwrap());

    let result = query!(
        "SELECT user_id FROM join_tracks WHERE author_id = $1 AND guild_id = $2",
        i64::from(ctx.author().id),
        i64::from(guild_id)
    )
    .fetch_all(db_pool)
    .await;

    let mut description = String::new();
    if let Ok(rows) = result {
        for row in rows {
            let userid = UserId::new(row.user_id.unwrap() as u64);
            // future me will check for error.
            let user = userid.to_user(ctx).await?;
            description = format!("\n{} (ID:{})", user.name, user.id);
        }
    } else {
        ctx.say("You aren't tracking any users!").await?;
        return Ok(());
    }

    let embed = CreateEmbed::default()
        .title("Tracked Users")
        .description(description);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

// TODO: add autocomplete.
// TODO: add guild.
/// List your tracked users for a guild!
#[poise::command(
    rename = "remove-track",
    slash_command,
    prefix_command,
    guild_only,
    category = "Utility",
    user_cooldown = "5"
)]
pub async fn remove_track(
    ctx: Context<'_>,
    #[description = "User to track"] user: User,
) -> Result<(), Error> {
    let db_pool = &ctx.data().db;

    let _ = query!(
        "DELETE FROM join_tracks WHERE author_id = $1 AND guild_id = $2 AND user_id = $3",
        i64::from(ctx.author().id),
        i64::from(ctx.guild_id().unwrap()),
        i64::from(user.id)
    )
    .execute(db_pool)
    .await;

    ctx.send(
        poise::CreateReply::default()
            .content("Removing track for user if user is currently tracked!"),
    )
    .await?;

    Ok(())
}
