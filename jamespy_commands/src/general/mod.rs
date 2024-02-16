pub mod lob;

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    lob::commands().into()
}
