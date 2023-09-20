use std::collections::HashSet;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::sync::{Arc, RwLock};

use crate::utils::misc::read_words_from_file;
use crate::{Context, Error};
use lazy_static::lazy_static;
use rand::seq::SliceRandom;

lazy_static! {
    pub static ref LOBLIST: Arc<RwLock<HashSet<String>>> = {
        let data = std::fs::read_to_string("loblist.txt").unwrap_or_else(|_| String::new());
        let words: HashSet<String> = data.lines().map(String::from).collect();

        Arc::new(RwLock::new(words))
    };
}

pub fn get_random_lob() -> Option<String> {
    let loblist = LOBLIST.read().ok()?;

    let options: Vec<String> = loblist.iter().cloned().collect();

    let mut rng = rand::thread_rng();
    options.choose(&mut rng).cloned()
}

async fn update_lob() -> Result<(usize, usize), Error> {
    let new_lob = read_words_from_file("loblist.txt");
    let old_count;
    let new_count = new_lob.len();

    {
        let mut loblist = LOBLIST.write().unwrap();
        old_count = loblist.len();
        *loblist = new_lob;
    }

    Ok((old_count, new_count))
}

async fn unload_lob() -> Result<(), Error> {
    let mut loblist = LOBLIST.write().unwrap();
    *loblist = HashSet::new();

    Ok(())
}


/// i lob
#[poise::command(
    slash_command,
    prefix_command,
    category = "Utility",
    channel_cooldown = "5"
)]
pub async fn lob(ctx: Context<'_>) -> Result<(), Error> {
    let option = get_random_lob();
    if let Some(lob) = option {
        ctx.say(lob).await?;
    }

    Ok(())
}

/// reload lob
#[poise::command(
    rename = "reload-lob",
    aliases("reloadlob", "reload_lob", "update-lob", "updatelob", "update_lob"),
    prefix_command,
    category = "Utility",
    global_cooldown = "5",
    check = "trontin",
    hide_in_help
)]
pub async fn reload_lob(ctx: Context<'_>) -> Result<(), Error> {
    let (old_count, new_count) = update_lob().await?;
    ctx.say(format!("Reloaded lobs.\nOld Count: {}\nNew Count: {}", old_count, new_count)).await?;
    Ok(())
}

/// no lob
#[poise::command(
    rename = "unload-lob",
    aliases("unloadlob", "unload_lob"),
    prefix_command,
    category = "Utility",
    global_cooldown = "5",
    check = "trontin",
    hide_in_help
)]
pub async fn no_lob(ctx: Context<'_>) -> Result<(), Error> {
    unload_lob().await?;
    ctx.say(format!("Unloaded lob!")).await?;
    Ok(())
}


async fn add_lob(content: &String) -> Result<(), Error> {
    let loblist = "loblist.txt";
    let mut file = OpenOptions::new()
        .append(true)
        .open(loblist)?;

    file.write_all(content.as_bytes())?;
    Ok(())
}

async fn remove_lob(target: &str) -> Result<(), Error> {
    let loblist = "loblist.txt";
    let mut lines = Vec::new();

    {
        let file = File::open(loblist)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim() != target {
                lines.push(line);
            }
        }
    }

    // Write the updated lines back to the file
    let mut file = File::create(loblist)?;
    for line in lines {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}

#[poise::command(
    rename = "add-lob",
    aliases("addlob", "add_lob"),
    slash_command,
    prefix_command,
    category = "Utility",
    channel_cooldown = "5",
    check = "trontin"
)]
pub async fn new_lob(
    ctx: Context<'_>,
    #[description = "new lob"] item: String,
) -> Result<(), Error> {
    add_lob(&item).await?;
    ctx.send(poise::CreateReply::default().content(format!("Added `{}` to loblist!\nChanges will not be applied until bot restart or until reload-lob is called!", item))).await?;
    Ok(())
}

#[poise::command(
    rename = "remove-lob",
    aliases("removelob", "remove_lob"),
    slash_command,
    prefix_command,
    category = "Utility",
    channel_cooldown = "5",
    check = "trontin"
)]
pub async fn delete_lob(
    ctx: Context<'_>,
    #[description = "Lob to remove"] target: String,
) -> Result<(), Error> {
    remove_lob(&target).await?;
    ctx.send(poise::CreateReply::default().content(format!("Removed `{}` from loblist!\nChanges will not be applied until bot restart or until reload-lob is called!", target))).await?;
    Ok(())
}


// A check for Trash, so he can refresh the loblist. Includes me because, well I'm me.
// Also includes a few gg/osu mods because well why not!
async fn trontin(ctx: Context<'_>) -> Result<bool, Error> {
    let allowed_users = vec![158567567487795200, 288054604548276235, 291089948709486593, 718513035555242086]; // me, trontin, ruben, cv
    let user_id = ctx.author().id.get();
    let trontin = allowed_users.contains(&user_id);

    Ok(trontin)
}
