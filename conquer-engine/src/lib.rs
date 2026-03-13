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

pub mod combat;
pub mod commands;
pub mod diplomacy;
pub mod economy;
pub mod events;
pub mod magic;
pub mod monster;
pub mod movement;
pub mod nation;
pub mod navy;
pub mod npc;
pub mod rng;
pub mod trade;
pub mod turn;
pub mod utils;
pub mod worldgen;

pub use conquer_core;
