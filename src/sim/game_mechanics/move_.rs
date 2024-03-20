use crate::sim::{
    event::{EventFilteringOptions, EventHandlerDeck}, Battle, MonsterUID, MoveUID, Type, ALLY_5
};
use core::fmt::Debug;
use std::{cell::Cell, ops::Index};
use monsim_utils::{not, MaxSizedVec};

#[derive(Debug, Clone, Copy)]
pub struct Move<'a> {
    uid: MoveUID,
    move_data: &'a Cell<MoveData>
}

impl<'a> PartialEq for Move<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.uid() == other.uid()
    }
}

impl<'a> Eq for Move<'a> {}


impl<'a> Move<'a> {
    pub(crate) fn new(uid: MoveUID, data: &Cell<MoveData>) -> Self {
        Self {
            uid,
            move_data: data,
        }
    }
    
    pub fn species(&self) -> MoveSpecies {
        self.data_copy().species
    }
    
    pub(crate) fn data_copy(&self) -> MoveData {
        self.move_data.get()
    }

    pub(crate) fn uid(&self) -> MoveUID {
        self.data_copy().uid
    }
}

#[derive(Clone, Copy)]
pub struct MoveSpecies {
    pub dex_number: u16,
    pub name: &'static str,
    pub type_: Type,
    pub category: MoveCategory,
    pub base_power: u16,
    pub base_accuracy: u16,
    pub priority: u16,
    pub event_handler_deck: EventHandlerDeck,
    pub event_handler_deck_filtering_options: EventFilteringOptions,
    /// `fn(battle: &mut Battle, attacker: MonsterUID, target: MonsterUID)`
    pub on_activate: Option<fn(&mut Battle, MonsterUID, MonsterUID)>,
}

const MOVE_DEFAULTS: MoveSpecies = MoveSpecies {
    dex_number: 000,
    name: "Unnamed",
    type_: Type::Normal,
    category: MoveCategory::Physical,
    base_power: 50,
    base_accuracy: 100,
    priority: 0,
    event_handler_deck: EventHandlerDeck::default(),
    event_handler_deck_filtering_options: EventFilteringOptions::default(),
    on_activate: None,
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

impl Default for MoveSpecies {
    fn default() -> Self {
        MOVE_DEFAULTS
    }
}

impl MoveSpecies {
    pub const fn default() -> Self {
        MOVE_DEFAULTS
    }
}

impl Eq for MoveSpecies {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MoveData {
    pub uid: MoveUID,
    pub species: MoveSpecies,
}

impl MoveData {
    pub(crate) fn new(uid: MoveUID, species: MoveSpecies) -> Self {
        MoveData { 
            uid,
            species, 
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

    pub(crate) fn on_activate(&self, battle: &mut Battle, owner_uid: MonsterUID, target_uid: MonsterUID) {
        let on_activate_logic = self.species.on_activate;
        if let Some(on_activate_logic) = on_activate_logic {
            on_activate_logic(battle, owner_uid, target_uid);
        }
    }
    
    pub fn is_type(&self, type_: Type) -> bool {
        self.species.type_ == type_
    }
    
    const fn placeholder() -> Self {
        Self {
            uid: MoveUID { owner_uid: ALLY_5, move_number: MoveNumber::_4 },
            species: MoveSpecies::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

const MAX_MOVES_PER_MOVESET: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MoveSet {
    moves: MaxSizedVec<MoveData, 4>,
}

impl Index<usize> for MoveSet {
    type Output = MoveData;

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
    pub fn new(moves: &[MoveData]) -> Self {
        let number_of_moves = moves.len();
        assert!(not![moves.is_empty()], "Expected one Move, but found zero.");
        assert!(number_of_moves <= MAX_MOVES_PER_MOVESET, "Expected at most {MAX_MOVES_PER_MOVESET} but found {number_of_moves} moves.");
        let moves = MaxSizedVec::from_slice_with_default_padding(moves);
        MoveSet { moves }
    }

    pub fn moves(&self) -> impl Iterator<Item = MoveData> {
        self.moves.into_iter()
    }

    pub fn move_(&self, id: MoveNumber) -> &MoveData {
        &self.moves[id as usize]
    }

    pub fn move_mut(&mut self, id: MoveNumber) -> &mut MoveData {
        &mut self.moves[id as usize]
    }
    
    pub(crate) const fn placeholder() -> MoveSet {
        Self {
            moves: MaxSizedVec::placeholder(MoveData::placeholder()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MoveNumber {
    #[default]
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
