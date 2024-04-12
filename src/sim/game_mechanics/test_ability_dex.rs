#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim_macros::move_;
use monsim_utils::not;

use super::{ability::AbilitySpecies, Type};
use crate::{
    event_dex::*, monster, sim::{event::{contexts::TheMoveUsed, EventHandlerDeck, EventHandlerStorage, EventID}, EventFilteringOptions, EventHandler, Outcome, Reaction}, source_code_location, Battle, TheAbilityActivated
};

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    on_activate: |(entities, message_log), TheAbilityActivated { activated_ability }| {
        let owner_name = entities[activated_ability.owner].name();
        message_log.push(format!["{owner_name}'s Flash Fire activated!"]);
    },
    event_handlers: | | {
        EventHandlerDeck {
            on_try_move: Some(|(entities, message_log), TheMoveUsed { move_user, move_used, target}, _relay| {
                    if move_!(move_used).is_type(Type::Fire) {
                        let activation_succeeded = Reaction::activate_ability(battle, context);
                        return not!(activation_succeeded);
                    }
                    Outcome::Success
            }),
            ..EventHandlerDeck::const_default()
        }
    },
    ..AbilitySpecies::const_default()
};