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

pub static LINKS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https?://\S+|www\.\S+").unwrap());

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
// can I later just avoid allocating the String?
fn preprocess(content: &str) -> Cow<'_, str> {
    let mut offset = 0;

    let processed = EMOJI_REGEX.replace_all(content, |caps: &regex::Captures| {
        let processed = &caps[2];
        offset += processed.len();
        processed.to_string()
    });

    let processed = MENTIONS.replace_all(&processed, |caps: &regex::Captures| {
        let mention = &caps[0];
        offset += mention.len();
        ""
    });
    let processed = LINKS.replace_all(&processed, |caps: &regex::Captures| {
        let url = &caps[0];
        offset += url.len();
        ""
    });

    if processed == content {
        Cow::Borrowed(content)
    } else {
        Cow::Owned(processed.into())
    }
}

pub fn filter_content<'a>(
    content: &'a str,
    badlist: &HashSet<String>,
    fixlist: &HashSet<String>,
) -> Cow<'a, str> {
    let mut changed_words = Vec::new();
    let threshold = (Type::PROFANE | Type::OFFENSIVE)
        & !(Type::EVASIVE | Type::SPAM)
        & (Type::MODERATE | Type::SEVERE);

    let processed = preprocess(content);
    let mut censor = Censor::from_str(&processed);
    let censor = censor.with_trie(get_trie());
    let censor = censor.with_censor_threshold(threshold);

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
