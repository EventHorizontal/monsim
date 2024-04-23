use monsim_utils::Nothing;

use crate::{sim::{event_dispatch::EventFilteringOptions, EventHandlerDeck, MonsterUID}, AbilityUseContext, Effect};
use core::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ability {
    uid: AbilityUID, 
    species: &'static AbilitySpecies,
}

impl Ability {

    pub const fn new(uid: AbilityUID, species: &'static AbilitySpecies) -> Self {
        Self {
            uid,
            species,
        }
    }

    pub fn event_handlers(&self) -> EventHandlerDeck {
        (self.species.event_handlers)()
    }
    
    #[inline(always)]
    pub fn species(&self) -> & 'static AbilitySpecies {
        self.species
    }
    
    #[inline(always)]
    pub fn name(&self) -> &'static str {
        self.species.name
    }
    
    #[inline(always)]
    pub fn order(&self) -> u16 {
        self.species.order
    }
    
    #[inline(always)]
    pub fn dex_number(&self) -> u16 {
        self.species.dex_number
    }
    
    #[inline(always)]
    pub fn on_activate_effect(&self) -> Effect<Nothing, AbilityUseContext> {
        self.species.on_activate_effect
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
    dex_number: u16,
    name: &'static str,
    on_activate_effect: Effect<Nothing, AbilityUseContext>,
    event_handlers: fn() -> EventHandlerDeck,
    event_filtering_options: EventFilteringOptions,
    order: u16,
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

impl AbilitySpecies {
    pub const fn from_dex_data(dex_data: AbilityDexData) -> Self {
        let AbilityDexData { dex_number, name, on_activate_effect, event_handlers, event_filtering_options, order } = dex_data;
        
        Self {
            dex_number,
            name,
            on_activate_effect,
            event_handlers,
            event_filtering_options,
            order,
        }
    }

    #[inline(always)]
    pub fn event_handlers(&self) -> EventHandlerDeck {
        (self.event_handlers)()
    }
    
    #[inline(always)]
    pub fn on_activate_effect(&self) -> Effect<Nothing, AbilityUseContext> {
        self.on_activate_effect
    }
    
    #[inline(always)]
    pub fn name(&self) -> & 'static str {
        self.name
    }
    
    #[inline(always)]
    pub fn event_filtering_options(&self) -> EventFilteringOptions {
        self.event_filtering_options
    }
    
    #[inline(always)]
    pub fn order(&self) -> u16 {
        self.order
    }
    
    #[inline(always)]
    pub fn dex_number(&self) -> u16 {
        self.dex_number
    }
    
}

pub struct AbilityDexData {
    pub dex_number: u16,
    pub name: &'static str,
    pub on_activate_effect: Effect<Nothing, AbilityUseContext>,
    pub event_handlers: fn() -> EventHandlerDeck,
    pub event_filtering_options: EventFilteringOptions,
    pub order: u16,
}