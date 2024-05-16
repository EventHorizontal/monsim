use std::fmt::Display;
use monsim_utils::Count;

use crate::{EventFilteringOptions, EventHandlerDeck, Monster};

#[derive(Debug, Clone, Copy)]
pub struct VolatileStatus {
    pub(crate) species: & 'static VolatileStatusSpecies,
    pub(crate) remaining_turns: u8,
}

impl Display for VolatileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write![f, "{}", self.species.name]
    }
}

impl VolatileStatus {
    // HACK: We currently pass in the remaining turns because `prng` is inside `battle` sot the mutable references
    // conflict. A structural change is needed to resolve this correctly.
    pub(crate) fn new(lifetime_in_turns: u8, species: &'static VolatileStatusSpecies) -> VolatileStatus {
        VolatileStatus {
            species,
            remaining_turns: lifetime_in_turns,
        }
    }

    #[inline(always)]
    pub fn event_handlers(&self) -> EventHandlerDeck {
        self.species.event_handlers()
    }
    
    pub(crate) fn event_filtering_options(&self) -> EventFilteringOptions {
        self.species.event_filtering_options
    }
    
    pub fn remaining_turns(&self) -> u8 {
        self.remaining_turns
    }
    
}

// TODO: make these pub crate and make a data struct called `StatusDexEntry`
#[derive(Debug, Clone, Copy)]
pub struct VolatileStatusSpecies {
    pub id: VolatileStatusSpeciesID,
    pub name: &'static str,
    pub message: fn(&Monster) -> String,
    pub lifetime_in_turns: Count,
    pub event_handlers: fn() -> EventHandlerDeck,
    pub event_filtering_options: EventFilteringOptions,
}

impl PartialEq for VolatileStatusSpecies {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for VolatileStatusSpecies {}

impl VolatileStatusSpecies {
    #[inline(always)]
    pub fn event_handlers(&self) -> EventHandlerDeck {
        (self.event_handlers)()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolatileStatusSpeciesID(u16);

impl VolatileStatusSpeciesID {
    pub const fn new(number: u16) -> VolatileStatusSpeciesID {
        VolatileStatusSpeciesID(number)
    }
}

