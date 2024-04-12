use monsim_utils::Nothing;

use crate::{message_log::MessageLog, sim::event::{BattleAPI, EventFilteringOptions, EventHandlerDeck, EventHandlerStorage, EventResponse, OwnerInfo}, BattleEntities, Event, MonsterUID, TheAbilityActivated};
use core::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ability {
    pub(crate) uid: AbilityUID, 
    pub(crate) species: &'static AbilitySpecies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AbilityUID {
    pub owner: MonsterUID
}

#[derive(Clone, Copy)]
pub struct AbilitySpecies {
    pub dex_number: u16,
    pub name: &'static str,
    /// `fn(api: BattleAPI, activated_ability: TheActivatedAbility)`
    pub on_activate: fn(BattleAPI, TheAbilityActivated),
    /// A function that adds `OwnedEventHandler`s to the Battle's `EventHandlerStorage`
    pub event_handlers: fn() -> EventHandlerDeck,
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
    event_handlers: || { EventHandlerDeck::const_default() },
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

    pub fn activate(&self, api: BattleAPI, context: TheAbilityActivated) {
        (self.species.on_activate)(api, context);
    }
}
