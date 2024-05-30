use crate::{EventFilteringOptions, EventHandlerDeck, MonsterID};

#[derive(Debug, Clone, Copy)]
pub struct Item {
    pub(crate) id: ItemID,
    pub(crate) species: & 'static ItemSpecies
}

impl Item {
    pub(crate) fn name(&self) -> &'static str {
        &self.species.name
    }
    
    pub(crate) fn event_handlers(&self) -> EventHandlerDeck {
        (self.species.event_handlers)()
    }
    
    pub(crate) fn event_filtering_options(&self) -> EventFilteringOptions {
        self.species.event_filtering_options
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ItemID {
    pub owner_id: MonsterID,
}

#[derive(Debug, Clone, Copy)]
pub struct ItemSpecies {
    pub(crate) dex_number: u16,
    pub(crate) name: & 'static str,
    pub(crate) kind:  ItemKind,
    pub(crate) event_handlers: fn() -> EventHandlerDeck,
    pub(crate) event_filtering_options: EventFilteringOptions,
}

impl ItemSpecies {
    pub const fn from_dex_data(dex_data: ItemDexData) -> ItemSpecies {
        let ItemDexData { 
            dex_number,
            name, 
            event_handlers, 
            event_filtering_options, 
            kind 
        } = dex_data;

        ItemSpecies {
            dex_number,
            name,
            kind,
            event_handlers,
            event_filtering_options,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ItemKind {
    Misc,
    Berry
}

pub struct ItemDexData {
    pub dex_number: u16,
    pub name: & 'static str,
    pub kind: ItemKind,
    pub event_handlers: fn() -> EventHandlerDeck,
    pub event_filtering_options: EventFilteringOptions,
}