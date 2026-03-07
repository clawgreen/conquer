# Conquer — Claude Code Instructions

## Project Structure
- `original/` — Original C source code (1988). READ THIS for reference.
- `conquer-core/` — Shared types, constants, enums, structs
- `conquer-engine/` — Game logic (combat, economy, NPC, events, etc.)
- `conquer-db/` — Data store, game state persistence, turn processing
- `conquer-server/` — HTTP API server (Axum)
- `web/` — TypeScript web frontend

## Current Task
Read and follow `SPRINT-COMMANDS-PARITY.md` — it has 21 tasks for full C→Rust command parity.
(SPRINT-TURN-PIPELINE.md is DONE — do not redo it.)

## Key Files
- `conquer-db/src/store.rs` — `run_turn()` method is where turn pipeline lives (~line 737)
- `conquer-engine/src/npc.rs` — NPC AI (`nation_run()`)
- `conquer-engine/src/monster.rs` — Monster updates
- `conquer-engine/src/events.rs` — Random events
- `conquer-engine/src/trade.rs` — Trade processing
- `conquer-engine/src/economy.rs` — Economy (updsectors, updcomodities, updmil)
- `conquer-engine/src/movement.rs` — Army movement
- `conquer-engine/src/combat.rs` — Combat resolution
- `conquer-core/src/constants.rs` — Game constants
- `conquer-core/src/structs.rs` — GameState, Nation, Sector, etc.
- `conquer-core/src/actions.rs` — Action enum (player commands)

## Critical Rules
1. ALWAYS read the C source (`original/*.c`) before implementing anything
2. All functions must use `&mut GameState` (dynamic `Vec<Vec<>>`), NOT fixed-size arrays
3. Run `cargo test` after each change
4. Commit after each task (T1, T2, etc.)
5. One pre-existing test failure: `store::tests::test_join_game` — ignore it
6. The turn order must match the C original (see T11 in sprint doc)
