use std::ops::{IndexMut, Index, Range};

use monsim_utils::{Ally, ArrayOfOptionals, Opponent};

use crate::{sim::utils::slice_to_array_of_options, ActivationOrder};

use super::{game_mechanics::{MonsterUID, MoveUID}, TeamUID};


/// An action choice before certain details can be established, most often the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartiallySpecifiedChoice {
    /// TODO: This *should* be a move before targets are known, but since the targetting system is still unimplemented, for now we assume the one opponent monster is the target. 
    Move{ move_uid: MoveUID, target_uid: MonsterUID, activation_order: ActivationOrder, display_text: &'static str},
    /// A switch out action before we know which monster to switch with.
    SwitchOut { active_monster_uid: MonsterUID, switchable_benched_monster_uids: ArrayOfOptionals<MonsterUID, 5>, activation_order: ActivationOrder, display_text: &'static str },
}

/// An action whose details have been fully specified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullySpecifiedChoice {
    Move { move_uid: MoveUID, target_uid: MonsterUID, activation_order: ActivationOrder },
    SwitchOut { active_monster_uid: MonsterUID, benched_monster_uid: MonsterUID, activation_order: ActivationOrder },
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
    // All the Some variants should be in the beginning.
    moves: [Option<PartiallySpecifiedChoice>; 4],
    switch_out: Option<PartiallySpecifiedChoice>,
    iter_cursor: usize,
    // TODO: more actions will be added when they are added to the engine.
}

impl AvailableChoicesForTeam {
    pub fn new(moves_vec: &[PartiallySpecifiedChoice], switch_out: Option<PartiallySpecifiedChoice>) -> Self {
        let moves = slice_to_array_of_options(moves_vec);
        Self {
            moves,
            switch_out,
            iter_cursor: 0,
        }
    }
    
    pub fn move_choice_indices(&self) -> Range<usize> {
        let move_count = self.moves.iter().flatten().count();
        0..move_count
    }

    pub fn switch_out_choice(&self) -> Option<PartiallySpecifiedChoice> {
        self.switch_out
    }

    pub fn switch_out_choice_index(&self) -> Option<usize> {
        let move_count = self.moves.iter().flatten().count();
        self.switch_out.map(|_| move_count )
    }

    pub(crate) fn as_vec(&self) -> Vec<PartiallySpecifiedChoice> {
        [
            self.moves[0],
            self.moves[1],
            self.moves[2],
            self.moves[3],
            self.switch_out,
        ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    }

    /// panicks if there is no `PartiallySpecifiedAction` at the given index.
    pub(crate) fn get_by_index(&self, index: usize) -> PartiallySpecifiedChoice {
        let move_count = self.moves.iter().flatten().count();
        if index < move_count {
            self.moves[index].unwrap()
        } else if index == move_count && self.switch_out.is_some() {
            self.switch_out.unwrap()
        } else {
            panic!("Index out of bounds for AvailableActionsForTeam.")
        }
    }
    
    pub(crate) fn count(&self) -> usize {
        let mut count = 0;
        for index in 0..4 {
            if self.moves[index].is_some() { count += 1; };
        }
        if self.switch_out.is_some() { count += 1; }
        count
    }
}

impl Index<usize> for AvailableChoicesForTeam {
    type Output = Option<PartiallySpecifiedChoice>;
    
    fn index(&self, index: usize) -> &Self::Output {
        let move_count = self.moves.iter().flatten().count();
        if index < move_count {
            &self.moves[index]
        } else if index == move_count && self.switch_out.is_some() {
            &self.switch_out
        } else {
            unreachable!()
        }
    }
}

impl IndexMut<usize> for AvailableChoicesForTeam {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let move_count = self.moves.iter().flatten().count();
        if index < move_count {
            &mut self.moves[index]
        } else if index == move_count && self.switch_out.is_some() {
            &mut self.switch_out
        } else {
            unreachable!()
        }
    }
}
