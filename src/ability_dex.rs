#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{
    effects, move_,
    sim::{Ability, AbilitySpecies, EventFilteringOptions, EventHandler, EventHandlerSet, MoveUseContext, Type},
    AbilityActivationContext, AbilityDexEntry, AbilityID, MoveHitContext, Stat,
};
use monsim_macros::{mon, mov};
use monsim_utils::{not, Outcome};

#[cfg(feature = "debug")]
use monsim::source_code_location;

pub const FlashFire: AbilitySpecies = AbilitySpecies::from_dex_entry(AbilityDexEntry {
    dex_number: 001,
    name: "Flash Fire",
    event_handlers: || EventHandlerSet {
        on_try_move_hit: Some(EventHandler {
            #[cfg(feature = "debug")]
            source_code_location: source_code_location![],
            response: |battle,
                       broadcaster_id,
                       receiver_id,
                       MoveHitContext {
                           move_user_id,
                           move_used_id,
                           target_id,
                           number_of_hits,
                       },
                       _| {
                if mov![move_used_id].is_type(Type::Fire) && target_id == receiver_id {
                    let activation_outcome = effects::activate_ability(
                        battle,
                        receiver_id,
                        |battle,
                         AbilityActivationContext {
                             ability_owner_id,
                             ability_used_id,
                         }| {
                            let ability_owner_name = mon![ability_owner_id].name();
                            battle.queue_message(format!["{ability_owner_name}'s Flash Fire activated!"]);
                            Outcome::Success(())
                        },
                    );
                    return activation_outcome.opposite();
                }
                Outcome::Success(())
            },
            event_filtering_options: EventFilteringOptions::default(),
        }),
        ..EventHandlerSet::empty()
    },
    order: 0,
});

pub const Spiteful: AbilitySpecies = AbilitySpecies::from_dex_entry(AbilityDexEntry {
    dex_number: 002,
    name: "Spiteful",
    event_handlers: || EventHandlerSet {
        on_damaging_move_used: Some(EventHandler {
            #[cfg(feature = "debug")]
            source_code_location: source_code_location!(),
            response: |battle,
                       broadcaster_id,
                       receiver_id,
                       MoveUseContext {
                           move_user_id,
                           move_used_id,
                           target_ids,
                       },
                       _| {
                let _ = effects::activate_ability(
                    battle,
                    receiver_id,
                    |battle,
                     AbilityActivationContext {
                         ability_owner_id,
                         ability_used_id,
                     }| {
                        let stat_raise_outcome = effects::raise_stat(battle, ability_owner_id, Stat::PhysicalAttack, 3);
                        stat_raise_outcome
                    },
                );
            },
            event_filtering_options: EventFilteringOptions::default(),
        }),
        ..EventHandlerSet::empty()
    },
    order: 1,
});
