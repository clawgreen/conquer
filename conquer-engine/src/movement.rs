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
pub fn npc_army_move(
    state: &mut GameState,
    nation_idx: usize,
    army_idx: usize,
    attr: &Vec<Vec<i32>>,
    rng: &mut ConquerRng,
) -> i32 {
    let army = &state.nations[nation_idx].armies[army_idx];
    if army.soldiers <= 0 {
        return 0;
    }

    let astat = army.status;
    // Skip stationary units
    if astat == ArmyStatus::Militia.to_value()
        || astat == ArmyStatus::Garrison.to_value()
        || astat == ArmyStatus::OnBoard.to_value()
        || astat == ArmyStatus::Traded.to_value()
        || astat == ArmyStatus::Rule.to_value()
    {
        return 0;
    }

    let ax = army.x as i32;
    let ay = army.y as i32;
    let max_move = army.movement;

    if max_move == 0 {
        return 0;
    }

    // Find most attractive adjacent sector
    let mut best_x = ax;
    let mut best_y = ay;
    let mut best_attr = 0;

    for dx in -1i32..=1 {
        for dy in -1i32..=1 {
            let nx = ax + dx;
            let ny = ay + dy;
            if !state.on_map(nx, ny) {
                continue;
            }
            let cost = state.move_cost[nx as usize][ny as usize];
            if cost < 0 || cost > max_move as i16 {
                continue;
            }
            let a = attr[nx as usize][ny as usize];
            if a > best_attr {
                best_attr = a;
                best_x = nx;
                best_y = ny;
            }
        }
    }

    // Don't move if current sector is more attractive or tied
    if best_attr <= attr[ax as usize][ay as usize] {
        return 0;
    }

    // Move there
    if best_x != ax || best_y != ay {
        let cost = state.move_cost[best_x as usize][best_y as usize];
        state.nations[nation_idx].armies[army_idx].x = best_x as u8;
        state.nations[nation_idx].armies[army_idx].y = best_y as u8;
        let amove = state.nations[nation_idx].armies[army_idx].movement;
        state.nations[nation_idx].armies[army_idx].movement =
            amove.saturating_sub(cost as u8);

        // Take undefended sectors
        let sct = &state.sectors[best_x as usize][best_y as usize];
        let s_owner = sct.owner as usize;
        let army = &state.nations[nation_idx].armies[army_idx];
        let ts = takesector(state.nations[nation_idx].total_civ);

        if army.soldiers >= ts
            && s_owner != nation_idx
            && (s_owner == 0
                || solds_in_sector(&state.nations[s_owner], best_x as u8, best_y as u8) == 0)
            && (army.status >= ArmyStatus::Defend.to_value()
                || army.status >= NUMSTATUS)
        {
            if s_owner != 0 {
                flee(state, best_x, best_y, 0, false, rng);
            }
            state.sectors[best_x as usize][best_y as usize].owner = nation_idx as u8;
            state.nations[nation_idx].armies[army_idx].movement = 0;
            return 1;
        }
        return 1;
    }
    0
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
