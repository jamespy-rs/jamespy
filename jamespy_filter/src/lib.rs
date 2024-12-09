use std::fmt::Write;
use std::sync::{LazyLock, OnceLock};
use std::{borrow::Cow, collections::HashSet};

use regex::Regex;
use rustrict::{Censor, Trie, Type};

use jamespy_ansi::{BOLD, RED, RESET};

pub static WHITESPACE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"(\s*)(\S+)").unwrap());

pub static EMOJI_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<(a)?:([a-zA-Z0-9_]{2,32}):(\d{1,20})>").unwrap());
pub static MENTIONS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<(#|@&?)([a-zA-Z0-9_]{2,32})>").unwrap());
pub static NUMBERS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b\d{5,20}\b").unwrap());
pub static LINKS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https?://\S+|www\.\S+").unwrap());

fn get_threshold() -> Type {
    (Type::PROFANE | Type::OFFENSIVE)
        & !(Type::EVASIVE | Type::SPAM)
        & (Type::MODERATE | Type::SEVERE)
}

fn get_trie() -> &'static Trie {
    static TRIE: OnceLock<Trie> = OnceLock::new();
    TRIE.get_or_init(|| {
        let mut trie = Trie::default();
        // patch fix
        trie.set("fcing", Type::SAFE);
        trie.set("pp", Type::SAFE);
        trie.set("ppcat", Type::SAFE);

        trie
    })
}

/// A function that cleans stuff up that rustrict isn't good with.
pub fn preprocess(content: &str) -> Cow<'_, str> {
    let processed = EMOJI_REGEX.replace_all(content, |caps: &regex::Captures| {
        let matched = caps.get(2).unwrap();
        let start = matched.start();
        let end = matched.end();
        &content[start..end]
    });

    // mentions falsely trigger for spam/evasivze.
    let processed = MENTIONS.replace_all(&processed, |_: &regex::Captures| "");
    // remove a reasonable set of numbers, because for whatever reason its inappropriate.
    let processed = NUMBERS.replace_all(&processed, |_: &regex::Captures| "");
    // links can falsely trigger stuff, so they are ommitted.
    let processed = LINKS.replace_all(&processed, |_: &regex::Captures| "");

    if processed == content {
        Cow::Borrowed(content)
    } else {
        Cow::Owned(processed.into())
    }
}

pub fn analyze(content: &str) -> Type {
    let processed = preprocess(content);
    let mut censor = Censor::from_str(&processed);
    let censor = censor
        .with_trie(get_trie())
        .with_censor_threshold(get_threshold());

    censor.analyze()
}

pub fn filter_content<'a>(
    content: &'a str,
    badlist: &HashSet<String>,
    fixlist: &HashSet<String>,
) -> Cow<'a, str> {
    let mut changed_words = Vec::new();
    let processed = preprocess(content);
    let mut censor = Censor::from_str(&processed);
    let censor = censor
        .with_trie(get_trie())
        .with_censor_threshold(get_threshold());

    content.split_whitespace().for_each(|word| {
        let is_blacklisted = badlist.iter().any(|badword| word.contains(badword))
            && !fixlist.iter().any(|fixword| word.contains(fixword));
        if is_blacklisted {
            changed_words.push(word);
        }
    });

    if censor.analyze() != Type::NONE {
        censor.reset(processed.chars());
        let censored = censor.censor();

        let mut orig_iter = content.split_whitespace();
        let mut mapped_iter = processed.split_whitespace();
        let mut censored_iter = censored.split_whitespace();

        loop {
            match (orig_iter.next(), mapped_iter.next(), censored_iter.next()) {
                (Some(w1), Some(mapped), Some(w2)) if mapped != w2 => {
                    changed_words.push(w1);
                }
                (Some(w1), None, _) => changed_words.push(w1),
                (Some(_), Some(_), None) => continue,
                (None, None, None) => break,
                _ => continue,
            }
        }
    }

    if changed_words.is_empty() {
        Cow::Borrowed(content)
    } else {
        Cow::Owned(colour_string(content, &changed_words))
    }
}

fn colour_string(content: &str, changed_words: &[&str]) -> String {
    let mut result = String::new();
    for cap in WHITESPACE.captures_iter(content) {
        // leading whitespace
        let leading_whitespace = &cap[1];
        // The word
        let word = &cap[2];

        result.push_str(leading_whitespace);

        if changed_words.contains(&word) {
            write!(result, "{BOLD}{RED}{word}{RESET}").unwrap();
        } else {
            result.push_str(word);
        }
    }

    result
}
