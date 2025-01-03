use std::sync::Arc;

use crate::{Data, Error};
use ::serenity::all::{CreateInteractionResponseMessage, UserId};
use jamespy_data::database::StarboardStatus;
use poise::serenity_prelude as serenity;

use super::starboard::starboard_message;

pub const STARBOARD_CHANNEL: serenity::ChannelId = serenity::ChannelId::new(1324437745854316564);

pub async fn handle(
    ctx: &serenity::Context,
    data: Arc<Data>,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    if !std::env::var("STARBOARD_ACTIVE").map(|e| e.parse::<bool>())?? {
        return Ok(());
    };

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

    if interaction.data.custom_id == "starboard_accept" {
        // create new message
        // run approve function
        let mut starboard = data
            .database
            .get_starboard_msg_by_starboard_id(interaction.message.id)
            .await?;

        starboard.starboard_status = StarboardStatus::Accepted;

        let builder = CreateInteractionResponseMessage::new()
            .components(&[])
            .content("Approved!");

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
    } else if interaction.data.custom_id == "starboard_deny" {
        let builder = CreateInteractionResponseMessage::new()
            .components(&[])
            .content("Denied!");

        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::UpdateMessage(builder),
            )
            .await?;

        data.database.deny_starboard(interaction.message.id).await?;
    } else {
        return Ok(());
    }

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
