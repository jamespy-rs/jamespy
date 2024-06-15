use poise::serenity_prelude::{
    self as serenity, ComponentInteractionCollector, CreateActionRow, CreateEmbedFooter,
    CreateInteractionResponse,
};
use poise::{Context, CreateReply};

pub async fn presence_builder<U: Send + Sync + 'static, E>(
    ctx: Context<'_, U, E>,
    pages: Vec<Vec<(&str, u32)>>,
    total_members: usize,
    total_games: usize,
) -> Result<(), serenity::Error> {
    let ctx_id = ctx.id();
    let prev_button_id = format!("{ctx_id}prev");
    let next_button_id = format!("{ctx_id}next");

    let sctx = ctx.serenity_context();

    let mut current_page = 0;
    let footer = format!("{total_members} members are playing {total_games} games right now.");

    let msg = ctx
        .send(
            poise::CreateReply::default()
                .embed(create_presence_embed(current_page, &footer, &pages))
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
                &sctx.http,
                CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::default()
                        .embed(create_presence_embed(current_page, &footer, &pages)),
                ),
            )
            .await?;
    }
    msg.edit(
        ctx,
        CreateReply::default()
            .embed(create_presence_embed(current_page, &footer, &pages))
            .components(vec![]),
    )
    .await?;

    Ok(())
}

// This is split to make the code more pleasant
fn create_presence_embed<'a>(
    current_page: usize,
    footer_text: &str,
    pages: &[Vec<(&str, u32)>],
) -> serenity::CreateEmbed<'a> {
    serenity::CreateEmbed::default()
        .title("Top games being played right now:")
        .description(format_pages(&pages[current_page]))
        .footer(CreateEmbedFooter::new(footer_text.to_string()))
}

fn format_pages(pages: &[(&str, u32)]) -> String {
    pages
        .iter()
        .map(|(name, count)| format!("{name}: {count}"))
        .collect::<Vec<String>>()
        .join("\n")
}
