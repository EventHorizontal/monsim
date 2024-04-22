#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim_utils::not;

use super::{ability::AbilitySpecies, Type};
use crate::{
    event_dex::OnTryMove, sim::{event_dispatch::contexts::MoveUseContext, EventFilteringOptions, EventHandler, EventHandlerDeck, Outcome, Reaction}, source_code_location, AbilityUID, AbilityUseContext
};

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    event_handlers: | | {
        // TODO: builder syntax?
        EventHandlerDeck {
            on_try_move: Some(EventHandler {
                event: OnTryMove,
                callback: |sim, MoveUseContext { move_user, move_used, target}, _relay| {
                    if sim[move_used].is_type(Type::Fire) {
                        let activation_succeeded = Reaction::activate_ability(sim, target);
                        return not!(activation_succeeded);
                    }
                    Outcome::Success
                },
                debugging_information: source_code_location!(),
            }),
            ..EventHandlerDeck::empty()
        }
    },
    on_activate: |sim, AbilityUseContext { ability_used }| {
        let owner_name = sim[ability_used.owner].name();
        sim.push_message(format!["{owner_name}'s Flash Fire activated!"]);
    },
    ..AbilitySpecies::const_default()
};