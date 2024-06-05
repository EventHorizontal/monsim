use monsim_utils::{Ally, MaxSizedVec, Opponent};
use std::{
    fmt::{Debug, Display, Formatter},
    ops::{Index, IndexMut},
};

use super::Monster;
use crate::{
    sim::{event_dispatcher::EventContext, targetting::BoardPosition, MonsterNumber},
    Broadcaster, EventHandler, EventHandlerSet, OwnedEventHandler,
};

const MAX_BATTLERS_PER_TEAM: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterTeam {
    pub id: TeamID,
    monsters: MaxSizedVec<Monster, 6>,
}

impl Index<MonsterNumber> for MonsterTeam {
    type Output = Monster;

    fn index(&self, index: MonsterNumber) -> &Self::Output {
        &self.monsters[index as usize]
    }
}

impl IndexMut<MonsterNumber> for MonsterTeam {
    fn index_mut(&mut self, index: MonsterNumber) -> &mut Self::Output {
        &mut self.monsters[index as usize]
    }
}

impl MonsterTeam {
    pub fn new(monsters: Vec<Monster>, id: TeamID) -> Self {
        assert!(monsters.first().is_some(), "There is not a single monster in the team.");
        assert!(monsters.len() <= MAX_BATTLERS_PER_TEAM);
        let monsters = MaxSizedVec::from_vec(monsters);
        MonsterTeam { id, monsters }
    }

    pub fn active_monsters(&self) -> Vec<&Monster> {
        self.monsters()
            .filter(|monster| if let BoardPosition::Field(_) = monster.board_position { true } else { false })
            .collect()
    }

    pub fn monsters(&self) -> impl Iterator<Item = &Monster> {
        self.monsters.iter()
    }

    pub fn monsters_mut(&mut self) -> impl Iterator<Item = &mut Monster> {
        self.monsters.iter_mut()
    }

    pub fn owned_event_handlers<R: Copy, C: EventContext + Copy, B: Broadcaster + Copy>(
        &self,
        event_handler_selector: fn(EventHandlerSet) -> Vec<Option<EventHandler<R, C, B>>>,
    ) -> Vec<OwnedEventHandler<R, C, B>> {
        let mut out = Vec::new();
        for monster in self.monsters.iter() {
            out.append(&mut monster.owned_event_handlers(event_handler_selector))
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

    pub(crate) fn iter_with_team_id(self) -> impl Iterator<Item = (TeamID, T)> {
        let (ally_team_item, opponent_team_item) = self.unwrap_full();
        [(TeamID::Allies, ally_team_item), (TeamID::Opponents, opponent_team_item)].into_iter()
    }

    /// Consumes `self`
    fn unwrap_full(self) -> (T, T) {
        (self.ally_team_item.unwrap(), self.opponent_team_item.unwrap())
    }
}

impl<T: Clone> PerTeam<Option<T>> {
    pub fn to_option_pair(self) -> (Option<Ally<T>>, Option<Opponent<T>>) {
        let (ally_team_item, opponent_team_item) = self.unwrap();
        let (ally_team_item, opponent_team_item) = ((*ally_team_item).clone(), (*opponent_team_item).clone());
        (ally_team_item.map(|item| Ally::new(item)), opponent_team_item.map(|item| Opponent::new(item)))
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
