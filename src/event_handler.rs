use crate::{config::CONFIG, event_handlers, Data, Error};
use poise::serenity_prelude as serenity;

pub async fn event_handler(
    event: serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Message { ctx, new_message } => {
            event_handlers::messages::message(&ctx, new_message, data).await?;
        }
        serenity::FullEvent::MessageUpdate {
            ctx,
            old_if_available,
            new,
            event,
        } => {
            event_handlers::messages::message_edit(&ctx, old_if_available, new, event, data)
                .await?;
        }
        serenity::FullEvent::MessageDelete {
            ctx,
            channel_id,
            deleted_message_id,
            guild_id,
        } => {
            event_handlers::messages::message_delete(
                &ctx,
                channel_id,
                deleted_message_id,
                guild_id,
                data,
            )
            .await?;
        }
        // need serenity:FullEvent::MessageDeleteBulk
        serenity::FullEvent::GuildCreate { ctx, guild, is_new } => {
            event_handlers::guilds::guild_create(&ctx, guild, is_new).await?;
        }

        serenity::FullEvent::ReactionAdd { ctx, add_reaction } => {
            event_handlers::reactions::reaction_add(&ctx, add_reaction, data).await?;
        }
        serenity::FullEvent::ReactionRemove {
            ctx,
            removed_reaction,
        } => {
            event_handlers::reactions::reaction_remove(&ctx, removed_reaction, data).await?;
        }
        serenity::FullEvent::ReactionRemoveAll {
            ctx: _,
            channel_id: _,
            removed_from_message_id: _,
        } => {
            // Need to do the funny here.
            // Will leave it untouched until I have a better codebase.
        }
        serenity::FullEvent::ChannelCreate { ctx, channel } => {
            event_handlers::channels::channel_create(&ctx, channel).await?;
        }
        serenity::FullEvent::ChannelUpdate { ctx, old, new } => {
            event_handlers::channels::channel_update(&ctx, old, new).await?;
        }
        serenity::FullEvent::ChannelDelete {
            ctx,
            channel,
            messages: _,
        } => {
            event_handlers::channels::channel_delete(&ctx, channel).await?;
        }
        serenity::FullEvent::ThreadCreate { ctx, thread } => {
            event_handlers::channels::thread_create(&ctx, thread).await?;
        }
        serenity::FullEvent::ThreadUpdate { ctx, old, new } => {
            event_handlers::channels::thread_update(&ctx, old, new).await?;
        }
        serenity::FullEvent::ThreadDelete {
            ctx,
            thread,
            full_thread_data,
        } => {
            event_handlers::channels::thread_delete(&ctx, thread, full_thread_data).await?;
        }
        serenity::FullEvent::VoiceStateUpdate { ctx, old, new } => {
            event_handlers::voice::voice_state_update(&ctx, old, new).await?;
        }
        serenity::FullEvent::VoiceChannelStatusUpdate {
            ctx,
            old,
            status,
            id,
            guild_id,
        } => {
            let guilds_opt = {
                let config = CONFIG.read().unwrap();
                config.vcstatus.guilds.clone()
            };

            if let Some(guilds) = guilds_opt {
                if guilds.contains(&guild_id) {
                    event_handlers::channels::voice_channel_status_update(
                        &ctx, old, status, id, guild_id,
                    )
                    .await?;
                }
            }
        }
        serenity::FullEvent::Ready {
            ctx,
            data_about_bot: _,
        } => {
            event_handlers::misc::ready(&ctx, data).await?;
        }
        serenity::FullEvent::GuildMemberAddition { ctx, new_member } => {
            event_handlers::guilds::guild_member_addition(&ctx, new_member).await?;
        }
        serenity::FullEvent::GuildMemberRemoval {
            ctx,
            guild_id,
            user,
            member_data_if_available: _,
        } => {
            event_handlers::guilds::guild_member_removal(&ctx, guild_id, user).await?;
        }
        serenity::FullEvent::GuildMemberUpdate {
            ctx,
            old_if_available,
            new,
            event,
        } => {
            event_handlers::users::guild_member_update(&ctx, old_if_available, new, event).await?;
        }
        _ => (),
    }

    Ok(())
}
