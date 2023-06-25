use crate::sim::{event::EventFilterOptions, Battle, BattlerUID, CompositeEventResponder};
use core::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ability {
    pub species: AbilitySpecies,
}

#[derive(Clone, Copy)]
pub struct AbilitySpecies {
    pub dex_number: u16,
    pub name: &'static str,
    pub composite_event_responder: CompositeEventResponder,
    /// `fn(battle: &mut Battle, ability_holder: BattlerUID)`
    pub on_activate: fn(&mut Battle, BattlerUID),
    pub filters: EventFilterOptions,
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

impl Ability {
    pub fn new(species: AbilitySpecies) -> Self {
        Ability { species }
    }

    pub fn on_activate(&self, battle: &mut Battle, owner_uid: BattlerUID) {
        (self.species.on_activate)(battle, owner_uid);
    }

    pub fn composite_event_responder(&self) -> CompositeEventResponder {
        self.species.composite_event_responder
    }
}
