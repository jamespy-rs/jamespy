use jamespy_data::structs::{Data, Error};
use poise::serenity_prelude::{self as serenity, FullEvent};

pub mod attachments;
pub mod helper;
pub mod tasks;

pub mod handlers;
use handlers::*;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::Message { new_message } => {
            messages::message(ctx, new_message, data).await?;
        }
        FullEvent::MessageUpdate {
            old_if_available,
            new,
            event,
        } => {
            messages::message_edit(ctx, old_if_available, new, event, data).await?;
        }
        FullEvent::MessageDelete {
            channel_id,
            deleted_message_id,
            guild_id,
        } => {
            messages::message_delete(ctx, channel_id, deleted_message_id, guild_id, data).await?;
        }
        FullEvent::ReactionAdd { add_reaction } => {
            reactions::reaction_add(ctx, add_reaction, data).await?;
        }
        FullEvent::ReactionRemove { removed_reaction } => {
            reactions::reaction_remove(ctx, removed_reaction, data).await?;
        }
        FullEvent::GuildCreate { guild, is_new } => {
            guilds::guild_create(ctx, guild, is_new).await?;
        }
        FullEvent::GuildMemberAddition { new_member } => {
            guilds::guild_member_addition(ctx, new_member, data).await?;
        }
        FullEvent::GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available: _,
        } => {
            guilds::guild_member_removal(ctx, guild_id, user, data).await?;
        }
        FullEvent::GuildAuditLogEntryCreate { entry, guild_id } => {
            guilds::guild_audit_log_entry_create(ctx, entry, guild_id).await?;
        }
        FullEvent::ChannelCreate { channel } => {
            channels::channel_create(ctx, channel).await?;
        }
        FullEvent::ChannelDelete {
            channel,
            messages: _,
        } => {
            channels::channel_delete(ctx, channel).await?;
        }
        FullEvent::ChannelUpdate { old, new } => {
            channels::channel_update(ctx, old, new).await?;
        }
        FullEvent::ThreadCreate { thread } => {
            channels::thread_create(ctx, thread).await?;
        }
        FullEvent::ThreadDelete {
            thread,
            full_thread_data,
        } => {
            channels::thread_delete(ctx, thread, full_thread_data).await?;
        }
        FullEvent::ThreadUpdate { old, new } => {
            channels::thread_update(ctx, old, new).await?;
        }
        FullEvent::VoiceChannelStatusUpdate {
            old,
            status,
            id,
            guild_id,
        } => {
            let guilds = { data.config.read().unwrap().vcstatus.guilds.clone() };
            if let Some(guilds) = guilds {
                if guilds.contains(guild_id) {
                    channels::voice_channel_status_update(ctx, old, status, id, guild_id, data)
                        .await?;
                }
            }
        }
        FullEvent::VoiceStateUpdate { old, new } => {
            voice::voice_state_update(ctx, old, new).await?;
        }
        FullEvent::GuildMemberUpdate {
            old_if_available,
            new,
            event,
        } => {
            users::guild_member_update(ctx, old_if_available, new, event, data).await?;
        }
        FullEvent::Ready { data_about_bot: _ } => {
            misc::ready(ctx, data).await?;
        }
        FullEvent::CacheReady { guilds } => {
            misc::cache_ready(ctx, guilds, data).await?;
        }

        _ => {}
    }
    Ok(())
}
