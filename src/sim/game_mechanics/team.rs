use std::{fmt::{Debug, Display, Formatter}, ops::{Index, IndexMut}};
use monsim_utils::{Ally, MaxSizedVec, Opponent};

use crate::{sim::MonsterNumber, Event, OwnedEventHandler};
use super::{Monster, MonsterID};

const MAX_BATTLERS_PER_TEAM: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterTeam {
    pub id: TeamID,
    pub active_monster_id: MonsterID,
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
    pub fn new(monsters: Vec<Monster>, id: TeamID) -> Self {
        assert!(monsters.first().is_some(), "There is not a single monster in the team.");
        assert!(monsters.len() <= MAX_BATTLERS_PER_TEAM);
        let monsters = MaxSizedVec::from_vec(monsters);
        MonsterTeam {
            id,
            active_monster_id: MonsterID { team_id: id, monster_number: MonsterNumber::_1}, 
            monsters 
        }
    }

    pub fn monsters(&self) -> &MaxSizedVec<Monster, 6> {
        &self.monsters
    }

    pub fn monsters_mut(&mut self) -> &mut MaxSizedVec<Monster, 6> {
        &mut self.monsters
    }

    pub fn event_handlers_for<E: Event>(&self, event: E) -> Vec<OwnedEventHandler<E>> {
        let mut out = Vec::new();
        for monster in self.monsters.iter() {
            out.append(&mut monster.event_handlers_for(event))
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TeamID {
    #[default]
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

impl Display for TeamID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamID::Allies => write!(f, "Ally Team"),
            TeamID::Opponents => write!(f, "Opponent Team"),
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
        Ally::new(&self.ally_team_item)
    }

    pub fn ally_mut(&mut self) -> Ally<&mut T> {
        Ally::new(&mut self.ally_team_item)
    }

    pub fn opponent_ref(&self) -> Opponent<&T> {
        Opponent::new(&self.opponent_team_item)
    }

    pub fn opponent_mut(&mut self) -> Opponent<&mut T> {
        Opponent::new(&mut self.opponent_team_item)
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
        (ally_team_item.map(|item| { Ally::new(item) }), opponent_team_item.map(|item| { Opponent::new(item) }))
    }
}

impl<T: Clone> PerTeam<T> {
    pub(crate) fn _both(item: T) -> Self {
        Self {
            ally_team_item: Ally::new(item.clone()),
            opponent_team_item: Opponent::new(item),
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