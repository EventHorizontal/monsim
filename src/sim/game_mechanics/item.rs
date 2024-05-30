use crate::{EventFilteringOptions, EventHandlerDeck, MonsterID};

pub struct Item {
    id: ItemID,
    species: & 'static ItemSpecies
}

pub struct ItemID {
    pub owner_id: MonsterID,
}

pub struct ItemSpecies {
    pub(crate) dex_number: u16,
    pub(crate) name: & 'static str,
    pub(crate) kind:  ItemKind,
    pub(crate) event_handlers: EventHandlerDeck,
    pub(crate) event_filtering_options: EventFilteringOptions,
}
impl ItemSpecies {
    pub(crate) const fn from_dex_data(dex_data: ItemDexData) -> ItemSpecies {
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

pub enum ItemKind {
    Misc,
    Berry
}

pub struct ItemDexData {
    pub dex_number: u16,
    pub name: & 'static str,
    pub kind: ItemKind,
    pub event_handlers: EventHandlerDeck,
    pub event_filtering_options: EventFilteringOptions,
}