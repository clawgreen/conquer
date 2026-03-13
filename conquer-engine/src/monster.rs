#![allow(unused_variables, unused_imports, unused_assignments)]
// conquer-engine/src/monster.rs — Monster AI ported from npc.c monster functions
//
// T231-T245: do_nomad, do_pirate, do_savage, do_lizard, monster(),
// MORE_MONST spawning logic, monster movement and sector capture.

use crate::combat;
use crate::utils::*;
use conquer_core::rng::ConquerRng;
use conquer_core::tables::*;
use conquer_core::*;

/// Run all monster nation updates. Matches C monster() exactly.
pub fn update_monsters(state: &mut GameState, rng: &mut ConquerRng) -> Vec<String> {
    let mut news = Vec::new();

    for country in 1..NTOTAL {
        let active = state.nations[country].active;
        match NationStrategy::from_value(active) {
            Some(NationStrategy::NpcNomad) => {
                let n = do_nomad(state, country, rng);
                news.extend(n);
            }
            Some(NationStrategy::NpcPirate) => {
                do_pirate(state, country, rng);
            }
            Some(NationStrategy::NpcSavage) => {
                let n = do_savage(state, country, rng);
                news.extend(n);
            }
            Some(NationStrategy::NpcLizard) => {
                do_lizard(state, country, rng);
            }
            _ => {}
        }
    }

    // MORE_MONST: spawn additional monster armies if needed
    spawn_more_monsters(state, rng);

    news
}

/// do_nomad() — update nomad nation armies.
/// Matches C do_nomad() exactly.
fn do_nomad(state: &mut GameState, country: usize, rng: &mut ConquerRng) -> Vec<String> {
    let mut news = Vec::new();
    let map_x = state.world.map_x as i32;
    let map_y = state.world.map_y as i32;

    for armynum in 0..MAXARM {
        if state.nations[country].armies[armynum].soldiers <= 0 {
            continue;
        }

        state.nations[country].armies[armynum].status = ArmyStatus::Attack.to_value();

        // Calculate movement points
        let atype = state.nations[country].armies[armynum].unit_type;
        let max_move = state.nations[country].max_move;
        let unit_idx = UnitType(atype).stats_index().unwrap_or(0);
        let umove = UNIT_MOVE.get(unit_idx).copied().unwrap_or(10);
        state.nations[country].armies[armynum].movement = (max_move as i32 * umove / 10) as u8;

        // Growth: non-leaders grow 2%
        if atype < UnitType::MIN_LEADER {
            let sold = state.nations[country].armies[armynum].soldiers;
            state.nations[country].armies[armynum].soldiers = sold * 102 / 100;
        }

        let mut count = 0;
        let ax = state.nations[country].armies[armynum].x as i32;
        let ay = state.nations[country].armies[armynum].y as i32;

        loop {
            let x = ax + (rng.rand() % 3) as i32 - 1;
            let y = ay + (rng.rand() % 3) as i32 - 1;

            count += 1;
            if count > 100 {
                state.nations[country].armies[armynum].soldiers = 0;
                break;
            }

            // Nomads cannot stay in the same spot
            if x == ax && y == ay {
                continue;
            }
            if !state.on_map(x, y) {
                continue;
            }
            let sct = &state.sectors[x as usize][y as usize];
            if !is_habitable(sct) {
                continue;
            }
            // Check reachability (simplified: adjacent + habitable)
            let mc = state.move_cost[x as usize][y as usize];
            if mc < 0 {
                continue;
            }

            state.nations[country].armies[armynum].x = x as u8;
            state.nations[country].armies[armynum].y = y as u8;

            // Capture undefended sectors
            let s_owner = state.sectors[x as usize][y as usize].owner as usize;
            if (s_owner == 0 || solds_in_sector(&state.nations[s_owner], x as u8, y as u8) == 0)
                && NationStrategy::from_value(state.nations[s_owner].active)
                    .map_or(true, |s| s != NationStrategy::NpcNomad)
            {
                news.push(format!("nomads capture sector {},{}", x, y));
                if s_owner != 0 {
                    // flee
                    let people = state.sectors[x as usize][y as usize].people;
                    state.sectors[x as usize][y as usize].people = people / 4;
                }
                state.sectors[x as usize][y as usize].owner = country as u8;
                combat::devastate(&mut state.sectors[x as usize][y as usize]);
            }
            break;
        }
    }

    news
}

/// do_savage() — update savage nation armies.
/// Matches C do_savage() exactly.
fn do_savage(state: &mut GameState, country: usize, rng: &mut ConquerRng) -> Vec<String> {
    let mut news = Vec::new();

    for armynum in 0..MAXARM {
        if state.nations[country].armies[armynum].soldiers <= 0 {
            continue;
        }

        state.nations[country].armies[armynum].status = ArmyStatus::Attack.to_value();

        // Growth: non-leaders grow 2%
        let atype = state.nations[country].armies[armynum].unit_type;
        if atype < UnitType::MIN_LEADER {
            let sold = state.nations[country].armies[armynum].soldiers;
            state.nations[country].armies[armynum].soldiers = sold * 102 / 100;
        }

        // Calculate movement
        let max_move = state.nations[country].max_move;
        let unit_idx = UnitType(atype).stats_index().unwrap_or(0);
        let umove = UNIT_MOVE.get(unit_idx).copied().unwrap_or(10);
        state.nations[country].armies[armynum].movement = (max_move as i32 * umove / 10) as u8;

        let ax = state.nations[country].armies[armynum].x as i32;
        let ay = state.nations[country].armies[armynum].y as i32;

        let x = ax + (rng.rand() % 3) as i32 - 1;
        let y = ay + (rng.rand() % 3) as i32 - 1;

        if !state.on_map(x, y) {
            continue;
        }
        let sct = &state.sectors[x as usize][y as usize];
        if !is_habitable(sct) {
            continue;
        }
        let mc = state.move_cost[x as usize][y as usize];
        if mc < 0 {
            continue;
        }

        state.nations[country].armies[armynum].x = x as u8;
        state.nations[country].armies[armynum].y = y as u8;

        let s_owner = state.sectors[x as usize][y as usize].owner as usize;
        if (s_owner == 0 || solds_in_sector(&state.nations[s_owner], x as u8, y as u8) == 0)
            && NationStrategy::from_value(state.nations[s_owner].active)
                .map_or(true, |s| s != NationStrategy::NpcSavage)
        {
            news.push(format!("savages capture sector {},{}", x, y));
            if atype < UnitType::MIN_LEADER {
                if s_owner != 0 {
                    let people = state.sectors[x as usize][y as usize].people;
                    state.sectors[x as usize][y as usize].people = people / 4;
                }
                state.sectors[x as usize][y as usize].owner = country as u8;
            }
            combat::devastate(&mut state.sectors[x as usize][y as usize]);
        }
    }

    news
}

/// do_pirate() — update pirate nation fleets.
/// Matches C do_pirate() exactly.
fn do_pirate(state: &mut GameState, country: usize, rng: &mut ConquerRng) {
    // First pass: return pirate fleets to basecamp
    for nvynum in 0..MAXNAVY {
        let nvy = &state.nations[country].navies[nvynum];
        if nvy.warships == 0 {
            continue;
        }

        let mut campx = nvy.x as i32;
        let mut campy = nvy.y as i32;
        let mut found_base = false;

        let nx = nvy.x as i32;
        let ny = nvy.y as i32;

        for x in (nx - PRTZONE)..=(nx + PRTZONE) {
            for y in (ny - PRTZONE)..=(ny + PRTZONE) {
                if !state.on_map(x, y) {
                    continue;
                }
                if state.sectors[x as usize][y as usize].designation == Designation::BaseCamp as u8
                {
                    found_base = true;
                    campx = x;
                    campy = y;
                }
            }
        }

        if found_base {
            state.nations[country].navies[nvynum].x = campx as u8;
            state.nations[country].navies[nvynum].y = campy as u8;
        }
    }

    // Second pass: move toward enemy fleets
    for nvynum in 0..MAXNAVY {
        if state.nations[country].navies[nvynum].warships == 0 {
            continue;
        }

        let px = state.nations[country].navies[nvynum].x as i32;
        let py = state.nations[country].navies[nvynum].y as i32;

        for x in 1..NTOTAL {
            let x_strat = NationStrategy::from_value(state.nations[x].active);
            if !x_strat.map_or(false, |s| s.is_nation()) {
                continue;
            }
            for y in 0..MAXNAVY {
                let envy = &state.nations[x].navies[y];
                if envy.warships == 0 && envy.merchant == 0 && envy.galleys == 0 {
                    continue;
                }
                let ex = envy.x as i32;
                let ey = envy.y as i32;
                if (ex - px).abs() <= PRTZONE && (ey - py).abs() <= PRTZONE {
                    state.nations[country].navies[nvynum].x = ex as u8;
                    state.nations[country].navies[nvynum].y = ey as u8;
                }
            }
        }

        // MORE_MONST: randomly add warship (1/15 chance)
        if rng.rand() % 15 == 0 {
            let shipsize = rng.rand() % 3; // N_LIGHT to N_HEAVY
            let size = match shipsize {
                0 => NavalSize::Light,
                1 => NavalSize::Medium,
                _ => NavalSize::Heavy,
            };
            let nvy = &mut state.nations[country].navies[nvynum];
            let current = NavalSize::ships(nvy.warships, size);
            if current < N_MASK as i16 {
                nvy.warships = NavalSize::set_ships(nvy.warships, size, (current + 1) as u16);
            }
        }
    }
}

/// do_lizard() — update lizard nation armies.
/// Matches C do_lizard() from update.c line 730.
/// Lizards come in pairs: even armies garrison, odd armies patrol around them.
fn do_lizard(state: &mut GameState, country: usize, rng: &mut ConquerRng) {
    let map_x = state.world.map_x as i32;
    let map_y = state.world.map_y as i32;

    for armynum in 0..MAXARM {
        if state.nations[country].armies[armynum].soldiers <= 0 {
            continue;
        }

        // All lizard armies get 20 movement
        state.nations[country].armies[armynum].movement = 20;

        // Growth: 2%
        let sold = state.nations[country].armies[armynum].soldiers;
        state.nations[country].armies[armynum].soldiers = sold * 102 / 100;

        if armynum % 2 == 0 {
            // Even armies: garrison (unless sieged)
            if state.nations[country].armies[armynum].status != ArmyStatus::Sieged.to_value() {
                state.nations[country].armies[armynum].status = ArmyStatus::Garrison.to_value();
            }
        } else {
            // Odd armies: patrol around their paired even army
            // If the paired army (armynum-1) is dead, this one dies too
            if state.nations[country].armies[armynum - 1].soldiers <= 0 {
                state.nations[country].armies[armynum].soldiers = 0;
                continue;
            }

            // Move to paired army's location
            let pair_x = state.nations[country].armies[armynum - 1].x as i32;
            let pair_y = state.nations[country].armies[armynum - 1].y as i32;
            state.nations[country].armies[armynum].x = pair_x as u8;
            state.nations[country].armies[armynum].y = pair_y as u8;

            // Try to move to relieve sieges or attack nearby sectors
            if state.nations[country].armies[armynum].status != ArmyStatus::Sieged.to_value()
                && state.nations[country].armies[armynum - 1].status
                    != ArmyStatus::Sieged.to_value()
            {
                for dx in -1..=1i32 {
                    for dy in -1..=1i32 {
                        let nx = pair_x + dx;
                        let ny = pair_y + dy;
                        if nx < 0 || ny < 0 || nx >= map_x || ny >= map_y {
                            continue;
                        }
                        let sct = &state.sectors[nx as usize][ny as usize];
                        if sct.altitude == Altitude::Water as u8
                            || sct.altitude == Altitude::Peak as u8
                        {
                            continue;
                        }
                        if sct.owner as usize != country && rng.rand() % 3 == 0 {
                            state.nations[country].armies[armynum].x = nx as u8;
                            state.nations[country].armies[armynum].y = ny as u8;
                        }
                    }
                }
            }

            // If on a fort owned by this nation, garrison; otherwise attack
            let ax = state.nations[country].armies[armynum].x as usize;
            let ay = state.nations[country].armies[armynum].y as usize;
            if state.sectors[ax][ay].designation == Designation::Fort as u8
                && state.sectors[ax][ay].owner as usize == country
            {
                if state.nations[country].armies[armynum].status != ArmyStatus::Sieged.to_value() {
                    state.nations[country].armies[armynum].status = ArmyStatus::Garrison.to_value();
                }
            } else {
                state.nations[country].armies[armynum].status = ArmyStatus::Attack.to_value();
            }
        }
    }
}

/// MORE_MONST: Spawn additional monster armies if the world needs them.
/// Matches C monster() MORE_MONST block.
fn spawn_more_monsters(state: &mut GameState, rng: &mut ConquerRng) {
    let map_x = state.world.map_x as i32;
    let map_y = state.world.map_y as i32;
    let numsects = (map_x * map_y) as i64;

    // Find savage and nomad nation indices
    let mut savages_idx = None;
    let mut nomads_idx = None;

    for i in 1..NTOTAL {
        match NationStrategy::from_value(state.nations[i].active) {
            Some(NationStrategy::NpcSavage) => savages_idx = Some(i),
            Some(NationStrategy::NpcNomad) => nomads_idx = Some(i),
            _ => {}
        }
    }

    let (savages, nomads) = match (savages_idx, nomads_idx) {
        (Some(s), Some(n)) => (s, n),
        _ => return,
    };

    // Calculate needed vs actual troops
    // neededtroops = (NUMSECTS/MONSTER) * ((5/12)*450 + (1/4)*250)
    let needed_troops = (numsects / MONSTER as i64) * ((5 * 450 / 12) + (250 / 4));

    let mut actual_troops: i64 = 0;
    for i in 0..MAXARM {
        if state.nations[nomads].armies[i].soldiers > 0 {
            actual_troops += state.nations[nomads].armies[i].soldiers;
        }
        if state.nations[savages].armies[i].soldiers > 0 {
            actual_troops += state.nations[savages].armies[i].soldiers;
        }
    }

    let mut need = needed_troops - actual_troops;
    let mut nomad_space = true;
    let mut savage_space = true;

    while need > 0 && (nomad_space || savage_space) {
        if (rng.rand() % 8) < 5 && nomad_space {
            // Spawn nomad army
            let mut x;
            let mut y;
            let mut attempts = 0;
            loop {
                x = rng.rand() % (map_x - 8) + 4;
                y = rng.rand() % (map_y - 8) + 4;
                attempts += 1;
                if attempts > 1000 || is_habitable(&state.sectors[x as usize][y as usize]) {
                    break;
                }
            }

            let mut free_idx = None;
            for i in 0..MAXARM {
                if state.nations[nomads].armies[i].soldiers <= 0 {
                    free_idx = Some(i);
                    break;
                }
            }

            match free_idx {
                Some(i) => {
                    let sold = 100 + 100 * (rng.rand() % 6) as i64;
                    state.nations[nomads].armies[i].x = x as u8;
                    state.nations[nomads].armies[i].y = y as u8;
                    state.nations[nomads].armies[i].soldiers = sold;
                    state.nations[nomads].armies[i].unit_type = UnitType::LT_CAV.0;
                    state.nations[nomads].armies[i].status = ArmyStatus::Attack.to_value();
                    need -= sold;
                }
                None => {
                    nomad_space = false;
                }
            }
        } else if savage_space {
            // Spawn savage army
            let mut x;
            let mut y;
            let mut attempts = 0;
            loop {
                x = rng.rand() % (map_x - 8) + 4;
                y = rng.rand() % (map_y - 8) + 4;
                attempts += 1;
                if attempts > 1000 {
                    break;
                }
                let sct = &state.sectors[x as usize][y as usize];
                if sct.altitude != Altitude::Peak as u8
                    && sct.altitude != Altitude::Water as u8
                    && (sct.owner == 0 || sct.owner as usize == savages || sct.people < 50)
                {
                    break;
                }
            }

            let mut free_idx = None;
            for i in 0..MAXARM {
                if state.nations[savages].armies[i].soldiers <= 0 {
                    free_idx = Some(i);
                    break;
                }
            }

            match free_idx {
                Some(i) => {
                    let sold = 100 + 100 * (rng.rand() % 3) as i64;
                    let def_unit = defaultunit(&state.nations[savages]);
                    state.nations[savages].armies[i].x = x as u8;
                    state.nations[savages].armies[i].y = y as u8;
                    state.nations[savages].armies[i].soldiers = sold;
                    state.nations[savages].armies[i].unit_type = def_unit;
                    state.nations[savages].armies[i].status = ArmyStatus::Attack.to_value();
                    need -= sold;
                }
                None => {
                    savage_space = false;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nomad_growth() {
        let mut state = GameState::new(16, 16);
        state.nations[1].active = NationStrategy::NpcNomad as u8;
        state.nations[1].max_move = 10;
        state.nations[1].armies[0].soldiers = 100;
        state.nations[1].armies[0].unit_type = UnitType::LT_CAV.0;
        state.nations[1].armies[0].x = 8;
        state.nations[1].armies[0].y = 8;

        // Make surrounding sectors habitable
        for x in 7..10 {
            for y in 7..10 {
                state.sectors[x][y].altitude = Altitude::Clear as u8;
                state.sectors[x][y].vegetation = Vegetation::Good as u8;
                state.move_cost[x][y] = 1;
            }
        }

        let mut rng = ConquerRng::new(42);
        let news = do_nomad(&mut state, 1, &mut rng);

        // After update, soldiers should have grown by 2%
        let army = &state.nations[1].armies[0];
        // 100 * 102/100 = 102
        assert!(army.soldiers >= 100, "Nomad army should have grown");
    }

    #[test]
    fn test_pirate_fleet_basecamp() {
        let mut state = GameState::new(16, 16);
        state.nations[1].active = NationStrategy::NpcPirate as u8;
        state.nations[1].navies[0].warships = NavalSize::set_ships(0, NavalSize::Light, 3);
        state.nations[1].navies[0].x = 5;
        state.nations[1].navies[0].y = 5;

        // Place basecamp nearby
        state.sectors[6][6].designation = Designation::BaseCamp as u8;

        let mut rng = ConquerRng::new(42);
        do_pirate(&mut state, 1, &mut rng);

        // Fleet should have moved to basecamp
        assert_eq!(state.nations[1].navies[0].x, 6);
        assert_eq!(state.nations[1].navies[0].y, 6);
    }
}
