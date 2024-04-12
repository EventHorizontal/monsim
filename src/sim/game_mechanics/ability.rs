use monsim_utils::Nothing;

use crate::{sim::event::{BattleAPI, EventFilteringOptions, EventHandlerStorage, OwnerInfo}, BattleEntities, MonsterUID};
use core::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ability {
    pub(crate) uid: AbilityUID, 
    pub(crate) species: &'static AbilitySpecies,
}

pub type AbilityUID = MonsterUID;

#[derive(Clone, Copy)]
pub struct AbilitySpecies {
    pub dex_number: u16,
    pub name: &'static str,
    /// `fn(battle: &mut Battle, ability_holder: MonsterUID)`
    pub on_activate: fn(BattleAPI, MonsterUID),
    /// A function that adds `OwnedEventHandler`s to the Battle's `EventHandlerStorage`
    pub event_callbacks: fn(OwnerInfo, &mut EventHandlerStorage) -> Nothing,
    pub filtering_options: EventFilteringOptions,
    pub order: u16,
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

const ABILITY_DEFAULTS: AbilitySpecies = AbilitySpecies {
    dex_number: 000,
    name: "Unnamed",
    event_callbacks: |_, _| {},
    on_activate: |_,_| {},
    filtering_options: EventFilteringOptions::default(),
    order: 0,
};

impl AbilitySpecies {
    pub const fn const_default() -> Self {
        ABILITY_DEFAULTS
    }
}

impl Eq for AbilitySpecies {}

impl Ability {

    pub fn on_activate(&self, battle: &mut BattleEntities, owner_uid: MonsterUID) {
        (self.species.on_activate)(battle, owner_uid);
    }

    // pub fn event_handler_deck(&self) -> &'static EventHandlerDeck {
    //     self.species.event_handler_deck
    // }
}
