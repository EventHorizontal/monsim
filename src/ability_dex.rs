#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::sim::{
        AbilityInternal, AbilitySpecies, EventHandlerDeck, Type, EventFilteringOptions, EventHandler, MoveUsed, Effect,
        utils::{Outcome, not},
};

#[cfg(feature = "debug")]
use monsim::source_code_location;

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    event_handler_deck: EventHandlerDeck {
        on_try_move: Some(EventHandler {
            callback: |battle, MoveUsed { attacker_uid, move_uid, target_uid}, _relay| {
                            let current_move = battle.move_(move_uid);
                            let is_current_move_fire_type = (current_move.species.type_ == Type::Fire);
                            if is_current_move_fire_type {
                                let activation_succeeded = Effect::activate_ability(battle, target_uid);
                                return not!(activation_succeeded);
                            }
                Outcome::Success
            },
            #[cfg(feature = "debug")]
            debugging_information: source_code_location!(),
        }),
        ..EventHandlerDeck::default()
    },
    on_activate: |battle, owner_uid| {
        let owner_name = battle.monster(owner_uid).name();
        battle.message_log.push(format!["{owner_name}'s Flash Fire activated!"]);
    },
    ..AbilitySpecies::default()
};

pub const WaterAbsorb: AbilitySpecies = AbilitySpecies {
    dex_number: 002,
    name: "Water Absorb",
    event_handler_deck: EventHandlerDeck {
        on_try_move: Some(EventHandler {
            callback: |battle, MoveUsed { attacker_uid, move_uid, target_uid}, _relay| {
                            let current_move = battle.move_(move_uid);
                            let is_current_move_fire_type = (current_move.species.type_ == Type::Water);
                            if is_current_move_fire_type {
                                let activation_succeeded = Effect::activate_ability(battle, target_uid);
                                return not!(activation_succeeded);
                            }
                Outcome::Success
            },
            #[cfg(feature = "debug")]
            debugging_information: source_code_location!(),
        }),
        ..EventHandlerDeck::default()
    },
    on_activate: |battle, owner_uid| {
        let owner_name = battle.monster(owner_uid).name();
        battle.message_log.push(format!["{owner_name}'s Water Absorb activated!"]);
    },
    ..AbilitySpecies::default()
};
