use std::{fmt::{Debug, Display, Formatter}, ops::{Index, IndexMut}};
use monsim_utils::{Ally, MaxSizedVec, Opponent};

use crate::sim::{event::OwnedEventHandlerDeck, MonsterNumber};
use super::{Monster, MonsterUID, MoveNumber};

const MAX_BATTLERS_PER_TEAM: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterTeam {
    pub id: TeamUID,
    pub active_monster_uid: MonsterUID,
    monsters: MaxSizedVec<Monster, 6>,
}

impl Index<MonsterNumber> for MonsterTeam {
    type Output = Monster;

    fn index(&self, index: MonsterNumber) -> &Self::Output {
        &self.monsters()[index as usize]
    }
}

impl IndexMut<MonsterNumber> for MonsterTeam {
    fn index_mut(&mut self, index: MonsterNumber) -> &mut Self::Output {
        &mut self.monsters_mut()[index as usize]
    }
}

impl MonsterTeam {
    pub fn new(monsters: Vec<Monster>, id: TeamUID) -> Self {
        assert!(monsters.first().is_some(), "There is not a single monster in the team.");
        assert!(monsters.len() <= MAX_BATTLERS_PER_TEAM);
        let monsters = MaxSizedVec::from_vec(monsters);
        MonsterTeam {
            id,
            active_monster_uid: MonsterUID { team_uid: id, monster_number: MonsterNumber::_1}, 
            monsters 
        }
    }

    pub fn monsters(&self) -> &MaxSizedVec<Monster, 6> {
        &self.monsters
    }

    pub fn monsters_mut(&mut self) -> &mut MaxSizedVec<Monster, 6> {
        &mut self.monsters
    }

    pub fn event_handler_deck_instances(&self) -> Vec<OwnedEventHandlerDeck> {
        let mut out = Vec::new();
        for monster in self.monsters.iter() {
            out.append(&mut monster.event_handler_deck_instances())
        }
        out
    }

    pub(crate) fn team_status_string(&self) -> String {
        let mut out = String::new();
        for monster in self.monsters().iter() {
            out.push_str(&monster.status_string());
        }
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TeamUID {
    Allies,
    Opponents,
}

impl TeamUID {
    pub fn other(&self) -> TeamUID {
        match self {
            TeamUID::Allies => TeamUID::Opponents,
            TeamUID::Opponents => TeamUID::Allies,
        }
    }
}

/// A container for storing an object of type `T` for each team.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PerTeam<T> {
    ally_team_item: Ally<T>,
    opponent_team_item: Opponent<T>,
}

impl<T> PerTeam<T> {
    pub fn new(ally_team_item: Ally<T>, opponent_team_item: Opponent<T>) -> Self {
        Self {
            ally_team_item,
            opponent_team_item,
        }
    }

    pub fn ally_ref(&self) -> Ally<&T> {
        Ally(&self.ally_team_item)
    }

    pub fn ally_mut(&mut self) -> Ally<&mut T> {
        Ally(&mut self.ally_team_item)
    }

    pub fn opponent_ref(&self) -> Opponent<&T> {
        Opponent(&self.opponent_team_item)
    }

    pub fn opponent_mut(&mut self) -> Opponent<&mut T> {
        Opponent(&mut self.opponent_team_item)
    }

    pub fn map_consume<U, F>(self, f: F) -> PerTeam<U>
    where
        F: Fn(T) -> U,
    {
        let (ally_team_item, opponent_team_item) = self.unwrap();
        PerTeam::new(ally_team_item.map_consume(&f), opponent_team_item.map_consume(f))
    }

    /// Consumes `self`
    pub fn unwrap(self) -> (Ally<T>, Opponent<T>) {
        (self.ally_team_item, self.opponent_team_item)
    }

    /// Returns a reference to internals.
    pub fn unwrap_ref(&self) -> (&Ally<T>, &Opponent<T>) {
        (&self.ally_team_item, &self.opponent_team_item)
    }

    pub fn unwrap_mut(&mut self) -> (&mut Ally<T>, &mut Opponent<T>) {
        (&mut self.ally_team_item, &mut self.opponent_team_item)
    }
}

impl<T: Clone> PerTeam<Option<T>> {
    pub fn to_option_pair(self) -> (Option<Ally<T>>, Option<Opponent<T>>) {
        let (ally_team_item, opponent_team_item) = self.unwrap();
        let (ally_team_item, opponent_team_item) = ((*ally_team_item).clone(), (*opponent_team_item).clone());
        (ally_team_item.map(|item| { Ally(item) }), opponent_team_item.map(|item| { Opponent(item) }))
    }
}

impl<T: Clone> PerTeam<T> {
    pub(crate) fn both(item: T) -> Self {
        Self {
            ally_team_item: Ally(item.clone()),
            opponent_team_item: Opponent(item),
        }
    }

    /// Returns a copy of the items
    pub fn as_array(&self) -> [T; 2] {
        [(*self.ally_team_item).clone(), (*self.opponent_team_item).clone()]
    }

    pub fn map_clone<U, F>(&self, f: F) -> PerTeam<U>
    where
        F: Fn(T) -> U,
    {
        let (ally_team_item, opponent_team_item) = self.unwrap_ref();
        PerTeam::new(ally_team_item.map_clone(&f), opponent_team_item.map_clone(f))
    }
}

impl<T> Index<TeamUID> for PerTeam<T> {
    type Output = T;

    fn index(&self, index: TeamUID) -> &Self::Output {
        match index {
            TeamUID::Allies => &self.ally_team_item,
            TeamUID::Opponents => &self.opponent_team_item,
        }
    }
} 

impl<T> IndexMut<TeamUID> for PerTeam<T> {
    fn index_mut(&mut self, index: TeamUID) -> &mut Self::Output {
        match index {
            TeamUID::Allies => &mut self.ally_team_item,
            TeamUID::Opponents => &mut self.opponent_team_item,
        }
    }
}

impl<T: Clone> IntoIterator for PerTeam<T> {
    type Item = T;

    type IntoIter = std::array::IntoIter<T, 2>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_array().into_iter()
    }
}

impl Display for TeamUID {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MoveUID {
    pub owner_uid: MonsterUID,
    pub move_number: MoveNumber,
}