use std::ops::Mul;

use monsim_utils::Percent;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Type {
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
    Normal,
    Poison,
    Psychic,
    Rock,
    Steel,
    Water,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeEffectiveness {
    Ineffective,
    DoubleNotVeryEffective,
    NotVeryEffective,
    Effective,
    SuperEffective,
    DoubleSuperEffective,
}

impl Into<Percent> for TypeEffectiveness {
    fn into(self) -> Percent {
        match self {
            TypeEffectiveness::Ineffective => Percent(0),
            TypeEffectiveness::DoubleNotVeryEffective => Percent(25),
            TypeEffectiveness::NotVeryEffective => Percent(50),
            TypeEffectiveness::Effective => Percent(100),
            TypeEffectiveness::SuperEffective => Percent(200),
            TypeEffectiveness::DoubleSuperEffective => Percent(400),
        }
    }
}

impl From<Percent> for TypeEffectiveness {
    fn from(other: Percent) -> TypeEffectiveness {
        match other {
            Percent(0) => TypeEffectiveness::Ineffective,
            Percent(25) => TypeEffectiveness::DoubleNotVeryEffective,
            Percent(50) => TypeEffectiveness::NotVeryEffective,
            Percent(100) => TypeEffectiveness::Effective,
            Percent(200) => TypeEffectiveness::SuperEffective,
            Percent(400) => TypeEffectiveness::DoubleSuperEffective,
            _ => panic!(),
        }
    }
}

impl Mul for TypeEffectiveness {
    type Output = TypeEffectiveness;

    fn mul(self, rhs: Self) -> Self::Output {
        let self_as_percent: Percent = self.into();
        let rhs_as_percent: Percent = rhs.into();

        let output = self_as_percent * rhs_as_percent;
        output.into()
    }
}

impl TypeEffectiveness {
    pub fn is_matchup_ineffective(&self) -> bool {
        *self == TypeEffectiveness::Ineffective
    }

    pub fn is_matchup_effective(&self) -> bool {
        *self == TypeEffectiveness::Effective
    }

    pub fn is_matchup_not_very_effective(&self) -> bool {
        *self == TypeEffectiveness::DoubleNotVeryEffective || *self == TypeEffectiveness::NotVeryEffective
    }

    pub fn is_matchup_super_effective(&self) -> bool {
        *self == TypeEffectiveness::SuperEffective || *self == TypeEffectiveness::DoubleSuperEffective
    }
    
    pub(crate) fn as_text(&self) -> String {
        let text = match self {
            TypeEffectiveness::Ineffective => "ineffective",
            TypeEffectiveness::DoubleNotVeryEffective | TypeEffectiveness::NotVeryEffective => "not very effective",
            TypeEffectiveness::Effective => "effective",
            TypeEffectiveness::SuperEffective | TypeEffectiveness::DoubleSuperEffective => "super effective",
        };
        String::from(text)
    }
}

pub fn dual_type_matchup(move_type: Type, (primary_type, maybe_secondary_type): (Type, Option<Type>)) -> TypeEffectiveness {
    if let Some(secondary_type) = maybe_secondary_type {
        type_matchup(move_type, primary_type) * type_matchup(move_type, secondary_type)
    } else {
        type_matchup(move_type, primary_type)
    }
}

pub const fn type_matchup(move_type: Type, target_type: Type) -> TypeEffectiveness {
    match (move_type, target_type) {
        (Type::Bug, Type::Bug) => TypeEffectiveness::Effective,
        (Type::Bug, Type::Dark) => TypeEffectiveness::SuperEffective,
        (Type::Bug, Type::Dragon) => TypeEffectiveness::Effective,
        (Type::Bug, Type::Electric) => TypeEffectiveness::Effective,
        (Type::Bug, Type::Fairy) => TypeEffectiveness::NotVeryEffective,
        (Type::Bug, Type::Fighting) => TypeEffectiveness::NotVeryEffective,
        (Type::Bug, Type::Fire) => TypeEffectiveness::NotVeryEffective,
        (Type::Bug, Type::Flying) => TypeEffectiveness::NotVeryEffective,
        (Type::Bug, Type::Ghost) => TypeEffectiveness::NotVeryEffective,
        (Type::Bug, Type::Grass) => TypeEffectiveness::SuperEffective,
        (Type::Bug, Type::Ground) => TypeEffectiveness::Effective,
        (Type::Bug, Type::Ice) => TypeEffectiveness::Effective,
        (Type::Bug, Type::Poison) => TypeEffectiveness::NotVeryEffective,
        (Type::Bug, Type::Psychic) => TypeEffectiveness::SuperEffective,
        (Type::Bug, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Bug, Type::Rock) => TypeEffectiveness::Effective,
        (Type::Bug, Type::Steel) => TypeEffectiveness::NotVeryEffective,
        (Type::Bug, Type::Water) => TypeEffectiveness::Effective,

        (Type::Dark, Type::Bug) => TypeEffectiveness::Effective,
        (Type::Dark, Type::Dark) => TypeEffectiveness::NotVeryEffective,
        (Type::Dark, Type::Dragon) => TypeEffectiveness::Effective,
        (Type::Dark, Type::Electric) => TypeEffectiveness::Effective,
        (Type::Dark, Type::Fairy) => TypeEffectiveness::NotVeryEffective,
        (Type::Dark, Type::Fighting) => TypeEffectiveness::NotVeryEffective,
        (Type::Dark, Type::Fire) => TypeEffectiveness::Effective,
        (Type::Dark, Type::Flying) => TypeEffectiveness::Effective,
        (Type::Dark, Type::Ghost) => TypeEffectiveness::SuperEffective,
        (Type::Dark, Type::Grass) => TypeEffectiveness::Effective,
        (Type::Dark, Type::Ground) => TypeEffectiveness::Effective,
        (Type::Dark, Type::Ice) => TypeEffectiveness::Effective,
        (Type::Dark, Type::Poison) => TypeEffectiveness::Effective,
        (Type::Dark, Type::Psychic) => TypeEffectiveness::SuperEffective,
        (Type::Dark, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Dark, Type::Rock) => TypeEffectiveness::Effective,
        (Type::Dark, Type::Steel) => TypeEffectiveness::Effective,
        (Type::Dark, Type::Water) => TypeEffectiveness::Effective,

        (Type::Dragon, Type::Bug) => TypeEffectiveness::Effective,
        (Type::Dragon, Type::Dark) => TypeEffectiveness::Effective,
        (Type::Dragon, Type::Dragon) => TypeEffectiveness::SuperEffective,
        (Type::Dragon, Type::Electric) => TypeEffectiveness::Effective,
        (Type::Dragon, Type::Fairy) => TypeEffectiveness::Ineffective,
        (Type::Dragon, Type::Fighting) => TypeEffectiveness::Effective,
        (Type::Dragon, Type::Fire) => TypeEffectiveness::Effective,
        (Type::Dragon, Type::Flying) => TypeEffectiveness::Effective,
        (Type::Dragon, Type::Ghost) => TypeEffectiveness::Effective,
        (Type::Dragon, Type::Grass) => TypeEffectiveness::Effective,
        (Type::Dragon, Type::Ground) => TypeEffectiveness::Effective,
        (Type::Dragon, Type::Ice) => TypeEffectiveness::Effective,
        (Type::Dragon, Type::Poison) => TypeEffectiveness::Effective,
        (Type::Dragon, Type::Psychic) => TypeEffectiveness::Effective,
        (Type::Dragon, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Dragon, Type::Rock) => TypeEffectiveness::Effective,
        (Type::Dragon, Type::Steel) => TypeEffectiveness::NotVeryEffective,
        (Type::Dragon, Type::Water) => TypeEffectiveness::Effective,

        (Type::Electric, Type::Bug) => TypeEffectiveness::Effective,
        (Type::Electric, Type::Dark) => TypeEffectiveness::Effective,
        (Type::Electric, Type::Dragon) => TypeEffectiveness::NotVeryEffective,
        (Type::Electric, Type::Electric) => TypeEffectiveness::NotVeryEffective,
        (Type::Electric, Type::Fairy) => TypeEffectiveness::Effective,
        (Type::Electric, Type::Fighting) => TypeEffectiveness::Effective,
        (Type::Electric, Type::Fire) => TypeEffectiveness::Effective,
        (Type::Electric, Type::Flying) => TypeEffectiveness::SuperEffective,
        (Type::Electric, Type::Ghost) => TypeEffectiveness::Effective,
        (Type::Electric, Type::Grass) => TypeEffectiveness::NotVeryEffective,
        (Type::Electric, Type::Ground) => TypeEffectiveness::Ineffective,
        (Type::Electric, Type::Ice) => TypeEffectiveness::Effective,
        (Type::Electric, Type::Poison) => TypeEffectiveness::Effective,
        (Type::Electric, Type::Psychic) => TypeEffectiveness::Effective,
        (Type::Electric, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Electric, Type::Rock) => TypeEffectiveness::Effective,
        (Type::Electric, Type::Steel) => TypeEffectiveness::Effective,
        (Type::Electric, Type::Water) => TypeEffectiveness::SuperEffective,

        (Type::Fairy, Type::Bug) => TypeEffectiveness::Effective,
        (Type::Fairy, Type::Dark) => TypeEffectiveness::SuperEffective,
        (Type::Fairy, Type::Dragon) => TypeEffectiveness::SuperEffective,
        (Type::Fairy, Type::Electric) => TypeEffectiveness::Effective,
        (Type::Fairy, Type::Fairy) => TypeEffectiveness::Effective,
        (Type::Fairy, Type::Fighting) => TypeEffectiveness::SuperEffective,
        (Type::Fairy, Type::Fire) => TypeEffectiveness::NotVeryEffective,
        (Type::Fairy, Type::Flying) => TypeEffectiveness::Effective,
        (Type::Fairy, Type::Ghost) => TypeEffectiveness::Effective,
        (Type::Fairy, Type::Grass) => TypeEffectiveness::Effective,
        (Type::Fairy, Type::Ground) => TypeEffectiveness::Effective,
        (Type::Fairy, Type::Ice) => TypeEffectiveness::Effective,
        (Type::Fairy, Type::Poison) => TypeEffectiveness::Effective,
        (Type::Fairy, Type::Psychic) => TypeEffectiveness::Effective,
        (Type::Fairy, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Fairy, Type::Rock) => TypeEffectiveness::Effective,
        (Type::Fairy, Type::Steel) => TypeEffectiveness::NotVeryEffective,
        (Type::Fairy, Type::Water) => TypeEffectiveness::Effective,

        (Type::Fighting, Type::Bug) => TypeEffectiveness::NotVeryEffective,
        (Type::Fighting, Type::Dark) => TypeEffectiveness::SuperEffective,
        (Type::Fighting, Type::Dragon) => TypeEffectiveness::Effective,
        (Type::Fighting, Type::Electric) => TypeEffectiveness::Effective,
        (Type::Fighting, Type::Fairy) => TypeEffectiveness::NotVeryEffective,
        (Type::Fighting, Type::Fighting) => TypeEffectiveness::Effective,
        (Type::Fighting, Type::Fire) => TypeEffectiveness::Effective,
        (Type::Fighting, Type::Flying) => TypeEffectiveness::NotVeryEffective,
        (Type::Fighting, Type::Ghost) => TypeEffectiveness::Ineffective,
        (Type::Fighting, Type::Grass) => TypeEffectiveness::Effective,
        (Type::Fighting, Type::Ground) => TypeEffectiveness::Effective,
        (Type::Fighting, Type::Ice) => TypeEffectiveness::SuperEffective,
        (Type::Fighting, Type::Poison) => TypeEffectiveness::NotVeryEffective,
        (Type::Fighting, Type::Psychic) => TypeEffectiveness::NotVeryEffective,
        (Type::Fighting, Type::Normal) => TypeEffectiveness::SuperEffective,
        (Type::Fighting, Type::Rock) => TypeEffectiveness::SuperEffective,
        (Type::Fighting, Type::Steel) => TypeEffectiveness::Effective,
        (Type::Fighting, Type::Water) => TypeEffectiveness::Effective,

        (Type::Fire, Type::Bug) => TypeEffectiveness::SuperEffective,
        (Type::Fire, Type::Dark) => TypeEffectiveness::Effective,
        (Type::Fire, Type::Dragon) => TypeEffectiveness::NotVeryEffective,
        (Type::Fire, Type::Electric) => TypeEffectiveness::Effective,
        (Type::Fire, Type::Fairy) => TypeEffectiveness::Effective,
        (Type::Fire, Type::Fighting) => TypeEffectiveness::Effective,
        (Type::Fire, Type::Fire) => TypeEffectiveness::NotVeryEffective,
        (Type::Fire, Type::Flying) => TypeEffectiveness::Effective,
        (Type::Fire, Type::Ghost) => TypeEffectiveness::Effective,
        (Type::Fire, Type::Grass) => TypeEffectiveness::SuperEffective,
        (Type::Fire, Type::Ground) => TypeEffectiveness::Effective,
        (Type::Fire, Type::Ice) => TypeEffectiveness::SuperEffective,
        (Type::Fire, Type::Poison) => TypeEffectiveness::Effective,
        (Type::Fire, Type::Psychic) => TypeEffectiveness::Effective,
        (Type::Fire, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Fire, Type::Rock) => TypeEffectiveness::NotVeryEffective,
        (Type::Fire, Type::Steel) => TypeEffectiveness::SuperEffective,
        (Type::Fire, Type::Water) => TypeEffectiveness::NotVeryEffective,

        (Type::Flying, Type::Bug) => TypeEffectiveness::SuperEffective,
        (Type::Flying, Type::Dark) => TypeEffectiveness::Effective,
        (Type::Flying, Type::Dragon) => TypeEffectiveness::Effective,
        (Type::Flying, Type::Electric) => TypeEffectiveness::NotVeryEffective,
        (Type::Flying, Type::Fairy) => TypeEffectiveness::Effective,
        (Type::Flying, Type::Fighting) => TypeEffectiveness::SuperEffective,
        (Type::Flying, Type::Fire) => TypeEffectiveness::Effective,
        (Type::Flying, Type::Flying) => TypeEffectiveness::Effective,
        (Type::Flying, Type::Ghost) => TypeEffectiveness::Effective,
        (Type::Flying, Type::Grass) => TypeEffectiveness::SuperEffective,
        (Type::Flying, Type::Ground) => TypeEffectiveness::Effective,
        (Type::Flying, Type::Ice) => TypeEffectiveness::Effective,
        (Type::Flying, Type::Poison) => TypeEffectiveness::Effective,
        (Type::Flying, Type::Psychic) => TypeEffectiveness::Effective,
        (Type::Flying, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Flying, Type::Rock) => TypeEffectiveness::NotVeryEffective,
        (Type::Flying, Type::Steel) => TypeEffectiveness::NotVeryEffective,
        (Type::Flying, Type::Water) => TypeEffectiveness::Effective,

        (Type::Ghost, Type::Bug) => TypeEffectiveness::Effective,
        (Type::Ghost, Type::Dark) => TypeEffectiveness::NotVeryEffective,
        (Type::Ghost, Type::Dragon) => TypeEffectiveness::Effective,
        (Type::Ghost, Type::Electric) => TypeEffectiveness::Effective,
        (Type::Ghost, Type::Fairy) => TypeEffectiveness::Effective,
        (Type::Ghost, Type::Fighting) => TypeEffectiveness::Effective,
        (Type::Ghost, Type::Fire) => TypeEffectiveness::Effective,
        (Type::Ghost, Type::Flying) => TypeEffectiveness::Effective,
        (Type::Ghost, Type::Ghost) => TypeEffectiveness::SuperEffective,
        (Type::Ghost, Type::Grass) => TypeEffectiveness::Effective,
        (Type::Ghost, Type::Ground) => TypeEffectiveness::Effective,
        (Type::Ghost, Type::Ice) => TypeEffectiveness::Effective,
        (Type::Ghost, Type::Poison) => TypeEffectiveness::Effective,
        (Type::Ghost, Type::Psychic) => TypeEffectiveness::SuperEffective,
        (Type::Ghost, Type::Normal) => TypeEffectiveness::Ineffective,
        (Type::Ghost, Type::Rock) => TypeEffectiveness::Effective,
        (Type::Ghost, Type::Steel) => TypeEffectiveness::Effective,
        (Type::Ghost, Type::Water) => TypeEffectiveness::Effective,

        (Type::Grass, Type::Bug) => TypeEffectiveness::NotVeryEffective,
        (Type::Grass, Type::Dark) => TypeEffectiveness::Effective,
        (Type::Grass, Type::Dragon) => TypeEffectiveness::Effective,
        (Type::Grass, Type::Electric) => TypeEffectiveness::Effective,
        (Type::Grass, Type::Fairy) => TypeEffectiveness::Effective,
        (Type::Grass, Type::Fighting) => TypeEffectiveness::Effective,
        (Type::Grass, Type::Fire) => TypeEffectiveness::NotVeryEffective,
        (Type::Grass, Type::Flying) => TypeEffectiveness::NotVeryEffective,
        (Type::Grass, Type::Ghost) => TypeEffectiveness::Effective,
        (Type::Grass, Type::Grass) => TypeEffectiveness::Effective,
        (Type::Grass, Type::Ground) => TypeEffectiveness::SuperEffective,
        (Type::Grass, Type::Ice) => TypeEffectiveness::Effective,
        (Type::Grass, Type::Poison) => TypeEffectiveness::NotVeryEffective,
        (Type::Grass, Type::Psychic) => TypeEffectiveness::Effective,
        (Type::Grass, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Grass, Type::Rock) => TypeEffectiveness::SuperEffective,
        (Type::Grass, Type::Steel) => TypeEffectiveness::NotVeryEffective,
        (Type::Grass, Type::Water) => TypeEffectiveness::SuperEffective,

        (Type::Ground, Type::Bug) => TypeEffectiveness::NotVeryEffective,
        (Type::Ground, Type::Dark) => TypeEffectiveness::Effective,
        (Type::Ground, Type::Dragon) => TypeEffectiveness::Effective,
        (Type::Ground, Type::Electric) => TypeEffectiveness::SuperEffective,
        (Type::Ground, Type::Fairy) => TypeEffectiveness::Effective,
        (Type::Ground, Type::Fighting) => TypeEffectiveness::Effective,
        (Type::Ground, Type::Fire) => TypeEffectiveness::SuperEffective,
        (Type::Ground, Type::Flying) => TypeEffectiveness::Ineffective,
        (Type::Ground, Type::Ghost) => TypeEffectiveness::Effective,
        (Type::Ground, Type::Grass) => TypeEffectiveness::NotVeryEffective,
        (Type::Ground, Type::Ground) => TypeEffectiveness::SuperEffective,
        (Type::Ground, Type::Ice) => TypeEffectiveness::Effective,
        (Type::Ground, Type::Poison) => TypeEffectiveness::Effective,
        (Type::Ground, Type::Psychic) => TypeEffectiveness::Effective,
        (Type::Ground, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Ground, Type::Rock) => TypeEffectiveness::SuperEffective,
        (Type::Ground, Type::Steel) => TypeEffectiveness::SuperEffective,
        (Type::Ground, Type::Water) => TypeEffectiveness::Effective,

        (Type::Ice, Type::Bug) => TypeEffectiveness::Effective,
        (Type::Ice, Type::Dark) => TypeEffectiveness::Effective,
        (Type::Ice, Type::Dragon) => TypeEffectiveness::SuperEffective,
        (Type::Ice, Type::Electric) => TypeEffectiveness::Effective,
        (Type::Ice, Type::Fairy) => TypeEffectiveness::Effective,
        (Type::Ice, Type::Fighting) => TypeEffectiveness::Effective,
        (Type::Ice, Type::Fire) => TypeEffectiveness::NotVeryEffective,
        (Type::Ice, Type::Flying) => TypeEffectiveness::SuperEffective,
        (Type::Ice, Type::Ghost) => TypeEffectiveness::Effective,
        (Type::Ice, Type::Grass) => TypeEffectiveness::SuperEffective,
        (Type::Ice, Type::Ground) => TypeEffectiveness::SuperEffective,
        (Type::Ice, Type::Ice) => TypeEffectiveness::NotVeryEffective,
        (Type::Ice, Type::Poison) => TypeEffectiveness::Effective,
        (Type::Ice, Type::Psychic) => TypeEffectiveness::Effective,
        (Type::Ice, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Ice, Type::Rock) => TypeEffectiveness::Effective,
        (Type::Ice, Type::Steel) => TypeEffectiveness::NotVeryEffective,
        (Type::Ice, Type::Water) => TypeEffectiveness::NotVeryEffective,

        (Type::Poison, Type::Bug) => TypeEffectiveness::Effective,
        (Type::Poison, Type::Dark) => TypeEffectiveness::Effective,
        (Type::Poison, Type::Dragon) => TypeEffectiveness::Effective,
        (Type::Poison, Type::Electric) => TypeEffectiveness::Effective,
        (Type::Poison, Type::Fairy) => TypeEffectiveness::SuperEffective,
        (Type::Poison, Type::Fighting) => TypeEffectiveness::Effective,
        (Type::Poison, Type::Fire) => TypeEffectiveness::Effective,
        (Type::Poison, Type::Flying) => TypeEffectiveness::Effective,
        (Type::Poison, Type::Ghost) => TypeEffectiveness::NotVeryEffective,
        (Type::Poison, Type::Grass) => TypeEffectiveness::SuperEffective,
        (Type::Poison, Type::Ground) => TypeEffectiveness::NotVeryEffective,
        (Type::Poison, Type::Ice) => TypeEffectiveness::Effective,
        (Type::Poison, Type::Poison) => TypeEffectiveness::NotVeryEffective,
        (Type::Poison, Type::Psychic) => TypeEffectiveness::Effective,
        (Type::Poison, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Poison, Type::Rock) => TypeEffectiveness::NotVeryEffective,
        (Type::Poison, Type::Steel) => TypeEffectiveness::Ineffective,
        (Type::Poison, Type::Water) => TypeEffectiveness::Effective,

        (Type::Psychic, Type::Bug) => TypeEffectiveness::Effective,
        (Type::Psychic, Type::Dark) => TypeEffectiveness::Ineffective,
        (Type::Psychic, Type::Dragon) => TypeEffectiveness::Effective,
        (Type::Psychic, Type::Electric) => TypeEffectiveness::Effective,
        (Type::Psychic, Type::Fairy) => TypeEffectiveness::Effective,
        (Type::Psychic, Type::Fighting) => TypeEffectiveness::SuperEffective,
        (Type::Psychic, Type::Fire) => TypeEffectiveness::Effective,
        (Type::Psychic, Type::Flying) => TypeEffectiveness::Effective,
        (Type::Psychic, Type::Ghost) => TypeEffectiveness::Effective,
        (Type::Psychic, Type::Grass) => TypeEffectiveness::Effective,
        (Type::Psychic, Type::Ground) => TypeEffectiveness::Effective,
        (Type::Psychic, Type::Ice) => TypeEffectiveness::Effective,
        (Type::Psychic, Type::Poison) => TypeEffectiveness::SuperEffective,
        (Type::Psychic, Type::Psychic) => TypeEffectiveness::NotVeryEffective,
        (Type::Psychic, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Psychic, Type::Rock) => TypeEffectiveness::Effective,
        (Type::Psychic, Type::Steel) => TypeEffectiveness::NotVeryEffective,
        (Type::Psychic, Type::Water) => TypeEffectiveness::Effective,

        (Type::Normal, Type::Bug) => TypeEffectiveness::Effective,
        (Type::Normal, Type::Dark) => TypeEffectiveness::Effective,
        (Type::Normal, Type::Dragon) => TypeEffectiveness::Effective,
        (Type::Normal, Type::Electric) => TypeEffectiveness::Effective,
        (Type::Normal, Type::Fairy) => TypeEffectiveness::Effective,
        (Type::Normal, Type::Fighting) => TypeEffectiveness::Effective,
        (Type::Normal, Type::Fire) => TypeEffectiveness::Effective,
        (Type::Normal, Type::Flying) => TypeEffectiveness::Effective,
        (Type::Normal, Type::Ghost) => TypeEffectiveness::Ineffective,
        (Type::Normal, Type::Grass) => TypeEffectiveness::Effective,
        (Type::Normal, Type::Ground) => TypeEffectiveness::Effective,
        (Type::Normal, Type::Ice) => TypeEffectiveness::Effective,
        (Type::Normal, Type::Poison) => TypeEffectiveness::Effective,
        (Type::Normal, Type::Psychic) => TypeEffectiveness::Effective,
        (Type::Normal, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Normal, Type::Rock) => TypeEffectiveness::NotVeryEffective,
        (Type::Normal, Type::Steel) => TypeEffectiveness::NotVeryEffective,
        (Type::Normal, Type::Water) => TypeEffectiveness::Effective,

        (Type::Rock, Type::Bug) => TypeEffectiveness::SuperEffective,
        (Type::Rock, Type::Dark) => TypeEffectiveness::Effective,
        (Type::Rock, Type::Dragon) => TypeEffectiveness::Effective,
        (Type::Rock, Type::Electric) => TypeEffectiveness::Effective,
        (Type::Rock, Type::Fairy) => TypeEffectiveness::Effective,
        (Type::Rock, Type::Fighting) => TypeEffectiveness::NotVeryEffective,
        (Type::Rock, Type::Fire) => TypeEffectiveness::SuperEffective,
        (Type::Rock, Type::Flying) => TypeEffectiveness::SuperEffective,
        (Type::Rock, Type::Ghost) => TypeEffectiveness::Effective,
        (Type::Rock, Type::Grass) => TypeEffectiveness::Effective,
        (Type::Rock, Type::Ground) => TypeEffectiveness::NotVeryEffective,
        (Type::Rock, Type::Ice) => TypeEffectiveness::SuperEffective,
        (Type::Rock, Type::Poison) => TypeEffectiveness::Effective,
        (Type::Rock, Type::Psychic) => TypeEffectiveness::Effective,
        (Type::Rock, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Rock, Type::Rock) => TypeEffectiveness::Effective,
        (Type::Rock, Type::Steel) => TypeEffectiveness::NotVeryEffective,
        (Type::Rock, Type::Water) => TypeEffectiveness::Effective,

        (Type::Steel, Type::Bug) => TypeEffectiveness::Effective,
        (Type::Steel, Type::Dark) => TypeEffectiveness::Effective,
        (Type::Steel, Type::Dragon) => TypeEffectiveness::Effective,
        (Type::Steel, Type::Electric) => TypeEffectiveness::NotVeryEffective,
        (Type::Steel, Type::Fairy) => TypeEffectiveness::SuperEffective,
        (Type::Steel, Type::Fighting) => TypeEffectiveness::Effective,
        (Type::Steel, Type::Fire) => TypeEffectiveness::NotVeryEffective,
        (Type::Steel, Type::Flying) => TypeEffectiveness::Effective,
        (Type::Steel, Type::Ghost) => TypeEffectiveness::Effective,
        (Type::Steel, Type::Grass) => TypeEffectiveness::Effective,
        (Type::Steel, Type::Ground) => TypeEffectiveness::Effective,
        (Type::Steel, Type::Ice) => TypeEffectiveness::SuperEffective,
        (Type::Steel, Type::Poison) => TypeEffectiveness::Effective,
        (Type::Steel, Type::Psychic) => TypeEffectiveness::Effective,
        (Type::Steel, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Steel, Type::Rock) => TypeEffectiveness::SuperEffective,
        (Type::Steel, Type::Steel) => TypeEffectiveness::NotVeryEffective,
        (Type::Steel, Type::Water) => TypeEffectiveness::NotVeryEffective,

        (Type::Water, Type::Bug) => TypeEffectiveness::Effective,
        (Type::Water, Type::Dark) => TypeEffectiveness::Effective,
        (Type::Water, Type::Dragon) => TypeEffectiveness::NotVeryEffective,
        (Type::Water, Type::Electric) => TypeEffectiveness::Effective,
        (Type::Water, Type::Fairy) => TypeEffectiveness::Effective,
        (Type::Water, Type::Fighting) => TypeEffectiveness::Effective,
        (Type::Water, Type::Fire) => TypeEffectiveness::SuperEffective,
        (Type::Water, Type::Flying) => TypeEffectiveness::Effective,
        (Type::Water, Type::Ghost) => TypeEffectiveness::Effective,
        (Type::Water, Type::Grass) => TypeEffectiveness::NotVeryEffective,
        (Type::Water, Type::Ground) => TypeEffectiveness::SuperEffective,
        (Type::Water, Type::Ice) => TypeEffectiveness::Effective,
        (Type::Water, Type::Poison) => TypeEffectiveness::Effective,
        (Type::Water, Type::Psychic) => TypeEffectiveness::Effective,
        (Type::Water, Type::Normal) => TypeEffectiveness::Effective,
        (Type::Water, Type::Rock) => TypeEffectiveness::SuperEffective,
        (Type::Water, Type::Steel) => TypeEffectiveness::Effective,
        (Type::Water, Type::Water) => TypeEffectiveness::NotVeryEffective,
    }
}
