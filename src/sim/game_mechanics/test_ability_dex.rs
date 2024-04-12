#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim_utils::not;

use super::{ability::AbilitySpecies, Type};
use crate::{
    sim::{event::contexts::MoveUsed, EventFilteringOptions, EventHandler, EventHandlerDeck, Outcome, Reaction}, source_code_location, AbilityUID
};

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    event_handler_deck: &EventHandlerDeck {
        on_try_move: Some(EventHandler {
            callback: |battle, MoveUsed { move_user, move_used, target}, _relay| {
                if battle[move_used].is_type(Type::Fire) {
                    let activation_succeeded = Reaction::activate_ability(battle, AbilityUID { owner: target });
                    return not!(activation_succeeded);
                }
                Outcome::Success
            },
            #[cfg(feature = "debug")]
            debugging_information: source_code_location!(),
        }),
        ..EventHandlerDeck::const_default()
    },
    on_activate: |battle, crate::AbilityUseContext { ability_used }| {
        let owner_name = battle[ability_used.owner].name();
        battle.message_log.push(format!["{owner_name}'s Flash Fire activated!"]);
    },
    ..AbilitySpecies::const_default()
};

