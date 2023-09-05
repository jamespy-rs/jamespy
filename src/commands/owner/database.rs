 use poise::serenity_prelude as serenity;
use sqlx::{query, Row};
use crate::{Context, Data, Error};

#[poise::command(rename = "dbstats", prefix_command, category = "Database", owners_only, hide_in_help)]
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
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
