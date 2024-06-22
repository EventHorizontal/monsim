#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{
    Count, EventFilteringOptions, EventHandler, EventListener, MonsterID, MoveHitContext, Nothing, NullEventListener, Percent, PositionRelationFlags, Type,
    WeatherDexEntry, WeatherSpecies,
};
use monsim_macros::mov;

#[cfg(feature = "debug")]
use monsim::source_code_location;

pub const HarshSunlight: WeatherSpecies = WeatherSpecies::from_dex_entry(WeatherDexEntry {
    dex_number: 001,
    name: "Harsh Sunlight",
    lifetime_in_turns: Count::Fixed(5),
    event_listener: &HarshSunlightEventListener,
    on_start_message: "The sunlight became harsh!",
    on_clear_message: "The harsh sunlight faded!",
});

struct HarshSunlightEventListener;

impl EventListener<Nothing, Nothing> for HarshSunlightEventListener {
    fn on_modify_damage_handler(&self) -> Option<EventHandler<MoveHitContext, u16, Nothing, Nothing>> {
        Some(EventHandler {
            response: |battle, broadcaster_id, receiver_id, _, context, damage| {
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
        })
    }

    fn on_turn_end_handler(&self) -> Option<EventHandler<Nothing, Nothing, Nothing, Nothing, Nothing>> {
        Some(EventHandler {
            response: |battle, _, _, _, _, _| {
                battle.queue_message("The sunlight remains strong.");
            },
            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: PositionRelationFlags::all(),
                only_if_target_is: PositionRelationFlags::all(),
                only_if_receiver_is_active: true,
            },
        })
    }
}
