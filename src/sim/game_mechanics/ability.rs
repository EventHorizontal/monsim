use monsim_utils::Nothing;

use crate::{sim::{event_dispatch::EventFilteringOptions, EventHandlerDeck, MonsterUID}, AbilityUseContext, BattleSimulator, Effect};
use core::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ability {
    pub(crate) uid: AbilityUID, 
    pub(crate) species: &'static AbilitySpecies,
}

impl Ability {

    pub fn activate(&self, sim: &mut BattleSimulator, ability_use_context: AbilityUseContext) {
        (self.species.on_activate_effect)(sim, ability_use_context);
    }

    pub fn event_handler_deck(&self) -> EventHandlerDeck {
        (self.species.event_handlers)()
    }
    
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
    pub on_activate_effect: Effect<Nothing, AbilityUseContext>,
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

impl Eq for AbilitySpecies {}
