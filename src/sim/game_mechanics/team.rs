use std::{fmt::{Debug, Display, Formatter}, ops::{Index, IndexMut}};
use max_size_vec::MaxSizeVec;

use crate::sim::event::CompositeEventResponderInstanceList;
use super::{Monster, MonsterUID, MoveNumber};

const MAX_BATTLERS_PER_TEAM: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterTeam {
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
pub struct PerTeam<T: Clone> {
    ally_team_item: T,
    opponent_team_item: T,
}

impl<T: Clone> PerTeam<T> {
    pub fn new(ally_team_item: T, opponent_team_item: T) -> Self {
        Self {
            ally_team_item,
            opponent_team_item,
        }
    }

    pub(crate) fn both(item: T) -> Self {
        Self {
            ally_team_item: item.clone(),
            opponent_team_item: item,
        }
    }

    pub(crate) fn unwrap(&self) -> (&T, &T) {
        (&self.ally_team_item, &self.opponent_team_item)
    }

    pub(crate) fn unwrap_mut(&mut self) -> (&mut T, &mut T) {
        (&mut self.ally_team_item, &mut self.opponent_team_item)
    }

    /// Returns a copy of the items
    pub(crate) fn as_array(&self) -> [T; 2] {
        [self.ally_team_item.clone(), self.opponent_team_item.clone()]
    }
    
}

impl<T: Clone> Index<TeamID> for PerTeam<T> {
    type Output = T;

    fn index(&self, index: TeamID) -> &Self::Output {
        match index {
            TeamID::Allies => &self.ally_team_item,
            TeamID::Opponents => &self.opponent_team_item,
        }
    }
} 

impl<T: Clone> IndexMut<TeamID> for PerTeam<T> {
    fn index_mut(&mut self, index: TeamID) -> &mut Self::Output {
        match index {
            TeamID::Allies => &mut self.ally_team_item,
            TeamID::Opponents => &mut self.opponent_team_item,
        }
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
    pub fn new(monsters: Vec<Monster>) -> Self {
        assert!(monsters.first().is_some(), "There is not a single monster in the team.");
        assert!(monsters.len() <= MAX_BATTLERS_PER_TEAM);
        let monsters_iter = monsters.into_iter();
        let mut monsters = MaxSizeVec::new();
        monsters_iter.for_each(|monster| {monsters.push(monster)});
        MonsterTeam { monsters }
    }

    pub fn monsters(&self) -> &MaxSizeVec<Monster, 6> {
        &self.monsters
    }

    pub fn monsters_mut(&mut self) -> &mut MaxSizeVec<Monster, 6> {
        &mut self.monsters
    }

    pub fn composite_event_responder_instances(&self) -> CompositeEventResponderInstanceList {
        let mut out = Vec::new();
        for monster in self.monsters.iter() {
            out.append(&mut monster.composite_event_responder_instances())
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