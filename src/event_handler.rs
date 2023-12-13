use crate::{config::CONFIG, event_handlers, Data, Error};
use poise::serenity_prelude as serenity;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Message { new_message } => {
            event_handlers::messages::message(ctx, new_message, data).await?;
        }
        serenity::FullEvent::MessageUpdate {
            old_if_available,
            new,
            event,
        } => {
            event_handlers::messages::message_edit(ctx, old_if_available, new, event, data).await?;
        }
        serenity::FullEvent::MessageDelete {
            channel_id,
            deleted_message_id,
            guild_id,
        } => {
            event_handlers::messages::message_delete(
                ctx,
                channel_id,
                deleted_message_id,
                guild_id,
                data,
            )
            .await?;
        }
        serenity::FullEvent::GuildCreate { guild, is_new } => {
            event_handlers::guilds::guild_create(ctx, guild, is_new).await?;
        }

        serenity::FullEvent::ReactionAdd { add_reaction } => {
            event_handlers::reactions::reaction_add(ctx, add_reaction, data).await?;
        }
        serenity::FullEvent::ReactionRemove { removed_reaction } => {
            event_handlers::reactions::reaction_remove(ctx, removed_reaction, data).await?;
        }
        serenity::FullEvent::ChannelCreate { channel } => {
            event_handlers::channels::channel_create(ctx, channel).await?;
        }
        serenity::FullEvent::ChannelUpdate { old, new } => {
            event_handlers::channels::channel_update(ctx, old, new).await?;
        }
        serenity::FullEvent::ChannelDelete {
            channel,
            messages: _,
        } => {
            event_handlers::channels::channel_delete(ctx, channel).await?;
        }
        serenity::FullEvent::ThreadCreate { thread } => {
            event_handlers::channels::thread_create(ctx, thread).await?;
        }
        serenity::FullEvent::ThreadUpdate { old, new } => {
            event_handlers::channels::thread_update(ctx, old, new).await?;
        }
        serenity::FullEvent::ThreadDelete {
            thread,
            full_thread_data,
        } => {
            event_handlers::channels::thread_delete(ctx, thread, full_thread_data).await?;
        }
        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            event_handlers::voice::voice_state_update(ctx, old, new).await?;
        }
        serenity::FullEvent::VoiceChannelStatusUpdate {
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
                        ctx, old, status, id, guild_id,
                    )
                    .await?;
                }
            }
        }
        serenity::FullEvent::Ready { data_about_bot: _ } => {
            event_handlers::misc::ready(ctx, data).await?;
        }
        serenity::FullEvent::CacheReady { guilds } => {
            event_handlers::misc::cache_ready(ctx, guilds, data).await?;
        }
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            event_handlers::guilds::guild_member_addition(ctx, new_member, data).await?;
        }
        serenity::FullEvent::GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available: _,
        } => {
            event_handlers::guilds::guild_member_removal(ctx, guild_id, user, data).await?;
        }
        serenity::FullEvent::GuildMemberUpdate {
            old_if_available,
            new,
            event,
        } => {
            event_handlers::users::guild_member_update(ctx, old_if_available, new, event).await?;
        }
        serenity::FullEvent::GuildAuditLogEntryCreate { entry, guild_id } => {
            event_handlers::guilds::guild_audit_log_entry_create(ctx, entry, guild_id).await?;
        }
        _ => (),
    }

    Ok(())
}
