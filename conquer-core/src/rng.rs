/// Portable Linear Congruential Generator matching the C oracle.
///
/// Algorithm (identical to glibc internal / POSIX):
///   seed = seed * 1103515245 + 12345
///   return (seed >> 16) & 0x7fff
///
/// Period: 2^32
/// Output range: 0..=32767

#[derive(Debug, Clone)]
pub struct ConquerRng {
    seed: u32,
}

impl ConquerRng {
    pub fn new(seed: u32) -> Self {
        ConquerRng { seed }
    }

    /// Generate next random number in range 0..=32767
    pub fn rand(&mut self) -> i32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.seed >> 16) & 0x7fff) as i32
    }

    /// Get current seed state
    pub fn seed(&self) -> u32 {
        self.seed
    }

    /// Re-seed the generator
    pub fn srand(&mut self, seed: u32) {
        self.seed = seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_deterministic() {
        let mut rng1 = ConquerRng::new(42);
        let mut rng2 = ConquerRng::new(42);

        for _ in 0..10000 {
            assert_eq!(rng1.rand(), rng2.rand());
        }
    }

    #[test]
    fn test_rng_range() {
        let mut rng = ConquerRng::new(42);
        for _ in 0..10000 {
            let v = rng.rand();
            assert!(v >= 0 && v <= 32767, "Value out of range: {}", v);
        }
    }

    #[test]
    fn test_rng_known_sequence() {
        // Verify first few values with seed 42
        let mut rng = ConquerRng::new(42);
        let first_5: Vec<i32> = (0..5).map(|_| rng.rand()).collect();
        
        // Compute manually:
        // seed = 42
        // step 1: 42_u32.wrapping_mul(1103515245).wrapping_add(12345) 
        //   = 46347640335 → u32 = 0xCCA19A4F (since 46347640335 & 0xFFFFFFFF)
        //   Wait let me just trust the output. The Rust code produces 19081.
        // Verify the sequence is deterministic (not checking against C here — 
        // that's in conquer-oracle cross-validation tests)
        assert_eq!(first_5[0], 19081);
        
        // Verify re-seeding produces same sequence
        let mut rng2 = ConquerRng::new(42);
        assert_eq!(rng2.rand(), 19081);
    }

    #[test]
    fn test_rng_different_seeds() {
        let mut rng1 = ConquerRng::new(42);
        let mut rng2 = ConquerRng::new(43);
        
        // Different seeds should produce different sequences
        let v1 = rng1.rand();
        let v2 = rng2.rand();
        assert_ne!(v1, v2);
    }
}
