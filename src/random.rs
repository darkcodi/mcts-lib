use rand::{Rng, random};

const MULTIPLIER_A: i64 = 1103515245;
const INCREMENT_C: i64 = 12345;
const DEFAULT_SEED: i64 = 3819201;

/// A trait for random number generation.
///
/// This allows for different random number generation strategies to be used with the MCTS search,
/// which is particularly useful for testing and ensuring reproducibility.
pub trait RandomGenerator: Default {
    /// Returns the next random `i32`.
    fn next(&mut self) -> i32;
    /// Returns a random `i32` within the specified range (exclusive of `to`).
    fn next_range(&mut self, from: i32, to: i32) -> i32;

    /// Selects a random element from a vector.
    fn get_random_from_vec<'a, K>(&mut self, vec: &'a Vec<K>) -> &'a K {
        vec.get(self.next_range(0, vec.len() as i32) as usize)
            .unwrap()
    }
}

/// A `RandomGenerator` that uses the `rand` crate for random number generation.
pub struct StandardRandomGenerator;

impl Default for StandardRandomGenerator {
    fn default() -> Self {
        StandardRandomGenerator
    }
}

impl RandomGenerator for StandardRandomGenerator {
    fn next(&mut self) -> i32 {
        random()
    }

    fn next_range(&mut self, from: i32, to: i32) -> i32 {
        rand::rng().random_range(from..to)
    }
}

/// A deterministic pseudo-random number generator.
///
/// This generator is useful for testing, as it will always produce the same sequence of numbers
/// for a given seed.
pub struct CustomNumberGenerator {
    seed: i64,
}

impl Default for CustomNumberGenerator {
    fn default() -> Self {
        CustomNumberGenerator::new(DEFAULT_SEED)
    }
}

impl RandomGenerator for CustomNumberGenerator {
    fn next(&mut self) -> i32 {
        self.seed = (self.seed * MULTIPLIER_A + INCREMENT_C) % (i32::MAX as i64);
        self.seed as i32
    }

    fn next_range(&mut self, from: i32, to: i32) -> i32 {
        (self.next() % (to - from)).abs() + from
    }
}

impl CustomNumberGenerator {
    /// Creates a new `CustomNumberGenerator` with the specified seed.
    pub const fn new(seed: i64) -> Self {
        Self { seed }
    }
}

#[cfg(test)]
mod tests {
    use crate::random::{CustomNumberGenerator, RandomGenerator};

    #[test]
    fn outputs_same_numbers() {
        let mut crg = CustomNumberGenerator::new(42);
        assert_eq!(crg.next_range(0, 10), 8);
        assert_eq!(crg.next_range(0, 10), 4);
        assert_eq!(crg.next_range(0, 10), 1);
        assert_eq!(crg.next_range(0, 10), 2);
        assert_eq!(crg.next_range(0, 10), 4);
    }

    #[test]
    fn random_from_vec_should_be_same() {
        let vec = vec![432, 6542, 534, 6, 13, 645, 88, 2352, 345, 2667, 8287];
        let mut crg = CustomNumberGenerator::default();
        assert_eq!(*crg.get_random_from_vec(&vec), 6);
        assert_eq!(*crg.get_random_from_vec(&vec), 2667);
        assert_eq!(*crg.get_random_from_vec(&vec), 534);
        assert_eq!(*crg.get_random_from_vec(&vec), 8287);
        assert_eq!(*crg.get_random_from_vec(&vec), 6);
    }
}
