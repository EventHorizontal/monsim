use std::{time, u64::MAX};

use std::ops::RangeInclusive;

#[test]
fn test_prng_percentage_chance() {
    let mut lcrng = LCRNG::new(seed_from_time_now());
    let mut dist = [0u64; 100];
    for _ in 0..=10_000_000 {
        let n = lcrng.generate_number_in_range(0..=99) as usize;
        dist[n] += 1;
    }
    let avg_deviation = dist
        .iter()
        .map(|it| ((*it as f32 / 100_000.0) - 1.0).abs())
        .reduce(|it, acc| it + acc)
        .expect("We should always get some average value.")
        / 100.0;
    let avg_deviation = f32::floor(avg_deviation * 100_000.0) / 100_000.0;
    println!(
        "LCRNG has {:?}% average deviation (threshold is at 0.005%)",
        avg_deviation
    );
    assert!(avg_deviation < 5.0e-3);
}

#[test]
fn test_prng_idempotence() {
    let seed = seed_from_time_now();
    let mut lcrng_1 = LCRNG::new(seed);
    let mut generated_numbers_1 = [0; 10_000];
    for i in 0..10_000 {
        generated_numbers_1[i] = lcrng_1.next();
    }
    let mut lcrng_2 = LCRNG::new(seed);
    let mut generated_numbers_2 = [0; 10_000];
    for i in 0..10_000 {
        generated_numbers_2[i] = lcrng_2.next();
    }
    assert_eq!(generated_numbers_1, generated_numbers_2);
}

pub fn seed_from_time_now() -> u64 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[test]
fn test_prng_chance() {
    let mut lcrng = LCRNG::new(
        time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );

    let mut success = 0.0;
    for _ in 0..=10_000_000 {
        if lcrng.chance(33, 100) {
            success += 1.0;
        }
    }
    let avg_probability_deviation = (((success / 10_000_000.0) - 0.3333333333) as f64).abs();
    let avg_probability_deviation = f64::floor(avg_probability_deviation * 100_000.0) / 100_000.0;
    println!(
        "Average probability of LCRNG is off by {}% (threshold is at 0.005%)",
        avg_probability_deviation
    );
    assert!(avg_probability_deviation < 5.0e-3);
}

// LCRNG -> Linear Congruential Random Number Generator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LCRNG {
    start_seed: u64,
    current_seed: u64,
}

const A: u64 = 0x5D588B656C078965;
const C: u64 = 0x00269EC3;

impl LCRNG {
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
