#![allow(non_upper_case_globals)]

use super::{ability::AbilitySpecies, MonType};
use crate::{
    event::{EventHandlerFilters, EventHandlerSet, DEFAULT_HANDLERS},
    Action, FAILURE, SUCCESS,
};

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    event_handlers: EventHandlerSet {
        on_try_move: Some(|context, prng, owner_uid, _relay| {
            // TEMP: This ugly predicate will be easier on the eyes once more of the API is written
            if context.read_move({
                    if let crate::ActionChoice::Move { move_uid, target_uid: _ } = context.current_action {
                        move_uid
                    } else {
                        panic!()
                    }
                }).species.type_ == MonType::Fire {
                if Action::activate_ability(context, prng, owner_uid) {
                    return FAILURE;
                }
            }
            SUCCESS
        }),
        ..DEFAULT_HANDLERS
    },
    on_activate: |context, _owner_uid| {
        context.message_buffer.push("Flash Fire activated".to_string())
    },
    event_handler_filters: EventHandlerFilters::default(),
    order: 0,
};
