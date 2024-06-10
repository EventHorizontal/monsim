#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{EventFilteringOptions, EventHandler, EventHandlerSet, Percent, PositionRelationFlags, Type, WeatherDexEntry, WeatherSpecies};
use monsim_macros::mov;

#[cfg(feature = "debug")]
use monsim::source_code_location;

pub const HarshSunlight: WeatherSpecies = WeatherSpecies::from_dex_entry(WeatherDexEntry {
    dex_number: 001,
    name: "Harsh Sunlight",
    on_event_behaviour: || EventHandlerSet {
        on_modify_damage: Some(EventHandler {
            #[cfg(feature = "debug")]
            source_code_location: source_code_location!(),
            response: |battle, broadcaster_id, receiver_id, context, damage| {
                if mov![context.move_used_id].is_type(Type::Fire) {
                    battle.queue_debug_message("(Harsh Sunlight boosted the move's damage)");
                    damage * Percent(150)
                } else if mov![context.move_used_id].is_type(Type::Water) {
                    battle.queue_debug_message("(Harsh Sunlight reduced the move's damage)");
                    damage * Percent(50)
                } else {
                    damage
                }
            },
            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: PositionRelationFlags::all(),
                only_if_target_is: PositionRelationFlags::all(),
                only_if_receiver_is_active: true,
            },
        }),
        on_turn_end: Some(EventHandler {
            #[cfg(feature = "debug")]
            source_code_location: source_code_location!(),
            response: |battle, _, _, _, _| {
                battle.queue_message("The sunlight remains strong.");
            },
            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: PositionRelationFlags::all(),
                only_if_target_is: PositionRelationFlags::all(),
                only_if_receiver_is_active: true,
            },
        }),
        ..EventHandlerSet::default_for_environment()
    },
});
