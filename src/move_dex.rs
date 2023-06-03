#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::sim::{
    move_::{MoveCategory, MoveSpecies},
    prng::Prng,
    BattleContext, BattlerUID, EventHandlerFilters, MonType, SecondaryAction, Stat,
    DEFAULT_HANDLERS,
};

// TEMP: Probably will be replaced due to a possible rework to how damaging and status moves ar calculated, potentially making all moves have an on_activate
fn no_on_activate(
    _context: &mut BattleContext,
    _prng: &mut Prng,
    _attacker_uid: BattlerUID,
    _target_uid: BattlerUID,
) {
}

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
    on_activate: no_on_activate,
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
    on_activate: no_on_activate,
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
    on_activate: no_on_activate,
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
    on_activate: no_on_activate,
};

pub const Growl: MoveSpecies = MoveSpecies {
    dex_number: 005,
    name: "Growl",
    type_: MonType::Normal,
    category: MoveCategory::Status,
    base_power: 0,
    base_accuracy: 100,
    event_handlers: DEFAULT_HANDLERS,
    priority: 0,
    event_handler_filters: EventHandlerFilters::default(),
    on_activate: |ctx: &mut BattleContext,
                  prng,
                  _attacker_uid: BattlerUID,
                  target_uid: BattlerUID| {
        let _ = SecondaryAction::lower_stat(ctx, prng, target_uid, Stat::PhysicalAttack, 1);
    },
};
