use poise::serenity_prelude::{self as serenity, CreateEmbedFooter};
use poise::Context;

// The function that constructs the paginated messages for the guild_message_cache command.
pub async fn guild_message_cache_builder<U, E>(
    ctx: Context<'_, U, E>,
    pages: &[&str],
    total_messages_cached: usize,
) -> Result<(), serenity::Error> {
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    let mut current_page = 0;
    let footer_text = format!(
        "Total messages cached in the guild: {}",
        total_messages_cached
    );

    ctx.send(
        poise::CreateReply::default()
            .embed(
                serenity::CreateEmbed::default()
                    .title("Channels with most cached messages:")
                    .description(pages[current_page])
                    .footer(CreateEmbedFooter::new(footer_text.clone())),
            )
            .components(vec![serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new(&prev_button_id).emoji('◀'),
                serenity::CreateButton::new(&next_button_id).emoji('▶'),
            ])]),
    )
    .await?;

    while let Some(press) = serenity::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(60))
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
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::default().embed(
                        serenity::CreateEmbed::default()
                            .title("Channels with most cached messages:")
                            .description(pages[current_page])
                            .footer(CreateEmbedFooter::new(footer_text.clone())),
                    ),
                ),
            )
            .await?;
    }

    Ok(())
}
