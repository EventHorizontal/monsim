#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{
    effects,
    sim::{Battle, EventFilteringOptions, MonsterID, MoveCategory, MoveSpecies, Stat, Type},
    Count, EventHandlerSet, MoveDexEntry, MoveHitContext, Outcome, PositionRelationFlags,
};
use monsim_macros::mon;

use super::status_dex::*;

pub const Tackle: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 001,
    name: "Tackle",
    on_use_effect: effects::deal_calculated_damage,
    base_accuracy: Some(50),
    base_power: 40,
    category: MoveCategory::Physical,
    max_power_points: 35,
    hits_per_target: Count::Fixed(1),
    priority: 0,
    targets: PositionRelationFlags::ANY
        .union(PositionRelationFlags::ADJACENT)
        .union(PositionRelationFlags::OPPONENTS)
        .union(PositionRelationFlags::ALLIES),
    type_: Type::Normal,
});

pub const Scratch: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 002,
    name: "Scratch",
    on_use_effect: effects::deal_calculated_damage,
    base_accuracy: Some(100),
    base_power: 40,
    category: MoveCategory::Physical,
    max_power_points: 35,
    hits_per_target: Count::Fixed(1),
    priority: 0,
    targets: PositionRelationFlags::ANY
        .union(PositionRelationFlags::ADJACENT)
        .union(PositionRelationFlags::OPPONENTS)
        .union(PositionRelationFlags::ALLIES),
    type_: Type::Normal,
});

pub const Ember: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 003,
    name: "Ember",
    on_use_effect: |battle, context| {
        let hit_outcome = effects::deal_calculated_damage(battle, context);
        if battle.roll_chance(1, 10) && hit_outcome.is_success() {
            effects::add_persistent_status(battle, context.target_id, &Burned);
        }
        hit_outcome
    },
    base_accuracy: Some(100),
    base_power: 40,
    category: MoveCategory::Special,
    max_power_points: 35,
    hits_per_target: Count::Fixed(1),
    priority: 0,
    targets: PositionRelationFlags::ANY
        .union(PositionRelationFlags::ADJACENT)
        .union(PositionRelationFlags::OPPONENTS),
    type_: Type::Fire,
});

pub const Bubble: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 004,
    name: "Bubble",
    on_use_effect: effects::deal_calculated_damage,
    base_accuracy: Some(100),
    base_power: 40,
    category: MoveCategory::Special,
    max_power_points: 35,
    hits_per_target: Count::Fixed(1),
    priority: 0,
    targets: PositionRelationFlags::ALL
        .union(PositionRelationFlags::ADJACENT)
        .union(PositionRelationFlags::OPPONENTS),
    type_: Type::Water,
});

pub const Growl: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 005,
    name: "Growl",
    on_use_effect: |sim, context| {
        let stat_lowering_outcome = effects::change_stat(sim, context.target_id, Stat::PhysicalAttack, -1);
        stat_lowering_outcome.empty()
    },
    base_accuracy: Some(100),
    base_power: 0,
    category: MoveCategory::Status,
    max_power_points: 40,
    hits_per_target: Count::Fixed(1),
    priority: 0,
    targets: PositionRelationFlags::ALL
        .union(PositionRelationFlags::ADJACENT)
        .union(PositionRelationFlags::OPPONENTS),
    type_: Type::Normal,
});

pub const DragonDance: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 006,
    name: "Dragon Dance",
    on_use_effect: |sim, context| {
        let first_stat_raise_outcome = effects::change_stat(sim, context.target_id, Stat::PhysicalAttack, 1);
        let second_stat_raise_outcome = effects::change_stat(sim, context.target_id, Stat::Speed, 1);
        first_stat_raise_outcome.empty() & second_stat_raise_outcome.empty()
    },
    base_accuracy: Some(100),
    base_power: 0,
    category: MoveCategory::Status,
    max_power_points: 20,
    hits_per_target: Count::Fixed(1),
    priority: 0,
    targets: PositionRelationFlags::SELF,
    type_: Type::Dragon,
});

pub const BulletSeed: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 007,
    name: "Bullet Seed",
    on_use_effect: effects::deal_calculated_damage,
    hits_per_target: Count::RandomInRange { min: 2, max: 5 },
    base_accuracy: Some(100),
    base_power: 25,
    category: MoveCategory::Physical,
    max_power_points: 20,
    priority: 0,
    targets: PositionRelationFlags::ANY
        .union(PositionRelationFlags::ADJACENT)
        .union(PositionRelationFlags::ALLIES)
        .union(PositionRelationFlags::OPPONENTS),
    type_: Type::Grass,
});

pub const Confusion: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 008,
    name: "Confusion",
    on_use_effect: |battle, context| {
        let hit_outcome = effects::deal_calculated_damage(battle, context);
        if battle.roll_chance(1, 10) && hit_outcome.is_success() {
            effects::add_volatile_status(battle, context.target_id, &Confused);
        }
        hit_outcome
    },
    hits_per_target: Count::Fixed(1),
    base_accuracy: Some(100),
    base_power: 50,
    category: MoveCategory::Special,
    max_power_points: 25,
    priority: 0,
    targets: PositionRelationFlags::ANY
        .union(PositionRelationFlags::ADJACENT)
        .union(PositionRelationFlags::ALLIES)
        .union(PositionRelationFlags::OPPONENTS),
    type_: Type::Psychic,
});

pub const Recycle: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 009,
    name: "Recycle",
    on_use_effect: |battle,
                    MoveHitContext {
                        move_user_id,
                        move_used_id,
                        target_id,
                        number_of_hits,
                    }| {
        let consumed_item = mon![mut move_user_id].consumed_item_mut().take();
        // Recycle only works if there exists a consumed item and no held item.
        if let Some(consumed_item) = consumed_item {
            if mon![mut move_user_id].held_item_mut().is_none() {
                battle.queue_message(format!["Recycle replenished {}'s {}", mon![move_user_id].name(), consumed_item.name()]);
                *mon![mut move_user_id].held_item_mut() = Some(consumed_item);
                *mon![mut move_user_id].consumed_item_mut() = None;
                Outcome::Success(())
            } else {
                Outcome::Failure
            }
        } else {
            Outcome::Failure
        }
    },
    hits_per_target: Count::Fixed(1),
    base_accuracy: Some(100),
    base_power: 0,
    category: MoveCategory::Status,
    max_power_points: 10,
    priority: 0,
    targets: PositionRelationFlags::SELF,
    type_: Type::Normal,
});

pub const Swift: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 010,
    name: "Swift",
    on_use_effect: effects::deal_calculated_damage,
    hits_per_target: Count::Fixed(1),
    base_accuracy: None,
    base_power: 60,
    category: MoveCategory::Special,
    max_power_points: 20,
    priority: 0,
    targets: PositionRelationFlags::ALL
        .union(PositionRelationFlags::ADJACENT)
        .union(PositionRelationFlags::OPPONENTS),
    type_: Type::Normal,
});

pub const ShadowBall: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 011,
    name: "Shadow Ball",
    on_use_effect: |battle, context| {
        let hit_outcome = effects::deal_calculated_damage(battle, context);

        if hit_outcome.is_success() && battle.roll_chance(2, 10) {
            let _ = effects::change_stat(battle, context.target_id, Stat::SpecialDefense, -1);
        }
        hit_outcome
    },
    hits_per_target: Count::Fixed(1),
    base_accuracy: Some(100),
    base_power: 80,
    category: MoveCategory::Special,
    max_power_points: 15,
    priority: 0,
    targets: PositionRelationFlags::ANY
        .union(PositionRelationFlags::ADJACENT)
        .union(PositionRelationFlags::ALLIES)
        .union(PositionRelationFlags::OPPONENTS),
    type_: Type::Ghost,
});
