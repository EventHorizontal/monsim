#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::sim::{
    define_ability, Ability, AbilitySpecies, SpecificEventResponder, EventResponderFilters, EventResponder,
    MonType, SecondaryAction, DEFAULT_RESPONSE, FAILURE, SUCCESS,
};

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    event_responder: EventResponder {
        on_try_move: Some(SpecificEventResponder {
            #[cfg(feature = "debug")]
            dbg_location: monsim::debug_location!(stringify![FlashFire.on_try_move]),
            callback: |battle, prng, owner_uid, _relay| {
                let current_move = *battle.get_current_action_as_move().expect(
                    "The current action should be a move within on_try_move responder context.",
                );
                let is_current_move_fire_type = (current_move.species.type_ == MonType::Fire);
                if is_current_move_fire_type {
                    let activation_succeeded =
                        SecondaryAction::activate_ability(battle, prng, owner_uid);
                    if activation_succeeded {
                        return FAILURE;
                    }
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

define_ability!(
    002 WaterAbsorb = "Water Absorb" {
        {
            on_try_move: |battle, prng, owner_uid, _relay| {
                let current_move = *battle.get_current_action_as_move()
                .expect("The current action should be a move within on_try_move responder context.");
                let is_current_move_fire_type = current_move.species.type_ == MonType::Water;
                if is_current_move_fire_type
                {
                    let activation_succeeded = SecondaryAction::activate_ability(battle, prng, owner_uid);
                    if activation_succeeded { return FAILURE; }
                }
                SUCCESS
            },
        },
        on_activate: |battle, _owner_uid| {
            battle.push_message(&"Flash Fire activated!");
        },
        filters: DEFAULT,
        order: 0,
    }
);
