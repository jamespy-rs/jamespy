use poise::serenity_prelude::{self as serenity, Attachment, CreateAttachment, EditProfile};

use crate::{Context, Error};

/// Change specific settings of jamespy!
#[poise::command(prefix_command, hide_in_help, owners_only)]
pub async fn jamespy(ctx: Context<'_>, attachment: Attachment) -> Result<(), Error> {
    let avatar = CreateAttachment::url(ctx, &attachment.url).await?;
    let builder = EditProfile::default().avatar(&avatar);
    let mut user = ctx.cache().current_user().clone(); // cacheref aaaaaa
    user.edit(ctx, builder).await?;

    Ok(())
}
