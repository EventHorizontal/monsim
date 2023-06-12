pub const SUCCESS: bool = true;
pub const FAILURE: bool = false;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    Success,
    Failure
}

impl From<bool> for Outcome {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Success,
            false => Self::Failure,
        }
    }
}

impl Into<bool> for Outcome {
    fn into(self) -> bool {
        match self {
            Outcome::Success => true,
            Outcome::Failure => false,
        }
    }
}

impl Not for Outcome {
    type Output = Outcome;

    fn not(self) -> Self::Output {
        match self {
            Outcome::Success => Outcome::Failure,
            Outcome::Failure => Outcome::Success,
        }
    }
}

/// A percentage between 0 and 100
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Percent(u16);

impl Percent {
    pub fn new(value: u16) -> Self {
        assert!(value <= 100, "Percentage must be between 0 and 100");
        Self(value)
    }
}

impl Add for Percent {
    type Output = Percent;

    fn add(self, rhs: Self) -> Self::Output {
        (Percent(self.0 + rhs.0)).min(Percent(100))
    }
}

impl Sub for Percent {
    type Output = Percent;

    fn sub(self, rhs: Self) -> Self::Output {
        Percent(self.0.saturating_sub(rhs.0))
    }
}

impl<T: Into<f64>> Mul<T> for Percent {
    type Output = f64;

    fn mul(self, rhs: T) -> Self::Output {
        rhs.into() * (self.0 as f64 / 100.0f64)
    }
}

use std::ops::{Not, Add, Sub, Mul};

use super::MonsterType;

pub const INEFFECTIVE: f64 = 0.0;
pub const NOT_VERY_EFFECTIVE: f64 = 0.5;
pub const EFFECTIVE: f64 = 1.0;
pub const SUPER_EFFECTIVE: f64 = 2.0;

pub const EMPTY_LINE: &str = "";

pub const fn type_matchup(move_type: MonsterType, target_type: MonsterType) -> f64 {
    match (move_type, target_type) {
        (MonsterType::Bug, MonsterType::Bug) => EFFECTIVE,
        (MonsterType::Bug, MonsterType::Dark) => SUPER_EFFECTIVE,
        (MonsterType::Bug, MonsterType::Dragon) => EFFECTIVE,
        (MonsterType::Bug, MonsterType::Electric) => EFFECTIVE,
        (MonsterType::Bug, MonsterType::Fairy) => NOT_VERY_EFFECTIVE,
        (MonsterType::Bug, MonsterType::Fighting) => NOT_VERY_EFFECTIVE,
        (MonsterType::Bug, MonsterType::Fire) => NOT_VERY_EFFECTIVE,
        (MonsterType::Bug, MonsterType::Flying) => NOT_VERY_EFFECTIVE,
        (MonsterType::Bug, MonsterType::Ghost) => NOT_VERY_EFFECTIVE,
        (MonsterType::Bug, MonsterType::Grass) => SUPER_EFFECTIVE,
        (MonsterType::Bug, MonsterType::Ground) => EFFECTIVE,
        (MonsterType::Bug, MonsterType::Ice) => EFFECTIVE,
        (MonsterType::Bug, MonsterType::Poison) => NOT_VERY_EFFECTIVE,
        (MonsterType::Bug, MonsterType::Psychic) => SUPER_EFFECTIVE,
        (MonsterType::Bug, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Bug, MonsterType::Rock) => EFFECTIVE,
        (MonsterType::Bug, MonsterType::Steel) => NOT_VERY_EFFECTIVE,
        (MonsterType::Bug, MonsterType::Water) => EFFECTIVE,

        (MonsterType::Dark, MonsterType::Bug) => EFFECTIVE,
        (MonsterType::Dark, MonsterType::Dark) => NOT_VERY_EFFECTIVE,
        (MonsterType::Dark, MonsterType::Dragon) => EFFECTIVE,
        (MonsterType::Dark, MonsterType::Electric) => EFFECTIVE,
        (MonsterType::Dark, MonsterType::Fairy) => NOT_VERY_EFFECTIVE,
        (MonsterType::Dark, MonsterType::Fighting) => NOT_VERY_EFFECTIVE,
        (MonsterType::Dark, MonsterType::Fire) => EFFECTIVE,
        (MonsterType::Dark, MonsterType::Flying) => EFFECTIVE,
        (MonsterType::Dark, MonsterType::Ghost) => SUPER_EFFECTIVE,
        (MonsterType::Dark, MonsterType::Grass) => EFFECTIVE,
        (MonsterType::Dark, MonsterType::Ground) => EFFECTIVE,
        (MonsterType::Dark, MonsterType::Ice) => EFFECTIVE,
        (MonsterType::Dark, MonsterType::Poison) => EFFECTIVE,
        (MonsterType::Dark, MonsterType::Psychic) => SUPER_EFFECTIVE,
        (MonsterType::Dark, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Dark, MonsterType::Rock) => EFFECTIVE,
        (MonsterType::Dark, MonsterType::Steel) => EFFECTIVE,
        (MonsterType::Dark, MonsterType::Water) => EFFECTIVE,

        (MonsterType::Dragon, MonsterType::Bug) => EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Dark) => EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Dragon) => SUPER_EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Electric) => EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Fairy) => INEFFECTIVE,
        (MonsterType::Dragon, MonsterType::Fighting) => EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Fire) => EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Flying) => EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Ghost) => EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Grass) => EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Ground) => EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Ice) => EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Poison) => EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Psychic) => EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Rock) => EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Steel) => NOT_VERY_EFFECTIVE,
        (MonsterType::Dragon, MonsterType::Water) => EFFECTIVE,

        (MonsterType::Electric, MonsterType::Bug) => EFFECTIVE,
        (MonsterType::Electric, MonsterType::Dark) => EFFECTIVE,
        (MonsterType::Electric, MonsterType::Dragon) => NOT_VERY_EFFECTIVE,
        (MonsterType::Electric, MonsterType::Electric) => NOT_VERY_EFFECTIVE,
        (MonsterType::Electric, MonsterType::Fairy) => EFFECTIVE,
        (MonsterType::Electric, MonsterType::Fighting) => EFFECTIVE,
        (MonsterType::Electric, MonsterType::Fire) => EFFECTIVE,
        (MonsterType::Electric, MonsterType::Flying) => SUPER_EFFECTIVE,
        (MonsterType::Electric, MonsterType::Ghost) => EFFECTIVE,
        (MonsterType::Electric, MonsterType::Grass) => NOT_VERY_EFFECTIVE,
        (MonsterType::Electric, MonsterType::Ground) => INEFFECTIVE,
        (MonsterType::Electric, MonsterType::Ice) => EFFECTIVE,
        (MonsterType::Electric, MonsterType::Poison) => EFFECTIVE,
        (MonsterType::Electric, MonsterType::Psychic) => EFFECTIVE,
        (MonsterType::Electric, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Electric, MonsterType::Rock) => EFFECTIVE,
        (MonsterType::Electric, MonsterType::Steel) => EFFECTIVE,
        (MonsterType::Electric, MonsterType::Water) => SUPER_EFFECTIVE,

        (MonsterType::Fairy, MonsterType::Bug) => EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Dark) => SUPER_EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Dragon) => SUPER_EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Electric) => EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Fairy) => EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Fighting) => SUPER_EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Fire) => NOT_VERY_EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Flying) => EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Ghost) => EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Grass) => EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Ground) => EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Ice) => EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Poison) => EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Psychic) => EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Rock) => EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Steel) => NOT_VERY_EFFECTIVE,
        (MonsterType::Fairy, MonsterType::Water) => EFFECTIVE,

        (MonsterType::Fighting, MonsterType::Bug) => NOT_VERY_EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Dark) => SUPER_EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Dragon) => EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Electric) => EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Fairy) => NOT_VERY_EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Fighting) => EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Fire) => EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Flying) => NOT_VERY_EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Ghost) => INEFFECTIVE,
        (MonsterType::Fighting, MonsterType::Grass) => EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Ground) => EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Ice) => SUPER_EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Poison) => NOT_VERY_EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Psychic) => NOT_VERY_EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Normal) => SUPER_EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Rock) => SUPER_EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Steel) => EFFECTIVE,
        (MonsterType::Fighting, MonsterType::Water) => EFFECTIVE,

        (MonsterType::Fire, MonsterType::Bug) => SUPER_EFFECTIVE,
        (MonsterType::Fire, MonsterType::Dark) => EFFECTIVE,
        (MonsterType::Fire, MonsterType::Dragon) => NOT_VERY_EFFECTIVE,
        (MonsterType::Fire, MonsterType::Electric) => EFFECTIVE,
        (MonsterType::Fire, MonsterType::Fairy) => EFFECTIVE,
        (MonsterType::Fire, MonsterType::Fighting) => EFFECTIVE,
        (MonsterType::Fire, MonsterType::Fire) => NOT_VERY_EFFECTIVE,
        (MonsterType::Fire, MonsterType::Flying) => EFFECTIVE,
        (MonsterType::Fire, MonsterType::Ghost) => EFFECTIVE,
        (MonsterType::Fire, MonsterType::Grass) => SUPER_EFFECTIVE,
        (MonsterType::Fire, MonsterType::Ground) => EFFECTIVE,
        (MonsterType::Fire, MonsterType::Ice) => SUPER_EFFECTIVE,
        (MonsterType::Fire, MonsterType::Poison) => EFFECTIVE,
        (MonsterType::Fire, MonsterType::Psychic) => EFFECTIVE,
        (MonsterType::Fire, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Fire, MonsterType::Rock) => NOT_VERY_EFFECTIVE,
        (MonsterType::Fire, MonsterType::Steel) => SUPER_EFFECTIVE,
        (MonsterType::Fire, MonsterType::Water) => NOT_VERY_EFFECTIVE,

        (MonsterType::Flying, MonsterType::Bug) => SUPER_EFFECTIVE,
        (MonsterType::Flying, MonsterType::Dark) => EFFECTIVE,
        (MonsterType::Flying, MonsterType::Dragon) => EFFECTIVE,
        (MonsterType::Flying, MonsterType::Electric) => NOT_VERY_EFFECTIVE,
        (MonsterType::Flying, MonsterType::Fairy) => EFFECTIVE,
        (MonsterType::Flying, MonsterType::Fighting) => SUPER_EFFECTIVE,
        (MonsterType::Flying, MonsterType::Fire) => EFFECTIVE,
        (MonsterType::Flying, MonsterType::Flying) => EFFECTIVE,
        (MonsterType::Flying, MonsterType::Ghost) => EFFECTIVE,
        (MonsterType::Flying, MonsterType::Grass) => SUPER_EFFECTIVE,
        (MonsterType::Flying, MonsterType::Ground) => EFFECTIVE,
        (MonsterType::Flying, MonsterType::Ice) => EFFECTIVE,
        (MonsterType::Flying, MonsterType::Poison) => EFFECTIVE,
        (MonsterType::Flying, MonsterType::Psychic) => EFFECTIVE,
        (MonsterType::Flying, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Flying, MonsterType::Rock) => NOT_VERY_EFFECTIVE,
        (MonsterType::Flying, MonsterType::Steel) => NOT_VERY_EFFECTIVE,
        (MonsterType::Flying, MonsterType::Water) => EFFECTIVE,

        (MonsterType::Ghost, MonsterType::Bug) => EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Dark) => NOT_VERY_EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Dragon) => EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Electric) => EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Fairy) => EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Fighting) => EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Fire) => EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Flying) => EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Ghost) => SUPER_EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Grass) => EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Ground) => EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Ice) => EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Poison) => EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Psychic) => SUPER_EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Normal) => INEFFECTIVE,
        (MonsterType::Ghost, MonsterType::Rock) => EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Steel) => EFFECTIVE,
        (MonsterType::Ghost, MonsterType::Water) => EFFECTIVE,

        (MonsterType::Grass, MonsterType::Bug) => NOT_VERY_EFFECTIVE,
        (MonsterType::Grass, MonsterType::Dark) => EFFECTIVE,
        (MonsterType::Grass, MonsterType::Dragon) => EFFECTIVE,
        (MonsterType::Grass, MonsterType::Electric) => EFFECTIVE,
        (MonsterType::Grass, MonsterType::Fairy) => EFFECTIVE,
        (MonsterType::Grass, MonsterType::Fighting) => EFFECTIVE,
        (MonsterType::Grass, MonsterType::Fire) => NOT_VERY_EFFECTIVE,
        (MonsterType::Grass, MonsterType::Flying) => NOT_VERY_EFFECTIVE,
        (MonsterType::Grass, MonsterType::Ghost) => EFFECTIVE,
        (MonsterType::Grass, MonsterType::Grass) => EFFECTIVE,
        (MonsterType::Grass, MonsterType::Ground) => SUPER_EFFECTIVE,
        (MonsterType::Grass, MonsterType::Ice) => EFFECTIVE,
        (MonsterType::Grass, MonsterType::Poison) => NOT_VERY_EFFECTIVE,
        (MonsterType::Grass, MonsterType::Psychic) => EFFECTIVE,
        (MonsterType::Grass, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Grass, MonsterType::Rock) => SUPER_EFFECTIVE,
        (MonsterType::Grass, MonsterType::Steel) => NOT_VERY_EFFECTIVE,
        (MonsterType::Grass, MonsterType::Water) => SUPER_EFFECTIVE,

        (MonsterType::Ground, MonsterType::Bug) => NOT_VERY_EFFECTIVE,
        (MonsterType::Ground, MonsterType::Dark) => EFFECTIVE,
        (MonsterType::Ground, MonsterType::Dragon) => EFFECTIVE,
        (MonsterType::Ground, MonsterType::Electric) => SUPER_EFFECTIVE,
        (MonsterType::Ground, MonsterType::Fairy) => EFFECTIVE,
        (MonsterType::Ground, MonsterType::Fighting) => EFFECTIVE,
        (MonsterType::Ground, MonsterType::Fire) => SUPER_EFFECTIVE,
        (MonsterType::Ground, MonsterType::Flying) => INEFFECTIVE,
        (MonsterType::Ground, MonsterType::Ghost) => EFFECTIVE,
        (MonsterType::Ground, MonsterType::Grass) => NOT_VERY_EFFECTIVE,
        (MonsterType::Ground, MonsterType::Ground) => SUPER_EFFECTIVE,
        (MonsterType::Ground, MonsterType::Ice) => EFFECTIVE,
        (MonsterType::Ground, MonsterType::Poison) => EFFECTIVE,
        (MonsterType::Ground, MonsterType::Psychic) => EFFECTIVE,
        (MonsterType::Ground, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Ground, MonsterType::Rock) => SUPER_EFFECTIVE,
        (MonsterType::Ground, MonsterType::Steel) => SUPER_EFFECTIVE,
        (MonsterType::Ground, MonsterType::Water) => EFFECTIVE,

        (MonsterType::Ice, MonsterType::Bug) => EFFECTIVE,
        (MonsterType::Ice, MonsterType::Dark) => EFFECTIVE,
        (MonsterType::Ice, MonsterType::Dragon) => SUPER_EFFECTIVE,
        (MonsterType::Ice, MonsterType::Electric) => EFFECTIVE,
        (MonsterType::Ice, MonsterType::Fairy) => EFFECTIVE,
        (MonsterType::Ice, MonsterType::Fighting) => EFFECTIVE,
        (MonsterType::Ice, MonsterType::Fire) => NOT_VERY_EFFECTIVE,
        (MonsterType::Ice, MonsterType::Flying) => SUPER_EFFECTIVE,
        (MonsterType::Ice, MonsterType::Ghost) => EFFECTIVE,
        (MonsterType::Ice, MonsterType::Grass) => SUPER_EFFECTIVE,
        (MonsterType::Ice, MonsterType::Ground) => SUPER_EFFECTIVE,
        (MonsterType::Ice, MonsterType::Ice) => NOT_VERY_EFFECTIVE,
        (MonsterType::Ice, MonsterType::Poison) => EFFECTIVE,
        (MonsterType::Ice, MonsterType::Psychic) => EFFECTIVE,
        (MonsterType::Ice, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Ice, MonsterType::Rock) => EFFECTIVE,
        (MonsterType::Ice, MonsterType::Steel) => NOT_VERY_EFFECTIVE,
        (MonsterType::Ice, MonsterType::Water) => NOT_VERY_EFFECTIVE,

        (MonsterType::Poison, MonsterType::Bug) => EFFECTIVE,
        (MonsterType::Poison, MonsterType::Dark) => EFFECTIVE,
        (MonsterType::Poison, MonsterType::Dragon) => EFFECTIVE,
        (MonsterType::Poison, MonsterType::Electric) => EFFECTIVE,
        (MonsterType::Poison, MonsterType::Fairy) => SUPER_EFFECTIVE,
        (MonsterType::Poison, MonsterType::Fighting) => EFFECTIVE,
        (MonsterType::Poison, MonsterType::Fire) => EFFECTIVE,
        (MonsterType::Poison, MonsterType::Flying) => EFFECTIVE,
        (MonsterType::Poison, MonsterType::Ghost) => NOT_VERY_EFFECTIVE,
        (MonsterType::Poison, MonsterType::Grass) => SUPER_EFFECTIVE,
        (MonsterType::Poison, MonsterType::Ground) => NOT_VERY_EFFECTIVE,
        (MonsterType::Poison, MonsterType::Ice) => EFFECTIVE,
        (MonsterType::Poison, MonsterType::Poison) => NOT_VERY_EFFECTIVE,
        (MonsterType::Poison, MonsterType::Psychic) => EFFECTIVE,
        (MonsterType::Poison, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Poison, MonsterType::Rock) => NOT_VERY_EFFECTIVE,
        (MonsterType::Poison, MonsterType::Steel) => INEFFECTIVE,
        (MonsterType::Poison, MonsterType::Water) => EFFECTIVE,

        (MonsterType::Psychic, MonsterType::Bug) => EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Dark) => INEFFECTIVE,
        (MonsterType::Psychic, MonsterType::Dragon) => EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Electric) => EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Fairy) => EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Fighting) => SUPER_EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Fire) => EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Flying) => EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Ghost) => EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Grass) => EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Ground) => EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Ice) => EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Poison) => SUPER_EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Psychic) => NOT_VERY_EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Rock) => EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Steel) => NOT_VERY_EFFECTIVE,
        (MonsterType::Psychic, MonsterType::Water) => EFFECTIVE,

        (MonsterType::Normal, MonsterType::Bug) => EFFECTIVE,
        (MonsterType::Normal, MonsterType::Dark) => EFFECTIVE,
        (MonsterType::Normal, MonsterType::Dragon) => EFFECTIVE,
        (MonsterType::Normal, MonsterType::Electric) => EFFECTIVE,
        (MonsterType::Normal, MonsterType::Fairy) => EFFECTIVE,
        (MonsterType::Normal, MonsterType::Fighting) => EFFECTIVE,
        (MonsterType::Normal, MonsterType::Fire) => EFFECTIVE,
        (MonsterType::Normal, MonsterType::Flying) => EFFECTIVE,
        (MonsterType::Normal, MonsterType::Ghost) => INEFFECTIVE,
        (MonsterType::Normal, MonsterType::Grass) => EFFECTIVE,
        (MonsterType::Normal, MonsterType::Ground) => EFFECTIVE,
        (MonsterType::Normal, MonsterType::Ice) => EFFECTIVE,
        (MonsterType::Normal, MonsterType::Poison) => EFFECTIVE,
        (MonsterType::Normal, MonsterType::Psychic) => EFFECTIVE,
        (MonsterType::Normal, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Normal, MonsterType::Rock) => NOT_VERY_EFFECTIVE,
        (MonsterType::Normal, MonsterType::Steel) => NOT_VERY_EFFECTIVE,
        (MonsterType::Normal, MonsterType::Water) => EFFECTIVE,

        (MonsterType::Rock, MonsterType::Bug) => SUPER_EFFECTIVE,
        (MonsterType::Rock, MonsterType::Dark) => EFFECTIVE,
        (MonsterType::Rock, MonsterType::Dragon) => EFFECTIVE,
        (MonsterType::Rock, MonsterType::Electric) => EFFECTIVE,
        (MonsterType::Rock, MonsterType::Fairy) => EFFECTIVE,
        (MonsterType::Rock, MonsterType::Fighting) => NOT_VERY_EFFECTIVE,
        (MonsterType::Rock, MonsterType::Fire) => SUPER_EFFECTIVE,
        (MonsterType::Rock, MonsterType::Flying) => SUPER_EFFECTIVE,
        (MonsterType::Rock, MonsterType::Ghost) => EFFECTIVE,
        (MonsterType::Rock, MonsterType::Grass) => EFFECTIVE,
        (MonsterType::Rock, MonsterType::Ground) => NOT_VERY_EFFECTIVE,
        (MonsterType::Rock, MonsterType::Ice) => SUPER_EFFECTIVE,
        (MonsterType::Rock, MonsterType::Poison) => EFFECTIVE,
        (MonsterType::Rock, MonsterType::Psychic) => EFFECTIVE,
        (MonsterType::Rock, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Rock, MonsterType::Rock) => EFFECTIVE,
        (MonsterType::Rock, MonsterType::Steel) => NOT_VERY_EFFECTIVE,
        (MonsterType::Rock, MonsterType::Water) => EFFECTIVE,

        (MonsterType::Steel, MonsterType::Bug) => EFFECTIVE,
        (MonsterType::Steel, MonsterType::Dark) => EFFECTIVE,
        (MonsterType::Steel, MonsterType::Dragon) => EFFECTIVE,
        (MonsterType::Steel, MonsterType::Electric) => NOT_VERY_EFFECTIVE,
        (MonsterType::Steel, MonsterType::Fairy) => SUPER_EFFECTIVE,
        (MonsterType::Steel, MonsterType::Fighting) => EFFECTIVE,
        (MonsterType::Steel, MonsterType::Fire) => NOT_VERY_EFFECTIVE,
        (MonsterType::Steel, MonsterType::Flying) => EFFECTIVE,
        (MonsterType::Steel, MonsterType::Ghost) => EFFECTIVE,
        (MonsterType::Steel, MonsterType::Grass) => EFFECTIVE,
        (MonsterType::Steel, MonsterType::Ground) => EFFECTIVE,
        (MonsterType::Steel, MonsterType::Ice) => SUPER_EFFECTIVE,
        (MonsterType::Steel, MonsterType::Poison) => EFFECTIVE,
        (MonsterType::Steel, MonsterType::Psychic) => EFFECTIVE,
        (MonsterType::Steel, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Steel, MonsterType::Rock) => SUPER_EFFECTIVE,
        (MonsterType::Steel, MonsterType::Steel) => NOT_VERY_EFFECTIVE,
        (MonsterType::Steel, MonsterType::Water) => NOT_VERY_EFFECTIVE,

        (MonsterType::Water, MonsterType::Bug) => EFFECTIVE,
        (MonsterType::Water, MonsterType::Dark) => EFFECTIVE,
        (MonsterType::Water, MonsterType::Dragon) => NOT_VERY_EFFECTIVE,
        (MonsterType::Water, MonsterType::Electric) => EFFECTIVE,
        (MonsterType::Water, MonsterType::Fairy) => EFFECTIVE,
        (MonsterType::Water, MonsterType::Fighting) => EFFECTIVE,
        (MonsterType::Water, MonsterType::Fire) => SUPER_EFFECTIVE,
        (MonsterType::Water, MonsterType::Flying) => EFFECTIVE,
        (MonsterType::Water, MonsterType::Ghost) => EFFECTIVE,
        (MonsterType::Water, MonsterType::Grass) => NOT_VERY_EFFECTIVE,
        (MonsterType::Water, MonsterType::Ground) => SUPER_EFFECTIVE,
        (MonsterType::Water, MonsterType::Ice) => EFFECTIVE,
        (MonsterType::Water, MonsterType::Poison) => EFFECTIVE,
        (MonsterType::Water, MonsterType::Psychic) => EFFECTIVE,
        (MonsterType::Water, MonsterType::Normal) => EFFECTIVE,
        (MonsterType::Water, MonsterType::Rock) => SUPER_EFFECTIVE,
        (MonsterType::Water, MonsterType::Steel) => EFFECTIVE,
        (MonsterType::Water, MonsterType::Water) => NOT_VERY_EFFECTIVE,
    }
}
