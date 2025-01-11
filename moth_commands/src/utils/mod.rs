pub mod checks;
pub mod pagination;

use std::time::Duration;

pub use checks::*;
use moth_data::structs::{Context, Error};
pub use pagination::*;
use poise::CreateReply;

pub async fn handle_cooldown(remaining_cooldown: Duration, ctx: Context<'_>) -> Result<(), Error> {
    let msg = format!(
        "You're too fast. Please wait {} seconds before retrying",
        remaining_cooldown.as_secs()
    );
    ctx.send(CreateReply::default().content(msg).ephemeral(true))
        .await?;

    Ok(())
}
