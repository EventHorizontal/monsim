use std::ops::{Add, Mul, Not, Sub};

/// I prefer this for readability
pub type Nothing = ();
/// I prefer this for readability
pub const NOTHING: () = ();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    Success,
    Failure,
}

impl From<bool> for Outcome {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Success,
            false => Self::Failure,
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
        assert!(
            value <= 100,
            "ClampedPercent only takes values between 0 and 100. If you want unbound percentages, use Percent instead."
        );
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

#[macro_export]
macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        core::convert::From::from([$(($k, $v),)*])
    }};
    // set-like
    ($($v:expr),* $(,)?) => {{
        core::convert::From::from([$($v,)*])
    }};
}
