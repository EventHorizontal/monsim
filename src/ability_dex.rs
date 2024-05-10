#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim_macros::{mon, mov};
use monsim_utils::{not, Outcome};
use monsim::{effects::*, event_dex::*, move_, sim::{
        Ability, AbilitySpecies, EventFilteringOptions, EventHandler, EventHandlerDeck, MoveUseContext, Type
}, AbilityDexEntry, AbilityID, AbilityUseContext, MoveHitContext};

#[cfg(feature = "debug")]
use monsim::source_code_location;
use tap::Pipe;

pub const FlashFire: AbilitySpecies = AbilitySpecies::from_dex_data( 
    AbilityDexEntry {
        dex_number: 001,
        name: "Flash Fire",
        event_handlers: | | {
            // HACK: Keeping this here until I think of better long-term solution.
            #[cfg(feature="debug")]
            let out = EventHandlerDeck::empty()
                .add(OnTryMoveHit, |sim, effector_id, MoveHitContext { move_user_id, move_used_id, target_id}| {
                    if mov![move_used_id].is_type(Type::Fire) {
                        let activation_succeeded = ActivateAbility(sim, effector_id,AbilityUseContext::new(effector_id));
                        return not!(activation_succeeded);
                    }
                    Outcome::Success
                }, source_code_location!());
            
            #[cfg(not(feature="debug"))]
            let out = EventHandlerDeck::empty()
                .add(OnTryMoveHit, |sim, effector_id, MoveHitContext { move_user_id, move_used_id, target_id}| {
                    if mov![move_used_id].is_type(Type::Fire) {
                        let activation_succeeded = ActivateAbility(sim, effector_id, AbilityUseContext::new(target_id));
                        return not!(activation_succeeded);
                    }
                    Outcome::Success
                });
            out
        },
        on_activate_effect: Effect::from(|sim, effector_id, AbilityUseContext { ability_used_id, ability_owner_id }| {
            let effector_name = mon![effector_id].name();
            sim.push_message(format!["{effector_name}'s Flash Fire activated!"]);
        }),
        event_filtering_options: EventFilteringOptions::default(),
        order: 0,
    }
);