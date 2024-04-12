use crate::{sim::{event::EventFilteringOptions, BattleState, EventHandlerDeck, MonsterUID}, AbilityUseContext, DEFAULT_DECK};
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
    /// `fn(battle: &mut Battle, ability_holder: MonsterUID)`
    pub on_activate: fn(&mut BattleState, AbilityUseContext),
    pub event_handler_deck: &'static EventHandlerDeck,
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
    event_handler_deck: &DEFAULT_DECK,
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

impl Ability {

    pub fn activate(&self, battle: &mut BattleState, ability_use_context: AbilityUseContext) {
        (self.species.on_activate)(battle, ability_use_context);
    }

    pub fn event_handler_deck(&self) -> &'static EventHandlerDeck {
        self.species.event_handler_deck
    }
}
