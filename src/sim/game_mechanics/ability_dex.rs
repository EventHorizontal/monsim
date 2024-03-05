#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use super::{ability::AbilitySpecies, ElementalType};
use crate::{
    debug_location,
    sim::{event::broadcast_contexts::MoveUsed, EventHandlerDeck, EventFilteringOptions, EventHandler, Outcome, SecondaryAction, DEFAULT_RESPONSE},
};

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    event_handler_deck: EventHandlerDeck {
        on_try_move: Some(EventHandler {
            #[cfg(feature = "debug")]
            dbg_location: debug_location!("FlashFire->on_try_move"),
            callback: |battle,
                       MoveUsed {
                           attacker_uid,
                           move_uid,
                           target_uid,
                       },
                       _relay| {
                let current_move = battle.move_(move_uid);
                let is_current_move_fire_type = (current_move.species.elemental_type == ElementalType::Fire);
                if is_current_move_fire_type {
                    let activation_succeeded = SecondaryAction::activate_ability(battle, target_uid);
                    return !activation_succeeded;
                }
                Outcome::Success
            },
        }),
        ..DEFAULT_RESPONSE
    },
    on_activate: |battle, _owner_uid| {
        battle.message_log.push("Flash Fire activated!".to_string());
    },
    filtering_options: EventFilteringOptions::default(),
    order: 0,
};

pub const WaterAbsorb: AbilitySpecies = AbilitySpecies {
    dex_number: 002,
    name: "Water Absorb",
    event_handler_deck: EventHandlerDeck {
        on_try_move: Some(EventHandler {
            #[cfg(feature = "debug")]
            dbg_location: debug_location!("WaterAbsorb->on_try_move"),
            callback: |battle,
                       MoveUsed {
                           attacker_uid,
                           move_uid,
                           target_uid,
                       },
                       _relay| {
                let current_move = battle.move_(move_uid);
                let is_current_move_water_type = (current_move.species.elemental_type == ElementalType::Water);
                if is_current_move_water_type {
                    let activation_succeeded = SecondaryAction::activate_ability(battle, target_uid);
                    return !activation_succeeded;
                }
                Outcome::Success
            },
        }),
        ..DEFAULT_RESPONSE
    },
    on_activate: |battle, _owner_uid| {
        battle.message_log.push("Water Absorb activated!".to_string());
    },
    filtering_options: EventFilteringOptions::default(),
    order: 0,
};
