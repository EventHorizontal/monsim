use core::{fmt::Debug, panic};
use std::ops::{Index, IndexMut};

use crate::sim::{CompositeEventResponder, MonType};

#[derive(Clone, Copy)]
pub struct MonsterSpecies {
    pub dex_number: u16,
    pub name: &'static str,
    pub primary_type: MonType,
    pub secondary_type: Option<MonType>,
    pub base_stats: StatSet,
    pub composite_event_responder: CompositeEventResponder,
}

impl Debug for MonsterSpecies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:03} {},\n\t type: {:?}/{:?}",
            self.dex_number, self.name, self.primary_type, self.secondary_type
        )
    }
}

impl PartialEq for MonsterSpecies {
    fn eq(&self, other: &Self) -> bool {
        self.dex_number == other.dex_number
    }
}

impl Eq for MonsterSpecies {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatSet {
    hp: u16,
    att: u16,
    def: u16,
    spa: u16,
    spd: u16,
    spe: u16,
}

impl Index<Stat> for StatSet {
    type Output = u16;

    fn index(&self, index: Stat) -> &Self::Output {
        match index {
            Stat::Hp => &self.hp,
            Stat::PhysicalAttack => &self.att,
            Stat::PhysicalDefense => &self.def,
            Stat::SpecialAttack => &self.spa,
            Stat::SpecialDefense => &self.spd,
            Stat::Speed => &self.spe,
        }
    }
}

impl StatSet {
    pub const fn new(hp: u16, att: u16, def: u16, spa: u16, spd: u16, spe: u16) -> Self {
        Self {
            hp,
            att,
            def,
            spa,
            spd,
            spe,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatModifierSet {
    att: i8,
    def: i8,
    spa: i8,
    spd: i8,
    spe: i8,
}

impl Index<Stat> for StatModifierSet {
    type Output = i8;

    fn index(&self, index: Stat) -> &Self::Output {
        match index {
            Stat::Hp => panic!("Error: StatModifierSet does not have an HP entry and so cannot be indexed by Stat::HP."),
            Stat::PhysicalAttack => &self.att,
            Stat::PhysicalDefense => &self.def,
            Stat::SpecialAttack => &self.spa,
            Stat::SpecialDefense => &self.spd,
            Stat::Speed => &self.spe,
        }
    }
}

impl IndexMut<Stat> for StatModifierSet {
    fn index_mut(&mut self, index: Stat) -> &mut Self::Output {
        match index {
            Stat::Hp => panic!("Error: StatModifierSet does not have an HP entry and so cannot be indexed by Stat::HP."),
            Stat::PhysicalAttack => &mut self.att,
            Stat::PhysicalDefense => &mut self.def,
            Stat::SpecialAttack => &mut self.spa,
            Stat::SpecialDefense => &mut self.spd,
            Stat::Speed => &mut self.spe,
        }
    }
}

impl StatModifierSet {
    pub const fn new(att: i8, def: i8, spa: i8, spd: i8, spe: i8) -> Self {
        Self {
            att,
            def,
            spa,
            spd,
            spe,
        }
    }

    pub fn raise_stat(&mut self, stat: Stat, by_stages: u8) -> u8 {
        // So far stat raises have never been larger than +3, but this might be removed in the
        // interest of innovation.
        assert!(by_stages <= 3);
        let effective_stages = (6 - self[stat]).min(by_stages as i8);
        assert!(effective_stages >= 0);
        self[stat] += effective_stages;
        effective_stages as u8
    }

    pub fn lower_stat(&mut self, stat: Stat, by_stages: u8) -> u8 {
        // So far stat drops have never been larger than -3, but this might be removed in the
        // interest of innovation.
        assert!(by_stages <= 3);
        let effective_stages = (6 + self[stat]).min(by_stages as i8);
        assert!(effective_stages >= 0);
        self[stat] -= effective_stages;
        effective_stages as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stat {
    Hp,
    PhysicalAttack,
    PhysicalDefense,
    SpecialAttack,
    SpecialDefense,
    Speed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Monster {
    pub nickname: &'static str,
    pub level: u16,
    pub max_health: u16,
    pub nature: MonsterNature,
    pub stats: StatSet,
    pub stat_modifiers: StatModifierSet,
    pub current_health: u16,
    pub species: MonsterSpecies,
}

impl Monster {
    pub fn new(species: MonsterSpecies, nickname: &'static str) -> Self {
        let level = 50;
        // TODO: EVs and IVs are hardcoded for now. Decide what to do with this later.
        let iv_in_stat = 31;
        let ev_in_stat = 252;
        // In-game hp-stat determination formula
        let health_stat =
            ((2 * species.base_stats[Stat::Hp] + iv_in_stat + (ev_in_stat / 4)) * level) / 100
                + level
                + 10;
        let nature = MonsterNature::Serious;

        // In-game non-hp-stat determination formula
        let get_non_hp_stat = |stat: Stat| -> u16 {
            // TODO: EVs and IVs are hardcoded for now. Decide what to do with this later.
            let iv_in_stat = 31;
            let ev_in_stat = 252;
            let mut out =
                ((2 * species.base_stats[stat] + iv_in_stat + (ev_in_stat / 4)) * level) / 100 + 5;
            out = f64::floor(out as f64 * nature[stat]) as u16;
            out
        };

        Monster {
            nickname,
            level,
            nature,
            species,
            stats: StatSet {
                hp: health_stat,
                att: get_non_hp_stat(Stat::PhysicalAttack),
                def: get_non_hp_stat(Stat::PhysicalDefense),
                spa: get_non_hp_stat(Stat::SpecialAttack),
                spd: get_non_hp_stat(Stat::SpecialDefense),
                spe: get_non_hp_stat(Stat::Speed),
            },
            stat_modifiers: StatModifierSet {
                att: 0,
                def: 0,
                spa: 0,
                spd: 0,
                spe: 0,
            },
            max_health: health_stat,
            current_health: health_stat,
        }
    }

    pub fn is_type(&self, type_: MonType) -> bool {
        self.species.primary_type == type_ || self.species.secondary_type == Some(type_)
    }

    pub fn composite_event_responder(&self) -> CompositeEventResponder {
        self.species.composite_event_responder
    }

    pub(crate) fn fainted(&self) -> bool {
        self.current_health == 0
    }

    pub fn name(&self) -> String {
        self.nickname.to_owned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattlerNumber {
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
}

impl From<usize> for BattlerNumber {
    fn from(value: usize) -> Self {
        match value {
            0 => BattlerNumber::_1,
            1 => BattlerNumber::_2,
            2 => BattlerNumber::_3,
            3 => BattlerNumber::_4,
            4 => BattlerNumber::_5,
            5 => BattlerNumber::_6,
            _ => panic!("BattlerNumber can only be formed from usize 0 to 5."),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterNature {
    Hardy,
    Lonely,
    Brave,
    Adamant,
    Naughty,
    Bold,
    Docile,
    Relaxed,
    Impish,
    Lax,
    Timid,
    Hasty,
    Serious,
    Jolly,
    Naive,
    Modest,
    Mild,
    Quiet,
    Bashful,
    Rash,
    Calm,
    Gentle,
    Sassy,
    Careful,
    Quirky,
}

impl Index<Stat> for MonsterNature {
    type Output = f64;

    fn index(&self, index: Stat) -> &Self::Output {
        match self {
            MonsterNature::Hardy => &1.0,

            MonsterNature::Lonely => match index {
                Stat::PhysicalAttack => &1.1,
                Stat::PhysicalDefense => &0.9,
                _ => &1.0,
            },

            MonsterNature::Brave => match index {
                Stat::PhysicalAttack => &1.1,
                Stat::Speed => &0.9,
                _ => &1.0,
            },

            MonsterNature::Adamant => match index {
                Stat::PhysicalAttack => &1.1,
                Stat::SpecialAttack => &0.9,
                _ => &1.0,
            },

            MonsterNature::Naughty => match index {
                Stat::PhysicalAttack => &1.1,
                Stat::SpecialDefense => &0.9,
                _ => &1.0,
            },

            MonsterNature::Docile => &1.0,

            MonsterNature::Bold => match index {
                Stat::PhysicalDefense => &1.1,
                Stat::PhysicalAttack => &0.9,
                _ => &1.0,
            },

            MonsterNature::Relaxed => match index {
                Stat::PhysicalDefense => &1.1,
                Stat::Speed => &0.9,
                _ => &1.0,
            },

            MonsterNature::Impish => match index {
                Stat::PhysicalDefense => &1.1,
                Stat::SpecialAttack => &0.9,
                _ => &1.0,
            },

            MonsterNature::Lax => match index {
                Stat::PhysicalDefense => &1.1,
                Stat::SpecialAttack => &0.9,
                _ => &1.0,
            },

            MonsterNature::Serious => &1.0,

            MonsterNature::Timid => match index {
                Stat::Speed => &1.1,
                Stat::PhysicalAttack => &0.9,
                _ => &1.0,
            },

            MonsterNature::Hasty => match index {
                Stat::Speed => &1.1,
                Stat::PhysicalDefense => &0.9,
                _ => &1.0,
            },

            MonsterNature::Jolly => match index {
                Stat::Speed => &1.1,
                Stat::SpecialAttack => &0.9,
                _ => &1.0,
            },

            MonsterNature::Naive => match index {
                Stat::Speed => &1.1,
                Stat::SpecialDefense => &0.9,
                _ => &1.0,
            },

            MonsterNature::Bashful => &1.0,

            MonsterNature::Modest => match index {
                Stat::SpecialAttack => &1.1,
                Stat::PhysicalAttack => &0.9,
                _ => &1.0,
            },

            MonsterNature::Mild => match index {
                Stat::SpecialAttack => &1.1,
                Stat::PhysicalDefense => &0.9,
                _ => &1.0,
            },

            MonsterNature::Quiet => match index {
                Stat::SpecialAttack => &1.1,
                Stat::Speed => &0.9,
                _ => &1.0,
            },

            MonsterNature::Rash => match index {
                Stat::SpecialAttack => &1.1,
                Stat::SpecialDefense => &0.9,
                _ => &1.0,
            },

            MonsterNature::Quirky => &1.0,

            MonsterNature::Calm => match index {
                Stat::SpecialDefense => &1.1,
                Stat::PhysicalAttack => &0.9,
                _ => &1.0,
            },

            MonsterNature::Gentle => match index {
                Stat::SpecialDefense => &1.1,
                Stat::PhysicalDefense => &0.9,
                _ => &1.0,
            },

            MonsterNature::Sassy => match index {
                Stat::SpecialDefense => &1.1,
                Stat::Speed => &0.9,
                _ => &1.0,
            },

            MonsterNature::Careful => match index {
                Stat::SpecialDefense => &1.1,
                Stat::SpecialAttack => &0.9,
                _ => &1.0,
            },
        }
    }
}
