use poise::serenity_prelude::{self as serenity, ActivityData, ActivityType};

use crate::{Data, Error};

pub async fn ready(ctx: &serenity::Context, _data: &Data) -> Result<(), Error> {
    ctx.cache.set_max_messages(350);

    let activity_data = ActivityData {
        name: "you inside your home.".to_string(),
        kind: ActivityType::Watching,
        state: None,
        url: None,
    };
    ctx.set_activity(Some(activity_data));

    Ok(())
}
