// conquer-engine: Game logic (Phase 2)
//
// Modules:
// - rng: Deterministic RNG wrapper with game-specific helpers
// - utils: Core utility functions (is_habitable, tofood, fort_val, etc.)
// - worldgen: Complete world generation matching C makeworl.c
// - economy: Food, gold, population, military upkeep, inflation
// - combat: Full combat resolution (land & naval)
// - movement: Army/navy movement costs, ZoC, sector taking
// - magic: All 31 powers, spell casting, power acquisition
// - npc: NPC AI (diplomacy, drafting, city building, army management)
// - monster: Monster/pirate/nomad/savage AI
// - navy: Fleet management (add/sub ships, loading, storms)

pub mod rng;
pub mod utils;
pub mod worldgen;
pub mod economy;
pub mod combat;
pub mod movement;
pub mod magic;
pub mod npc;
pub mod monster;
pub mod navy;

pub use conquer_core;
