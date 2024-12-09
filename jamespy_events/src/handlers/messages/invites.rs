use std::sync::{Arc, LazyLock};

use jamespy_data::structs::Data;
use poise::serenity_prelude::{self as serenity, ChannelId, CreateMessage, GuildId, Message};
use regex::Regex;

use ::serenity::all::{CreateAllowedMentions, CreateEmbedAuthor};
use resvg::{tiny_skia::Pixmap, usvg::Tree};

use crate::Error;

pub static INVITE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"discord(?:(?:app)?\.com/invite|\.gg)/([a-z0-9-]+)").unwrap());

pub async fn moderate_invites(
    ctx: &serenity::Context,
    data: &Arc<Data>,
    msg: &Message,
) -> Result<(), Error> {
    let Some(guild_id) = msg.guild_id else {
        return Ok(());
    };

    if guild_id != GuildId::new(98226572468690944) {
        return Ok(());
    };

    let mut invites = Vec::new();
    for invite in INVITE.find_iter(&msg.content) {
        let Ok(details) = ctx
            .http
            .get_invite(invite.as_str(), false, false, None)
            .await
        else {
            continue;
        };

        println!("here");

        let Some(guild) = details.guild else {
            continue;
        };

        invites.push((details.code, guild.name));
    }

    let mut embeds = Vec::with_capacity(invites.len());
    let mut builder = CreateMessage::new().allowed_mentions(
        CreateAllowedMentions::new()
            .all_users(false)
            .all_roles(false)
            .everyone(false),
    );
    let mut first_name = None;
    for (index, (code, name)) in invites.iter().enumerate() {
        let Ok(response) = data
            .reqwest
            .get(format!("https://invidget.switchblade.xyz/{code}"))
            .send()
            .await
        else {
            continue;
        };

        // https://github.com/SwitchbladeBot/invidget/pull/82
        let text = response.text().await?.replace("image/jpg", "image/jpeg");

        let Ok(png_data) = convert_svg_to_png(&text) else {
            continue;
        };

        let attachment_name = format!("{index}.png");
        let attachment = serenity::CreateAttachment::bytes(png_data, attachment_name.clone());
        builder = builder.add_file(attachment);

        let mut embed = serenity::CreateEmbed::new()
            .attachment(attachment_name)
            .description(format!("https://discord.gg/{code}"));

        if index == 0 {
            embed = embed.author(CreateEmbedAuthor::from(&msg.author));
        }

        if first_name.is_none() {
            first_name = Some(name);
        };

        embeds.push(embed);
    }

    builder = builder.embeds(&embeds);

    if !embeds.is_empty() {
        if embeds.len() == 1 {
            builder = builder.content(format!(
                "{} posted an invite to {} in <#{}>",
                msg.author,
                first_name.expect("This should always be populated."),
                msg.channel_id
            ));
        } else {
            builder = builder.content(format!(
                "{} posted multiple invites in <#{}>",
                msg.author, msg.channel_id
            ));
        }

        ChannelId::new(277163440999628800)
            .send_message(&ctx.http, builder)
            .await?;
    }

    Ok(())
}

fn convert_svg_to_png(svg_data: &str) -> Result<Vec<u8>, Error> {
    let tree = Tree::from_str(svg_data, &usvg::Options::default())?;

    let size = tree.size().to_int_size();
    let mut pixmap = Pixmap::new(size.width(), size.height()).expect("Failed to create a pixmap");

    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    Ok(pixmap.encode_png()?)
}
