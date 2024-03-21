use std::ops::Range;

use monsim_utils::MaxSizedVec;

use super::{MonsterRef, MoveRef};


/// An action choice before certain details can be established, most often the target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartiallySpecifiedChoice<'a> {
    /// TODO: This *should* be a move before targets are known, but since the targetting system is still unimplemented, for now we assume the one opponent monster is the target. 
    Move{ attacker: MonsterRef<'a>, move_: MoveRef<'a>, target: MonsterRef<'a>, display_text: &'static str},
    /// A switch out action before we know which monster to switch with.
    SwitchOut { active_monster: MonsterRef<'a>, switchable_benched_monsters: Vec<MonsterRef<'a>>, display_text: &'static str },
}

/// An action whose details have been fully specified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullySpecifiedChoice<'a> {
    Move { attacker: MonsterRef<'a>, move_: MoveRef<'a>, target: MonsterRef<'a> },
    SwitchOut { active_monster: MonsterRef<'a>, benched_monster: MonsterRef<'a> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableChoicesForTeam<'a> {
    // All the Some variants should be in the beginning.
    moves: MaxSizedVec<PartiallySpecifiedChoice<'a>, 4>,
    switch_out: Option<PartiallySpecifiedChoice<'a>>,
    iter_cursor: usize,
    // TODO: more actions will be added when they are added to the engine.
}

impl<'a> AvailableChoicesForTeam<'a> {
    pub fn new(
        moves_vec: Vec<PartiallySpecifiedChoice<'a>>, 
        switch_out: Option<PartiallySpecifiedChoice<'a>>, 
    ) -> Self {
        let moves = MaxSizedVec::from_vec(moves_vec);
        Self {
            moves,
            switch_out,
            iter_cursor: 0,
        }
    }

    fn move_choice_count(&self) -> usize {
        self.moves.iter().count()
    }
    
    pub fn move_choice_indices(&self) -> Range<usize> {
        0..self.move_choice_count()
    }

    pub fn switch_out_choice(&'a self) -> Option<PartiallySpecifiedChoice> {
        self.switch_out.clone()
    }

    pub fn switch_out_choice_index(&self) -> Option<usize> {
        self.switch_out.as_ref().map(|_| self.move_choice_count() )
    }

    pub(crate) fn as_vec(&'a self) -> Vec<PartiallySpecifiedChoice> {
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
    pub(crate) fn get_by_index(&'a self, index: usize) -> PartiallySpecifiedChoice {
        if index < self.move_choice_count() {
            self.moves[index].clone()
        } else if index == self.move_choice_count() && self.switch_out.is_some() {
            self.switch_out.clone().unwrap()
        } else {
            panic!("Index out of bounds for AvailableActionsForTeam.")
        }
    }
    
    pub(crate) fn count(&self) -> usize {
        let mut count = self.moves.len();
        if self.switch_out.is_some() { count += 1; }
        count
    }
}
