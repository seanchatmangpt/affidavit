//! A tiny, zero-dependency deterministic RNG (SplitMix64).
//!
//! The genetic algorithm needs reproducible randomness without pulling in the
//! `rand` crate (confevo is deliberately dependency-free). [`Rng`] is a SplitMix64
//! generator: fast, well-distributed, and — most importantly — *seeded and
//! deterministic*, so the same seed yields the same sequence forever.

/// A seeded SplitMix64 pseudo-random number generator.
///
/// SplitMix64 is the seeding generator recommended by the xoshiro authors. It has
/// a 64-bit state, a period of 2^64, and passes the usual statistical batteries —
/// more than enough for a feature-flag search, while staying a handful of lines of
/// pure arithmetic with no dependencies.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Create a generator seeded with `seed`. Equal seeds produce equal streams.
    pub fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    /// Return the next 64-bit value and advance the state.
    pub fn next_u64(&mut self) -> u64 {
        // SplitMix64: https://prng.di.unimi.it/splitmix64.c
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Return a uniform `f64` in the half-open interval `[0.0, 1.0)`.
    ///
    /// Uses the top 53 bits (the f64 mantissa width) so every representable value
    /// in `[0, 1)` is reachable and the distribution is uniform.
    pub fn next_f64(&mut self) -> f64 {
        const SCALE: f64 = 1.0 / ((1u64 << 53) as f64);
        ((self.next_u64() >> 11) as f64) * SCALE
    }

    /// Return `true` with probability `p` (clamped to `[0, 1]`).
    ///
    /// `p <= 0` is always `false`; `p >= 1` is always `true`.
    pub fn gen_bool(&mut self, p: f64) -> bool {
        if p <= 0.0 {
            return false;
        }
        if p >= 1.0 {
            return true;
        }
        self.next_f64() < p
    }

    /// Return a uniformly-distributed index in `[0, n)`.
    ///
    /// Panics if `n == 0`. (Callers in this crate always pass a non-empty pool.)
    pub fn below(&mut self, n: usize) -> usize {
        assert!(n > 0, "Rng::below requires n > 0");
        (self.next_u64() % (n as u64)) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = Rng::new(12345);
        let mut b = Rng::new(12345);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);
        // Astronomically unlikely to match across 100 draws if seeds differ.
        let any_diff = (0..100).any(|_| a.next_u64() != b.next_u64());
        assert!(any_diff);
    }

    #[test]
    fn next_f64_in_unit_interval() {
        let mut r = Rng::new(7);
        for _ in 0..10_000 {
            let x = r.next_f64();
            assert!((0.0..1.0).contains(&x), "out of range: {x}");
        }
    }

    #[test]
    fn gen_bool_bounds_are_deterministic() {
        let mut r = Rng::new(0);
        assert!(!r.gen_bool(0.0));
        assert!(r.gen_bool(1.0));
        assert!(!r.gen_bool(-5.0));
        assert!(r.gen_bool(2.0));
    }

    #[test]
    fn below_is_in_range() {
        let mut r = Rng::new(99);
        for _ in 0..10_000 {
            let i = r.below(7);
            assert!(i < 7);
        }
    }
}
