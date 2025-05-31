#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{
    effects,
    sim::{MonsterSpecies, StatSet, Type},
    EventFilteringOptions, EventHandler, EventListener, MonsterDexEntry, MonsterForm, MonsterID, Nothing, NullEventListener, PositionRelationFlags,
};
use monsim_macros::mon;

use crate::ability_dex::*;

pub const Dandyleo: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 001,
    name: "Dandyleo",
    primary_type: Type::Grass,
    secondary_type: None,
    // TODO: Not sure yet if we should have this field.
    form_name: None,
    base_stats: StatSet::new(40, 45, 35, 65, 55, 70),
    allowed_abilities: (&Pickup, None, None),
    event_listener: &NullEventListener,
});

pub const Squirecoal: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 003,
    name: "Squirecoal",
    form_name: None,
    primary_type: Type::Fire,
    secondary_type: None,
    allowed_abilities: (&Pickup, Some(&FlashFire), None),
    base_stats: StatSet::new(45, 60, 40, 70, 50, 45),
    event_listener: &NullEventListener,
});

pub const Merkey: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 009,
    name: "Merkey",
    form_name: None,
    primary_type: Type::Water,
    secondary_type: Some(Type::Bug),
    allowed_abilities: (&Pickup, None, None),
    base_stats: StatSet::new(50, 70, 50, 50, 50, 40),
    event_listener: &NullEventListener,
});

pub const Zombler: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 045,
    name: "Zombler",
    primary_type: Type::Ghost,
    form_name: None,
    secondary_type: Some(Type::Dark),
    allowed_abilities: (&Contrary, None, None),
    base_stats: StatSet::new(90, 50, 34, 60, 44, 71),
    event_listener: &NullEventListener,
});

pub const MonstrossiveFullForm: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 047,
    name: "Monstrossive",
    form_name: Some("Full"),
    primary_type: Type::Ghost,
    secondary_type: None,
    allowed_abilities: (&Zombie, None, None),
    base_stats: StatSet::new(100, 110, 90, 81, 20, 55),
    event_listener: &NullEventListener,
});

pub const MonstrossiveHungryForm: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 047,
    name: "Monstrossive",
    form_name: Some("Hungry"),
    primary_type: Type::Ghost,
    secondary_type: Some(Type::Dark),
    allowed_abilities: (&Zombie, None, None),
    base_stats: StatSet::new(100, 90, 10, 81, 20, 155),
    event_listener: &NullEventListener,
});
