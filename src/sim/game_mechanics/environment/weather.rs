use monsim_utils::{Count, Nothing};

use crate::{
    prng::Prng,
    sim::{
        builder::InitCount,
        event_dispatcher::{EventListener, MechanicID},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Weather {
    pub(crate) species: &'static WeatherSpecies,
    pub(crate) remaining_turns: u8,
}

impl Weather {
    pub fn from_species(species: &'static WeatherSpecies, prng: &mut Prng) -> Weather {
        let remaining_turns = species.lifetime_in_turns().init(prng);
        Weather { species, remaining_turns }
    }

    #[inline(always)]
    pub fn species(&self) -> &'static WeatherSpecies {
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
    pub fn event_listener(&self) -> &'static dyn EventListener<WeatherID, Nothing> {
        self.species.event_listener
    }
}

#[derive(Debug, Clone)]
pub struct WeatherSpecies {
    dex_number: u16,
    name: &'static str,
    lifetime_in_turns: Count,
    event_listener: &'static dyn EventListener<WeatherID, Nothing>,
    on_start_message: &'static str,
    on_clear_message: &'static str,
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
            lifetime_in_turns,
            event_listener,
            on_start_message,
            on_clear_message,
        } = dex_entry;

        WeatherSpecies {
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
    pub fn event_listener(&self) -> &'static dyn EventListener<WeatherID, Nothing> {
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

pub struct WeatherDexEntry {
    pub dex_number: u16,
    pub name: &'static str,
    pub lifetime_in_turns: Count,
    pub event_listener: &'static dyn EventListener<WeatherID, Nothing>,
    pub on_start_message: &'static str,
    pub on_clear_message: &'static str,
}

/// Marker for Weather
#[derive(Copy, Clone)]
pub struct WeatherID;

impl MechanicID for WeatherID {
    #[cfg(feature = "debug")]
    fn mechanic_name(&self, battle: &crate::Battle) -> &'static str {
        battle.environment().weather().expect("Expected weather because id exists").name()
    }
}
