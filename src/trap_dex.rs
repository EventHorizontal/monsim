#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{dual_type_matchup, effects, EventFilteringOptions, EventHandler, EventListener, Nothing, Percent, TrapDexEntry, TrapID, TrapSpecies, Type};
use monsim_macros::mon;

pub const PointedStones: TrapSpecies = TrapSpecies::from_dex_entry(TrapDexEntry {
    dex_number: 001,
    name: "Pointed Stones",
    event_listener: &PointedStonesEventListener,
    on_start_message: "Pointed rocks were scattered around the opponents feet!",
    on_clear_message: "The pointed rocks were scattered away.",
});

struct PointedStonesEventListener;

impl EventListener<TrapID, Nothing> for PointedStonesEventListener {
    fn on_monster_enter_battle_handler(&self) -> Option<EventHandler<Nothing, Nothing, TrapID, Nothing>> {
        Some(EventHandler {
            response: |battle, broadcaster_id, receiver_id, mechanic_id, context, relay| {
                // TODO: Fix Percent type to allow floats.
                battle.queue_message(format!["Pointed stones dug into {}'s feet!", mon![broadcaster_id].name()]);
                let rock_effectiveness_multiplier: Percent = dual_type_matchup(Type::Rock, mon![broadcaster_id].type_()).into();
                let damage_amount = mon![broadcaster_id].max_health() * Percent(12) * rock_effectiveness_multiplier;
                let outcome = effects::deal_raw_damage(battle, broadcaster_id, damage_amount);
            },
            event_filtering_options: EventFilteringOptions::default(),
        })
    }
}
