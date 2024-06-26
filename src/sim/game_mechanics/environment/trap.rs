use monsim_utils::Nothing;

use crate::{sim::event_dispatcher::EventListener, TeamID};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trap {
    pub(crate) id: TrapID,
    pub(crate) layers: u8,
    pub(crate) species: &'static TrapSpecies,
}

impl Trap {
    pub(crate) fn from_species(species: &'static TrapSpecies, team_id: TeamID) -> Trap {
        let id = TrapID { team_id, species };
        Trap { id, species, layers: 1 }
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
    pub fn max_layers(&self) -> u8 {
        self.species().max_layers
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
    pub fn event_listener(&self) -> &'static dyn EventListener<TrapID, Nothing> {
        self.species.event_listener
    }

    #[inline(always)]
    pub fn number_of_layers(&self) -> u8 {
        self.layers
    }
}

#[derive(Debug, Clone)]
pub struct TrapSpecies {
    dex_number: u16,
    name: &'static str,
    max_layers: u8,
    event_listener: &'static dyn EventListener<TrapID, Nothing>,
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
            max_layers,
        } = dex_entry;

        TrapSpecies {
            dex_number,
            name,
            event_listener,
            on_start_message,
            on_clear_message,
            max_layers,
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
    pub fn max_layers(&self) -> u8 {
        self.max_layers
    }

    #[inline(always)]
    pub fn event_listener(&self) -> &'static dyn EventListener<TrapID, Nothing> {
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
    pub max_layers: u8,
    pub event_listener: &'static dyn EventListener<TrapID, Nothing>,
    pub on_start_message: &'static str,
    pub on_clear_message: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrapID {
    pub(crate) team_id: TeamID,
    pub(crate) species: &'static TrapSpecies,
}
impl TrapID {
    pub(crate) fn new(team_id: TeamID, species: &'static TrapSpecies) -> TrapID {
        TrapID { team_id, species }
    }
}
