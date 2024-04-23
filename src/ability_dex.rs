#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim_utils::{not, Outcome};
use monsim::{event_dex::*, move_, sim::{
        Ability, AbilitySpecies, EventFilteringOptions, EventHandler, EventHandlerDeck, MoveUseContext, Type
}, AbilityUID, AbilityUseContext, actions::*};

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
                effect: Effect::from(|sim, MoveUseContext { move_user, move_used, target}| {
                    if sim[move_used].is_type(Type::Fire) {
                        let activation_succeeded = ActivateAbility(sim, AbilityUseContext::new(target));
                        return not!(activation_succeeded);
                    }
                    Outcome::Success
                }),
                #[cfg(feature = "debug")]
                source_code_location: source_code_location!(),
            }),
            ..EventHandlerDeck::empty()
        }
    },
    on_activate: |sim, AbilityUseContext { ability_used, ability_owner }| {
        let owner_name = sim[ability_used.owner].name();
        sim.push_message(format!["{owner_name}'s Flash Fire activated!"]);
    },
    ..AbilitySpecies::const_default()
};