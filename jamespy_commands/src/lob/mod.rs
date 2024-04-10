pub mod text;
pub mod voice;

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    text::commands()
        .into_iter()
        .chain(voice::commands())
        .collect()
}
