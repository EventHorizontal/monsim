use std::{cell::Cell, fmt::{Debug, Display, Formatter}, ops::{Index, IndexMut}};
use monsim_utils::{not, Ally, MaxSizedVec, Opponent};

use crate::sim::{event::OwnedEventHandlerDeck, MonsterNumber};
use super::{MonsterInternal, MonsterUID, MoveNumber};

const MAX_BATTLERS_PER_TEAM: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterTeam<'a> {
    id: TeamUID,
    active_monster: &'a Cell<MonsterUID>,
    monsters: &'a MaxSizedVec<Cell<MonsterInternal>, 6>
}

impl<'a> MonsterTeam<'a> {
    pub fn new(active_monster: &Cell<MonsterUID>, monsters: &MaxSizedVec<Cell<MonsterInternal>, 6>, id: TeamUID) -> Self {
        let number_of_monsters = monsters.len();
        assert!(not!(monsters.is_empty()), "Expected at least 1 monster but none were given.");
        assert!(number_of_monsters <= MAX_BATTLERS_PER_TEAM, "Expected at most 6 monsters but {number_of_monsters} were given.");
        Self {
            id,
            active_monster,
            // First monster is the default active monster TODO: multi-monster battle.
            monsters, 
        }
    }

    pub fn monsters(&self) -> &MaxSizedVec<Cell<MonsterInternal>, 6> {
        &self.monsters
    }

    pub fn active_monster(&self) -> &Cell<MonsterInternal> {
        self.monsters.iter()
            .find(|monster| { monster.get().uid == self.active_monster.get() })
            .expect("Expected the active monster to be a valid Monster within the team.")
    }

    pub fn set_active_monster(&self, to_which_monster: &Cell<MonsterInternal>) {
        self.active_monster.set(to_which_monster.get().uid)
    }

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

/// For use by the engine itself, this stores the data in a not-explicitly-synchronised but `Copy` format. When the data is presented to the user,
/// it will be as a `MonsterTeam`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MonsterTeamInternal {
    pub id: TeamUID,
    pub active_monster_uid: MonsterUID,
    pub monsters: MaxSizedVec<MonsterUID, 6>,
}

impl MonsterTeamInternal {
    pub fn new(monsters: MaxSizedVec<MonsterUID, 6>, id: TeamUID) -> Self {
        let number_of_monsters = monsters.len();
        assert!(not!(monsters.is_empty()), "Expected at least 1 monster but none were given.");
        assert!(number_of_monsters <= MAX_BATTLERS_PER_TEAM, "Expected at most 6 monsters but {number_of_monsters} were given.");
        Self {
            id,
            active_monster_uid: MonsterUID { team_uid: id, monster_number: MonsterNumber::_1},
            // First monster is the default active monster TODO: multi-monster battle.
            monsters, 
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MoveUID {
    pub owner_uid: MonsterUID,
    pub move_number: MoveNumber,
}