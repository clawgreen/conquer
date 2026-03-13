/// Debug test: trace NPC army movement for a single nation to find why armies don't move.
use conquer_core::*;
use conquer_core::constants::*;
use conquer_core::enums::*;
use conquer_engine::rng::ConquerRng;
use conquer_oracle::OracleSnapshot;
use std::fs;
use std::path::PathBuf;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn load_snapshot(name: &str) -> Option<OracleSnapshot> {
    let path = project_root().join("oracle/snapshots/seed42_turns").join(name);
    if !path.exists() { return None; }
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

#[test]
fn debug_army_status_all_nations() {
    let t1 = match load_snapshot("turn1.json") {
        Some(s) => s,
        None => { eprintln!("No snapshot"); return; },
    };
    let gs = t1.to_game_state();

    eprintln!("\n=== INITIAL ARMY STATUS ALL NATIONS ===");
    for n in 1..16 {
        let nation = &gs.nations[n];
        let mut alive = 0;
        let mut total_soldiers = 0i64;
        let mut has_movement = 0;
        for i in 0..MAXARM {
            let a = &nation.armies[i];
            if a.soldiers > 0 {
                alive += 1;
                total_soldiers += a.soldiers;
                if a.movement > 0 { has_movement += 1; }
            }
        }
        let sectors: usize = (0..gs.world.map_x as usize)
            .flat_map(|x| (0..gs.world.map_y as usize).map(move |y| (x, y)))
            .filter(|&(x, y)| gs.sectors[x][y].owner == n as u8)
            .count();
        eprintln!("  Nation {:2} ({:12}): {} armies ({} soldiers), {} with move>0, {} sectors, max_move={}",
            n, nation.name, alive, total_soldiers, has_movement, sectors, nation.max_move);
    }
}

#[test]
fn debug_nation_run_trace() {
    let t1 = match load_snapshot("turn1.json") {
        Some(s) => s,
        None => return,
    };
    let mut gs = t1.to_game_state();
    let mut rng = ConquerRng::new(42);

    // Consume 19 RNG values for nation ordering
    for _ in 0..19 { rng.rand(); }

    // First nation in C order is 6 (fung)
    let nation_idx = 6;
    eprintln!("\n=== RUNNING NATION {} ({}) ===", nation_idx, gs.nations[nation_idx].name);

    // Armies before
    let mut before = Vec::new();
    for i in 0..MAXARM {
        let a = &gs.nations[nation_idx].armies[i];
        if a.soldiers > 0 {
            before.push((i, a.x, a.y, a.status, a.unit_type, a.soldiers, a.movement));
            eprintln!("  Before Army[{:2}]: ({},{}) type={} stat={} sold={} move={}",
                i, a.x, a.y, a.unit_type, a.status, a.soldiers, a.movement);
        }
    }

    let before_sectors: usize = (0..gs.world.map_x as usize)
        .flat_map(|x| (0..gs.world.map_y as usize).map(move |y| (x, y)))
        .filter(|&(x, y)| gs.sectors[x][y].owner == nation_idx as u8)
        .count();
    eprintln!("Before sectors: {}", before_sectors);

    // Run nation
    let news = conquer_engine::npc::nation_run(&mut gs, nation_idx, &mut rng);

    // Armies after
    let mut moved = 0;
    let mut stat_changed = 0;
    for i in 0..MAXARM {
        let a = &gs.nations[nation_idx].armies[i];
        if a.soldiers > 0 {
            if let Some(&(_, bx, by, bstat, _, _, _)) = before.iter().find(|&&(idx, ..)| idx == i) {
                if a.x != bx || a.y != by {
                    moved += 1;
                    eprintln!("  MOVED Army[{:2}]: ({},{})->({},{}) stat={}->{}",
                        i, bx, by, a.x, a.y, bstat, a.status);
                } else if a.status != bstat {
                    stat_changed += 1;
                    eprintln!("  STAT  Army[{:2}]: ({},{}) stat={}->{}",
                        i, a.x, a.y, bstat, a.status);
                }
            }
        }
    }

    let after_sectors: usize = (0..gs.world.map_x as usize)
        .flat_map(|x| (0..gs.world.map_y as usize).map(move |y| (x, y)))
        .filter(|&(x, y)| gs.sectors[x][y].owner == nation_idx as u8)
        .count();

    eprintln!("After: {} moved, {} stat changed, sectors {}->{}",
        moved, stat_changed, before_sectors, after_sectors);
    if !news.is_empty() {
        eprintln!("News: {:?}", &news[..news.len().min(5)]);
    }
}

#[test]
fn debug_full_turn_sector_trace() {
    let t1 = match load_snapshot("turn1.json") {
        Some(s) => s,
        None => return,
    };
    let mut gs = t1.to_game_state();
    let initial = t1.to_game_state();
    let mut rng = ConquerRng::new(42);

    eprintln!("\n=== FULL TURN SECTOR TRACE ===");

    // Run the full turn
    conquer_engine::turn::update_turn(&mut gs, &mut rng);

    // For each nation, count sector changes
    let map_x = gs.world.map_x as usize;
    let map_y = gs.world.map_y as usize;
    for n in 1..16 {
        let mut gained = 0;
        let mut lost = 0;
        for x in 0..map_x {
            for y in 0..map_y {
                let was_mine = initial.sectors[x][y].owner == n as u8;
                let is_mine = gs.sectors[x][y].owner == n as u8;
                if !was_mine && is_mine { gained += 1; }
                if was_mine && !is_mine { lost += 1; }
            }
        }
        if gained > 0 || lost > 0 {
            let total: usize = (0..map_x)
                .flat_map(|x| (0..map_y).map(move |y| (x, y)))
                .filter(|&(x, y)| gs.sectors[x][y].owner == n as u8)
                .count();
            eprintln!("  Nation {:2} ({:12}): gained={:2} lost={:2} total={}",
                n, gs.nations[n].name, gained, lost, total);
        }
    }
    // Also check monsters
    for n in 31..35 {
        if n >= gs.nations.len() { continue; }
        let mut gained = 0;
        let mut lost = 0;
        for x in 0..map_x {
            for y in 0..map_y {
                let was_mine = initial.sectors[x][y].owner == n as u8;
                let is_mine = gs.sectors[x][y].owner == n as u8;
                if !was_mine && is_mine { gained += 1; }
                if was_mine && !is_mine { lost += 1; }
            }
        }
        if gained > 0 || lost > 0 {
            eprintln!("  Nation {:2} ({:12}): gained={:2} lost={:2}",
                n, gs.nations[n].name, gained, lost);
        }
    }
}

#[test]
fn debug_move_costs() {
    let t1 = match load_snapshot("turn1.json") {
        Some(s) => s,
        None => return,
    };
    let mut gs = t1.to_game_state();

    // nation 1 (argos, Human)
    conquer_engine::movement::update_move_costs(&mut gs, 'H', 1);

    let cx = gs.nations[1].cap_x as i32;
    let cy = gs.nations[1].cap_y as i32;
    eprintln!("\n=== MOVE COSTS FOR NATION 1 (argos, cap={},{}) ===", cx, cy);
    
    let mut passable = 0;
    let mut impassable = 0;
    let mut zero = 0;
    let map_x = gs.world.map_x as usize;
    let map_y = gs.world.map_y as usize;
    for x in 0..map_x {
        for y in 0..map_y {
            let cost = gs.move_cost[x][y];
            if cost > 0 { passable += 1; }
            else if cost < 0 { impassable += 1; }
            else { zero += 1; }
        }
    }
    eprintln!("Passable: {}, Impassable: {}, Zero(water?): {}", passable, impassable, zero);
    
    // Show costs near capitol
    for dx in -4..=4i32 {
        let mut row = String::new();
        for dy in -4..=4i32 {
            let x = cx + dx;
            let y = cy + dy;
            if gs.on_map(x, y) {
                let cost = gs.move_cost[x as usize][y as usize];
                row += &format!("{:4}", cost);
            } else {
                row += "   .";
            }
        }
        eprintln!("  row {}: {}", cx + dx, row);
    }
    eprintln!("Max move: {}", gs.nations[1].max_move);
}
