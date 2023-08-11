use bb8_redis::redis::AsyncCommands;
use poise::serenity_prelude::Colour;
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


pub fn parse_colour(value: &str) -> Option<Colour> {
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

pub async fn set_all_snippets(data: &Data) -> Result<(), Box<dyn std::error::Error>> {
    let snippets_data = sqlx::query!(
        "SELECT guild_id, name, title, description, image, thumbnail, color FROM snippets"
    )
    .fetch_all(&data.db)
    .await?;

    for snippet in snippets_data {
        let guild_id = snippet.guild_id.unwrap_or_default();
        let snippet_name: &String = &snippet.name;

        let snippet_key = format!("snippet:{}:{}", guild_id, snippet_name);

        let mut redis_conn = data.redis.get().await?;

        let name = snippet_name.to_owned();
        let title = snippet.title.as_ref().unwrap_or(&"".to_string()).to_owned();
        let description = snippet.description.as_ref().unwrap_or(&"".to_string()).to_owned();
        let image = snippet.image.as_ref().unwrap_or(&"".to_string()).to_owned();
        let thumbnail = snippet.thumbnail.as_ref().unwrap_or(&"".to_string()).to_owned();
        let color = snippet.color.as_ref().unwrap_or(&"".to_string()).to_owned();

        let snippet_properties = vec![
            ("name", &name),
            ("title", &title),
            ("description", &description),
            ("image", &image),
            ("thumbnail", &thumbnail),
            ("color", &color),
        ];

        redis_conn
            .hset_multiple(&snippet_key, &snippet_properties)
            .await?;
    }

    Ok(())
}
