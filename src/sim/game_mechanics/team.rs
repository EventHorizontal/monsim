use std::{fmt::{Debug, Display, Formatter}, ops::{Index, IndexMut}};
use max_size_vec::MaxSizeVec;
use monsim_utils::{Ally, Opponent};

use crate::sim::{event::OwnedEventHandlerDeck, MonsterNumber};
use super::{Monster, MonsterUID, MoveNumber};

const MAX_BATTLERS_PER_TEAM: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterTeam {
    pub id: TeamID,
    pub active_monster_uid: MonsterUID,
    monsters: MaxSizeVec<Monster, 6>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TeamID {
    Allies,
    Opponents,
}

impl TeamID {
    pub fn other(&self) -> TeamID {
        match self {
            TeamID::Allies => TeamID::Opponents,
            TeamID::Opponents => TeamID::Allies,
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

    pub fn ally(&self) -> &Ally<T> {
        &self.ally_team_item
    }

    pub fn ally_mut(&mut self) -> &mut Ally<T> {
        &mut self.ally_team_item
    }

    pub fn opponent(&self) -> &Opponent<T> {
        &self.opponent_team_item
    }

    pub fn opponent_mut(&mut self) -> &mut Opponent<T> {
        &mut self.opponent_team_item
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
    pub fn as_pair_of_options(self) -> (Option<Ally<T>>, Option<Opponent<T>>) {
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

    pub fn map<U: Clone, F>(self, f: F) -> PerTeam<U>
    where
        F: Fn(T) -> U,
    {
        PerTeam::new(self.ally_team_item.map(&f), self.opponent_team_item.map(f))
    }
}

impl<T> Index<TeamID> for PerTeam<T> {
    type Output = T;

    fn index(&self, index: TeamID) -> &Self::Output {
        match index {
            TeamID::Allies => &self.ally_team_item,
            TeamID::Opponents => &self.opponent_team_item,
        }
    }
} 

impl<T> IndexMut<TeamID> for PerTeam<T> {
    fn index_mut(&mut self, index: TeamID) -> &mut Self::Output {
        match index {
            TeamID::Allies => &mut self.ally_team_item,
            TeamID::Opponents => &mut self.opponent_team_item,
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

impl Display for TeamID {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MoveUID {
    pub owner_uid: MonsterUID,
    pub move_number: MoveNumber,
}

impl MonsterTeam {
    pub fn new(monsters: Vec<Monster>, id: TeamID) -> Self {
        assert!(monsters.first().is_some(), "There is not a single monster in the team.");
        assert!(monsters.len() <= MAX_BATTLERS_PER_TEAM);
        let monsters_iter = monsters.into_iter();
        let mut monsters = MaxSizeVec::new();
        monsters_iter.for_each(|monster| {monsters.push(monster)});
        MonsterTeam {
            id,
            active_monster_uid: MonsterUID { team_id: id, monster_number: MonsterNumber::_1}, 
            monsters 
        }
    }

    pub fn monsters(&self) -> &MaxSizeVec<Monster, 6> {
        &self.monsters
    }

    pub fn monsters_mut(&mut self) -> &mut MaxSizeVec<Monster, 6> {
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
        for monster in self.monsters() {
            out.push_str(&monster.status_string());
        }
        out
    }
}