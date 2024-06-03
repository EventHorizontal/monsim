#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{
    effects,
    sim::{BattleState, EventFilteringOptions, MonsterID, MoveCategory, MoveSpecies, Stat, Type},
    Count, EventHandlerSet, MoveDexEntry, MoveHitContext, Outcome, TargetFlags,
};
use monsim_macros::mon;

use super::status_dex::*;

pub const Tackle: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 001,
    name: "Tackle",
    on_hit_effect: effects::deal_default_damage,
    base_accuracy: Some(100),
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
});

pub const Scratch: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 002,
    name: "Scratch",
    on_hit_effect: effects::deal_default_damage,
    base_accuracy: Some(100),
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
});

pub const Ember: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 003,
    name: "Ember",
    on_hit_effect: |battle, context| {
        let hit_outcome = effects::deal_default_damage(battle, context);
        if battle.roll_chance(1, 10) && hit_outcome.succeeded() {
            effects::add_persistent_status(battle, (context.target_id, &Burned));
        }
        hit_outcome
    },
    base_accuracy: Some(100),
    base_power: 40,
    category: MoveCategory::Special,
    max_power_points: 35,
    hits_per_target: Count::Fixed(1),
    priority: 0,
    targets: TargetFlags::ANY.union(TargetFlags::ADJACENT).union(TargetFlags::OPPONENTS),
    type_: Type::Fire,
});

pub const Bubble: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 004,
    name: "Bubble",
    on_hit_effect: effects::deal_default_damage,
    base_accuracy: Some(100),
    base_power: 40,
    category: MoveCategory::Special,
    max_power_points: 35,
    hits_per_target: Count::Fixed(1),
    priority: 0,
    targets: TargetFlags::ALL.union(TargetFlags::ADJACENT).union(TargetFlags::OPPONENTS),
    type_: Type::Water,
});

pub const Growl: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 005,
    name: "Growl",
    on_hit_effect: |sim, context| {
        let stat_lowering_succeeded = effects::lower_stat(sim, (context.target_id, Stat::PhysicalAttack, 1));
        stat_lowering_succeeded
    },
    base_accuracy: Some(100),
    base_power: 0,
    category: MoveCategory::Status,
    max_power_points: 40,
    hits_per_target: Count::Fixed(1),
    priority: 0,
    targets: TargetFlags::ALL.union(TargetFlags::ADJACENT).union(TargetFlags::OPPONENTS),
    type_: Type::Normal,
});

pub const DragonDance: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 006,
    name: "Dragon Dance",
    on_hit_effect: |sim, context| {
        let first_stat_raise_succeeded = effects::raise_stat(sim, (context.target_id, Stat::PhysicalAttack, 1));
        let second_stat_raise_succeeded = effects::raise_stat(sim, (context.target_id, Stat::Speed, 1));
        first_stat_raise_succeeded & second_stat_raise_succeeded
    },
    base_accuracy: Some(100),
    base_power: 0,
    category: MoveCategory::Status,
    max_power_points: 20,
    hits_per_target: Count::Fixed(1),
    priority: 0,
    targets: TargetFlags::SELF,
    type_: Type::Dragon,
});

pub const BulletSeed: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 007,
    name: "Bullet Seed",
    on_hit_effect: effects::deal_default_damage,
    hits_per_target: Count::RandomInRange { min: 2, max: 5 },
    base_accuracy: Some(100),
    base_power: 25,
    category: MoveCategory::Physical,
    max_power_points: 20,
    priority: 0,
    targets: TargetFlags::ANY
        .union(TargetFlags::ADJACENT)
        .union(TargetFlags::ALLIES)
        .union(TargetFlags::OPPONENTS),
    type_: Type::Grass,
});

pub const Confusion: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 008,
    name: "Confusion",
    on_hit_effect: |battle, context| {
        let hit_outcome = effects::deal_default_damage(battle, context);
        if battle.roll_chance(1, 10) && hit_outcome.succeeded() {
            effects::add_volatile_status(battle, (context.target_id, &Confused));
        }
        hit_outcome
    },
    hits_per_target: Count::Fixed(1),
    base_accuracy: Some(100),
    base_power: 50,
    category: MoveCategory::Special,
    max_power_points: 25,
    priority: 0,
    targets: TargetFlags::ANY
        .union(TargetFlags::ADJACENT)
        .union(TargetFlags::ALLIES)
        .union(TargetFlags::OPPONENTS),
    type_: Type::Psychic,
});

pub const Recycle: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 009,
    name: "Recycle",
    on_hit_effect: |battle,
                    MoveHitContext {
                        move_user_id,
                        move_used_id,
                        target_id,
                    }| {
        let consumed_item = mon![mut move_user_id].consumed_item_mut().take();
        // Recycle only works if there exists a consumed item and no held item.
        if let Some(consumed_item) = consumed_item {
            if mon![mut move_user_id].held_item_mut().is_none() {
                battle.queue_message(format![
                    "Recycle replenished {}'s {}",
                    battle.monster(move_user_id).name(),
                    consumed_item.name()
                ]);
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
    targets: TargetFlags::SELF,
    type_: Type::Normal,
});

pub const Swift: MoveSpecies = MoveSpecies::from_dex_entry(MoveDexEntry {
    dex_number: 010,
    name: "Swift",
    on_hit_effect: effects::deal_default_damage,
    hits_per_target: Count::Fixed(1),
    base_accuracy: None,
    base_power: 60,
    category: MoveCategory::Special,
    max_power_points: 20,
    priority: 0,
    targets: TargetFlags::ALL.union(TargetFlags::ADJACENT).union(TargetFlags::OPPONENTS),
    type_: Type::Normal,
});
