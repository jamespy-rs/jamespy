use crate::{Context, Error};
use poise::serenity_prelude::{self as serenity, Colour, CreateEmbedAuthor};
use rand::rngs::OsRng;
use rand::RngCore;

// A choose command, can't use this slash command until i fix the arguments
#[poise::command(prefix_command, category = "Utility", user_cooldown = "5")]
pub async fn choose(
    ctx: Context<'_>,
    #[description = "List of choices"] choices: Vec<String>,
) -> Result<(), Error> {
    if choices.is_empty() {
        ctx.say("Please provide some choices to pick from.").await?;
        return Ok(());
    }
    let author = ctx.author();
    let mut rng = OsRng::default();
    let random_index = rng.next_u32() as usize % choices.len();
    let chosen_option = &choices[random_index];

    let image_url = author.avatar_url().unwrap_or_default();

    ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .author(CreateEmbedAuthor::new(&author.name).icon_url(image_url))
                .description(format!("{}", chosen_option))
                .color(Colour::from_rgb(0, 255, 0)),
        ),
    )
    .await?;

    Ok(())
}

// TODO: Add roll command!
