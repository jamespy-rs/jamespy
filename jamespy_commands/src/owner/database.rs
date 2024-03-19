use crate::{owner::owner, Context, Error};
use poise::serenity_prelude::{self as serenity, CreateEmbedFooter};
use sqlx::{query, Pool, Postgres, Row};
use std::fmt::Write;

#[poise::command(
    rename = "dbstats",
    aliases("db-stats", "db-info"),
    prefix_command,
    category = "Owner - Database",
    check = "owner",
    hide_in_help
)]
pub async fn dbstats(ctx: Context<'_>) -> Result<(), Error> {
    let db_pool = &ctx.data().db;

    let messages_tables = [
        ("msgs", "message_id"),
        ("msgs_edits", "message_id"),
        ("msgs_deletions", "message_id"),
    ];
    let names_tables = [
        ("usernames", "user_id"),
        ("global_names", "user_id"),
        ("nicknames", "user_id"),
    ];
    let misc_tables = [("join_tracks", "user_id"), ("dm_activity", "user_id")];

    let mut embed = serenity::CreateEmbed::default().title("Database Stats");

    let messages_info = query_table_info(db_pool, &messages_tables).await?;
    embed = embed.field("Messages", messages_info, true);

    let names_info = query_table_info(db_pool, &names_tables).await?;
    embed = embed.field("Names", names_info, true);

    let misc_info = query_table_info(db_pool, &misc_tables).await?;
    embed = embed.field("Miscellaneous", misc_info, true);

    let db_size_query = "SELECT pg_database_size(current_database())";
    let row = query(db_size_query).fetch_one(db_pool).await?;
    let db_size_bytes: i64 = row.get(0);
    let db_size = format!("{:.2} MB", db_size_bytes / (1024 * 1024));

    embed = embed.footer(CreateEmbedFooter::new(format!("Database size: {db_size}")));
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

async fn query_table_info(
    db_pool: &Pool<Postgres>,
    tables: &[(&str, &str)],
) -> Result<String, Error> {
    let mut info = String::new();

    for (table_name, pk_column) in tables {
        let sql_query = format!("SELECT COUNT({pk_column}) FROM {table_name}");
        let row = query(&sql_query).fetch_one(db_pool).await?;

        let count: i64 = row.get(0);

        writeln!(info, "**{table_name}**\n{count}").unwrap();
    }

    Ok(info)
}

/// Inserts all users in the current guild into the names cache.
#[poise::command(
    rename = "insert-all-names",
    prefix_command,
    category = "Owner - Database",
    owners_only,
    guild_only,
    hide_in_help
)]
pub async fn insert_all_users(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    // expensive, maybe find a better way.
    let members = {
        let guild = ctx.cache().guild(guild_id).unwrap();
        guild.members.clone()
    };
    // this is not efficient in the slightest and locks the data several thousand times for massive guilds.
    // I should really provide a better way of doing this in bulk.
    for member in members {
        ctx.data().check_or_insert_user(&member.user).await;
        ctx.data()
            .check_or_insert_nick(guild_id, member.user.id, member.nick.map(|s| s.to_string()))
            .await;
    }

    Ok(())
}

#[poise::command(
    rename = "sql",
    prefix_command,
    category = "Owner - Database",
    owners_only,
    hide_in_help
)]
#[allow(clippy::similar_names)] // "now" and "row" are too close.
pub async fn sql(
    ctx: Context<'_>,
    #[description = "SQL query"]
    #[rest]
    query: String,
) -> Result<(), Error> {
    //TODO: completely overhaul this.
    let sql_query = query;
    let db_pool = &ctx.data().db;

    let now = std::time::Instant::now();

    println!("\x1B[31;40mWARNING: SQL COMMAND WAS TRIGGERED\x1B[0m");

    let result = sqlx::query(&sql_query).fetch_optional(db_pool).await;

    let elapsed = now.elapsed().as_millis();

    match result {
        Ok(Some(row)) => {
            if row.len() == 1 && row.try_get::<i64, _>(0).is_ok() {
                let count = row.get::<i64, _>(0);
                let formatted = format!("Counted {count} rows in {elapsed}ms");
                let message = poise::CreateReply::default().content(formatted);
                ctx.send(message).await?;
            } else {
                let formatted = format!("Query executed successfully in {elapsed}ms");
                let message = poise::CreateReply::default().content(formatted);
                ctx.send(message).await?;
            }
        }
        Ok(None) => {
            let formatted = format!("Query executed successfully in {elapsed}ms");
            let message = poise::CreateReply::default().content(formatted);
            ctx.send(message).await?;
        }
        Err(err) => {
            let error_message = format!("Error executing query: {err:?}");
            let message = poise::CreateReply::default().content(error_message);
            ctx.send(message).await?;
        }
    }

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 3] {
    [dbstats(), insert_all_users(), sql()]
}
