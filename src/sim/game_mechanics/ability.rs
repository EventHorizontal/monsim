use crate::sim::{event::{EventFilteringOptions, OwnedEventHandlerDeck}, ActivationOrder, Battle, EventHandlerDeck, MonsterUID};
use core::fmt::Debug;
use std::cell::Cell;

#[derive(Debug, Clone, Copy)]
pub struct Ability<'a> {
    uid: AbilityUID,
    ability_data: &'a Cell<AbilityData>
}

impl<'a> PartialEq for Ability<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.uid() == other.uid()
    }
}

impl<'a> Eq for Ability<'a> {}


impl<'a> Ability<'a> {
    pub fn new(uid: AbilityUID, ability_data: &Cell<AbilityData>) -> Self {
        Self {
            uid,
            ability_data
        }
    }
    
    pub fn species(&self) -> AbilitySpecies {
        self.data().species
    }
    
    pub(crate) fn data(&self) -> AbilityData {
        self.ability_data.get()
    }

    pub(crate) fn uid(&self) -> AbilityUID {
        self.data().uid
    }

    pub(crate) fn event_handler_deck(&self) -> EventHandlerDeck {
        self.data().event_handler_deck()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct AbilityData {
    pub uid: AbilityUID,
    pub species: AbilitySpecies,
}

pub type AbilityUID = MonsterUID;

#[derive(Clone, Copy)]
pub struct AbilitySpecies {
    pub dex_number: u16,
    pub name: &'static str,
    pub event_handler_deck: EventHandlerDeck,
    /// `fn(battle: &mut Battle, ability_holder: MonsterUID)`
    pub on_activate: fn(&mut Battle, MonsterUID),
    pub filtering_options: EventFilteringOptions,
    pub order: u16,
}

impl Debug for AbilitySpecies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:03} {}", self.dex_number, self.name)
    }
}

impl Default for AbilitySpecies {
    fn default() -> Self {
        ABILITY_DEFAULTS
    }
}

impl PartialEq for AbilitySpecies {
    fn eq(&self, other: &Self) -> bool {
        self.dex_number == other.dex_number
    }
}

const ABILITY_DEFAULTS: AbilitySpecies = AbilitySpecies {
    dex_number: 000,
    name: "Unnamed",
    event_handler_deck: EventHandlerDeck::default(),
    on_activate: |_battle, _ability_holder_uid| {},
    filtering_options: EventFilteringOptions::default(),
    order: 0,
};

impl AbilitySpecies {
    pub const fn default() -> Self {
        ABILITY_DEFAULTS
    }
}

impl Eq for AbilitySpecies {}

impl AbilityData {
    pub fn new(owner_uid: AbilityUID, species: AbilitySpecies) -> Self {
        Self {
            uid: owner_uid, 
            species, 
        }
    }

    pub fn on_activate(&self, battle: &mut Battle, owner_uid: MonsterUID) {
        (self.species.on_activate)(battle, owner_uid);
    }

    pub fn event_handler_deck(&self) -> EventHandlerDeck {
        self.species.event_handler_deck
    }
    
    pub(crate) const fn placeholder() -> AbilityData {
        Self {
            uid: MonsterUID::default(),
            species: AbilitySpecies::default(),
        }
    }
}
