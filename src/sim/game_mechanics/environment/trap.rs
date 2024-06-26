use monsim_utils::Nothing;

use crate::sim::event_dispatcher::EventListener;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trap {
    pub(crate) species: &'static TrapSpecies,
}

impl Trap {
    pub(crate) fn from_species(species: &'static TrapSpecies) -> Trap {
        Trap { species }
    }

    #[inline(always)]
    pub fn species(&self) -> &'static TrapSpecies {
        self.species
    }

    #[inline(always)]
    pub fn name(&self) -> &'static str {
        self.species().name()
    }

    #[inline(always)]
    pub fn on_start_message(&self) -> &'static str {
        self.species.on_start_message
    }

    #[inline(always)]
    pub fn on_clear_message(&self) -> &'static str {
        self.species.on_clear_message
    }

    #[inline(always)]
    pub fn event_listener(&self) -> &'static dyn EventListener<Nothing, Nothing> {
        self.species.event_listener
    }
}

#[derive(Debug, Clone)]
pub struct TrapSpecies {
    dex_number: u16,
    name: &'static str,
    event_listener: &'static dyn EventListener<Nothing, Nothing>,
    on_start_message: &'static str,
    on_clear_message: &'static str,
}

impl PartialEq for TrapSpecies {
    fn eq(&self, other: &Self) -> bool {
        self.dex_number == other.dex_number
    }
}

impl Eq for TrapSpecies {}

impl TrapSpecies {
    pub const fn from_dex_entry(dex_entry: TrapDexEntry) -> TrapSpecies {
        let TrapDexEntry {
            dex_number,
            name,
            event_listener,
            on_start_message,
            on_clear_message,
        } = dex_entry;

        TrapSpecies {
            dex_number,
            name,
            event_listener,
            on_start_message,
            on_clear_message,
        }
    }

    #[inline(always)]
    pub fn dex_number(&self) -> u16 {
        self.dex_number
    }

    #[inline(always)]
    pub fn name(&self) -> &'static str {
        self.name
    }

    #[inline(always)]
    pub fn event_listener(&self) -> &'static dyn EventListener<Nothing, Nothing> {
        self.event_listener
    }

    #[inline(always)]
    pub fn on_start_message(&self) -> &'static str {
        self.on_start_message
    }

    #[inline(always)]
    pub fn on_clear_message(&self) -> &'static str {
        self.on_clear_message
    }
}

pub struct TrapDexEntry {
    pub dex_number: u16,
    pub name: &'static str,
    pub event_listener: &'static dyn EventListener<Nothing, Nothing>,
    pub on_start_message: &'static str,
    pub on_clear_message: &'static str,
}
