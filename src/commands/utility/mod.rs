pub mod info;
pub mod join_tracks;
pub mod random;
pub mod users;

pub fn commands() -> Vec<crate::Command> {
    info::commands()
        .into_iter()
        .chain(join_tracks::commands())
        .chain(random::commands())
        .chain(users::commands())
        .collect()
}
