use crate::event_handlers::messages::{BADLIST, FIXLIST};
use crate::utils::misc::read_words_from_file;
use crate::{Context, Error};

#[poise::command(
    rename = "update-lists",
    prefix_command,
    category = "Lists",
    owners_only,
    hide_in_help
)]
pub async fn update_lists(ctx: Context<'_>) -> Result<(), Error> {
    // Read new words from files
    let new_bad_words = read_words_from_file("badwords.txt");
    let new_fix_words = read_words_from_file("fixwords.txt");

    // Update BADLIST
    {
        let mut badlist = BADLIST.write().unwrap();
        *badlist = new_bad_words;
    }

    // Update FIXLIST
    {
        let mut fixlist = FIXLIST.write().unwrap();
        *fixlist = new_fix_words;
    }

    ctx.say("Updated lists!").await?;
    Ok(())
}
