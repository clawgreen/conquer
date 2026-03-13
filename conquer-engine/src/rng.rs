// conquer-engine/src/rng.rs — Deterministic RNG wrapper for game engine
//
// Re-exports ConquerRng from conquer-core and adds game-specific helpers.

pub use conquer_core::rng::ConquerRng;

/// Extension trait for game-specific RNG operations
pub trait RngExt {
    /// Random number in range [0, n) — matches C `rand() % n`
    fn rand_mod(&mut self, n: i32) -> i32;

    /// Random number in range [lo, hi] inclusive
    fn rand_range(&mut self, lo: i32, hi: i32) -> i32;

    /// Returns true with probability percent/100
    fn percent_chance(&mut self, percent: i32) -> bool;
}

impl RngExt for ConquerRng {
    fn rand_mod(&mut self, n: i32) -> i32 {
        if n <= 0 {
            return 0;
        }
        self.rand() % n
    }

    fn rand_range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + self.rand_mod(hi - lo + 1)
    }

    fn percent_chance(&mut self, percent: i32) -> bool {
        self.rand_mod(100) < percent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rand_mod() {
        let mut rng = ConquerRng::new(42);
        for _ in 0..1000 {
            let v = rng.rand_mod(10);
            assert!(v >= 0 && v < 10);
        }
    }

    #[test]
    fn test_rand_range() {
        let mut rng = ConquerRng::new(42);
        for _ in 0..1000 {
            let v = rng.rand_range(5, 15);
            assert!(v >= 5 && v <= 15);
        }
    }

    #[test]
    fn test_percent_chance_boundaries() {
        let mut rng = ConquerRng::new(42);
        // 0% should never happen
        for _ in 0..100 {
            assert!(!rng.percent_chance(0));
        }
        // 100% should always happen
        for _ in 0..100 {
            assert!(rng.percent_chance(100));
        }
    }

    #[test]
    fn test_deterministic_sequence() {
        let mut rng1 = ConquerRng::new(42);
        let mut rng2 = ConquerRng::new(42);
        for _ in 0..1000 {
            assert_eq!(rng1.rand_mod(100), rng2.rand_mod(100));
        }
    }
}
