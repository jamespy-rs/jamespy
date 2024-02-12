//! Utilities for registering application commands

use poise::serenity_prelude as serenity;

// Modified version of the builtin poise function.
pub fn create_application_commands<U, E>(
    commands: &[poise::Command<U, E>],
) -> (
    Vec<serenity::CreateCommand<'static>>,
    Vec<serenity::CreateCommand<'static>>,
) {
    fn recursively_add_context_menu_commands<U, E>(
        builder: &mut Vec<serenity::CreateCommand<'static>>,
        command: &poise::Command<U, E>,
    ) {
        if let Some(context_menu_command) = command.create_as_context_menu_command() {
            builder.push(context_menu_command);
        }
        for subcommand in &command.subcommands {
            recursively_add_context_menu_commands(builder, subcommand);
        }
    }

    let mut commands_builder = Vec::with_capacity(commands.len());
    let mut owner_commands = Vec::new();

    for command in commands {
        if let Some(slash_command) = command.create_as_slash_command() {
            if command
                .category
                .as_deref()
                .map_or(false, |desc| desc.to_lowercase().starts_with("owner"))
            {
                owner_commands.push(slash_command);
            } else {
                commands_builder.push(slash_command);
            }
        }
        recursively_add_context_menu_commands(&mut commands_builder, command);
    }
    (commands_builder, owner_commands)
}

// Modified version of the inbuilt poise function.
// Will cleanup eventually.
pub async fn register_application_commands_buttons<U: Send + Sync + 'static, E>(
    ctx: poise::Context<'_, U, E>,
    data: std::sync::Arc<crate::Data>,
) -> Result<(), serenity::Error> {
    let create_commands = create_application_commands(&ctx.framework().options().commands);
    let num_commands = create_commands.0.len();
    let num_owner = create_commands.1.len();

    let is_bot_owner = ctx.framework().options().owners.contains(&ctx.author().id);
    if !is_bot_owner {
        ctx.say("Can only be used by bot owner").await?;
        return Ok(());
    }

    // Ew.
    let (spy_guild_active, guild_id) = {
        let config = data.config.read().unwrap();

        if let Some(spy_guild) = &config.spy_guild {
            let active = spy_guild.status.enabled;

            if let Some(guild_id) = spy_guild.status.guild_id {
                (active, Some(guild_id))
            } else {
                (active, None)
            }
        } else {
            (false, None)
        }
    };

    let components = if spy_guild_active {
        serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("register.guild")
                .label("Register spy guild")
                .style(serenity::ButtonStyle::Primary)
                .emoji('📋'),
            serenity::CreateButton::new("unregister.guild")
                .label("Register spy guild")
                .style(serenity::ButtonStyle::Danger)
                .emoji('🗑'),
            serenity::CreateButton::new("register.global")
                .label("Register globally")
                .style(serenity::ButtonStyle::Primary)
                .emoji('📋'),
            serenity::CreateButton::new("unregister.global")
                .label("Unregister globally")
                .style(serenity::ButtonStyle::Danger)
                .emoji('🗑'),
        ])
    } else {
        serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("register.global")
                .label("Register globally")
                .style(serenity::ButtonStyle::Primary)
                .emoji('📋'),
            serenity::CreateButton::new("unregister.global")
                .label("Unregister globally")
                .style(serenity::ButtonStyle::Danger)
                .emoji('🗑'),
        ])
    };

    let builder = poise::CreateReply::default()
        .content("Choose what to do with the commands:")
        .components(vec![components]);

    let reply = ctx.send(builder).await?;

    let interaction = reply
        .message()
        .await?
        .await_component_interaction(ctx.serenity_context().shard.clone())
        .author_id(ctx.author().id)
        .await;

    reply
        .edit(
            ctx,
            poise::CreateReply::default()
                .components(vec![])
                .content("Processing... Please wait."),
        )
        .await?; // remove buttons after button press and edit message
    let pressed_button_id = match &interaction {
        Some(m) => &m.data.custom_id,
        None => {
            ctx.say(":warning: You didn't interact in time - please run the command again.")
                .await?;
            return Ok(());
        }
    };

    // I'm aware this checks for the spy guild stuff,
    // It'll fail anyway if somehow this is sent, so it is fine.
    let (register, global) = match &**pressed_button_id {
        "register.global" => (true, true),
        "unregister.global" => (false, true),
        "register.guild" => (true, false),
        "unregister.guild" => (false, false),
        other => {
            tracing::warn!("unknown register button ID: {:?}", other);
            return Ok(());
        }
    };

    let start_time = std::time::Instant::now();

    if global {
        if register {
            ctx.say(format!(
                ":gear: Registering {num_commands} global commands...",
            ))
            .await?;
            serenity::Command::set_global_commands(ctx.http(), &create_commands.0).await?;
        } else {
            ctx.say(":gear: Unregistering global commands...").await?;
            serenity::Command::set_global_commands(ctx.http(), &[]).await?;
        }
    } else {
        let guild_id = match guild_id {
            Some(x) => x,
            None => {
                ctx.say(":x: Cannot register spy commands if spy guild isn't set!")
                    .await?;
                return Ok(());
            }
        };
        if register {
            ctx.say(format!(
                ":gear: Registering {num_owner} spy guild commands...",
            ))
            .await?;
            guild_id
                .set_commands(ctx.http(), &create_commands.1)
                .await?;
        } else {
            ctx.say(":gear: Unregistering spy guild commands...")
                .await?;
            guild_id.set_commands(ctx.http(), &[]).await?;
        }
    }

    let time_taken = start_time.elapsed();
    ctx.say(format!(
        ":white_check_mark: Done! Took {}ms",
        time_taken.as_millis()
    ))
    .await?;

    Ok(())
}
