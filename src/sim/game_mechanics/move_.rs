use crate::sim::{
    event::{CompositeEventResponder, EventResponderFilters},
    Battle, BattlerUID, MonsterType, DEFAULT_RESPONSE,
};
use core::{fmt::Debug, slice::Iter};
use std::ops::Index;

#[derive(Clone, Copy)]
pub struct MoveSpecies {
    pub dex_number: u16,
    pub name: &'static str,
    pub monster_type: MonsterType,
    pub category: MoveCategory,
    pub base_power: u16,
    pub base_accuracy: u16,
    pub priority: u16,
    pub composite_event_responder: CompositeEventResponder,
    pub composite_event_responder_filters: EventResponderFilters,
    pub on_activate: Option<fn(&mut Battle, BattlerUID, BattlerUID) -> ()>,
}

pub const MOVE_DEFAULTS: MoveSpecies = MoveSpecies { 
    dex_number: 000, 
    name: "Unnamed", 
    monster_type: MonsterType::Normal, 
    category: MoveCategory::Physical, 
    base_power: 50, 
    base_accuracy: 100, 
    priority: 0, 
    composite_event_responder: DEFAULT_RESPONSE, 
    composite_event_responder_filters: EventResponderFilters::default(), 
    on_activate: None 
};


impl Debug for MoveSpecies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:03} {},\n\t type: {:?},\n\t base accuracy: {}",
            self.dex_number, self.name, self.monster_type, self.base_accuracy
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

    pub(crate) fn on_activate(
        &self,
        battle: &mut Battle,
        owner_uid: BattlerUID,
        target_uid: BattlerUID,
    ) {
        let on_activate_logic = self.species.on_activate;
        if let Some(on_activate_logic) = on_activate_logic {
            on_activate_logic(battle, owner_uid, target_uid);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveSet {
    moves: Vec<Move>,
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
        MoveSet { moves }
    }

    pub fn moves(&self) -> Iter<Move> {
        return self.moves.iter();
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
