use std::ops::{Index, Range};

use monsim_utils::{Ally, FLArray, Opponent};

use super::{game_mechanics::{MonsterUID, MoveUID}, TeamUID};


/// An action choice before certain details can be established, most often the target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartiallySpecifiedChoice {
    /// TODO: This *should* be a move before targets are known, but since the targetting system is still unimplemented, for now we assume the one opponent monster is the target. 
    Move{ move_uid: MoveUID, target_uid: MonsterUID, display_text: &'static str},
    /// A switch out action before we know which monster to switch with.
    SwitchOut { switcher_uid: MonsterUID, candidate_switchee_uids: Vec<MonsterUID>, display_text: &'static str },
}

impl Default for PartiallySpecifiedChoice {
    fn default() -> Self {
        PartiallySpecifiedChoice::Move { move_uid: MoveUID::default(), target_uid: MonsterUID::default(), display_text: "" }
    }
}

/// An action whose details have been fully specified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullySpecifiedChoice {
    Move { move_uid: MoveUID, target_uid: MonsterUID },
    SwitchOut { switcher_uid: MonsterUID, candidate_switchee_uids: MonsterUID },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableChoices {
    pub ally_team_available_choices: AvailableChoicesForTeam,
    pub opponent_team_available_choices: AvailableChoicesForTeam,
}

impl Index<TeamUID> for AvailableChoices {
    type Output = AvailableChoicesForTeam;

    fn index(&self, index: TeamUID) -> &Self::Output {
        match index {
            TeamUID::Allies => &self.ally_team_available_choices,
            TeamUID::Opponents => &self.opponent_team_available_choices,
        }
    }
}

impl AvailableChoices {
    pub(crate) fn unwrap(self) -> (Ally<AvailableChoicesForTeam>, Opponent<AvailableChoicesForTeam>) {
        (Ally(self.ally_team_available_choices), Opponent(self.opponent_team_available_choices))
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableChoicesForTeam {
    // All the Some variants should be in the beginning.
    moves: FLArray<PartiallySpecifiedChoice, 4>,
    switch_out: Option<PartiallySpecifiedChoice>,
    iter_cursor: usize,
    // TODO: more actions will be added when they are added to the engine.
}

impl AvailableChoicesForTeam {
    pub fn new(moves_vec: &[PartiallySpecifiedChoice], switch_out: Option<PartiallySpecifiedChoice>) -> Self {
        let moves = FLArray::with_default_padding(&moves_vec);
        Self {
            moves,
            switch_out,
            iter_cursor: 0,
        }
    }

    fn move_count(&self) -> usize {
        self.moves.iter().count()
    }
    
    pub fn move_choice_indices(&self) -> Range<usize> {
        0..self.move_count()
    }

    pub fn switch_out_choice(&self) -> Option<PartiallySpecifiedChoice> {
        self.switch_out.clone()
    }

    pub fn switch_out_choice_index(&self) -> Option<usize> {
        self.switch_out.as_ref().map(|_| self.move_count() )
    }

    pub(crate) fn as_vec(&self) -> Vec<PartiallySpecifiedChoice> {
        [
            Some(self.moves[0].clone()),
            Some(self.moves[1].clone()),
            Some(self.moves[2].clone()),
            Some(self.moves[3].clone()),
            self.switch_out.clone(),
        ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    }

    /// panicks if there is no `PartiallySpecifiedAction` at the given index.
    pub(crate) fn get_by_index(&self, index: usize) -> PartiallySpecifiedChoice {
        if index < self.move_count() {
            self.moves[index].clone()
        } else if index == self.move_count() && self.switch_out.is_some() {
            self.switch_out.clone().unwrap()
        } else {
            panic!("Index out of bounds for AvailableActionsForTeam.")
        }
    }
    
    pub(crate) fn count(&self) -> usize {
        let mut count = self.moves.valid_elements();
        if self.switch_out.is_some() { count += 1; }
        count
    }
}

// impl Index<usize> for AvailableChoicesForTeam {
//     type Output = PartiallySpecifiedChoice;
    
//     fn index(&self, index: usize) -> &Self::Output {
//         let move_count = self.moves.into_iter().count();
//         if index < move_count {
//             &self.moves[index]
//         } else if index == move_count && self.switch_out.is_some() {
//             &self.switch_out
//         } else {
//             unreachable!()
//         }
//     }
// }

// impl IndexMut<usize> for AvailableChoicesForTeam {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         let move_count = self.moves.into_iter().count();
//         if index < move_count {
//             &mut self.moves[index]
//         } else if index == move_count && self.switch_out.is_some() {
//             &mut self.switch_out
//         } else {
//             unreachable!()
//         }
//     }
// }
