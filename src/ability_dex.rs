#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim_utils::{not, Outcome};
use monsim::{move_, sim::{
        Ability, AbilitySpecies, Effect, EventFilteringOptions, EventHandler, EventHandlerDeck, MoveUsed, Type
}};

#[cfg(feature = "debug")]
use monsim::source_code_location;

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    event_handler_deck: &EventHandlerDeck {
        on_try_move: Some(EventHandler {
            callback: |battle, MoveUsed { move_user, move_used, target } , _relay| {
                            if move_![move_used].is_type(Type::Fire) {
                                let activation_succeeded = Effect::activate_ability(battle, target);
                                return not!(activation_succeeded);
                            }
                Outcome::Success
            },
            #[cfg(feature = "debug")]
            debugging_information: source_code_location!(),
        }),
        ..EventHandlerDeck::const_default()
    },
    on_activate: |battle, owner_uid| {
        let owner_name = battle.monster(owner_uid).name();
        battle.message_log.push(format!["{owner_name}'s Flash Fire activated!"]);
    },
    ..AbilitySpecies::const_default()
};

pub const WaterAbsorb: AbilitySpecies = AbilitySpecies {
    dex_number: 002,
    name: "Water Absorb",
    event_handler_deck: &EventHandlerDeck {
        on_try_move: Some(EventHandler {
            callback: |battle, MoveUsed { move_user, move_used, target}, _relay| {
                            if move_![move_used].is_type(Type::Water) {
                                let activation_succeeded = Effect::activate_ability(battle, target);
                                return not!(activation_succeeded);
                            }
                Outcome::Success
            },
            #[cfg(feature = "debug")]
            debugging_information: source_code_location!(),
        }),
        ..EventHandlerDeck::const_default()
    },
    on_activate: |battle, owner_uid| {
        let owner_name = battle.monster(owner_uid).name();
        battle.message_log.push(format!["{owner_name}'s Water Absorb activated!"]);
    },
    ..AbilitySpecies::const_default()
};
