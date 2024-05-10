use std::ops::{IndexMut, Index};

use monsim_utils::MaxSizedVec;

use crate::{ActivationOrder, BattleState, MonsterID};

use super::{game_mechanics::MoveID, targetting::FieldPosition};

pub trait SimulatorUi {
    fn update_battle_status(&self, battle: &mut BattleState);
    
    fn prompt_user_to_select_action_for_monster(&self, battle: &mut BattleState, monster_id: MonsterID, available_choices_for_monster: AvailableChoices) -> PartiallySpecifiedActionChoice;
    
    fn prompt_user_to_select_target_position(&self, battle: &mut BattleState, move_id: MoveID, possible_targets: MaxSizedVec<FieldPosition, 6>) -> FieldPosition;
    
    fn prompt_user_to_select_benched_monster_to_switch_in(&self, battle: &mut BattleState, switch_position: FieldPosition, switchable_benched_monster_ids: MaxSizedVec<MonsterID, 5>) -> MonsterID;
}


/// An action choice before certain details can be established, most often the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartiallySpecifiedActionChoice {
    Move{ 
        move_id: MoveID, 
        /**
        Only the positions with Monsters in them (at the time of calculation) are included. 
        So there is no chance of targetting an empty position (A position can be empty if one 
        of the teams has less Monsters than the total required battlers per side for the format).
        */
        possible_target_positions: MaxSizedVec<FieldPosition, 6>,
        activation_order: ActivationOrder, 
    },
    /// A switch out action before we know which monster to switch with.
    SwitchOut { 
        active_monster_id: MonsterID, 
        switchable_benched_monster_ids: MaxSizedVec<MonsterID, 5>,
        activation_order: ActivationOrder, 
    },
    CancelSimulation
}

/// An action whose details have been fully specified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullySpecifiedActionChoice {
    Move { 
        move_id: MoveID,
        /// There may be 1-6 valid targets for a move. 
        target_positions: MaxSizedVec<FieldPosition, 6>, 
        activation_order: ActivationOrder 
    },
    SwitchOut { 
        active_monster_id: MonsterID, 
        benched_monster_id: MonsterID, 
        activation_order: ActivationOrder 
    },
}
impl FullySpecifiedActionChoice {
    pub(crate) fn activation_order(&self) -> ActivationOrder {
        match *self {
            FullySpecifiedActionChoice::Move { activation_order, .. } => activation_order,
            FullySpecifiedActionChoice::SwitchOut { activation_order, .. } => activation_order,
        }
    }
    
    pub(crate) fn actor_id(&self) -> MonsterID {
        match *self {
            FullySpecifiedActionChoice::Move { move_id, .. } => move_id.owner_id,
            FullySpecifiedActionChoice::SwitchOut { active_monster_id, .. } => active_monster_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvailableChoices {
    choices: MaxSizedVec<PartiallySpecifiedActionChoice, 5>,
    switch_index: usize,
    iter_cursor: usize,
    // TODO: more actions will be added when they are added to the engine.
}

impl AvailableChoices {
    pub fn new(move_choices: Vec<PartiallySpecifiedActionChoice>, switch_out_choice: Option<PartiallySpecifiedActionChoice>) -> Self {
        let move_count = move_choices.len();
        let mut choices = MaxSizedVec::from_vec(move_choices);
        if let Some(switch_out) = switch_out_choice { choices.push(switch_out); };
        Self {
            choices,
            switch_index: move_count,
            iter_cursor: 0,
        }
    }
    
    pub fn move_choices(&self) -> impl Iterator<Item = &PartiallySpecifiedActionChoice> {
        self.choices[0..self.switch_index].iter().flatten()
    }

    pub fn switch_out_choice(&self) -> Option<&PartiallySpecifiedActionChoice> {
        self.choices.get(self.switch_index)
    }

    pub fn choices(&self) -> &MaxSizedVec<PartiallySpecifiedActionChoice, 5> {
        &self.choices
    }
    
    pub(crate) fn count(&self) -> usize {
        self.choices.count()
    }
}

impl Index<usize> for AvailableChoices {
    type Output = PartiallySpecifiedActionChoice;
    
    fn index(&self, index: usize) -> &Self::Output {
        &self.choices[index]
    }
}

impl IndexMut<usize> for AvailableChoices {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.choices[index]
    }
}
