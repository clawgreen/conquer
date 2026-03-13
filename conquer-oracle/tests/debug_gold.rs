/// Debug test: trace gold flow per-nation to find economic drift source.
use conquer_core::*;
use conquer_engine::rng::ConquerRng;
use conquer_oracle::OracleSnapshot;
use std::fs;
use std::path::PathBuf;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn load_snapshot(name: &str) -> Option<OracleSnapshot> {
    let path = project_root()
        .join("oracle/snapshots/seed42_turns")
        .join(name);
    if !path.exists() {
        return None;
    }
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Trace gold flow step-by-step for one turn.
/// Runs each phase of update_turn manually and records gold after each.
#[test]
fn debug_gold_flow_per_phase() {
    let t1 = match load_snapshot("turn1.json") {
        Some(s) => s,
        None => {
            eprintln!("No snapshot");
            return;
        }
    };
    let t2 = match load_snapshot("turn2.json") {
        Some(s) => s,
        None => {
            eprintln!("No turn2");
            return;
        }
    };

    let mut gs = t1.to_game_state();
    let mut rng = ConquerRng::new(42);

    // Record gold at each phase
    let initial: Vec<i64> = (0..NTOTAL).map(|i| gs.nations[i].treasury_gold).collect();

    eprintln!("\n=== GOLD FLOW TRACE (per phase) ===");
    eprintln!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Phase", "N1", "N2", "N3", "N12", "N14"
    );
    eprintln!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "initial",
        gs.nations[1].treasury_gold,
        gs.nations[2].treasury_gold,
        gs.nations[3].treasury_gold,
        gs.nations[12].treasury_gold,
        gs.nations[14].treasury_gold
    );

    // Phase 1: updexecs (NPC runs) - replicate turn.rs logic
    let mut execed = vec![false; NTOTAL];
    for country in 0..NTOTAL {
        let strat = NationStrategy::from_value(gs.nations[country].active);
        if !strat.map_or(false, |s| s.is_nation()) {
            execed[country] = true;
        }
    }
    let active_count = execed.iter().filter(|&&e| !e).count();
    for loop_idx in 0..active_count {
        let remaining = active_count - loop_idx;
        let mut number = (rng.rand() % remaining as i32) + 1;
        let mut country = 0;
        for c in 0..NTOTAL {
            if !execed[c] {
                number -= 1;
                if number == 0 {
                    country = c;
                    execed[c] = true;
                    break;
                }
            }
        }
        if gs.nations[country].active == 0 {
            continue;
        }
        let strat = NationStrategy::from_value(gs.nations[country].active);
        if strat.map_or(false, |s| s.is_npc()) {
            let _news = conquer_engine::npc::nation_run(&mut gs, country, &mut rng);
        }
        // Leader check
        let leader_type =
            conquer_engine::utils::getleader(gs.nations[country].class).wrapping_sub(1);
        let mut has_leader = false;
        for armynum in 0..MAXARM {
            if gs.nations[country].armies[armynum].unit_type == leader_type
                && gs.nations[country].armies[armynum].soldiers > 0
            {
                has_leader = true;
                break;
            }
        }
        if !has_leader {
            if rng.rand() % 100 < 30 {
                let next_type = leader_type + 1;
                for armynum in 0..MAXARM {
                    if gs.nations[country].armies[armynum].unit_type == next_type
                        && gs.nations[country].armies[armynum].soldiers > 0
                    {
                        gs.nations[country].armies[armynum].unit_type = leader_type;
                        break;
                    }
                }
            }
        }
        conquer_engine::economy::move_people_single_nation(&mut gs, country);
    }
    // Zero tmil/tships + spell decay
    for country in 1..NTOTAL {
        let strat = NationStrategy::from_value(gs.nations[country].active);
        if !strat.map_or(false, |s| s.is_nation()) {
            continue;
        }
        gs.nations[country].total_ships = 0;
        gs.nations[country].total_mil = 0;
        if rng.rand() % 4 == 0 {
            gs.nations[country].spell_points /= 2;
        }
    }

    eprintln!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "updexecs",
        gs.nations[1].treasury_gold,
        gs.nations[2].treasury_gold,
        gs.nations[3].treasury_gold,
        gs.nations[12].treasury_gold,
        gs.nations[14].treasury_gold
    );

    // Phase 2: monsters
    let _mn = conquer_engine::monster::update_monsters(&mut gs, &mut rng);
    eprintln!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "monsters",
        gs.nations[1].treasury_gold,
        gs.nations[2].treasury_gold,
        gs.nations[3].treasury_gold,
        gs.nations[12].treasury_gold,
        gs.nations[14].treasury_gold
    );

    // Phase 3: combat
    let _cr = conquer_engine::combat::run_combat(&mut gs, &mut rng);
    eprintln!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "combat",
        gs.nations[1].treasury_gold,
        gs.nations[2].treasury_gold,
        gs.nations[3].treasury_gold,
        gs.nations[12].treasury_gold,
        gs.nations[14].treasury_gold
    );

    // Phase 4: capture
    let _cn = conquer_engine::movement::update_capture(&mut gs, &mut rng);
    eprintln!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "capture",
        gs.nations[1].treasury_gold,
        gs.nations[2].treasury_gold,
        gs.nations[3].treasury_gold,
        gs.nations[12].treasury_gold,
        gs.nations[14].treasury_gold
    );

    // Phase 5: trade
    let _tn = conquer_engine::trade::process_trades_gs(&mut gs);
    eprintln!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "trade",
        gs.nations[1].treasury_gold,
        gs.nations[2].treasury_gold,
        gs.nations[3].treasury_gold,
        gs.nations[12].treasury_gold,
        gs.nations[14].treasury_gold
    );

    // Phase 6: updmil (movement reset, maintenance)
    conquer_engine::economy::updmil(&mut gs, &mut rng);
    eprintln!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "updmil",
        gs.nations[1].treasury_gold,
        gs.nations[2].treasury_gold,
        gs.nations[3].treasury_gold,
        gs.nations[12].treasury_gold,
        gs.nations[14].treasury_gold
    );

    // Phase 7: random events
    let _en = conquer_engine::events::process_events_gs(&mut gs, &mut rng);
    eprintln!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "events",
        gs.nations[1].treasury_gold,
        gs.nations[2].treasury_gold,
        gs.nations[3].treasury_gold,
        gs.nations[12].treasury_gold,
        gs.nations[14].treasury_gold
    );

    // Phase 8: updsectors (tax, production, pop growth)
    conquer_engine::economy::updsectors(&mut gs, &mut rng);
    eprintln!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "updsectors",
        gs.nations[1].treasury_gold,
        gs.nations[2].treasury_gold,
        gs.nations[3].treasury_gold,
        gs.nations[12].treasury_gold,
        gs.nations[14].treasury_gold
    );

    // Phase 9: updcomodities (food consumption, jewel buying)
    // Trace jewel-buy for nations that'll be affected
    for &ni in &[3usize, 12] {
        let gold = gs.nations[ni].treasury_gold;
        let jewels = gs.nations[ni].jewels;
        let gt = 10i64; // GOLDTHRESH
        let excess = gold - gt * jewels;
        eprintln!(
            "  Pre-updcomod N{}: gold={} jewels={} excess={} (will buy? {})",
            ni,
            gold,
            jewels,
            excess,
            excess > 0
        );
    }
    conquer_engine::economy::updcomodities(&mut gs, &mut rng);
    for &ni in &[3usize, 12] {
        let gold = gs.nations[ni].treasury_gold;
        let jewels = gs.nations[ni].jewels;
        eprintln!("  Post-updcomod N{}: gold={} jewels={}", ni, gold, jewels);
    }
    eprintln!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "updcomod",
        gs.nations[1].treasury_gold,
        gs.nations[2].treasury_gold,
        gs.nations[3].treasury_gold,
        gs.nations[12].treasury_gold,
        gs.nations[14].treasury_gold
    );

    // Phase 10: leaders
    conquer_engine::economy::update_leaders(&mut gs, &mut rng);
    eprintln!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "leaders",
        gs.nations[1].treasury_gold,
        gs.nations[2].treasury_gold,
        gs.nations[3].treasury_gold,
        gs.nations[12].treasury_gold,
        gs.nations[14].treasury_gold
    );

    // Phase 11: score + att_base + att_bonus
    conquer_engine::turn::calculate_scores_gs(&mut gs);
    conquer_engine::economy::att_base_gs(&mut gs, &mut rng);
    conquer_engine::economy::att_bonus_gs(&mut gs);
    eprintln!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "att_base",
        gs.nations[1].treasury_gold,
        gs.nations[2].treasury_gold,
        gs.nations[3].treasury_gold,
        gs.nations[12].treasury_gold,
        gs.nations[14].treasury_gold
    );

    // Expected from C oracle
    let en = t2.nations.as_ref().unwrap();
    eprintln!(
        "{:>12} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "C expected", en[1].tgold, en[2].tgold, en[3].tgold, en[12].tgold, en[14].tgold
    );

    eprintln!("\n=== GOLD DELTAS (Rust - C) ===");
    for i in [1, 2, 3, 12, 14] {
        let diff = gs.nations[i].treasury_gold - en[i].tgold;
        eprintln!(
            "  Nation {:2} ({}): Rust={} C={} diff={:+}",
            i, gs.nations[i].name, gs.nations[i].treasury_gold, en[i].tgold, diff
        );
    }
}
