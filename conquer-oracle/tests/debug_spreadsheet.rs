/// Debug: compare spreadsheet() output with C expected values
use conquer_core::*;
use conquer_engine::economy::spreadsheet;
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

/// Compare spreadsheet on initial state (before any turn processing)
#[test]
fn debug_spreadsheet_initial_state() {
    let t1 = match load_snapshot("turn1.json") {
        Some(s) => s,
        None => return,
    };
    let gs = t1.to_game_state();

    eprintln!("\n=== SPREADSHEET ON INITIAL STATE ===");
    for country in 1..=15 {
        let spread = spreadsheet(&gs, country);
        let ntn = &gs.nations[country];
        eprintln!("Nation {} ({}):", country, ntn.name);
        eprintln!(
            "  sectors={} civilians={} tax_rate={}",
            spread.sectors, spread.civilians, ntn.tax_rate
        );
        eprintln!(
            "  gold: start={} revenue={} (food={} metal={} jewel={} city={} cap={} other={})",
            ntn.treasury_gold,
            spread.gold - ntn.treasury_gold,
            spread.rev_food,
            spread.rev_metal,
            spread.rev_jewels,
            spread.rev_city,
            spread.rev_cap,
            spread.rev_other
        );
        eprintln!(
            "  food: start={} prod={}",
            ntn.total_food,
            spread.food - ntn.total_food
        );
        eprintln!(
            "  metal: start={} prod={}",
            ntn.metals,
            spread.metal - ntn.metals
        );
        eprintln!(
            "  jewels: start={} prod={}",
            ntn.jewels,
            spread.jewels - ntn.jewels
        );
    }
}

/// Check NPC enlistment spending for a single nation
#[test]
fn debug_npc_enlistment_spending() {
    let t1 = match load_snapshot("turn1.json") {
        Some(s) => s,
        None => return,
    };
    let mut gs = t1.to_game_state();
    let mut rng = ConquerRng::new(42);

    // Pick nation 3 (bobland) which has the biggest gold discrepancy
    let ni = 3;
    let gold_before = gs.nations[ni].treasury_gold;
    let mil_before = gs.nations[ni].total_mil;

    eprintln!(
        "\n=== NPC NATION_RUN FOR NATION {} ({}) ===",
        ni, gs.nations[ni].name
    );
    eprintln!(
        "Before: gold={} mil={} active={} class={}",
        gold_before, mil_before, gs.nations[ni].active, gs.nations[ni].class
    );
    eprintln!("Powers: {}", gs.nations[ni].powers);
    eprintln!("Diplomacy: {:?}", &gs.nations[ni].diplomacy[..16]);

    // Count armies before
    let mut army_count = 0;
    let mut total_sold = 0i64;
    for a in 0..MAXARM {
        if gs.nations[ni].armies[a].soldiers > 0 {
            army_count += 1;
            total_sold += gs.nations[ni].armies[a].soldiers;
            eprintln!(
                "  Army[{:2}]: ({},{}) type={} stat={} sold={} move={}",
                a,
                gs.nations[ni].armies[a].x,
                gs.nations[ni].armies[a].y,
                gs.nations[ni].armies[a].unit_type,
                gs.nations[ni].armies[a].status,
                gs.nations[ni].armies[a].soldiers,
                gs.nations[ni].armies[a].movement
            );
        }
    }
    eprintln!("Total: {} armies, {} soldiers", army_count, total_sold);

    // Need to consume RNG for nation ordering first — but for isolated test, just run directly
    let news = conquer_engine::npc::nation_run(&mut gs, ni, &mut rng);

    let gold_after = gs.nations[ni].treasury_gold;
    let mil_after = gs.nations[ni].total_mil;
    eprintln!("\nAfter nation_run:");
    eprintln!(
        "  gold: {} -> {} (spent {})",
        gold_before,
        gold_after,
        gold_before - gold_after
    );

    let mut army_count2 = 0;
    let mut total_sold2 = 0i64;
    for a in 0..MAXARM {
        if gs.nations[ni].armies[a].soldiers > 0 {
            army_count2 += 1;
            total_sold2 += gs.nations[ni].armies[a].soldiers;
        }
    }
    eprintln!("  armies: {} -> {}", army_count, army_count2);
    eprintln!(
        "  soldiers: {} -> {} (change: {})",
        total_sold,
        total_sold2,
        total_sold2 - total_sold
    );
    if !news.is_empty() {
        for n in &news[..news.len().min(5)] {
            eprintln!("  news: {}", n);
        }
    }
}
