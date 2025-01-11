mod text;

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    text::commands().into_iter().collect()
}
