use crate::{Context, Error};
use jamespy_config::{Castle, InitStatus, MediaStashingStatus, SelfRegex, StatsStatus};

use poise::serenity_prelude::{
    ButtonStyle, ChannelType, ComponentInteractionCollector, CreateActionRow, CreateButton,
    CreateChannel, CreateInteractionResponse, CreateInteractionResponseMessage,
};

/// Init
#[poise::command(
    rename = "init",
    prefix_command,
    hide_in_help,
    owners_only,
    guild_only,
    subcommands("force")
)]
pub async fn init_conf(ctx: Context<'_>) -> Result<(), Error> {
    // think about the clone usage here.
    let conf = { ctx.data().jamespy_config.read().unwrap().clone() };

    if let Some(conf) = conf.castle_conf {
        if let Some(base) = conf.base {
            if base.setup_complete {
                ctx.say(
                    "A setup has already been completed! Will not continue unless `force` is \
                     passed.",
                )
                .await?;
            } else {
                setup(ctx).await?;
            }
        }
    }

    Ok(())
}

#[poise::command(prefix_command, hide_in_help, owners_only, guild_only)]
pub async fn force(ctx: Context<'_>) -> Result<(), Error> {
    setup(ctx).await?;

    Ok(())
}

async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    let current_member = ctx
        .guild_id()
        .unwrap()
        .member(ctx, ctx.framework().bot_id)
        .await?;
    let has_permissions = {
        let guild = ctx.guild().unwrap();

        let permissions = guild.member_permissions(&current_member);

        permissions.manage_channels()
    };
    if !has_permissions {
        ctx.say("I am missing manage channels in this server, unable to proceed!")
            .await?;
        return Ok(());
    }

    let ctx_id = ctx.id();
    let not_okay_id = format!("{ctx_id}init_not_okay");
    let okay_id = format!("{ctx_id}init_okay");
    let okay = CreateActionRow::Buttons(vec![
        CreateButton::new(&not_okay_id)
            .label("No.")
            .style(ButtonStyle::Danger),
        CreateButton::new(&okay_id).label("Yes."),
    ]);

    let mut response = false;

    // Check if this guild is okay, run through settings, modify config.
    let msg = ctx
        .send(
            poise::CreateReply::default()
                .content(
                    "This may create multiple categories and create channels **in the current \
                     guild** depending on the config, is this okay?",
                )
                .components(vec![okay]),
        )
        .await?;

    while let Some(press) = ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(15))
        .await
    {
        let owners = &ctx.framework().options.owners;
        if !owners.contains(&press.user.id) {
            continue;
        }

        if press.data.custom_id == okay_id {
            press
                .create_response(
                    ctx.serenity_context(),
                    CreateInteractionResponse::Acknowledge,
                )
                .await?;
            response = true;
            setup_settings(ctx).await?;
        } else if press.data.custom_id == not_okay_id {
            press
                .create_response(
                    ctx.serenity_context(),
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::new()
                            .content("Aborted.")
                            .components(vec![]),
                    ),
                )
                .await?;
            response = true;
        } else {
            continue;
        }
    }

    if !response {
        msg.edit(
            ctx,
            poise::CreateReply::default()
                .content("Init process timeout.")
                .components(vec![]),
        )
        .await?;
    }
    Ok(())
}

async fn setup_settings(ctx: Context<'_>) -> Result<(), Error> {
    // TODO: error handling.
    let guild_id = ctx.guild_id().unwrap();
    let category_id = guild_id
        .create_channel(
            ctx,
            CreateChannel::new("jamespy").kind(ChannelType::Category),
        )
        .await?
        .id;

    // Enable stats?
    let stats_id = guild_id
        .create_channel(
            ctx,
            CreateChannel::new("stats")
                .kind(ChannelType::Text)
                .category(category_id),
        )
        .await?
        .id;
    let stats_message = stats_id.say(ctx, "initializing...").await?.id;

    let regex_id = guild_id
        .create_channel(
            ctx,
            CreateChannel::new("regex-mentions")
                .kind(ChannelType::Text)
                .category(category_id),
        )
        .await?
        .id;
    // Enable regex allocation channel?
    // - ask if want ping.
    // accept regexes in model, encode.

    let media_stashing_id = guild_id
        .create_channel(
            ctx,
            CreateChannel::new("deleted-media")
                .kind(ChannelType::Text)
                .category(category_id),
        )
        .await?
        .id;
    // enable media stashing?
    // go through limits.

    let built_config = Some(Castle {
        base: Some(InitStatus {
            setup_complete: true,
            guild_id: Some(guild_id),
        }),
        stats: Some(StatsStatus {
            stats_enabled: true,
            stats_channel: Some(stats_id),
            stats_message: Some(stats_message),
        }),
        self_regex: Some(SelfRegex {
            regex_self: true,
            regex_channel: Some(regex_id),
            regex_self_ping: false,
            self_regexes: None,
        }),
        media: Some(MediaStashingStatus {
            media_stashing_post: true,
            media_stash_channel: Some(media_stashing_id),
            single_limit: Some(500),
            soft_limit: Some(9000),
            hard_limit: Some(10000),
        }),
    });

    let mut config = ctx.data().jamespy_config.write().unwrap();

    config.castle_conf = built_config;
    config.write_config();

    Ok(())
}

pub fn commands() -> [crate::Command; 1] {
    [init_conf()]
}
