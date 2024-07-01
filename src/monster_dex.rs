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
    base_stats: StatSet::new(40, 45, 35, 65, 55, 70),
    allowed_abilities: (&Pickup, None, None),
    event_handlers: &NullEventListener,
});

pub const Squirecoal: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 003,
    name: "Squirecoal",
    primary_type: Type::Fire,
    secondary_type: None,
    allowed_abilities: (&Pickup, Some(&FlashFire), None),
    base_stats: StatSet::new(45, 60, 40, 70, 50, 45),
    event_handlers: &NullEventListener,
});

pub const Merkey: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 009,
    name: "Merkey",
    primary_type: Type::Water,
    secondary_type: Some(Type::Bug),
    allowed_abilities: (&Pickup, None, None),
    base_stats: StatSet::new(50, 70, 50, 50, 50, 40),
    event_handlers: &NullEventListener,
});

pub const Zombler: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 045,
    name: "Zombler",
    primary_type: Type::Ghost,
    secondary_type: Some(Type::Dark),
    allowed_abilities: (&Contrary, None, None),
    base_stats: StatSet::new(90, 50, 34, 60, 44, 71),
    event_handlers: &NullEventListener,
});

pub const Monstrossive: MonsterSpecies = MonsterSpecies::from_dex_entry(MonsterDexEntry {
    dex_number: 047,
    name: "Monstrossive",
    primary_type: Type::Ghost,
    secondary_type: None,
    allowed_abilities: (&Contrary, None, None),
    base_stats: StatSet::new(100, 110, 90, 81, 20, 55),
    event_handlers: &MonstrossiveEventListener,
});

struct MonstrossiveEventListener;

impl EventListener<MonsterID> for MonstrossiveEventListener {
    fn on_damage_dealt_handler(&self) -> Option<EventHandler<Nothing, Nothing, MonsterID, MonsterID>> {
        Some(EventHandler {
            response: |battle, broadcaster_id, receiver_id, self_id, context, relay| {
                if mon![self_id].current_health() <= mon![self_id].max_health() / 2 {
                    effects::change_form(battle, self_id, &MonstrossiveHungryForm);
                }
            },
            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: PositionRelationFlags::SELF,
                ..EventFilteringOptions::default()
            },
        })
    }
}

pub const MonstrossiveHungryForm: MonsterForm = MonsterForm {
    dex_number: 047,
    name: "Hungry Form",
    primary_type: Type::Ghost,
    secondary_type: Some(Type::Dark),
    ability: Some(&Zombie),
    // TODO: ModifiableStatSet.
    base_stats: StatSet::new(100, 90, 10, 81, 20, 155),
    event_handlers: &NullEventListener,
};
