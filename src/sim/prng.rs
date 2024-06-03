use std::{time, u64::MAX};

use std::ops::RangeInclusive;

// LCRNG -> Linear Congruential Random Number Generator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prng {
    start_seed: u64,
    current_seed: u64,
}

const A: u64 = 0x5D588B656C078965;
const C: u64 = 0x00269EC3;

impl Prng {
    #[cfg(test)]
    #[allow(unused)]
    pub(crate) fn new(start_seed: u64) -> Self {
        Self {
            start_seed,
            current_seed: start_seed,
        }
    }

    pub(crate) fn from_current_time() -> Self {
        let start_seed = Self::seed_from_current_time();
        Self {
            start_seed,
            current_seed: start_seed,
        }
    }

    /// Returns each number in the range with equal probability. If the range contains one number, it returns it with 100% certainty.
    pub(crate) fn generate_random_number_in_range(&mut self, mut range: RangeInclusive<u16>) -> u16 {
        let start = range.next().expect("The range given to generate_number_in_range must have a first element.");
        let end = range
            .next_back()
            .expect("The range given to generate_number_in_range must have a last element.");
        assert!(end >= 1, "The end of the range should be 1 or higher");
        if end == start {
            return start;
        }
        let random_number = self.next();
        let range = (end - start + 1) as f64;

        ((random_number as f64 / MAX as f64) * range) as u16 + start
    }

    /// Uses the formula `x_{n+1} = (A * x_n) + C` where `A = 0x5D588B656C078965` and
    /// C = 0x00269EC3.
    fn next(&mut self) -> u64 {
        self.current_seed = self.current_seed.wrapping_mul(A).wrapping_add(C);
        self.current_seed
    }

    pub fn roll_chance(&mut self, num: u16, denom: u16) -> bool {
        assert!(denom != 0);
        self.generate_random_number_in_range(1..=denom) <= num
    }

    fn seed_from_current_time() -> u64 {
        time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs()
    }
}
