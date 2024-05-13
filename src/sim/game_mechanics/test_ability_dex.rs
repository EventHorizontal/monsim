#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim_macros::{mon, mov};
use monsim_utils::not;

use super::{ability::AbilitySpecies, Type};
use crate::{
    effects, sim::{event_dispatch::contexts::MoveUseContext, EventFilteringOptions, EventHandler, EventHandlerDeck, Outcome}, source_code_location, AbilityDexEntry, AbilityID, AbilityUseContext, BattleSimulator, EventID, MoveHitContext
};

pub const FlashFire: AbilitySpecies = AbilitySpecies::from_dex_data( 
    AbilityDexEntry {
        dex_number: 001,
        name: "Flash Fire",
        event_handlers: | | {
            EventHandlerDeck {
                on_try_move_hit: Some(EventHandler { 
                    #[cfg(feature = "debug")]
                    source_code_location: source_code_location![],
                    effect: |sim, effector_id, MoveHitContext { move_user_id, move_used_id, target_id}| {
                        if mov![move_used_id].is_type(Type::Fire) {
                            let activation_succeeded = effects::activate_ability(sim, effector_id,AbilityUseContext::new(effector_id));
                            return not!(activation_succeeded);
                        }
                        Outcome::Success
                    },
                }),
                ..EventHandlerDeck::empty()
            }
        },
        on_activate_effect: |sim, effector_id, AbilityUseContext { ability_used_id, ability_owner_id }| {
            let effector_name = mon![effector_id].name();
            sim.push_message(format!["{effector_name}'s Flash Fire activated!"]);
        },
        event_filtering_options: EventFilteringOptions::default(),
        order: 0,
    }
);