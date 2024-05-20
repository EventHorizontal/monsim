#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use super::{
    move_::{MoveCategory, MoveSpecies},
    Type,
};
use crate::{effects, sim::{
    event_dispatch::EventFilteringOptions, BattleState, MonsterID, Move, Stat
}, test_status_dex::*, BattleSimulator, Count, EventHandlerDeck, MoveDexEntry, MoveUseContext, TargetFlags};

pub const Tackle: MoveSpecies = MoveSpecies::from_dex_entry(
    MoveDexEntry {
        dex_number: 001,
        name: "Tackle",
        on_hit_effect: effects::deal_default_damage,
        base_accuracy: 100,
        base_power: 40,
        category: MoveCategory::Physical,
        max_power_points: 35,
        hits_per_target: Count::Fixed(1),
        priority: 0,
        targets: TargetFlags::ANY
                    .union(TargetFlags::ADJACENT)
                    .union(TargetFlags::OPPONENTS)
                    .union(TargetFlags::ALLIES),
        type_: Type::Normal,
        event_handlers: EventHandlerDeck::empty,
        event_filtering_options: EventFilteringOptions::default(),
    }
);

pub const Scratch: MoveSpecies = MoveSpecies::from_dex_entry(
    MoveDexEntry {
        dex_number: 002,
        name: "Scratch",
        on_hit_effect: effects::deal_default_damage,
        base_accuracy: 100,
        base_power: 40,
        category: MoveCategory::Physical,
        max_power_points: 35,
        hits_per_target: Count::Fixed(1),
        priority: 0,
        targets: TargetFlags::ANY
                    .union(TargetFlags::ADJACENT)
                    .union(TargetFlags::OPPONENTS)
                    .union(TargetFlags::ALLIES),
        type_: Type::Normal,
        event_handlers: EventHandlerDeck::empty,
        event_filtering_options: EventFilteringOptions::default(),
    }
);

pub const Ember: MoveSpecies = MoveSpecies::from_dex_entry(
    MoveDexEntry {
        dex_number: 003,
        name: "Ember",
        on_hit_effect: |sim, context| {
            let hit_outcome = effects::deal_default_damage(sim, context);
            if sim.chance(9, 10) && hit_outcome.succeeded() {
                effects::add_persistent_status(sim, (context.target_id, &Burned));
            }
            hit_outcome
        },
        base_accuracy: 100,
        base_power: 40,
        category: MoveCategory::Special,
        max_power_points: 35,
        hits_per_target: Count::Fixed(1),
        priority: 0,
        targets: TargetFlags::ANY
                    .union(TargetFlags::ADJACENT)
                    .union(TargetFlags::OPPONENTS),
        type_: Type::Fire,
        event_handlers: EventHandlerDeck::empty,
        event_filtering_options: EventFilteringOptions::default(),
    }
);

pub const Bubble: MoveSpecies = MoveSpecies::from_dex_entry(
    MoveDexEntry {
        dex_number: 004,
        name: "Bubble",
        on_hit_effect: effects::deal_default_damage,
        base_accuracy: 100,
        base_power: 40,
        category: MoveCategory::Special,
        max_power_points: 35,
        hits_per_target: Count::Fixed(1),
        priority: 0,
        targets: TargetFlags::ALL
                    .union(TargetFlags::ADJACENT)
                    .union(TargetFlags::OPPONENTS),
        type_: Type::Water,
        event_handlers: EventHandlerDeck::empty,
        event_filtering_options: EventFilteringOptions::default(),
    }
);

pub const Growl: MoveSpecies = MoveSpecies::from_dex_entry(
    MoveDexEntry {
        dex_number: 005,
        name: "Growl",
        on_hit_effect: |sim, context| { 
            let stat_lowering_succeeded = effects::lower_stat(sim, (context.target_id, Stat::PhysicalAttack, 1)); 
            stat_lowering_succeeded
        },
        base_accuracy: 100,
        base_power: 0,
        category: MoveCategory::Status,
        max_power_points: 40,
        hits_per_target: Count::Fixed(1),
        priority: 0,
        targets: TargetFlags::ALL
                    .union(TargetFlags::ADJACENT)
                    .union(TargetFlags::OPPONENTS),
        type_: Type::Normal,
        event_handlers: EventHandlerDeck::empty,
        event_filtering_options: EventFilteringOptions::default(),
    }
);

pub const DragonDance: MoveSpecies = MoveSpecies::from_dex_entry(
    MoveDexEntry {
        dex_number: 006,
        name: "Dragon Dance",
        on_hit_effect: |sim, context| {
            let first_stat_raise_succeeded = effects::raise_stat(sim, (context.target_id, Stat::PhysicalAttack, 1));
            let second_stat_raise_succeeded = effects::raise_stat(sim, (context.target_id, Stat::Speed,         1));
            first_stat_raise_succeeded & second_stat_raise_succeeded
        },
        base_accuracy: 100,
        base_power: 0,
        category: MoveCategory::Status,
        max_power_points: 20,
        hits_per_target: Count::Fixed(1),
        priority: 0,
        targets: TargetFlags::SELF,
        type_: Type::Dragon,
        event_handlers: EventHandlerDeck::empty,
        event_filtering_options: EventFilteringOptions::default(),
    }
);

pub const BulletSeed: MoveSpecies = MoveSpecies::from_dex_entry(
    MoveDexEntry {
        dex_number: 007,
        name: "Bullet Seed",
        on_hit_effect: effects::deal_default_damage,
        hits_per_target: Count::RandomInRange { min: 2, max: 5 },
        base_accuracy: 100,
        base_power: 25,
        category: MoveCategory::Physical,
        max_power_points: 20,
        priority: 0,
        targets: TargetFlags::ANY
            .union(TargetFlags::ADJACENT)
            .union(TargetFlags::ALLIES)
            .union(TargetFlags::OPPONENTS),
        type_: Type::Grass,
        event_handlers: EventHandlerDeck::empty,
        event_filtering_options: EventFilteringOptions::default(),
    }
);

pub const Confusion: MoveSpecies = MoveSpecies::from_dex_entry(
    MoveDexEntry {
        dex_number: 008,
        name: "Confusion",
        on_hit_effect: |sim, context| {
            let hit_outcome = effects::deal_default_damage(sim, context);
            if sim.chance(1, 10) && hit_outcome.succeeded() {
                effects::add_volatile_status(sim, (context.target_id, &Confused));
            }
            hit_outcome
        },
        hits_per_target: Count::Fixed(1),
        base_accuracy: 100,
        base_power: 50,
        category: MoveCategory::Special,
        max_power_points: 25,
        priority: 0,
        targets: TargetFlags::ANY
            .union(TargetFlags::ADJACENT)
            .union(TargetFlags::ALLIES)
            .union(TargetFlags::OPPONENTS),
        type_: Type::Psychic,
        event_handlers: EventHandlerDeck::empty,
        event_filtering_options: EventFilteringOptions::default(),
    }
);