pub mod cache;
pub mod database;
pub mod other;
pub mod presence;

pub fn commands() -> Vec<crate::Command> {
    {
        cache::commands()
            .into_iter()
            .chain(database::commands())
            .chain(presence::commands())
            .chain(other::commands())
            .collect()
    }
}
