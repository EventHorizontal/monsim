#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::sim::{
    MoveCategory, MoveSpecies,
    prng::Prng,
    Battle, BattlerUID, EventResponderFilters, MonType, SecondaryAction, Stat,
    DEFAULT_RESPONSE,
};

// TEMP: Probably will be replaced due to a possible rework to how damaging and status moves ar calculated, potentially making all moves have an on_activate
fn no_on_activate(
    _context: &mut Battle,
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
    event_responder: DEFAULT_RESPONSE,
    priority: 0,
    event_responder_filters: EventResponderFilters::default(),
    on_activate: no_on_activate,
};

pub const Scratch: MoveSpecies = MoveSpecies {
    dex_number: 002,
    name: "Scratch",
    type_: MonType::Normal,
    category: MoveCategory::Physical,
    base_power: 40,
    base_accuracy: 100,
    event_responder: DEFAULT_RESPONSE,
    priority: 0,
    event_responder_filters: EventResponderFilters::default(),
    on_activate: no_on_activate,
};

pub const Ember: MoveSpecies = MoveSpecies {
    dex_number: 003,
    name: "Ember",
    type_: MonType::Fire,
    category: MoveCategory::Special,
    base_power: 40,
    base_accuracy: 100,
    event_responder: DEFAULT_RESPONSE,
    priority: 0,
    event_responder_filters: EventResponderFilters::default(),
    on_activate: no_on_activate,
};

pub const Bubble: MoveSpecies = MoveSpecies {
    dex_number: 004,
    name: "Bubble",
    type_: MonType::Water,
    category: MoveCategory::Special,
    base_power: 40,
    base_accuracy: 100,
    event_responder: DEFAULT_RESPONSE,
    priority: 0,
    event_responder_filters: EventResponderFilters::default(),
    on_activate: no_on_activate,
};

pub const Growl: MoveSpecies = MoveSpecies {
    dex_number: 005,
    name: "Growl",
    type_: MonType::Normal,
    category: MoveCategory::Status,
    base_power: 0,
    base_accuracy: 100,
    event_responder: DEFAULT_RESPONSE,
    priority: 0,
    event_responder_filters: EventResponderFilters::default(),
    on_activate: |battle: &mut Battle,
                  prng: &mut Prng,
                  _attacker_uid: BattlerUID,
                  target_uid: BattlerUID| {
        _ = SecondaryAction::lower_stat(battle, prng, target_uid, Stat::PhysicalAttack, 1);
    },
};
