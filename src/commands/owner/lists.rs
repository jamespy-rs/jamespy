use std::collections::HashSet;

use crate::event_handlers::messages::{BADLIST, FIXLIST};
use crate::utils::misc::read_words_from_file;
use crate::{Context, Error};

#[derive(Debug, poise::ChoiceParameter)]
pub enum Lists {
    Badlist,
    Fixlist,
}

#[poise::command(
    rename = "update-lists",
    aliases(
        "updatelists",
        "update_lists",
        "update-list",
        "updatelist",
        "update_lists"
    ),
    prefix_command,
    category = "Lists",
    owners_only,
    hide_in_help
)]
pub async fn update_lists(
    ctx: Context<'_>,
    #[description = "What list to unload"] choice: Option<Lists>,
) -> Result<(), Error> {
    let new_bad_words = read_words_from_file("badwords.txt");
    let new_fix_words = read_words_from_file("fixwords.txt");

    let updated_list_description = match choice {
        Some(Lists::Badlist) => {
            let mut badlist = BADLIST.write().unwrap();
            let old_badlist_count = badlist.len();
            *badlist = new_bad_words;
            let new_badlist_count = badlist.len();

            let updated_list_description = format!(
                "Updated badlist successfully!\nWords currently in the list updated from {} to {}",
                old_badlist_count, new_badlist_count
            );

            updated_list_description
        }
        Some(Lists::Fixlist) => {
            let mut fixlist = FIXLIST.write().unwrap();
            let old_fixlist_count = fixlist.len();
            *fixlist = new_fix_words;
            let new_fixlist_count = fixlist.len();

            let updated_list_description = format!(
                "Updated fixlist successfully!\nWords currently in the list updated from {} to {}",
                old_fixlist_count, new_fixlist_count
            );

            updated_list_description
        }
        None => {
            let mut badlist = BADLIST.write().unwrap();
            let mut fixlist = FIXLIST.write().unwrap();
            let old_badlist_count = badlist.len();
            let old_fixlist_count = fixlist.len();

            *badlist = new_bad_words;
            *fixlist = new_fix_words;
            let new_badlist_count = badlist.len();
            let new_fixlist_count = fixlist.len();

            let updated_list_description = format!(
                "Updated all lists successfully!\nbadlist: {} to {}\nfixlist: {} to {}",
                old_badlist_count, new_badlist_count, old_fixlist_count, new_fixlist_count
            );

            updated_list_description
        }
    };

    ctx.say(updated_list_description).await?;
    Ok(())
}

#[poise::command(
    rename = "unload-lists",
    prefix_command,
    category = "Lists",
    owners_only,
    hide_in_help
)]
pub async fn unload_lists(ctx: Context<'_>) -> Result<(), Error> {
    {
        let mut badlist = BADLIST.write().unwrap();
        *badlist = HashSet::new()
    }

    {
        let mut fixlist = FIXLIST.write().unwrap();
        *fixlist = HashSet::new();
    }

    ctx.say("Unloaded lists!").await?;
    Ok(())
}
