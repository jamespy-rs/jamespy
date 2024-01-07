use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use std::sync::{OnceLock, RwLock};

use jamespy_data::structs::{Context, Error};
use rand::seq::SliceRandom;

fn get_loblist() -> &'static RwLock<HashSet<String>> {
    static LOBLIST: OnceLock<RwLock<HashSet<String>>> = OnceLock::new();
    LOBLIST.get_or_init(|| {
        let data = std::fs::read_to_string("config/lists/loblist.txt").unwrap_or_else(|_| String::new());
        let words: HashSet<String> = data.lines().map(String::from).collect();
        RwLock::new(words)
    })
}

pub fn get_random_lob() -> Option<String> {
    let loblist = get_loblist().read().unwrap();

    let options: Vec<String> = loblist.iter().cloned().collect();

    let mut rng = rand::thread_rng();
    options.choose(&mut rng).cloned()
}

pub async fn update_lob() -> Result<(usize, usize), Error> {
    let new_lob = std::fs::read_to_string("config/lists/loblist.txt")?;
    let old_count;
    let new_count;

    {
        let mut loblist = get_loblist().write().unwrap();
        let lines: HashSet<String> = new_lob
            .lines()
            .map(std::string::ToString::to_string)
            .collect();
        old_count = loblist.len();
        *loblist = lines;
        new_count = loblist.len();
    }

    Ok((old_count, new_count))
}

pub async fn unload_lob() -> Result<(), Error> {
    let mut loblist = get_loblist().write().unwrap();
    *loblist = HashSet::new();

    Ok(())
}

pub async fn add_lob(content: &String) -> Result<(), Error> {
    let loblist = "config/lists/loblist.txt";
    let mut file = OpenOptions::new().append(true).open(loblist)?;

    let content_with_newline = if file.metadata()?.len() > 0 {
        format!("\n{content}")
    } else {
        content.clone()
    };

    file.write_all(content_with_newline.as_bytes())?;

    let file_content = std::fs::read_to_string(loblist)?;
    let lines: Vec<&str> = file_content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    let updated_content = lines.join("\n");

    std::fs::write(loblist, updated_content)?;

    Ok(())
}

pub async fn remove_lob(target: &str) -> Result<bool, Error> {
    let loblist = "config/lists/loblist.txt";
    let mut lines = Vec::new();
    let mut line_removed = false;

    {
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

    let mut count = 0;
    for _ in reader.lines() {
        count += 1;
    }

    Ok(count)
}

// A check for Trash, so he can refresh the loblist. Includes me because, well I'm me.
// Also includes a few gg/osu mods because well why not!
pub async fn trontin(ctx: Context<'_>) -> Result<bool, Error> {
    let allowed_users = [
        158567567487795200,
        288054604548276235,
        291089948709486593,
        718513035555242086,
        326444255361105920,
    ]; // me, trontin, ruben, cv, link
    let user_id = ctx.author().id.get();
    let trontin = allowed_users.contains(&user_id);

    Ok(trontin)
}
