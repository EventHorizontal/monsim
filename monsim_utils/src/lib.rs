use std::ops::{Add, Deref, DerefMut, Mul, Not, Sub};

/// Type alias for readability of parentheses
pub type Nothing = ();
/// Type alias for readability of parentheses
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
impl Percent {
    pub fn is_matchup_ineffective(&self) -> bool {
        *self == Percent(0)
    }

    pub fn is_matchup_effective(&self) -> bool {
        *self == Percent(100)
    }

    pub fn is_matchup_not_very_effective(&self) -> bool {
        *self == Percent(25) || *self == Percent(50)
    }

    pub fn is_matchup_super_effective(&self) -> bool {
        *self == Percent(200) || *self == Percent(400)
    }
}

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

pub type ArrayOfOptionals<T, const N: usize> = [Option<T>; N];

pub fn slice_to_array_of_options<T: Copy, const N: usize>(vec: &[T]) -> ArrayOfOptionals<T, N> {
    assert!(vec.len() <= N, "Vector must have a length less than or equal to the required array size.");
    let mut arr = [None; N];
    let mut idx = 0;
    for element in vec {
        arr[idx] = Some(*element);
        idx += 1;
    }
    arr
}

/// Makes `!` more readable
#[macro_export]
macro_rules! not {
    ($x: expr) => {
        !$x
    };
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// A type that can be deferenced to get data marked as belonging to the Ally Team
pub struct Ally<T> {
    item: T
}

impl<T> Deref for Ally<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        & self.item
    }
}

impl<T> DerefMut for Ally<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<T> Ally<T> {
    pub fn new(item: T) -> Self {
        Self { item }
    }
    
    pub fn map<U, F>(self, f: F) -> Ally<U> where F: FnOnce(T) -> U {
        let item = f(self.item);
        Ally { item }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// A type that can be deferenced to get data marked as belonging to the Opponent Team
pub struct Opponent<T> {
    item: T
}

impl<T> Deref for Opponent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        & self.item
    }
}

impl<T> DerefMut for Opponent<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<T> Opponent<T> {
    pub fn new(item: T) -> Self {
        Self { item }
    }

    pub fn map<U, F>(self, f: F) -> Opponent<U> where F: FnOnce(T) -> U {
        let item = f(self.item);
        Opponent { item }
    }
}

pub trait TeamAffiliation<T> {
    type R<V>;
    fn map<U, F>(self, f: F) -> Self::R<U> where F: FnOnce(T) -> U;
}

impl<T> TeamAffiliation<T> for Ally<T> {
    type R<V> = Ally<V>;

    fn map<U, F>(self, f: F) -> Self::R<U> where F: FnOnce(T) -> U {
        self.map(f)
    }
}

impl<T> TeamAffiliation<T> for Opponent<T> {
    type R<V> = Opponent<V>;

    fn map<U, F>(self, f: F) -> Self::R<U> where F: FnOnce(T) -> U {
        self.map(f)
    }
}
