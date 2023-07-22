use std::ops::{IndexMut, Index};

use crate::sim::utils::vector_to_array_of_options;

use super::game_mechanics::{BattlerUID, MoveUID};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionChoice {
    Move { move_uid: MoveUID, target_uid: BattlerUID },
    SwitchOut { active_battler_uid: BattlerUID, benched_battler_uid: BattlerUID },
}

impl ActionChoice {
    pub(crate) fn chooser(&self) -> BattlerUID {
        match self {
            ActionChoice::Move { move_uid, target_uid: _ } => move_uid.battler_uid,
            ActionChoice::SwitchOut { active_battler_uid, benched_battler_uid: _ } => *active_battler_uid,
        }
    }
    
    pub(crate) fn target(&self) -> BattlerUID {
        match self {
            ActionChoice::Move { move_uid: _, target_uid } => *target_uid,
            ActionChoice::SwitchOut { active_battler_uid: _, benched_battler_uid } => *benched_battler_uid,
        }
    }
}

pub type ChosenActions = [ActionChoice; 2];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvailableActions {
    pub ally_team_available_actions: TeamAvailableActions,
    pub opponent_team_available_actions: TeamAvailableActions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamAvailableActions {
    moves: [Option<ActionChoice>; 4],
    switch_out: Option<ActionChoice>,
    iter_cursor: usize,
    // TODO: more actions will be added when they are added to the engine.
}

impl TeamAvailableActions {
    pub fn new(moves_vec: Vec<ActionChoice>, switch_out: Option<ActionChoice>) -> Self {
        let moves = vector_to_array_of_options(moves_vec);
        Self {
            moves,
            switch_out,
            iter_cursor: 0,
        }
    }
}

impl Index<usize> for TeamAvailableActions {
    type Output = Option<ActionChoice>;

    fn index(&self, index: usize) -> &Self::Output {
        let move_count = self.moves.iter().flatten().count(); // we keep the Some variants at the beginning so we should get the Length of the array.
        if index < move_count {
            &self.moves[index]
        } else if index == move_count && self.switch_out.is_some() {
            &self.switch_out
        } else {
            unreachable!()
        }
    }
}

impl IndexMut<usize> for TeamAvailableActions {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let move_count = self.moves.iter().flatten().count(); // we keep the Some variants at the beginning so we should get the Length of the array.
        if index < move_count {
            &mut self.moves[index]
        } else if index == move_count && self.switch_out.is_some() {
            &mut self.switch_out
        } else {
            unreachable!()
        }
    }
}

impl Iterator for TeamAvailableActions {
    type Item = ActionChoice;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.iter_cursor;
        let move_count = self.moves.iter().flatten().count(); // we keep the Some variants at the beginning so we should get the Length of the array.
        if index < move_count {
            self.iter_cursor += 1;
            Some(self.moves[index].expect("validated index"))
        } else if index == move_count && self.switch_out.is_some() {
            self.iter_cursor += 1;
            Some(self.switch_out.expect("validated index"))
        } else {
            self.iter_cursor = 0;
            None
        }
    }
}
 

