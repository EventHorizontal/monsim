#![allow(non_upper_case_globals, clippy::zero_prefixed_literal)]

use monsim::{
    effects, Count, EventFilteringOptions, EventHandler, EventListener, Outcome, Percent, PersistentStatusDexEntry, PersistentStatusSpecies,
    PositionRelationFlags, VolatileStatusDexEntry, VolatileStatusSpecies,
};
use monsim_macros::mon;

#[cfg(feature = "debug")]
use monsim::source_code_location;

pub const Burned: PersistentStatusSpecies = PersistentStatusSpecies::from_dex_entry(PersistentStatusDexEntry {
    dex_number: 001,
    name: "Burned",
    on_acquired_message: |affected_monster| format!["{} was burned!", affected_monster.name()],
    event_listener: &BurnedEventListener,
});

struct BurnedEventListener;

impl EventListener for BurnedEventListener {
    fn on_calculate_attack_stat_handler(&self) -> Option<monsim::EventHandler<u16, monsim::MoveHitContext, monsim::MonsterID>> {
        Some(EventHandler {
            response: |_, _, _, _, current_attack_stat| current_attack_stat * Percent(50),

            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: PositionRelationFlags::SELF,
                ..EventFilteringOptions::default()
            },
        })
    }

    fn on_turn_end_handler(&self) -> Option<EventHandler<(), (), monsim::MonsterID, ()>> {
        Some(EventHandler {
            response: |battle, _, receiver_id, _context, _| {
                battle.queue_message(format!["{} is burned.", mon![receiver_id].name()]);
                let damage = (mon![receiver_id].max_health() as f64 * 1.0 / 8.0) as u16;
                let _ = effects::deal_raw_damage(battle, receiver_id, damage);
            },

            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: PositionRelationFlags::SELF,
                ..EventFilteringOptions::default()
            },
        })
    }
}

pub const Confused: VolatileStatusSpecies = VolatileStatusSpecies::from_dex_entry(VolatileStatusDexEntry {
    dex_number: 001,
    name: "Confused",
    lifetime_in_turns: Count::RandomInRange { min: 2, max: 4 },
    event_listener: &ConfusedEventListener,

    on_acquired_message: |affected_monster| format!["{} became confused!", affected_monster.name()],
});

struct ConfusedEventListener;

impl EventListener for ConfusedEventListener {
    fn on_try_move_handler(&self) -> Option<EventHandler<Outcome, monsim::MoveUseContext, monsim::MonsterID>> {
        Some(EventHandler {
            response: |battle, _broadcaster_id, receiver_id, _context, _relay| {
                battle.queue_message(format!["{} is confused!", mon![receiver_id].name()]);

                if mon![receiver_id]
                    .volatile_status(Confused)
                    .expect("self must have confused for this function to be called.")
                    .remaining_turns()
                    == 0
                {
                    battle.queue_message(format!["{} snapped out of confusion!", mon![receiver_id].name()]);
                    return Outcome::Success(());
                } else if battle.roll_chance(1, 3) {
                    battle.queue_message(format!["{} hit itself in confusion!", mon![receiver_id].name()]);
                    let one_eight_of_max_hp = (mon![receiver_id].max_health() as f64 * 1.0 / 8.0) as u16;
                    let _damage = effects::deal_raw_damage(battle, receiver_id, one_eight_of_max_hp);
                    return Outcome::Failure;
                }
                Outcome::Success(())
            },

            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: PositionRelationFlags::SELF,
                ..EventFilteringOptions::default()
            },
        })
    }
}
