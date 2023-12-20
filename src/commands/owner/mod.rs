pub mod cache;
pub mod database;
pub mod lists;
pub mod other;
pub mod presence;
pub mod vcstatus;

#[cfg(feature = "castle")]
pub mod castle;

pub fn commands() -> Vec<crate::Command> {
    // cleanup another day.
    #[cfg(not(feature = "castle"))]
    cache::commands()
        .into_iter()
        .chain(database::commands())
        .chain(lists::commands())
        .chain(other::commands())
        .chain(presence::commands())
        .chain(vcstatus::commands())
        .collect();

    #[cfg(feature = "castle")]
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
