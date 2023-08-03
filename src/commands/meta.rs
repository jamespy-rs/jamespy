use crate::{Context, Error};

// ... (other imports)

#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("**Bailing out, you are on your own. Good luck.**").await?;
    ctx.framework().shard_manager().lock().await.shutdown_all().await;
    Ok(())
}

// Post a link to my source code!
#[poise::command(slash_command, prefix_command, category = "Meta")]
pub async fn source(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("<https://github.com/jamesbt365/jamespy-rs>\n<https://github.com/jamesbt365/jamespy/tree/frontend>").await?;
    Ok(())
}
