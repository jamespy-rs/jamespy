use crate::{Context, Error};
use bb8_redis::redis::AsyncCommands;
use poise::serenity_prelude as serenity;
use serenity::{
    all::{GuildMemberFlags, Member},
    model::Colour,
};

#[poise::command(
    rename = "guild-flags",
    aliases("guild_flags", "guildflags"),
    slash_command,
    prefix_command,
    guild_only,
    category = "Utility",
    required_permissions = "MANAGE_MESSAGES",
    default_member_permissions = "MANAGE_MESSAGES",
    track_edits,
    user_cooldown = 4
)]
pub async fn guild_flags(
    ctx: Context<'_>,
    #[description = "The member whose flags are to be checked."] member: Member,
) -> Result<(), Error> {
    let member_flags = member.flags;

    let mut outcome = String::new();

    if member_flags.contains(GuildMemberFlags::DID_REJOIN) {
        outcome.push_str("Member has left and rejoined the guild.\n");
    }

    if member_flags.contains(GuildMemberFlags::COMPLETED_ONBOARDING) {
        outcome.push_str("Member has completed onboarding.\n");
    }

    if member_flags.contains(GuildMemberFlags::BYPASSES_VERIFICATION) {
        outcome.push_str("Member is exempt from guild verification requirements.\n");
    }

    if member_flags.contains(GuildMemberFlags::STARTED_ONBOARDING) {
        outcome.push_str("Member has started onboarding.\n");
    }

    if !outcome.is_empty() {
        ctx.say(outcome).await?;
    } else {
        ctx.say("Member has no special flags.").await?;
    }

    Ok(())
}

#[poise::command(
    rename = "last-reactions",
    aliases("lastreactions", "last_reactions"),
    slash_command,
    prefix_command,
    category = "Utility",
    required_permissions = "MANAGE_MESSAGES",
    default_member_permissions = "MANAGE_MESSAGES",
    guild_only,
    user_cooldown = 3
)]
pub async fn last_reactions(ctx: Context<'_>) -> Result<(), Error> {
    let redis_pool = &ctx.data().redis;
    let mut redis_conn = redis_pool.get().await?;
    let reaction_key = format!("reactions:{}", ctx.guild_id().unwrap());

    let reactions: Vec<String> = redis_conn.lrange(reaction_key, 0, 24).await?;

    let formatted: String = reactions
        .iter()
        .map(|reaction| {
            let components: Vec<&str> = reaction
                .trim_matches(|c| c == '[' || c == ']' || c == '"')
                .split(',')
                .map(|component| component.trim_matches(|c| c == '"'))
                .collect();

            if components.len() == 4 {
                let (emoji, user_id, reaction_id, state) = (
                    components[0],
                    components[1].parse::<u64>().unwrap(),
                    components[2].parse::<u64>().unwrap(),
                    components[3].parse::<u32>().unwrap(),
                );

                let username = ctx
                    .cache()
                    .user(user_id)
                    .map(|user| user.name.clone())
                    .unwrap_or_else(|| "Unknown User".to_string());

                format!(
                    "**{}** {} {} Message ID: {}",
                    username,
                    if state == 1 { "added" } else { "removed" },
                    emoji,
                    reaction_id
                )
            } else {
                "Invalid reaction format".to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::default()
                .title("Last reaction events")
                .description(format!("{}", formatted))
                .color(Colour::from_rgb(0, 255, 0)),
        ),
    )
    .await?;

    Ok(())
}
