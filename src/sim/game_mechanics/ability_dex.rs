#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use super::{ability::AbilitySpecies, ElementalType};
use crate::sim::{
    CompositeEventResponder, EventResponderFilters, SecondaryAction, EventResponder,
    DEFAULT_RESPONSE, Outcome,
};

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    composite_event_responder: CompositeEventResponder {
        on_try_move: Some(EventResponder {
            #[cfg(feature = "debug")]
            dbg_location: crate::debug_location!("FlashFire.OnTryMove"),
            callback: |battle, owner_uid, _relay| {
                let current_move = *battle.get_current_action_as_move().expect(
                    "The current action should be a move within on_try_move responder context.",
                );
                if current_move.species.elemental_type == ElementalType::Fire
                    && SecondaryAction::activate_ability(battle, owner_uid) == Outcome::Success
                {
                    return Outcome::Failure;
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
