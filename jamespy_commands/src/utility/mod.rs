pub mod join_tracks;
pub mod random;
pub mod users;
pub mod guild;

#[must_use]
pub fn commands() -> Vec<crate::Command> {
    {
        join_tracks::commands()
            .into_iter()
            .chain(random::commands())
            .chain(users::commands()).chain(guild::commands())
            .collect()
    }
}
