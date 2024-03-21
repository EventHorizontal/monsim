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
            callback: |battle, MoveUsed { attacker, move_, target }, _relay| {
                // If the move is fire type, we activate the ability. If the ability activation succeeds, then the move fails.
                if move_.is_type(Type::Fire) {
                    let ability_succeeded = Effect::activate_ability(battle, target);
                    let move_succeeded = not!(ability_succeeded);
                    move_succeeded
                } else {
                    // If the move is _not_ fire type, the move always succeeds.
                    Outcome::Success
                }
            },
            #[cfg(feature = "debug")]
            debugging_information: source_code_location!(),
        }),
        ..EventHandlerDeck::const_default()
    },
    on_activate: |battle, owner| {
        battle.message_log.push(format![
            "{owner_name}'s Flash Fire activated!",
            owner_name = owner.name()
        ]);
    },
    ..AbilitySpecies::const_default()
};

pub const WaterAbsorb: AbilitySpecies = AbilitySpecies {
    dex_number: 002,
    name: "Water Absorb",
    event_handler_deck: EventHandlerDeck {
        on_try_move: Some(EventHandler {
            callback: |battle, MoveUsed { attacker, move_, target }, _relay| {
                // If the move is water type, we activate the ability. If the ability activation succeeds, then the move fails.
                if move_.is_type(Type::Water) {
                    let ability_succeeded = Effect::activate_ability(battle, target);
                    let move_succeeded = not!(ability_succeeded);
                    move_succeeded
                } else {
                    // If the move is _not_ water type, the move always succeeds.
                    Outcome::Success
                }
            },
            #[cfg(feature = "debug")]
            debugging_information: source_code_location!(),
        }),
        ..EventHandlerDeck::const_default()
    },
    on_activate: |battle, owner| {
        battle.message_log.push(format![
            "{owner_name}'s Water Absorb activated!",
            owner_name = owner.name()
        ]);
    },
    ..AbilitySpecies::const_default()
};
