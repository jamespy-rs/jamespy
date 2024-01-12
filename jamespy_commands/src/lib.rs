use jamespy_data::structs::{Command, Context, Error};

pub mod general;
pub mod meta;
pub mod owner;
pub mod utility;

pub mod utils;

pub fn commands() -> Vec<Command> {
    meta::commands()
        .into_iter()
        .chain(owner::commands())
        .chain(general::commands())
        .chain(utility::commands())
        .collect()
}
