use monsim_utils::Count;
use std::fmt::Display;

use crate::{prng::Prng, sim::event_dispatcher::EventHandlerCache, Monster};

// Permanent Statuses
#[derive(Debug, Clone, Copy)]
pub struct PersistentStatus {
    pub(crate) species: &'static PersistentStatusSpecies,
}

impl PersistentStatus {
    pub(crate) fn from_species(species: &'static PersistentStatusSpecies) -> PersistentStatus {
        PersistentStatus { species }
    }

    #[inline(always)]
    pub fn name(&self) -> &'static str {
        self.species.name
    }

    #[inline(always)]
    pub fn dex_number(&self) -> u16 {
        self.species.dex_number
    }

    pub(crate) fn bind_event_handlers(&self, event_handler_cache: &mut EventHandlerCache) {
        (self.species.bind_event_handlers)(event_handler_cache)
    }
}

/// `fn(monster_that_acquired_the_status: &Monster) -> message: String`
type OnAcquiredMessageConstructor = fn(&Monster) -> String;

#[derive(Debug, Clone, Copy)]
pub struct PersistentStatusSpecies {
    pub(crate) dex_number: u16,
    pub(crate) name: &'static str,
    pub(crate) on_acquired_message: OnAcquiredMessageConstructor,
    pub(crate) bind_event_handlers: fn(&mut EventHandlerCache),
}

impl PersistentStatusSpecies {
    pub const fn from_dex_entry(dex_entry: PersistentStatusDexEntry) -> PersistentStatusSpecies {
        let PersistentStatusDexEntry {
            dex_number,
            name,
            on_acquired_message,
            bind_event_handlers,
        } = dex_entry;

        PersistentStatusSpecies {
            dex_number,
            name,
            on_acquired_message,
            bind_event_handlers,
        }
    }
}

#[derive(Debug)]
pub struct PersistentStatusDexEntry {
    pub dex_number: u16,
    pub name: &'static str,
    pub on_acquired_message: OnAcquiredMessageConstructor,
    pub bind_event_handlers: fn(&mut EventHandlerCache),
}

// Volatile Statuses
#[derive(Debug, Clone, Copy)]
pub struct VolatileStatus {
    pub(crate) species: &'static VolatileStatusSpecies,
    pub(crate) remaining_turns: u8,
}

impl Display for VolatileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write![f, "{}", self.species.name]
    }
}

impl VolatileStatus {
    pub(crate) fn from_species(prng: &mut Prng, species: &'static VolatileStatusSpecies) -> VolatileStatus {
        let lifetime_in_turns = match species.lifetime_in_turns {
            Count::Fixed(n) => n,
            Count::RandomInRange { min, max } => prng.roll_random_number_in_range(min as u16..=max as u16) as u8,
        };
        VolatileStatus {
            species,
            remaining_turns: lifetime_in_turns,
        }
    }

    #[inline(always)]
    pub fn bind_event_handlers(&self, event_handler_cache: &mut EventHandlerCache) {
        self.species.event_handlers(event_handler_cache)
    }

    #[inline(always)]
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
    pub(crate) bind_event_handlers: fn(&mut EventHandlerCache),
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
        } = dex_entry;

        VolatileStatusSpecies {
            dex_number,
            name,
            on_acquired_message,
            lifetime_in_turns,
            bind_event_handlers: event_handlers,
        }
    }

    #[inline(always)]
    pub fn event_handlers(&self, event_handler_cache: &mut EventHandlerCache) {
        (self.bind_event_handlers)(event_handler_cache)
    }
}

#[derive(Debug)]
pub struct VolatileStatusDexEntry {
    pub dex_number: u16,
    pub name: &'static str,
    pub on_acquired_message: OnAcquiredMessageConstructor,
    pub lifetime_in_turns: Count,
    pub event_handlers: fn(&mut EventHandlerCache),
}
