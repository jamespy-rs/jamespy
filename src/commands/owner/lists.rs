use std::collections::HashSet;

use crate::{Context, Error};
use jamespy_config::read_words_from_file;

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
    let new_bad_words = read_words_from_file("config/lists/badwords.txt");
    let new_fix_words = read_words_from_file("config/lists/fixwords.txt");
    let updated_list_description = {
        let mut config = ctx.data().jamespy_config.write().unwrap();
        let badlist = config.events_config.badlist.clone().unwrap_or_default();
        let fixlist = config.events_config.badlist.clone().unwrap_or_default();

        match choice {
            Some(Lists::Badlist) => {
                let old_badlist_count = badlist.len();
                config.events_config.badlist = Some(new_bad_words);
                let new_badlist_count = badlist.len();

                format!(
                    "Updated badlist successfully!\nWords currently in the list updated from \
                     {old_badlist_count} to {new_badlist_count}"
                )
            }
            Some(Lists::Fixlist) => {
                let old_fixlist_count = fixlist.len();
                config.events_config.fixlist = Some(new_fix_words);
                let new_fixlist_count = fixlist.len();

                format!(
                    "Updated fixlist successfully!\nWords currently in the list updated from \
                     {old_fixlist_count} to {new_fixlist_count}"
                )
            }
            None => {
                let old_badlist_count = badlist.len();
                let old_fixlist_count = fixlist.len();

                config.events_config.badlist = Some(new_bad_words);
                config.events_config.fixlist = Some(new_fix_words);
                let new_badlist_count = badlist.len();
                let new_fixlist_count = fixlist.len();

                format!(
                    "Updated all lists successfully!\nbadlist: {old_badlist_count} to \
                     {new_badlist_count}\nfixlist: {old_fixlist_count} to {new_fixlist_count}"
                )
            }
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
        let mut config = ctx.data().jamespy_config.write().unwrap();
        config.events_config.badlist = Some(HashSet::new());
        config.events_config.fixlist = Some(HashSet::new());
    }

    ctx.say("Unloaded lists!").await?;
    Ok(())
}

pub fn commands() -> [crate::Command; 2] {
    [update_lists(), unload_lists()]
}
