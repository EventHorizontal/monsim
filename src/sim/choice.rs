use std::ops::{IndexMut, Index};

use monsim_utils::MaxSizedVec;

use crate::ActivationOrder;

use super::game_mechanics::{MonsterID, MoveID};


/// An action choice before certain details can be established, most often the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartiallySpecifiedChoice {
    /// TODO: This *should* be a move before targets are known, but since the targetting system is still unimplemented, for now we assume the one opponent monster is the target. 
    Move{ 
        move_user_id: MonsterID, 
        move_id: MoveID, 
        target_id: MonsterID,
        activation_order: ActivationOrder, 
        display_text: &'static str
    },
    /// A switch out action before we know which monster to switch with.
    SwitchOut { 
        active_monster_id: MonsterID, 
        switchable_benched_monster_ids: MaxSizedVec<MonsterID, 5>,
        activation_order: ActivationOrder, 
        display_text: &'static str 
    },
}

/// An action whose details have been fully specified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullySpecifiedChoice {
    Move { move_user_id: MonsterID, move_id: MoveID, target_id: MonsterID, activation_order: ActivationOrder },
    SwitchOut { active_monster_id: MonsterID, benched_monster_id: MonsterID, activation_order: ActivationOrder },
}
impl FullySpecifiedChoice {
    pub(crate) fn activation_order(&self) -> ActivationOrder {
        match self {
            FullySpecifiedChoice::Move { activation_order, .. } => *activation_order,
            FullySpecifiedChoice::SwitchOut { activation_order, .. } => *activation_order,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvailableChoicesForTeam {
    choices: MaxSizedVec<PartiallySpecifiedChoice, 5>,
    switch_index: usize,
    iter_cursor: usize,
    // TODO: more actions will be added when they are added to the engine.
}

impl AvailableChoicesForTeam {
    pub fn new(move_choices: Vec<PartiallySpecifiedChoice>, switch_out_choice: Option<PartiallySpecifiedChoice>) -> Self {
        let move_count = move_choices.len();
        let mut choices = MaxSizedVec::from_vec(move_choices);
        if let Some(switch_out) = switch_out_choice { choices.push(switch_out); };
        Self {
            choices,
            switch_index: move_count,
            iter_cursor: 0,
        }
    }
    
    pub fn move_choices(&self) -> &[PartiallySpecifiedChoice] {
        &self.choices[0..self.switch_index]
    }

    pub fn switch_out_choice(&self) -> Option<&PartiallySpecifiedChoice> {
        self.choices.get(self.switch_index)
    }

    pub fn choices(&self) -> &MaxSizedVec<PartiallySpecifiedChoice, 5> {
        &self.choices
    }
    
    pub(crate) fn count(&self) -> usize {
        self.choices.count()
    }
}

impl Index<usize> for AvailableChoicesForTeam {
    type Output = PartiallySpecifiedChoice;
    
    fn index(&self, index: usize) -> &Self::Output {
        &self.choices[index]
    }
}

impl IndexMut<usize> for AvailableChoicesForTeam {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.choices[index]
    }
}
