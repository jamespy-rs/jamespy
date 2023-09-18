use poise::serenity_prelude::{self as serenity, VoiceState};

use crate::Error;

pub async fn voice_state_update(
    ctx: &serenity::Context,
    old: Option<VoiceState>,
    new: VoiceState,
) -> Result<(), Error> {
    if let Some(old) = old {
        if old.channel_id != new.channel_id && new.channel_id != None {
            let mut guild_name = String::from("Unknown");
            let mut user_name = String::from("Unknown User");
            let mut old_channel = String::from("Unknown");
            let mut old_channel_id_ = String::from("Unknown");
            let mut new_channel = String::from("Unknown");
            let mut new_channel_id_ = String::from("Unknown");

            if let Some(guild_id) = old.guild_id {
                guild_name = guild_id
                    .name(ctx.clone())
                    .unwrap_or_else(|| guild_name.clone());
            }
            if let Some(member) = new.member {
                user_name = member.user.name;
            }
            if let Some(old_channel_id) = old.channel_id {
                old_channel_id_ = old_channel_id.get().to_string();
                if let Ok(channel_name) = old_channel_id.name(ctx.clone()).await {
                    old_channel = channel_name;
                } else {
                    old_channel = "Unknown".to_owned();
                }
            }
            if let Some(new_channel_id) = new.channel_id {
                new_channel_id_ = new_channel_id.get().to_string();
                if let Ok(channel_name) = new_channel_id.name(ctx.clone()).await {
                    new_channel = channel_name;
                } else {
                    new_channel = "Unknown".to_owned();
                }
            }
            println!(
                "\x1B[32m[{}] {}: {} (ID:{}) -> {} (ID:{})\x1B[0m",
                guild_name, user_name, old_channel, old_channel_id_, new_channel, new_channel_id_
            )
        } else {
            if new.channel_id == None {
                let mut guild_name = String::from("Unknown");
                let mut user_name = String::from("Unknown User");
                let mut old_channel = String::from("Unknown");
                let mut old_channel_id_ = String::from("Unknown");

                if let Some(guild_id) = old.guild_id {
                    guild_name = guild_id
                        .name(ctx.clone())
                        .unwrap_or_else(|| guild_name.clone());
                }
                if let Some(member) = new.member {
                    user_name = member.user.name;
                }
                if let Some(old_channel_id) = old.channel_id {
                    old_channel_id_ = old_channel_id.get().to_string();
                    if let Ok(channel_name) = old_channel_id.name(ctx.clone()).await {
                        old_channel = channel_name;
                    } else {
                        old_channel = "Unknown".to_owned();
                    }
                }
                println!(
                    "\x1B[32m[{}] {} left {} (ID:{})\x1B[0m",
                    guild_name, user_name, old_channel, old_channel_id_
                )
            } else {
                // mutes, unmutes, deafens, etc are here.
            }
        }
    } else {
        let mut guild_name = String::from("Unknown");
        let mut user_name = String::from("Unknown User");
        let mut new_channel = String::from("Unknown");
        let mut new_channel_id_ = String::from("Unknown");

        if let Some(guild_id) = new.guild_id {
            guild_name = guild_id
                .name(ctx.clone())
                .unwrap_or_else(|| guild_name.clone());
        }
        if let Some(member) = new.member {
            user_name = member.user.name;
        }
        if let Some(new_channel_id) = new.channel_id {
            new_channel_id_ = new_channel_id.get().to_string();
            if let Ok(channel_name) = new_channel_id.name(ctx.clone()).await {
                new_channel = channel_name;
            } else {
                new_channel = "Unknown".to_owned();
            }
        }

        println!(
            "\x1B[32m[{}] {} joined {} (ID:{})\x1B[0m",
            guild_name, user_name, new_channel, new_channel_id_
        );
    }

    Ok(())
}
