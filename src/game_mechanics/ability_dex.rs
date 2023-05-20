#![allow(non_upper_case_globals, clippy::zero_prefixed_literal)]

use super::{ability::AbilitySpecies, MonType};
use crate::{
    event::{EventHandlerFilters, EventHandlerSet, DEFAULT_HANDLERS},
    EventHandler, SecondaryAction, FAILURE, SUCCESS,
};

pub const FlashFire: AbilitySpecies = AbilitySpecies {
    dex_number: 001,
    name: "Flash Fire",
    event_handlers: EventHandlerSet {
        on_try_move: Some(EventHandler {
            #[cfg(feature = "debug")]
            dbg_location: const_format::formatcp![
                "[{}:{}] {:#?}",
                std::file!(),
                std::line!(),
                std::stringify!(FlashFire.on_try_move)
            ],
            callback: |ctx, prng, owner_uid, _relay| {
                let current_move = *ctx.get_current_action_as_move().expect(
                    "The current action should be a move within on_try_move handler context.",
                );
                if current_move.species.type_ == MonType::Fire
                    && SecondaryAction::activate_ability(ctx, prng, owner_uid)
                {
                    return FAILURE;
                }
                SUCCESS
            },
        }),
        ..DEFAULT_HANDLERS
    },
    on_activate: |ctx, _owner_uid| {
        ctx.push_message(&"Flash Fire activated!");
    },
    event_handler_filters: EventHandlerFilters::default(),
    order: 0,
};
