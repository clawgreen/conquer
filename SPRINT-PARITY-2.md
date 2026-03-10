# Sprint: C Parity Fixes (Round 2)

## 9 gaps between Rust and C original. All must match C behavior exactly.

### P1: deplete() — capitol loss depletion
- C: `original/misc.c:783`, called from `updsectors()` at `original/update.c:1004`
- When nation has no capitol (lost/occupied), disband PDEPLETE% of armies, all monsters, scatter people, reduce gold
- Add to `conquer-engine/src/economy.rs` in `updsectors()` per-nation loop

### P2: MEETNTN auto-diplomacy in updsectors
- C: `original/update.c:978-988` — check adjacent sectors within MEETNTN=2 range, call newdip() when nations first meet
- Add to `updsectors()` in the per-sector loop

### P3: ROADS movement penalty
- C: `original/update.c:1240-1243` — in `updmil()`, if nation has ROADS power and army is in enemy territory: smove>7 → -4, else smove>4 → cap at 4
- Add to `updmil()` after movement calculation

### P4: move_people() — proper attractiveness-based migration
- C: `original/update.c:1557-1622` — uses attr[][] grid from attract() function (update.c:135)
- Replace simplified neighbor-diffusion with proper C algorithm
- C attract() at update.c:135 calculates attractiveness per sector

### P5: Group movement (GENERAL)
- C: `original/update.c:1260-1275` — groups move at slowest unit speed +2; GENERAL with no followers → DEFEND
- Add to `updmil()` after individual army movement calc

### P6: Siege validation in updmil
- C: `original/update.c:1155-1200` + `1296-1340` — count attackers vs defenders (siege units 3x), SIEGED status, movement removal
- Add siege validation pass to `updmil()`

### P7: Navy storms
- C: `original/update.c:1278-1310` — PSTORM% chance to sink fleet, crewless ship destruction
- Add to `updmil()` navy loop

### P8: PC leader movement halved
- C: `original/update.c:1250` — `if(ispc && occ[AX][AY]==0) A->smove /= 2`
- PC armies not near a leader get halved movement
- Add to `updmil()` in maintenance section

### P9: attract() for move_people
- C: `original/update.c:135-244` — full sector attractiveness for civilian migration
- Port attract() and use it in move_people()
