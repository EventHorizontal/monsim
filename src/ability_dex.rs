#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::sim::{
        Ability, AbilitySpecies, EventHandlerDeck, ElementalType, EventFilterOptions, EventHandler, MoveUsed, SecondaryAction,
        DEFAULT_RESPONSE,
        utils::{Outcome, not},
};

#[cfg(feature = "debug")]
use monsim::debug_location;

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
                    return not!(activation_succeeded);
                }
                Outcome::Success
            },
        }),
        ..DEFAULT_RESPONSE
    },
    on_activate: |battle, _owner_uid| {
        battle.message_log.push("Flash Fire activated!".to_string());
    },
    filters: EventFilterOptions::default(),
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
                    return not!(activation_succeeded);
                }
                Outcome::Success
            },
        }),
        ..DEFAULT_RESPONSE
    },
    on_activate: |battle, owner_uid| {
        let owner_name = battle.monster(owner_uid).name();
        battle.message_log.push(format!["{owner_name}'s Water Absorb activated!"]);
    },
    filters: EventFilterOptions::default(),
    order: 0,
};
