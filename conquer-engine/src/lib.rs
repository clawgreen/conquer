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
// - trade: Trade routes, commodities, market logic (Sprint 3)
// - diplomacy: Diplomatic status, nation relationships (Sprint 3)
// - events: Random events, weather, tax revolts, volcanoes (Sprint 3)
// - commands: Player commands (form, attack, designate, construct) (Sprint 3)
// - turn: Turn processing, end-of-turn updates (Sprint 3)
// - nation: Nation creation, new nation formation (Sprint 3)

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

// Sprint 3 modules - scaffolded but disabled due to API mismatch with conquer-core
// pub mod trade;
// pub mod diplomacy;
// pub mod events;
// pub mod commands;
// pub mod turn;
// pub mod nation;

pub use conquer_core;
