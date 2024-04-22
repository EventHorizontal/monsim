#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim_utils::{not, Outcome};
use monsim::{event_dex::*, move_, sim::{
        Ability, AbilitySpecies, EventFilteringOptions, EventHandler, EventHandlerDeck, MoveUseContext, Reaction, Type
}, AbilityUID, AbilityUseContext};

#[cfg(feature = "debug")]
use monsim::source_code_location;

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
                #[cfg(feature = "debug")]
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