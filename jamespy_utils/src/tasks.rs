use crate::Data;

use poise::serenity_prelude::{self as serenity, UserId};

pub async fn check_space(ctx: &serenity::Context, data: &Data) -> Result<(), crate::Error> {
    let castle_conf = {
        let data = data.jamespy_config.read().unwrap();

        data.castle_conf.clone()
    };

    if let Some(castle) = &castle_conf {
        if castle.base.as_ref().unwrap().setup_complete
            && castle.media.as_ref().unwrap().media_stashing_post
        {
            if let Some(media) = &castle.media {
                if media.soft_limit.is_some() || media.hard_limit.is_some() {
                    let folder_size_result = fs_extra::dir::get_size("config/attachments");

                    match folder_size_result {
                        Ok(folder_size) => {
                            if let Some(soft_limit) = media.soft_limit {
                                if folder_size > soft_limit * 1_000_000 {
                                    let user_id = UserId::from(158567567487795200);
                                    let user = user_id.to_user(ctx.clone()).await?;
                                    user.dm(
                                        &ctx,
                                        serenity::CreateMessage::default().content(format!(
                                            "Soft limit has been reached!: {}MB/{}MB",
                                            folder_size / 1_000_000,
                                            soft_limit
                                        )),
                                    )
                                    .await?;
                                }
                            }
                            if let Some(hard_limit) = media.hard_limit {
                                if folder_size > hard_limit * 1_000_000 {
                                    let user_id = UserId::from(58567567487795200);
                                    let user = user_id.to_user(ctx.clone()).await?;
                                    user.dm(
                                        &ctx,
                                        serenity::CreateMessage::default().content(format!(
                                            "Hard limit has been reached, Disabling!: {}MB/{}MB",
                                            folder_size / 1_000_000,
                                            hard_limit
                                        )),
                                    )
                                    .await?;

                                    data.jamespy_config
                                        .write()
                                        .unwrap()
                                        .castle_conf
                                        .as_mut()
                                        .unwrap()
                                        .media
                                        .as_mut()
                                        .unwrap()
                                        .media_stashing_post = false;
                                    data.jamespy_config.read().unwrap().write_config();
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("Failure to check folder size: {err}");
                        }
                    }
                }
            }
        };
    };

    Ok(())
}

pub async fn update_stats(ctx: &serenity::Context, data: &Data) -> Result<(), crate::Error> {
    let castle_conf = {
        let data = data.jamespy_config.read().unwrap();

        data.castle_conf.clone()
    };

    if let Some(castle) = &castle_conf {
        if let Some(castle_stats) = &castle.stats {
            if castle_stats.stats_enabled {
                if let Some(msg) = castle_stats.stats_message {
                    let msg = ctx
                        .http
                        .get_message(castle_stats.stats_channel.unwrap(), msg)
                        .await;
                    match msg {
                        Ok(mut msg) => {
                            let embed = serenity::CreateEmbed::default().title("Stats");
                            msg.edit(
                                ctx,
                                serenity::EditMessage::default().content("").embed(embed),
                            )
                            .await?;
                        }
                        Err(err) => {
                            let user_id = UserId::from(158567567487795200);
                            let user = user_id.to_user(ctx.clone()).await?;
                            user.dm(
                                &ctx,
                                serenity::CreateMessage::default()
                                    .content(format!("Failure to get stats message: {err}")),
                            )
                            .await?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
