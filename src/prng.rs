use std::{time, u64::MAX};

use std::ops::RangeInclusive;

// LCRNG -> Linear Congruential Random Number Generator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lcrng {
    start_seed: u64,
    current_seed: u64,
}

pub fn seed_from_time_now() -> u64 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

const A: u64 = 0x5D588B656C078965;
const C: u64 = 0x00269EC3;

impl Lcrng {
    pub(crate) fn new(start_seed: u64) -> Self {
        Self {
            start_seed,
            current_seed: start_seed,
        }
    }

    fn next(&mut self) -> u64 {
        // x_{n+1} = (A * x_n) + C <- Core function for the LCRNG
        self.current_seed = self.current_seed.wrapping_mul(A).wrapping_add(C);
        self.current_seed
    }

    /// Returns each u16 in the range with equal probability. If the range contains one number, it returns it with 100% certainty.
    pub(crate) fn generate_number_in_range(&mut self, range: RangeInclusive<u16>) -> u16 {
        let mut range_iter = range.into_iter();
        let start = range_iter
            .next()
            .expect("The range given to generate_number_in_range must have a first element.");
        let end = range_iter
            .next_back()
            .expect("The range given to generate_number_in_range must have a last element.");
        assert!(end >= 1u16, "The end of the range should be 1 or higher");
        if end == start {
            return start;
        }
        let random_number = self.next();
        let range = (end - start + 1) as f64;

        let result = ((random_number as f64 / MAX as f64) * range) as u16 + start;
        result
    }

    pub(crate) fn chance(&mut self, num: u16, denom: u16) -> bool {
        assert!(denom != 0);
        self.generate_number_in_range(1..=denom) <= num
    }
}
