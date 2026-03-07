# Sprint: Wire Remaining Turn Pipeline

## Context
The C original runs 13 steps per turn. Only 4 are wired in the Rust server (`store.rs` `run_turn`).
The engine code EXISTS for most of these — they just need to be called from `run_turn` in `conquer-db/src/store.rs`.

Reference: `original/update.c` function `update()` lines 48-128.

## Rules
- Read the C source for EACH feature before implementing
- All engine functions must take `&mut GameState` (dynamic vecs), NOT fixed-size arrays
- If a function currently uses `[[Sector; MAPY]; MAPX]`, refactor it to use `&mut GameState`
- Run `cargo test` after each change
- Run `cargo clippy` to check for warnings
- Commit after each numbered item below

## Tasks (in order)

### T1: Wire NPC AI into turn pipeline
- File: `conquer-db/src/store.rs` in `run_turn()`
- Call `conquer_engine::npc::nation_run()` for each NPC nation (active > 16)
- The function already takes `&mut GameState` — just call it
- Reference: `original/update.c` line 59 `updexecs()` → calls `nationrun()` for NPC nations
- Also handle CMOVE: if a PC nation didn't submit actions (`execute(TRUE)==0`), run NPC AI for them too
- Track which nations submitted actions this turn from `game.actions`

### T2: Wire monster updates
- Call `conquer_engine::monster::update_monsters()` in turn pipeline
- Should run BEFORE combat (C order: monster → combat)
- Reference: `original/npc.c` `monster()` function — runs `do_nomad()`, `do_pirate()`, `do_savage()`, `do_lizard()`
- Check that `update_monsters` handles all monster types, compare with C

### T3: Implement sector capture (updcapture)
- Port `original/update.c` `updcapture()` (line 797) to Rust
- Add `pub fn update_capture(state: &mut GameState)` to `conquer-engine/src/movement.rs` or new file
- Logic: for each nation, for each army, if army is sole occupier of a sector:
  - Unowned sector → capture it (set owner)
  - Enemy sector (at war) → capture it, flee civilians, news entry
  - Army must have >= TAKESECTOR soldiers (or >= 75 for NPCs)
  - Skip armies ONBOARD (on fleet)
  - Skip water sectors
- Wire into turn pipeline AFTER combat

### T4: Wire trade processing
- Call `conquer_engine::trade::check_trade()` or equivalent in turn pipeline
- Reference: `original/update.c` line 74 `uptrade()`
- Check what `original/trade.c` `uptrade()` does and ensure Rust equivalent exists
- Wire AFTER capture, BEFORE military reset

### T5: Wire random events
- Call `conquer_engine::events::process_nation_events()` or `generate_random_event()` in turn pipeline
- Reference: `original/randeven.c` `randomevent()` (line 309)
- Events include: tax revolts, storms, volcanoes, plagues, discoveries, weather bonuses, barbarian raids
- Wire AFTER military reset, BEFORE sector updates

### T6: Implement leader updates (updleader)
- Port `original/update.c` `updleader()` (line 1471) to Rust
- Add `pub fn update_leaders(state: &mut GameState, rng: &mut ConquerRng)` to economy.rs or new file
- Logic:
  - Monster nations spawn new monsters in Spring
  - Leader birth rates by class (King/Emperor/Wizard etc)
  - Leaders gain experience/wisdom over time
- Wire AFTER commodities update

### T7: Wire NPC score cheat
- Port `original/update.c` `cheat()` (line 459) to Rust
- If NPC nation score is far below average, give them bonus gold
- Controlled by game setting `npc_cheat` (already in GameSettings)
- Wire into scoring step

### T8: Implement tradegood attribute bonuses (att_bonus)
- Port `original/update.c` `att_bonus()` (line 128) to Rust
- Tradegoods in sectors provide attribute bonuses to nations
- Add to turn pipeline after scoring

### T9: Implement civilian migration (move_people)
- Port `original/update.c` `move_people()` (line 1558) to Rust
- Civilians migrate between sectors based on attractiveness
- Called per-nation inside updexecs — add to sector updates or as separate step
- Reference: attractiveness = f(designation, population, fortress level)

### T10: Fix fixed-array legacy code
- Refactor `conquer-engine/src/turn.rs` `update_turn()` to use `&mut GameState` instead of fixed arrays
- Remove `arrays_to_gamestate` / `gamestate_to_arrays` conversion functions
- Remove or update `MAPX`/`MAPY` constants in `conquer-core/src/constants.rs` (they limit maps to 32x32)
- Refactor `events.rs` functions that take `[[Sector; MAPY]; MAPX]` to take `&GameState`
- Refactor `commands.rs` `is_next_to_water` similarly

### T11: Verify turn order matches C
- After all wiring, verify the turn pipeline order in `run_turn()` matches C:
  1. updexecs (execute player commands + NPC AI for idle PCs)
  2. monster (monster nation updates)
  3. combat
  4. updcapture (sector capture)
  5. uptrade (trade processing)
  6. updmil (military reset)
  7. randomevent (random events)
  8. updsectors (population/economy)
  9. updcomodities (food/resources)
  10. updleader (leader births/growth)
  11. score + cheat (scoring + NPC cheat)
  12. att_bonus (tradegood bonuses)
- Reorder the existing calls in `run_turn()` to match

### T12: Add tests
- Add integration test that creates a game with NPCs, runs 5 turns, verifies:
  - NPCs have moved armies
  - NPCs have captured sectors
  - Random events have fired
  - Scores have changed
  - Monster nations have acted
- Test in `conquer-db/src/store.rs` tests module
