use crate::sim::{event::EventFilteringOptions, Battle, EventHandlerDeck, MonsterRef, MonsterUID};
use core::fmt::Debug;
use std::{cell::Cell, ops::Deref};

#[derive(Debug, Clone, Copy)]
pub struct AbilityRef<'a> {
    ability_data: &'a Ability
}

impl<'a> PartialEq for AbilityRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl<'a> Eq for AbilityRef<'a> {}

impl<'a> Deref for AbilityRef<'a> {
    type Target = Ability;

    fn deref(&self) -> &Self::Target {
        self.ability_data
    }
}

impl<'a> AbilityRef<'a> {
    pub fn new(ability_data: &'a Ability) -> Self {
        Self {
            ability_data
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Ability {
    pub uid: AbilityUID,
    pub species: Cell<AbilitySpecies>,
}

pub type AbilityUID = MonsterUID;

impl Ability {
    pub fn new(owner_uid: AbilityUID, species: AbilitySpecies) -> Self {
        Self {
            uid: owner_uid, 
            species: Cell::new(species), 
        }
    }

    pub fn on_activate(&self, battle: &mut Battle, owner: MonsterRef) {
        (self.species.get().on_activate)(battle, owner);
    }

    pub fn event_handler_deck<'a>(&'a self) -> EventHandlerDeck {
        self.species.get().event_handler_deck
    }
}

#[derive(Clone, Copy)]
pub struct AbilitySpecies {
    pub dex_number: u16,
    pub name: &'static str,
    pub event_handler_deck: EventHandlerDeck,
    /// `fn(battle: &mut Battle, ability_holder: MonsterUID)`
    pub on_activate: for<'b> fn(&mut Battle, MonsterRef<'b>),
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
    event_handler_deck: EventHandlerDeck::const_default(),
    on_activate: |_battle, _ability_holder_uid| {},
    filtering_options: EventFilteringOptions::default(),
    order: 0,
};

impl AbilitySpecies {
    pub const fn const_default() -> Self {
        ABILITY_DEFAULTS
    }
}

impl Eq for AbilitySpecies {}


