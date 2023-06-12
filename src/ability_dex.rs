#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{sim::{
    Ability, AbilitySpecies, CompositeEventResponder, EventResponderFilters, ElementalType,
    SecondaryAction, EventResponder, DEFAULT_RESPONSE, Outcome,
}, not};

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    composite_event_responder: CompositeEventResponder {
        on_try_move: Some(EventResponder {
            #[cfg(feature = "debug")]
            dbg_location: monsim::debug_location!("FlashFire.on_try_move"),
            callback: |battle, owner_uid, _relay| {
                let current_move = *battle.get_current_action_as_move()
                    .expect("The current action should be a move within on_try_move responder context.");
                let is_current_move_fire_type = (current_move.species.elemental_type == ElementalType::Fire);
                if is_current_move_fire_type {
                    let activation_succeeded =
                        SecondaryAction::activate_ability(battle, owner_uid);
                    return not!(activation_succeeded);
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
            dbg_location: monsim::debug_location!("WaterAbsorb.on_try_move"),
            callback: |battle, owner_uid, _relay| {
                let current_move = *battle.get_current_action_as_move()
                    .expect("The current action should be a move within on_try_move responder context.");
                let is_current_move_water_type = (current_move.species.elemental_type == ElementalType::Water);
                if is_current_move_water_type {
                    let activation_succeeded =
                        SecondaryAction::activate_ability(battle, owner_uid);
                    return not!(activation_succeeded);

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