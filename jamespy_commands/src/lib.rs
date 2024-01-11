use jamespy_data::structs::{Command, Context, Error};

pub mod meta;
pub mod owner;
pub mod general;
pub mod utility;

pub fn commands() -> Vec<Command> {
    meta::commands()
        .into_iter()
        .chain(owner::commands()).chain(general::commands()).chain(utility::commands())
        .collect()
}
