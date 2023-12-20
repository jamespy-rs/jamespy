use crate::Command;

pub mod general;
pub mod meta;
pub mod owner;
pub mod utility;

pub fn commands() -> Vec<Command> {
    meta::commands()
        .into_iter()
        .chain(general::commands())
        .chain(utility::commands())
        .chain(owner::commands())
        .collect()
}
