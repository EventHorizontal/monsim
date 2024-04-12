#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use crate::sim::EventHandlerDeck;

use super::{
    monster::{MonsterSpecies, StatSet},
    Type,
};

pub const Dandyleo: MonsterSpecies = MonsterSpecies {
    name: "Dandyleo",
    dex_number: 252,
    primary_type: Type::Grass,
    secondary_type: None,
    base_stats: StatSet::new(40, 45, 35, 65, 55, 70),
    ..MonsterSpecies::const_default()
};

pub const Squirecoal: MonsterSpecies = MonsterSpecies {
    name: "Squirecoal",
    dex_number: 255,
    primary_type: Type::Fire,
    secondary_type: None,
    base_stats: StatSet::new(45, 60, 40, 70, 50, 45),
    ..MonsterSpecies::const_default()
};

pub const Merkey: MonsterSpecies = MonsterSpecies {
    name: "Merkey",
    dex_number: 258,
    primary_type: Type::Water,
    secondary_type: Some(Type::Bug),
    base_stats: StatSet::new(50, 70, 50, 50, 50, 40),
    ..MonsterSpecies::const_default()
};

pub const Zombler: MonsterSpecies = MonsterSpecies {
    name: "Zombler",
    dex_number: 425,
    primary_type: Type::Ghost,
    secondary_type: Some(Type::Dark),
    base_stats: StatSet::new(90, 50, 34, 60, 44, 71),
    ..MonsterSpecies::const_default()
};