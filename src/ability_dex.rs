#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::sim::{
    Ability, AbilitySpecies, CompositeEventResponder, EventResponderFilters, MonType,
    SecondaryAction, EventResponder, DEFAULT_RESPONSE, FAILURE, SUCCESS,
};

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    composite_event_responder: CompositeEventResponder {
        on_try_move: Some(EventResponder {
            #[cfg(feature = "debug")]
            dbg_location: monsim::debug_location!("FlashFire.on_try_move"),
            callback: |battle, prng, owner_uid, _relay| {
                let current_move = *battle.get_current_action_as_move()
                    .expect("The current action should be a move within on_try_move responder context.");
                let is_current_move_fire_type = (current_move.species.type_ == MonType::Fire);
                if is_current_move_fire_type {
                    let activation_succeeded =
                        SecondaryAction::activate_ability(battle, prng, owner_uid);
                    if activation_succeeded { return FAILURE; }
                }
                SUCCESS
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
            callback: |battle, prng, owner_uid, _relay| {
                let current_move = *battle.get_current_action_as_move()
                    .expect("The current action should be a move within on_try_move responder context.");
                let is_current_move_water_type = (current_move.species.type_ == MonType::Water);
                if is_current_move_water_type {
                    let activation_succeeded =
                        SecondaryAction::activate_ability(battle, prng, owner_uid);
                    if activation_succeeded { return FAILURE; }
                }
                SUCCESS
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