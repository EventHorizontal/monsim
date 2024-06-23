use crate::{
    sim::{MonsterID, MoveID},
    AbilityID, ItemID, ModifiableStat, PersistentStatusSpecies, VolatileStatusSpecies,
};
use monsim_utils::MaxSizedVec;

use super::EventContext;

/// `move_user_id`: MonsterID of the Monster using the move.
///
/// `move_used_id`: MoveID of the Move being used.
///
/// `target_ids`: MonsterIDs of the Monsters the move is being used on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveUseContext {
    pub move_user_id: MonsterID,
    pub move_used_id: MoveID,
    pub target_ids: MaxSizedVec<MonsterID, 6>,
}

impl EventContext for MoveUseContext {}

impl MoveUseContext {
    pub fn new(move_used_id: MoveID, target_ids: MaxSizedVec<MonsterID, 6>) -> Self {
        Self {
            move_user_id: move_used_id.owner_id,
            move_used_id,
            target_ids,
        }
    }
}

/// `move_user_id`: MonsterID of the Monster hitting.
///
/// `move_used_id`: MoveID of the Move being used.
///
/// `target_id`: MonsterID of the Monster being hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveHitContext {
    pub move_user_id: MonsterID,
    pub move_used_id: MoveID,
    pub target_id: MonsterID,
    pub number_of_hits: u8,
    pub number_of_targets: u8,
}

impl MoveHitContext {
    pub fn new(move_used_id: MoveID, target_id: MonsterID, number_of_hits: u8, number_of_targets: u8) -> Self {
        Self {
            move_user_id: move_used_id.owner_id,
            move_used_id,
            target_id,
            number_of_hits,
            number_of_targets,
        }
    }
}

impl EventContext for MoveHitContext {
    fn target(&self) -> Option<MonsterID> {
        Some(self.target_id)
    }
}

/// `ability_owner_id`: MonsterID of the Monster whose ability is being used.
///
/// `ability_used_id`: AbilityID of the Ability being used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbilityActivationContext {
    pub ability_owner_id: MonsterID,
    pub ability_used_id: AbilityID,
}

impl AbilityActivationContext {
    pub fn from_owner(ability_owner: MonsterID) -> Self {
        Self {
            ability_used_id: AbilityID { owner_id: ability_owner },
            ability_owner_id: ability_owner,
        }
    }
}

impl EventContext for AbilityActivationContext {}

/// `active_monster_id`: MonsterID of the Monster to be switched out.
///
/// `benched_monster_id`: MonsterID of the Monster to be switched in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchContext {
    pub active_monster_id: MonsterID,
    pub benched_monster_id: MonsterID,
}

impl SwitchContext {
    pub fn new(active_monster_id: MonsterID, benched_monster_id: MonsterID) -> Self {
        Self {
            active_monster_id,
            benched_monster_id,
        }
    }
}

impl EventContext for SwitchContext {}

/// `active_monster_id`: MonsterID of the Monster to be switched out.
///
/// `benched_monster_id`: MonsterID of the Monster to be switched in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemUseContext {
    pub item_id: ItemID,
    pub item_holder_id: MonsterID,
}

impl ItemUseContext {
    pub fn from_holder(item_holder_id: MonsterID) -> Self {
        let item_id = ItemID::from_holder(item_holder_id);
        Self { item_id, item_holder_id }
    }
}

impl EventContext for ItemUseContext {}

/// `active_monster_id`: MonsterID of the Monster to be switched out.
///
/// `benched_monster_id`: MonsterID of the Monster to be switched in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatChangeContext {
    pub affected_monster_id: MonsterID,
    pub stat: ModifiableStat,
    pub number_of_stages: i8,
}

impl EventContext for StatChangeContext {}

#[derive(Debug, Clone, Copy)]
pub struct InflictPersistentStatusContext {
    pub affected_monster_id: MonsterID,
    pub status_condition: &'static PersistentStatusSpecies,
}

impl EventContext for InflictPersistentStatusContext {}

#[derive(Debug, Clone, Copy)]
pub struct InflictVolatileStatusContext {
    pub affected_monster_id: MonsterID,
    pub status_condition: &'static VolatileStatusSpecies,
}

impl EventContext for InflictVolatileStatusContext {}
