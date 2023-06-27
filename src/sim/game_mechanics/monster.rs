use core::{fmt::Debug, panic};
use std::{ops::{Index, IndexMut}, fmt::Display};

use crate::sim::{CompositeEventResponder, ElementalType};

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

#[derive(Clone, Copy)]
pub struct MonsterSpecies {
    pub dex_number: u16,
    pub name: &'static str,
    pub primary_type: ElementalType,
    pub secondary_type: Option<ElementalType>,
    pub base_stats: StatSet,
    pub composite_event_responder: CompositeEventResponder,
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
pub struct StatSet {
    hp: u16,
    att: u16,
    def: u16,
    spa: u16,
    spd: u16,
    spe: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatModifierSet {
    att: i8,
    def: i8,
    spa: i8,
    spd: i8,
    spe: i8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterNature {
    /// Neutral (+-Attack)
    Hardy, 
    /// +Attack, -Defense
    Lonely,
    /// +Attack, -Speed
    Brave,  
    /// +Attack, -Special Attack
    Adamant, 
    /// +Attack, -Special Defense
    Naughty, 
    
    /// Neutral (+-Defense)
    Docile,
    /// +Defense, -Attack
    Bold, 
    /// +Defense, -Speed
    Relaxed, 
    /// +Defense, -Special Attack
    Impish, // - Special Attack
    /// +Defense, -Special Defense
    Lax, // - Special Defense
    
    /// Neutral (+-Speed)
    Serious,
    /// +Speed, -Attack
    Timid, 
    /// +Speed, -Defense
    Hasty, 
    /// +Speed, -Special Attack
    Jolly,  
    /// +Speed, -Special Defense
    Naive,
    
    /// Neutral (+-Special Attack)
    Bashful,
    /// +Special Attack, -Attack
    Modest,
    /// +Special Attack, -Defense
    Mild,
    /// +Special Attack, -Speed
    Quiet,
    /// +Special Attack, -Special Defense
    Rash,
    
    /// Neutral (+-Special Defense)
    Quirky,
    /// +Special Defense, -Attack
    Calm,
    /// +Special Defense, -Defense
    Gentle,
    /// +SpecialDefense, -Speed
    Sassy, 
    /// +SpecialDefense, -Special Attack
    Careful,
    
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

    pub fn is_type(&self, test_elemental_type: ElementalType) -> bool {
        self.species.primary_type == test_elemental_type || self.species.secondary_type == Some(test_elemental_type)
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
        // So far stat rises have never been larger than +3, but this might be removed in the
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


impl Display for Stat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stat::Hp => write!(f, "HP"),
            Stat::PhysicalAttack => write!(f, "Attack"),
            Stat::PhysicalDefense => write!(f, "Defense"),
            Stat::SpecialAttack => write!(f, "Special Attack"),
            Stat::SpecialDefense => write!(f, "Special Defense"),
            Stat::Speed => write!(f, "Speed"),
        }
    }
}


impl Index<Stat> for MonsterNature {
    type Output = f64;

    fn index(&self, index: Stat) -> &Self::Output {
        const LOWERED: f64 = 0.9;
        const NEUTRAL: f64 = 1.0;
        const RAISED: f64 = 1.1;
        
        match self {
            MonsterNature::Hardy => &NEUTRAL,

            MonsterNature::Lonely => match index {
                Stat::PhysicalAttack => &RAISED,
                Stat::PhysicalDefense => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Brave => match index {
                Stat::PhysicalAttack => &RAISED,
                Stat::Speed => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Adamant => match index {
                Stat::PhysicalAttack => &RAISED,
                Stat::SpecialAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Naughty => match index {
                Stat::PhysicalAttack => &RAISED,
                Stat::SpecialDefense => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Docile => &NEUTRAL,

            MonsterNature::Bold => match index {
                Stat::PhysicalDefense => &RAISED,
                Stat::PhysicalAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Relaxed => match index {
                Stat::PhysicalDefense => &RAISED,
                Stat::Speed => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Impish => match index {
                Stat::PhysicalDefense => &RAISED,
                Stat::SpecialAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Lax => match index {
                Stat::PhysicalDefense => &RAISED,
                Stat::SpecialAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Serious => &NEUTRAL,

            MonsterNature::Timid => match index {
                Stat::Speed => &RAISED,
                Stat::PhysicalAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Hasty => match index {
                Stat::Speed => &RAISED,
                Stat::PhysicalDefense => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Jolly => match index {
                Stat::Speed => &RAISED,
                Stat::SpecialAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Naive => match index {
                Stat::Speed => &RAISED,
                Stat::SpecialDefense => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Bashful => &NEUTRAL,

            MonsterNature::Modest => match index {
                Stat::SpecialAttack => &RAISED,
                Stat::PhysicalAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Mild => match index {
                Stat::SpecialAttack => &RAISED,
                Stat::PhysicalDefense => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Quiet => match index {
                Stat::SpecialAttack => &RAISED,
                Stat::Speed => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Rash => match index {
                Stat::SpecialAttack => &RAISED,
                Stat::SpecialDefense => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Quirky => &NEUTRAL,

            MonsterNature::Calm => match index {
                Stat::SpecialDefense => &RAISED,
                Stat::PhysicalAttack => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Gentle => match index {
                Stat::SpecialDefense => &RAISED,
                Stat::PhysicalDefense => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Sassy => match index {
                Stat::SpecialDefense => &RAISED,
                Stat::Speed => &LOWERED,
                _ => &NEUTRAL,
            },

            MonsterNature::Careful => match index {
                Stat::SpecialDefense => &RAISED,
                Stat::SpecialAttack => &LOWERED,
                _ => &NEUTRAL,
            },
        }
    }
}
