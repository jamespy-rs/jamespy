pub mod cache;
pub mod database;
pub mod lists;
pub mod other;
pub mod presence;
pub mod vcstatus;

pub mod castle;

pub fn commands() -> Vec<crate::Command> {
    {
        cache::commands()
            .into_iter()
            .chain(database::commands())
            .chain(lists::commands())
            .chain(other::commands())
            .chain(presence::commands())
            .chain(vcstatus::commands())
            .chain(castle::commands())
            .collect()
    }
}
