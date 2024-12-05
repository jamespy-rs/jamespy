use std::fmt::Write;
use std::{borrow::Cow, collections::HashSet};

use rustrict::{Censor, Type};

pub static WHITESPACE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"(\s*)(\S+)").unwrap());

pub fn filter_content<'a>(
    content: &'a str,
    badlist: &HashSet<String>,
    fixlist: &HashSet<String>,
) -> (Cow<'a, str>, Vec<&'a str>) {
    let mut changed_words = Vec::new();
    let mut censor = Censor::from_str(content);

    content.split_whitespace().for_each(|word| {
        let is_blacklisted = badlist.iter().any(|badword| word.contains(badword))
            && !fixlist.iter().any(|fixword| word.contains(fixword));
        if is_blacklisted {
            changed_words.push(word);
        }
    });

    if censor.analyze() != Type::NONE {
        // scuffed stuff.
        censor.reset(content.chars());
        // it doesn't return the difference, there's no other way than to do some weird comparison.
        let censored = censor.censor();

        let mut orig = content.split_whitespace();
        let mut censored = censored.split_whitespace();

        loop {
            match (orig.next(), censored.next()) {
                (Some(w1), Some(w2)) if w1 != w2 => {
                    changed_words.push(w1);
                }
                (Some(w1), None) => changed_words.push(w1),
                (Some(_) | None, Some(_)) => continue,
                (None, None) => break,
            }
        }
    }

    if changed_words.is_empty() {
        (Cow::Borrowed(content), changed_words)
    } else {
        (
            Cow::Owned(colour_string(content, &changed_words)),
            changed_words,
        )
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
            write!(result, "\x1B[1m\x1B[31m{word}\x1B[0m").unwrap();
        } else {
            result.push_str(word);
        }
    }

    result
}
