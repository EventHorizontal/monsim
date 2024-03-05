use crate::sim::{
    event::{EventHandlerDeck, EventFilteringOptions},
    Battle, MonsterUID, ElementalType, DEFAULT_RESPONSE,
};
use core::{fmt::Debug, slice::Iter};
use std::ops::Index;
use max_size_vec::MaxSizeVec;

#[derive(Clone, Copy)]
pub struct MoveSpecies {
    pub dex_number: u16,
    pub name: &'static str,
    pub elemental_type: ElementalType,
    pub category: MoveCategory,
    pub base_power: u16,
    pub base_accuracy: u16,
    pub priority: u16,
    pub event_handler_deck: EventHandlerDeck,
    pub event_handler_deck_filtering_options: EventFilteringOptions,
    /// `fn(battle: &mut Battle, attacker: MonsterUID, target: MonsterUID)`
    pub on_activate: Option<fn(&mut Battle, MonsterUID, MonsterUID)>,
}

pub const MOVE_DEFAULTS: MoveSpecies = MoveSpecies {
    dex_number: 000,
    name: "Unnamed",
    elemental_type: ElementalType::Normal,
    category: MoveCategory::Physical,
    base_power: 50,
    base_accuracy: 100,
    priority: 0,
    event_handler_deck: DEFAULT_RESPONSE,
    event_handler_deck_filtering_options: EventFilteringOptions::default(),
    on_activate: None,
};

impl Debug for MoveSpecies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:03} {},\n\t type: {:?},\n\t base accuracy: {}",
            self.dex_number, self.name, self.elemental_type, self.base_accuracy
        )
    }
}

impl PartialEq for MoveSpecies {
    fn eq(&self, other: &Self) -> bool {
        self.dex_number == other.dex_number
    }
}

impl Eq for MoveSpecies {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub species: MoveSpecies,
}

impl Move {
    pub fn new(species: MoveSpecies) -> Self {
        Move { species }
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
    
    pub fn is_type(&self, elemental_type: ElementalType) -> bool {
        self.species.elemental_type == elemental_type
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

const MAX_MOVES_PER_MOVESET: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveSet {
    moves: MaxSizeVec<Move, 4>,
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
        let moves_iter = moves.into_iter();
        let mut moves = MaxSizeVec::new();
        moves_iter.for_each(|move_| { moves.push(move_)});
        MoveSet { moves }
    }

    pub fn moves(&self) -> Iter<Move> {
        self.moves.iter()
    }

    pub fn move_(&self, id: MoveNumber) -> &Move {
        self.moves
            .get(id as usize)
            .unwrap_or_else(|| panic!("The move at the {:?} index should exist.", id))
    }

    pub fn move_mut(&mut self, id: MoveNumber) -> &mut Move {
        self.moves
            .get_mut(id as usize)
            .unwrap_or_else(|| panic!("The move at the {:?} index should exist.", id))
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
