use crate::sim::{
    event::{EventFilteringOptions, EventHandlerDeck}, Battle, MonsterRef, MoveUID, Type
};
use core::fmt::Debug;
use std::{cell::Cell, ops::Deref};

#[derive(Debug, Clone, Copy)]
pub struct MoveRef<'a> {
    move_data: &'a Move
}

impl<'a> PartialEq for MoveRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl<'a> Eq for MoveRef<'a> {}

impl<'a> Deref for MoveRef<'a> {
    type Target = Move;

    fn deref(&self) -> &Self::Target {
        self.move_data
    }
}

impl<'a> MoveRef<'a> {
    pub(crate) fn new(move_data: &'a Move) -> Self {
        Self {
            move_data,
        }
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
    pub priority: i8,
    pub event_handler_deck: EventHandlerDeck,
    pub event_handler_deck_filtering_options: EventFilteringOptions,
    /// `fn(battle: &mut Battle, attacker: MonsterUID, target: MonsterUID)`
    pub on_activate: Option<for<'a> fn(&'a mut Battle, MonsterRef<'a>, MonsterRef<'a>)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Move {
    pub(crate) uid: MoveUID,
    pub type_: Cell<Type>,
    pub base_power: Cell<u16>,
    pub category: Cell<MoveCategory>,
    pub base_accuracy: Cell<u16>,
    pub priority: Cell<i8>,
    species: Cell<MoveSpecies>,
}

impl Move {
    pub(crate) fn new(uid: MoveUID, species: MoveSpecies) -> Self {
        Move { 
            uid,
            species: Cell::new(species),
            type_: Cell::new(species.type_),
            base_power: Cell::new(species.base_power),
            category: Cell::new(species.category),
            priority: Cell::new(species.priority),
            base_accuracy: Cell::new(species.base_accuracy), 
        }
    }

    pub(crate) fn on_activate(&self, battle: &mut Battle, owner: MonsterRef, target: MonsterRef) {
        let on_activate_logic = self.species.get().on_activate;
        if let Some(on_activate_logic) = on_activate_logic {
            on_activate_logic(battle, owner, target);
        }
    }

    pub fn name(&self) -> &'static str {
        self.species.get().name
    }
    
    pub fn is_type(&self, type_: Type) -> bool {
        self.species.get().type_ == type_
    }

    pub fn event_handler_deck(&self) -> EventHandlerDeck {
        self.species.get().event_handler_deck
    }
}

const MOVE_DEFAULTS: MoveSpecies = MoveSpecies {
    dex_number: 000,
    name: "Unnamed",
    type_: Type::Normal,
    category: MoveCategory::Physical,
    base_power: 50,
    base_accuracy: 100,
    priority: 0,
    event_handler_deck: EventHandlerDeck::const_default(),
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
pub enum MoveCategory {
    #[default]
    Physical,
    Special,
    Status,
}

const MAX_MOVES_PER_MOVESET: usize = 4;

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
