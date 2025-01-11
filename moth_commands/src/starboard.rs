use crate::{Context, Error};

use moth_data::database::StarboardStatus;
use poise::serenity_prelude::{self as serenity, UserId};

use itertools::Itertools;
use std::fmt::Write;

#[poise::command(prefix_command, hide_in_help, guild_only, check = "allowed_user")]
pub async fn list_queued(ctx: Context<'_>) -> Result<(), Error> {
    let sorted_starboard = ctx
        .data()
        .database
        .get_all_starboard()
        .await?
        .iter()
        .filter(|m| m.starboard_status == StarboardStatus::InReview)
        .sorted_by(|a, b| b.star_count.cmp(&a.star_count))
        .cloned()
        .collect::<Vec<_>>();

    let mut description = String::new();

    for entry in sorted_starboard {
        // hardcoded GuildId because its a single guild bot
        let link = format!(
            "https://discord.com/channels/98226572468690944/{}/{}",
            *entry.starboard_message_channel, *entry.starboard_message_id
        );
        writeln!(description, "{} ⭐ {link}", entry.star_count).unwrap();
    }

    // TODO: won't be a problem for some time but paginating this command would be good, but i'm too lazy.
    if description.len() > 4000 {
        ctx.say(
            "Output is too long, printed into terminal instead. Ask james for the output then \
             force them to paginate this",
        )
        .await?;
        println!("{description}");
    } else {
        let embed = serenity::CreateEmbed::new()
            .title("Starboard entries in review")
            .description(description)
            .colour(serenity::Colour::BLUE);
        let builder = poise::CreateReply::new().embed(embed);
        ctx.send(builder).await?;
    }

    Ok(())
}

#[must_use]
pub fn commands() -> [crate::Command; 1] {
    [list_queued()]
}

// TODO: dedupe this with moth_core
async fn allowed_user(ctx: Context<'_>) -> Result<bool, Error> {
    // Phil, Ruben, me
    let a = [
        UserId::new(101090238067113984),
        UserId::new(291089948709486593),
        UserId::new(158567567487795200),
    ];

    Ok(a.contains(&ctx.author().id))
}
