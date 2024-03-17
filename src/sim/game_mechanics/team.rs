use std::{cell::Cell, fmt::{Debug, Display, Formatter}, ops::{Index, IndexMut}};
use monsim_utils::{not, Ally, FLArray, Opponent};

use crate::sim::{event::OwnedEventHandlerDeck, MonsterNumber};
use super::{Monster, MonsterUID, MoveNumber};

const MAX_BATTLERS_PER_TEAM: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterTeam {
    pub id: TeamUID,
    pub active_monster_uid: MonsterUID,
    monsters: FLArray<Cell<Monster>, 6>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TeamUID {
    #[default]
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

    pub fn map<U: Clone, F>(self, f: F) -> PerTeam<U>
    where
        F: Fn(T) -> U,
    {
        PerTeam::new(self.ally_team_item.map(&f), self.opponent_team_item.map(f))
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

impl MonsterTeam {
    pub fn new(monsters: &[Monster], id: TeamUID) -> Self {
        assert!(not!(monsters.is_empty()), "Expected at least 1 monster but none were given.");
        assert!(monsters.len() <= MAX_BATTLERS_PER_TEAM, "Expected at most 6 monsters but {m} were given.", m = monsters.len());
        let monsters = FLArray::with_default_padding(monsters).map(|monster| { Cell::new(monster) });
        MonsterTeam {
            id,
            active_monster_uid: MonsterUID { team_uid: id, monster_number: MonsterNumber::_1}, 
            monsters 
        }
    }

    pub fn monsters(&self) -> &FLArray<Cell<Monster>, 6> {
        &self.monsters
    }

    // pub fn monsters_mut(&mut self) -> &mut FLArray<Cell<Monster>, 6> {
    //     &mut self.monsters
    // }

    pub fn event_handler_deck_instances(&self) -> Vec<OwnedEventHandlerDeck> {
        let mut out = Vec::new();
        for monster in self.monsters.clone().into_iter() {
            out.append(&mut monster.get().event_handler_deck_instances())
        }
        out
    }

    pub(crate) fn team_status_string(&self) -> String {
        let mut out = String::new();
        for monster in self.monsters().clone().into_iter() {
            out.push_str(&monster.get().status_string());
        }
        out
    }
}