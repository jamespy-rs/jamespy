pub mod lob;

pub fn commands() -> Vec<crate::Command> {
    lob::commands().into()
}
