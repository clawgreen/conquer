# Conquer Modernization — Master TODO Plan

> **Goal:** Bring the 1988 Conquer game to the web with ZERO gameplay changes.
> Full Rust rewrite — port all C game logic module by module, verified against the original C code used as a test oracle. Postgres for persistence, Canvas 2D frontend preserving the terminal aesthetic, WebSocket for real-time updates, and a side chat panel for players.

## Human Checkpoints 🚦

These are gates where automated development STOPS and Trevor reviews before continuing:

1. **After Phase 0 (Oracle)** — Verify C oracle runs, JSON dumps look sane, seeded RNG produces deterministic output. This is the foundation everything tests against.
2. **After Phase 2 (Game Engine)** — Play test: run a 10-turn game in Rust-only test harness. Does combat feel right? Do NPCs behave? Subtle divergence hides here.
3. **After Phase 4 (Web Frontend MVP)** — Browser play test: actually play in the browser. Does the terminal feel land? Is the UX tolerable?
4. **After Phase 6 (Platform)** — Full flow test: create account → make game → invite friend → play a turn together. End-to-end.

## Dev Loop Instrumentation

Every cron run must produce actionable feedback for the next run:

- **`build-state.json`** — current TODO, phase, pass/fail, blockers, timestamp (cron reads this to know where it is)
- **`build-log.jsonl`** — append-only log of every TODO attempted: `{todo, status, duration, errors, tests_passed, tests_failed}`
- **`cargo test` output** — captured to `logs/test-output-TXXXX.txt` on any failure
- **Oracle diff output** — when Rust vs C state diverges, capture field-level diff to `logs/oracle-diff-TXXXX.json`
- **Error pattern tracking** — `logs/error-patterns.md` — recurring compile/test errors so future runs don't repeat mistakes
- **Snapshot test results** — `logs/snapshot-results.json` — which seeds pass, which fail, which fields diverge

### Build State File (`conquer-build-state.json`)
```json
{
  "current_phase": 1,
  "current_todo": "T037",
  "completed": ["T001", "T002", "..."],
  "failed": [],
  "blocked": [],
  "last_run": "2026-03-04T15:00:00Z",
  "oracle_seeds_passing": [42, 123, 999],
  "oracle_seeds_failing": [],
  "tests_total": 0,
  "tests_passing": 0
}
```

## Bug Replication Policy 🐛

When the Rust port reveals bugs in the original C code (undefined behavior, off-by-one errors, integer overflow, etc.):

1. **Replicate the bug** in Rust — oracle parity takes priority. The game must behave identically.
2. **Document the bug** in `docs/KNOWN-BUGS.md` — what it is, where it lives in C, how we replicated it in Rust, and what the correct behavior should be.
3. **Mark the Rust code** with `// BUG-COMPAT: <description>` comments so we can find and fix them later.
4. **Never let a bug block progress** — if a bug makes oracle testing impossible (e.g., C crashes, memory corruption), document it and skip that specific test scenario. Add a `SKIP` entry to `conquer-build-state.json`.
5. **Phase 8 (future)** — after full parity is achieved, we can optionally fix bugs behind a `--classic` / `--fixed` mode flag.

## Strategy

### Test Oracle Approach
The original C code is the **source of truth**. Before porting anything to Rust, we build the C code into a non-interactive test oracle that dumps complete game state to JSON. We then use **seeded RNG snapshot testing**: given the same seed, the same sequence of actions must produce byte-for-byte identical state in both C and Rust. Every Rust module is verified against C oracle output before moving on.

### Architecture Overview
- **Full Rust rewrite** — no FFI, no C at runtime. Every formula, constant, and mechanic ported to Rust.
- **GameState struct** — single owner of all game state, replacing C globals (`world`, `ntn[]`, `sct[][]`, `occ[][]`, `movecost[][]`).
- **Action queue** — the C execute-file protocol (`fprintf(fexe, ...)`) becomes a `Vec<Action>` with typed enum variants.
- **ADMIN/CONQUER dual-compile** — becomes Rust feature flags or module-level configuration (not separate binaries).
- **Postgres persistence** — all game state, turns, chat stored in Postgres. No flat files.
- **Railway deployment** — Docker + one-click Postgres on Railway.

---

## Phase 0: Build & Oracle
*Get the C code building and running, strip interactive blockers, build a test oracle that dumps game state to JSON for snapshot testing.*

- **T001** — Clone repo, verify `gpl-release/` directory structure contains all source, headers, Makefile
- **T002** — Install build dependencies (`libncurses-dev`, `build-essential`, `pkg-config`) on dev machine (macOS: `brew install ncurses`)
- **T003** — Run `make clean && make all` — capture and document every warning to `docs/warnings-baseline.txt`
- **T004** — Verify all four executables produced: `conquer`, `conqrun`, `conqsort`, `conqps`
- **T005** — Run `make new_game` to generate a world in `$HOME/conquer/`
- **T006** — Launch `conquer`, log in as God, verify map renders and basic commands work
- **T007** — Run `conqrun -m` to create a fresh world, then `conqrun -a` to advance a turn — verify no crashes
- **T008** — Create a test player nation, perform basic actions (move army, redesignate, draft)
- **T009** — Document complete build+run process in `docs/DEV-SETUP.md`

### 0A: Strip Interactive Blockers

- **T010** — Remove `getpass()` calls — replace with env var or CLI arg for password input (`CONQUER_PASS` or `--pass`)
- **T011** — Strip `CHECKUSER` / uid-checking logic — compile with `#undef CHECKUSER` or remove the `#ifdef CHECKUSER` blocks entirely
- **T012** — Remove file locking (`flock()`, `lockf()`, `FILELOCK`) — not needed for oracle mode
- **T013** — Remove `setuid` requirements — strip `chmod 4751` from Makefile install targets
- **T014** — Strip curses dependency from `conqrun` (admin tool) — it should run headless for oracle generation
- **T015** — Strip signal handlers that call `endwin()` in oracle mode
- **T016** — Verify `conqrun -m && conqrun -a` still works after all stripping — world creates and turn advances without any interactive prompts

### 0B: Test Oracle

- **T017** — Create `oracle/` directory in repo for oracle tooling
- **T018** — Write `oracle/dump_state.c` — links against C game objects, calls `readdata()`, dumps complete game state to JSON on stdout:
  - `s_world` struct (all fields)
  - `ntn[0..NTOTAL-1]` (all nation fields including all 50 armies, 10 navies, diplomacy array)
  - `sct[0..MAPX-1][0..MAPY-1]` (all sector fields)
- **T019** — Write `oracle/dump_after_turn.c` — loads world, runs `update()` pipeline, dumps state after turn advance
- **T020** — Add seeded RNG control: patch `srand()` call in `conqrun` to accept `--seed N` CLI arg, so `srand(N)` produces deterministic output
- **T020a** — **CRITICAL**: Replace platform `rand()`/`srand()` in C oracle with a portable LCG implementation (e.g., POSIX formula: `seed = seed * 1103515245 + 12345; return (seed >> 16) & 0x7fff`). This MUST match across macOS and Linux. Document the exact algorithm in `docs/RNG.md`
- **T020b** — Implement the identical LCG in Rust (`conquer-core/src/rng.rs`) with the same formula, same seed, same sequence. Write a cross-validation test: generate 10,000 numbers from both C and Rust with seed 42, verify identical sequence
- **T021** — Write `oracle/run_actions.c` — accepts a JSON file of actions (matching execute-file format), applies them via `execute()`, dumps resulting state
- **T022** — Create `oracle/Makefile` to build oracle tools linking against game `.o` files (headless, no curses)
- **T023** — Generate first snapshot: `oracle/dump_state --seed 42` → `oracle/snapshots/seed42_init.json`
- **T024** — Generate turn-advance snapshot: `oracle/dump_after_turn --seed 42` → `oracle/snapshots/seed42_turn1.json`
- **T025** — Generate 5 snapshots with different seeds (42, 123, 999, 7777, 31337) for world generation
- **T026** — Generate action-replay snapshots: create world, apply known action sequence, dump state — for 3 different scenarios (basic moves, combat, magic)
- **T027** — Write snapshot diffing tool (`oracle/diff_states.py` or similar) — compares two JSON state dumps, reports field-level differences
- **T028** — Document oracle usage and snapshot format in `docs/ORACLE.md`
- **T029** — Add `.gitignore` for build artifacts (`.o`, executables, help files, install dirs)

### 0C: Dev Loop Infrastructure

- **T029a** — Create `conquer-build-state.json` — cron state file with current phase, current TODO, completed list, failed list, blocked list
- **T029b** — Create `logs/` directory with `.gitkeep` — all build/test/oracle output goes here
- **T029c** — Create `scripts/run-todo.sh` — wrapper that captures test output, updates build-state.json, appends to `logs/build-log.jsonl`
- **T029d** — Create `logs/error-patterns.md` — document recurring issues and their fixes so cron agents don't repeat mistakes
- **T029e** — Create `docs/CRON-AGENT.md` — instructions for the cron agent: how to read build-state, pick next TODO, run it, log results, handle failures

### 🚦 CHECKPOINT: Phase 0 Complete — Trevor Reviews Oracle Output

---

## Phase 1: Rust Data Layer
*Port all structs, enums, constants from header.h + data.h to Rust. Design Postgres schema. Verify struct layout matches C oracle output.*

### 1A: Project Setup

- **T030** — Initialize Rust workspace: `cargo init --name conquer` with workspace layout
- **T031** — Create sub-crate `conquer-core` — all game types, constants, enums, state
- **T032** — Create sub-crate `conquer-engine` — all game logic (world gen, economy, combat, etc.)
- **T033** — Create sub-crate `conquer-server` — Axum HTTP/WebSocket server
- **T034** — Create sub-crate `conquer-db` — Postgres schema, queries, migrations (sqlx or diesel)
- **T035** — Create sub-crate `conquer-oracle` — test harness that loads C oracle snapshots and compares against Rust output
- **T036** — Set up `cargo test` infrastructure with snapshot test framework

### 1B: Core Constants

- **T037** — Port all game limit constants from `header.h`: `NTOTAL` (35), `MAXPTS` (65), `MAXARM` (50), `MAXNAVY` (10), `PDEPLETE` (30), `PFINDSCOUT` (50), etc. — as Rust `const` values
- **T038** — Port all taxation constants: `TAXFOOD`, `TAXMETAL`, `TAXGOLD`, `TAXOTHR`, `TAXCITY`, `TAXTOWN`
- **T039** — Port all economic constants: `SHIPMAINT`, `TOMANYPEOPLE`, `ABSMAXPEOPLE`, `MILLSIZE`, `TOMUCHMINED`, `DESFOOD`, `LONGTRIP`
- **T040** — Port all combat constants: `MAXLOSS`, `FINDPERCENT`, `DESCOST`, `FORTCOST`, `STOCKCOST`, `REBUILDCOST`
- **T041** — Port all naval constants: `WARSHPCOST`, `MERSHPCOST`, `GALSHPCOST`, `SHIPCREW`, `SHIPHOLD`, `N_CITYCOST`, `N_WSPD`, `N_GSPD`, `N_MSPD`, `N_BITSIZE`, `N_MASK`
- **T042** — Port all NPC behavior constants: `CITYLIMIT`, `CITYPERCENT`, `MILRATIO`, `MILINCAP`, `MILINCITY`, `NPCTOOFAR`, `BRIBE`, `METALORE`
- **T043** — Port all defense constants: `DEF_BASE`, `FORTSTR`, `TOWNSTR`, `CITYSTR`, `LATESTART`
- **T044** — Port all vision/movement constants: `LANDSEE`, `NAVYSEE`, `ARMYSEE`, `PRTZONE`, `MEETNTN`, `MOVECOST`
- **T045** — Port all random event constants: `RANEVENT`, `PWEATHER`, `PREVOLT`, `PVULCAN`, `PSTORM`
- **T046** — Port all environmental constants: `MONSTER` (45), `NPC` (45), `PMOUNT`, `ORCTAKE`, `TAKEPOINTS`
- **T047** — Port all starting mercenary values: `ST_MMEN`, `ST_MATT`, `ST_MDEF`
- **T048** — Port all magic power cost constants: `BASEMAGIC`, `DWFMAGIC`, `HUMMAGIC`, `ORCMAGIC`, `DWFCIVIL`, `ORCCIVIL`, `HUMCIVIL`, `DWFMILIT`, `ORCMILIT`
- **T049** — Port attractiveness constants for all races (Human, Dwarf, Elf, Orc): `xMNTNATTR`, `xHILLATTR`, `xCLERATTR`, `xCITYATTR`, etc.
- **T050** — Port trade good value thresholds: `GOLDTHRESH`, `TRADEPCT`, `METALPCT`, `JEWELPCT`
- **T051** — Port God market constants: `GODFOOD`, `GODMETAL`, `GODJEWL`, `GODPRICE`

### 1C: Core Enums

- **T052** — Create `Race` enum: `God`, `Orc`, `Elf`, `Dwarf`, `Lizard`, `Human`, `Pirate`, `Savage`, `Nomad`, `Unknown` — with `From<char>` / `Into<char>` matching C char constants (`'-'`, `'O'`, `'E'`, etc.)
- **T053** — Create `Designation` enum: `Town`, `City`, `Mine`, `Farm`, `Devastated`, `GoldMine`, `Fort`, `Ruin`, `Stockade`, `Capitol`, `Special`, `LumberYard`, `Blacksmith`, `Road`, `Mill`, `Granary`, `Church`, `University`, `NoDesig`, `BaseCamp` — with char conversion matching `des[]` array
- **T054** — Create `Vegetation` enum: `Volcano`, `Desert`, `Tundra`, `Barren`, `LtVeg`, `Good`, `Wood`, `Forest`, `Jungle`, `Swamp`, `Ice`, `None` — with char conversion matching `veg[]` array
- **T055** — Create `Altitude` enum: `Water`, `Peak`, `Mountain`, `Hill`, `Clear` — with char conversion matching `ele[]` array
- **T056** — Create `DiplomaticStatus` enum: `Unmet` (0), `Treaty` (1), `Allied` (2), `Friendly` (3), `Neutral` (4), `Hostile` (5), `War` (6), `Jihad` (7) — exact integer values preserved
- **T057** — Create `ArmyStatus` enum: `March` (1), `Scout` (2), `Garrison` (3), `Traded` (4), `Militia` (5), `Flight` (6), `Defend` (7), `MagDef` (8), `Attack` (9), `MagAtt` (10), `General` (11), `Sortie` (12), `Siege` (13), `Sieged` (14), `OnBoard` (15), `Rule` (16) — plus group status (`>= NUMSTATUS` → group id)
- **T058** — Create `UnitType` enum: all 27 unit types (`A_MILITIA` through `A_SCOUT`), plus leader types (`L_KING` through `L_NAZGUL` with `UTYPE` offset), plus monster types (`SPIRIT` through `DRAGON` with `TWOUTYPE` offset) — preserve exact integer values
- **T059** — Create `Season` enum: `Winter` (0), `Spring` (1), `Summer` (2), `Fall` (3) — with `SEASON(turn)` → `turn % 4` logic
- **T060** — Create `NationClass` enum: `NPC` (0), `King` (1), `Emperor` (2), `Wizard` (3), `Priest` (4), `Pirate` (5), `Trader` (6), `Warlord` (7), `Demon` (8), `Dragon` (9), `Shadow` (10)
- **T061** — Create `NationStrategy` enum: all 22 values from `INACTIVE` (0) through `NPC_SAVAGE` (21) — with helper functions `is_pc()`, `is_npc()`, `is_monster()`, `is_active()`, `npc_type()`, `is_good()`, `is_neutral()`, `is_evil()` matching C macros
- **T062** — Create `Direction` enum: `Centered` (0), `North` (1), `NorthEast` (2), `East` (3), `SouthEast` (4), `South` (5), `SouthWest` (6), `West` (7), `NorthWest` (8)
- **T063** — Create `NavalSize` enum: `Light` (0), `Medium` (1), `Heavy` (2) — with bitfield packing/unpacking matching `SHIPS()`, `N_BITSIZE`, `N_MASK` macros
- **T064** — Create `TradeGood` enum: all 62 trade goods (`TG_furs` through `TG_platinum`, plus `TG_none`) — with category boundaries (`END_POPULARITY`, `END_COMMUNICATION`, `END_EATRATE`, `END_SPOILRATE`, `END_KNOWLEDGE`, `END_FARM`, `END_SPELL`, `END_HEALTH`, `END_TERROR`, `END_NORMAL`, `END_MINE`, `END_WEALTH`)
- **T065** — Create `Power` bitmask type: all 31 powers — Military (`WARRIOR` through `AV_MONST`, `MA_MONST`), Civilian (`SLAVER` through `ROADS`), Magical (`THE_VOID` through `SORCERER`) — with `magic(nation, power)` check matching C macro
- **T066** — Create `HighlightMode` enum: `Own` (0), `Army` (1), `None` (2), `YourArmy` (3), `Move` (4), `Good` (5)
- **T067** — Create `DisplayMode` enum: `Vegetation` (1), `Designation` (2), `Contour` (3), `Food` (4), `Nation` (5), `Race` (6), `Move` (7), `Defense` (8), `People` (9), `Gold` (10), `Metal` (11), `Items` (12)
- **T068** — Create `NationPlacement` enum: `Great`, `Fair`, `Random`, `Oops`
- **T069** — Create `MailStatus` enum: `DoneMail` (-3), `NewsMail` (-2), `AbortMail` (-1)

### 1D: Core Structs

- **T070** — Create `World` struct matching `s_world`: `map_x: i16`, `map_y: i16`, `nations: i16`, `other_nations: i16`, `turn: i16`, `merc_mil: i64`, `merc_aplus: i16`, `merc_dplus: i16`, `world_jewels: i64`, `world_gold: i64`, `world_food: i64`, `world_metal: i64`, `world_civ: i64`, `world_mil: i64`, `world_sectors: i64`, `score: i64`
- **T071** — Create `Sector` struct matching `s_sector`: `designation: Designation`, `altitude: Altitude`, `vegetation: Vegetation`, `owner: u8`, `people: i64`, `initial_people: i16`, `jewels: u8`, `fortress: u8`, `metal: u8`, `trade_good: TradeGood`
- **T072** — Create `Army` struct matching C `army`: `unit_type: UnitType`, `x: u8`, `y: u8`, `movement: u8`, `soldiers: i64`, `status: ArmyStatus`
- **T073** — Create `Navy` struct matching C `navy`: `warships: u16`, `merchant: u16`, `galleys: u16`, `x: u8`, `y: u8`, `movement: u8`, `crew: u8`, `people: u8`, `commodity: u8`, `army_num: u8` — with ship count unpacking methods matching `SHIPS()` macro for light/medium/heavy via bitfield
- **T074** — Create `Nation` struct matching `s_nation`: all fields — `name`, `password`, `leader`, `race`, `location`, `mark`, `cap_x`, `cap_y`, `active` (NationStrategy), `max_move`, `repro`, `score`, `treasury_gold`, `jewels`, `total_mil`, `total_civ`, `metals`, `total_food`, `powers` (Power bitmask), `class` (NationClass), `attack_plus`, `defense_plus`, `spell_points`, `total_sectors`, `total_ships`, `inflation`, `charity`, `armies: [Army; MAXARM]`, `navies: [Navy; MAXNAVY]`, `diplomacy: [DiplomaticStatus; NTOTAL]`, `tax_rate`, `prestige`, `popularity`, `power`, `communications`, `wealth`, `eat_rate`, `spoil_rate`, `knowledge`, `farm_ability`, `mine_ability`, `poverty`, `terror`, `reputation`
- **T075** — Create `Spreadsheet` struct matching `sprd_sht`: all revenue and resource tracking fields
- **T076** — Create `GameState` struct — **single owner of all state**: `world: World`, `nations: [Nation; NTOTAL]`, `sectors: Vec<Vec<Sector>>` (MAPX × MAPY), `occupied: Vec<Vec<i8>>`, `move_cost: Vec<Vec<i16>>` — replaces all C globals
- **T077** — Implement `Serialize`/`Deserialize` (serde) on all structs for JSON oracle comparison
- **T078** — Implement `PartialEq` on all structs for direct comparison in tests

### 1E: Data Tables

- **T079** — Port `ele[]`, `elename[]` arrays — altitude chars and names
- **T080** — Port `veg[]`, `vegfood[]`, `vegname[]` arrays — vegetation chars, food values, names
- **T081** — Port `des[]`, `desname[]` arrays — designation chars and names
- **T082** — Port `soldname[]`, `unittype[]`, `shunittype[]` arrays — unit display names
- **T083** — Port `unitmove[]`, `unitattack[]`, `unitdefend[]` arrays — unit stats
- **T084** — Port `unitminsth[]`, `u_enmetal[]`, `u_encost[]`, `unitmaint[]` arrays — unit costs/requirements
- **T085** — Port `Class[]`, `races[]`, `diploname[]` arrays — class names, race names, diplomacy names
- **T086** — Port `tg_value[]`, `tg_name[]`, `tg_stype[]` arrays — trade good values, names, stat types
- **T087** — Port `powers[]`, `pwrname[]` arrays — power bitmasks and names
- **T088** — Port `seasonstr[]`, `directions[]`, `alignment[]` arrays
- **T089** — Write tests: verify all data tables match C values (cross-reference with oracle dump)

### 1F: Action Types

- **T090** — Create `Action` enum with typed variants replacing the execute-file macros — one variant per C macro:
  - `AdjustArmyStat { nation, army, status }` (AADJSTAT / XASTAT)
  - `AdjustArmyMen { nation, army, soldiers, unit_type }` (AADJMEN / XAMEN)
  - `BribeNation { nation, cost, target }` (BRIBENATION / XBRIBE)
  - `MoveArmy { nation, army, x, y }` (AADJLOC / XALOC)
  - `MoveNavy { nation, fleet, x, y }` (NADJLOC / XNLOC)
  - `AdjustNavyMerchant { nation, fleet, merchant }` (NADJMER / XNAMER)
  - `AdjustNavyCrew { nation, fleet, crew, army_num }` (NADJCRW / XNACREW)
  - `ChangeName { nation, name }` (ECHGNAME / XECNAME)
  - `ChangePassword { nation, password }` (ECHGPAS / XECPAS)
  - `AdjustSpellPoints { nation, cost }` (EDECSPL / EDSPL)
  - `DesignateSector { nation, x, y, designation }` (SADJDES / XSADES)
  - `AdjustSectorCiv { nation, people, x, y }` (SADJCIV / XSACIV)
  - `AddSectorCiv { nation, people, x, y }` (SADJCIV3 / XSACIV3)
  - `IncreaseFort { nation, x, y }` (INCFORT / XSIFORT)
  - `AdjustNavyGold { nation, gold }` (XNAGOLD)
  - `AdjustArmyMove { nation, army, movement }` (AADJMOV / XAMOV)
  - `AdjustNavyMove { nation, fleet, movement }` (NADJMOV / XNMOV)
  - `TakeSectorOwnership { nation, x, y }` (SADJOWN / XSAOWN)
  - `AdjustDiplomacy { nation_a, nation_b, status }` (EADJDIP / EDADJ)
  - `AdjustNavyWarships { nation, fleet, warships }` (NADJWAR / XNAWAR)
  - `AdjustNavyGalleys { nation, fleet, galleys }` (NADJGAL / XNAGAL)
  - `AdjustNavyHold { nation, fleet, army_num, people }` (NADJHLD / XNAHOLD)
  - `AdjustPopulation { nation, popularity, terror, reputation }` (NADJNTN2 / NPOP)
  - `AdjustTax { nation, tax_rate, active, charity }` (NADJNTN / NTAX)
  - `IncreaseAttack { nation }` (I_APLUS / INCAPLUS)
  - `IncreaseDefense { nation }` (I_DPLUS / INCDPLUS)
  - `ChangeMagic { nation, powers, new_power }` (CHGMGK / CHG_MGK)
  - `DestroyNation { target, by }` (DESTROY / DESTRY)
  - `HireMercenaries { nation, men }` (AADJMERC / MSETA)
  - `DisbandToMerc { nation, men, attack, defense }` (AADJDISB / MSETB)
- **T091** — Implement `Action::to_execute_line()` — serializes to the C execute-file format (for oracle comparison)
- **T092** — Implement `Action::from_execute_line()` — parses a C execute-file line into an Action variant
- **T093** — Write round-trip tests: parse execute file → Action → serialize → compare original

### 1G: Postgres Schema

- **T094** — Design and create migration: `games` table — `id UUID PK`, `name TEXT`, `seed BIGINT`, `status TEXT` (waiting/active/paused/completed), `created_at TIMESTAMPTZ`, `updated_at TIMESTAMPTZ`, `settings JSONB` (map size, max nations, NPC count, turn timer, etc.)
- **T095** — Create migration: `game_worlds` table — `game_id UUID FK → games`, `turn INT`, `data JSONB` (serialized `World` struct), `created_at TIMESTAMPTZ` — one row per turn for history
- **T096** — Create migration: `game_nations` table — `game_id UUID FK`, `nation_id INT` (0–34), `turn INT`, `data JSONB` (serialized `Nation` struct), `PK (game_id, nation_id, turn)`
- **T097** — Create migration: `game_sectors` table — `game_id UUID FK`, `turn INT`, `sector_data BYTEA` (compressed serialized sector grid) — full grid per turn for rollback
- **T098** — Create migration: `game_actions` table — `id UUID PK`, `game_id UUID FK`, `nation_id INT`, `turn INT`, `action JSONB` (serialized `Action`), `submitted_at TIMESTAMPTZ`, `order INT`
- **T099** — Create migration: `users` table — `id UUID PK`, `username TEXT UNIQUE`, `email TEXT UNIQUE`, `password_hash TEXT`, `display_name TEXT`, `created_at TIMESTAMPTZ`, `is_admin BOOLEAN DEFAULT false`
- **T100** — Create migration: `game_players` table — `game_id UUID FK`, `user_id UUID FK`, `nation_id INT`, `joined_at TIMESTAMPTZ`, `is_done_this_turn BOOLEAN DEFAULT false`, `PK (game_id, user_id)`
- **T101** — Create migration: `chat_messages` table — `id UUID PK`, `game_id UUID FK`, `sender_nation_id INT` (NULL for system), `channel TEXT` (public / `nation_X_Y` for private), `content TEXT`, `created_at TIMESTAMPTZ`
- **T102** — Create migration: `game_invites` table — `id UUID PK`, `game_id UUID FK`, `invite_code TEXT UNIQUE`, `created_by UUID FK → users`, `expires_at TIMESTAMPTZ`, `max_uses INT`, `uses INT DEFAULT 0`
- **T103** — Implement database connection pool setup (sqlx + `PgPool`)
- **T104** — Implement `GameRepository` — CRUD for games, world state, nations, sectors
- **T105** — Implement `UserRepository` — CRUD for users, password verification (argon2)
- **T106** — Implement `ActionRepository` — insert actions, query by game/turn/nation
- **T107** — Implement `ChatRepository` — insert messages, query with pagination
- **T108** — Write integration tests against a test Postgres instance (use `sqlx::test` or testcontainers)

### 1H: Oracle Verification

- **T109** — Write `oracle_loader` module — reads C oracle JSON snapshots into Rust `GameState`
- **T110** — Write comparison tests: load C oracle initial state → verify all fields match Rust struct defaults when populated from same data
- **T111** — Verify all enum conversions round-trip correctly against oracle data
- **T112** — Verify all data table values match between C arrays and Rust arrays

---

## Phase 2: Rust Game Engine
*Port game logic module by module from C to Rust. Each module tested against C oracle snapshots. Seeded RNG: same seed = same results in both C and Rust.*

### 2A: RNG & Core Utilities

- **T113** — Implement deterministic RNG wrapper matching C `random()` / `srandom()` (BSD) or `lrand48()` / `srand48()` (SysV) — same seed must produce identical sequence
- **T114** — Write RNG sequence test: seed with 42, generate 1000 values, compare against C oracle output
- **T115** — Port `is_habitable(x, y)` — checks if sector is habitable land
- **T116** — Port `ONMAP(x, y)` macro — bounds check
- **T117** — Port `land_reachp()`, `land_2reachp()` — land reachability checks
- **T118** — Port `water_reachp()`, `water_2reachp()` — water reachability checks
- **T119** — Port `canbeseen()` — visibility check for sectors
- **T120** — Port `markok()` — army placement validity
- **T121** — Port `solds_in_sector()` — count soldiers in a sector
- **T122** — Port `units_in_sector()` — count unit count in a sector
- **T123** — Port `tofood()` — vegetation food value conversion
- **T124** — Port `flightcost()` — flying movement cost
- **T125** — Port `armymove()` — army movement calculation
- **T126** — Port `cbonus()` — combat bonus calculation
- **T127** — Port `fort_val()` — fortress value calculation
- **T128** — Port `avian()` — check if unit type can fly
- **T129** — Port `unitvalid()` — validate unit type for nation
- **T130** — Port `defaultunit()` — default unit type for race/class
- **T131** — Port `startcost()` — starting cost for nation creation
- **T132** — Port `getclass()` / `doclass()` — class attribute application
- **T133** — Port `num_powers()` — count number of powers a nation has
- **T134** — Write snapshot tests for all utility functions against C oracle

### 2B: World Generation (`makeworl.c`)

- **T135** — Port `createworld()` — main world generation entry point
- **T136** — Port `makeworld()` — terrain generation (altitude, vegetation placement, `PMOUNT` % mountains)
- **T137** — Port `fill_edge()` — water border generation
- **T138** — Port `populate()` — initial population placement
- **T139** — Port `place()` — nation placement logic (Great/Fair/Random)
- **T140** — Port trade good distribution (`TRADEPCT`, `METALPCT`, `JEWELPCT`)
- **T141** — Port NPC/monster nation generation (`MONSTER` sectors, `NPC` sectors)
- **T142** — Port starting mercenary pool initialization (`ST_MMEN`, `ST_MATT`, `ST_MDEF`)
- **T143** — Write snapshot test: seed 42 → generate world → compare every field against C oracle `seed42_init.json`
- **T144** — Repeat for seeds 123, 999, 7777, 31337

### 2C: Economy & Production (`update.c`, `misc.c`)

- **T145** — Port `updsectors()` — sector economic update (food production, mining, population growth)
- **T146** — Port `produce()` — resource production per sector based on designation and vegetation
- **T147** — Port `rawmaterials()` — raw material extraction
- **T148** — Port `updcomodities()` — commodity/trade good effects on nation stats
- **T149** — Port `updmil()` — military upkeep, maintenance costs, `SHIPMAINT`
- **T150** — Port `spreadsheet()` — full economic spreadsheet calculation (the `sprd_sht` struct)
- **T151** — Port `budget()` — budget display data generation
- **T152** — Port taxation logic: `TAXFOOD`, `TAXMETAL`, `TAXGOLD`, `TAXOTHR`, `TAXCITY`, `TAXTOWN` per-unit rates
- **T153** — Port population growth: `P_REPRORATE`, `TOMANYPEOPLE`, `ABSMAXPEOPLE` caps
- **T154** — Port food consumption: `P_EATRATE`, spoilage (`spoilrate`), starvation
- **T155** — Port inflation calculation
- **T156** — Port depletion mechanics: `PDEPLETE` without capitol, mining depletion (`TOMUCHMINED`)
- **T157** — Write snapshot tests: load initial state → run economy update → compare against C oracle

### 2D: Combat (`combat.c`)

- **T158** — Port `combat()` — main combat resolution
- **T159** — Port `fight()` — individual battle resolution between armies
- **T160** — Port `att_setup()` — attacker setup
- **T161** — Port `att_base()` — base attack value calculation
- **T162** — Port `att_bonus()` — attack bonuses (terrain, magic, unit type)
- **T163** — Port `atkattr()` — attacker attributes
- **T164** — Port `defattr()` — defender attributes
- **T165** — Port `MAXLOSS` (60%) cap on casualties per battle
- **T166** — Port `TAKESECTOR` formula — `min(500, max(75, tciv/350))` soldiers needed to capture
- **T167** — Port `takeover()` — sector capture logic
- **T168** — Port `retreat()`, `fdxyretreat()` — retreat mechanics
- **T169** — Port `flee()` — army fleeing logic
- **T170** — Port `navalcbt()` — naval combat resolution
- **T171** — Port `reduce()` — army reduction after combat
- **T172** — Port `sackem()` — sacking captured cities
- **T173** — Port siege mechanics: `SIEGE`, `SIEGED` status, fortress effects (`FORTSTR`, `TOWNSTR`, `CITYSTR`)
- **T174** — Port sortie mechanics: `SORTIE` status — quick attacks from cities
- **T175** — Write snapshot tests: set up known combat scenarios → resolve → compare casualties and state against C oracle

### 2E: NPC AI (`npc.c`)

- **T176** — Port `nationrun()` — main NPC decision loop
- **T177** — Port `n_atpeace()` — peaceful NPC behavior
- **T178** — Port `n_trespass()` — NPC response to trespassing
- **T179** — Port `n_people()` — NPC population management
- **T180** — Port `n_toofar()` — NPC army recall when too far from capitol (`NPCTOOFAR`)
- **T181** — Port `n_unowned()` — NPC expansion into unowned territory
- **T182** — Port `npcredes()` — NPC sector redesignation logic
- **T183** — Port `cheat()` — NPC cheating for competitiveness (`#ifdef CHEAT`)
- **T184** — Port `pceattr()` — NPC peace attributes
- **T185** — Port NPC strategy system: expansionist (0/2/4/6 free sectors) vs isolationist
- **T186** — Port alignment-based NPC behavior (good/neutral/evil)
- **T187** — Write snapshot tests: advance NPC-only game 10 turns → compare all NPC states against C oracle

### 2F: Monster AI (`npc.c` monster functions)

- **T188** — Port `do_pirate()` — pirate nation behavior
- **T189** — Port `do_nomad()` — nomad nation behavior
- **T190** — Port `do_savage()` — savage nation behavior
- **T191** — Port `do_lizard()` — lizard nation behavior
- **T192** — Port `monster()` — monster respawn logic (`#ifdef MORE_MONST`)
- **T193** — Port `peasant_revolt()` — peasant revolt generation
- **T194** — Port `other_revolt()` — other revolt types
- **T195** — Write snapshot tests for monster/revolt behavior

### 2G: Magic System (`magic.c`)

- **T196** — Port `domagic()` — main spell casting entry point
- **T197** — Port `wizardry()` — wizard-specific spells
- **T198** — Port `getmagic()` — magic point calculation
- **T199** — Port `getmgkcost()` — spell cost calculation
- **T200** — Port `removemgk()` — remove magic effects
- **T201** — Port `exenewmgk()` — execute new magic power acquisition
- **T202** — Port all power effects: Military powers (`WARRIOR` through `MA_MONST`), Civilian powers (`SLAVER` through `ROADS`), Magical powers (`THE_VOID` through `SORCERER`)
- **T203** — Port `MAGATT` and `MAGDEF` status effects on combat
- **T204** — Port orc takeover mechanic (`ORCTAKE`, `TAKEPOINTS`)
- **T205** — Write snapshot tests for magic scenarios against C oracle

### 2H: Navy (`navy.c`)

- **T206** — Port naval movement logic — speed calculation from ship types (`N_WSPD`, `N_GSPD`, `N_MSPD`, `N_SIZESPD`)
- **T207** — Port `loadfleet()` — army loading/unloading with `N_CITYCOST` movement penalty
- **T208** — Port ship construction: `WARSHPCOST`, `MERSHPCOST`, `GALSHPCOST` — cost and resource requirements
- **T209** — Port ship bitfield packing: light/medium/heavy stored as 5-bit fields in `u16` warships/merchant/galleys
- **T210** — Port `fltships()`, `fltspeed()`, `flthold()`, `fltghold()`, `fltwhold()`, `fltmhold()` — fleet calculation functions
- **T211** — Port `addwships()`, `addmships()`, `addgships()`, `subwships()`, `submships()`, `subgships()` — ship count modification
- **T212** — Port storm mechanics: `PSTORM` % chance, `LONGTRIP` attrition
- **T213** — Port `SHIPMAINT` maintenance costs
- **T214** — Port crew mechanics: `SHIPCREW` full strength, crew effects on combat
- **T215** — Write snapshot tests for naval operations against C oracle

### 2I: Movement (`move.c`)

- **T216** — Port army movement logic — terrain-based movement costs
- **T217** — Port `movecost[][]` grid calculation
- **T218** — Port `coffmap()` / `offmap()` — edge-of-map handling
- **T219** — Port group movement: `GENERAL` status, groups always attack mode
- **T220** — Port `ONBOARD` status — armies on ships
- **T221** — Port flying movement: `FLIGHT` status, `flightcost()`
- **T222** — Port `combinearmies()` — merge armies in same sector
- **T223** — Port `splitarmy()` — divide army
- **T224** — Port `reducearmy()` — reduce army size
- **T225** — Port `adjarm()` — adjust army properties
- **T226** — Write snapshot tests for movement scenarios

### 2J: Trade (`trade.c`)

- **T227** — Port `trade()` — trade execution between nations
- **T228** — Port `uptrade()` — trade update per turn
- **T229** — Port `checktrade()` — validate trade availability
- **T230** — Port trade good effects on nation stats (popularity, communication, eat_rate, spoil_rate, knowledge, farm_ability, terror)
- **T231** — Port `tg_ok()` — trade good validity check
- **T232** — Write snapshot tests for trade scenarios

### 2K: Diplomacy

- **T233** — Port `newdip()` — diplomacy change logic
- **T234** — Port `getdstatus()` — get diplomatic status display
- **T235** — Port `BREAKJIHAD` cost (200000 gold to break jihad/confederacy)
- **T236** — Port `MEETNTN` distance requirement for diplomatic changes
- **T237** — Port diplomacy cascading effects (allies follow to war, etc.)
- **T238** — Write snapshot tests for diplomacy changes

### 2L: Random Events (`randeven.c`)

- **T239** — Port `randomevent()` — main random event dispatcher
- **T240** — Port `wdisaster()` — weather disaster events
- **T241** — Port `weather()` — weather effects
- **T242** — Port `erupt()` — volcanic eruption (`PVULCAN` % chance)
- **T243** — Port `deplete()` — resource depletion events
- **T244** — Port `DEVASTATE()` macro — sector devastation logic
- **T245** — Write snapshot tests with known RNG seeds → verify same events fire

### 2M: Commands & Actions (`commands.c`, `cexecute.c`)

- **T246** — Port `execute()` — the execute-file parser that applies queued actions to game state
- **T247** — Port `redesignate()` — sector redesignation with `DESCOST`, `DESFOOD` constraints
- **T248** — Port `draft()` — military drafting logic
- **T249** — Port `construct()` — building construction (forts, stockades, ships)
- **T250** — Port `change_status()` — army status changes
- **T251** — Port `change()` — miscellaneous state changes
- **T252** — Port `blowup()` — destruction mechanics
- **T253** — Write snapshot tests for command execution

### 2N: Turn Update Pipeline

- **T254** — Port `update()` — the complete turn update pipeline from `conqrun`
- **T255** — Port `updexecs()` — execute all queued actions
- **T256** — Port `updcapture()` — process sector captures
- **T257** — Port `updmove()` — process all movement
- **T258** — Port `updleader()` — leader updates
- **T259** — Port `moveciv()` — civilian migration between sectors
- **T260** — Port `redomil()` — military recalculation
- **T261** — Port `verify_ntn()`, `verify_sct()`, `verifydata()` — data integrity checks
- **T262** — Port `prep()` — pre-turn preparation
- **T263** — Port `whatcansee()` — visibility recalculation after turn
- **T264** — Port `init_hasseen()`, `mapprep()` — map preparation and fog of war
- **T265** — Port scoring: `score_one()`, `score()`, `showscore()`, `printscore()`
- **T266** — Port news generation — `newspaper()` data, `MAXNEWS` file rotation
- **T267** — Port NPC messaging: `makemess()` (`#ifdef SPEW`) — random NPC messages
- **T268** — **FULL PIPELINE TEST**: seed 42 → create world → simulate 20 turns (all NPC) → compare final state against C oracle running same 20 turns — must be **identical**

### 2O: Nation Creation (`newlogin.c`)

- **T269** — Port `newlogin()` — complete nation creation flow
- **T270** — Port race selection logic and racial attribute bonuses
- **T271** — Port class selection and `doclass()` attribute application
- **T272** — Port `MAXPTS` (65) point-buy system for starting armies/powers
- **T273** — Port nation placement: `GREAT`/`FAIR`/`RANDOM` with `rand_sector()` selection
- **T274** — Port `LASTADD` (5) — last turn players may join without password
- **T275** — Port `LATESTART` — late joiners get bonus points (1 per `LATESTART` turns)
- **T276** — Write snapshot tests for nation creation with fixed seeds

### 🚦 CHECKPOINT: Phase 2 Complete — Trevor Play-Tests Rust Engine (10-turn game, verify combat/NPC/economy feel right)

---

## Phase 3: Rust Server + API
*Axum HTTP + WebSocket server, game lifecycle management, real-time updates.*

### 3A: Server Setup

- **T277** — Set up Axum server scaffold with graceful shutdown
- **T278** — Implement CORS middleware for web frontend
- **T279** — Implement structured JSON logging with `tracing`
- **T280** — Implement request ID middleware for tracing
- **T281** — Implement health check: `GET /api/health` (returns DB status, version, uptime)
- **T282** — Implement static file serving for frontend assets

### 3B: Authentication

- **T283** — Implement user registration: `POST /api/auth/register` — username, email, password → argon2 hash → JWT
- **T284** — Implement user login: `POST /api/auth/login` → JWT token
- **T285** — Implement JWT middleware — extract and validate token on protected routes
- **T286** — Implement admin role check middleware
- **T287** — Implement password reset flow (optional, email-based)

### 3C: Game Management

- **T288** — `POST /api/games` — create new game (settings: map size, max nations, NPC count, turn timer, seed)
- **T289** — `GET /api/games` — list games with status filter (waiting/active/completed)
- **T290** — `GET /api/games/{id}` — game details (turn, season, player list, settings)
- **T291** — `POST /api/games/{id}/join` — join game as new nation (race, class, placement, name, leader, password)
- **T292** — `POST /api/games/{id}/login` — authenticate as existing nation (nation name + password matching C `crypt()` logic, or new argon2)
- **T293** — `DELETE /api/games/{id}` — archive game (admin only)
- **T294** — Implement game lifecycle state machine: `waiting_for_players` → `active` → `paused` → `completed`
- **T295** — Implement configurable turn timer — auto-advance after N hours

### 3D: Game State Endpoints

- **T296** — `GET /api/games/{id}/map` — visible map for authenticated nation (fog of war via `whatcansee()` / `mapprep()`)
- **T297** — `GET /api/games/{id}/nation` — own nation data (full `Nation` struct minus password)
- **T298** — `GET /api/games/{id}/nations` — public info for all known nations (name, race, class, mark, diplomacy — no hidden stats)
- **T299** — `GET /api/games/{id}/armies` — own army list with all fields
- **T300** — `GET /api/games/{id}/navies` — own navy list with ship breakdowns
- **T301** — `GET /api/games/{id}/sector/{x}/{y}` — detailed sector info (if visible)
- **T302** — `GET /api/games/{id}/news` — current turn's news
- **T303** — `GET /api/games/{id}/scores` — scoreboard (respect `NOSCORE` — only God sees full scores)
- **T304** — `GET /api/games/{id}/budget` — spreadsheet/budget data for own nation

### 3E: Action Endpoints

- **T305** — `POST /api/games/{id}/actions` — submit a batch of `Action` variants as JSON array (replaces execute-file writes)
- **T306** — `GET /api/games/{id}/actions` — get own submitted actions for current turn (review before end-turn)
- **T307** — `DELETE /api/games/{id}/actions/{action_id}` — retract an action before turn ends
- **T308** — `POST /api/games/{id}/end-turn` — mark nation as done for this turn
- **T309** — `POST /api/games/{id}/run-turn` — advance game turn (admin or auto-trigger when all done/timeout)
- **T310** — Implement action validation — reject invalid actions (wrong nation, out-of-range, insufficient resources) before queuing
- **T311** — Implement action ordering — process in submission order within each nation, then interleave per original C execution order

### 3F: WebSocket Real-Time

- **T312** — Implement WebSocket upgrade: `GET /api/games/{id}/ws` with JWT auth
- **T313** — Define WebSocket message protocol (JSON):
  - Server → Client: `map_update`, `nation_update`, `army_update`, `news`, `turn_start`, `turn_end`, `player_joined`, `player_done`, `chat_message`, `system_message`
  - Client → Server: `action`, `chat_send`, `ping`
- **T314** — Implement per-game broadcast groups — each game has a connection pool
- **T315** — Broadcast `turn_end` + refreshed state to all connected players when turn advances
- **T316** — Broadcast `player_done` when a nation submits end-turn
- **T317** — Broadcast `player_joined` when a new nation joins
- **T318** — Implement WebSocket heartbeat/ping-pong (30s interval, 60s timeout)
- **T319** — Handle disconnection gracefully — mark player as away, allow seamless reconnect with state resync
- **T320** — Implement nation-scoped events — combat results only sent to involved nations

### 3G: Game Invite System

- **T321** — `POST /api/games/{id}/invites` — create invite code (max uses, expiry)
- **T322** — `GET /api/invites/{code}` — validate invite, return game info
- **T323** — `POST /api/invites/{code}/accept` — join game via invite

### 3H: API Testing

- **T324** — Write API integration tests: create game → join → get map → submit actions → run turn → verify state
- **T325** — Test fog of war: verify player A cannot see player B's hidden sectors/armies
- **T326** — Test concurrent action submission: 10 players submitting simultaneously
- **T327** — Test turn timer: verify auto-advance after timeout
- **T328** — Test reconnection: disconnect WebSocket → reconnect → verify state resync

---

## Phase 4: Web Frontend
*Canvas 2D terminal-style renderer preserving the text-based aesthetic. Keyboard-driven interface.*

### 4A: Project Setup

- **T329** — Initialize TypeScript + Vite project in `frontend/`
- **T330** — Set up project structure: `src/renderer/`, `src/network/`, `src/ui/`, `src/game/`, `src/state/`
- **T331** — Configure dev proxy to route `/api` and `/ws` to Rust server
- **T332** — Set up ESLint + Prettier for code quality

### 4B: Terminal-Style Canvas Renderer

- **T333** — Create `TerminalRenderer` class — Canvas 2D engine rendering a grid of character cells (monospace font, fixed cell size)
- **T334** — Implement character cell model: each cell = character + fg color + bg color + bold/inverse/blink attributes
- **T335** — Implement color palette matching original 8 curses colors (black, red, green, yellow, blue, magenta, cyan, white) + bold variants
- **T336** — Implement cursor rendering: blinking block cursor at current grid position
- **T337** — Implement screen resize handling — recalculate grid dimensions (equivalent to LINES/COLS)
- **T338** — Implement font size picker — allow users to scale the terminal cells
- **T339** — Implement `standout()` equivalent — inverse video mode for highlighted cells

### 4C: Map Display

- **T340** — Create `MapView` — renders the game map using `TerminalRenderer`
- **T341** — Implement sector rendering: designation characters, vegetation, contour — matching original `see()` function output exactly
- **T342** — Implement all 6 highlight modes matching `HI_OWN`, `HI_ARMY`, `HI_NONE`, `HI_YARM`, `HI_MOVE`, `HI_GOOD`
- **T343** — Implement all 12 display modes matching `DI_VEGE`, `DI_DESI`, `DI_CONT`, `DI_FOOD`, `DI_NATI`, `DI_RACE`, `DI_MOVE`, `DI_DEFE`, `DI_PEOP`, `DI_GOLD`, `DI_METAL`, `DI_ITEMS`
- **T344** — Implement fog of war: unvisited sectors render as blank/dark
- **T345** — Implement army markers on map (nation marks at army positions)
- **T346** — Implement navy markers on map
- **T347** — Implement `SCREEN_X_SIZE` and `SCREEN_Y_SIZE` calculation for viewport
- **T348** — Implement map scrolling with offset tracking (`xoffset`, `yoffset`)

### 4D: Side Panels

- **T349** — Create right-side info panel matching `makeside()`: nation info (name, race, class, leader, treasury, military, civilians, etc.)
- **T350** — Implement army list in side panel: scrollable with `selector` and `pager` (matching C variables), showing status/position/strength
- **T351** — Implement navy list in side panel: fleet composition, position, cargo
- **T352** — Create bottom panel matching `makebottom()`: sector detail when cursor is on a sector
- **T353** — Implement command prompt area for text input in bottom panel

### 4E: Keyboard Input & Commands

- **T354** — Create `InputHandler` — captures keyboard events, maps to game commands
- **T355** — Implement arrow key / hjkl map navigation (cursor movement with `xcurs`/`ycurs`)
- **T356** — Implement army selection: `selector` cycling, army/navy toggle (`AORN`)
- **T357** — Implement army movement commands (directional movement, path following)
- **T358** — Implement army status change commands (all 16+ statuses)
- **T359** — Implement sector redesignation flow — select sector, choose new designation
- **T360** — Implement draft command flow — choose unit type, quantity
- **T361** — Implement diplomacy screen — `diploscrn()` equivalent with nation list and status changes
- **T362** — Implement magic/spell screen — `domagic()` equivalent with power list and casting
- **T363** — Implement navy commands — fleet creation, movement, loading/unloading, construction
- **T364** — Implement budget/spreadsheet view — `spreadsheet()` / `budget()` display
- **T365** — Implement newspaper view — news display
- **T366** — Implement score view — scoreboard (respecting `NOSCORE`)
- **T367** — Implement help screens — render help0 through help5 content
- **T368** — Implement extended command mode: ESC prefix matching `ext_cmd()` / `EXT_CMD` ('\033')
- **T369** — Implement display mode toggle keys: `d`(designation), `r`(race), `M`(move), `p`(people), `D`(defense), `f`(food), `c`(contour), `v`(vegetation), `m`(metal), `n`(nation), `j`(jewels/gold), `i`(items)
- **T370** — Implement highlight mode toggle keys: `o`(own), `a`(army), `y`(your army), `l`(move range), `s`(special/good), `x`(none)
- **T371** — Implement `centermap()` — center map on cursor position or capitol
- **T372** — Implement `jump_to()` — jump cursor to specific coordinates

### 4F: Network Integration

- **T373** — Create `GameClient` class — REST API client + WebSocket connection manager
- **T374** — Implement login/register/join flow in UI
- **T375** — Implement map data fetching and local state cache
- **T376** — Implement action submission — send `Action` JSON to REST, receive confirmation
- **T377** — Implement real-time updates — process WebSocket events, update local state
- **T378** — Implement turn transition — show notification, refresh all state, update display
- **T379** — Implement connection loss detection and automatic reconnection with state resync
- **T380** — Implement end-turn button/command and "waiting for other players" indicator

### 4G: Frontend Polish

- **T381** — Implement game lobby screen — list games (waiting/active/completed), create game, join via invite
- **T382** — Implement nation creation flow — race/class/name/leader selection matching `newlogin()` point-buy system
- **T383** — Implement notification toasts for game events (turn advanced, under attack, army destroyed, etc.)
- **T384** — Implement trade interface matching `trade()` UI
- **T385** — Implement mercenary hiring interface
- **T386** — Implement God mode commands (for admin nations)
- **T387** — Add optional sound effects: beep on error (matching `#ifdef BEEP`), chime on turn advance

### 🚦 CHECKPOINT: Phase 4 Complete — Trevor Browser Play-Tests (does the terminal feel land? UX tolerable?)

---

## Phase 5: Player Chat
*WebSocket chat alongside the game. Per-game rooms, diplomatic private channels, game-aware system messages.*

### 5A: Chat Backend

- **T388** — Implement chat message handling in WebSocket — `chat_send` from client, `chat_message` broadcast to recipients
- **T389** — Implement public game chat channel — all players in a game see messages
- **T390** — Implement private nation-to-nation channels — only two nations can see messages (diplomatic channel)
- **T391** — Implement chat persistence — store in `chat_messages` Postgres table
- **T392** — Implement chat history endpoint: `GET /api/games/{id}/chat?channel=public&before=<timestamp>&limit=50`
- **T393** — Implement chat rate limiting (max 5 messages/10 seconds per player)

### 5B: Game-Aware System Messages

- **T394** — Generate system messages on turn advance: "Turn X (Season, Year Y) has begun"
- **T395** — Generate system messages on nation join: "The nation of [Name] ([Race] [Class]) has entered the world"
- **T396** — Generate system messages on nation destruction: "The nation of [Name] has fallen"
- **T397** — Generate system messages on diplomacy changes: "[Nation A] has declared [status] on [Nation B]" (public declarations only)
- **T398** — Generate system messages from NPC `makemess()` / SPEW content — port the random NPC messages to chat
- **T399** — Generate system messages for random events: volcano eruptions, storms, revolts (public knowledge)

### 5C: Chat Frontend

- **T400** — Create `ChatPanel` component — collapsible side panel (right of game or toggleable drawer)
- **T401** — Implement message list with auto-scroll and infinite scroll history loading
- **T402** — Implement chat input field — send on Enter, multi-line with Shift+Enter
- **T403** — Implement channel switcher: Public / Private (dropdown to select target nation from known nations)
- **T404** — Implement unread message badge per channel
- **T405** — Implement player presence indicators (online/offline based on WebSocket connection)
- **T406** — Style chat to match terminal aesthetic — monospace font, dark background, colored nation names using their mark color
- **T407** — Implement system message styling — distinct from player messages (italics, different color)
- **T408** — Implement `/` slash commands in chat: `/who` (list players), `/diplo` (show diplomacy), `/score`, `/help`

---

## Phase 6: Platform
*User accounts, game creation wizard, invitations, admin dashboards, spectator mode. Self-service game management.*

### 6A: User Management

- **T409** — Implement user profile page: display name, email, game history, stats
- **T410** — Implement user settings: change password, display name, notification preferences
- **T411** — Implement user game history: list of all games played with nation name, final score, outcome

### 6B: Game Creation Wizard

- **T412** — Implement multi-step game creation form: name → map settings → nation limits → turn timer → NPC config → review → create
- **T413** — Map settings: map size (small/medium/large/custom), mountain percentage (`PMOUNT`), vegetation distribution
- **T414** — Nation limits: max player nations, NPC nations count, monster nations count
- **T415** — Turn timer settings: hours per turn, auto-advance on/off, grace period
- **T416** — NPC configuration: enable/disable `CHEAT`, `NPC_SEE_CITIES`, monster respawn (`MORE_MONST`), `SPEW` messages
- **T417** — Advanced settings: `TRADE` on/off, `RANEVENT` probability, `STORMS`, `VULCANIZE`, starting gold
- **T418** — Game seed option: random or specific seed (for reproducible games)

### 6C: Invitation System

- **T419** — Implement invite link generation — shareable URL with invite code
- **T420** — Implement invite management page — list active invites, revoke, set expiry
- **T421** — Implement invite landing page — show game info, join button, nation creation flow
- **T422** — Implement game browser — public games list with filters (status, player count, open slots)

### 6D: Admin Dashboard

- **T423** — Implement admin game management — view all games, pause/resume/archive, force turn advance
- **T424** — Implement admin player management — view all users, ban/unban, reset passwords
- **T425** — Implement admin nation management — view any nation state (God mode), force actions
- **T426** — Implement turn rollback — restore game to previous turn from `game_worlds`/`game_nations` history
- **T427** — Implement server status dashboard — active games, connected players, resource usage, DB stats

### 6E: Spectator Mode

- **T428** — Implement spectator join — view game without playing (public info only, fog of war applies globally)
- **T429** — Implement spectator WebSocket — receive turn updates and public events
- **T430** — Implement spectator map view — show what any player could see (selectable perspective)
- **T431** — Implement spectator chat — read-only public chat, or separate spectator channel

### 6F: Notifications

- **T432** — Implement in-app notification system — turn advanced, your turn, game invite, under attack
- **T433** — Implement email notifications (optional) — turn reminders, game invites
- **T434** — Implement notification preferences — per-event toggle, email on/off

### 🚦 CHECKPOINT: Phase 6 Complete — Trevor Full Flow Test (create account → make game → invite friend → play turn together)

---

## Phase 7: Deploy & CI
*Docker, Railway deployment, Postgres setup, GitHub Actions, monitoring.*

### 7A: Docker

- **T435** — Create multi-stage Dockerfile: build Rust server → build frontend → combine into runtime image
- **T436** — Create `docker-compose.yml` for local development: server + Postgres + optional pgAdmin
- **T437** — Create `docker-compose.prod.yml` with Nginx reverse proxy, TLS termination
- **T438** — Implement environment variable configuration: `DATABASE_URL`, `JWT_SECRET`, `PORT`, `LOG_LEVEL`, `TURN_TIMER_HOURS`, `CORS_ORIGIN`
- **T439** — Create health check in Docker: `/api/health` with DB connectivity check

### 7B: Railway Deployment

- **T440** — Create `railway.toml` configuration
- **T441** — Set up Railway Postgres instance (one-click provisioning)
- **T442** — Configure Railway environment variables (DATABASE_URL auto-injected, JWT_SECRET, CORS_ORIGIN)
- **T443** — Set up Railway custom domain (if available)
- **T444** — Implement database migration on deploy — `sqlx migrate run` on startup
- **T445** — Test deployment: push to Railway → verify game creation → play test game end-to-end

### 7C: CI/CD (GitHub Actions)

- **T446** — Set up CI workflow: on push/PR → `cargo fmt --check` → `cargo clippy` → `cargo test`
- **T447** — Add oracle comparison tests to CI — build C oracle, run snapshot comparisons
- **T448** — Add frontend CI: lint → type-check → build
- **T449** — Add integration test step: spin up test Postgres, run API tests
- **T450** — Set up CD workflow: on merge to main → build Docker image → deploy to Railway
- **T451** — Add dependency audit: `cargo audit` for security vulnerabilities

### 7D: Monitoring & Reliability

- **T452** — Implement structured logging with correlation IDs (game_id, nation_id, user_id in every log line)
- **T453** — Implement metrics endpoint: active games, connected players, actions per minute, turn durations
- **T454** — Implement automatic game state backups — Postgres point-in-time recovery via Railway
- **T455** — Implement graceful server shutdown — save WebSocket state, complete in-flight requests, close DB pool
- **T456** — Implement rate limiting on all API endpoints (per-user, per-IP)

### 7E: Final Integration Testing

- **T457** — End-to-end test: create game → 3 players join via invite → each takes 5 turns of actions → verify all see correct state
- **T458** — Test concurrent games: run 5 games simultaneously with overlapping players
- **T459** — Test persistence: restart server mid-game → verify all state preserved from Postgres
- **T460** — Test chat during gameplay: verify messages arrive in real-time, history loads correctly, system messages fire
- **T461** — Browser compatibility test: Chrome, Firefox, Safari (Canvas 2D should work everywhere)
- **T462** — Performance test: 35-nation game (max NTOTAL) with full map — verify acceptable render and API response times
- **T463** — Test spectator mode during active game
- **T464** — Test turn rollback and resume
- **T465** — Security test: verify fog of war cannot be bypassed via API, nation passwords are not leaked, rate limits work

---

## Summary

| Phase | Description | TODOs | Range |
|-------|-------------|-------|-------|
| 0 | Build & Oracle | 36 | T001–T029e |
| 1 | Rust Data Layer | 83 | T030–T112 |
| 2 | Rust Game Engine | 164 | T113–T276 |
| 3 | Rust Server + API | 52 | T277–T328 |
| 4 | Web Frontend | 59 | T329–T387 |
| 5 | Player Chat | 21 | T388–T408 |
| 6 | Platform | 26 | T409–T434 |
| 7 | Deploy & CI | 31 | T435–T465 |
| **Total** | | **472** | |

---

## Key Principles

1. **ZERO gameplay changes** — every mechanic, constant, formula, and balance parameter stays identical to the 1988 C code
2. **The C code is the test oracle** — build it, seed it, dump state, and verify Rust output matches byte-for-byte
3. **Full Rust rewrite** — no C at runtime, no FFI in production; C is only used for oracle generation during testing
4. **Seeded RNG equivalence** — same seed in C and Rust must produce identical game state after any sequence of operations
5. **GameState owns everything** — single struct replaces all C globals; no mutable statics, no hidden state
6. **Action queue replaces execute-files** — `Vec<Action>` with typed enum variants, not `fprintf(fexe, ...)` string formatting
7. **ADMIN/CONQUER via feature flags** — Rust modules with `cfg` attributes, not separate binaries with `#ifdef` compilation
8. **Postgres for persistence** — all state in the database, no flat files, full turn history for rollback
9. **Terminal aesthetic preserved** — Canvas 2D renderer that looks and feels like a curses terminal, not a modern game UI
10. **Each TODO is atomic** — one task, testable, completable in a sprint; phases are sequential but TODOs within a phase can parallelize where dependencies allow
