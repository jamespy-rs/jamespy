use poise::serenity_prelude as serenity;
use ::serenity::all::{Member, GuildMemberFlags};

use crate::{Context, Error};

#[poise::command(rename = "guild-flags", aliases("guild_flags", "guildflags"), slash_command, prefix_command, category = "Utility", required_permissions = "MANAGE_MESSAGES")]
pub async fn guild_flags(
    ctx: Context<'_>,
    #[description = "The member whose flags are to be checked."] member: Member
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

    // Send the outcome message
    if !outcome.is_empty() {
        ctx.say(outcome).await?;
    } else {
        ctx.say("Member has no special flags.").await?;
    }

    Ok(())
}
