use crate::{event::EventHandlerFilters, BattleContext};

use super::{BattlerUID, Debug, EventHandlerSet};

#[derive(Clone, Copy)]
pub struct AbilitySpecies {
    pub dex_number: u16,
    pub name: &'static str,
    pub event_handlers: EventHandlerSet,
    pub on_activate: fn(&mut BattleContext, BattlerUID) -> (),
    pub event_handler_filters: EventHandlerFilters,
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

impl Eq for AbilitySpecies {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ability {
    pub species: AbilitySpecies,
}

impl Ability {
    pub fn new(species: AbilitySpecies) -> Self {
        Ability { species }
    }

    pub fn on_activate(&self, context: &mut BattleContext, owner_uid: BattlerUID) {
        (self.species.on_activate)(context, owner_uid);
    }

    pub fn event_handlers(&self) -> EventHandlerSet {
        self.species.event_handlers
    }
}
