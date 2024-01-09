use crate::{Data, Error};
use jamespy_utils::misc::get_guild_name;
use poise::serenity_prelude::{self as serenity, ActivityData, ActivityType, GuildId, UserId};
use sqlx::query;

pub async fn ready(ctx: &serenity::Context, data: &Data) -> Result<(), Error> {
    let _ = data;

    let activity_data = ActivityData {
        name: "you inside your home.".to_string().into(),
        kind: ActivityType::Watching,
        state: None,
        url: None,
    };
    ctx.set_activity(Some(activity_data));

    Ok(())
}

// TODO: Cache join tracking.
pub async fn cache_ready(
    ctx: &serenity::Context,
    guilds: Vec<GuildId>,
    data: &Data,
) -> Result<(), Error> {
    println!("{:?}", ctx.cache.settings().clone());
    println!("done");
    let db_pool = &data.db;

    for guild in guilds {
        let guild_name = get_guild_name(ctx, guild);
        let result = query!(
            "SELECT author_id, user_id FROM join_tracks WHERE guild_id = $1",
            i64::from(guild)
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
                            member.user.name, member.user.id, guild_name
                        );
                        let reply_builder =
                            serenity::CreateMessage::default().content(reply_content);
                        author.user.dm(ctx, reply_builder).await?;
                    }
                } else {
                    let _ = query!(
                        "DELETE FROM join_tracks WHERE author_id = $1 AND guild_id = $2",
                        i64::from(authorid),
                        i64::from(guild)
                    )
                    .execute(db_pool)
                    .await;
                }
            }
        }
    }

    Ok(())
}
