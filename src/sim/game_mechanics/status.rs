use std::fmt::Display;
use monsim_utils::Count;

use crate::{EventFilteringOptions, EventHandlerDeck, Monster};

// Permanent Statuses
#[derive(Debug, Clone, Copy)]
pub struct PersistentStatus {
    pub(crate) species: & 'static PersistentStatusSpecies,
}

impl PersistentStatus {
    pub(crate) fn new(species: &'static PersistentStatusSpecies) -> PersistentStatus {
        PersistentStatus {
            species,
        }
    }
    
    pub(crate) fn event_handlers(&self) -> EventHandlerDeck {
        (self.species.event_handlers)()
    }
    
    pub(crate) fn event_filtering_options(&self) -> EventFilteringOptions {
        self.species.event_filtering_options
    }
    
    pub(crate) fn name(&self) -> &'static str {
        self.species.name
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PersistentStatusSpecies {
    pub(crate) dex_number: u16,
    pub(crate) name: &'static str,
    pub(crate) on_acquired_message: fn(&Monster) -> String,
    pub(crate) event_handlers: fn() -> EventHandlerDeck,
    pub(crate) event_filtering_options: EventFilteringOptions,
}

impl PersistentStatusSpecies {
    pub const fn from_dex_entry(dex_entry: PersistentStatusDexEntry) -> PersistentStatusSpecies {
        let PersistentStatusDexEntry { 
            dex_number, 
            name,
            on_acquired_message, 
            event_handlers, 
            event_filtering_options 
        } = dex_entry;

        PersistentStatusSpecies {
            dex_number,
            name,
            on_acquired_message,
            event_handlers,
            event_filtering_options,
        }
    }
}

#[derive(Debug)]
pub struct PersistentStatusDexEntry {
    pub dex_number: u16,
    pub name: &'static str,
    pub on_acquired_message: fn(&Monster) -> String,
    pub event_handlers: fn() -> EventHandlerDeck,
    pub event_filtering_options: EventFilteringOptions,
}

// Volatile Statuses
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

#[derive(Debug, Clone, Copy)]
pub struct VolatileStatusSpecies {
    pub(crate) dex_number: u16,
    pub(crate) name: &'static str,
    pub(crate) on_acquired_message: fn(&Monster) -> String,
    pub(crate) lifetime_in_turns: Count,
    pub(crate) event_handlers: fn() -> EventHandlerDeck,
    pub(crate) event_filtering_options: EventFilteringOptions,
}

impl PartialEq for VolatileStatusSpecies {
    fn eq(&self, other: &Self) -> bool {
        self.dex_number == other.dex_number
    }
}

impl Eq for VolatileStatusSpecies {}

impl VolatileStatusSpecies {
    pub const fn from_dex_entry(dex_entry: VolatileStatusDexEntry) -> VolatileStatusSpecies {
        let VolatileStatusDexEntry { 
            dex_number,
            name, 
            on_acquired_message, 
            lifetime_in_turns, 
            event_handlers, 
            event_filtering_options 
        } = dex_entry;

        VolatileStatusSpecies {
            dex_number,
            name,
            on_acquired_message,
            lifetime_in_turns,
            event_handlers,
            event_filtering_options,
        }
    }

    #[inline(always)]
    pub fn event_handlers(&self) -> EventHandlerDeck {
        (self.event_handlers)()
    }
}

#[derive(Debug)]
pub struct VolatileStatusDexEntry {
    pub dex_number: u16,
    pub name: &'static str,
    /// fn(affected_monster: &Monster) -> message: String 
    pub on_acquired_message: fn(&Monster) -> String,
    pub lifetime_in_turns: Count,
    pub event_handlers: fn() -> EventHandlerDeck,
    pub event_filtering_options: EventFilteringOptions,
}