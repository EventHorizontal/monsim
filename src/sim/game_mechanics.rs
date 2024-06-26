pub mod ability;
pub mod environment;
pub mod item;
pub mod monster;
pub mod move_;
pub mod status;
pub mod team;
pub(crate) mod types;

pub use ability::*;
pub use environment::*;
pub use item::*;
pub use monster::*;
pub use move_::*;
pub use team::*;
pub use types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MechanicKind {
    Monster,
    Move,
    Ability,
    Item,
    VolatileStatus,
    PersistentStatus,
    Weather,
    Terrain,
    Trap { team_id: TeamID },
}
