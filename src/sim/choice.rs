use std::ops::{IndexMut, Index, Range};

use crate::sim::utils::vector_to_array_of_options;

use super::{game_mechanics::{BattlerUID, MoveUID}, Move, TeamID};


/// An action choice before certain details can be established, most often the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartialActionChoice {
    /// This *should* be a move before targets are known, but since the targetting system is still unimplemented, for now we assume the one opponent battler is the target. 
    Move{ move_uid: MoveUID, target_uid: BattlerUID, display_text: &'static str},
    /// A switch out action before we know which battler to switch with.
    SwitchOut { switcher_uid: BattlerUID },
}

/// An action whose details have been fully specified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionChoice {
    Move { move_uid: MoveUID, target_uid: BattlerUID },
    SwitchOut { switcher_uid: BattlerUID, switchee_uid: BattlerUID },
}

pub type ChosenActionsForTurn = [ActionChoice; 2];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvailableActions {
    pub ally_team_available_actions: AvailableActionsForTeam,
    pub opponent_team_available_actions: AvailableActionsForTeam,
}

impl Index<TeamID> for AvailableActions {
    type Output = AvailableActionsForTeam;

    fn index(&self, index: TeamID) -> &Self::Output {
        match index {
            TeamID::Allies => &self.ally_team_available_actions,
            TeamID::Opponents => &self.opponent_team_available_actions,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AvailableActionsForTeam {
    moves: [Option<PartialActionChoice>; 4],
    switch_out: Option<PartialActionChoice>,
    iter_cursor: usize,
    // TODO: more actions will be added when they are added to the engine.
}

impl AvailableActionsForTeam {
    pub fn new(moves_vec: Vec<PartialActionChoice>, switch_out: Option<PartialActionChoice>) -> Self {
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
        let move_count = self.moves.iter().flatten().count();
        self.switch_out.map(|_| { move_count })
    }
}

impl Index<usize> for AvailableActionsForTeam {
    type Output = Option<PartialActionChoice>;
    
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

impl IndexMut<usize> for AvailableActionsForTeam {
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

impl Iterator for AvailableActionsForTeam {
    type Item = PartialActionChoice;

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
 

