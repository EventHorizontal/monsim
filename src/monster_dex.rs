#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::sim::{MonType, MonsterSpecies, StatSet, DEFAULT_HANDLERS};

pub const Treecko: MonsterSpecies = MonsterSpecies {
    name: "Treecko",
    dex_number: 252,
    primary_type: MonType::Grass,
    secondary_type: MonType::None,
    event_handlers: DEFAULT_HANDLERS,
    base_stats: StatSet::new(40, 45, 35, 65, 55, 70),
};

pub const Torchic: MonsterSpecies = MonsterSpecies {
    name: "Torchic",
    dex_number: 255,
    primary_type: MonType::Fire,
    secondary_type: MonType::None,
    event_handlers: DEFAULT_HANDLERS,
    base_stats: StatSet::new(45, 60, 40, 70, 50, 45),
};

pub const Mudkip: MonsterSpecies = MonsterSpecies {
    name: "Mudkip",
    dex_number: 258,
    primary_type: MonType::Water,
    secondary_type: MonType::None,
    event_handlers: DEFAULT_HANDLERS,
    base_stats: StatSet::new(50, 70, 50, 50, 50, 40),
};

pub const Drifloon: MonsterSpecies = MonsterSpecies {
    name: "Drifloon",
    dex_number: 425,
    primary_type: MonType::Ghost,
    secondary_type: MonType::Flying,
    event_handlers: DEFAULT_HANDLERS,
    base_stats: StatSet::new(90, 50, 34, 60, 44, 70),
};

pub const Drifblim: MonsterSpecies = MonsterSpecies {
    name: "Drifblim",
    dex_number: 426,
    primary_type: MonType::Ghost,
    secondary_type: MonType::Flying,
    event_handlers: DEFAULT_HANDLERS,
    base_stats: StatSet::new(150, 80, 44, 90, 54, 80),
};
