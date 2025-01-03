mod components;
pub mod starboard;

pub use components::handle_component;
pub use starboard::{starboard_add_handler, starboard_remove_handler};

pub(crate) use jamespy_data::structs::{Data, Error};
