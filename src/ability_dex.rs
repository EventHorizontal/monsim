#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{
    effects, move_,
    sim::{Ability, AbilitySpecies, EventFilteringOptions, EventHandler, MoveUseContext, Type},
    AbilityActivationContext, AbilityDexEntry, AbilityID, EventHandlerRegistry, ModifiableStat, MoveHitContext, PositionRelationFlags,
};
use monsim_macros::{mon, mov};
use monsim_utils::{not, Outcome};

#[cfg(feature = "debug")]
use monsim::source_code_location;

// Flash Fire in-engine causes all Fire type moves on the user to fail.
pub const FlashFire: AbilitySpecies = AbilitySpecies::from_dex_entry(AbilityDexEntry {
    dex_number: 001,
    name: "Flash Fire",
    bind_event_handlers: |registry| {
        registry.register(
            |registry| &mut registry.on_try_move_hit,
            EventHandler {
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
                               number_of_targets,
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
            },
        );
    },
    order: 0,
});

/// Pickup in-engine does nothing (for now).
pub const Pickup: AbilitySpecies = AbilitySpecies::from_dex_entry(AbilityDexEntry {
    dex_number: 002,
    name: "Pickup",
    bind_event_handlers: |registry| {},
    order: 1,
});

/// Contrary reverse all stat changes for the user.
pub const Contrary: AbilitySpecies = AbilitySpecies::from_dex_entry(AbilityDexEntry {
    dex_number: 003,
    name: "Contrary",
    bind_event_handlers: |registry| {
        registry.register(
            |registry| &mut registry.on_modify_stat_change,
            EventHandler {
                #[cfg(feature = "debug")]
                source_code_location: source_code_location![],
                response: |battle,
                           broadcaster_id,
                           receiver_id,
                           monsim::StatChangeContext {
                               affected_monster_id,
                               stat,
                               number_of_stages,
                           },
                           _|
                 -> i8 {
                    #[cfg(feature = "debug")]
                    battle.queue_message(format!["(Contrary reversed {}'s stat changes).", mon![affected_monster_id].name()]);
                    -number_of_stages
                },
                event_filtering_options: EventFilteringOptions {
                    only_if_broadcaster_is: PositionRelationFlags::SELF,
                    ..EventFilteringOptions::default()
                },
            },
        );
    },
    order: 2,
});
