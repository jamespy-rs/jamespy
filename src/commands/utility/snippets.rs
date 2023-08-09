use bb8_redis::redis::AsyncCommands;
use sqlx::query;
use regex::Regex;
use crate::{Context, Error, Data};

pub async fn save_snippet(_ctx: &Context<'_>, guild_id: i64, data: &Data, snippet_name: &str, snippet_properties: &[(&str, &str)]) -> Result<(), Error> {
    let redis_pool = &data.redis;

    let snippet_key = format!("snippet:{}:{}", guild_id, snippet_name);

    let mut redis_conn = redis_pool.get().await?;

    redis_conn
        .hset_multiple(&snippet_key, snippet_properties)
        .await?;
    // Need to save persistently.

    Ok(())
}

/// set a snippet for everyone to use!
#[poise::command(slash_command, guild_only, category = "Utility", required_permissions = "MANAGE_MESSAGES", user_cooldown = "3")]
pub async fn set_snippet(
    ctx: Context<'_>,
    #[description = "The name of the snippet"]
    name: String,
    #[description = "The title of the snippet"]
    title: Option<String>,
    #[description = "The description of the snippet"]
    description: Option<String>,
    #[description = "The image URL of the snippet"]
    image: Option<String>,
    #[description = "The thumbnail URL of the snippet"]
    thumbnail: Option<String>,
    #[description = "The color of the snippet"]
    color: Option<String>,
) -> Result<(), Error> {
    let at_least_one_property_set = title.is_some() || description.is_some() || image.is_some() || thumbnail.is_some();

    if !at_least_one_property_set {
        ctx.say("Please provide at least one of title, description, image, or thumbnail.").await?;
        return Ok(());
    }

    let name_regex = Regex::new(r"^[a-zA-Z0-9\-_.]+$").unwrap(); // enforces only some characters.
    if !name_regex.is_match(&name) {
        ctx.say("Invalid name format. It should only contain letters (a-z), hyphens (-), underscores (_), and periods (.)").await?;
        return Ok(());
    }

    let valid_colour = Regex::new(r"^(#[0-9A-Fa-f]{6}|[0-9A-Fa-f]{6})$").unwrap();
    if let Some(ref color) = color {
        if !valid_colour.is_match(color) {
            ctx.say("Invalid hex color format!").await?;
            return Ok(());
        }
    }

    let guild_id = ctx.guild_id().unwrap().0 as i64;

    save_snippet(
        &ctx,
        guild_id,
        ctx.data(),
        &name,
        &[
            ("title", title.as_deref().unwrap_or_default()),
            ("description", description.as_deref().unwrap_or_default()),
            ("image", image.as_deref().unwrap_or_default()),
            ("thumbnail", thumbnail.as_deref().unwrap_or_default()),
            ("color", color.as_deref().unwrap_or_default()),
        ],
    ).await?;

    ctx.say("Snippet saved successfully!").await?;

    Ok(())
}

/// Show a snippet.
#[poise::command(slash_command, prefix_command, guild_only, category = "Utility")]
pub async fn snippet(
    ctx: Context<'_>,
    #[description = "The name of the snippet"]
    name: String,
) -> Result<(), Error> {
    // Need to cap name length.
    let name_regex = Regex::new(r"^[a-zA-Z0-9\-_.]+$").unwrap(); // enforces only some characters.
    if !name_regex.is_match(&name) {
        ctx.say("Invalid name format. It should only contain letters (a-z), hyphens (-), underscores (_), and periods (.)").await?;
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap().0 as i64;
    let snippet_key = format!("snippet:{}:{}", guild_id, name);

    let redis_pool = &ctx.data().redis;
    let mut redis_conn = redis_pool.get().await?;

    let snippet_properties: Vec<(String, String)> = redis_conn.hgetall(&snippet_key).await?;

    if snippet_properties.is_empty() {
        ctx.say("Snippet not found.").await?;
        return Ok(());
    }

    let mut properties_string = String::new();
    for (key, value) in snippet_properties {
        properties_string.push_str(&format!("{}: {}\n", key, value));
    }

    ctx.say(properties_string).await?;

    Ok(())
}
