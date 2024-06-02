mod max_sized_vec;
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
    RandomInRange{ min: u8, max: u8 }
}

