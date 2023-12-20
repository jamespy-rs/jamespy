use crate::{Context, Error};
use poise::serenity_prelude::{self as serenity, Colour, CreateEmbedAuthor};
use rand::rngs::OsRng;
use rand::RngCore;

#[allow(clippy::too_many_arguments)]
/// Make me decide for you!
#[poise::command(
    slash_command,
    prefix_command,
    category = "Utility",
    user_cooldown = "3",
    track_edits
)]
pub async fn choose(
    ctx: Context<'_>,
    #[description = "Choice 1"] choice1: String,
    #[description = "Choice 2"] choice2: String,
    #[description = "Choice 3"] choice3: Option<String>,
    #[description = "Choice 4"] choice4: Option<String>,
    #[description = "Choice 5"] choice5: Option<String>,
    #[description = "Choice 6"] choice6: Option<String>,
    #[description = "Choice 7"] choice7: Option<String>,
    #[description = "Choice 8"] choice8: Option<String>,
    #[description = "Choice 9"] choice9: Option<String>,
    #[description = "Choice 10"] choice10: Option<String>,
) -> Result<(), Error> {
    let mut choices = vec![choice1, choice2];
    let optional_choices = vec![
        choice3, choice4, choice5, choice6, choice7, choice8, choice9, choice10,
    ];

    choices.extend(optional_choices.into_iter().flatten());

    let author = ctx.author();
    let mut rng = OsRng;
    let random_index = rng.next_u32() as usize % choices.len();
    let chosen_option = &choices[random_index];

    let image_url = author.avatar_url().unwrap_or_default();

    ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .author(CreateEmbedAuthor::new(&author.name).icon_url(image_url))
                .description(chosen_option.to_string())
                .color(Colour::from_rgb(0, 255, 0)),
        ),
    )
    .await?;

    Ok(())
}

pub fn commands() -> [crate::Command; 1] {
    [choose()]
}
