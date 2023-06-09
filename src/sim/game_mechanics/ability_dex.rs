#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use super::{ability::AbilitySpecies, MonType};
use crate::sim::{
    EventResponderFilters, EventResponder, DEFAULT_RESPONSE,
    SpecificEventResponder, SecondaryAction, FAILURE, SUCCESS,
};

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    event_responder: EventResponder {
        on_try_move: Some(SpecificEventResponder {
            #[cfg(feature = "debug")]
            dbg_location: crate::debug_location!("FlashFire.OnTryMove"),
            callback: |battle, prng, owner_uid, _relay| {
                let current_move = *battle.get_current_action_as_move().expect(
                    "The current action should be a move within on_try_move responder context.",
                );
                if current_move.species.type_ == MonType::Fire
                    && SecondaryAction::activate_ability(battle, prng, owner_uid)
                {
                    return FAILURE;
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
