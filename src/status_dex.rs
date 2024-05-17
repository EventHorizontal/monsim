#![allow(non_upper_case_globals)]

use monsim::{effects, source_code_location, status::{VolatileStatusSpecies, VolatileStatusSpeciesID}, Count, EventFilteringOptions, EventHandler, EventHandlerDeck, Outcome, TargetFlags};
use monsim_macros::mon;

pub const Confused: VolatileStatusSpecies = VolatileStatusSpecies {
    id: VolatileStatusSpeciesID::new(1),
    name: "Confused",
    lifetime_in_turns: Count::RandomInRange { min: 2, max: 4 },
    event_handlers: || {
        EventHandlerDeck {
            on_try_move: Some(EventHandler {
                #[cfg(feature="debug")]
                source_code_location: source_code_location!(),
                effect: |sim, self_id, _context | {
                    
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
    message: |affected_monster| {
        format!["{} became confused!", affected_monster.name()]
    },
};