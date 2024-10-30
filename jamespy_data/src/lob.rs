use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use std::sync::{OnceLock, RwLock};

use crate::structs::{Context, Error};
use rand::seq::IteratorRandom;

fn get_loblist() -> &'static RwLock<HashSet<String>> {
    static LOBLIST: OnceLock<RwLock<HashSet<String>>> = OnceLock::new();
    LOBLIST.get_or_init(|| {
        let data =
            std::fs::read_to_string("config/lists/loblist.txt").unwrap_or_else(|_| String::new());
        let words: HashSet<String> = data.lines().map(String::from).collect();
        RwLock::new(words)
    })
}

#[must_use]
pub fn get_random_lob() -> Option<String> {
    let loblist = get_loblist().read().unwrap();

    let mut rng = rand::thread_rng();
    loblist.iter().choose(&mut rng).cloned()
}

pub fn update_lob() -> Result<(usize, usize), Error> {
    let new_lob = std::fs::read_to_string("config/lists/loblist.txt")?;
    let old_count;

    let lines: HashSet<String> = new_lob
        .lines()
        .map(std::string::ToString::to_string)
        .collect();
    let new_count = lines.len();

    {
        let mut loblist = get_loblist().write().unwrap();
        old_count = loblist.len();
        *loblist = lines;
    }

    Ok((old_count, new_count))
}

pub fn unload_lob() -> Result<(), Error> {
    let mut loblist = get_loblist().write().unwrap();
    *loblist = HashSet::new();

    Ok(())
}

pub fn add_lob(content: &str) -> Result<(), Error> {
    let loblist = "config/lists/loblist.txt";
    let mut file = OpenOptions::new().append(true).open(loblist)?;

    let content = format!("\n{content}");

    file.write_all(content.as_bytes())?;

    let file_content = std::fs::read_to_string(loblist)?;

    let mut unique_lines = HashSet::new();
    let deduplicated_lines: Vec<&str> = file_content
        .lines()
        .filter(|&line| !line.trim().is_empty() && unique_lines.insert(line))
        .collect();

    let updated_content = deduplicated_lines.join("\n");

    std::fs::write(loblist, updated_content)?;

    Ok(())
}

pub fn remove_lob(target: &str) -> Result<bool, Error> {
    let loblist = "config/lists/loblist.txt";
    let mut lines = Vec::new();
    let mut line_removed = false;

    let file = File::open(loblist)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.trim() == target {
            line_removed = true;
        } else {
            lines.push(line);
        }
    }

    if line_removed {
        let mut file = File::create(loblist)?;
        for line in lines {
            writeln!(file, "{line}")?;
        }
    }

    Ok(line_removed)
}

pub fn count_lob() -> Result<usize, Error> {
    let file = File::open("config/lists/loblist.txt")?;
    let reader = BufReader::new(file);

    Ok(reader.lines().count())
}

// A check for Trash, so he can refresh the loblist. Includes me because, well I'm me.
// Also includes a few gg/osu mods because well why not!
#[allow(clippy::unused_async)]
pub async fn trontin(ctx: Context<'_>) -> Result<bool, Error> {
    let allowed_users = [
        158567567487795200,
        288054604548276235,
        291089948709486593,
        718513035555242086,
        326444255361105920,
        274967232596148224,
    ]; // me, trontin, ruben, cv, link, phoenix
    let user_id = ctx.author().id.get();
    if allowed_users.contains(&user_id) {
        return Ok(true);
    }

    Err("You are not worthy of the lob.".into())
}
