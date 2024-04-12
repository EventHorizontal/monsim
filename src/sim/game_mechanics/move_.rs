use monsim_utils::{MaxSizedVec, Nothing};

use crate::{sim::{
    event::{BattleAPI, EventFilteringOptions, EventHandlerStorage, OwnerInfo}, MonsterUID, Type,
}, BattleEntities, MoveUID, TheMoveUsed};
use core::{fmt::Debug, slice::Iter};
use std::ops::Index;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub(crate) uid: MoveUID,// TODO: Move UID type to this module.
    pub(crate) species: &'static MoveSpecies,
    pub(crate) base_accuracy: u16,
    pub(crate) base_power: u16,
    pub(crate) category: MoveCategory,
    pub(crate) power_points: u8,
    pub(crate) priority: i8, 
    pub(crate) type_: Type,
}

impl Move {
    pub fn new(uid: MoveUID, species: &'static MoveSpecies) -> Self {
        Self { 
            uid,
            species,
            base_accuracy: species.base_accuracy,
            base_power: species.base_power,
            category: species.category,
            power_points: species.max_power_points,
            priority: species.priority,
            type_: species.type_, 
        }
    }

    pub fn category(&self) -> MoveCategory {
        self.species.category
    }

    pub fn base_power(&self) -> u16 {
        self.species.base_power
    }

    pub fn base_accuracy(&self) -> u16 {
        self.species.base_accuracy
    }

    pub(crate) fn activate(&self, api: BattleAPI, context: TheMoveUsed) {
        let on_activate_logic = self.species.on_activate;
        if let Some(on_activate_logic) = on_activate_logic {
            on_activate_logic(api, context);
        }
    }
    
    pub fn is_type(&self, type_: Type) -> bool {
        self.species.type_ == type_
    }
}

#[derive(Clone, Copy)]
pub struct MoveSpecies {
    pub dex_number: u16,
    pub name: &'static str,
    
    pub base_accuracy: u16,
    pub base_power: u16,
    pub category: MoveCategory,
    pub max_power_points: u8,
    pub priority: i8,
    pub type_: Type,
    
    /// A special callback for what the move itself does. More pertinent to status moves.
    pub on_activate: Option<fn(BattleAPI, TheMoveUsed)>,
    pub event_callbacks: fn(OwnerInfo, &mut EventHandlerStorage) -> Nothing,
    pub event_handler_deck_filtering_options: EventFilteringOptions,
}

const MOVE_DEFAULTS: MoveSpecies = MoveSpecies {
    dex_number: 000,
    name: "Unnamed",
    
    on_activate: None,
    base_power: 50,
    base_accuracy: 100,
    category: MoveCategory::Physical,
    max_power_points: u8::MAX,
    priority: 0,
    type_: Type::Normal,

    event_callbacks: |_, _| {},
    event_handler_deck_filtering_options: EventFilteringOptions::default(),
};

impl Debug for MoveSpecies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:03} {},\n\t type: {:?},\n\t base accuracy: {}",
            self.dex_number, self.name, self.type_, self.base_accuracy
        )
    }
}

impl PartialEq for MoveSpecies {
    fn eq(&self, other: &Self) -> bool {
        self.dex_number == other.dex_number
    }
}

impl MoveSpecies {
    pub const fn const_default() -> Self {
        MOVE_DEFAULTS
    }
}

impl Eq for MoveSpecies {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

const MAX_MOVES_PER_MOVESET: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveSet {
    moves: MaxSizedVec<Move, 4>,
}

impl Index<usize> for MoveSet {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(
            index < MAX_MOVES_PER_MOVESET,
            "MoveSet can only be indexed with 0-3. The index passed was {}",
            index
        );
        &self.moves[index]
    }
}

impl MoveSet {
    pub fn new(moves: Vec<Move>) -> Self {
        assert!(moves.first().is_some(), "There is no first move.");
        assert!(moves.len() <= MAX_MOVES_PER_MOVESET);
        let moves = MaxSizedVec::from_vec(moves);
        MoveSet { moves }
    }

    pub fn moves(&self) -> Iter<Move> {
        self.moves.iter()
    }

    pub fn move_(&self, id: MoveNumber) -> &Move {
        &self.moves[id as usize]
    }

    pub fn move_mut(&mut self, id: MoveNumber) -> &mut Move {
        &mut self.moves[id as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveNumber {
    _1,
    _2,
    _3,
    _4,
}

impl From<usize> for MoveNumber {
    fn from(value: usize) -> Self {
        match value {
            0 => MoveNumber::_1,
            1 => MoveNumber::_2,
            2 => MoveNumber::_3,
            3 => MoveNumber::_4,
            _ => panic!("MoveNumber can only be formed from usize 0 to 3."),
        }
    }
}
