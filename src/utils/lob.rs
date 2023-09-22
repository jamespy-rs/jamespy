use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

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

pub async fn update_lob() -> Result<(usize, usize), Error> {
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

pub async fn unload_lob() -> Result<(), Error> {
    let mut loblist = LOBLIST.write().unwrap();
    *loblist = HashSet::new();

    Ok(())
}

pub async fn add_lob(content: &String) -> Result<(), Error> {
    let loblist = "loblist.txt";
    let mut file = OpenOptions::new().append(true).open(loblist)?;

    let content_with_newline = if file.metadata()?.len() > 0 {
        format!("\n{}", content)
    } else {
        content.clone()
    };

    file.write_all(content_with_newline.as_bytes())?;
    Ok(())
}

pub async fn remove_lob(target: &str) -> Result<(), Error> {
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
    // Flush useless blank lines.
    let mut file = File::create(loblist)?;
    for line in lines {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}

pub fn count_lob() -> Result<usize, Error> {
    let file = File::open("loblist.txt")?;
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
    ]; // me, trontin, ruben, cv
    let user_id = ctx.author().id.get();
    let trontin = allowed_users.contains(&user_id);

    Ok(trontin)
}
