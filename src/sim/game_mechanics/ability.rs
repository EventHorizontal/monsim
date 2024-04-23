use crate::{sim::{event_dispatch::EventFilteringOptions, EventHandlerDeck, MonsterUID}, AbilityUseContext, BattleSimulator};
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
impl AbilityUID {
    pub(crate) fn _from_owner(ability_owner: MonsterUID) -> AbilityUID {
        AbilityUID { owner: ability_owner }
    }
}

#[derive(Clone, Copy)]
pub struct AbilitySpecies {
    pub dex_number: u16,
    pub name: &'static str,
    /// `fn(battle: &mut Battle, ability_holder: MonsterUID)`
    pub on_activate: fn(&mut BattleSimulator, AbilityUseContext),
    pub event_handlers: fn() -> EventHandlerDeck,
    pub event_filtering_options: EventFilteringOptions,
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
    on_activate: |_,_| {},
    event_handlers: EventHandlerDeck::empty,
    event_filtering_options: EventFilteringOptions::default(),
    order: 0,
};

impl AbilitySpecies {
    pub const fn const_default() -> Self {
        ABILITY_DEFAULTS
    }
}

impl Eq for AbilitySpecies {}

impl Ability {

    pub fn activate(&self, sim: &mut BattleSimulator, ability_use_context: AbilityUseContext) {
        (self.species.on_activate)(sim, ability_use_context);
    }

    pub fn event_handler_deck(&self) -> EventHandlerDeck {
        (self.species.event_handlers)()
    }
}
