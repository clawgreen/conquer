#![allow(unused_variables, unused_imports, unused_assignments)]
// conquer-engine/src/movement.rs — Movement system ported from move.c & misc.c
//
// T200-T210: movement costs, updmove, land_reachp, army movement resolution,
// zone of control, sector taking, flee mechanics.

use conquer_core::*;
use conquer_core::powers::Power;
use conquer_core::rng::ConquerRng;
use conquer_core::tables::*;
use crate::utils::*;
use crate::combat::takesector;

/// Update the movement cost grid for a given race and nation.
/// Matches C updmove() exactly.
/// Movement cost is sum of altitude cost + vegetation cost.
/// Negative values = water (impassable for land, passable for navy).
pub fn update_move_costs(
    state: &mut GameState,
    race: char,
    nation_idx: usize,
) {
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;
    let nation = &state.nations[nation_idx];

    let (ele_costs, veg_costs) = match race {
        'O' => (O_ELE_COST.as_bytes(), O_VEG_COST.as_bytes()),
        'E' => (E_ELE_COST.as_bytes(), E_VEG_COST.as_bytes()),
        'D' => (D_ELE_COST.as_bytes(), D_VEG_COST.as_bytes()),
        _ => (H_ELE_COST.as_bytes(), H_VEG_COST.as_bytes()),
    };

    // Dervish/destroyer modify costs
    let has_dervish = Power::has_power(nation.powers, Power::DERVISH);
    let has_destroyer = Power::has_power(nation.powers, Power::DESTROYER);

    for x in 0..map_x {
        for y in 0..map_y {
            let sct = &state.sectors[x][y];
            let alt = sct.altitude as usize;
            let veg = sct.vegetation as usize;

            // Water
            if alt == Altitude::Water as usize {
                // Coastal water = -1, deep water = -3
                // Check adjacency for coast
                let mut is_coast = false;
                for dx in -1i32..=1 {
                    for dy in -1i32..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if state.on_map(nx, ny)
                            && state.sectors[nx as usize][ny as usize].altitude
                                != Altitude::Water as u8
                        {
                            is_coast = true;
                        }
                    }
                }
                state.move_cost[x][y] = if is_coast { -1 } else { -3 };
                continue;
            }

            // Get base costs from tables
            let ele_c = if alt < ele_costs.len() {
                let c = ele_costs[alt];
                if c == b'/' {
                    -1i16
                } else {
                    (c as i16) - (b'0' as i16)
                }
            } else {
                -1
            };

            let veg_c = if veg < veg_costs.len() {
                let c = veg_costs[veg];
                if c == b'/' {
                    -1i16
                } else {
                    (c as i16) - (b'0' as i16)
                }
            } else {
                -1
            };

            if ele_c < 0 || veg_c < 0 {
                state.move_cost[x][y] = -2; // impassable land
                continue;
            }

            let mut cost = ele_c + veg_c;

            // Dervish/destroyer: desert/ice costs 1 instead of normal
            if (has_dervish || has_destroyer)
                && (sct.vegetation == Vegetation::Desert as u8
                    || sct.vegetation == Vegetation::Ice as u8)
            {
                cost = 1;
            }

            // Roads reduce cost
            if sct.designation == Designation::Road as u8 && cost > 1 {
                cost = 1;
            }

            // Minimum cost of 1 for habitable land
            if cost < 1 {
                cost = 1;
            }

            state.move_cost[x][y] = cost;
        }
    }
}

/// land_reachp(x1, y1, x2, y2, max_move, nation_idx) — can an army reach from (x1,y1) to (x2,y2)?
/// Simplified adjacency check matching C logic: must be adjacent (1 step) and cost <= max_move.
pub fn land_reachp(
    state: &GameState,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    max_move: u8,
    _nation_idx: usize,
) -> bool {
    if !state.on_map(x2, y2) {
        return false;
    }
    // Must be adjacent
    if (x2 - x1).abs() > 1 || (y2 - y1).abs() > 1 {
        return false;
    }
    let cost = state.move_cost[x2 as usize][y2 as usize];
    if cost < 0 {
        return false; // water or impassable
    }
    cost <= max_move as i16
}

/// Zone of control calculation.
/// Returns total enemy soldiers in the sector that would trigger ZoC.
pub fn zone_of_control(
    state: &GameState,
    x: i32,
    y: i32,
    moving_nation: usize,
) -> i64 {
    let mut total: i64 = 0;
    for nation_idx in 0..NTOTAL {
        if nation_idx == moving_nation {
            continue;
        }
        for anum in 0..MAXARM {
            let army = &state.nations[nation_idx].armies[anum];
            if army.soldiers <= 0 {
                continue;
            }
            if army.x as i32 != x || army.y as i32 != y {
                continue;
            }
            // Must be hostile or at war
            if state.nations[moving_nation].diplomacy[nation_idx] < DiplomaticStatus::Hostile as u8
                && state.nations[nation_idx].diplomacy[moving_nation]
                    < DiplomaticStatus::Hostile as u8
            {
                continue;
            }
            if army.status == ArmyStatus::Scout.to_value() {
                continue;
            }
            if army.unit_type == UnitType::NINJA.0 {
                continue;
            }
            total += army.soldiers;
        }
    }
    total
}

/// Move a single army one step. Returns remaining movement points.
/// Handles movement cost, zone of control, and sector taking.
pub fn move_army_step(
    state: &mut GameState,
    nation_idx: usize,
    army_idx: usize,
    new_x: i32,
    new_y: i32,
    rng: &mut ConquerRng,
) -> Result<u8, &'static str> {
    if !state.on_map(new_x, new_y) {
        return Err("Off map");
    }

    let army = &state.nations[nation_idx].armies[army_idx];
    let astat = army.status;
    let atype = army.unit_type;
    let asold = army.soldiers;
    let amove = army.movement;

    if amove == 0 {
        return Err("No movement remaining");
    }

    // Flight movement
    if astat == ArmyStatus::Flight.to_value() {
        let sct = &state.sectors[new_x as usize][new_y as usize];
        let fcost = flightcost(sct);
        let mcost = state.move_cost[new_x as usize][new_y as usize];
        let effective_cost = if mcost > 0 && fcost > mcost as i32 {
            mcost as i32
        } else {
            fcost
        };
        if effective_cost < 0 || effective_cost > amove as i32 {
            return Err("Can't fly there");
        }
        state.nations[nation_idx].armies[army_idx].movement -= effective_cost as u8;
    } else {
        let cost = state.move_cost[new_x as usize][new_y as usize];
        if cost < 0 || cost > amove as i16 {
            return Err("Movement cost too high or impassable");
        }

        // Check trespassing rules
        let sct = &state.sectors[new_x as usize][new_y as usize];
        let s_owner = sct.owner as usize;
        if astat != ArmyStatus::Scout.to_value()
            && atype != UnitType::NINJA.0
            && (atype < UnitType::MIN_LEADER || atype >= UnitType::MIN_MONSTER || astat == ArmyStatus::General.to_value())
            && s_owner != 0
            && s_owner != nation_idx
            && sct.people > 100
            && state.nations[s_owner].diplomacy[nation_idx] > DiplomaticStatus::Allied as u8
            && state.nations[nation_idx].diplomacy[s_owner] < DiplomaticStatus::War as u8
        {
            return Err("Cannot enter non-allied land without declaring war");
        }

        if s_owner != nation_idx
            && s_owner != 0
            && astat != ArmyStatus::Scout.to_value()
            && state.nations[nation_idx].diplomacy[s_owner] == DiplomaticStatus::Unmet as u8
        {
            return Err("Cannot enter unmet nation's land");
        }

        state.nations[nation_idx].armies[army_idx].movement -= cost as u8;
    }

    // Update position
    state.nations[nation_idx].armies[army_idx].x = new_x as u8;
    state.nations[nation_idx].armies[army_idx].y = new_y as u8;

    // Zone of control
    let army = &state.nations[nation_idx].armies[army_idx];
    let astat = army.status;
    let atype = army.unit_type;

    if astat != ArmyStatus::Scout.to_value()
        && atype != UnitType::NINJA.0
        && (atype < UnitType::MIN_LEADER
            || astat == ArmyStatus::General.to_value()
            || atype >= UnitType::MIN_MONSTER)
        && astat != ArmyStatus::Flight.to_value()
    {
        let total_enemy = zone_of_control(state, new_x, new_y, nation_idx);
        let my_troops = army.soldiers;
        if my_troops < total_enemy {
            // Stop completely
            state.nations[nation_idx].armies[army_idx].movement = 0;
        } else if total_enemy > 0 {
            // Reduce movement proportionally
            let ntn = &state.nations[nation_idx];
            let atype = ntn.armies[army_idx].unit_type;
            let unit_move = UNIT_MOVE
                .get(UnitType(atype).stats_index().unwrap_or(0))
                .copied()
                .unwrap_or(10);
            let reduction =
                (total_enemy * ntn.max_move as i64 * unit_move as i64) / (10 * my_troops);
            let current = state.nations[nation_idx].armies[army_idx].movement as i64;
            let new_move = current - reduction;
            if new_move <= 0 || new_move > 150 {
                state.nations[nation_idx].armies[army_idx].movement = 0;
            } else {
                state.nations[nation_idx].armies[army_idx].movement = new_move as u8;
            }
        }
    }

    Ok(state.nations[nation_idx].armies[army_idx].movement)
}

/// flee(x, y, direction, slaver) — people flee from a captured sector.
/// Matches C flee() from misc.c.
pub fn flee(
    state: &mut GameState,
    x: i32,
    y: i32,
    _direction: i32,
    is_slaver: bool,
    rng: &mut ConquerRng,
) {
    let people = state.sectors[x as usize][y as usize].people;
    if people <= 0 {
        return;
    }

    if is_slaver {
        // Slavers keep the people
        return;
    }

    // People scatter to adjacent owned sectors
    let owner = state.sectors[x as usize][y as usize].owner as usize;
    let mut fled: i64 = 0;
    let flee_amount = people * 3 / 4; // 75% flee

    for dx in -1i32..=1 {
        for dy in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x + dx;
            let ny = y + dy;
            if !state.on_map(nx, ny) {
                continue;
            }
            let target = &state.sectors[nx as usize][ny as usize];
            if target.owner as usize == owner && is_habitable(target) {
                let amount = flee_amount / 8;
                state.sectors[nx as usize][ny as usize].people += amount;
                fled += amount;
            }
        }
    }

    state.sectors[x as usize][y as usize].people -= fled;
    // Rest of the people die
    state.sectors[x as usize][y as usize].people /= 4;
}

/// armymove() for NPC armies — used by the NPC AI to move armies toward
/// attractive sectors. Returns number of free armies (for NPC status calc).
/// Matches C npc.c armymove() pattern.
/// NPC army movement — matches C armymove() from update.c.
/// Returns number of sectors taken (used to update NPC active status).
/// C: original/update.c:247
pub fn npc_army_move(
    state: &mut GameState,
    nation_idx: usize,
    army_idx: usize,
    attr: &mut Vec<Vec<i32>>,
    rng: &mut ConquerRng,
) -> i32 {
    let astat = state.nations[nation_idx].armies[army_idx].status;
    let max_move = state.nations[nation_idx].armies[army_idx].movement;

    // C: if(P_ASTAT>=NUMSTATUS || P_AMOVE==0) return(takesctr);
    if astat >= NUMSTATUS || max_move == 0 {
        return 0;
    }

    let ax = state.nations[nation_idx].armies[army_idx].x as i32;
    let ay = state.nations[nation_idx].armies[army_idx].y as i32;
    let unit_type = state.nations[nation_idx].armies[army_idx].unit_type;
    let soldiers = state.nations[nation_idx].armies[army_idx].soldiers;
    let mut take_sector = 0i32;

    // Leader without a group: type >= MINLEADER AND < MINMONSTER AND not GENERAL
    let is_leader = unit_type >= UnitType::MIN_LEADER
        && unit_type < UnitType::MIN_MONSTER;
    let lead_flag = is_leader && astat != ArmyStatus::General.to_value();

    if lead_flag {
        // Leader looking for infantry to lead.
        // Sum all unattached infantry soldiers as weights.
        let mut sum: i64 = 0;
        for i in 0..MAXARM {
            let a = &state.nations[nation_idx].armies[i];
            if a.unit_type < UnitType::MIN_LEADER
                && a.status != ArmyStatus::Militia.to_value()
                && a.status != ArmyStatus::OnBoard.to_value()
                && a.status != ArmyStatus::Garrison.to_value()
                && a.status != ArmyStatus::Traded.to_value()
                && a.status < NUMSTATUS
            {
                sum += a.soldiers as i64;
            }
        }

        if sum == 0 {
            // No troops to lead — return to capitol and defend
            let cx = state.nations[nation_idx].cap_x;
            let cy = state.nations[nation_idx].cap_y;
            state.nations[nation_idx].armies[army_idx].x = cx;
            state.nations[nation_idx].armies[army_idx].y = cy;
            state.nations[nation_idx].armies[army_idx].status =
                ArmyStatus::Defend.to_value();
            return 0;
        }

        // Weighted random selection: pick an infantry army to join
        let mut where_ = rng.rand() as i64 % sum;
        let mut found_x = None;
        let mut found_y = None;
        for i in 0..MAXARM {
            let a = &state.nations[nation_idx].armies[i];
            if a.unit_type < UnitType::MIN_LEADER
                && a.status != ArmyStatus::Militia.to_value()
                && a.status != ArmyStatus::OnBoard.to_value()
                && a.status != ArmyStatus::Garrison.to_value()
                && a.status != ArmyStatus::Traded.to_value()
                && a.status < NUMSTATUS
            {
                where_ -= a.soldiers as i64;
                if where_ <= 0 {
                    found_x = Some(a.x);
                    found_y = Some(a.y);
                    break;
                }
            }
        }

        if let (Some(fx), Some(fy)) = (found_x, found_y) {
            state.nations[nation_idx].armies[army_idx].x = fx;
            state.nations[nation_idx].armies[army_idx].y = fy;
            // Form a group: find a compatible infantry at that location
            for i in 0..MAXARM {
                let a = &state.nations[nation_idx].armies[i];
                if a.unit_type < UnitType::MIN_LEADER
                    && a.status < NUMSTATUS
                    && a.soldiers >= 0
                    && a.status != ArmyStatus::Militia.to_value()
                    && a.status != ArmyStatus::Garrison.to_value()
                    && a.status != ArmyStatus::Sieged.to_value()
                    && a.status != ArmyStatus::Scout.to_value()
                    && a.status != ArmyStatus::OnBoard.to_value()
                    && a.status != ArmyStatus::Traded.to_value()
                    && a.x == fx
                    && a.y == fy
                {
                    state.nations[nation_idx].armies[i].status =
                        NUMSTATUS + army_idx as u8;
                    state.nations[nation_idx].armies[army_idx].status =
                        ArmyStatus::General.to_value();
                    break;
                }
            }
        }
        return take_sector;
    }

    // Normal unit (or GENERAL leader) movement.
    // menok: army can take sectors (large enough or is leader/monster)
    let menok_count = if is_leader && astat == ArmyStatus::General.to_value() {
        // GENERAL: count grouped infantry
        let mut count: i64 = 0;
        for i in 0..MAXARM {
            let a = &state.nations[nation_idx].armies[i];
            if a.status == NUMSTATUS + army_idx as u8
                && a.unit_type < UnitType::MIN_LEADER
            {
                count += a.soldiers as i64;
            }
        }
        count
    } else {
        soldiers as i64
    };
    let ts = takesector(state.nations[nation_idx].total_civ);
    let menok = menok_count > ts || unit_type >= UnitType::MIN_LEADER;

    // C: range of 4 if menok=FALSE, 2 if menok=TRUE
    let range: i32 = if menok { 2 } else { 4 };

    // Sum attractiveness across range (city-only filter when small army)
    let mut sum: i64 = 0;
    for x in (ax - range)..=(ax + range) {
        for y in (ay - range)..=(ay + range) {
            if !state.on_map(x, y) {
                continue;
            }
            let des = state.sectors[x as usize][y as usize].designation;
            if menok || is_city(des) {
                let a = attr[x as usize][y as usize];
                if a > 0 {
                    sum += a as i64;
                }
            }
        }
    }

    if sum == 0 {
        // Nowhere to go — return to capitol and defend
        let cx = state.nations[nation_idx].cap_x;
        let cy = state.nations[nation_idx].cap_y;
        state.nations[nation_idx].armies[army_idx].x = cx;
        state.nations[nation_idx].armies[army_idx].y = cy;
        state.nations[nation_idx].armies[army_idx].status =
            ArmyStatus::Defend.to_value();
        return take_sector;
    }

    // Weighted random sector selection from range
    let mut where_ = rng.rand() as i64 % sum;
    let mut moved = false;
    'outer: for x in (ax - range)..=(ax + range) {
        for y in (ay - range)..=(ay + range) {
            if !state.on_map(x, y) {
                continue;
            }
            let des = state.sectors[x as usize][y as usize].designation;
            if menok || is_city(des) {
                let a = attr[x as usize][y as usize];
                if a > 0 {
                    where_ -= a as i64;
                }
            }
            if where_ < 0 {
                let cost = state.move_cost[x as usize][y as usize];
                // Must be passable land and reachable
                if cost >= 1
                    && cost <= max_move as i16
                    && land_reachp(state, ax, ay, x, y, max_move, nation_idx)
                {
                    // Reduce attractiveness for own non-city sectors (spread armies)
                    if state.sectors[x as usize][y as usize].owner == nation_idx as u8
                        && !is_city(des)
                    {
                        attr[x as usize][y as usize] /= 8;
                    }

                    // Take unowned sector
                    if state.sectors[x as usize][y as usize].owner == 0 {
                        state.sectors[x as usize][y as usize].owner = nation_idx as u8;
                        attr[x as usize][y as usize] /= 8;
                        if state.nations[nation_idx].popularity < 127 {
                            state.nations[nation_idx].popularity += 1;
                        }
                        take_sector += 1;
                    }

                    // Move group with GENERAL leader
                    if is_leader && astat == ArmyStatus::General.to_value() {
                        for i in 0..MAXARM {
                            if state.nations[nation_idx].armies[i].soldiers > 0
                                && state.nations[nation_idx].armies[i].status
                                    == NUMSTATUS + army_idx as u8
                            {
                                state.nations[nation_idx].armies[i].x = x as u8;
                                state.nations[nation_idx].armies[i].y = y as u8;
                            }
                        }
                    }

                    state.nations[nation_idx].armies[army_idx].x = x as u8;
                    state.nations[nation_idx].armies[army_idx].y = y as u8;
                    moved = true;
                    break 'outer;
                }
            }
        }
    }

    // Fallback retry with ±2 range if first pass found no move
    if !moved {
        let fallback_range = 2i32;
        let mut sum2: i64 = 0;
        for x in (ax - fallback_range)..=(ax + fallback_range) {
            for y in (ay - fallback_range)..=(ay + fallback_range) {
                if state.on_map(x, y) {
                    sum2 += attr[x as usize][y as usize].max(0) as i64;
                }
            }
        }
        if sum2 > 0 {
            let mut where2 = rng.rand() as i64 % sum2;
            'outer2: for x in (ax - fallback_range)..=(ax + fallback_range) {
                for y in (ay - fallback_range)..=(ay + fallback_range) {
                    if !state.on_map(x, y) {
                        continue;
                    }
                    where2 -= attr[x as usize][y as usize].max(0) as i64;
                    if where2 < 0 {
                        let cost = state.move_cost[x as usize][y as usize];
                        if cost >= 1
                            && cost <= max_move as i16
                            && land_reachp(state, ax, ay, x, y, max_move, nation_idx)
                        {
                            if state.sectors[x as usize][y as usize].owner == 0 {
                                state.sectors[x as usize][y as usize].owner =
                                    nation_idx as u8;
                                attr[x as usize][y as usize] = 1;
                                if state.nations[nation_idx].popularity < 127 {
                                    state.nations[nation_idx].popularity += 1;
                                }
                                take_sector += 1;
                            }
                            if is_leader && astat == ArmyStatus::General.to_value() {
                                for i in 0..MAXARM {
                                    if state.nations[nation_idx].armies[i].soldiers > 0
                                        && state.nations[nation_idx].armies[i].status
                                            == NUMSTATUS + army_idx as u8
                                    {
                                        state.nations[nation_idx].armies[i].x = x as u8;
                                        state.nations[nation_idx].armies[i].y = y as u8;
                                    }
                                }
                            }
                            state.nations[nation_idx].armies[army_idx].x = x as u8;
                            state.nations[nation_idx].armies[army_idx].y = y as u8;
                            break 'outer2;
                        }
                    }
                }
            }
        }
    }

    take_sector
}

// ── Sector occupancy and capture ──

/// Compute which nation (if any) exclusively occupies each sector.
/// Returns a 2D vec: 0 = unoccupied or contested, N = nation N owns it exclusively.
/// Matches C prep() occupancy array logic.
/// compute_occupancy() — matches C prep(0, FALSE/-1).
/// Builds occ[][] grid: sector -> nation that solely occupies it.
/// - Excludes SCOUT armies (C: P_ASTAT!=SCOUT)
/// - Includes navies (C: fleet presence counts as occupation)
/// - Contested sectors (multiple nations) -> NTOTAL
fn compute_occupancy(state: &GameState) -> Vec<Vec<usize>> {
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;
    let mut occ: Vec<Vec<usize>> = vec![vec![0usize; map_y]; map_x];

    for country in 0..NTOTAL {
        if state.nations[country].active == NationStrategy::Inactive as u8 {
            continue;
        }
        // Armies (C: skip scouts only)
        for army in &state.nations[country].armies {
            if army.soldiers <= 0 { continue; }
            if army.status == ArmyStatus::Scout.to_value() { continue; }
            let x = army.x as usize;
            let y = army.y as usize;
            if x >= map_x || y >= map_y { continue; }
            if occ[x][y] == 0 || occ[x][y] == country {
                occ[x][y] = country;
            } else {
                occ[x][y] = NTOTAL; // Contested
            }
        }
        // Navies (C: fleets count as occupation)
        for nvy in &state.nations[country].navies {
            if nvy.warships == 0 && nvy.galleys == 0 && nvy.merchant == 0 { continue; }
            let x = nvy.x as usize;
            let y = nvy.y as usize;
            if x >= map_x || y >= map_y { continue; }
            if occ[x][y] == 0 || occ[x][y] == country {
                occ[x][y] = country;
            } else {
                occ[x][y] = NTOTAL;
            }
        }
    }
    occ
}

/// updcapture() — assign sector ownership to armies that sole-occupy them.
/// Matches C updcapture() structure.
/// Called AFTER combat, BEFORE trade.
pub fn update_capture(state: &mut GameState, rng: &mut ConquerRng) -> Vec<String> {
    let mut news = Vec::new();
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;

    let occ = compute_occupancy(state);

    for country in 1..NTOTAL {
        if state.nations[country].active == NationStrategy::Inactive as u8 {
            continue;
        }

        for armynum in 0..MAXARM {
            let army = &state.nations[country].armies[armynum];
            if army.soldiers <= 0 {
                continue;
            }

            let atype = army.unit_type;
            let ax = army.x as usize;
            let ay = army.y as usize;
            let astat = army.status;
            let soldiers = army.soldiers;

            if ax >= map_x || ay >= map_y {
                continue;
            }

            // C: only base units (P_ATYPE<MINLEADER) capture sectors
            if atype >= UnitType::MIN_LEADER {
                // But C also has a scout capture branch — handle below
                // C: else if(P_ASTAT==A_SCOUT && P_ATYPE!=A_SPY && P_ASOLD>0)
                if astat == ArmyStatus::Scout.to_value()
                    && atype != UnitType::SPY.0
                    && soldiers > 0
                {
                    // Scout can be captured by enemy in same sector
                    let occval = occ[ax][ay];
                    if occval != 0 && occval != country && occval < NTOTAL {
                        let enemy_dip = state.nations[occval].diplomacy[country];
                        let sct_owner = state.sectors[ax][ay].owner as usize;
                        let mut captured = false;
                        if enemy_dip >= DiplomaticStatus::Hostile as u8
                            && (rng.rand() % 100) < PFINDSCOUT
                        {
                            captured = true;
                        } else if sct_owner == occval
                            && state.nations[occval].diplomacy[country] != DiplomaticStatus::Treaty as u8
                            && state.nations[occval].diplomacy[country] != DiplomaticStatus::Allied as u8
                            && (rng.rand() % 100) < PFINDSCOUT / 5
                        {
                            captured = true;
                        }
                        if captured {
                            state.nations[country].armies[armynum].soldiers = 0;
                        }
                    }
                }
                continue;
            }

            // PC: needs TAKESECTOR; NPC: needs > 75 (C "cheat in favor of npcs")
            let strat = NationStrategy::from_value(state.nations[country].active);
            let is_pc = strat.map_or(false, |s| s.is_pc());
            let ts = takesector(state.nations[country].total_civ);

            let enough = if is_pc { soldiers >= ts } else { soldiers > 75 };
            if !enough {
                continue;
            }

            // Can't capture while on a fleet
            if astat == ArmyStatus::OnBoard.to_value() {
                continue;
            }

            // Can't capture water sectors
            if state.sectors[ax][ay].altitude == Altitude::Water as u8 {
                continue;
            }

            // Must be the sole occupier of this sector
            if occ[ax][ay] != country {
                continue;
            }

            let sct_owner = state.sectors[ax][ay].owner as usize;

            if sct_owner == 0 {
                // Unowned — capture it
                state.sectors[ax][ay].owner = country as u8;
                let pop = state.nations[country].popularity;
                if pop < MAXTGVAL as u8 {
                    state.nations[country].popularity = pop.saturating_add(1);
                }
            } else if sct_owner != country {
                // Enemy — capture if at war
                let dstatus = state.nations[country].diplomacy[sct_owner];
                if dstatus >= DiplomaticStatus::War as u8 {
                    // C: flee civilians (SLAVER variant)
                    let has_slaver = Power::has_power(state.nations[country].powers, Power::SLAVER);
                    crate::movement::flee(state, ax as i32, ay as i32, 1, has_slaver, rng);

                    state.sectors[ax][ay].owner = country as u8;
                    let pop = state.nations[country].popularity;
                    state.nations[country].popularity = pop.saturating_add(1);
                    news.push(format!(
                        "area {},{} captured by {} from {}",
                        ax, ay, state.nations[country].name, state.nations[sct_owner].name.clone()
                    ));
                }
            }
        }
    }

    // Check for capitols being sacked (C: sackem)
    for country in 1..NTOTAL {
        let strat = NationStrategy::from_value(state.nations[country].active);
        if !strat.map_or(false, |s| s.is_nation()) {
            continue;
        }
        let cap_x = state.nations[country].cap_x as usize;
        let cap_y = state.nations[country].cap_y as usize;
        if cap_x < map_x && cap_y < map_y {
            if state.sectors[cap_x][cap_y].owner as usize != country {
                news.push(format!(
                    "Capitol of {} has been sacked!",
                    state.nations[country].name
                ));
                // Deplete armies by PDEPLETE%
                for army in &mut state.nations[country].armies {
                    if army.soldiers > 0 {
                        army.soldiers = army.soldiers * (100 - PDEPLETE as i64) / 100;
                    }
                }
            }
        }
    }

    news
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_land_reachp_adjacent() {
        let state = GameState::new(16, 16);
        // Default movecost is 0, so everything is reachable
        assert!(land_reachp(&state, 5, 5, 6, 5, 10, 0));
        assert!(land_reachp(&state, 5, 5, 6, 6, 10, 0));
        assert!(!land_reachp(&state, 5, 5, 7, 5, 10, 0)); // too far
    }

    #[test]
    fn test_zone_of_control_no_enemies() {
        let state = GameState::new(16, 16);
        assert_eq!(zone_of_control(&state, 5, 5, 0), 0);
    }

    #[test]
    fn test_takesector_formula() {
        assert_eq!(takesector(0), 75);
        assert_eq!(takesector(100000), 285); // 100000/350 = ~285
    }
}
