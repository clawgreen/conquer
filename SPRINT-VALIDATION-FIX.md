# Sprint: Fix All Action Validation Gaps

## Context
The `apply_action_to_state` function in `conquer-db/src/store.rs` has many actions that 
skip validation the C original enforces. Some actions are "internal-only" (used by NPC AI 
and turn processing) but the web client can submit them directly via the REST API.

Two-part fix:
1. **Split actions into PLAYER and ENGINE categories** — player-submitted actions get full 
   validation; engine-internal actions are blocked from the REST API
2. **Add missing validation** to every player action, matching C behavior exactly

## Reference Files
- `original/cexecute.c` — the C action executor (what validation it does)
- `original/commands.c` — the C interactive command loop (what it allows)
- `original/misc.c` — movement cost, reachability, scoring
- `original/move.c` — army movement with move point deduction
- `conquer-db/src/store.rs` — `apply_action_to_state()` and route handlers

## Rules
- Read the C source for EACH action before fixing
- Every player action must validate: ownership, cost, game rules
- Engine-only actions must be blocked from player API submissions
- Run `cargo test` after each change
- Commit after each task

## Tasks

### T1: Categorize Actions — Player vs Engine-Only
Add a method `Action::is_player_action() -> bool` in `conquer-core/src/actions.rs`.

**Player actions** (can be submitted via REST API):
- MoveArmy, MoveNavy
- AdjustArmyStat (status change)
- SplitArmy, CombineArmies, DivideArmy
- DraftUnit, ConstructFort, BuildRoad, ConstructShip
- LoadArmyOnFleet, UnloadArmyFromFleet, LoadPeopleOnFleet, UnloadPeople
- CastSpell, BuyMagicPower
- ProposeTrade, AcceptTrade, RejectTrade
- AdjustDiplomacy, BribeNation, SendTribute
- HireMercenaries, DisbandToMerc
- DesignateSector (with validation)
- AdjustTax (tax rate/charity only, not active status)
- AdjustPopulation (terror adjustment only — from C 'P' command)
- ChangeName, ChangePassword

**Engine-only actions** (blocked from player API, used by NPC AI and turn processing):
- AdjustArmyMen — lets you set arbitrary soldier counts
- AdjustArmyMove — lets you set arbitrary movement points
- AdjustNavyMove — same for navies
- AdjustNavyGold — lets you add/remove gold directly
- AdjustNavyMerchant, AdjustNavyWarships, AdjustNavyGalleys, AdjustNavyHold, AdjustNavyCrew
- AddSectorCiv, AdjustSectorCiv — direct population manipulation
- TakeSectorOwnership — direct ownership (bypasses capture rules)
- IncreaseFort — bypasses cost (use ConstructFort instead)
- IncreaseAttack, IncreaseDefense — bypasses cost
- ChangeMagic — bypasses cost/prereqs (use BuyMagicPower instead)
- AdjustSpellPoints — direct manipulation
- DestroyNation — god-only action

In the REST API route that accepts player actions (`POST /games/{id}/actions`), 
filter out any action where `is_player_action()` returns false. Return 400 error.

### T2: MoveArmy — Add Movement Point Deduction
Read `original/update.c` `armymove()` (line 247) and `original/misc.c` `land_reachp()`.

Fix MoveArmy handler to:
- Check army has movement points > 0
- Calculate move cost for destination terrain (altitude + vegetation)
- Use `conquer_engine::movement::update_move_costs()` or port the cost table
- Deduct movement points
- If not enough movement, reject the move
- Armies with status MARCH get 2x movement (C: armymove checks MARCH)
- Armies with GARRISON or RULE status can't move at all
- Skip move cost for SCOUT status (moves freely but can't fight)

### T3: MoveNavy — Add Water Validation + Movement Deduction
Read `original/misc.c` `water_reachp()` (line 401).

Fix MoveNavy handler to:
- Validate destination is WATER altitude (navies can only be on water)
- Check fleet has movement points > 0
- Deduct movement cost (simpler than army — usually 1 per tile)
- Adjacent moves only (no teleporting)
- Fleet speed affects max moves per turn (from `fltspeed()`)

### T4: DesignateSector — Add Ownership + Cost + Rules
Read `original/cexecute.c` XSADES and `original/commands.c` redesignation.

Fix DesignateSector handler to:
- Verify sector is owned by the acting nation (C: "ERROR: redesignate sector not owned")
- Charge DESCOST gold for redesignation
- If setting to CAPITOL, update nation.cap_x/cap_y
- Validate designation is allowed for the terrain (no farms on mountains, etc.)
- Can't redesignate to ROAD if not enough population (use BuildRoad action instead)
- Track roads-per-turn limit

### T5: Fix IncreaseFort (engine-only) — Players Must Use ConstructFort
Already blocked by T1 (IncreaseFort becomes engine-only).
Verify ConstructFort has all the right validation (ownership, cost, max level).
The existing ConstructFort handler looks correct — just verify.

### T6: Fix IncreaseAttack/IncreaseDefense — Add Cost
Read `original/commands.c` for attack/defense upgrade cost.

These should either:
a) Be engine-only (blocked by T1) and have a new `BuyAttackBonus`/`BuyDefenseBonus` action, OR
b) Add gold cost validation directly

In C, increasing attack/defense costs gold (APTS/DPTS constants).
Add proper player actions with cost validation.

### T7: Fix AdjustTax — Restrict Player Fields
Players should only be able to change:
- tax_rate (0-20 range, matching C)
- charity (0-10 range)

They should NOT be able to change `active` (NPC/PC status) via this action from the API.
Split validation: if player-submitted, ignore the `active` field.

### T8: Fix AdjustPopulation — Restrict to Terror Only
In C, the player 'P' command adjusts terror (spending gold for propaganda).
Players should NOT be able to directly set popularity or reputation.
Add gold cost for terror adjustment. Block popularity/reputation changes from player API.

### T9: Fix BribeNation — Add Probability Roll
Currently bribery ALWAYS succeeds (improves status by 1 step).
C has probability: 50% same NPC type, 30% neutral, 20% otherwise, +20% same race.
Add the RNG roll. Use ConquerRng seeded from turn + nation for determinism.

### T10: Fix LoadArmyOnFleet — Check Fleet Capacity  
Read `original/navy.c` `loadfleet()`.
Current handler checks `army_num >= MAXARM` which may not be right.
Fix to check actual hold capacity via `fleet_hold()` vs army size.
Also verify army location matches fleet location.

### T11: Movement Point Reset
Verify `updmil()` in the turn pipeline properly resets movement points for all armies/navies
at the start of each turn. Check it matches C:
- Army movement = nation.maxmove (varies by race)
- Navy movement = fltspeed() (varies by ship composition)
- GARRISON/RULE status = 0 movement

### T12: Web Client — Remove Internal Action Submissions
Audit `web/src/game/gameScreen.ts` for any code that submits engine-only actions.
Replace with proper player actions:
- Don't submit IncreaseFort → use ConstructFort
- Don't submit IncreaseAttack/IncreaseDefense → use new BuyAttackBonus/BuyDefenseBonus
- Don't submit AdjustArmyMen/AdjustSectorCiv → these should never come from UI
- Don't submit TakeSectorOwnership → handled by engine during capture

### T13: Add Server-Side Action Validation Tests
Add tests in store.rs that verify:
- Engine-only actions are rejected from player API
- MoveArmy deducts movement and blocks on water
- MoveNavy blocks on land
- DesignateSector requires ownership
- ConstructFort charges gold
- BribeNation has probability (sometimes fails)
- Armies can't move with 0 movement points
- Tax rate clamped to valid range
