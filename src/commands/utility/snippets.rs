use bb8_redis::redis::AsyncCommands;
use poise::serenity_prelude::{Colour, CreateEmbed};
use regex::Regex;
use crate::{Context, Error, Data};

pub async fn save_snippet(_ctx: &Context<'_>, guild_id: i64, data: &Data, snippet_name: &str, snippet_properties: &[(&str, &str)]) -> Result<(), Error> {
    let redis_pool = &data.redis;
    let db_pool = &data.db;

    let snippet_key = format!("snippet:{}:{}", guild_id, snippet_name);

    let mut redis_conn = redis_pool.get().await?;

    redis_conn
        .hset_multiple(&snippet_key, snippet_properties)
        .await?;

    let mut title = None;
    let mut description = None;
    let mut image = None;
    let mut thumbnail = None;
    let mut color = None;

    for (prop_name, prop_value) in snippet_properties {
        match *prop_name {
            "title" => title = Some(prop_value),
            "description" => description = Some(prop_value),
            "image" => image = Some(prop_value),
            "thumbnail" => thumbnail = Some(prop_value),
            "color" => color = Some(prop_value),
            _ => (),
        }
    }

    sqlx::query!(
        "INSERT INTO snippets (guild_id, name, title, description, image, thumbnail, color)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        guild_id,
        snippet_name,
        title,
        description,
        image,
        thumbnail,
        color
    )
    .execute(db_pool)
    .await?;

    Ok(())
}


#[poise::command(slash_command, aliases("remove-snippet", "delsnippet", "del-snippet"), guild_only, category = "Utility", required_permissions = "MANAGE_MESSAGES")]
pub async fn removesnippet(ctx: Context<'_>, snippet_name: String) -> Result<(), Error> {
    let guild_id: i64 = ctx.guild_id().unwrap().0 as i64;
    let snippet_key = format!("snippet:{}:{}", guild_id, snippet_name);

    let redis_pool = &ctx.data().redis;
    let db_pool = &ctx.data().db;

    let mut redis_conn = redis_pool.get().await?;

    // Remove snippet data from Redis
    let deleted: i64 = redis_conn.del(&snippet_key).await?;

    // Perform the SQL deletion
    sqlx::query!(
        "DELETE FROM snippets
         WHERE guild_id = $1 AND name = $2",
        guild_id,
        &snippet_name
    )
    .execute(db_pool)
    .await?;

    if deleted > 0 {
        ctx.say(format!("Snippet '{}' has been removed.", snippet_name)).await?;
    } else {
        ctx.say(format!("Snippet '{}' not found.", snippet_name)).await?;
    }

    Ok(())
}



// No idea how to set the actual name of the command so I'm going to change it to setsnippet for now.
/// set a snippet for everyone to use!
#[poise::command(slash_command, guild_only, aliases("setsnippet", "set_snippet", "setsnippets", "set_snippets"), category = "Utility", required_permissions = "MANAGE_MESSAGES", user_cooldown = "3")]
pub async fn setsnippet(
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
    if name.len() > 32 {
        ctx.say("Snippet name must be 32 characters or less.").await?;
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

#[poise::command(slash_command, prefix_command, guild_only, category = "Utility")]
pub async fn snippet(
    ctx: Context<'_>,
    #[description = "The name of the snippet"] name: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0 as i64;
    let snippet_key = format!("snippet:{}:{}", guild_id, name);

    let redis_pool = &ctx.data().redis;
    let mut redis_conn = redis_pool.get().await?;

    let snippet_properties: Vec<(String, String)> = redis_conn.hgetall(&snippet_key).await?;

    if snippet_properties.is_empty() {
        ctx.say("Snippet not found.").await?;
        return Ok(());
    }

    ctx.send(|e| {
        e.embed(|e| {
            let _embed = CreateEmbed::default();

            for (key, value) in &snippet_properties {
                match key.as_str() {
                    "title" => {
                        e.title(value);
                    }
                    "description" => {
                        e.description(value.replace("\\n", "\n"));
                    }
                    "image" => {
                        e.image(value);
                    }
                    "thumbnail" => {
                        e.thumbnail(value);
                    }
                    "color" => {
                        if let Some(color) = parse_colour(value) {
                            e.color(color);
                        }
                    }
                    _ => {}
                }
            }
            e
        })
    })
    .await?;

    Ok(())
}


fn parse_colour(value: &str) -> Option<Colour> {
    let valid_colour = regex::Regex::new(r"^(#[0-9A-Fa-f]{6}|[0-9A-Fa-f]{6})$").unwrap();
    if valid_colour.is_match(value) {
        let rgb = u32::from_str_radix(&value[1..], 16).ok()?;
        let red = ((rgb >> 16) & 0xFF) as u8;
        let green = ((rgb >> 8) & 0xFF) as u8;
        let blue = (rgb & 0xFF) as u8;
        Some(Colour::from_rgb(red, green, blue))
    } else {
        None
    }
}

#[poise::command(slash_command, prefix_command, aliases("list-snippets", "list_snippets", "list_snippet", "list-snippet"), guild_only, category = "Utility")]
pub async fn listsnippets(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().0 as i64;
    let snippet_prefix = format!("snippet:{}:", guild_id);

    let redis_pool = &ctx.data().redis;
    let mut redis_conn = redis_pool.get().await?;

    let snippet_keys: Vec<String> = redis_conn.keys(format!("{}*", snippet_prefix)).await?;

    if snippet_keys.is_empty() {
        ctx.say("No snippets found.").await?;
        return Ok(());
    }

    let snippet_names: Vec<String> = snippet_keys
        .into_iter()
        .map(|key| format!("`{}`", key.trim_start_matches(&snippet_prefix)))
        .collect();

    let snippet_list = snippet_names.join("\n");

    ctx.send(|e| {
        e.embed(|e| {
            e.title("Snippets");
            e.description(format!("{}", snippet_list));
            e.color(Colour::from_rgb(0, 255, 0))
        })
    })
    .await?;
    Ok(())
}

