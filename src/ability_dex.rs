#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim_macros::{mon, mov};
use monsim_utils::{not, Outcome};
use monsim::{effects::*, event_dex::*, move_, sim::{
        Ability, AbilitySpecies, EventFilteringOptions, EventHandler, EventHandlerDeck, MoveUseContext, Type
}, AbilityDexEntry, AbilityID, AbilityUseContext};

#[cfg(feature = "debug")]
use monsim::source_code_location;
use tap::Pipe;

pub const FlashFire: AbilitySpecies = AbilitySpecies::from_dex_data( 
    AbilityDexEntry {
        dex_number: 001,
        name: "Flash Fire",
        event_handlers: | | {
            EventHandlerDeck::empty()
                .add(OnTryMove, |sim, MoveUseContext { move_user_id, move_used_id, target_id: target}| {
                    if mov![move_used_id].is_type(Type::Fire) {
                        let activation_succeeded = ActivateAbility(sim, AbilityUseContext::new(target));
                        return not!(activation_succeeded);
                    }
                    Outcome::Success
                }, source_code_location!())
        },
        on_activate_effect: Effect::from(|sim, AbilityUseContext { ability_used_id, ability_owner_id }| {
            let owner_name = mon![ability_used_id.owner_id].name();
            sim.push_message(format!["{owner_name}'s Flash Fire activated!"]);
        }),
        event_filtering_options: EventFilteringOptions::default(),
        order: 0,
    }
);