use crate::helper::get_guild_name;
use crate::{Data, Error};
use poise::serenity_prelude::{
    self as serenity, ActivityData, ActivityType, GuildId, Ready, UserId,
};
use sqlx::query;

use small_fixed_array::FixedString;

use std::sync::atomic::Ordering;
use std::sync::Arc;

pub async fn ready(ctx: &serenity::Context, ready: &Ready, data: Arc<Data>) -> Result<(), Error> {
    let activity_data = ActivityData {
        name: FixedString::from_str_trunc("you inside your home."),
        kind: ActivityType::Watching,
        state: None,
        url: None,
    };
    ctx.set_activity(Some(activity_data));

    let shard_count = ctx.cache.shard_count();
    let is_last_shard = (ctx.shard_id.0 + 1) == shard_count.get();

    if is_last_shard && !data.has_started.load(Ordering::SeqCst) {
        finalize_start(ctx, &data);
        println!("Logged in as {}", ready.user.tag());
    }

    Ok(())
}

fn finalize_start(ctx: &serenity::Context, data: &Arc<Data>) {
    let ctx_clone = ctx.clone();
    let data_clone = data.clone();

    tokio::spawn(async move {
        let mut interval: tokio::time::Interval =
            tokio::time::interval(std::time::Duration::from_secs(60 * 10));
        loop {
            interval.tick().await;
            let _ = crate::tasks::check_space(&ctx_clone, &data_clone).await;
        }
    });

    data.has_started.store(true, Ordering::SeqCst);
}

// TODO: Cache join tracking.
pub async fn cache_ready(
    ctx: &serenity::Context,
    guilds: &Vec<GuildId>,
    data: Arc<Data>,
) -> Result<(), Error> {
    let db_pool = &data.db;

    for guild in guilds {
        let guild_name: String = get_guild_name(ctx, Some(*guild));
        let result = query!(
            "SELECT author_id, user_id FROM join_tracks WHERE guild_id = $1",
            guild.get() as i64
        )
        .fetch_all(db_pool)
        .await;

        if let Ok(records) = result {
            for record in records {
                let authorid = UserId::new(record.author_id.unwrap() as u64);
                let userid = UserId::new(record.user_id.unwrap() as u64);

                // Author is still in guild.
                if let Ok(author) = guild.member(ctx, authorid).await {
                    // Should check if user exists first?
                    if let Ok(member) = guild.member(ctx, userid).await {
                        let reply_content = format!(
                            "{} (<@{}>) joined {}!",
                            member.user.tag(),
                            member.user.id,
                            guild_name
                        );
                        let reply_builder =
                            serenity::CreateMessage::default().content(reply_content);
                        author.user.dm(ctx, reply_builder).await?;
                    }
                } else {
                    let _ = query!(
                        "DELETE FROM join_tracks WHERE author_id = $1 AND guild_id = $2",
                        i64::from(authorid),
                        guild.get() as i64,
                    )
                    .execute(db_pool)
                    .await;
                }
            }
        }
    }

    Ok(())
}
