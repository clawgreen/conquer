// conquer-engine: Game logic (Phase 2)
//
// Modules:
// - rng: Deterministic RNG wrapper with game-specific helpers
// - utils: Core utility functions (is_habitable, tofood, fort_val, etc.)
// - worldgen: Complete world generation matching C makeworl.c
// - economy: Food, gold, population, military upkeep, inflation

pub mod rng;
pub mod utils;
pub mod worldgen;
pub mod economy;

pub use conquer_core;
