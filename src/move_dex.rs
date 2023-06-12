#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::sim::{
    Battle, BattlerUID, EventResponderFilters, MonsterType, MoveCategory, MoveSpecies,
    SecondaryAction, Stat, DEFAULT_RESPONSE,
};

pub const Tackle: MoveSpecies = MoveSpecies {
    dex_number: 001,
    name: "Tackle",
    monster_type: MonsterType::Normal,
    category: MoveCategory::Physical,
    base_power: 40,
    base_accuracy: 100,
    composite_event_responder: DEFAULT_RESPONSE,
    priority: 0,
    composite_event_responder_filters: EventResponderFilters::default(),
    on_activate: None,
};

pub const Scratch: MoveSpecies = MoveSpecies {
    dex_number: 002,
    name: "Scratch",
    monster_type: MonsterType::Normal,
    category: MoveCategory::Physical,
    base_power: 40,
    base_accuracy: 100,
    composite_event_responder: DEFAULT_RESPONSE,
    priority: 0,
    composite_event_responder_filters: EventResponderFilters::default(),
    on_activate: None,
};

pub const Ember: MoveSpecies = MoveSpecies {
    dex_number: 003,
    name: "Ember",
    monster_type: MonsterType::Fire,
    category: MoveCategory::Special,
    base_power: 40,
    base_accuracy: 100,
    composite_event_responder: DEFAULT_RESPONSE,
    priority: 0,
    composite_event_responder_filters: EventResponderFilters::default(),
    on_activate: None,
};

pub const Bubble: MoveSpecies = MoveSpecies {
    dex_number: 004,
    name: "Bubble",
    monster_type: MonsterType::Water,
    category: MoveCategory::Special,
    base_power: 40,
    base_accuracy: 100,
    composite_event_responder: DEFAULT_RESPONSE,
    priority: 0,
    composite_event_responder_filters: EventResponderFilters::default(),
    on_activate: None,
};

pub const Growl: MoveSpecies = MoveSpecies {
    dex_number: 005,
    name: "Growl",
    monster_type: MonsterType::Normal,
    category: MoveCategory::Status,
    base_power: 0,
    base_accuracy: 100,
    composite_event_responder: DEFAULT_RESPONSE,
    priority: 0,
    composite_event_responder_filters: EventResponderFilters::default(),
    on_activate: Some(|battle: &mut Battle,
                  _attacker_uid: BattlerUID,
                  target_uid: BattlerUID| {
        _ = SecondaryAction::lower_stat(battle, target_uid, Stat::PhysicalAttack, 1);
    }),
};
