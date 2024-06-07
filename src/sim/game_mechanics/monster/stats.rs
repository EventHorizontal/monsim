use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stat {
    Hp,
    PhysicalAttack,
    PhysicalDefense,
    SpecialAttack,
    SpecialDefense,
    Speed,
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

impl Stat {
    fn to_modifier(&self) -> Option<ModifiableStat> {
        match self {
            Stat::Hp => None,
            Stat::PhysicalAttack => Some(ModifiableStat::PhysicalAttack),
            Stat::PhysicalDefense => Some(ModifiableStat::PhysicalDefense),
            Stat::SpecialAttack => Some(ModifiableStat::SpecialAttack),
            Stat::SpecialDefense => Some(ModifiableStat::SpecialDefense),
            Stat::Speed => Some(ModifiableStat::Speed),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifiableStat {
    PhysicalAttack,
    PhysicalDefense,
    SpecialAttack,
    SpecialDefense,
    Speed,
    Accuracy,
    Evasion,
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
        Self { hp, att, def, spa, spd, spe }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StatModifierSet {
    att: i8,
    def: i8,
    spa: i8,
    spd: i8,
    spe: i8,
    acc: i8,
    eva: i8,
}

impl Display for ModifiableStat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModifiableStat::PhysicalAttack => write!(f, "Attack"),
            ModifiableStat::PhysicalDefense => write!(f, "Defense"),
            ModifiableStat::SpecialAttack => write!(f, "Special Attack"),
            ModifiableStat::SpecialDefense => write!(f, "Special Defense"),
            ModifiableStat::Speed => write!(f, "Speed"),
            ModifiableStat::Accuracy => write!(f, "Accuracy"),
            ModifiableStat::Evasion => write!(f, "Evasion"),
        }
    }
}

impl Index<ModifiableStat> for StatModifierSet {
    type Output = i8;

    fn index(&self, index: ModifiableStat) -> &Self::Output {
        match index {
            ModifiableStat::PhysicalAttack => &self.att,
            ModifiableStat::PhysicalDefense => &self.def,
            ModifiableStat::SpecialAttack => &self.spa,
            ModifiableStat::SpecialDefense => &self.spd,
            ModifiableStat::Speed => &self.spe,
            ModifiableStat::Accuracy => &self.acc,
            ModifiableStat::Evasion => &self.eva,
        }
    }
}

impl IndexMut<ModifiableStat> for StatModifierSet {
    fn index_mut(&mut self, index: ModifiableStat) -> &mut Self::Output {
        match index {
            ModifiableStat::PhysicalAttack => &mut self.att,
            ModifiableStat::PhysicalDefense => &mut self.def,
            ModifiableStat::SpecialAttack => &mut self.spa,
            ModifiableStat::SpecialDefense => &mut self.spd,
            ModifiableStat::Speed => &mut self.spe,
            ModifiableStat::Accuracy => &mut self.acc,
            ModifiableStat::Evasion => &mut self.eva,
        }
    }
}

impl StatModifierSet {
    pub const fn blank() -> Self {
        Self {
            att: 0,
            def: 0,
            spa: 0,
            spd: 0,
            spe: 0,
            acc: 0,
            eva: 0,
        }
    }

    pub fn raise_stat(&mut self, stat: ModifiableStat, by_number_of_stages: u8) -> u8 {
        assert!(
            by_number_of_stages > 0 && by_number_of_stages <= 6,
            "Expected stat change to be between 1 and 6 stages, found {} stages",
            by_number_of_stages
        );
        let effective_stages = (6 - self[stat]).min(by_number_of_stages as i8);
        assert!(effective_stages >= 0);
        self[stat] += effective_stages;
        effective_stages as u8
    }

    pub fn lower_stat(&mut self, stat: ModifiableStat, by_number_of_stages: u8) -> u8 {
        assert!(
            by_number_of_stages > 0 && by_number_of_stages <= 6,
            "Expected stat change to be between 1 and 6 stages, found {} stages",
            by_number_of_stages
        );
        let effective_stages = (6 + self[stat]).min(by_number_of_stages as i8);
        assert!(effective_stages >= 0);
        self[stat] -= effective_stages;
        effective_stages as u8
    }
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
