use monsim_utils::Nothing;

use crate::{sim::{event_dispatch::{EventFilteringOptions, EventHandlerDeck}, Type}, Effect, MoveUID, MoveUseContext};
use core::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub(crate) uid: MoveUID,// TODO: Move UID type to this module.
    pub(crate) species: &'static MoveSpecies, 
}

impl Move {
    pub fn new(uid: MoveUID, species: &'static MoveSpecies) -> Self {
        Self { 
            uid,
            species,
        }
    }

    #[inline(always)]
    pub fn name(&self) -> &'static str {
        self.species.name
    }

    pub fn on_activate_effect(&self) -> Effect<Nothing, MoveUseContext> {
        self.species.on_use_effect
    }

    #[inline(always)]
    pub fn category(&self) -> MoveCategory {
        self.species.category
    }

    #[inline(always)]
    pub fn base_power(&self) -> u16 {
        self.species.base_power
    }

    #[inline(always)]
    pub fn base_accuracy(&self) -> u16 {
        self.species.base_accuracy
    }

    #[inline(always)]
    pub fn max_power_points(&self) -> u8 {
        self.species.max_power_points
    }
    
    #[inline(always)]
    pub fn priority(&self) -> i8 {
        self.species.priority
    } 

    #[inline(always)]
    pub fn type_(&self) -> Type {
        self.species.type_
    }
    
    #[inline(always)]
    pub fn is_type(&self, type_: Type) -> bool {
        self.species.type_ == type_
    }

    #[inline(always)]
    pub fn species(&self) -> &'static MoveSpecies {
        self.species
    }
    
    #[inline(always)]
    pub(crate) fn event_handlers(&self) -> EventHandlerDeck {
        (self.species.event_handlers)()
    }
}

#[derive(Clone, Copy)]
pub struct MoveSpecies {
    dex_number: u16,
    name: &'static str,
    on_use_effect: Effect<Nothing, MoveUseContext>,
    base_accuracy: u16,
    base_power: u16,
    category: MoveCategory,
    max_power_points: u8,
    priority: i8,
    type_: Type,
    event_handlers: fn() -> EventHandlerDeck,
    _event_filtering_options: EventFilteringOptions,
}

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

impl Eq for MoveSpecies {}

impl MoveSpecies {
    pub const fn from_dex_entry(dex_entry: MoveDexEntry) -> Self {
        let MoveDexEntry { dex_number, name, on_use_effect, base_accuracy, base_power, category, max_power_points, priority, type_, event_handlers, event_filtering_options } = dex_entry;
        MoveSpecies {
            dex_number,
            name,
            on_use_effect,
            base_accuracy,
            base_power,
            category,
            max_power_points,
            priority,
            type_,
            event_handlers,
            _event_filtering_options: event_filtering_options,
        }
    }

    #[inline(always)]
    pub fn name(&self) -> &'static str {
        self.name
    }

    #[inline(always)]
    pub fn max_power_points(&self) -> u8 {
        self.max_power_points
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
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

pub struct MoveDexEntry {
    pub dex_number: u16,
    pub name: &'static str,

    pub on_use_effect: Effect<Nothing, MoveUseContext>,
    pub base_accuracy: u16,
    pub base_power: u16,
    pub category: MoveCategory,
    pub max_power_points: u8,
    pub priority: i8,
    pub type_: Type,
     
    pub event_handlers: fn() -> EventHandlerDeck,
    pub event_filtering_options: EventFilteringOptions,
}
