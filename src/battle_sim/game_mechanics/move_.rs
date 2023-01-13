use core::slice::Iter;
use std::ops::Index;
use crate::battle_sim::event::EventHandlerFilters;

use super::{Debug, EventHandlerSet, MonType};

#[derive(Clone, Copy)]
pub struct MoveSpecies {
    pub dex_number: u16,
    pub name: &'static str,
    pub type_: MonType,
    pub category: MoveCategory,
    pub base_power: u16,
    pub base_accuracy: u16,
    pub priority: u16,
    pub event_handlers: EventHandlerSet,
    pub event_handler_filters: EventHandlerFilters
}

impl Debug for MoveSpecies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:03} {},\n\t type: {:?},\n\t base accuracy: {}",
            self.dex_number, self.name, self.type_, self.base_accuracy
        )
    }
}

impl PartialEq for MoveSpecies {
    fn eq(&self, other: &Self) -> bool {
        self.dex_number == other.dex_number
    }
}

impl Eq for MoveSpecies {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub species: MoveSpecies,
}

impl Move {
    pub fn new(species: MoveSpecies) -> Self {
        Move {
            species
        }
    }

    pub fn category(&self) -> MoveCategory {
        self.species.category
    }

    pub fn base_power(&self) -> u16 {
        self.species.base_power
    }

    pub fn base_accuracy(&self) -> u16 {
        self.species.base_accuracy
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveCategory {
    Physical,
    Special,
    Status
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveSet {
    moves: [Option<Move>; 4],
}

impl Index<usize> for MoveSet {
    type Output = Option<Move>;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < 4, "MoveSet can only be indexed with 0-3. The index passed was {}", index);
        &self.moves[index]
    }
}

impl MoveSet {
    pub fn new(moves: [Option<Move>; 4]) -> Self {
        assert!(moves.first() != None, "There is no first move.");
        return MoveSet { moves };
    }

    pub fn moves(&self) -> Iter<Option<Move>> {
        return self.moves.iter();
    }

    pub fn move_(&self, id: MoveNumber) -> &Option<Move> {
        &self.moves.get(id as usize)
            .expect(&format!["The move at the {:?} index should exist.", id])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveNumber {
    First,
    Second,
    Third,
    Fourth,
}
