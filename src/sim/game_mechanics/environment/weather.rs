use monsim_utils::Nothing;

use crate::EventHandlerSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Weather {
    pub(crate) species: &'static WeatherSpecies,
}

impl Weather {
    #[inline(always)]
    pub fn species(&self) -> &'static WeatherSpecies {
        self.species
    }

    #[inline(always)]
    pub fn name(&self) -> &'static str {
        self.species().name()
    }

    #[inline(always)]
    pub fn event_handlers(&self) -> EventHandlerSet<Nothing> {
        self.species().event_handlers()
    }
}

#[derive(Debug, Clone)]
pub struct WeatherSpecies {
    dex_number: u16,
    name: &'static str,
    on_event_behaviour: fn() -> EventHandlerSet<Nothing>,
}

impl PartialEq for WeatherSpecies {
    fn eq(&self, other: &Self) -> bool {
        self.dex_number == other.dex_number
    }
}

impl Eq for WeatherSpecies {}

impl WeatherSpecies {
    pub const fn from_dex_entry(dex_entry: WeatherDexEntry) -> WeatherSpecies {
        let WeatherDexEntry {
            dex_number,
            name,
            on_event_behaviour,
        } = dex_entry;

        WeatherSpecies {
            dex_number,
            name,
            on_event_behaviour,
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
    pub fn event_handlers(&self) -> EventHandlerSet<Nothing> {
        (self.on_event_behaviour)()
    }
}

pub struct WeatherDexEntry {
    pub dex_number: u16,
    pub name: &'static str,
    pub on_event_behaviour: fn() -> EventHandlerSet<Nothing>,
}
