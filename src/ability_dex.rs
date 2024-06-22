#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{
    effects, move_,
    sim::{Ability, AbilitySpecies, EventFilteringOptions, MoveUseContext, Type},
    AbilityActivationContext, AbilityDexEntry, AbilityID, EventHandler, EventListener, ModifiableStat, MonsterID, MoveHitContext, PositionRelationFlags,
    StatChangeContext,
};
use monsim_macros::{mon, mov};
use monsim_utils::{not, Outcome};

#[cfg(feature = "debug")]
use monsim::source_code_location;

// Flash Fire in-engine causes all Fire type moves on the user to fail.
pub const FlashFire: AbilitySpecies = AbilitySpecies::from_dex_entry(AbilityDexEntry {
    dex_number: 001,
    name: "Flash Fire",
    event_listener: &FlashFireEventListener,
    order: 0,
});

struct FlashFireEventListener;

impl EventListener<AbilityID> for FlashFireEventListener {
    fn on_try_move_hit_handler(&self) -> Option<EventHandler<MoveHitContext, Outcome, AbilityID, MonsterID>> {
        Some(EventHandler {
            response: |battle, broadcaster_id, receiver_id, ability_id, move_hit_context, _| {
                if mov![move_hit_context.move_used_id].is_type(Type::Fire) && move_hit_context.target_id == receiver_id {
                    let activation_outcome = effects::activate_ability(battle, move_hit_context.target_id, |battle, ability_activation_context| {
                        let ability_owner_name = mon![ability_activation_context.ability_owner_id].name();
                        battle.queue_message(format!["{ability_owner_name}'s Flash Fire activated!"]);
                        Outcome::Success(())
                    });
                    return activation_outcome.opposite();
                }
                Outcome::Success(())
            },
            event_filtering_options: EventFilteringOptions::default(),
        })
    }
}

/// Pickup in-engine does nothing (for now).
pub const Pickup: AbilitySpecies = AbilitySpecies::from_dex_entry(AbilityDexEntry {
    dex_number: 002,
    name: "Pickup",
    event_listener: &PickupEventListener,
    order: 1,
});

struct PickupEventListener;

impl EventListener<AbilityID> for PickupEventListener {}

/// Contrary reverse all stat changes for the user.
pub const Contrary: AbilitySpecies = AbilitySpecies::from_dex_entry(AbilityDexEntry {
    dex_number: 003,
    name: "Contrary",
    event_listener: &ContraryEventListener,
    order: 2,
});

struct ContraryEventListener;

impl EventListener<AbilityID> for ContraryEventListener {
    fn on_modify_stat_change_handler(&self) -> Option<EventHandler<StatChangeContext, i8, AbilityID, MonsterID>> {
        Some(EventHandler {
            response: |battle, broadcaster_id, receiver_id, ability_id, context, _| -> i8 {
                #[cfg(feature = "debug")]
                battle.queue_message(format!["(Contrary reversed {}'s stat changes).", mon![context.affected_monster_id].name()]);
                -context.number_of_stages
            },
            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: PositionRelationFlags::SELF,
                ..EventFilteringOptions::default()
            },
        })
    }
}
