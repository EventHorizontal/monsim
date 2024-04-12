#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim_macros::move_;
use monsim_utils::not;

use super::{ability::AbilitySpecies, Type};
use crate::{
    event_dex::*, monster, sim::{event::{contexts::MoveUsed, EventHandlerStorage, EventID}, EventFilteringOptions, EventHandler, Outcome, Reaction}, source_code_location, Battle
};

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    on_activate: |(entities, message_log), owner| {
        let owner_name = monster!(owner).name();
        message_log.push(format!["{owner_name}'s Flash Fire activated!"]);
    },
    // event_callbacks: |owner, storage| {
    //     storage.add(owner, OnTryMove, |(entities, message_log), MoveUsed { move_user, move_used, target}, _relay| {
    //         if move_!(move_used).is_type(Type::Fire) {
    //             let activation_succeeded = Reaction::activate_ability(battle, target);
    //             return not!(activation_succeeded);
    //         }
    //         Outcome::Success
    //     }, source_code_location!());
    // },
    event_callbacks: bind_events! {
        OnTryMove => |(entities, message_log), MoveUsed { move_user, move_used, target}, _relay| {
            if move_!(move_used).is_type(Type::Fire) {
                let activation_succeeded = Reaction::activate_ability(battle, target);
                return not!(activation_succeeded);
            }
            Outcome::Success
        },
    },
    ..AbilitySpecies::const_default()
};