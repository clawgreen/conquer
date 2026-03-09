# Sprint: NPC AI & Economy Parity

## Context
The 472-TODO build sprint completed the structure, but two critical areas are incomplete:
1. **NPC AI doesn't move armies** — `nation_run()` handles diplomacy/tax/forts/magic but never calls `npc_army_move()`. The entire attractiveness system that guides NPC army decisions is unimplemented.
2. **Economy has gaps** — core functions exist but some update.c pipeline steps are simplified or missing details.

## Reference Files
- `original/npc.c` — NPC AI (1720 lines). Key functions: `nationrun()`, `pceattr()`, `atkattr()`, `defattr()`, `n_unowned()`, `n_trespass()`, `n_toofar()`, `n_people()`, `n_defend()`, `n_attack()`, `n_undefended()`, `n_between()`, `n_survive()`, `find_avg_sector()`, `armymove()` (in original/move.c)
- `original/update.c` — Turn pipeline (1622 lines). Key: `update()`, `updsectors()`, `updcomodities()`, `updmil()`, `updexecs()`, `updcapture()`, `updleader()`, `moveciv()`, `verify_ntn()`, `whatcansee()`
- `original/combat.c` — Combat resolution (1377 lines)
- `original/misc.c` — Utilities (1903 lines): `armymove()` lives here (NPC movement)
- `conquer-engine/src/npc.rs` — Current Rust NPC AI (896 lines, missing movement)
- `conquer-engine/src/movement.rs` — Has `npc_army_move()` (never called from NPC AI)
- `conquer-engine/src/economy.rs` — Economy (1049 lines)
- `conquer-db/src/store.rs` — `run_turn()` pipeline (line 754)

## Rules
1. ALWAYS read the C source before implementing
2. All functions use `&mut GameState` — no fixed-size arrays
3. `cargo test` after each change
4. Commit after each task
5. One pre-existing test failure: `store::tests::test_join_game` — ignore it

---

## Part A: NPC Attractiveness System (the brain)

The C NPC AI uses a 2D `attr[][]` grid (same size as map) that scores how "attractive" each sector is. NPCs move armies toward high-attractiveness sectors. This entire system is missing.

### T1: Add attractiveness grid infrastructure
- Add `attr: Vec<Vec<i32>>` as a temporary structure passed into NPC functions
- Create helper `fn create_attr_grid(map_x: usize, map_y: usize) -> Vec<Vec<i32>>` (initialized to 0)
- Create `fn clear_attr_grid(attr: &mut Vec<Vec<i32>>)`

### T2: Port `find_avg_sector()` — global average calculations
- C: `original/npc.c:980` — calculates `Avg_food`, `Avg_tradegood`, `Avg_soldiers[nation]`
- Counts useable land (not water/peak), averages food and tradegood values
- For each nation whose armies can't be seen, estimates avg soldiers per occupied sector
- Return struct `NpcAverages { avg_food: i32, avg_tradegood: i32, avg_soldiers: [i64; NTOTAL] }`

### T3: Port `n_unowned()` — attract toward unowned land
- C: `original/npc.c:1355` 
- +450 near capitol (within 4), +300/+500 for trade goods, +300 unowned, +100 nomad land
- +50*tofood for visible sectors, Avg_food for unseen
- Avg_tradegood for unseen sectors
- /5 if not habitable

### T4: Port `n_trespass()` — avoid foreign territory
- C: `original/npc.c:1327`
- Set attr=1 for sectors owned by non-allied nations (>2 from capitol, not at war, not allied)

### T5: Port `n_toofar()` — stay near capitol  
- C: `original/npc.c:1344`
- Set attr=1 for all sectors outside NPC range (stx..endx, sty..endy)

### T6: Port `n_people()` — population attractiveness
- C: `original/npc.c:1527`
- Add (or subtract) people/4 to owned habitable sectors
- Called with `doadd=TRUE` before infantry moves, `doadd=FALSE` after, before leader moves

### T7: Port `n_survive()` — capitol defense urgency
- C: `original/npc.c:1579`
- +1000 if capitol lost
- For each nation at war: add enemy soldier count near capitol (within 2)
- Handles both visible and estimated (Avg_soldiers) cases

### T8: Port `n_defend()` — defensive sector scoring
- C: `original/npc.c:1471`
- Add enemy soldiers/10 in own sectors
- +80 near capitol
- Score by movement cost (cheap terrain = good defense)
- +50 for cities, proportional to population share

### T9: Port `n_attack()` — offensive targeting
- C: `original/npc.c:1510`  
- Target enemy cities with +500 if attacker outnumbers 3:2
- Handle both visible and estimated cases
- +UNS_CITY_VALUE for unseen enemy sectors

### T10: Port `n_undefended()` — target empty enemy sectors
- C: `original/npc.c:1542`
- +100 if habitable & unoccupied, +60 if occupied, +30 if not habitable

### T11: Port `n_between()` — strategic blocking
- C: `original/npc.c:1544`
- +60 for sectors between two capitols (bounding box)

### T12: Port `pceattr()` — peacetime attractiveness
- C: `original/npc.c:1708`
- `n_unowned()` ×3, then `n_trespass()`, `n_toofar()`, `n_survive()`

### T13: Port `atkattr()` — attack attractiveness
- C: `original/npc.c:1674`
- `n_unowned()`, then per-enemy: `n_between()` + `n_undefended()` + `n_attack()` (WAR), ×4 for JIHAD
- Then `n_toofar()`, `n_trespass()`, `n_survive()`

### T14: Port `defattr()` — defensive attractiveness
- C: `original/npc.c:1650`
- `n_unowned()`, then per-enemy: `n_defend()` + `n_between()` + `n_undefended()`
- Then `n_trespass()`, `n_toofar()`, `n_survive()`

---

## Part B: Wire NPC Army Movement into nation_run()

### T15: Port attacker/defender decision logic into `nation_run()`
- C: `original/npc.c:1105-1135`
- If at peace → set peace=8, call pceattr()
- If at war → peace=12, decide attacker vs defender per enemy:
  - Compare `tmil*(aplus+100)` ratio → if >rand%100, set ATTACK on non-militia armies, call atkattr()
  - Otherwise, small armies → DEFEND, big → ATTACK, call defattr()

### T16: Wire army movement loop into `nation_run()`
- C: `original/npc.c:1140-1160`
- `n_people(TRUE)` — add population attractiveness
- Move infantry first: loop `armynum=1..MAXARM`, if soldiers>0 and type<MINLEADER, call `npc_army_move()`
- `n_people(FALSE)` — subtract population attractiveness
- Move leaders/monsters: loop again, if soldiers>0 and type>=MINLEADER, call `npc_army_move()`
- Track `loop` (number of successful moves) for status update

### T17: Port NPC strategy status update
- C: `original/npc.c:1163-1185`
- After movement, update NPC active status based on `loop` count:
  - loop<=1 → 0FREE, loop>=6 → 6FREE, loop>=4 → 4FREE, else 2FREE
  - Per alignment: GOOD_xFREE, NEUTRAL_xFREE, EVIL_xFREE
  - Skip if ISOLATIONIST

### T18: Port "don't attack from own city" rule
- C: `original/npc.c:1306-1315`
- After movement, for each army in ATTACK status sitting in own fortified sector:
  - 50% chance → DEFEND, 50% → GARRISON

---

## Part C: armymove() — NPC Movement Logic

### T19: Verify/fix existing `npc_army_move()` in movement.rs
- C: `original/misc.c` — the actual `armymove()` function
- Compare against existing `npc_army_move()` in `conquer-engine/src/movement.rs:346`
- Current Rust version only looks at immediate neighbors (1-step). Verify this matches C.
- Fix: move_cost grid must be properly calculated before NPC movement (C calls `prep()` which computes movecost)

### T20: Ensure move_cost grid is populated before NPC AI runs
- C: `prep()` in misc.c calculates `movecost[][]` for each nation
- Must be recalculated per-nation before their army moves (different races have different costs)
- Add `calculate_move_costs(state: &mut GameState, nation_idx: usize)` if not already present

---

## Part D: Economy Parity Fixes

### T21: Audit `updsectors()` against C original
- C: `original/update.c:440` — compare line-by-line with `conquer-engine/src/economy.rs:145`
- Check: depletion without capitol (PDEPLETE), spreadsheet revenue calculation, starvation
- Verify the `disarray` flag (no capitol → civil disarray → higher depletion)

### T22: Audit `updcomodities()` against C original
- C: `original/update.c:729` — food consumption, spoilage, starvation effects
- Verify starvation penalties: population loss, reputation damage, prestige loss
- Check eat_rate and spoil_rate application

### T23: Audit `updmil()` against C original
- C: `original/update.c:538` — movement point reset, maintenance, army max, dead army cleanup
- Verify army maintenance costs deducted from gold
- Verify `max_move` reset per race/terrain
- Verify dead army removal (soldiers <= 0)

### T24: Audit `move_people_gs()` against C original
- C: `original/update.c:1086` — civilian migration between sectors
- Port `moveciv()` exactly — people flow from high-pop to low-pop owned sectors
- Check that migration respects designation and terrain

### T25: Audit `update_leaders()` against C original
- C: `original/update.c:902` — leader spawning, monster spawning in Spring
- Verify leader births match C conditions
- Verify monster spawning frequency and placement

### T26: Audit scoring against C original
- C: `original/misc.c` — `score_one()`, `score()`
- Compare `conquer-engine/src/turn.rs` `calculate_scores_gs()` against C formulas
- Verify all score components: sectors, military, gold, jewels, powers, etc.

---

## Part E: Integration Tests

### T27: NPC movement smoke test
- Create a small world (16×16), place 2 NPC nations and 1 PC nation
- Run 10 turns, verify NPCs actually move armies (positions change)
- Verify NPCs expand into unowned territory
- Verify NPCs attack enemies when at war

### T28: Economy balance test
- Create world, run 20 turns NPC-only
- Verify nations grow (population increases)
- Verify economy flows (gold changes, food consumed)
- Verify no nation crashes to 0 in normal conditions

### T29: Full turn pipeline comparison
- Run 5 turns with seed 42 in Rust
- Verify no panics, all nations still active
- Log key stats per turn: total_civ, total_mil, treasury_gold, total_sectors per nation
- Compare growth curves between NPC nations (should be reasonably balanced)

---

## Summary

| Part | Description | Tasks | Critical |
|------|-------------|-------|----------|
| A | Attractiveness system | T1-T14 | ⚡ Core NPC brain |
| B | Wire NPC movement | T15-T18 | ⚡ Makes NPCs act |
| C | Movement logic fixes | T19-T20 | ⚡ Correct movement |
| D | Economy parity | T21-T26 | 🔧 Balance fixes |
| E | Integration tests | T27-T29 | ✅ Verification |
| **Total** | | **29** | |

Parts A+B+C are the critical path — without them, NPCs are decoration.
Part D brings economy closer to C parity.
Part E proves it works.
