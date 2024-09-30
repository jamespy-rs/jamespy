use crate::Error;
use ocrs::{ImageSource, OcrEngine, TextItem};
/* use serenity::all::{
    Attachment, ChannelId, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, Http,
    User,
}; */

use serenity::all::{Attachment, Http, User};

use small_fixed_array::{FixedArray, FixedString};
use strsim::levenshtein;

#[allow(clippy::similar_names)]
pub async fn check(
    engine: &OcrEngine,
    user: &User,
    _: &Http,
    attachments: &FixedArray<Attachment>,
) -> Result<(), Error> {
    let Some(first) = attachments.first() else {
        return Ok(());
    };

    // 20MB
    if first.size > 20_000_000 {
        return Ok(());
    };

    if first
        .content_type
        .as_ref()
        .map(FixedString::as_str)
        .map(|s| s.starts_with("image/"))
        != Some(true)
    {
        return Ok(());
    };

    let bytes = first.download().await?;

    let Ok(image) = image::load_from_memory(&bytes) else {
        return Ok(());
    };
    let img = image.into_rgb8();

    let img_source = ImageSource::from_bytes(img.as_raw(), img.dimensions())?;
    let ocr_input = engine.prepare_input(img_source)?;

    let word_rects = engine.detect_words(&ocr_input)?;
    let line_rects = engine.find_text_lines(&ocr_input, &word_rects);
    let line_texts = engine.recognize_text(&ocr_input, &line_rects)?;

    let mut full_string = String::new();
    for line in line_texts.iter().flatten() {
        let mut string = String::new();

        for c in line.chars() {
            string.push(c.char);
        }

        let cleaned_line = string.trim();

        full_string.push_str(cleaned_line);
        full_string.push(' ');
    }

    let match_against = [
        "This can't be posted because it contains",
        "contains content blocked by this server.",
        "Only you can see this Dismiss message",
    ];

    let mut matched = None;
    for item in match_against {
        let maybe_match = fuzzy_match(&full_string, item, 3);

        if let Some(maybe_match) = maybe_match {
            matched = Some(maybe_match);
            break;
        }
    }

    if let Some(matched) = matched {
        println!(
            "\x1b[31m{} possibly evaded automod, matched on {}",
            user.name, matched
        );
    }

    /*     if let Some(matched) = matched {
        let embed = CreateEmbed::new()
            .author(CreateEmbedAuthor::from(user.clone()))
            .footer(CreateEmbedFooter::new(format!("Matched on:{matched}")))
            .image(first.url.clone());
        let _ = ChannelId::new(277163440999628800)
            .send_message(http, CreateMessage::new().embed(embed))
            .await;
    } */

    Ok(())
}

fn fuzzy_match<'a>(paragraph: &'a str, target: &'a str, max_distance: usize) -> Option<&'a str> {
    let target_len = target.len();
    let paragraph_len = paragraph.len();

    if paragraph_len < target_len {
        return None;
    }

    for i in 0..=paragraph_len - target_len {
        let substring = &paragraph[i..i + target_len];
        let distance = levenshtein(target, substring);

        if distance <= max_distance {
            return Some(substring);
        }
    }

    None
}
