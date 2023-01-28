#![allow(non_upper_case_globals)]

use super::{
    move_::{MoveCategory, MoveSpecies},
    MonType,
};
use crate::battle_sim::event::{EventHandlerFilters, DEFAULT_HANDLERS};

pub const Tackle: MoveSpecies = MoveSpecies {
    dex_number: 001,
    name: "Tackle",
    type_: MonType::Normal,
    category: MoveCategory::Physical,
    base_power: 40,
    base_accuracy: 100,
    event_handlers: DEFAULT_HANDLERS,
    priority: 0,
    event_handler_filters: EventHandlerFilters::default(),
};

pub const Scratch: MoveSpecies = MoveSpecies {
    dex_number: 002,
    name: "Scratch",
    type_: MonType::Normal,
    category: MoveCategory::Physical,
    base_power: 40,
    base_accuracy: 100,
    event_handlers: DEFAULT_HANDLERS,
    priority: 0,
    event_handler_filters: EventHandlerFilters::default(),
};

pub const Ember: MoveSpecies = MoveSpecies {
    dex_number: 003,
    name: "Ember",
    type_: MonType::Fire,
    category: MoveCategory::Special,
    base_power: 40,
    base_accuracy: 100,
    event_handlers: DEFAULT_HANDLERS,
    priority: 0,
    event_handler_filters: EventHandlerFilters::default(),
};

pub const Bubble: MoveSpecies = MoveSpecies {
    dex_number: 004,
    name: "Bubble",
    type_: MonType::Water,
    category: MoveCategory::Special,
    base_power: 40,
    base_accuracy: 100,
    event_handlers: DEFAULT_HANDLERS,
    priority: 0,
    event_handler_filters: EventHandlerFilters::default(),
};
