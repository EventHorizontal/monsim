use crate::sim::{event_dispatcher::EventListener, MonsterID};
use core::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ability {
    pub(crate) id: AbilityID,
    pub(crate) species: &'static AbilitySpecies,
}

impl Ability {
    pub fn event_listener(&self) -> &'static dyn EventListener {
        self.species.event_listener
    }

    #[inline(always)]
    pub fn species(&self) -> &'static AbilitySpecies {
        self.species
    }

    #[inline(always)]
    pub fn name(&self) -> &'static str {
        self.species.name
    }

    #[inline(always)]
    pub fn order(&self) -> u16 {
        self.species.order
    }

    #[inline(always)]
    pub fn dex_number(&self) -> u16 {
        self.species.dex_number
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AbilityID {
    pub owner_id: MonsterID,
}
impl AbilityID {
    pub(crate) fn _from_owner(ability_owner: MonsterID) -> AbilityID {
        AbilityID { owner_id: ability_owner }
    }
}

#[derive(Clone, Copy)]
pub struct AbilitySpecies {
    dex_number: u16,
    name: &'static str,
    event_listener: &'static dyn EventListener,
    order: u16,
}

impl Debug for AbilitySpecies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:03} {}", self.dex_number, self.name)
    }
}

impl PartialEq for AbilitySpecies {
    fn eq(&self, other: &Self) -> bool {
        self.dex_number == other.dex_number
    }
}

impl Eq for AbilitySpecies {}

impl AbilitySpecies {
    pub const fn from_dex_entry(dex_entry: AbilityDexEntry) -> Self {
        let AbilityDexEntry {
            dex_number,
            name,
            event_listener: event_handlers,
            order,
        } = dex_entry;

        Self {
            dex_number,
            name,
            event_listener: event_handlers,
            order,
        }
    }

    #[inline(always)]
    pub fn event_listener(&self) -> &'static dyn EventListener {
        self.event_listener
    }

    #[inline(always)]
    pub fn name(&self) -> &'static str {
        self.name
    }

    #[inline(always)]
    pub fn order(&self) -> u16 {
        self.order
    }

    #[inline(always)]
    pub fn dex_number(&self) -> u16 {
        self.dex_number
    }
}

pub struct AbilityDexEntry {
    pub dex_number: u16,
    pub name: &'static str,
    pub event_listener: &'static dyn EventListener,
    pub order: u16,
}
