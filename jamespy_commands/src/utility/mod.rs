pub mod join_tracks;
pub mod random;

pub fn commands() -> Vec<crate::Command> {
    {
        join_tracks::commands()
            .into_iter()
            .chain(random::commands())
            .collect()
    }
}
