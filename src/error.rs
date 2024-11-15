use jamespy_commands::utils::handle_cooldown;
use jamespy_data::structs::{Data, Error, InvocationData};
use poise::serenity_prelude as serenity;

pub async fn handler(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            let Some(invocation_data) = ctx.invocation_data::<InvocationData>().await else {
                println!("Error in command `{}`: {:?}", ctx.command().name, error);
                return;
            };

            let Some(duration) = invocation_data.cooldown_remaining else {
                println!("Error in command `{}`: {:?}", ctx.command().name, error);
                return;
            };

            let _ = handle_cooldown(duration, ctx).await;
        }
        poise::FrameworkError::NotAnOwner { ctx, .. } => {
            let owner_bypass = {
                let data = ctx.data();

                let checks = data.database.inner_overwrites();
                checks.owners_all.contains(&ctx.author().id)
            };

            let msg = if owner_bypass {
                "You may have access to most owner commands, but not this one <3"
            } else {
                "Only bot owners can call this command"
            };

            let _ = ctx.say(msg).await;
        }
        poise::FrameworkError::CommandCheckFailed { error, ctx, .. } => {
            let mut embed = serenity::CreateEmbed::new()
                .title("You do not have permission to access this command.")
                .colour(serenity::Colour::RED);

            if let Some(err) = error {
                embed = embed.description(err.to_string());
            };

            let msg = poise::CreateReply::new().embed(embed);
            let _ = ctx.send(msg).await;
        }
        poise::FrameworkError::EventHandler { error, .. } => {
            println!("Error in event handler: {error}");
        }
        poise::FrameworkError::CooldownHit {
            remaining_cooldown,
            ctx,
            ..
        } => {
            // all cooldowns (framework and manual) should go to the same route.
            let _ = handle_cooldown(remaining_cooldown, ctx).await;
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {e}");
            }
        }
    }
}
