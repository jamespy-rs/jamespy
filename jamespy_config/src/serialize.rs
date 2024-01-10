use std::collections::HashSet;

pub fn read_words_from_file(filename: &str) -> HashSet<String> {
    std::fs::read_to_string(filename)
        .expect("Failed to read the file")
        .lines()
        .map(|line| line.trim().to_lowercase())
        .collect()
}

pub mod regex_patterns {
    use base64::engine::{general_purpose, Engine as _};
    use regex::Regex;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<Regex>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let patterns: Option<Vec<String>> = Option::deserialize(deserializer)?;

        let regex_patterns = patterns.map(|patterns| {
            patterns
                .into_iter()
                .filter_map(|pattern| {
                    let bytes = general_purpose::STANDARD.decode(pattern).unwrap();
                    let pattern = String::from_utf8(bytes).unwrap();
                    Regex::new(&pattern).ok()
                })
                .collect()
        });
        Ok(regex_patterns)
    }

    pub fn serialize<S: Serializer>(
        patterns: &Option<Vec<Regex>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let unwrapped = patterns.as_ref().cloned().unwrap_or_default();
        let mut new: Vec<String> = Vec::new();

        for pattern in unwrapped {
            new.push(general_purpose::STANDARD.encode(pattern.as_str()));
        }

        serializer.collect_seq(new)
    }
}
