#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use super::{
    move_::{MoveCategory, MoveSpecies},
    ElementalType,
};
use crate::sim::{
    action::SecondaryAction,
    event::{EventFilteringOptions, DEFAULT_RESPONSE},
    Battle, MonsterUID, Stat, MOVE_DEFAULTS,
};

pub const Tackle: MoveSpecies = MoveSpecies {
    dex_number: 001,
    name: "Tackle",
    elemental_type: ElementalType::Normal,
    category: MoveCategory::Physical,
    base_power: 40,
    base_accuracy: 100,
    ..MOVE_DEFAULTS
};

pub const Scratch: MoveSpecies = MoveSpecies {
    dex_number: 002,
    name: "Scratch",
    elemental_type: ElementalType::Normal,
    category: MoveCategory::Physical,
    base_power: 40,
    base_accuracy: 100,
    ..MOVE_DEFAULTS
};

pub const Ember: MoveSpecies = MoveSpecies {
    dex_number: 003,
    name: "Ember",
    elemental_type: ElementalType::Fire,
    category: MoveCategory::Special,
    base_power: 40,
    base_accuracy: 100,
    ..MOVE_DEFAULTS
};

pub const Bubble: MoveSpecies = MoveSpecies {
    dex_number: 004,
    name: "Bubble",
    elemental_type: ElementalType::Water,
    category: MoveCategory::Special,
    base_power: 40,
    base_accuracy: 100,
    ..MOVE_DEFAULTS
};

pub const Growl: MoveSpecies = MoveSpecies {
    dex_number: 005,
    name: "Growl",
    elemental_type: ElementalType::Normal,
    category: MoveCategory::Status,
    base_power: 0,
    base_accuracy: 100,
    on_activate: Some(|battle: &mut Battle, _attacker_uid: MonsterUID, target_uid: MonsterUID| {
        _ = SecondaryAction::lower_stat(battle, target_uid, Stat::PhysicalAttack, 1);
    }),
    ..MOVE_DEFAULTS
};
