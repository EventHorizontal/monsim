#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{
    sim::{MonsterSpecies, StatSet, Type},
    MonsterDexEntry, NullEventListener,
};

use crate::{Contrary, Pickup};

pub const Dandyleo: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 001,
    name: "Dandyleo",
    primary_type: Type::Grass,
    secondary_type: None,
    base_stats: StatSet::new(40, 45, 35, 65, 55, 70),
    abilities: (&Pickup, None, None),
    event_handlers: &NullEventListener,
});

pub const Squirecoal: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 003,
    name: "Squirecoal",
    primary_type: Type::Fire,
    secondary_type: None,
    abilities: (&Pickup, None, None),
    base_stats: StatSet::new(45, 60, 40, 70, 50, 45),
    event_handlers: &NullEventListener,
});

pub const Merkey: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 009,
    name: "Merkey",
    primary_type: Type::Water,
    secondary_type: Some(Type::Bug),
    abilities: (&Pickup, None, None),
    base_stats: StatSet::new(50, 70, 50, 50, 50, 40),
    event_handlers: &NullEventListener,
});

pub const Zombler: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 045,
    name: "Zombler",
    primary_type: Type::Ghost,
    secondary_type: Some(Type::Dark),
    abilities: (&Pickup, None, None),
    base_stats: StatSet::new(90, 50, 34, 60, 44, 71),
    event_handlers: &NullEventListener,
});

pub const Monstrossive: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 047,
    name: "Monstrossive",
    primary_type: Type::Ghost,
    secondary_type: Some(Type::Dark),
    abilities: (&Contrary, None, None),
    base_stats: StatSet::new(100, 110, 90, 81, 20, 55),
    event_handlers: &NullEventListener,
});
