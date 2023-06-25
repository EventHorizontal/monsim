#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use super::{ability::AbilitySpecies, ElementalType};
use crate::{sim::{
    CompositeEventResponder, EventResponderFilters, SecondaryAction, EventResponder,
    DEFAULT_RESPONSE, Outcome,
}, debug_location};

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    composite_event_responder: CompositeEventResponder {
        on_try_move: Some(EventResponder {
            #[cfg(feature = "debug")]
            dbg_location: debug_location!("FlashFire->on_try_move"),
            callback: |battle, move_context, _relay| {
                let current_move = battle.move_(move_context.move_uid);
                let is_current_move_fire_type = (current_move.species.elemental_type == ElementalType::Fire);
                if is_current_move_fire_type {
                    let activation_succeeded =
                        SecondaryAction::activate_ability(battle, move_context.target_uid);
                    return !activation_succeeded;
                }
                Outcome::Success
            },
        }),
        ..DEFAULT_RESPONSE
    },
    on_activate: |battle, _owner_uid| {
        battle.push_message(&"Flash Fire activated!");
    },
    filters: EventResponderFilters::default(),
    order: 0,
};

pub const WaterAbsorb: AbilitySpecies = AbilitySpecies {
    dex_number: 002,
    name: "Water Absorb",
    composite_event_responder: CompositeEventResponder {
        on_try_move: Some(EventResponder {
            #[cfg(feature = "debug")]
            dbg_location: debug_location!("WaterAbsorb->on_try_move"),
            callback: |battle, move_context, _relay| {
                let current_move = battle.move_(move_context.move_uid);
                let is_current_move_water_type = (current_move.species.elemental_type == ElementalType::Water);
                if is_current_move_water_type {
                    let activation_succeeded =
                        SecondaryAction::activate_ability(battle, move_context.target_uid);
                    return !activation_succeeded;

                }
                Outcome::Success
            },
        }),
        ..DEFAULT_RESPONSE
    },
    on_activate: |battle, _owner_uid| {
        battle.push_message(&"Water Absorb activated!");
    },
    filters: EventResponderFilters::default(),
    order: 0,
};
