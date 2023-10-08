use poise::serenity_prelude::{self as serenity, GuildMemberUpdateEvent, Member};

use crate::Error;

pub async fn guild_member_update(
    ctx: &serenity::Context,
    old_if_available: Option<Member>,
    new: Option<Member>,
    event: GuildMemberUpdateEvent,
) -> Result<(), Error> {
    if let Some(old_member) = old_if_available {
        if let Some(new_member) = new {
            let guild_id = event.guild_id;
            let guild_name = if guild_id == 1 {
                "None".to_owned()
            } else {
                match guild_id.name(ctx.clone()) {
                    Some(name) => name,
                    None => "Unknown".to_owned(),
                }
            };

            let old_nickname = old_member.nick.as_deref().unwrap_or("None");
            let new_nickname = new_member.nick.as_deref().unwrap_or("None");

            if old_nickname != new_nickname {
                println!(
                    "\x1B[92m[{}] Nickname change: {}: {} -> {} (ID:{})\x1B[0m",
                    guild_name,
                    new_member.user.name,
                    old_nickname,
                    new_nickname,
                    new_member.user.id
                );
            };

            if old_member.user.avatar != new_member.user.avatar {
                super::glow::avatar_change(ctx, &old_member, &new_member).await?;
            }
            if old_member.avatar != new_member.avatar {
                super::glow::guild_avatar_change(ctx, &old_member, &new_member).await?;
            }
            if old_member.user.banner != new_member.user.banner {
                super::glow::banner_change(ctx, &old_member, &new_member).await?;
            }

            if old_member.user.name != new_member.user.name {
                println!(
                    "\x1B[92mUsername change: {} -> {} (ID:{})\x1B[0m",
                    old_member.user.name, new_member.user.name, new_member.user.id
                );
            }
            if old_member.user.global_name != new_member.user.global_name {
                println!(
                    "\x1B[92mDisplay name change: {}: {} -> {} (ID:{})\x1B[0m",
                    old_member.user.name,
                    old_member.user.global_name.unwrap_or("None".to_owned()),
                    new_member.user.global_name.unwrap_or("None".to_owned()),
                    new_member.user.id
                )
            }
        }
    }

    Ok(())
}
