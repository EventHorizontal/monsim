#![allow(non_upper_case_globals)]
#![allow(clippy::zero_prefixed_literal)]

use monsim::{effects, source_code_location, Count, EventFilteringOptions, EventHandler, EventHandlerDeck, Outcome, TargetFlags, VolatileStatusDexEntry, VolatileStatusSpecies, PersistentStatusSpecies, PersistentStatusDexEntry};
use monsim_macros::mon;

pub const Burned: PersistentStatusSpecies = PersistentStatusSpecies::from_dex_entry(PersistentStatusDexEntry {
    dex_number: 001,
    on_acquired_message: |affected_monster| {
        format!["{} was burned!", affected_monster.name()]
    },
    event_handlers: || {
        EventHandlerDeck {
            // TODO:
            // on_calculate_attack_stat: Some(EventHandler { halve_your_attack_stat })
            // on_turn_end: Some(EventHandler { take damage 1/8th of the users max damage })
            ..EventHandlerDeck::empty()
        }
    },
    event_filtering_options: EventFilteringOptions {
        allowed_broadcaster_relation_flags: TargetFlags::SELF,
        ..EventFilteringOptions::default()
    },
}); 


pub const Confused: VolatileStatusSpecies = VolatileStatusSpecies::from_dex_entry(VolatileStatusDexEntry {
    dex_number: 001,
    name: "Confused",
    lifetime_in_turns: Count::RandomInRange { min: 2, max: 4 },
    event_handlers: || {
        EventHandlerDeck {
            on_try_move: Some(EventHandler {
                #[cfg(feature="debug")]
                source_code_location: source_code_location!(),
                effect: |sim, self_id, _context| {
                    
                    sim.push_message(format!["{} is confused!", mon![self_id].name()]);
                    
                    if mon![self_id].volatile_status(Confused)
                        .expect("self must have confused for this function to be called.")
                        .remaining_turns() == 0 {
                        sim.push_message(format!["{} snapped out of confusion!", mon![self_id].name()]);
                        return Outcome::Success;
                    } else if sim.chance(1, 3) {
                        sim.push_message(format!["{} hit itself in confusion!", mon![self_id].name()]);
                        let one_eight_of_max_hp = (mon![self_id].max_health() as f64 * 1.0/8.0) as u16;  
                        let _damage = effects::deal_raw_damage(sim, self_id, (self_id, one_eight_of_max_hp));
                        return Outcome::Failure;
                    }
                    Outcome::Success
                },
            }),
            ..EventHandlerDeck::empty()
        }
    },
    event_filtering_options: EventFilteringOptions {
        allowed_broadcaster_relation_flags: TargetFlags::SELF,
        ..EventFilteringOptions::default()
    },
    on_acquired_message: |affected_monster| {
        format!["{} became confused!", affected_monster.name()]
    },
});