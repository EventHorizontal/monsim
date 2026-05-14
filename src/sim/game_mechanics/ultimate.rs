use std::fmt::{Debug, Display};

use monsim_macros::mon;
use monsim_utils::Outcome;

pub const MAX_ULTIMATES_PER_MONSTER: usize = 4;

pub const ULTIMATE_KIND_MEGA: UltimateKind = UltimateKind(0);
pub const ULTIMATE_KIND_ZMOVE: UltimateKind = UltimateKind(1);
pub const ULTIMATE_KIND_DYNAMAX: UltimateKind = UltimateKind(2);
pub const ULTIMATE_KIND_TERA: UltimateKind = UltimateKind(3);

// TODO: Choose available ultimates based on ruleset.
use crate::{effects, Battle, ItemSpecies, MonsterID, MonsterSpecies};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UltimateKind(pub usize);

pub trait Ultimate: Debug + Display {
    fn kind(&self) -> UltimateKind;
    fn name(&self) -> &'static str;
    /// Assumption: If you start the battle with an ultimate condition met it cannot be revoked. Also one
    /// ultimate per team per battle
    fn is_condition_met(&self, battle: &Battle, ultimate_user_id: MonsterID) -> bool;
    fn activate(&self, battle: &mut Battle, ultimate_user_id: MonsterID) -> Outcome;
    fn on_deactivate(&self, _battle: &mut Battle, _ultimate_user_id: MonsterID) -> Outcome {
        Outcome::Failure
    }
    fn call_to_action(&self) -> String;
}

#[derive(Debug, Copy, Clone)]
pub struct MegaEvolution {
    pub mega_stone: &'static ItemSpecies,
    pub mega_evolved_form: &'static MonsterSpecies,
}

impl Ultimate for MegaEvolution {
    fn kind(&self) -> UltimateKind {
        ULTIMATE_KIND_MEGA
    }

    fn name(&self) -> &'static str {
        "Mega Evolution"
    }
    fn is_condition_met(&self, battle: &Battle, ultimate_user_id: MonsterID) -> bool {
        let monster_has_compatible_megastone = mon![ultimate_user_id].held_item().is_some_and(|item| item.species == self.mega_stone);
        monster_has_compatible_megastone
    }
    fn activate(&self, battle: &mut Battle, ultimate_user_id: MonsterID) -> Outcome {
        let outcome = effects::change_form(battle, ultimate_user_id, self.mega_evolved_form);
        outcome
    }
    fn call_to_action(&self) -> String {
        String::from("Mega Evolve")
    }
}

impl Display for MegaEvolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UltimateID {
    pub user_id: MonsterID,
    pub kind: UltimateKind,
}
