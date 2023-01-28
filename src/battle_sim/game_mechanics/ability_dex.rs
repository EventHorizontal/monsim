#![allow(non_upper_case_globals)]

use super::{ability::AbilitySpecies, MonType};
use crate::battle_sim::{
    event::{EventHandlerFilters, EventHandlerSet, DEFAULT_HANDLERS},
    Action, FAILURE, SUCCESS,
};

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    event_handlers: EventHandlerSet {
        on_try_move: Some(|context, owner_uid, _relay| {
            if context.current_action_user().is_type(MonType::Fire) {
                if Action::activate_ability(context, owner_uid) {
                    return FAILURE;
                }
            }
            SUCCESS
        }),
        ..DEFAULT_HANDLERS
    },
    on_activate: |context, _owner_uid| Action::display_message(context, &"Flash Fire activated!"),
    event_handler_filters: EventHandlerFilters::default(),
    order: 0,
};
