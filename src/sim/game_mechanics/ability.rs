use crate::sim::{event::EventFilteringOptions, BattleState, MonsterUID, EventHandlerDeck};
use core::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ability {
    pub species: AbilitySpecies,
}

#[derive(Clone, Copy)]
pub struct AbilitySpecies {
    pub dex_number: u16,
    pub name: &'static str,
    pub event_handler_deck: EventHandlerDeck,
    /// `fn(battle: &mut Battle, ability_holder: MonsterUID)`
    pub on_activate: fn(&mut BattleState, MonsterUID),
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

impl Ability {
    pub fn new(species: AbilitySpecies) -> Self {
        Ability { species }
    }

    pub fn on_activate(&self, battle: &mut BattleState, owner_uid: MonsterUID) {
        (self.species.on_activate)(battle, owner_uid);
    }

    pub fn event_handler_deck(&self) -> EventHandlerDeck {
        self.species.event_handler_deck
    }
}
