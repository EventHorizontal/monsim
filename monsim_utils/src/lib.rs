mod max_sized_vec;
use std::fmt::Display;

pub use max_sized_vec::MaxSizedVec;
mod outcome;
pub use outcome::Outcome;
mod percentage;
pub use percentage::{ClampedPercent, Percent};
mod team_affl;
pub use team_affl::*;

/// Type alias for readability.
pub type Nothing = ();
/// Type alias for readability.
pub const NOTHING: () = ();

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

/// Makes `!` more readable
#[macro_export]
macro_rules! not {
    ($x: expr) => {
        !$x
    };
}

#[derive(Debug, Clone, Copy)]
pub enum Count {
    Fixed(u8),
    RandomInRange { min: u8, max: u8 },
}

pub trait RoundTiesDownExt {
    type Result;
    fn round_ties_down(self) -> Self::Result;
}

impl RoundTiesDownExt for f64 {
    type Result = u16;

    fn round_ties_down(self) -> Self::Result {
        let integer_part = self.floor();
        match (self - integer_part).total_cmp(&0.5) {
            std::cmp::Ordering::Less | std::cmp::Ordering::Equal => self.floor() as u16,
            std::cmp::Ordering::Greater => self.ceil() as u16,
        }
    }
}

#[test]
fn test_round_ties_down() {
    assert_eq!(34, 34.1.round_ties_down());
    assert_eq!(34, 34.5.round_ties_down());
    assert_eq!(256, 255.7.round_ties_down());
    assert_eq!(134, 133.6.round_ties_down());
    assert_eq!(133, 133.4999.round_ties_down());
    assert_eq!(423238237847384783, 423238237847384783.4999.round_ties_down());
}

pub fn vec_to_string<T: Display>(iterator: &mut impl Iterator<Item = T>) -> String {
    if let Some(item) = iterator.next() {
        let mut output = format!["[{}", item];
        output += &iterator.fold("".to_owned(), |mut acc, item| {
            acc += &format![", {}", item];
            acc
        });
        output += "]";
        output
    } else {
        "[]".to_string()
    }
}
