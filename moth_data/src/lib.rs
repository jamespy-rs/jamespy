#![warn(clippy::pedantic)]
// clippy warns for u64 -> i64 conversions despite this being totally okay in this scenario.
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::unreadable_literal,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

pub mod database;
pub mod lob;
pub mod structs;
