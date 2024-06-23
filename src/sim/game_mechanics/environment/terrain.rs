use monsim_utils::{Count, Nothing};

use crate::{prng::Prng, sim::builder::InitCount, sim::event_dispatcher::EventListener};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terrain {
    pub(crate) species: &'static TerrainSpecies,
    pub(crate) remaining_turns: u8,
}

impl Terrain {
    pub(crate) fn from_species(species: &'static TerrainSpecies, prng: &mut Prng) -> Terrain {
        let remaining_turns = species.lifetime_in_turns().init(prng);
        Terrain { species, remaining_turns }
    }

    #[inline(always)]
    pub fn species(&self) -> &'static TerrainSpecies {
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
pub struct TerrainSpecies {
    dex_number: u16,
    name: &'static str,
    lifetime_in_turns: Count,
    event_listener: &'static dyn EventListener<Nothing, Nothing>,
    on_start_message: &'static str,
    on_clear_message: &'static str,
}

impl PartialEq for TerrainSpecies {
    fn eq(&self, other: &Self) -> bool {
        self.dex_number == other.dex_number
    }
}

impl Eq for TerrainSpecies {}

impl TerrainSpecies {
    pub const fn from_dex_entry(dex_entry: TerrainDexEntry) -> TerrainSpecies {
        let TerrainDexEntry {
            dex_number,
            name,
            lifetime_in_turns,
            event_listener,
            on_start_message,
            on_clear_message,
        } = dex_entry;

        TerrainSpecies {
            dex_number,
            name,
            lifetime_in_turns,
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
    pub fn lifetime_in_turns(&self) -> Count {
        self.lifetime_in_turns
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

pub struct TerrainDexEntry {
    pub dex_number: u16,
    pub name: &'static str,
    pub lifetime_in_turns: Count,
    pub event_listener: &'static dyn EventListener<Nothing, Nothing>,
    pub on_start_message: &'static str,
    pub on_clear_message: &'static str,
}
