use crate::{Context, Error};
use poise::serenity_prelude::{
    self as serenity, ComponentInteractionCollector, CreateActionRow, CreateInteractionResponse,
    EmojiId, StickerFormatType,
};
use std::fmt::Write;

#[poise::command(slash_command, prefix_command, category = "Utility", guild_only)]
pub async fn stickers(ctx: Context<'_>) -> Result<(), Error> {
    let stickers = {
        let guild = ctx.guild().unwrap();
        guild.stickers.clone()
    };

    let mut pages = vec![];
    for sticker in stickers {
        let mut embed =
            serenity::CreateEmbed::new().title(format!("{} (ID:{})", sticker.name, sticker.id));

        let mut description = String::new();
        if let Some(desc) = sticker.description.clone() {
            println!("{}: {}", sticker.name, desc.len());
            writeln!(description, "**Description:** {desc}").unwrap();
        };

        // if it can be parsed its just numbers and therefore a guild emote.
        // or it was custom set outside the discord client and is just random numbers.
        let related_emoji = if let Ok(id) = sticker.tags[0].parse::<u64>() {
            if let Some(emoji) = ctx.guild().unwrap().emojis.get(&EmojiId::from(id)) {
                format!("{emoji}")
            } else {
                id.to_string()
            }
        } else {
            let emoji_regex = regex::Regex::new(r"[\p{Emoji}]+").unwrap();

            // technically this isn't flawless given discord lets you put random text
            // if you just use the api directly.
            // at least that is what i think.
            if emoji_regex.is_match(&sticker.tags[0]) {
                sticker.tags[0].to_string()
            } else {
                format!(":{}:", sticker.tags[0])
            }
        };

        writeln!(description, "**Related Emoji:** {related_emoji}").unwrap();

        writeln!(
            description,
            "**Format Type:** {}",
            sticker_format_type_str(&sticker.format_type)
        )
        .unwrap();
        writeln!(description, "**Available:** {}", sticker.available).unwrap();
        embed = embed.description(description);

        if let Some(url) = sticker.image_url() {
            embed = embed.thumbnail(url);
        }

        pages.push(embed);
    }

    let ctx_id = ctx.id();
    let prev_button_id = format!("{ctx_id}prev");
    let next_button_id = format!("{ctx_id}next");

    let mut current_page = 0;

    let msg = ctx
        .send(
            poise::CreateReply::default()
                .embed(pages[0].clone())
                .components(vec![CreateActionRow::Buttons(vec![
                    serenity::CreateButton::new(&prev_button_id).emoji('◀'),
                    serenity::CreateButton::new(&next_button_id).emoji('▶'),
                ])]),
        )
        .await?;

    while let Some(press) = ComponentInteractionCollector::new(ctx.serenity_context().shard.clone())
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(180))
        .await
    {
        if press.data.custom_id == next_button_id {
            current_page += 1;
            if current_page >= pages.len() {
                current_page = 0;
            }
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(pages.len() - 1);
        } else {
            continue;
        }

        press
            .create_response(
                ctx.http(),
                CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::default()
                        .embed(pages[current_page].clone()),
                ),
            )
            .await?;
    }
    // clear components.
    msg.edit(
        ctx,
        poise::CreateReply::new()
            .embed(pages[current_page].clone())
            .components(vec![]),
    )
    .await?;

    Ok(())
}

fn sticker_format_type_str(sticker_fmt: &StickerFormatType) -> &str {
    match *sticker_fmt {
        StickerFormatType::Png => "PNG",
        StickerFormatType::Lottie => "LOTTIE",
        StickerFormatType::Apng => "APNG",
        StickerFormatType::Gif => "GIF",
        _ => "Unknown",
    }
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [stickers()]
}
