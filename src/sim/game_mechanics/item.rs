use std::default;

use monsim_utils::Outcome;

use crate::{EventFilteringOptions, EventHandlerDeck, MonsterID};

#[derive(Debug, Clone, Copy)]
pub struct Item {
    pub(crate) id: ItemID,
    pub(crate) state: ItemState,
    pub(crate) species: & 'static ItemSpecies
}

impl Item {
    pub(crate) fn name(&self) -> &'static str {
        &self.species.name
    }

    pub fn consume(&mut self) -> Outcome {
        if self.species.consumable && self.state == ItemState::Active {
            self.state = ItemState::Consumed;
            Outcome::Success
        } else {
            Outcome::Failure
        }
    }
    
    pub(crate) fn event_handlers(&self) -> EventHandlerDeck {
        (self.species.event_handlers)()
    }
    
    pub(crate) fn event_filtering_options(&self) -> EventFilteringOptions {
        self.species.event_filtering_options
    }
}

/*
    Maybe we will want to replace "Destroyed" with just setting item to None? I mean if
    the item is destroyed it can't be recycled anyway. I'm also thinking of moving consumed
    items to a "consumption history" rather than showing the Monster as having a consumed item.
    The thought process behind this is basically, "what if you give the Monster a different item
    via Trick or something else and then override the item slot?". So then Recycle should fail
        if the tricked item
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ItemState {
    #[default]
    Active,
    Consumed,
    Destroyed
}

#[derive(Debug, Clone, Copy)]
pub struct ItemID {
    pub owner_id: MonsterID,
}

impl ItemID {
    pub fn from_owner(owner_id: MonsterID) -> ItemID {
        ItemID {
            owner_id,
        }
    } 
}

#[derive(Debug, Clone, Copy)]
pub struct ItemSpecies {
    pub(crate) dex_number: u16,
    pub(crate) name: & 'static str,
    pub(crate) kind: ItemFlags,
    pub(crate) consumable: bool,
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
            kind,
            consumable, 
        } = dex_data;

        ItemSpecies {
            dex_number,
            name,
            kind,
            consumable,
            event_handlers,
            event_filtering_options,
        }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ItemFlags: u8 {
        const _     = 0b1111_1111;
        
        const NONE  = 0b0000_0000; 
        const BERRY = 0b0000_0001; 
    }
}

pub struct ItemDexData {
    pub dex_number: u16,
    pub name: & 'static str,
    pub kind: ItemFlags,
    pub consumable: bool,
    pub event_handlers: fn() -> EventHandlerDeck,
    pub event_filtering_options: EventFilteringOptions,
}