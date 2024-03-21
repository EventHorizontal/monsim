#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use crate::sim::EventHandlerDeck;

use super::{
    monster::{MonsterSpecies, StatSet},
    Type,
};

pub const Treecko: MonsterSpecies = MonsterSpecies {
    name: "Treecko",
    dex_number: 252,
    primary_type: Type::Grass,
    secondary_type: None,
    base_stats: StatSet::new(40, 45, 35, 65, 55, 70),
    ..MonsterSpecies::const_default()
};

pub const Torchic: MonsterSpecies = MonsterSpecies {
    name: "Torchic",
    dex_number: 255,
    primary_type: Type::Fire,
    secondary_type: None,
    base_stats: StatSet::new(45, 60, 40, 70, 50, 45),
    ..MonsterSpecies::const_default()
};

pub const Mudkip: MonsterSpecies = MonsterSpecies {
    name: "Mudkip",
    dex_number: 258,
    primary_type: Type::Water,
    secondary_type: None,
    base_stats: StatSet::new(50, 70, 50, 50, 50, 40),
    ..MonsterSpecies::const_default()
};

pub const Drifloon: MonsterSpecies = MonsterSpecies {
    name: "Drifloon",
    dex_number: 425,
    primary_type: Type::Ghost,
    secondary_type: Some(Type::Flying),
    base_stats: StatSet::new(90, 50, 34, 60, 44, 70),
    ..MonsterSpecies::const_default()
};

pub const Drifblim: MonsterSpecies = MonsterSpecies {
    name: "Drifblim",
    dex_number: 426,
    primary_type: Type::Ghost,
    secondary_type: Some(Type::Flying),
    base_stats: StatSet::new(150, 80, 44, 90, 54, 80),
    ..MonsterSpecies::const_default()
};
