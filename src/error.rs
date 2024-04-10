use jamespy_data::structs::{Data, Error};


pub async fn handler(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        poise::FrameworkError::NotAnOwner { ctx, .. } => {
            let owner_bypass = {
                let data = ctx.data();
                let config = data.config.read();

                if let Some(check) = &config.command_checks {
                    check.owners_all.contains(&ctx.author().id)
                } else {
                    false
                }
            };

            let msg = if owner_bypass {
                "You may have access to most owner commands, but not this one <3"
            } else {
                "Only bot owners can call this command"
            };

            let _ = ctx.say(msg).await;
        }
        poise::FrameworkError::EventHandler { error, .. } => {
            println!("Error in event handler: {error}");
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {e}");
            }
        }
    }
}
