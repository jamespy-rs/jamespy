use crate::{Context, Error};
use jamespy_config::SpyGuild;
use poise::serenity_prelude::{ChannelType, CreateChannel};

#[poise::command(
    rename = "init",
    prefix_command,
    category = "Spy Guild",
    hide_in_help,
    subcommands("here"),
    guild_only
)]
#[allow(clippy::no_effect_underscore_binding)]
pub async fn init(_ctx: Context<'_>) -> Result<(), Error> {
    // TODO

    Ok(())
}

#[poise::command(
    rename = "here-force",
    prefix_command,
    category = "Spy Guild",
    hide_in_help
)]
pub async fn here(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap(); // only allowed execution in guilds.

    let mut spy_conf = SpyGuild::new(guild_id);

    // TODO: unhardcode, adhere to preferences.

    // self regex, deleted media.
    let jamespy = guild_id
        .create_channel(
            ctx,
            CreateChannel::new("jamespy").kind(ChannelType::Category),
        )
        .await?
        .id;

    let self_regex = guild_id
        .create_channel(
            ctx,
            CreateChannel::new("self-regex")
                .kind(ChannelType::Text)
                .category(jamespy),
        )
        .await?
        .id;

    let attachment_hook_channel = guild_id
        .create_channel(
            ctx,
            CreateChannel::new("deleted-media")
                .kind(ChannelType::Text)
                .category(jamespy),
        )
        .await?
        .id;

    // other regexes.
    let regexes = guild_id
        .create_channel(
            ctx,
            CreateChannel::new("regexes").kind(ChannelType::Category),
        )
        .await?
        .id;

    let regex_channel = guild_id
        .create_channel(
            ctx,
            CreateChannel::new("regex-default")
                .kind(ChannelType::Text)
                .category(regexes),
        )
        .await?
        .id;

    // TODO: add proper builders in jamespy_config.

    spy_conf.self_regex = Some(jamespy_config::SelfRegex {
        enabled: true,
        channel_id: Some(self_regex),
        use_events_regex: true,
        extra_regex: None,
        context_info: true,
        mention: true,
    });

    spy_conf.attachment_hook = Some(jamespy_config::AttachmentHook {
        enabled: true,
        channel_id: Some(attachment_hook_channel),
    });

    spy_conf.patterns = Some(jamespy_config::PatternAnnounce {
        enabled: true,
        default_channel_id: Some(regex_channel),
        list: vec![],
    });

    {
        let data = ctx.data();
        let mut config = data.config.write().unwrap();

        config.spy_guild = Some(spy_conf);

        config.write_config();
    }

    // TODO: handle errors.
    ctx.say("Success!").await?;

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [init()]
}
