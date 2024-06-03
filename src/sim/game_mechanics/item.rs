use crate::{EventHandlerDeck, MonsterID};

#[derive(Debug, Clone, Copy)]
pub struct Item {
    pub(crate) _id: ItemID,
    pub(crate) species: & 'static ItemSpecies
}

impl Item {

    #[inline(always)]
    pub fn name(&self) -> &'static str {
        &self.species.name
    }

    #[inline(always)]
    pub fn dex_number(&self) -> u16 {
        self.species.dex_number
    }

    #[inline(always)]
    pub fn kind(&self) -> ItemFlags {
        self.species.kind
    }
    
    #[inline(always)]
    pub fn species(&self) -> & 'static ItemSpecies {
        self.species
    }
    
    pub(crate) fn event_handlers(&self) -> EventHandlerDeck {
        (self.species.event_handlers)()
    }   
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemID {
    pub item_holder_id: MonsterID,
}

impl ItemID {
    pub fn from_holder(item_holder_id: MonsterID) -> ItemID {
        ItemID {
            item_holder_id,
        }
    } 
}

#[derive(Debug, Clone, Copy)]
pub struct ItemSpecies {
    pub(crate) dex_number: u16,
    pub(crate) name: & 'static str,
    pub(crate) kind: ItemFlags,
    pub(crate) is_consumable: bool,
    pub(crate) event_handlers: fn() -> EventHandlerDeck,
}

impl ItemSpecies {
    pub const fn from_dex_entry(dex_entry: ItemDexEntry) -> ItemSpecies {
        let ItemDexEntry { 
            dex_number,
            name, 
            kind, 
            is_consumable,
            event_handlers, 
        } = dex_entry;

        ItemSpecies {
            dex_number,
            name,
            kind,
            is_consumable,
            event_handlers,
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

pub struct ItemDexEntry {
    pub dex_number: u16,
    pub name: & 'static str,
    pub kind: ItemFlags,
    pub is_consumable: bool,
    pub event_handlers: fn() -> EventHandlerDeck,
}