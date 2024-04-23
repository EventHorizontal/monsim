#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use super::{
    move_::{MoveCategory, MoveSpecies},
    Type,
};
use crate::{sim::{
    event_dispatch::EventFilteringOptions, BattleState, MonsterUID, Move, Stat
}, BattleSimulator, effects::*, EventHandlerDeck, MoveDexEntry, MoveUseContext};

pub const Tackle: MoveSpecies = MoveSpecies::from_dex_entry(
    MoveDexEntry {
        dex_number: 001,
        name: "Tackle",
        type_: Type::Normal,
        category: MoveCategory::Physical,
        base_power: 40,
        base_accuracy: 100,
        on_use_effect: DealDefaultDamage,
        max_power_points: 35,
        priority: 0,
        event_handlers: EventHandlerDeck::empty,
        event_filtering_options: EventFilteringOptions::default(),
    }
);

pub const Scratch: MoveSpecies = MoveSpecies::from_dex_entry(
    MoveDexEntry {
        dex_number: 002,
        name: "Scratch",
        type_: Type::Normal,
        category: MoveCategory::Physical,
        base_power: 40,
        base_accuracy: 100,
        on_use_effect: DealDefaultDamage,
        max_power_points: 35,
        priority: 0,
        event_handlers: EventHandlerDeck::empty,
        event_filtering_options: EventFilteringOptions::default(),
    }
);

pub const Ember: MoveSpecies = MoveSpecies::from_dex_entry(
    MoveDexEntry {
        dex_number: 003,
        name: "Ember",
        type_: Type::Fire,
        category: MoveCategory::Special,
        base_power: 40,
        base_accuracy: 100,
        on_use_effect: DealDefaultDamage,
        max_power_points: 35,
        priority: 0,
        event_handlers: EventHandlerDeck::empty,
        event_filtering_options: EventFilteringOptions::default(),
    }
);

pub const Bubble: MoveSpecies = MoveSpecies::from_dex_entry(
    MoveDexEntry {
        dex_number: 004,
        name: "Bubble",
        type_: Type::Water,
        category: MoveCategory::Physical,
        base_power: 40,
        base_accuracy: 100,
        on_use_effect: DealDefaultDamage,
        max_power_points: 35,
        priority: 0,
        event_handlers: EventHandlerDeck::empty,
        event_filtering_options: EventFilteringOptions::default(),
    }
);

pub const Growl: MoveSpecies = MoveSpecies::from_dex_entry(
    MoveDexEntry {
        dex_number: 005,
        name: "Growl",
        type_: Type::Normal,
        category: MoveCategory::Status,
        base_power: 0,
        base_accuracy: 100,
        on_use_effect: Effect::from(|sim, context| { 
            _ = LowerStat(sim, (context.target, Stat::PhysicalAttack, 1)); 
        }),
        max_power_points: 40,
        priority: 0,
        event_handlers: EventHandlerDeck::empty,
        event_filtering_options: EventFilteringOptions::default(),
    }
);
