use crate::{Context, Error};
use rand::rngs::OsRng;
use rand::seq::SliceRandom;


/// lob
#[poise::command(slash_command, prefix_command, category = "Utility", user_cooldown = "5")]
pub async fn lob(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let loblist = std::fs::read_to_string("loblist.txt")?;
    let options: Vec<&str> = loblist.lines().collect();

    // Create an instance of OsRng
    let mut rng = OsRng::default();

    if let Some(chosen_option) = options.choose(&mut rng) {
        ctx.say(chosen_option.to_string()).await?;
    }

    Ok(())
}

// TODO: Add roll command!
