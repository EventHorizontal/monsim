use std::ops::{IndexMut, Index, Range};

use monsim_utils::ArrayOfOptionals;

use crate::sim::utils::slice_to_array_of_options;

use super::{game_mechanics::{MonsterUID, MoveUID}, PerTeam, TeamID};


/// An action choice before certain details can be established, most often the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartiallySpecifiedAction {
    /// This *should* be a move before targets are known, but since the targetting system is still unimplemented, for now we assume the one opponent monster is the target. 
    Move{ move_uid: MoveUID, target_uid: MonsterUID, display_text: &'static str},
    /// A switch out action before we know which monster to switch with.
    SwitchOut { switcher_uid: MonsterUID, possible_switchee_uids: ArrayOfOptionals<MonsterUID, 5> },
}

/// An action whose details have been fully specified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullySpecifiedAction {
    Move { move_uid: MoveUID, target_uid: MonsterUID },
    SwitchOut { switcher_uid: MonsterUID, switchee_uid: MonsterUID },
}

pub type ChosenActionsForTurn = PerTeam<FullySpecifiedAction>;

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
    // All the Some variants should be in the beginning.
    moves: [Option<PartiallySpecifiedAction>; 4],
    switch_out: Option<PartiallySpecifiedAction>,
    iter_cursor: usize,
    // TODO: more actions will be added when they are added to the engine.
}

impl AvailableActionsForTeam {
    pub fn new(moves_vec: &[PartiallySpecifiedAction], switch_out: Option<PartiallySpecifiedAction>) -> Self {
        let moves = slice_to_array_of_options(moves_vec);
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

    pub fn switch_out_action(&self) -> Option<PartiallySpecifiedAction> {
        self.switch_out
    }

    pub(crate) fn as_vec(&self) -> Vec<PartiallySpecifiedAction> {
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
    pub(crate) fn get_by_index(&self, index: usize) -> PartiallySpecifiedAction {
        let move_count = self.moves.iter().flatten().count();
        if index < move_count {
            self.moves[index].unwrap()
        } else if index == move_count && self.switch_out.is_some() {
            self.switch_out.unwrap()
        } else {
            panic!("Index out of bounds for AvailableActionsForTeam.")
        }
    }
}

impl Index<usize> for AvailableActionsForTeam {
    type Output = Option<PartiallySpecifiedAction>;
    
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

impl IndexMut<usize> for AvailableActionsForTeam {
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
