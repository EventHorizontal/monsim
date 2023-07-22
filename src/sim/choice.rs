use std::ops::{IndexMut, Index, Range};

use crate::sim::utils::vector_to_array_of_options;

use super::game_mechanics::{BattlerUID, MoveUID};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionChoice {
    Move { move_uid: MoveUID, target_uid: BattlerUID },
    SwitchOut { active_battler_uid: BattlerUID, benched_battler_uid: Option<BattlerUID> },
}

impl ActionChoice {
    pub(crate) fn chooser(&self) -> BattlerUID {
        match self {
            ActionChoice::Move { move_uid, target_uid: _ } => move_uid.battler_uid,
            ActionChoice::SwitchOut { active_battler_uid, benched_battler_uid: _ } => *active_battler_uid,
        }
    }
    
    /// Panics if the `ActionChoice` is a `SwitchOut` with no chosen benched partner
    pub(crate) fn target(&self) -> BattlerUID {
        match self {
            ActionChoice::Move { move_uid: _, target_uid } => *target_uid,
            ActionChoice::SwitchOut { active_battler_uid: _, benched_battler_uid } => {
                benched_battler_uid.expect("No benched battler for SwitchOut")
            },
        }
    }
}

pub type ChosenActions = [EnumeratedActionChoice; 2];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvailableActions {
    pub ally_team_available_actions: TeamAvailableActions,
    pub opponent_team_available_actions: TeamAvailableActions,
}

pub type EnumeratedActionChoice = (usize, ActionChoice);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamAvailableActions {
    moves: [Option<EnumeratedActionChoice>; 4],
    switch_out: Option<EnumeratedActionChoice>,
    iter_cursor: usize,
    // TODO: more actions will be added when they are added to the engine.
}

impl TeamAvailableActions {
    pub fn new(moves_vec: Vec<EnumeratedActionChoice>, switch_out: Option<EnumeratedActionChoice>) -> Self {
        let moves = vector_to_array_of_options(moves_vec);
        Self {
            moves,
            switch_out,
            iter_cursor: 0,
        }
    }

    pub fn move_action_indices(&self) -> Range<usize> {
        let move_count = self.moves.iter().flatten().count();
        0..move_count
    }

    pub fn switch_out_action_index(&self) -> Option<usize> {
        self.switch_out.map(|it| { it.0 })
    }
}

impl Index<usize> for TeamAvailableActions {
    type Output = Option<EnumeratedActionChoice>;

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
    type Item = EnumeratedActionChoice;

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
 

