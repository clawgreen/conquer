# Sprint: C→Rust Command Parity

## Context
The turn pipeline is now complete. But many player COMMANDS from the original C game 
have no Rust implementation. These are things a player does during their turn that 
generate Actions. The engine needs the logic; the server needs to handle them in 
`apply_action_to_state`.

Reference files:
- `original/commands.c` — main command loop (redesignate, draft, build)
- `original/extcmds.c` — army commands (split, combine, status change)
- `original/cexecute.c` — action executor (all XASTAT/XAMEN/etc codes)
- `original/forms.c` — diplomacy screen, score display, change settings
- `original/magic.c` — spell casting (domagic, dosummon, wizardry, god_magk)
- `original/navy.c` — fleet management (load/unload armies, build ships)
- `original/trade.c` — trade system (propose, accept, execute trades)
- `original/misc.c` — utility functions (spreadsheet, scoring, destroy, sackem)
- `original/move.c` — movement with pathfinding, zone of control

## Rules
- Read the C source for EACH feature before implementing
- All functions must take `&mut GameState`, NOT fixed arrays
- New Action variants go in `conquer-core/src/actions.rs`
- New engine functions go in the appropriate engine module
- Action handling goes in `apply_action_to_state` in `conquer-db/src/store.rs`
- Run `cargo test` after each change
- Commit after each task

## Tasks

### T1: Army Split
- Port `original/extcmds.c` `splitarmy()` (line 315)
- Add `SplitArmy { nation, army, soldiers }` action to actions.rs
- Logic: take soldiers from army, create new army at same location
- Find first empty army slot (soldiers == 0), put split soldiers there
- Add to apply_action_to_state

### T2: Army Combine  
- Port `original/extcmds.c` `combinearmies()` (line 115)
- Add `CombineArmies { nation, army1, army2 }` action
- Logic: merge army2 into army1 (must be same location, compatible types)
- Rules from C: can't combine different unit types, max soldiers cap
- Zero out army2 after merge

### T3: Army Status Change
- Port `original/extcmds.c` `change_status()` (line 161) 
- Already have `AdjustArmyStat` action — verify it handles all statuses:
  ATTACK, DEFEND, MARCH, SCOUT, GARRISON, RULE
- Port `nocomb_stat()` (line 91) — validate which statuses can't be set during combat
- Ensure TRADED, FLIGHT, MAGATT, MAGDEF, ONBOARD can't be manually set

### T4: Army Divide (equal split)
- Port `original/extcmds.c` — the `/` command divides army equally
- Add `DivideArmy { nation, army }` action
- Logic: split army in half, second half goes to new empty slot

### T5: Draft/Enlist Units
- `conquer-engine/src/commands.rs` has `draft_unit()` — verify it matches C
- Add `DraftUnit { nation, x, y, unit_type, count }` action
- Logic from C: costs gold, takes civilians from sector, creates army
- Must validate unit type is valid for nation's race
- Port `original/misc.c` `unitvalid()` equivalent

### T6: Construct Fortifications
- `commands.rs` has `construct_fort()` — verify vs C
- Add `ConstructFort { nation, x, y }` action
- Logic: costs gold, increases sector fortress level
- Only in owned sectors with cities/towns/forts

### T7: Build Roads
- `commands.rs` has `build_road()` — verify vs C
- Add `BuildRoad { nation, x, y }` action
- Limit roads per turn (C: `roads_this_turn` counter)

### T8: Construct Ships (Navy)
- `commands.rs` has `construct_ship()` — verify vs C
- Add `ConstructShip { nation, x, y, ship_type, size, count }` action
- Must be in coastal city/port sector
- Types: warship, merchant, galley

### T9: Load/Unload Armies onto Fleets
- Port `original/navy.c` `loadfleet()` (line 358)
- Add `LoadArmyOnFleet { nation, army, fleet }` action
- Add `UnloadArmyFromFleet { nation, army, fleet }` action  
- `navy.rs` has `load_army()` and `unload_army()` — wire them
- Also: `LoadPeopleOnFleet`, `UnloadPeople` for civilian transport

### T10: Magic — Spell Casting
- Port `original/magic.c` `domagic()` (line 190) 
- Add `CastSpell { nation, spell_type, target_x, target_y, target_nation }` action
- Spells: summon creatures, flight, attack/defense enhancement, destroy
- Port `dosummon()` (line 504) — summon monster units
- Port `wizardry()` (line 923) — wizard-specific spells  
- Port `god_magk()` (line 830) — god/deity powers
- Validate spell points cost via `getmgkcost()`

### T11: Magic — Buy New Powers
- Port `original/magic.c` `getmagic()` (line 32)
- Add `BuyMagicPower { nation, power }` action
- Logic: costs gold, adds power bit to nation.powers
- npc_buy_magic already exists in engine — make player version
- Validate cost, check prerequisites

### T12: Trade — Propose and Accept
- Port `original/trade.c` `trade()` (line 68)
- Add `ProposeTrade { nation, target_nation, offer_type, offer_amount, request_type, request_amount }` action
- Add `AcceptTrade { nation, trade_id }` action  
- Add `RejectTrade { nation, trade_id }` action
- Store pending trades in GameState or game store
- Types: gold, food, metal, jewels, armies, land, ships

### T13: Diplomacy Changes
- Port `original/forms.c` `diploscrn()` (line 149) — the logic, not the UI
- `AdjustDiplomacy` action exists — verify it enforces rules:
  - Can only change one step at a time per turn
  - Allied → Friendly → Neutral → Hostile → War (or reverse)
  - Some races auto-hostile (orcs vs humans etc)
- Port `original/forms.c` `change()` (line 387) — nation settings changes

### T14: Hire Mercenaries
- Port from C: MSETA/MSETB commands in cexecute.c
- Implement `HireMercenaries` action properly (currently stubbed)
- Logic: pay gold, get troops with world mercenary attack/defense stats
- Limited by available mercenary pool (world.merc_men)

### T15: Disband to Mercenary Pool
- Implement `DisbandToMerc` action properly (currently stubbed)
- Logic: dismiss army, soldiers go back to merc pool
- World merc stats adjust based on disbanded army quality

### T16: Bribery
- Implement `BribeNation` action properly (currently stubbed)
- Port from C cexecute.c XBRIBE logic:
  - Cost gold, chance of improving diplomacy
  - 50% if same NPC type, 30% if neutral, 20% otherwise, +20% same race

### T17: Send Tribute
- `commands.rs` has `send_tribute()` — verify vs C
- Add `SendTribute { nation, target, gold, food, metal, jewels }` action
- Logic: transfer resources to another nation, may improve diplomacy

### T18: Nation Destruction (sackem/destroy)
- Port `original/misc.c` `sackem()` (line 876) and `destroy()` (line 968)
- Verify `DestroyNation` action handles:
  - Freeing all sectors (set owner=0)
  - Removing all armies
  - Distributing remaining resources
  - News announcements

### T19: Movement Validation
- Port `original/misc.c` `land_reachp()` (line 250) — can army reach destination?
- Port `water_reachp()` (line 401) — can navy reach destination?
- `movement.rs` has `land_reachp()` and `move_army_step()` — verify they match C
- Ensure zone of control (`zone_of_control()`) matches C behavior
- Validate move costs per terrain type match C `updmove()`

### T20: Spreadsheet/Reports API  
- `economy.rs` has `spreadsheet()` — verify it matches C `misc.c` `spreadsheet()` (line 1189)
- Port `original/reports.c` key report functions
- Port `original/misc.c` `deplete()` (line 783) — resource depletion
- Port scoring: `score_one()` (line 490) — verify matches Rust
- These are read-only but needed for the web UI to show accurate data

### T21: Cleanup Fixed-Array Functions
- Refactor remaining fixed-array functions to use &GameState:
  - `events.rs` volcano_damage, spread_fire (lines 214, 468, 539)
  - `commands.rs` is_next_to_water (line 329)  
  - `nation.rs` place_nation, count_nation_sectors (lines 40, 240, 354)
- Either port to _gs versions or delete if unused dead code
- Remove MAPX/MAPY constants from constants.rs if no longer needed
