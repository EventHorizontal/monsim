#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim_macros::{mon, mov};
use monsim_utils::{not, NOTHING};

use super::{ability::AbilitySpecies, Type};
use crate::{
    effects, sim::{event_dispatcher::{contexts::MoveUseContext, EventFilteringOptions, EventHandler, EventHandlerDeck}, AbilityDexEntry, AbilityID, AbilityUseContext, BattleSimulator, MoveHitContext}, Outcome, source_code_location
};

pub const FlashFire: AbilitySpecies = AbilitySpecies::from_dex_entry( 
    AbilityDexEntry {
        dex_number: 001,
        name: "Flash Fire",
        event_handlers: | | {
            EventHandlerDeck {
                on_try_move_hit: Some(EventHandler { 
                    #[cfg(feature = "debug")]
                    source_code_location: source_code_location![],
                    response: |sim, broadcaster_id, receiver_id, MoveHitContext { move_user_id, move_used_id, target_id}, _| {
                        if mov![move_used_id].is_type(Type::Fire) && target_id == receiver_id {
                            let activation_outcome = effects::activate_ability(sim, AbilityUseContext::from_owner(receiver_id), |sim, AbilityUseContext { ability_owner_id, ability_used_id }| {
                                let ability_owner_name = mon![ability_owner_id].name();
                                sim.push_message(format!["{ability_owner_name}'s Flash Fire activated!"]);
                                Outcome::Success(())
                            });
                            return activation_outcome.opposite();
                        }
                        Outcome::Success(())
                    },
                    event_filtering_options: EventFilteringOptions::default(),
                }),
                ..EventHandlerDeck::empty()
            }
        },
        order: 0,
    }
);