#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{effects::*, sim::{BattleState, EventFilteringOptions, MonsterID, MoveCategory, MoveSpecies, Stat, Type}, EventHandlerDeck, MoveDexEntry, TargetFlags};

pub const Tackle: MoveSpecies = MoveSpecies::from_dex_entry(
    MoveDexEntry {
        dex_number: 001,
        name: "Tackle",
        on_use_effect: DealDefaultDamage,
        base_accuracy: 100,
        base_power: 40,
        category: MoveCategory::Physical,
        max_power_points: 35,
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
        on_use_effect: DealDefaultDamage,
        base_accuracy: 100,
        base_power: 40,
        category: MoveCategory::Physical,
        max_power_points: 35,
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
        on_use_effect: DealDefaultDamage,
        base_accuracy: 100,
        base_power: 40,
        category: MoveCategory::Special,
        max_power_points: 35,
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
        on_use_effect: DealDefaultDamage,
        base_accuracy: 100,
        base_power: 40,
        category: MoveCategory::Special,
        max_power_points: 35,
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
        on_use_effect: Effect::from(|sim, context| { 
            _ = LowerStat(sim, (context.target_id, Stat::PhysicalAttack, 1)); 
        }),
        base_accuracy: 100,
        base_power: 0,
        category: MoveCategory::Status,
        max_power_points: 40,
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
        on_use_effect: Effect::from(|sim, context| {
            RaiseStat(sim, (context.target_id, Stat::PhysicalAttack, 1));
            RaiseStat(sim, (context.target_id, Stat::Speed,          1));
        }),
        base_accuracy: 100,
        base_power: 0,
        category: MoveCategory::Status,
        max_power_points: 20,
        priority: 0,
        targets: TargetFlags::SELF,
        type_: Type::Dragon,
        event_handlers: EventHandlerDeck::empty,
        event_filtering_options: EventFilteringOptions::default(),
    }
);