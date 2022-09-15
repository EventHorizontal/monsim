#![allow(non_upper_case_globals)]

// Modules
pub mod monster_dex;
pub mod move_dex;

pub type BattlerID = u8;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveID {
    First,
    Second,
    Third,
    Fourth,
}

pub enum TeamID {
    Ally,
    Opponent,
    Environment
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MonType {
    None, // Reserved for empty type slots

    Bug,
    Dark,
    Dragon,
    Electric,
    Fairy,
    Fighting,
    Fire,
    Flying,
    Ghost,
    Grass,
    Ground,
    Ice,
    Poison,
    Psychic,
    Normal,
    Rock,
    Steel,
    Water,
}

#[derive(Clone, Copy, Debug)]
pub struct Stats {
    pub hp: u16,
    pub att: u16,
    pub def: u16,
    pub spa: u16,
    pub spd: u16,
    pub spe: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct EventHandlerSet {
    pub on_try_move: Option<fn() -> bool>,
    pub on_move_calc_accuracy_stages: Option<fn() -> i8>,
    pub on_move_calc_evasion_stages: Option<fn() -> i8>,
    pub on_calc_move_type: Option<fn() -> MonType>,
    pub on_calc_type_multiplier: Option<fn() -> f32>,
    pub on_calc_move_power: Option<fn() -> u16>,
    pub on_calc_attacking_stat: Option<fn() -> u16>,
    pub on_calc_defending_stat: Option<fn() -> u16>,
    pub on_calc_crit_multiplier: Option<fn() -> f32>,
}

impl EventHandlerSet {
    pub const fn default() -> Self {
        EventHandlerSet {
            on_try_move: None,
            on_move_calc_accuracy_stages: None,
            on_move_calc_evasion_stages: None,
            on_calc_move_type: None,
            on_calc_type_multiplier: None,
            on_calc_move_power: None,
            on_calc_attacking_stat: None,
            on_calc_defending_stat: None,
            on_calc_crit_multiplier: None,
        }
    }
}

pub trait Entity {
    fn get_battler_id(&self) -> BattlerID;
    fn get_event_handlers(&self) -> EventHandlerSet;
    fn active(&self) -> bool;
}

// Monster //

#[derive(Clone, Copy, Debug)]
pub struct Monster {
    pub battler_id: BattlerID,
    pub event_handlers: EventHandlerSet,
    pub active: bool,
    pub nickname: &'static str,
    pub primary_type: MonType,
    pub secondary_type: MonType,
    pub level: u8,
    pub current_health: u16,
    pub max_health: u16,
    pub accuracy_stages: i8,
    pub evasion_stages: i8,
    pub stats: Stats,
    pub species: MonsterSpecies,
}


impl Monster {
    pub fn from_species(
        species: MonsterSpecies,
        battler_id: BattlerID,
        nickname: &'static str,
    ) -> Self {
        let level: u8 = 50;
        let max_health = num::integer::div_floor(
            (2 * species.base_stats.hp + 31 + num::integer::div_floor(252, 4))
                * (level as u16),
            100,
        ) + (level as u16)
            + 10;
        // TODO: Implement nature multipliers.
        let nature_multiplier = 1.0f64;

        let get_non_hp_stat = |stat: u16| -> u16 {
            let mut out = num::integer::div_floor(
                2 * stat + 31 + num::integer::div_floor(252, 4) * (level as u16),
                100u16,
            ) + 5;
            out = math::round::floor(out as f64 * nature_multiplier, 0) as u16;
            out
        };

        Monster {
            battler_id,
            event_handlers: species.event_handlers,
            active: false,
            nickname,
            primary_type: species.primary_type,
            secondary_type: species.secondary_type,
            level,
            max_health,
            current_health: max_health,
            accuracy_stages: 0,
            evasion_stages: 0,
            stats: Stats {
                hp: max_health,
                att: get_non_hp_stat(species.base_stats.att),
                def: get_non_hp_stat(species.base_stats.def),
                spa: get_non_hp_stat(species.base_stats.spa),
                spd: get_non_hp_stat(species.base_stats.spd),
                spe: get_non_hp_stat(species.base_stats.spe),
            },
            species,
        }
    }

    pub fn set_active(&mut self, on: bool) -> Self {
        self.active = on;
        *self
    }
}

impl Entity for Monster {
    fn get_battler_id(&self) -> BattlerID {
        self.battler_id
    }

    fn get_event_handlers(&self) -> EventHandlerSet {
        self.event_handlers
    }

    fn active(&self) -> bool {
        self.active
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MonsterSpecies {
    dex: u16,
    name: &'static str,
    primary_type: MonType,
    secondary_type: MonType,
    base_stats: Stats,
    event_handlers: EventHandlerSet,
}

// Move //

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub battler_id: BattlerID,
    pub move_id: MoveID,
    pub event_handlers: EventHandlerSet,
    pub active: bool,
    pub name: &'static str,
    pub _type: MonType,
    pub base_power: u16,
    pub base_accuracy: u8,
    pub species: MoveSpecies,
}


impl Move {
    pub fn from_species(
        species: MoveSpecies,
        battler_id: BattlerID,
        move_id: MoveID,
    ) -> Self {
        Move {
            battler_id,
            move_id,
            event_handlers: species.event_handlers,
            active: false,
            name: species.name,
            _type: species._type,
            base_power: species.base_power,
            base_accuracy: species.base_accuracy,
            species,
        }
    }

    pub fn set_active(&mut self, on: bool) -> Self {
        self.active = on;
        *self
    }
}

impl Entity for Move {
    fn get_battler_id(&self) -> BattlerID {
        self.battler_id
    }

    fn get_event_handlers(&self) -> EventHandlerSet {
        self.event_handlers
    }

    fn active(&self) -> bool {
        self.active
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MoveSpecies {
    pub dex: u16,
    pub name: &'static str,
    pub _type: MonType,
    pub base_power: u16,
    pub base_accuracy: u8,
    pub event_handlers: EventHandlerSet,
}