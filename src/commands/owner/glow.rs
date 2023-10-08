use poise::serenity_prelude::{self as serenity, ChannelId};

use crate::event_handlers::glow::CONFIG;
use crate::{Context, Error};

#[poise::command(prefix_command, category = "Glow", hide_in_help, owners_only)]
pub async fn glow(ctx: Context<'_>) -> Result<(), Error> {
    let current_value;
    {
        let mut write_lock = CONFIG.write().unwrap();
        write_lock.action = !write_lock.action;
        current_value = write_lock.action;
    }

    let content = format!("Glow status is now set to {}!", current_value);
    ctx.say(content).await?;

    Ok(())
}
