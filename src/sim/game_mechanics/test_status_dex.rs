#![allow(non_upper_case_globals, clippy::zero_prefixed_literal)]

use monsim_macros::mon;
use monsim_utils::{Count, Outcome, Percent, NOTHING};

use crate::{
    effects, source_code_location, EventFilteringOptions, EventHandler, EventHandlerDeck, PersistentStatusDexEntry, PersistentStatusSpecies, TargetFlags,
    VolatileStatusDexEntry, VolatileStatusSpecies,
};

pub const Burned: PersistentStatusSpecies = PersistentStatusSpecies::from_dex_entry(PersistentStatusDexEntry {
    dex_number: 001,
    name: "Burned",
    on_acquired_message: |affected_monster| format!["{} was burned!", affected_monster.name()],
    event_handlers: || EventHandlerDeck {
        on_calculate_attack_stat: Some(EventHandler {
            #[cfg(feature = "debug")]
            source_code_location: source_code_location!(),

            response: |_sim, _, _receiver_id, _context, current_attack_stat| current_attack_stat * Percent(50),

            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: TargetFlags::SELF,
                ..EventFilteringOptions::default()
            },
        }),
        on_turn_end: Some(EventHandler {
            #[cfg(feature = "debug")]
            source_code_location: source_code_location!(),

            response: |battle, _, receiver_id, _context, _| {
                battle.queue_message(format!["{} is burned.", mon![receiver_id].name()]);
                let damage = (mon![receiver_id].max_health() as f64 * 1.0 / 8.0) as u16;
                let _ = effects::deal_raw_damage(battle, (receiver_id, damage));
            },

            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: TargetFlags::SELF,
                ..EventFilteringOptions::default()
            },
        }),
        ..EventHandlerDeck::empty()
    },
});

pub const Confused: VolatileStatusSpecies = VolatileStatusSpecies::from_dex_entry(VolatileStatusDexEntry {
    dex_number: 001,
    name: "Confused",
    lifetime_in_turns: Count::RandomInRange { min: 2, max: 4 },
    event_handlers: || EventHandlerDeck {
        on_try_move: Some(EventHandler {
            #[cfg(feature = "debug")]
            source_code_location: source_code_location!(),

            response: |battle, _broadcaster_id, receiver_id, _context, _relay| {
                battle.queue_message(format!["{} is confused!", mon![receiver_id].name()]);

                if mon![receiver_id]
                    .volatile_status(Confused)
                    .expect("self must have confused for this function to be called.")
                    .remaining_turns()
                    == 0
                {
                    battle.queue_message(format!["{} snapped out of confusion!", mon![receiver_id].name()]);
                    return Outcome::Success(NOTHING);
                } else if battle.roll_chance(1, 3) {
                    battle.queue_message(format!["{} hit itself in confusion!", mon![receiver_id].name()]);
                    let one_eight_of_max_hp = (mon![receiver_id].max_health() as f64 * 1.0 / 8.0) as u16;
                    let _damage = effects::deal_raw_damage(battle, (receiver_id, one_eight_of_max_hp));
                    return Outcome::Failure;
                }
                Outcome::Success(NOTHING)
            },

            event_filtering_options: EventFilteringOptions {
                only_if_broadcaster_is: TargetFlags::SELF,
                ..EventFilteringOptions::default()
            },
        }),
        ..EventHandlerDeck::empty()
    },

    on_acquired_message: |affected_monster| format!["{} became confused!", affected_monster.name()],
});
