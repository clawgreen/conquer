// conquer-engine/src/worldgen.rs — World generation ported from makeworl.c
//
// T135-T144: createworld, rawmaterials, populate, fill_edge, etc.
// Same seed = identical map.

use conquer_core::*;
use conquer_core::rng::ConquerRng;
use conquer_core::powers::Power;
use conquer_core::tables::*;
use crate::utils::*;

const HALF: u8 = 2;
const LAND: u8 = 3;

/// Initialize all nations to default state — matches C zeroworld().
pub fn zeroworld(state: &mut GameState) {
    for i in 0..NTOTAL {
        let ntn = &mut state.nations[i];
        for army in &mut ntn.armies {
            army.soldiers = 0;
            army.x = 0;
            army.y = 0;
            army.unit_type = 0;
            army.movement = 0;
            army.status = ArmyStatus::Defend.to_value();
        }
        for navy in &mut ntn.navies {
            navy.warships = 0;
            navy.merchant = 0;
            navy.galleys = 0;
            navy.crew = 0;
            navy.people = 0;
            navy.army_num = 0;
            navy.x = 0;
            navy.y = 0;
            navy.movement = 0;
        }
        ntn.active = NationStrategy::Inactive as u8;
        ntn.repro = 0;
        ntn.jewels = 0;
        ntn.treasury_gold = 0;
        ntn.metals = 0;
        ntn.powers = 0;
        ntn.total_civ = 0;
        ntn.total_mil = 0;
        ntn.score = 0;
        ntn.race = '?';
        ntn.max_move = 0;
        ntn.spell_points = 0;
        ntn.class = 0;
        ntn.attack_plus = 0;
        ntn.defense_plus = 0;
        ntn.inflation = 0;
        ntn.total_sectors = 0;
        ntn.total_ships = 0;
    }
}

/// Full world generation entry point — matches C createworld() + rawmaterials() + populate().
pub fn create_world(state: &mut GameState, rng: &mut ConquerRng, pwater: i32) {
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;
    let maxx = map_x / 8; // MAXX
    let maxy = map_y / 8; // MAXY
    let numareas = maxx * maxy;
    let numsects = map_x * map_y;

    // Allocate temp arrays
    let mut tplace = vec![vec![0u8; maxy]; maxx];
    let mut area_map = vec![vec![0u8; maxy]; maxx];
    let mut type_map = vec![vec![0u8; map_y]; map_x];

    // Initialize vegetation to NONE
    for x in 0..map_x {
        for y in 0..map_y {
            state.sectors[x][y].vegetation = Vegetation::None as u8;
        }
    }

    // -- Step 1: Determine area types --
    let avvalue: f32 = (100 - pwater) as f32 / 25.0;
    let mut number = [0i32; 5];
    for i in 0..5 {
        number[i] = (numareas / 5) as i32;
    }
    number[2] = numareas as i32 - 4 * number[0]; // correct for roundoff

    let mut alloc = (numareas * 2) as i32;

    // Balance area type distribution
    for _ in 0..250 {
        if (avvalue * numareas as f32) > alloc as f32 {
            let x = (rng.rand() % 4) as usize;
            if number[x] > 0 {
                number[x] -= 1;
                number[x + 1] += 1;
                alloc += 1;
            }
        } else {
            let x = (rng.rand() % 4 + 1) as usize;
            if number[x] > 0 {
                number[x] -= 1;
                number[x - 1] += 1;
                alloc -= 1;
            }
        }
    }

    // -- Step 2: Place type-4 (full land) sectors --
    let mut i = 0;
    while number[4] > 0 && i < 500 {
        i += 1;
        let ax = (rng.rand() % (maxx as i32 - 2) + 1) as usize;
        let ay = (rng.rand() % (maxy as i32 - 2) + 1) as usize;
        if tplace[ax][ay] == 0 {
            tplace[ax][ay] = 1;
            area_map[ax][ay] = 4;
            number[4] -= 1;
            // Place surrounding sectors
            for &(dx, dy) in &[(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let nx = (ax as i32 + dx) as usize;
                let ny = (ay as i32 + dy) as usize;
                if nx < maxx && ny < maxy && tplace[nx][ny] == 0 {
                    let rnd = rng.rand() % 100 + 1;
                    if rnd < 25 && number[4] > 0 {
                        area_map[nx][ny] = 4;
                        number[4] -= 1;
                        tplace[nx][ny] = 1;
                    }
                    // BUG-COMPAT: C uses `if(rnd>25)` not `else if`, so both can fire
                    // but since tplace is set, only first matters in practice
                    if rnd > 25 && number[3] > 0 && tplace[nx][ny] == 0 {
                        area_map[nx][ny] = 3;
                        number[3] -= 1;
                        tplace[nx][ny] = 1;
                    }
                }
            }
        }
    }

    // Place all other areas
    for ax in 0..maxx {
        for ay in 0..maxy {
            while tplace[ax][ay] == 0 {
                let rnd = (rng.rand() % 5) as usize;
                if number[rnd] > 0 {
                    area_map[ax][ay] = rnd as u8;
                    number[rnd] -= 1;
                    tplace[ax][ay] = 1;
                }
            }
        }
    }

    // -- Step 3: Fill edges and centers --
    for ay in 0..maxy {
        for ax in 0..maxx {
            fill_edge(ax, ay, maxx, maxy, &area_map, &mut type_map, rng);
            // Fill center (1..7)
            for ci in 1..7usize {
                for cj in 1..7usize {
                    let tx = ax * 8 + ci;
                    let ty = ay * 8 + cj;
                    match area_map[ax][ay] {
                        0 => {
                            if rng.rand() % 100 < 95 {
                                type_map[tx][ty] = Altitude::Water as u8;
                            } else {
                                type_map[tx][ty] = HALF;
                            }
                        }
                        1 => {
                            if rng.rand() % 2 == 0 {
                                type_map[tx][ty] = Altitude::Water as u8;
                            } else {
                                type_map[tx][ty] = HALF;
                            }
                        }
                        2 => {
                            if rng.rand() % 2 == 0 {
                                type_map[tx][ty] = Altitude::Water as u8;
                            } else {
                                type_map[tx][ty] = LAND;
                            }
                        }
                        3 => {
                            if rng.rand() % 2 == 0 {
                                type_map[tx][ty] = LAND;
                            } else {
                                type_map[tx][ty] = HALF;
                            }
                        }
                        4 => {
                            if rng.rand() % 100 < 95 {
                                type_map[tx][ty] = LAND;
                            } else {
                                type_map[tx][ty] = HALF;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // -- Step 4: Resolve HALF tiles --
    for x in 0..map_x {
        for y in 0..map_y {
            if type_map[x][y] == HALF {
                if rng.rand() % 100 >= (100 - pwater) {
                    type_map[x][y] = LAND;
                } else {
                    type_map[x][y] = Altitude::Water as u8;
                }
            }
        }
    }

    // -- Step 5: Smooth the world --
    // BUG-COMPAT: C iterates 1..MAPX-1, 1..MAPY-1
    for x in 1..(map_x - 1) {
        for y in 1..(map_y - 1) {
            let mut chance = 0;
            for i in (x - 1)..=(x + 1) {
                for j in (y - 1)..=(y + 1) {
                    if type_map[i][j] == LAND {
                        chance += 1;
                    }
                }
            }
            if rng.rand() % 9 < chance {
                type_map[x][y] = LAND;
            } else {
                type_map[x][y] = Altitude::Water as u8;
            }
        }
    }

    // -- Step 6: Set altitudes --
    for i in 0..map_x {
        for j in 0..map_y {
            if type_map[i][j] == LAND {
                state.sectors[i][j].altitude = Altitude::Clear as u8;
            } else {
                state.sectors[i][j].altitude = Altitude::Water as u8;
            }
        }
    }

    // -- Step 7: Place mountains --
    let avmount: f32 = (PMOUNT as f32 * (100 - pwater) as f32) / 10000.0;
    let mut nmountains: i64 = (numsects as f32 * avmount) as i64;
    let one_third = (numsects as f32 * avmount / 3.0) as i64;

    // Place mountain ranges
    while nmountains > 5 && nmountains > one_third {
        let x1 = (rng.rand() % (map_x as i32 - 8)) as usize;
        let y1 = (rng.rand() % (map_y as i32 - 8)) as usize;
        let x2 = (rng.rand() % 8) as usize + x1;
        let y2 = (rng.rand() % 8) as usize + y1;

        // Validate: check for water nearby
        let mut valid = false;
        for x in x1..=x2 {
            let y_center = if x1 < x2 {
                ((y2 as i32 - y1 as i32) * (x as i32 - x1 as i32) / (x2 as i32 - x1 as i32)) + y1 as i32
            } else {
                y1 as i32
            };
            for j in (y_center - 2)..=(y_center + 2) {
                if j >= 0 && (j as usize) < map_y && x < map_x {
                    if type_map[x][j as usize] != LAND && rng.rand() % 2 == 0 {
                        valid = true;
                        break;
                    }
                }
            }
            if valid { break; }
        }
        if valid { continue; }

        // Fill mountain range
        for x in x1..=x2 {
            if x >= map_x { break; }
            let y_center = if x1 < x2 {
                ((y2 as i32 - y1 as i32) * (x as i32 - x1 as i32) / (x2 as i32 - x1 as i32)) + y1 as i32
            } else {
                y1 as i32
            };
            let yc = y_center as usize;

            if yc < map_y && type_map[x][yc] == LAND {
                if rng.rand() % 100 > 80 {
                    if nmountains > 0 { state.sectors[x][yc].altitude = Altitude::Peak as u8; nmountains -= 1; }
                } else {
                    if nmountains > 0 { state.sectors[x][yc].altitude = Altitude::Mountain as u8; nmountains -= 1; }
                }
            }

            // y+1
            if yc + 1 < map_y && type_map[x][yc + 1] == LAND {
                let rnd = rng.rand() % 100 + 1;
                if rnd > 90 { if nmountains > 0 { state.sectors[x][yc + 1].altitude = Altitude::Peak as u8; nmountains -= 1; } }
                else if rnd > 50 { if nmountains > 0 { state.sectors[x][yc + 1].altitude = Altitude::Mountain as u8; nmountains -= 1; } }
                else if rnd > 20 { if nmountains > 0 { state.sectors[x][yc + 1].altitude = Altitude::Hill as u8; nmountains -= 1; } }
            }
            // y-1
            if yc >= 1 && type_map[x][yc - 1] == LAND {
                let rnd = rng.rand() % 100 + 1;
                if rnd > 90 { if nmountains > 0 { state.sectors[x][yc - 1].altitude = Altitude::Peak as u8; nmountains -= 1; } }
                else if rnd > 50 { if nmountains > 0 { state.sectors[x][yc - 1].altitude = Altitude::Mountain as u8; nmountains -= 1; } }
                else if rnd > 20 { if nmountains > 0 { state.sectors[x][yc - 1].altitude = Altitude::Hill as u8; nmountains -= 1; } }
            }
            // y-2
            if yc >= 2 && type_map[x][yc - 2] == LAND {
                let rnd = rng.rand() % 100 + 1;
                if rnd > 90 { if nmountains > 0 { state.sectors[x][yc - 2].altitude = Altitude::Mountain as u8; nmountains -= 1; } }
                else if rnd > 50 { if nmountains > 0 { state.sectors[x][yc - 2].altitude = Altitude::Hill as u8; nmountains -= 1; } }
            }
            // y+2
            if yc + 2 < map_y && type_map[x][yc + 2] == LAND {
                let rnd = rng.rand() % 100 + 1;
                if rnd > 90 { if nmountains > 0 { state.sectors[x][yc + 2].altitude = Altitude::Mountain as u8; nmountains -= 1; } }
                else if rnd > 50 { if nmountains > 0 { state.sectors[x][yc + 2].altitude = Altitude::Hill as u8; nmountains -= 1; } }
            }
        }
    }

    // Fill random hills
    while nmountains > 0 {
        let x = (rng.rand() % (map_x as i32 - 1)) as usize;
        let y = (rng.rand() % (map_y as i32 - 1)) as usize;
        if type_map[x][y] == LAND && type_map[x + 1][y] == LAND {
            state.sectors[x][y].altitude = Altitude::Hill as u8;
            nmountains -= 1;
        }
    }

    // Ensure no peak/mountain adjacent to water
    for y in 1..(map_y - 1) {
        for x in 1..(map_x - 1) {
            let alt = state.sectors[x][y].altitude;
            if alt == Altitude::Peak as u8 || alt == Altitude::Mountain as u8 {
                'outer: for di in 0..=2 {
                    for dj in 0..=2 {
                        if state.sectors[x + di - 1][y + dj - 1].altitude == Altitude::Water as u8 {
                            state.sectors[x][y].altitude = Altitude::Hill as u8;
                            break 'outer;
                        }
                    }
                }
            }
        }
    }

    // -- Step 8: Vegetation --
    let veg_chars = VEG_CHARS.as_bytes();
    for x in 0..map_x {
        for y in 0..map_y {
            if type_map[x][y] == LAND {
                // Random vegetation: veg[3 + rand()%5] = one of BARREN/LT_VEG/GOOD/WOOD/FOREST
                let idx = 3 + (rng.rand() % 5) as usize;
                state.sectors[x][y].vegetation = veg_chars[idx];

                let alt = state.sectors[x][y].altitude;
                if alt == Altitude::Hill as u8 {
                    // Decrement vegetation
                    for n in 3..9 {
                        if state.sectors[x][y].vegetation == veg_chars[n] {
                            state.sectors[x][y].vegetation = veg_chars[n - 1];
                            break;
                        }
                    }
                    // If area_map <= 1 (watery area), set to volcano
                    if area_map[x / 8][y / 8] <= 1 {
                        state.sectors[x][y].vegetation = Vegetation::Volcano as u8;
                    }
                } else if alt == Altitude::Mountain as u8 {
                    if rng.rand() % 6 == 4
                        && (y > map_y / 2 + 8 || y + 8 < map_y / 2)
                    {
                        state.sectors[x][y].vegetation = Vegetation::Ice as u8;
                    } else {
                        let idx = 2 + (rng.rand() % 3) as usize;
                        state.sectors[x][y].vegetation = veg_chars[idx];
                    }
                } else if alt == Altitude::Peak as u8 {
                    if rng.rand() % 3 == 0
                        && (y > map_y / 2 + 8 || y + 8 < map_y / 2)
                    {
                        state.sectors[x][y].vegetation = Vegetation::Ice as u8;
                    } else {
                        state.sectors[x][y].vegetation = Vegetation::Volcano as u8;
                    }
                }
            }
        }
    }

    // NOTE: In the C code, vegetation values are stored as char codes (e.g., 'b', 'l', 'g').
    // Our enums use u8 indices. The veg_chars assignment above stores char codes.
    // We need to convert back to enum indices for our Rust structs.
    // Actually, looking at the C code more carefully, sct[x][y].vegetation stores the CHAR value
    // from the veg[] array. But in our Rust struct, we store enum indices (0-11).
    // Let's fix: convert veg char back to index.
    for x in 0..map_x {
        for y in 0..map_y {
            let v = state.sectors[x][y].vegetation;
            // If it's already an index (0-11), leave it; if it's a char code, convert
            if v > 11 {
                // It's a char code — convert to index
                if let Some(veg) = Vegetation::from_char(v as char) {
                    state.sectors[x][y].vegetation = veg as u8;
                }
            }
        }
    }

    // Polar work: first 6 and last 7 rows
    for x in 0..map_x {
        for y in 0..6 {
            if type_map[x][y] == LAND {
                if rng.rand() % 4 == 0 {
                    state.sectors[x][y].vegetation = Vegetation::Ice as u8;
                } else {
                    // Decrement vegetation
                    let cur = state.sectors[x][y].vegetation;
                    for n in 3..10usize {
                        if n < 12 && cur == n as u8 {
                            state.sectors[x][y].vegetation = (n - 1) as u8;
                            break;
                        }
                    }
                }
            }
        }
        for y in (map_y.saturating_sub(7))..map_y {
            if type_map[x][y] == LAND {
                if rng.rand() % 4 == 0 {
                    state.sectors[x][y].vegetation = Vegetation::Ice as u8;
                } else {
                    let cur = state.sectors[x][y].vegetation;
                    for n in 3..10usize {
                        if n < 12 && cur == n as u8 {
                            state.sectors[x][y].vegetation = (n - 1) as u8;
                            break;
                        }
                    }
                }
            }
        }
    }

    // Equator: MAPY/2 +/- 8
    let equator = map_y / 2;
    for y in equator.saturating_sub(8)..=(equator + 8).min(map_y - 1) {
        for x in 0..map_x {
            if type_map[x][y] == LAND {
                if rng.rand() % 10 == 0 {
                    state.sectors[x][y].vegetation = Vegetation::Desert as u8;
                } else {
                    let cur = state.sectors[x][y].vegetation;
                    for n in 2..9usize {
                        if cur == n as u8
                            && state.sectors[x][y].altitude == Altitude::Clear as u8
                            && rng.rand() % 4 == 0
                        {
                            state.sectors[x][y].vegetation = (n + 1) as u8;
                            break;
                        }
                    }
                }
            }
        }
    }

    // Inner equator: +/- 2
    for y in equator.saturating_sub(2)..=(equator + 2).min(map_y - 1) {
        for x in 0..map_x {
            if type_map[x][y] == LAND && state.sectors[x][y].altitude == Altitude::Clear as u8 {
                if rng.rand() % 10 == 0 {
                    state.sectors[x][y].vegetation = Vegetation::Desert as u8;
                } else if rng.rand() % 10 == 0 {
                    state.sectors[x][y].vegetation = Vegetation::Jungle as u8;
                } else if rng.rand() % 10 == 0 {
                    state.sectors[x][y].vegetation = Vegetation::Swamp as u8;
                } else {
                    let cur = state.sectors[x][y].vegetation;
                    for n in 2..4usize {
                        if cur == n as u8 {
                            state.sectors[x][y].vegetation = (n + 1) as u8;
                            break;
                        }
                    }
                }
            }
        }
    }

    // Expand swamps
    for y in 2..map_y {
        for x in 2..map_x {
            if state.sectors[x][y].vegetation == Vegetation::Swamp as u8 {
                for di in 0..2 {
                    for dj in 0..2 {
                        if type_map[x - di][y - dj] == LAND && rng.rand() % 3 == 0 {
                            state.sectors[x - di][y - dj].vegetation = Vegetation::Swamp as u8;
                        }
                    }
                }
            }
        }
    }

    // Expand deserts
    for y in 2..map_y {
        for x in 2..map_x {
            if state.sectors[x][y].vegetation == Vegetation::Desert as u8 {
                for di in 0..2 {
                    for dj in 0..2 {
                        if type_map[x - di][y - dj] == LAND && rng.rand() % 3 == 0 {
                            state.sectors[x - di][y - dj].vegetation = Vegetation::Desert as u8;
                        }
                    }
                }
            }
        }
    }

    // Change all volcanos to peaks
    for y in 1..map_y {
        for x in 1..map_x {
            if state.sectors[x][y].vegetation == Vegetation::Volcano as u8 {
                state.sectors[x][y].altitude = Altitude::Peak as u8;
            }
        }
    }

    // No desert next to water
    for y in 1..(map_y - 1) {
        for x in 1..(map_x - 1) {
            if state.sectors[x][y].vegetation == Vegetation::Desert as u8 {
                'check: for di in 0..=2 {
                    for dj in 0..=2 {
                        if state.sectors[x + di - 1][y + dj - 1].altitude == Altitude::Water as u8 {
                            state.sectors[x][y].vegetation = Vegetation::LtVeg as u8;
                            break 'check;
                        }
                    }
                }
            }
        }
    }

    // -- Step 9: Raw materials --
    raw_materials(state, rng);
}

/// fill_edge(AX, AY) — fill in a square's edges with land or sea.
/// Matches C exactly including wrap-around.
fn fill_edge(
    ax: usize,
    ay: usize,
    maxx: usize,
    maxy: usize,
    area_map: &[Vec<u8>],
    type_map: &mut [Vec<u8>],
    rng: &mut ConquerRng,
) {
    let x0 = ax;
    let y0 = ay;
    let x1 = if ax == 0 { maxx - 1 } else { ax - 1 };
    let x2 = if ax + 1 >= maxx { 0 } else { ax + 1 };
    let y3 = if ay == 0 { maxy - 1 } else { ay - 1 };
    let y4 = if ay + 1 >= maxy { 0 } else { ay + 1 };

    let area = area_map[x0][y0] as i32;

    // Fill south edge (Y0*8+7)
    let ea = area_map[x0][y4] as i32;
    for i in 0..8 {
        let tx = x0 * 8 + i;
        let ty = y0 * 8 + 7;
        if area + ea > 6 {
            type_map[tx][ty] = LAND;
        } else if area + ea > 3 {
            type_map[tx][ty] = if rng.rand() % 2 == 0 { LAND } else { Altitude::Water as u8 };
        } else {
            type_map[tx][ty] = Altitude::Water as u8;
        }
    }

    // Fill east edge (X0*8+7)
    let ea = area_map[x2][y0] as i32;
    for i in 0..8 {
        let tx = x0 * 8 + 7;
        let ty = y0 * 8 + i;
        if area + ea > 6 {
            type_map[tx][ty] = LAND;
        } else if area + ea > 3 {
            type_map[tx][ty] = if rng.rand() % 2 == 0 { LAND } else { Altitude::Water as u8 };
        } else {
            type_map[tx][ty] = Altitude::Water as u8;
        }
    }

    // Fill west edge (X0*8)
    let ea = area_map[x1][y0] as i32;
    for i in 0..=7 {
        let tx = x0 * 8;
        let ty = y0 * 8 + i;
        if area + ea > 6 {
            type_map[tx][ty] = LAND;
        } else if area + ea > 3 {
            type_map[tx][ty] = if rng.rand() % 2 == 0 { LAND } else { Altitude::Water as u8 };
        } else {
            type_map[tx][ty] = Altitude::Water as u8;
        }
    }

    // Fill north edge (Y0*8)
    let ea = area_map[x0][y3] as i32;
    for i in 0..8 {
        let tx = x0 * 8 + i;
        let ty = y0 * 8;
        if area + ea > 6 {
            type_map[tx][ty] = LAND;
        } else if area + ea > 3 {
            type_map[tx][ty] = if rng.rand() % 2 == 0 { LAND } else { Altitude::Water as u8 };
        } else {
            type_map[tx][ty] = Altitude::Water as u8;
        }
    }
}

/// raw_materials() — place each sector's raw materials, then populate.
/// Matches C rawmaterials() exactly.
fn raw_materials(state: &mut GameState, rng: &mut ConquerRng) {
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;

    state.world.turn = 1;

    // Compute nmountains for random trade good weighting
    let mut nmountains: i32 = 10 * (END_NORMAL as i32 + 1);
    for i in 0..=(END_NORMAL as usize) {
        nmountains -= tg_value(i) as i32;
    }

    for y in 0..map_y {
        for x in 0..map_x {
            let sct = &mut state.sectors[x][y];
            sct.designation = Designation::NoDesig as u8;
            sct.trade_good = TradeGood::None as u8;
            sct.owner = 0;
            sct.metal = 0;
            sct.jewels = 0;
            sct.fortress = 0;
            sct.people = 0;

            if !is_habitable(&state.sectors[x][y]) {
                continue;
            }

            // Exotic trade goods
            if rng.rand() % 100 < TRADEPCT {
                let is_mountain = state.sectors[x][y].altitude == Altitude::Mountain as u8;
                if rng.rand() % 100 < METALPCT || is_mountain {
                    getmetal(&mut state.sectors[x][y], rng);
                } else if rng.rand() % (100 - METALPCT) < JEWELPCT {
                    getjewel(&mut state.sectors[x][y], rng);
                } else {
                    // Random good
                    let mut valid = false;
                    let mut attempts = 0;
                    while !valid && attempts < 1000 {
                        attempts += 1;
                        let mut j = rng.rand() % nmountains;
                        let mut tg_idx = 0usize;
                        for ii in 0..=(END_NORMAL as usize) {
                            j -= 10 - tg_value(ii) as i32;
                            if j <= 0 {
                                tg_idx = ii;
                                break;
                            }
                        }

                        // Fish: must be next to water
                        if tg_idx == TradeGood::Fish as usize {
                            let mut found_water = false;
                            for dx in -1i32..=1 {
                                for dy in -1i32..=1 {
                                    let nx = x as i32 + dx;
                                    let ny = y as i32 + dy;
                                    if on_map(nx, ny, map_x as i32, map_y as i32)
                                        && state.sectors[nx as usize][ny as usize].altitude
                                            == Altitude::Water as u8
                                    {
                                        found_water = true;
                                    }
                                }
                            }
                            if !found_water {
                                continue;
                            }
                        }

                        // Corn/fruit: needs arable land (food >= 6)
                        if (tg_idx == TradeGood::Corn as usize
                            || tg_idx == TradeGood::Fruit as usize)
                            && tofood(&state.sectors[x][y], None) < 6
                        {
                            continue;
                        }

                        // Timber/pine/oak: needs wood/forest
                        let sct_veg = state.sectors[x][y].vegetation;
                        if (tg_idx == TradeGood::Timber as usize
                            || tg_idx == TradeGood::Pine as usize
                            || tg_idx == TradeGood::Oak as usize)
                            && sct_veg != Vegetation::Forest as u8
                            && sct_veg != Vegetation::Wood as u8
                        {
                            continue;
                        }

                        valid = true;
                        state.sectors[x][y].trade_good = tg_idx as u8;
                    }
                }
            }
        }
    }

    // Populate monster nations
    populate_monsters(state, rng);

    // Set mercenary pool
    state.world.merc_mil = ST_MMEN;
    state.world.merc_aplus = ST_MATT;
    state.world.merc_dplus = ST_MDEF;
}

/// populate_monsters() — place pirate/savage/nomad/lizard nations.
/// Matches C populate() monster placement section.
fn populate_monsters(state: &mut GameState, rng: &mut ConquerRng) {
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;
    let numsects = map_x * map_y;

    // Set up god
    state.nations[0].name = "unowned".to_string();
    state.nations[0].race = '-';
    state.nations[0].location = '-';
    state.nations[0].powers = (Power::KNOWALL | Power::NINJA).bits();
    state.nations[0].mark = '-';

    // Initialize all non-god nations
    for i in 1..NTOTAL {
        state.nations[i].password = state.nations[0].password.clone();
        state.nations[i].powers = 0;
        state.nations[i].repro = 0;
        state.nations[i].active = NationStrategy::Inactive as u8;
        state.nations[i].max_move = 0;
        state.nations[i].mark = '-';
    }

    // Set diplomacy
    for i in 0..NTOTAL {
        for j in i..NTOTAL {
            state.nations[i].diplomacy[j] = DiplomaticStatus::Unmet as u8;
            state.nations[j].diplomacy[i] = DiplomaticStatus::Unmet as u8;
        }
    }

    // Set up monster nations (last 4)
    let monster_configs: [(usize, &str, &str, u8, char); 4] = [
        (NTOTAL - 1, "lizard", "dragon", NationStrategy::NpcLizard as u8, 'L'),
        (NTOTAL - 2, "savages", "shaman", NationStrategy::NpcSavage as u8, 'S'),
        (NTOTAL - 3, "nomad", "khan", NationStrategy::NpcNomad as u8, 'N'),
        (NTOTAL - 4, "pirate", "captain", NationStrategy::NpcPirate as u8, 'P'),
    ];

    for &(idx, name, leader, strategy, race) in &monster_configs {
        let ntn = &mut state.nations[idx];
        ntn.name = name.to_string();
        ntn.leader = leader.to_string();
        ntn.active = strategy;
        ntn.race = race;
        ntn.attack_plus = 0;
        ntn.defense_plus = 0;
        ntn.powers = Power::KNOWALL.bits();
        ntn.mark = '*';
        ntn.max_move = 12;
        ntn.repro = 5;

        // Everyone hates monsters
        for j in 1..NTOTAL {
            state.nations[idx].diplomacy[j] = DiplomaticStatus::War as u8;
            state.nations[j].diplomacy[idx] = DiplomaticStatus::War as u8;
        }
    }

    // Count monster units to place
    let temp = numsects / (MONSTER as usize);
    let mut npirates: i32 = 0;
    let mut nbarbarians: i32 = 0;
    let mut nnomads: i32 = 0;
    let mut nlizards: i32 = 0;

    for _ in 0..temp {
        match rng.rand() % 4 {
            0 => { if npirates < MAXNAVY as i32 { npirates += 1; } }
            1 => { if nbarbarians < MAXARM as i32 { nbarbarians += 1; } }
            2 => { if nnomads < MAXARM as i32 { nnomads += 1; } }
            3 => {
                if rng.rand() % 3 == 0 {
                    if nlizards < MAXARM as i32 / 2 { nlizards += 1; }
                } else {
                    if nnomads < MAXARM as i32 { nnomads += 1; }
                }
            }
            _ => {}
        }
    }

    let mut lizarmy: usize = 0;
    let mut barbarmy: usize = 0;
    let mut nomadarmy: usize = 0;
    let mut pirarmy: usize = 0;
    let mut loopcnt = 0;

    while (nlizards + npirates + nbarbarians + nnomads > 0) && loopcnt < 5000 {
        loopcnt += 1;

        let (country, _armynum) = if nlizards > 0 {
            let c = (0..NTOTAL)
                .find(|&i| state.nations[i].active == NationStrategy::NpcLizard as u8)
                .unwrap_or(NTOTAL - 1);
            (c, lizarmy)
        } else if nbarbarians > 0 {
            let c = (0..NTOTAL)
                .find(|&i| state.nations[i].active == NationStrategy::NpcSavage as u8)
                .unwrap_or(NTOTAL - 2);
            (c, barbarmy)
        } else if nnomads > 0 {
            let c = (0..NTOTAL)
                .find(|&i| state.nations[i].active == NationStrategy::NpcNomad as u8)
                .unwrap_or(NTOTAL - 3);
            (c, nomadarmy)
        } else {
            let c = (0..NTOTAL)
                .find(|&i| state.nations[i].active == NationStrategy::NpcPirate as u8)
                .unwrap_or(NTOTAL - 4);
            (c, pirarmy)
        };

        // Choose location
        let (x, y);
        let active = state.nations[country].active;
        if rng.rand() % 2 == 0 && active != NationStrategy::NpcLizard as u8 {
            if rng.rand() % 2 == 0 {
                let mut tx = (rng.rand() % 20) as usize;
                let mut ty = (rng.rand() % 20) as usize;
                if rng.rand() % 2 == 0 { tx = (rng.rand() % map_x as i32) as usize; }
                else { ty = (rng.rand() % map_y as i32) as usize; }
                x = tx.min(map_x - 1);
                y = ty.min(map_y - 1);
            } else {
                let mut tx = map_x - (rng.rand() % 20) as usize - 1;
                let mut ty = map_y - (rng.rand() % 20) as usize - 1;
                if rng.rand() % 2 == 0 { tx = (rng.rand() % map_x as i32) as usize; }
                else { ty = (rng.rand() % map_y as i32) as usize; }
                x = tx.min(map_x - 1);
                y = ty.min(map_y - 1);
            }
        } else {
            x = (rng.rand() % map_x as i32) as usize;
            y = (rng.rand() % map_y as i32) as usize;
        }

        if state.sectors[x][y].owner != 0 { continue; }
        if !is_habitable(&state.sectors[x][y]) { continue; }

        state.sectors[x][y].owner = country as u8;

        match active {
            a if a == NationStrategy::NpcLizard as u8 => {
                nlizards -= 1;
                state.sectors[x][y].designation = Designation::Fort as u8;
                state.sectors[x][y].metal = 0;
                let i_val = rng.rand() % 30;
                state.sectors[x][y].jewels = (8 + i_val) as u8;
                state.sectors[x][y].trade_good = TradeGood::Platinum as u8;
                state.sectors[x][y].fortress = (6 + i_val / 5) as u8;
                // Own surrounding sectors
                for di in -1i32..=1 {
                    for dj in -1i32..=1 {
                        let nx = x as i32 + di;
                        let ny = y as i32 + dj;
                        if on_map(nx, ny, map_x as i32, map_y as i32)
                            && state.sectors[nx as usize][ny as usize].altitude != Altitude::Water as u8
                        {
                            state.sectors[nx as usize][ny as usize].owner = country as u8;
                        }
                    }
                }
                // Garrison army
                if lizarmy < MAXARM {
                    let ntn = &mut state.nations[country];
                    ntn.armies[lizarmy].movement = 0;
                    ntn.armies[lizarmy].x = x as u8;
                    ntn.armies[lizarmy].y = y as u8;
                    ntn.armies[lizarmy].status = ArmyStatus::Garrison.to_value();
                    ntn.armies[lizarmy].soldiers = 750 + 100 * (rng.rand() as i64 % 10);
                    ntn.armies[lizarmy].unit_type = defaultunit(ntn);
                    lizarmy += 1;
                }
                // Attack army
                if lizarmy < MAXARM {
                    let ntn = &mut state.nations[country];
                    ntn.armies[lizarmy].movement = 8;
                    ntn.armies[lizarmy].x = x as u8;
                    ntn.armies[lizarmy].y = y as u8;
                    ntn.armies[lizarmy].status = ArmyStatus::Attack.to_value();
                    ntn.armies[lizarmy].soldiers = 750 + 100 * (rng.rand() as i64 % 10);
                    ntn.armies[lizarmy].unit_type = defaultunit(ntn);
                    lizarmy += 1;
                }
            }
            a if a == NationStrategy::NpcPirate as u8 => {
                // Pirates must be on islands - check surroundings
                let mut temp_ok = true;
                for di in -1i32..=1 {
                    for dj in -1i32..=1 {
                        let nx = x as i32 + di;
                        let ny = y as i32 + dj;
                        if (di != 0 || dj != 0)
                            && on_map(nx, ny, map_x as i32, map_y as i32)
                            && state.sectors[nx as usize][ny as usize].altitude != Altitude::Water as u8
                        {
                            if state.sectors[nx as usize][ny as usize].owner != 0 || rng.rand() % 2 == 0 {
                                temp_ok = false;
                            }
                        }
                    }
                }
                if !temp_ok {
                    state.sectors[x][y].owner = 0;
                    continue;
                }
                // Build the island (make surrounding non-center water)
                for di in -1i32..=1 {
                    for dj in -1i32..=1 {
                        let nx = x as i32 + di;
                        let ny = y as i32 + dj;
                        if (di != 0 || dj != 0)
                            && on_map(nx, ny, map_x as i32, map_y as i32)
                        {
                            let s = &mut state.sectors[nx as usize][ny as usize];
                            if s.altitude != Altitude::Water as u8 {
                                s.altitude = Altitude::Water as u8;
                                s.vegetation = Vegetation::None as u8; // BUG-COMPAT: C sets to WATER char
                                s.trade_good = TradeGood::None as u8;
                                s.jewels = 0;
                                s.metal = 0;
                            }
                        }
                    }
                }
                npirates -= 1;
                state.sectors[x][y].designation = Designation::BaseCamp as u8;
                if pirarmy < MAXARM {
                    let ntn = &mut state.nations[country];
                    ntn.armies[pirarmy].movement = 8;
                    ntn.armies[pirarmy].x = x as u8;
                    ntn.armies[pirarmy].y = y as u8;
                    ntn.armies[pirarmy].status = ArmyStatus::Attack.to_value();
                    ntn.armies[pirarmy].soldiers = 150 + 100 * (rng.rand() as i64 % 3);
                    ntn.armies[pirarmy].unit_type = defaultunit(ntn);
                    pirarmy += 1;
                }
                // Place navy
                let nvy_idx = npirates.max(0) as usize; // use decremented count as index
                if nvy_idx < MAXNAVY {
                    let ntn = &mut state.nations[country];
                    // Find next available navy slot
                    if let Some(ni) = (0..MAXNAVY).find(|&i| !ntn.navies[i].has_ships()) {
                        ntn.navies[ni].x = x as u8;
                        ntn.navies[ni].y = y as u8;
                        ntn.navies[ni].people = 0;
                        ntn.navies[ni].army_num = MAXARM as u8;
                        // Light warships: 2-6
                        let w = (rng.rand() % 5 + 2) as u16;
                        ntn.navies[ni].warships = NavalSize::set_ships(ntn.navies[ni].warships, NavalSize::Light, w);
                        // Medium warships: 1-3
                        let w = (rng.rand() % 3 + 1) as u16;
                        ntn.navies[ni].warships = NavalSize::set_ships(ntn.navies[ni].warships, NavalSize::Medium, w);
                        // Heavy warships: 0-1
                        let w = (rng.rand() % 2) as u16;
                        ntn.navies[ni].warships = NavalSize::set_ships(ntn.navies[ni].warships, NavalSize::Heavy, w);
                        ntn.navies[ni].crew = SHIPCREW as u8;
                    }
                }
            }
            a if a == NationStrategy::NpcNomad as u8 => {
                nnomads -= 1;
                if nomadarmy < MAXARM {
                    let ntn = &mut state.nations[country];
                    ntn.armies[nomadarmy].x = x as u8;
                    ntn.armies[nomadarmy].y = y as u8;
                    ntn.armies[nomadarmy].status = ArmyStatus::Attack.to_value();
                    ntn.armies[nomadarmy].soldiers = 100 + 100 * (rng.rand() as i64 % 8);
                    ntn.armies[nomadarmy].unit_type = defaultunit(ntn);
                    nomadarmy += 1;
                }
            }
            a if a == NationStrategy::NpcSavage as u8 => {
                nbarbarians -= 1;
                if barbarmy < MAXARM {
                    let ntn = &mut state.nations[country];
                    ntn.armies[barbarmy].x = x as u8;
                    ntn.armies[barbarmy].y = y as u8;
                    ntn.armies[barbarmy].status = ArmyStatus::Attack.to_value();
                    ntn.armies[barbarmy].soldiers = 100 + 100 * (rng.rand() as i64 % 4);
                    ntn.armies[barbarmy].unit_type = defaultunit(ntn);
                    barbarmy += 1;
                }
            }
            _ => {}
        }
    }

    // Put random monsters around the world (for savage nation)
    for country in 0..NTOTAL {
        if state.nations[country].active != NationStrategy::NpcSavage as u8 {
            continue;
        }
        let mut armynum = barbarmy;
        while armynum < MAXARM {
            let x = (rng.rand() % map_x as i32) as usize;
            let y = (rng.rand() % map_y as i32) as usize;
            if is_habitable(&state.sectors[x][y]) && state.sectors[x][y].owner == 0 {
                state.sectors[x][y].owner = country as u8;
                if state.sectors[x][y].jewels == 0 {
                    getjewel(&mut state.sectors[x][y], rng);
                }
                let min_monster = UnitType::MIN_MONSTER;
                let max_monster = UnitType::MAX_MONSTER;
                let atype = min_monster + (rng.rand() % (max_monster as i32 - min_monster as i32 + 1)) as u8;
                let stats_idx = (atype % UTYPE) as usize;
                let ntn = &mut state.nations[country];
                ntn.armies[armynum].x = x as u8;
                ntn.armies[armynum].y = y as u8;
                ntn.armies[armynum].status = ArmyStatus::Attack.to_value();
                ntn.armies[armynum].unit_type = atype;
                ntn.armies[armynum].soldiers = UNIT_MIN_STRENGTH
                    .get(stats_idx)
                    .copied()
                    .unwrap_or(50) as i64;
                ntn.armies[armynum].movement = 10;
                armynum += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_world_deterministic() {
        let mut state1 = GameState::new(32, 32);
        let mut rng1 = ConquerRng::new(42);
        zeroworld(&mut state1);
        create_world(&mut state1, &mut rng1, 30);

        let mut state2 = GameState::new(32, 32);
        let mut rng2 = ConquerRng::new(42);
        zeroworld(&mut state2);
        create_world(&mut state2, &mut rng2, 30);

        // Same seed = identical results
        for x in 0..32 {
            for y in 0..32 {
                assert_eq!(state1.sectors[x][y], state2.sectors[x][y],
                    "Sector mismatch at ({}, {})", x, y);
            }
        }
        for i in 0..NTOTAL {
            assert_eq!(state1.nations[i].armies, state2.nations[i].armies,
                "Army mismatch for nation {}", i);
        }
    }

    #[test]
    fn test_zeroworld() {
        let mut state = GameState::new(32, 32);
        state.nations[5].active = NationStrategy::PcGood as u8;
        state.nations[5].armies[0].soldiers = 1000;
        zeroworld(&mut state);
        assert_eq!(state.nations[5].active, NationStrategy::Inactive as u8);
        assert_eq!(state.nations[5].armies[0].soldiers, 0);
    }

    #[test]
    fn test_world_has_land_and_water() {
        let mut state = GameState::new(32, 32);
        let mut rng = ConquerRng::new(42);
        zeroworld(&mut state);
        create_world(&mut state, &mut rng, 30);

        let mut water = 0;
        let mut land = 0;
        for x in 0..32 {
            for y in 0..32 {
                if state.sectors[x][y].altitude == Altitude::Water as u8 {
                    water += 1;
                } else {
                    land += 1;
                }
            }
        }
        assert!(water > 0, "World should have water");
        assert!(land > 0, "World should have land");
    }
}
