use std::collections::HashSet;

pub fn read_words_from_file(filename: &str) -> HashSet<String> {
    std::fs::read_to_string(filename)
        .expect("Failed to read the file")
        .lines()
        .map(|line| line.trim().to_lowercase())
        .collect()
}
