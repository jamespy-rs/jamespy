use crate::Command;

pub mod meta;

pub fn commands() -> Vec<Command> {
    meta::commands().into_iter().collect()
}
