use std::sync::Arc;

use crate::{Data, Error};
use ::serenity::all::{CreateInteractionResponseMessage, UserId};
use moth_data::database::StarboardStatus;
use poise::serenity_prelude as serenity;

use super::starboard::starboard_message;

pub const STARBOARD_CHANNEL: serenity::ChannelId = serenity::ChannelId::new(1324437745854316564);

pub async fn handle_component(
    ctx: &serenity::Context,
    data: Arc<Data>,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    if !std::env::var("STARBOARD_ACTIVE").map(|e| e.parse::<bool>())?? {
        return Ok(());
    };

    if !matches!(
        interaction.data.custom_id.as_str(),
        "starboard_accept" | "starboard_deny"
    ) {
        return Ok(());
    }

    if !allowed_user(interaction.user.id) {
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("You aren't Phil, Ruben or Moxy")
                        .ephemeral(true),
                ),
            )
            .await?;
    }

    // on the race condition case i should probably send a response?
    if interaction.data.custom_id == "starboard_accept" {
        // create new message
        // run approve function
        if !data.database.handle_starboard(interaction.message.id) {
            let _ = accept(ctx, &data, interaction).await;
            data.database.stop_handle_starboard(&interaction.message.id);
        }
    } else if interaction.data.custom_id == "starboard_deny" {
        if !data.database.handle_starboard(interaction.message.id) {
            let _ = deny(ctx, &data, interaction).await;
            data.database.stop_handle_starboard(&interaction.message.id);
        }
    } else {
        return Ok(());
    }

    Ok(())
}

async fn accept(
    ctx: &serenity::Context,
    data: &Arc<Data>,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    let mut starboard = data
        .database
        .get_starboard_msg_by_starboard_id(interaction.message.id)
        .await?;

    starboard.starboard_status = StarboardStatus::Accepted;

    let builder = CreateInteractionResponseMessage::new()
        .components(&[])
        .content(format!("Approved by <@{}>", interaction.user.id));

    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(builder),
        )
        .await?;

    let new_msg = STARBOARD_CHANNEL
        .send_message(&ctx.http, starboard_message(ctx, &starboard))
        .await?;

    data.database
        .approve_starboard(interaction.message.id, new_msg.id, new_msg.channel_id)
        .await?;

    Ok(())
}

async fn deny(
    ctx: &serenity::Context,
    data: &Arc<Data>,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    let builder = CreateInteractionResponseMessage::new()
        .components(&[])
        .content(format!("Denied by <@{}>", interaction.user.id));

    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(builder),
        )
        .await?;

    data.database.deny_starboard(interaction.message.id).await?;

    Ok(())
}

fn allowed_user(user_id: UserId) -> bool {
    // Phil, Ruben, me
    let a = [
        UserId::new(101090238067113984),
        UserId::new(291089948709486593),
        UserId::new(158567567487795200),
    ];

    a.contains(&user_id)
}
