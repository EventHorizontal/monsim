pub mod ability;
#[cfg(feature = "debug")]
pub(crate) mod test_ability_dex;
pub mod monster;
#[cfg(feature = "debug")]
pub(crate) mod test_monster_dex;
pub mod move_;
#[cfg(feature = "debug")]
pub(crate) mod test_move_dex;
pub mod team;
pub mod status;
#[cfg(feature = "debug")]
pub(crate) mod test_status_dex;

#[cfg(feature = "debug")]
pub(crate) mod test_item_dex;
pub mod item;

pub use ability::*;
pub use monster::*;
pub use move_::*;
pub use team::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Type {
    Bug,
    Dark,
    Dragon,
    Electric,
    Fairy,
    Fighting,
    Fire,
    Flying,
    Ghost,
    Grass,
    Ground,
    Ice,
    Normal,
    Poison,
    Psychic,
    Rock,
    Steel,
    Water,
}
