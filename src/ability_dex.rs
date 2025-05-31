#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{
    effects, move_,
    sim::{Ability, AbilitySpecies, EventFilteringOptions, MoveUseContext, Type},
    AbilityActivationContext, AbilityDexEntry, AbilityID, EventHandler, EventListener, ModifiableStat, MonsterID, MoveHitContext, Nothing,
    PositionRelationFlags, StatChangeContext, NOTHING,
};
use monsim_macros::{mon, mov};
use monsim_utils::{not, Outcome};

#[cfg(feature = "debug")]
use monsim::source_code_location;

use crate::status_dex::FlashFireStatus;

/// Flash Fire in-engine adds the FlashFireStatus to the user which boosts its attack by 50% when the move it uses is Fire type.
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
                        if let Some(flash_fire) = mon![ability_activation_context.ability_owner_id].volatile_status(FlashFireStatus) {
                            Outcome::Success(NOTHING)
                        } else {
                            battle.queue_debug_message(format!["(Flash Fire added FlashFireStatus to {})", ability_owner_name]);
                            let inflict_status_outcome =
                                effects::inflict_volatile_status(battle, ability_activation_context.ability_owner_id, &FlashFireStatus);
                            inflict_status_outcome
                        }
                    });
                    return activation_outcome.opposite();
                }
                Outcome::Success(NOTHING)
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

pub const Zombie: AbilitySpecies = AbilitySpecies::from_dex_entry(AbilityDexEntry {
    dex_number: 004,
    name: "Zombie",
    event_listener: &ZombieEventListener,
    order: 3,
});

struct ZombieEventListener;

impl EventListener<AbilityID> for ZombieEventListener {
    fn on_damage_received_handler(&self) -> Option<EventHandler<Nothing, Nothing, AbilityID, MonsterID>> {
        Some(EventHandler {
            response: |battle, broadcaster_id, receiver_id, self_id, context, relay| {
                let health_is_less_than_half = mon![broadcaster_id].current_health() <= mon![broadcaster_id].max_health() / 2;
                let is_currently_full_form = mon![broadcaster_id].species().form_name() == crate::monster_dex::MonstrossiveFullForm.form_name();
                if health_is_less_than_half & is_currently_full_form {
                    effects::change_form(battle, broadcaster_id, &crate::monster_dex::MonstrossiveHungryForm);
                }
            },
            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: PositionRelationFlags::SELF,
                ..EventFilteringOptions::default()
            },
        })
    }

    fn on_health_recovered_handler(&self) -> Option<EventHandler<Nothing, Nothing, AbilityID, MonsterID>> {
        Some(EventHandler {
            response: |battle, broadcaster_id, receiver_id, self_id, context, relay| {
                let health_is_more_than_half = mon![broadcaster_id].current_health() > mon![broadcaster_id].max_health() / 2;
                let is_currently_hungry_form = mon![broadcaster_id].species().form_name() == crate::monster_dex::MonstrossiveHungryForm.form_name();
                if health_is_more_than_half & is_currently_hungry_form {
                    effects::change_form(battle, broadcaster_id, &crate::monster_dex::MonstrossiveFullForm);
                }
            },
            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: PositionRelationFlags::SELF,
                ..EventFilteringOptions::default()
            },
        })
    }
}
