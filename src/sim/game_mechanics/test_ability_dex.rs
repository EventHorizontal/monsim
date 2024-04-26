#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim_macros::{mon, mov};
use monsim_utils::not;

use super::{ability::AbilitySpecies, Type};
use crate::{
    effects::*, event_dex::OnTryMove, sim::{event_dispatch::contexts::MoveUseContext, EventFilteringOptions, EventHandler, EventHandlerDeck, Outcome}, source_code_location, AbilityDexEntry, AbilityUID, AbilityUseContext
};

pub const FlashFire: AbilitySpecies = AbilitySpecies::from_dex_data( 
    AbilityDexEntry {
        dex_number: 001,
        name: "Flash Fire",
        event_handlers: | | {
            EventHandlerDeck::empty()
                .add(OnTryMove, |sim, MoveUseContext { move_user, move_used, target}| {
                    if mov![move_used].is_type(Type::Fire) {
                        let activation_succeeded = ActivateAbility(sim, AbilityUseContext::new(target));
                        return not!(activation_succeeded);
                    }
                    Outcome::Success
                }, source_code_location!())
        },
        on_activate_effect: Effect::from(|sim, AbilityUseContext { ability_used, ability_owner }| {
            let owner_name = mon![ability_used.owner].name();
            sim.push_message(format!["{owner_name}'s Flash Fire activated!"]);
        }),
        event_filtering_options: EventFilteringOptions::default(),
        order: 0,
    }
);