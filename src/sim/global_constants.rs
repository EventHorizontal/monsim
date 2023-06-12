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

/// A percentage that is unbound above
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Percent(pub u16);

impl Add for Percent {
    type Output = Percent;

    fn add(self, rhs: Self) -> Self::Output {
        Percent(self.0 + rhs.0)
    }
}

impl Sub for Percent {
    type Output = Percent;

    fn sub(self, rhs: Self) -> Self::Output {
        Percent(self.0.saturating_sub(rhs.0))
    }
}

impl Mul for Percent {
    type Output = Percent;

    fn mul(self, rhs: Self) -> Self::Output {
        Self((self.0 as f64 * (rhs.0 as f64 / 100.0f64)) as u16)
    }
}

impl Mul<f64> for Percent {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        rhs * (self.0 as f64 / 100.0f64)
    }
}

impl Mul<Percent> for f64 {
    type Output = f64;

    fn mul(self, rhs: Percent) -> Self::Output {
        self * (rhs.0 as f64 / 100.0f64)
    }
}

/// A percentage that must be between 0 and 100
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClampedPercent(u16);

impl ClampedPercent {
    pub fn from(value: u16) -> Self {
        assert!(value <= 100, "ClampedPercent only takes values between 0 and 100. If you want unbound percentages, use Percent instead.");
        Self(value)
    }
}

impl Add for ClampedPercent {
    type Output = ClampedPercent;

    fn add(self, rhs: Self) -> Self::Output {
        ClampedPercent(self.0 + rhs.0).min(ClampedPercent(100))
    }
}

impl Sub for ClampedPercent {
    type Output = ClampedPercent;

    fn sub(self, rhs: Self) -> Self::Output {
        ClampedPercent(self.0.saturating_sub(rhs.0))
    }
}

impl Mul<f64> for ClampedPercent {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        rhs * (self.0 as f64 / 100.0f64)
    }
}

impl Mul<ClampedPercent> for f64 {
    type Output = f64;

    fn mul(self, rhs: ClampedPercent) -> Self::Output {
        self * (rhs.0 as f64 / 100.0f64)
    }
}

#[test]
fn test_percent_type() {
    let fifty_percent = ClampedPercent(50);
    assert_eq!(5.0 * fifty_percent, 2.5f64);
    assert_eq!(fifty_percent * 5.0, 2.5f64);
    assert_eq!(fifty_percent + ClampedPercent(51), ClampedPercent(100));
    assert_eq!(fifty_percent - ClampedPercent(51), ClampedPercent(0));
}

use std::ops::{Not, Add, Sub, Mul};

use super::ElementalType;

pub const INEFFECTIVE: Percent = Percent(0);
pub const NOT_VERY_EFFECTIVE: Percent = Percent(50);
pub const EFFECTIVE: Percent = Percent(100);
pub const SUPER_EFFECTIVE: Percent = Percent(200);

pub const EMPTY_LINE: &str = "";

pub const fn type_matchup(move_type: ElementalType, target_type: ElementalType) -> Percent {
    match (move_type, target_type) {
        (ElementalType::Bug, ElementalType::Bug) => EFFECTIVE,
        (ElementalType::Bug, ElementalType::Dark) => SUPER_EFFECTIVE,
        (ElementalType::Bug, ElementalType::Dragon) => EFFECTIVE,
        (ElementalType::Bug, ElementalType::Electric) => EFFECTIVE,
        (ElementalType::Bug, ElementalType::Fairy) => NOT_VERY_EFFECTIVE,
        (ElementalType::Bug, ElementalType::Fighting) => NOT_VERY_EFFECTIVE,
        (ElementalType::Bug, ElementalType::Fire) => NOT_VERY_EFFECTIVE,
        (ElementalType::Bug, ElementalType::Flying) => NOT_VERY_EFFECTIVE,
        (ElementalType::Bug, ElementalType::Ghost) => NOT_VERY_EFFECTIVE,
        (ElementalType::Bug, ElementalType::Grass) => SUPER_EFFECTIVE,
        (ElementalType::Bug, ElementalType::Ground) => EFFECTIVE,
        (ElementalType::Bug, ElementalType::Ice) => EFFECTIVE,
        (ElementalType::Bug, ElementalType::Poison) => NOT_VERY_EFFECTIVE,
        (ElementalType::Bug, ElementalType::Psychic) => SUPER_EFFECTIVE,
        (ElementalType::Bug, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Bug, ElementalType::Rock) => EFFECTIVE,
        (ElementalType::Bug, ElementalType::Steel) => NOT_VERY_EFFECTIVE,
        (ElementalType::Bug, ElementalType::Water) => EFFECTIVE,

        (ElementalType::Dark, ElementalType::Bug) => EFFECTIVE,
        (ElementalType::Dark, ElementalType::Dark) => NOT_VERY_EFFECTIVE,
        (ElementalType::Dark, ElementalType::Dragon) => EFFECTIVE,
        (ElementalType::Dark, ElementalType::Electric) => EFFECTIVE,
        (ElementalType::Dark, ElementalType::Fairy) => NOT_VERY_EFFECTIVE,
        (ElementalType::Dark, ElementalType::Fighting) => NOT_VERY_EFFECTIVE,
        (ElementalType::Dark, ElementalType::Fire) => EFFECTIVE,
        (ElementalType::Dark, ElementalType::Flying) => EFFECTIVE,
        (ElementalType::Dark, ElementalType::Ghost) => SUPER_EFFECTIVE,
        (ElementalType::Dark, ElementalType::Grass) => EFFECTIVE,
        (ElementalType::Dark, ElementalType::Ground) => EFFECTIVE,
        (ElementalType::Dark, ElementalType::Ice) => EFFECTIVE,
        (ElementalType::Dark, ElementalType::Poison) => EFFECTIVE,
        (ElementalType::Dark, ElementalType::Psychic) => SUPER_EFFECTIVE,
        (ElementalType::Dark, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Dark, ElementalType::Rock) => EFFECTIVE,
        (ElementalType::Dark, ElementalType::Steel) => EFFECTIVE,
        (ElementalType::Dark, ElementalType::Water) => EFFECTIVE,

        (ElementalType::Dragon, ElementalType::Bug) => EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Dark) => EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Dragon) => SUPER_EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Electric) => EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Fairy) => INEFFECTIVE,
        (ElementalType::Dragon, ElementalType::Fighting) => EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Fire) => EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Flying) => EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Ghost) => EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Grass) => EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Ground) => EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Ice) => EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Poison) => EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Psychic) => EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Rock) => EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Steel) => NOT_VERY_EFFECTIVE,
        (ElementalType::Dragon, ElementalType::Water) => EFFECTIVE,

        (ElementalType::Electric, ElementalType::Bug) => EFFECTIVE,
        (ElementalType::Electric, ElementalType::Dark) => EFFECTIVE,
        (ElementalType::Electric, ElementalType::Dragon) => NOT_VERY_EFFECTIVE,
        (ElementalType::Electric, ElementalType::Electric) => NOT_VERY_EFFECTIVE,
        (ElementalType::Electric, ElementalType::Fairy) => EFFECTIVE,
        (ElementalType::Electric, ElementalType::Fighting) => EFFECTIVE,
        (ElementalType::Electric, ElementalType::Fire) => EFFECTIVE,
        (ElementalType::Electric, ElementalType::Flying) => SUPER_EFFECTIVE,
        (ElementalType::Electric, ElementalType::Ghost) => EFFECTIVE,
        (ElementalType::Electric, ElementalType::Grass) => NOT_VERY_EFFECTIVE,
        (ElementalType::Electric, ElementalType::Ground) => INEFFECTIVE,
        (ElementalType::Electric, ElementalType::Ice) => EFFECTIVE,
        (ElementalType::Electric, ElementalType::Poison) => EFFECTIVE,
        (ElementalType::Electric, ElementalType::Psychic) => EFFECTIVE,
        (ElementalType::Electric, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Electric, ElementalType::Rock) => EFFECTIVE,
        (ElementalType::Electric, ElementalType::Steel) => EFFECTIVE,
        (ElementalType::Electric, ElementalType::Water) => SUPER_EFFECTIVE,

        (ElementalType::Fairy, ElementalType::Bug) => EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Dark) => SUPER_EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Dragon) => SUPER_EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Electric) => EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Fairy) => EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Fighting) => SUPER_EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Fire) => NOT_VERY_EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Flying) => EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Ghost) => EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Grass) => EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Ground) => EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Ice) => EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Poison) => EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Psychic) => EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Rock) => EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Steel) => NOT_VERY_EFFECTIVE,
        (ElementalType::Fairy, ElementalType::Water) => EFFECTIVE,

        (ElementalType::Fighting, ElementalType::Bug) => NOT_VERY_EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Dark) => SUPER_EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Dragon) => EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Electric) => EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Fairy) => NOT_VERY_EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Fighting) => EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Fire) => EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Flying) => NOT_VERY_EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Ghost) => INEFFECTIVE,
        (ElementalType::Fighting, ElementalType::Grass) => EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Ground) => EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Ice) => SUPER_EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Poison) => NOT_VERY_EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Psychic) => NOT_VERY_EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Normal) => SUPER_EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Rock) => SUPER_EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Steel) => EFFECTIVE,
        (ElementalType::Fighting, ElementalType::Water) => EFFECTIVE,

        (ElementalType::Fire, ElementalType::Bug) => SUPER_EFFECTIVE,
        (ElementalType::Fire, ElementalType::Dark) => EFFECTIVE,
        (ElementalType::Fire, ElementalType::Dragon) => NOT_VERY_EFFECTIVE,
        (ElementalType::Fire, ElementalType::Electric) => EFFECTIVE,
        (ElementalType::Fire, ElementalType::Fairy) => EFFECTIVE,
        (ElementalType::Fire, ElementalType::Fighting) => EFFECTIVE,
        (ElementalType::Fire, ElementalType::Fire) => NOT_VERY_EFFECTIVE,
        (ElementalType::Fire, ElementalType::Flying) => EFFECTIVE,
        (ElementalType::Fire, ElementalType::Ghost) => EFFECTIVE,
        (ElementalType::Fire, ElementalType::Grass) => SUPER_EFFECTIVE,
        (ElementalType::Fire, ElementalType::Ground) => EFFECTIVE,
        (ElementalType::Fire, ElementalType::Ice) => SUPER_EFFECTIVE,
        (ElementalType::Fire, ElementalType::Poison) => EFFECTIVE,
        (ElementalType::Fire, ElementalType::Psychic) => EFFECTIVE,
        (ElementalType::Fire, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Fire, ElementalType::Rock) => NOT_VERY_EFFECTIVE,
        (ElementalType::Fire, ElementalType::Steel) => SUPER_EFFECTIVE,
        (ElementalType::Fire, ElementalType::Water) => NOT_VERY_EFFECTIVE,

        (ElementalType::Flying, ElementalType::Bug) => SUPER_EFFECTIVE,
        (ElementalType::Flying, ElementalType::Dark) => EFFECTIVE,
        (ElementalType::Flying, ElementalType::Dragon) => EFFECTIVE,
        (ElementalType::Flying, ElementalType::Electric) => NOT_VERY_EFFECTIVE,
        (ElementalType::Flying, ElementalType::Fairy) => EFFECTIVE,
        (ElementalType::Flying, ElementalType::Fighting) => SUPER_EFFECTIVE,
        (ElementalType::Flying, ElementalType::Fire) => EFFECTIVE,
        (ElementalType::Flying, ElementalType::Flying) => EFFECTIVE,
        (ElementalType::Flying, ElementalType::Ghost) => EFFECTIVE,
        (ElementalType::Flying, ElementalType::Grass) => SUPER_EFFECTIVE,
        (ElementalType::Flying, ElementalType::Ground) => EFFECTIVE,
        (ElementalType::Flying, ElementalType::Ice) => EFFECTIVE,
        (ElementalType::Flying, ElementalType::Poison) => EFFECTIVE,
        (ElementalType::Flying, ElementalType::Psychic) => EFFECTIVE,
        (ElementalType::Flying, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Flying, ElementalType::Rock) => NOT_VERY_EFFECTIVE,
        (ElementalType::Flying, ElementalType::Steel) => NOT_VERY_EFFECTIVE,
        (ElementalType::Flying, ElementalType::Water) => EFFECTIVE,

        (ElementalType::Ghost, ElementalType::Bug) => EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Dark) => NOT_VERY_EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Dragon) => EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Electric) => EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Fairy) => EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Fighting) => EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Fire) => EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Flying) => EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Ghost) => SUPER_EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Grass) => EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Ground) => EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Ice) => EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Poison) => EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Psychic) => SUPER_EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Normal) => INEFFECTIVE,
        (ElementalType::Ghost, ElementalType::Rock) => EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Steel) => EFFECTIVE,
        (ElementalType::Ghost, ElementalType::Water) => EFFECTIVE,

        (ElementalType::Grass, ElementalType::Bug) => NOT_VERY_EFFECTIVE,
        (ElementalType::Grass, ElementalType::Dark) => EFFECTIVE,
        (ElementalType::Grass, ElementalType::Dragon) => EFFECTIVE,
        (ElementalType::Grass, ElementalType::Electric) => EFFECTIVE,
        (ElementalType::Grass, ElementalType::Fairy) => EFFECTIVE,
        (ElementalType::Grass, ElementalType::Fighting) => EFFECTIVE,
        (ElementalType::Grass, ElementalType::Fire) => NOT_VERY_EFFECTIVE,
        (ElementalType::Grass, ElementalType::Flying) => NOT_VERY_EFFECTIVE,
        (ElementalType::Grass, ElementalType::Ghost) => EFFECTIVE,
        (ElementalType::Grass, ElementalType::Grass) => EFFECTIVE,
        (ElementalType::Grass, ElementalType::Ground) => SUPER_EFFECTIVE,
        (ElementalType::Grass, ElementalType::Ice) => EFFECTIVE,
        (ElementalType::Grass, ElementalType::Poison) => NOT_VERY_EFFECTIVE,
        (ElementalType::Grass, ElementalType::Psychic) => EFFECTIVE,
        (ElementalType::Grass, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Grass, ElementalType::Rock) => SUPER_EFFECTIVE,
        (ElementalType::Grass, ElementalType::Steel) => NOT_VERY_EFFECTIVE,
        (ElementalType::Grass, ElementalType::Water) => SUPER_EFFECTIVE,

        (ElementalType::Ground, ElementalType::Bug) => NOT_VERY_EFFECTIVE,
        (ElementalType::Ground, ElementalType::Dark) => EFFECTIVE,
        (ElementalType::Ground, ElementalType::Dragon) => EFFECTIVE,
        (ElementalType::Ground, ElementalType::Electric) => SUPER_EFFECTIVE,
        (ElementalType::Ground, ElementalType::Fairy) => EFFECTIVE,
        (ElementalType::Ground, ElementalType::Fighting) => EFFECTIVE,
        (ElementalType::Ground, ElementalType::Fire) => SUPER_EFFECTIVE,
        (ElementalType::Ground, ElementalType::Flying) => INEFFECTIVE,
        (ElementalType::Ground, ElementalType::Ghost) => EFFECTIVE,
        (ElementalType::Ground, ElementalType::Grass) => NOT_VERY_EFFECTIVE,
        (ElementalType::Ground, ElementalType::Ground) => SUPER_EFFECTIVE,
        (ElementalType::Ground, ElementalType::Ice) => EFFECTIVE,
        (ElementalType::Ground, ElementalType::Poison) => EFFECTIVE,
        (ElementalType::Ground, ElementalType::Psychic) => EFFECTIVE,
        (ElementalType::Ground, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Ground, ElementalType::Rock) => SUPER_EFFECTIVE,
        (ElementalType::Ground, ElementalType::Steel) => SUPER_EFFECTIVE,
        (ElementalType::Ground, ElementalType::Water) => EFFECTIVE,

        (ElementalType::Ice, ElementalType::Bug) => EFFECTIVE,
        (ElementalType::Ice, ElementalType::Dark) => EFFECTIVE,
        (ElementalType::Ice, ElementalType::Dragon) => SUPER_EFFECTIVE,
        (ElementalType::Ice, ElementalType::Electric) => EFFECTIVE,
        (ElementalType::Ice, ElementalType::Fairy) => EFFECTIVE,
        (ElementalType::Ice, ElementalType::Fighting) => EFFECTIVE,
        (ElementalType::Ice, ElementalType::Fire) => NOT_VERY_EFFECTIVE,
        (ElementalType::Ice, ElementalType::Flying) => SUPER_EFFECTIVE,
        (ElementalType::Ice, ElementalType::Ghost) => EFFECTIVE,
        (ElementalType::Ice, ElementalType::Grass) => SUPER_EFFECTIVE,
        (ElementalType::Ice, ElementalType::Ground) => SUPER_EFFECTIVE,
        (ElementalType::Ice, ElementalType::Ice) => NOT_VERY_EFFECTIVE,
        (ElementalType::Ice, ElementalType::Poison) => EFFECTIVE,
        (ElementalType::Ice, ElementalType::Psychic) => EFFECTIVE,
        (ElementalType::Ice, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Ice, ElementalType::Rock) => EFFECTIVE,
        (ElementalType::Ice, ElementalType::Steel) => NOT_VERY_EFFECTIVE,
        (ElementalType::Ice, ElementalType::Water) => NOT_VERY_EFFECTIVE,

        (ElementalType::Poison, ElementalType::Bug) => EFFECTIVE,
        (ElementalType::Poison, ElementalType::Dark) => EFFECTIVE,
        (ElementalType::Poison, ElementalType::Dragon) => EFFECTIVE,
        (ElementalType::Poison, ElementalType::Electric) => EFFECTIVE,
        (ElementalType::Poison, ElementalType::Fairy) => SUPER_EFFECTIVE,
        (ElementalType::Poison, ElementalType::Fighting) => EFFECTIVE,
        (ElementalType::Poison, ElementalType::Fire) => EFFECTIVE,
        (ElementalType::Poison, ElementalType::Flying) => EFFECTIVE,
        (ElementalType::Poison, ElementalType::Ghost) => NOT_VERY_EFFECTIVE,
        (ElementalType::Poison, ElementalType::Grass) => SUPER_EFFECTIVE,
        (ElementalType::Poison, ElementalType::Ground) => NOT_VERY_EFFECTIVE,
        (ElementalType::Poison, ElementalType::Ice) => EFFECTIVE,
        (ElementalType::Poison, ElementalType::Poison) => NOT_VERY_EFFECTIVE,
        (ElementalType::Poison, ElementalType::Psychic) => EFFECTIVE,
        (ElementalType::Poison, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Poison, ElementalType::Rock) => NOT_VERY_EFFECTIVE,
        (ElementalType::Poison, ElementalType::Steel) => INEFFECTIVE,
        (ElementalType::Poison, ElementalType::Water) => EFFECTIVE,

        (ElementalType::Psychic, ElementalType::Bug) => EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Dark) => INEFFECTIVE,
        (ElementalType::Psychic, ElementalType::Dragon) => EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Electric) => EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Fairy) => EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Fighting) => SUPER_EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Fire) => EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Flying) => EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Ghost) => EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Grass) => EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Ground) => EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Ice) => EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Poison) => SUPER_EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Psychic) => NOT_VERY_EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Rock) => EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Steel) => NOT_VERY_EFFECTIVE,
        (ElementalType::Psychic, ElementalType::Water) => EFFECTIVE,

        (ElementalType::Normal, ElementalType::Bug) => EFFECTIVE,
        (ElementalType::Normal, ElementalType::Dark) => EFFECTIVE,
        (ElementalType::Normal, ElementalType::Dragon) => EFFECTIVE,
        (ElementalType::Normal, ElementalType::Electric) => EFFECTIVE,
        (ElementalType::Normal, ElementalType::Fairy) => EFFECTIVE,
        (ElementalType::Normal, ElementalType::Fighting) => EFFECTIVE,
        (ElementalType::Normal, ElementalType::Fire) => EFFECTIVE,
        (ElementalType::Normal, ElementalType::Flying) => EFFECTIVE,
        (ElementalType::Normal, ElementalType::Ghost) => INEFFECTIVE,
        (ElementalType::Normal, ElementalType::Grass) => EFFECTIVE,
        (ElementalType::Normal, ElementalType::Ground) => EFFECTIVE,
        (ElementalType::Normal, ElementalType::Ice) => EFFECTIVE,
        (ElementalType::Normal, ElementalType::Poison) => EFFECTIVE,
        (ElementalType::Normal, ElementalType::Psychic) => EFFECTIVE,
        (ElementalType::Normal, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Normal, ElementalType::Rock) => NOT_VERY_EFFECTIVE,
        (ElementalType::Normal, ElementalType::Steel) => NOT_VERY_EFFECTIVE,
        (ElementalType::Normal, ElementalType::Water) => EFFECTIVE,

        (ElementalType::Rock, ElementalType::Bug) => SUPER_EFFECTIVE,
        (ElementalType::Rock, ElementalType::Dark) => EFFECTIVE,
        (ElementalType::Rock, ElementalType::Dragon) => EFFECTIVE,
        (ElementalType::Rock, ElementalType::Electric) => EFFECTIVE,
        (ElementalType::Rock, ElementalType::Fairy) => EFFECTIVE,
        (ElementalType::Rock, ElementalType::Fighting) => NOT_VERY_EFFECTIVE,
        (ElementalType::Rock, ElementalType::Fire) => SUPER_EFFECTIVE,
        (ElementalType::Rock, ElementalType::Flying) => SUPER_EFFECTIVE,
        (ElementalType::Rock, ElementalType::Ghost) => EFFECTIVE,
        (ElementalType::Rock, ElementalType::Grass) => EFFECTIVE,
        (ElementalType::Rock, ElementalType::Ground) => NOT_VERY_EFFECTIVE,
        (ElementalType::Rock, ElementalType::Ice) => SUPER_EFFECTIVE,
        (ElementalType::Rock, ElementalType::Poison) => EFFECTIVE,
        (ElementalType::Rock, ElementalType::Psychic) => EFFECTIVE,
        (ElementalType::Rock, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Rock, ElementalType::Rock) => EFFECTIVE,
        (ElementalType::Rock, ElementalType::Steel) => NOT_VERY_EFFECTIVE,
        (ElementalType::Rock, ElementalType::Water) => EFFECTIVE,

        (ElementalType::Steel, ElementalType::Bug) => EFFECTIVE,
        (ElementalType::Steel, ElementalType::Dark) => EFFECTIVE,
        (ElementalType::Steel, ElementalType::Dragon) => EFFECTIVE,
        (ElementalType::Steel, ElementalType::Electric) => NOT_VERY_EFFECTIVE,
        (ElementalType::Steel, ElementalType::Fairy) => SUPER_EFFECTIVE,
        (ElementalType::Steel, ElementalType::Fighting) => EFFECTIVE,
        (ElementalType::Steel, ElementalType::Fire) => NOT_VERY_EFFECTIVE,
        (ElementalType::Steel, ElementalType::Flying) => EFFECTIVE,
        (ElementalType::Steel, ElementalType::Ghost) => EFFECTIVE,
        (ElementalType::Steel, ElementalType::Grass) => EFFECTIVE,
        (ElementalType::Steel, ElementalType::Ground) => EFFECTIVE,
        (ElementalType::Steel, ElementalType::Ice) => SUPER_EFFECTIVE,
        (ElementalType::Steel, ElementalType::Poison) => EFFECTIVE,
        (ElementalType::Steel, ElementalType::Psychic) => EFFECTIVE,
        (ElementalType::Steel, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Steel, ElementalType::Rock) => SUPER_EFFECTIVE,
        (ElementalType::Steel, ElementalType::Steel) => NOT_VERY_EFFECTIVE,
        (ElementalType::Steel, ElementalType::Water) => NOT_VERY_EFFECTIVE,

        (ElementalType::Water, ElementalType::Bug) => EFFECTIVE,
        (ElementalType::Water, ElementalType::Dark) => EFFECTIVE,
        (ElementalType::Water, ElementalType::Dragon) => NOT_VERY_EFFECTIVE,
        (ElementalType::Water, ElementalType::Electric) => EFFECTIVE,
        (ElementalType::Water, ElementalType::Fairy) => EFFECTIVE,
        (ElementalType::Water, ElementalType::Fighting) => EFFECTIVE,
        (ElementalType::Water, ElementalType::Fire) => SUPER_EFFECTIVE,
        (ElementalType::Water, ElementalType::Flying) => EFFECTIVE,
        (ElementalType::Water, ElementalType::Ghost) => EFFECTIVE,
        (ElementalType::Water, ElementalType::Grass) => NOT_VERY_EFFECTIVE,
        (ElementalType::Water, ElementalType::Ground) => SUPER_EFFECTIVE,
        (ElementalType::Water, ElementalType::Ice) => EFFECTIVE,
        (ElementalType::Water, ElementalType::Poison) => EFFECTIVE,
        (ElementalType::Water, ElementalType::Psychic) => EFFECTIVE,
        (ElementalType::Water, ElementalType::Normal) => EFFECTIVE,
        (ElementalType::Water, ElementalType::Rock) => SUPER_EFFECTIVE,
        (ElementalType::Water, ElementalType::Steel) => EFFECTIVE,
        (ElementalType::Water, ElementalType::Water) => NOT_VERY_EFFECTIVE,
    }
}
