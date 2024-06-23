#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{
    Count, EventFilteringOptions, EventHandler, EventListener, InflictPersistentStatusContext, InflictVolatileStatusContext, MonsterID, MoveHitContext,
    Nothing, Outcome, Percent, TerrainDexEntry, TerrainSpecies, Type, NOTHING,
};
use monsim_macros::{mon, mov};

use crate::status_dex::Confused;

pub const MistyTerrain: TerrainSpecies = TerrainSpecies::from_dex_entry(TerrainDexEntry {
    dex_number: 001,
    name: "Misty Terrain",
    lifetime_in_turns: Count::Fixed(5),
    event_listener: &MistyTerrainEventListener,
    on_start_message: "The terrain became misty!",
    on_clear_message: "The Misty Terrain faded!",
});

struct MistyTerrainEventListener;

impl EventListener<Nothing, Nothing> for MistyTerrainEventListener {
    fn on_modify_damage_handler(&self) -> Option<EventHandler<MoveHitContext, u16, Nothing, Nothing>> {
        Some(EventHandler {
            response: |battle, broadcaster_id, receiver_id, _, move_hit_context, damage| {
                if mov![move_hit_context.move_used_id].is_type(Type::Dragon) {
                    battle.queue_debug_message(format!["Misty terrain reduced the damage of {}", mov![move_hit_context.move_used_id].name()]);
                    damage * Percent(50)
                } else {
                    damage
                }
            },
            event_filtering_options: EventFilteringOptions::default(),
        })
    }

    fn on_try_inflict_persistent_status_handler(&self) -> Option<EventHandler<InflictPersistentStatusContext, Outcome, Nothing, Nothing>> {
        Some(EventHandler {
            response: |battle, broadcaster_id, receiver_id, _, context, relay| {
                battle.queue_debug_message(format![
                    "(Misty Terrain prevented {} from being {}.)",
                    mon![context.affected_monster_id].name(),
                    context.status_condition.name()
                ]);
                Outcome::Failure
            },
            event_filtering_options: EventFilteringOptions::default(),
        })
    }

    fn on_try_inflict_volatile_status_handler(&self) -> Option<EventHandler<InflictVolatileStatusContext, Outcome, Nothing, Nothing>> {
        Some(EventHandler {
            response: |battle, broadcaster_id, receiver_id, _, context, relay| {
                if context.status_condition == &Confused {
                    battle.queue_debug_message("(Misty Terrain prevented confusion.)");
                    Outcome::Failure
                } else {
                    Outcome::Success(NOTHING)
                }
            },
            event_filtering_options: EventFilteringOptions::default(),
        })
    }
}
