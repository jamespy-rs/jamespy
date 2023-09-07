use poise::serenity_prelude as serenity;
use ::serenity::builder::CreateEmbedFooter;
use sqlx::{query, Row};
use crate::{Context, Error};


#[poise::command(rename = "dbstats", aliases("db-stats", "db-info"), prefix_command, category = "Database", owners_only, hide_in_help)]
pub async fn dbstats(ctx: Context<'_>) -> Result<(), Error> {
    let db_pool = &ctx.data().db;
    let table_info = vec![
        ("msgs", "message_id"),
        ("msgs_edits", "message_id"),
        ("msgs_deletions", "message_id"),
        ("snippets", "name")
    ];

    let mut embed = serenity::CreateEmbed::default()
        .title("Database Stats");

    for (table_name, pk_column) in table_info {
        let sql_query = format!("SELECT COUNT({}) FROM {}", pk_column, table_name);

        let row = query(&sql_query).fetch_one(db_pool).await?;

        let count: i64 = row.get(0);

        embed = embed.field(table_name, count.to_string(), false);
    }

    let db_size_query = "SELECT pg_database_size(current_database())";
    let row = query(db_size_query).fetch_one(db_pool).await?;
    let db_size_bytes: i64 = row.get(0);
    let db_size = format!("{:.2} MB", db_size_bytes as f64 / (1024.0 * 1024.0));


    embed = embed.footer(CreateEmbedFooter::new(format!("Database size: {}", db_size)));
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(rename = "sql", prefix_command, category = "Database", owners_only, hide_in_help)]
pub async fn sql(
    ctx: Context<'_>,
    #[description = "SQL query"] #[rest] query: String,
) -> Result<(), Error> {
    let sql_query = query.clone();
    let db_pool = &ctx.data().db;

    let now = std::time::Instant::now();

    println!("\x1B[31;40mWARNING: SQL COMMAND WAS TRIGGERED\x1B[0m");

    let result = sqlx::query(&sql_query)
        .fetch_optional(db_pool)
        .await;

    let elapsed = now.elapsed().as_millis();

    match result {
        Ok(Some(row)) => {
            if row.len() == 1 && row.try_get::<i64, _>(0).is_ok() {
                let count = row.get::<i64, _>(0);
                let formatted = format!("Counted {} rows in {}ms", count, elapsed);
                let message = poise::CreateReply::default().content(formatted);
                ctx.send(message).await?;

            } else {
                let formatted = format!("Query executed successfully iqn {}ms", elapsed);
                let message = poise::CreateReply::default().content(formatted);
                ctx.send(message).await?;
            }
        }
        Ok(None) => {
            let formatted = format!("Query executed successfully in {}ms", elapsed);
            let message = poise::CreateReply::default().content(formatted);
            ctx.send(message).await?;
        }
        Err(err) => {
            let error_message = format!("Error executing query: {:?}", err);
            let message = poise::CreateReply::default().content(error_message);
            ctx.send(message).await?;
        }
    }

    Ok(())
}

