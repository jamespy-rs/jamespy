use std::{fs::File, io::Read};

use toml::Value;

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

// About jamespy!
#[poise::command(slash_command, prefix_command, category = "Meta")]
pub async fn about(ctx: Context<'_>) -> Result<(), Error> {
    let version = {
        let mut file = File::open("Cargo.toml")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let data = contents.parse::<Value>().unwrap();
        let version = data["package"]["version"].as_str().unwrap();
        version.to_string()
    };
    ctx.say(version).await?;
    Ok(())
}


/// Show general help or help to a specific command
#[poise::command(prefix_command, track_edits, slash_command, category = "Miscellaneous")]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {

    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            ephemeral: true,
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}
