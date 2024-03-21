#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use super::{
    move_::{MoveCategory, MoveSpecies},
    Type,
};
use crate::sim::{
    event::EventFilteringOptions, Battle, Effect, MonsterRef, MonsterUID, Move, Stat
};

pub const Tackle: MoveSpecies = MoveSpecies {
    dex_number: 001,
    name: "Tackle",
    type_: Type::Normal,
    category: MoveCategory::Physical,
    base_power: 40,
    base_accuracy: 100,
    ..MoveSpecies::default()
};

pub const Scratch: MoveSpecies = MoveSpecies {
    dex_number: 002,
    name: "Scratch",
    type_: Type::Normal,
    category: MoveCategory::Physical,
    base_power: 40,
    base_accuracy: 100,
    ..MoveSpecies::default()
};

pub const Ember: MoveSpecies = MoveSpecies {
    dex_number: 003,
    name: "Ember",
    type_: Type::Fire,
    category: MoveCategory::Special,
    base_power: 40,
    base_accuracy: 100,
    ..MoveSpecies::default()
};

pub const Bubble: MoveSpecies = MoveSpecies {
    dex_number: 004,
    name: "Bubble",
    type_: Type::Water,
    category: MoveCategory::Special,
    base_power: 40,
    base_accuracy: 100,
    ..MoveSpecies::default()
};

pub const Growl: MoveSpecies = MoveSpecies {
    dex_number: 005,
    name: "Growl",
    type_: Type::Normal,
    category: MoveCategory::Status,
    base_power: 0,
    base_accuracy: 100,
    on_activate: Some(|battle: &mut Battle, attacker: MonsterRef, target: MonsterRef| {
        _ = Effect::lower_stat(battle, target, Stat::PhysicalAttack, 1);
    }),
    ..MoveSpecies::default()
};
