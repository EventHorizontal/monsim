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

use core::marker::Copy;

pub use ability::*;
pub use monster::*;
pub use move_::*;
pub use team::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ElementalType {
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
