use crate::{Data, Error};
use poise::serenity_prelude::{self as serenity, ActivityData, ActivityType, Ready};

use small_fixed_array::FixedString;

use std::sync::atomic::Ordering;
use std::sync::Arc;

pub async fn ready(ctx: &serenity::Context, ready: &Ready, data: Arc<Data>) -> Result<(), Error> {
    let activity_data = ActivityData {
        name: FixedString::from_str_trunc("you inside your home."),
        kind: ActivityType::Watching,
        state: None,
        url: None,
    };
    ctx.set_activity(Some(activity_data));

    let shard_count = ctx.cache.shard_count();
    let is_last_shard = (ctx.shard_id.0 + 1) == shard_count.get();

    if is_last_shard && !data.has_started.swap(true, Ordering::SeqCst) {
        println!("Logged in as {}", ready.user.tag());
    }

    Ok(())
}
