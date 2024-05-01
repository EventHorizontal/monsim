#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{sim::{MonsterSpecies, StatSet, Type}, EventHandlerDeck, MonsterDexEntry};

pub const Dandyleo: MonsterSpecies = MonsterSpecies::from_dex_entry( 
    MonsterDexEntry {
        dex_number: 001,
        name: "Dandyleo",
        primary_type: Type::Grass,
        secondary_type: None,
        base_stats: StatSet::new(40, 45, 35, 65, 55, 70),
        event_handlers: EventHandlerDeck::empty,
    }
);

pub const Squirecoal: MonsterSpecies = MonsterSpecies::from_dex_entry( 
    MonsterDexEntry {
        dex_number: 003,
        name: "Squirecoal",
        primary_type: Type::Fire,
        secondary_type: None,
        base_stats: StatSet::new(45, 60, 40, 70, 50, 45),
        event_handlers: EventHandlerDeck::empty,
    }
);


pub const Merkey: MonsterSpecies = MonsterSpecies::from_dex_entry( 
    MonsterDexEntry {
        dex_number: 009,
        name: "Merkey",
        primary_type: Type::Water,
        secondary_type: Some(Type::Bug),
        base_stats: StatSet::new(50, 70, 50, 50, 50, 40),
        event_handlers: EventHandlerDeck::empty,
    }
);

pub const Zombler: MonsterSpecies = MonsterSpecies::from_dex_entry(
    MonsterDexEntry {
        dex_number: 045,
        name: "Zombler",
        primary_type: Type::Ghost,
        secondary_type: Some(Type::Dark),
        base_stats: StatSet::new(90, 50, 34, 60, 44, 71),
        event_handlers: EventHandlerDeck::empty,
    }
);