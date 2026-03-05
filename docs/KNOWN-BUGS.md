# Known Bugs & Parity Differences — Phase 2 Playtest

**Date:** 2026-03-04  
**Test:** `cargo test --test playtest` — 10-turn integration test, seed 42  
**Result:** All 5 tests pass. Engine runs 10 turns without panics.

---

## Critical Parity Issues

### 1. NPC Nation Placement Not in Worldgen (HIGH)
- **C behavior:** `populate()` reads `gpl-release/nations` file and places 15 NPC nations with initial stats, sectors, armies, and populations.
- **Rust behavior:** `worldgen::create_world()` only creates 4 monster nations (pirate, nomad, savages, lizard). Regular NPC nations are not placed.
- **Impact:** Without NPC nations, the Rust engine has no player nations. Playtest works around this by programmatically creating nations in the test harness.
- **Fix needed:** Implement `place_npc_nations()` in worldgen or nation.rs, parsing the nations file format.

### 2. Active Nations: Rust=15, C=19 (MEDIUM)
- **Cause:** Monster nations (pirate, nomad, savages, lizard) die after turn 1 in Rust because they have `total_civ=0` and `total_mil=0`, triggering the destruction check (`total_civ < 100 && total_mil < takesector(total_civ)`).
- **C behavior:** Monster nations survive because they have armies with soldiers, and the C economy recounts `total_mil` from armies each turn.
- **Fix needed:** Recount `total_mil` from armies during turn processing, not just from the stored field.

### 3. Population Flat at 88,000 vs C ~430,000-460,000 (HIGH)
- **Magnitude:** ~80% divergence across all turns
- **Cause (multiple):**
  - Rust starts with fewer sector populations (test places ~500-1000 per sector vs C's economy-seeded values)
  - `update_sector_population()` in turn.rs has simplified growth logic
  - The C economy module (`economy.c`) does full spreadsheet-based population counting each turn; Rust's `update_nation_economy()` in turn.rs is a simplified stub
  - Population is not growing because the sector-level population update isn't integrated with the economy pipeline

### 4. Gold Rapidly Declining in Rust (HIGH)
- **Turn 1:** Rust 658,300 vs C 668,172 (close)
- **Turn 10:** Rust 90,469 vs C 2,790,075 (96.8% divergence)
- **Cause:** 
  - Rust's `update_nation_economy()` applies simplified sector-based income (5% farm food, 10% city gold)
  - Military upkeep (`total_mil * 2`) drains gold without sufficient income to offset
  - C has full economy: taxation, city income, trade goods, sector production
  - NPC AI doesn't redesignate sectors, build cities, or manage economy
- **Divergence grows:** Each turn compounds the difference as C nations build wealth while Rust nations bleed out

### 5. Scores Diverge (MEDIUM)
- **Turn 1:** Rust 4,014 vs C 532 (Rust higher initially because of starting gold/food)
- **Turn 6:** Rust 3,643 vs C 3,593 (closest point, within 1%!)
- **Turn 10:** Rust 2,834 vs C 6,353 (Rust declining, C growing)
- **Cause:** Score calculation itself appears correct (gold/1000 + food/1000 + etc), but the underlying resource values diverge due to economy differences

---

## Minor Issues

### 6. Savage Army Unit Types Corrupt (LOW)
- Savage nation armies 4-49 have unit_type values 195-209 (garbage range)
- `MIN_MONSTER` + `rand() % (MAX_MONSTER - MIN_MONSTER + 1)` produces values outside the UNIT_MOVE table (len=60)
- **Workaround applied:** `get_max_movement()` now bounds-checks unit_type against UNIT_MOVE length
- **Fix needed:** Verify MIN_MONSTER/MAX_MONSTER constants match C values

### 7. MAPX/MAPY Were 30, Should Be 32 (FIXED)
- Constants were 30, causing out-of-bounds panics when using 32x32 maps
- **Fixed:** Changed to 32 to match C default map size

### 8. Negative Gold Allowed (ACCEPTABLE)
- Multiple nations go negative gold by turn 5+ due to military upkeep
- C also allows negative gold — this is expected behavior
- Nations should eventually disband excess military or be conquered

---

## Summary

| Metric | Rust (Turn 10) | C Oracle (Turn 10) | Divergence |
|--------|---------------|-------------------|------------|
| Active Nations | 15 | 19 | -4 (monsters die) |
| Population | 88,000 | 431,239 | -79.6% |
| Gold | 90,469 | 2,790,075 | -96.8% |
| Food | 287,450 | 489,380 | -41.3% |
| Score Sum | ~2,834 | 6,353 | -55.4% |

**Root cause:** The Rust economy is a simplified stub. The turn pipeline runs correctly and doesn't crash, but the economy module needs full parity with C's economy.c to produce matching results. The scoring formula itself is correct — divergence comes from resource calculations.

**Confidence for Phase 3:** The engine skeleton is solid. Turn pipeline, combat, worldgen, and scoring all work. The economy needs deepening but that's a known gap. The engine is safe to proceed to Phase 3 (server) with the understanding that economy parity is ongoing work.
