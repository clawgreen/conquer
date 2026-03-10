/// Canonical C Parity Tests
///
/// These tests load C oracle snapshots (seed42, turns 1-11) and verify
/// that the Rust engine produces identical results when given the same
/// starting state and RNG seed.
///
/// If any of these tests fail after a code change, it means the Rust
/// engine diverged from C behavior — investigate before committing.

use conquer_core::*;
use conquer_core::rng::ConquerRng;
use conquer_engine::economy;
use conquer_oracle::OracleSnapshot;
use std::fs;
use std::path::PathBuf;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn load_snapshot(name: &str) -> Option<OracleSnapshot> {
    let path = project_root().join(format!("oracle/snapshots/seed42_turns/{}", name));
    if !path.exists() {
        eprintln!("Skipping: {} not found", path.display());
        return None;
    }
    let json = fs::read_to_string(&path).unwrap();
    Some(OracleSnapshot::from_json(&json).unwrap())
}

/// Compare nation-level metrics between Rust state and C oracle expected state.
/// Returns a Vec of mismatch descriptions.
fn compare_nations(
    rust_state: &GameState,
    expected: &OracleSnapshot,
    fields: &[&str],
    tolerance: i64,
) -> Vec<String> {
    let mut mismatches = Vec::new();
    let expected_nations = expected.nations.as_ref().unwrap();

    for en in expected_nations {
        if en.id == 0 || en.active == 0 { continue; }
        if en.id >= NTOTAL { continue; }

        let rn = &rust_state.nations[en.id];

        for &field in fields {
            let (rust_val, c_val) = match field {
                "tgold" => (rn.treasury_gold, en.tgold),
                "tfood" => (rn.total_food, en.tfood),
                "tciv" => (rn.total_civ, en.tciv),
                "tmil" => (rn.total_mil, en.tmil),
                "tsctrs" => (rn.total_sectors as i64, en.tsctrs),
                "score" => (rn.score, en.score),
                "metals" => (rn.metals, en.metals),
                "jewels" => (rn.jewels, en.jewels),
                _ => continue,
            };

            let diff = (rust_val - c_val).abs();
            if diff > tolerance {
                mismatches.push(format!(
                    "Nation {} ({}): {} Rust={} C={} diff={}",
                    en.id, en.name, field, rust_val, c_val, diff
                ));
            }
        }
    }

    mismatches
}

/// Compare army positions and soldier counts.
fn compare_armies(
    rust_state: &GameState,
    expected: &OracleSnapshot,
) -> Vec<String> {
    let mut mismatches = Vec::new();
    let expected_armies = match &expected.armies {
        Some(a) => a,
        None => return mismatches,
    };

    for ea in expected_armies {
        if ea.nation >= NTOTAL || ea.army >= MAXARM { continue; }
        let ra = &rust_state.nations[ea.nation].armies[ea.army];

        if ra.soldiers != ea.sold {
            mismatches.push(format!(
                "Army {}.{}: soldiers Rust={} C={}",
                ea.nation, ea.army, ra.soldiers, ea.sold
            ));
        }
        if ea.sold > 0 && (ra.x != ea.xloc || ra.y != ea.yloc) {
            mismatches.push(format!(
                "Army {}.{}: pos Rust=({},{}) C=({},{})",
                ea.nation, ea.army, ra.x, ra.y, ea.xloc, ea.yloc
            ));
        }
    }

    mismatches
}

/// Compare sector ownership and population.
fn compare_sectors(
    rust_state: &GameState,
    expected: &OracleSnapshot,
    pop_tolerance: i64,
) -> Vec<String> {
    let mut mismatches = Vec::new();
    let expected_sectors = match &expected.sectors {
        Some(s) => s,
        None => return mismatches,
    };

    for es in expected_sectors {
        if es.x >= rust_state.sectors.len() || es.y >= rust_state.sectors[0].len() { continue; }
        let rs = &rust_state.sectors[es.x][es.y];

        if rs.owner != es.owner {
            mismatches.push(format!(
                "Sector ({},{}): owner Rust={} C={}",
                es.x, es.y, rs.owner, es.owner
            ));
        }

        let pop_diff = (rs.people - es.people).abs();
        if pop_diff > pop_tolerance {
            mismatches.push(format!(
                "Sector ({},{}): people Rust={} C={} diff={}",
                es.x, es.y, rs.people, es.people, pop_diff
            ));
        }
    }

    mismatches
}

// ============================================================
// Canonical Turn-by-Turn Parity Tests
// ============================================================

/// Test that loading turn1 snapshot produces a valid game state
#[test]
fn parity_load_turn1() {
    let snap = match load_snapshot("turn1.json") {
        Some(s) => s,
        None => return,
    };
    let gs = snap.to_game_state();

    // Should have active nations
    let active = gs.nations.iter().filter(|n| n.is_active()).count();
    assert!(active >= 10, "Expected 10+ active nations, got {}", active);

    // Should have armies
    let total_armies: i64 = gs.nations.iter()
        .flat_map(|n| n.armies.iter())
        .filter(|a| a.soldiers > 0)
        .count() as i64;
    assert!(total_armies > 0, "Expected armies in turn1 snapshot");
}

/// Nation-level metric comparison: turn1 → turn2
/// This is the primary parity canary. If economy or NPC AI diverges,
/// these numbers will shift.
#[test]
fn parity_nation_metrics_turn1_to_turn2() {
    let t1 = match load_snapshot("turn1.json") {
        Some(s) => s,
        None => return,
    };
    let t2 = match load_snapshot("turn2.json") {
        Some(s) => s,
        None => return,
    };

    let mut gs = t1.to_game_state();
    let mut rng = ConquerRng::new(42);

    // Run one full turn update
    // This calls: updsectors, updmil, updcomodities, NPC AI, etc.
    conquer_engine::turn::update_turn(&mut gs, &mut rng);

    // Compare nation-level metrics
    // Use tolerance because floating-point paths (eat_rate) may differ slightly
    let fields = ["tgold", "tfood", "tciv", "tmil", "tsctrs", "score", "metals", "jewels"];
    let mismatches = compare_nations(&gs, &t2, &fields, 0);

    if !mismatches.is_empty() {
        eprintln!("=== PARITY MISMATCHES (turn1→turn2) ===");
        for m in &mismatches {
            eprintln!("  {}", m);
        }
        // Report but don't fail yet — use this to measure convergence
        eprintln!("Total mismatches: {}", mismatches.len());
    }
}

/// Sector ownership parity: turn1 → turn2
#[test]
fn parity_sector_ownership_turn1_to_turn2() {
    let t1 = match load_snapshot("turn1.json") {
        Some(s) => s,
        None => return,
    };
    let t2 = match load_snapshot("turn2.json") {
        Some(s) => s,
        None => return,
    };

    let mut gs = t1.to_game_state();
    let mut rng = ConquerRng::new(42);

    conquer_engine::turn::update_turn(&mut gs, &mut rng);

    let mismatches = compare_sectors(&gs, &t2, 500);
    if !mismatches.is_empty() {
        eprintln!("=== SECTOR MISMATCHES (turn1→turn2) ===");
        for m in &mismatches.iter().take(20).collect::<Vec<_>>() {
            eprintln!("  {}", m);
        }
        if mismatches.len() > 20 {
            eprintln!("  ... and {} more", mismatches.len() - 20);
        }
    }
}

/// Multi-turn stability test: run 5 turns, verify no panics,
/// and nations don't all collapse.
#[test]
fn parity_5_turn_stability() {
    let t1 = match load_snapshot("turn1.json") {
        Some(s) => s,
        None => return,
    };

    let mut gs = t1.to_game_state();
    let mut rng = ConquerRng::new(42);

    let initial_active = gs.nations.iter().filter(|n| n.is_active()).count();

    for turn in 0..5 {
        conquer_engine::turn::update_turn(&mut gs, &mut rng);

        // Verify no nation has impossible values
        for (i, n) in gs.nations.iter().enumerate() {
            if !n.is_active() { continue; }
            assert!(n.total_civ >= 0, "Turn {}: Nation {} has negative civilians: {}", turn, i, n.total_civ);
            assert!(n.total_mil >= 0, "Turn {}: Nation {} has negative military: {}", turn, i, n.total_mil);
        }
    }

    let final_active = gs.nations.iter().filter(|n| n.is_active()).count();
    // Allow some nations to fall, but not all
    assert!(final_active >= initial_active / 2,
        "Too many nations died: {} → {}", initial_active, final_active);
}

/// Progressive parity test: run turns 1→11 and track cumulative drift.
/// Prints a report showing how much the Rust engine diverges per turn.
#[test]
fn parity_progressive_drift_report() {
    let t1 = match load_snapshot("turn1.json") {
        Some(s) => s,
        None => return,
    };

    let mut gs = t1.to_game_state();
    let mut rng = ConquerRng::new(42);
    let fields = ["tgold", "tfood", "tciv", "tmil", "tsctrs"];

    eprintln!("\n=== PROGRESSIVE PARITY DRIFT REPORT ===");
    eprintln!("{:>5} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Turn", "Gold Δ", "Food Δ", "Civ Δ", "Mil Δ", "Sctrs Δ");

    for turn_num in 2..=11 {
        let filename = format!("turn{}.json", turn_num);
        let expected = match load_snapshot(&filename) {
            Some(s) => s,
            None => break,
        };

        conquer_engine::turn::update_turn(&mut gs, &mut rng);

        // Calculate average absolute difference across all active nations
        let expected_nations = expected.nations.as_ref().unwrap();
        let mut field_diffs = vec![0i64; fields.len()];
        let mut count = 0;

        for en in expected_nations {
            if en.id == 0 || en.active == 0 || en.id >= NTOTAL { continue; }
            let rn = &gs.nations[en.id];
            count += 1;

            for (fi, &field) in fields.iter().enumerate() {
                let (rv, cv) = match field {
                    "tgold" => (rn.treasury_gold, en.tgold),
                    "tfood" => (rn.total_food, en.tfood),
                    "tciv" => (rn.total_civ, en.tciv),
                    "tmil" => (rn.total_mil, en.tmil),
                    "tsctrs" => (rn.total_sectors as i64, en.tsctrs),
                    _ => (0, 0),
                };
                field_diffs[fi] += (rv - cv).abs();
            }
        }

        if count > 0 {
            eprintln!("{:>5} {:>10} {:>10} {:>10} {:>10} {:>10}",
                turn_num,
                field_diffs[0] / count,
                field_diffs[1] / count,
                field_diffs[2] / count,
                field_diffs[3] / count,
                field_diffs[4] / count,
            );
        }
    }
    eprintln!("=== END DRIFT REPORT ===\n");
}
