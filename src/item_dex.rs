#![allow(non_upper_case_globals, clippy::zero_prefixed_literal, unused)]

use monsim::{
    dual_type_matchup, effects, EventHandler, EventListener, ItemID, MonsterID, MoveCategory, MoveHitContext, MoveUseContext, NullEventListener, Type,
};
use monsim_macros::{mon, mov};
use monsim_utils::Percent;

use crate::{
    item::{ItemDexEntry, ItemFlags, ItemSpecies},
    EventFilteringOptions, PositionRelationFlags,
};

#[cfg(feature = "debug")]
use monsim::source_code_location;

pub const LifeOrb: ItemSpecies = ItemSpecies::from_dex_entry(ItemDexEntry {
    dex_number: 001,
    name: "Life Orb",
    kind: ItemFlags::NONE,
    is_consumable: false,
    event_listener: &NullEventListener,
});

struct LifeOrbEventListener;

impl EventListener<ItemID> for LifeOrbEventListener {
    fn on_modify_damage_handler(&self) -> Option<monsim::EventHandler<u16, MoveHitContext, MonsterID, ItemID>> {
        Some(EventHandler {
            response: |battle, broadcaster_id, _receiver_id, _context, item_id, damage| {
                battle.queue_message(format!["Life orb boosted the damage of {}'s attack!", mon![broadcaster_id].name()]);
                damage * Percent(130)
            },
            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: PositionRelationFlags::SELF,
                ..EventFilteringOptions::default()
            },
        })
    }

    fn on_move_used_handler(&self) -> Option<EventHandler<(), MoveUseContext, MonsterID, ItemID>> {
        Some(EventHandler {
            response: |battle,
                       broadcaster_id,
                       receiver_id,
                       MoveUseContext {
                           move_user_id,
                           move_used_id,
                           target_ids,
                       },
                       item_id,
                       _| {
                if mov![move_used_id].category().is_damaging() {
                    let one_tenth_of_total_hp = mon![move_user_id].max_health() * Percent(10);
                    battle.queue_message(format!["Life orb drained some of {}'s life force!", mon![broadcaster_id].name()]);
                    let damage_dealt = effects::deal_raw_damage(battle, move_user_id, one_tenth_of_total_hp);
                }
            },
            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: PositionRelationFlags::SELF,
                ..EventFilteringOptions::default()
            },
        })
    }
}

pub const PasshoBerry: ItemSpecies = ItemSpecies::from_dex_entry(ItemDexEntry {
    dex_number: 002,
    name: "Passho Berry",
    kind: ItemFlags::BERRY,
    is_consumable: true,
    event_listener: &PasshoBerryEventListener,
});

struct PasshoBerryEventListener;

impl EventListener<ItemID> for PasshoBerryEventListener {
    fn on_modify_damage_handler(&self) -> Option<EventHandler<u16, MoveHitContext, MonsterID, ItemID>> {
        Some(EventHandler {
            response: |battle,
                       broadcaster_id,
                       receiver_id,
                       MoveHitContext {
                           move_user_id,
                           move_used_id,
                           target_id,
                           number_of_hits,
                           number_of_targets,
                       },
                       item_id,
                       damage| {
                let move_type = mov![move_used_id].type_();
                let target_type = mon![target_id].type_();

                let type_effectiveness = dual_type_matchup(move_type, target_type);
                if move_type == Type::Water && type_effectiveness.is_matchup_super_effective() {
                    let maybe_modified_damage = effects::use_item(battle, receiver_id, |battle, item_holder_id| {
                        battle.queue_message("Passho Berry activated! The damage was reduced.");
                        damage * Percent(50)
                    });

                    maybe_modified_damage.unwrap_or(damage)
                } else {
                    damage
                }
            },
            event_filtering_options: EventFilteringOptions::default(),
        })
    }
}
