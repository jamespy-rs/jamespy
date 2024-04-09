use crate::{Data, Error};

use poise::serenity_prelude::{self as serenity, UserId};

pub async fn check_space(ctx: &serenity::Context, data: &Data) -> Result<(), Error> {
    let attachments = {
        let data = data.config.read();

        match data.attachment_store {
            Some(store) if store.enabled => store,
            _ => return Ok(()),
        }
    };

    let folder_size = if attachments.soft_limit.is_some() || attachments.hard_limit.is_some() {
        if let Ok(folder_size_result) = fs_extra::dir::get_size("config/attachments") {
            folder_size_result
        } else {
            return Ok(());
        }
    } else {
        return Ok(());
    };

    if let Some(hard_limit) = attachments.hard_limit {
        if folder_size > hard_limit * 1_000_000 {
            hard_limit_hit(ctx, data, folder_size, hard_limit).await?;
            return Ok(());
        }
    }

    if let Some(soft_limit) = attachments.soft_limit {
        if folder_size > soft_limit * 1_000_000 {
            soft_limit_hit(ctx, folder_size, soft_limit).await?;
        }
        return Ok(());
    }

    Ok(())
}

pub async fn soft_limit_hit(
    ctx: &serenity::Context,
    folder_size: u64,
    limit: u64,
) -> Result<(), Error> {
    // TODO: make configurable or send to fw owner.
    let user_id = UserId::from(158567567487795200);
    let user = user_id.to_user(ctx.clone()).await?;
    user.dm(
        &ctx.http,
        serenity::CreateMessage::default().content(format!(
            "Soft limit has been reached!: {}MB/{}MB",
            folder_size / 1_000_000,
            limit
        )),
    )
    .await?;
    Ok(())
}

pub async fn hard_limit_hit(
    ctx: &serenity::Context,
    data: &Data,
    folder_size: u64,
    limit: u64,
) -> Result<(), Error> {
    // Ditto.
    let user_id = UserId::from(58567567487795200);
    let user = user_id.to_user(ctx.clone()).await?;
    user.dm(
        &ctx.http,
        serenity::CreateMessage::default().content(format!(
            "Hard limit has been reached, Disabling!: {}MB/{}MB",
            folder_size / 1_000_000,
            limit
        )),
    )
    .await?;

    let mut config = data.config.write();

    if let Some(attachments) = &mut config.attachment_store {
        attachments.enabled = false;
        config.write_config();
    }

    Ok(())
}
