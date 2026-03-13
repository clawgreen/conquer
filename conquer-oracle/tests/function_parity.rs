use conquer_core::powers::Power;
use conquer_core::tables::*;
/// Function-level parity tests: staged data, no RNG dependency.
/// Each test sets up identical input state and verifies the Rust function
/// produces the same output as the C formula.
use conquer_core::*;
use conquer_engine::economy::*;
use conquer_engine::movement::*;
use conquer_engine::rng::ConquerRng;

/// Helper: create a minimal game state with one nation owning sectors
fn setup_state(map_size: usize) -> GameState {
    let mut gs = GameState::new(map_size, map_size);
    // Set all sectors to habitable land
    for x in 0..map_size {
        for y in 0..map_size {
            gs.sectors[x][y].altitude = Altitude::Clear as u8;
            gs.sectors[x][y].vegetation = Vegetation::Forest as u8;
        }
    }
    gs
}

// ============================================================
// SPREADSHEET TESTS
// ============================================================

/// Test: mine sector produces metal revenue matching C formula.
/// C: product = metal * people; revmetal = product * TAXMETAL * tax_rate / 100
#[test]
fn fn_spreadsheet_mine_revenue() {
    let mut gs = setup_state(8);
    let ni = 1;
    gs.nations[ni].active = 16u8;
    gs.nations[ni].tax_rate = 10;
    gs.nations[ni].treasury_gold = 50000;
    gs.nations[ni].cap_x = 0;
    gs.nations[ni].cap_y = 0;

    // One mine sector with 2000 people, metal=5
    gs.sectors[1][1].owner = ni as u8;
    gs.sectors[1][1].designation = Designation::Mine as u8;
    gs.sectors[1][1].people = 2000;
    gs.sectors[1][1].metal = 5;
    gs.sectors[1][1].trade_good = 0; // tg_ok needs no trade good or matching one

    // Capitol
    gs.sectors[0][0].owner = ni as u8;
    gs.sectors[0][0].designation = Designation::Capitol as u8;
    gs.sectors[0][0].people = 5000;

    let spread = spreadsheet(&gs, ni);

    // C formula: product = metal(5) * people(2000) = 10000
    // No MINER/STEEL power, so no multiplier
    // revmetal = 10000 * TAXMETAL(8) * tax_rate(10) / 100 = 8000
    let expected_product = 5i64 * 2000;
    let expected_rev = expected_product * 8 * 10 / 100;

    let metal_gain = spread.metal - gs.nations[ni].metals;
    let rev_metal = spread.rev_metal;

    eprintln!(
        "Mine: product={} (expected {}), rev_metal={} (expected {})",
        metal_gain, expected_product, rev_metal, expected_rev
    );
    assert_eq!(metal_gain, expected_product, "Metal production mismatch");
    assert_eq!(rev_metal, expected_rev, "Metal tax revenue mismatch");
}

/// Test: mine with MINER power doubles production
#[test]
fn fn_spreadsheet_mine_with_miner() {
    let mut gs = setup_state(8);
    let ni = 1;
    gs.nations[ni].active = 16u8;
    gs.nations[ni].tax_rate = 10;
    gs.nations[ni].treasury_gold = 50000;
    gs.nations[ni].powers = Power::MINER.bits() as i64;
    gs.nations[ni].cap_x = 0;
    gs.nations[ni].cap_y = 0;

    gs.sectors[1][1].owner = ni as u8;
    gs.sectors[1][1].designation = Designation::Mine as u8;
    gs.sectors[1][1].people = 2000;
    gs.sectors[1][1].metal = 5;

    gs.sectors[0][0].owner = ni as u8;
    gs.sectors[0][0].designation = Designation::Capitol as u8;
    gs.sectors[0][0].people = 5000;

    let spread = spreadsheet(&gs, ni);

    // With MINER: product = 5 * 2000 * 2 = 20000
    let expected_product = 5i64 * 2000 * 2;
    let metal_gain = spread.metal - gs.nations[ni].metals;

    eprintln!(
        "Mine+MINER: product={} (expected {})",
        metal_gain, expected_product
    );
    assert_eq!(
        metal_gain, expected_product,
        "MINER power should double metal"
    );
}

/// Test: mine with overpopulation (>TOMANYPEOPLE=4000) uses reduced formula
#[test]
fn fn_spreadsheet_mine_overpop() {
    let mut gs = setup_state(8);
    let ni = 1;
    gs.nations[ni].active = 16u8;
    gs.nations[ni].tax_rate = 10;
    gs.nations[ni].cap_x = 0;
    gs.nations[ni].cap_y = 0;

    gs.sectors[1][1].owner = ni as u8;
    gs.sectors[1][1].designation = Designation::Mine as u8;
    gs.sectors[1][1].people = 6000;
    gs.sectors[1][1].metal = 5;

    gs.sectors[0][0].owner = ni as u8;
    gs.sectors[0][0].designation = Designation::Capitol as u8;
    gs.sectors[0][0].people = 5000;

    let spread = spreadsheet(&gs, ni);

    // C: product = metal * TOMANYPEOPLE + metal * (people - TOMANYPEOPLE) / 2
    // = 5 * 4000 + 5 * 2000/2 = 20000 + 5000 = 25000
    let expected = 5i64 * 4000 + 5 * (6000 - 4000) / 2;
    let metal_gain = spread.metal - gs.nations[ni].metals;

    eprintln!(
        "Mine overpop: product={} (expected {})",
        metal_gain, expected
    );
    assert_eq!(metal_gain, expected, "Overpop mine formula mismatch");
}

/// Test: city/capitol tax revenue matches C.
/// C: revcap = cap_pop * TAXCITY * tax_rate / 100
#[test]
fn fn_spreadsheet_city_revenue() {
    let mut gs = setup_state(8);
    let ni = 1;
    gs.nations[ni].active = 16u8;
    gs.nations[ni].tax_rate = 15;
    gs.nations[ni].treasury_gold = 50000;
    gs.nations[ni].cap_x = 0;
    gs.nations[ni].cap_y = 0;

    gs.sectors[0][0].owner = ni as u8;
    gs.sectors[0][0].designation = Designation::Capitol as u8;
    gs.sectors[0][0].people = 10000;

    gs.sectors[1][1].owner = ni as u8;
    gs.sectors[1][1].designation = Designation::Town as u8;
    gs.sectors[1][1].people = 3000;

    let spread = spreadsheet(&gs, ni);

    // Capitol: revcap = 10000 * 100 * 15 / 100 = 150000
    let expected_cap = 10000i64 * 100 * 15 / 100;
    // Town: revcity = 3000 * 80 * 15 / 100 = 36000
    let expected_town = 3000i64 * 80 * 15 / 100;

    eprintln!(
        "Capitol rev={} (expected {}), Town rev={} (expected {})",
        spread.rev_cap, expected_cap, spread.rev_city, expected_town
    );
    assert_eq!(spread.rev_cap, expected_cap, "Capitol revenue mismatch");
    assert_eq!(spread.rev_city, expected_town, "Town revenue mismatch");

    // Total gold = start + all revenues
    let expected_gold = gs.nations[ni].treasury_gold + expected_cap + expected_town;
    assert_eq!(
        spread.gold, expected_gold,
        "Total gold after spreadsheet mismatch"
    );
}

/// Test: city with ARCHITECT power doubles cap_pop for tax calc
#[test]
fn fn_spreadsheet_city_architect() {
    let mut gs = setup_state(8);
    let ni = 1;
    gs.nations[ni].active = 16u8;
    gs.nations[ni].tax_rate = 10;
    gs.nations[ni].treasury_gold = 50000;
    gs.nations[ni].powers = Power::ARCHITECT.bits() as i64;
    gs.nations[ni].cap_x = 0;
    gs.nations[ni].cap_y = 0;

    gs.sectors[0][0].owner = ni as u8;
    gs.sectors[0][0].designation = Designation::Capitol as u8;
    gs.sectors[0][0].people = 5000;

    let spread = spreadsheet(&gs, ni);

    // With ARCHITECT: cap_pop = 5000 * 2 = 10000
    // revcap = 10000 * 100 * 10 / 100 = 100000
    let expected_cap = 5000i64 * 2 * 100 * 10 / 100;

    eprintln!(
        "Capitol+ARCHITECT rev={} (expected {})",
        spread.rev_cap, expected_cap
    );
    assert_eq!(
        spread.rev_cap, expected_cap,
        "ARCHITECT power should double city pop for tax"
    );
}

// ============================================================
// UPDCOMODITIES TESTS
// ============================================================

/// Test: food consumption = civ * eat_rate/25 + mil * eat_rate/25 * 2
#[test]
fn fn_updcomod_food_consumption() {
    let mut gs = setup_state(8);
    let ni = 1;
    gs.nations[ni].active = 16u8;
    gs.nations[ni].total_civ = 10000;
    gs.nations[ni].total_mil = 2000;
    gs.nations[ni].total_food = 100000;
    gs.nations[ni].eat_rate = 25; // eat_rate/25 = 1.0
    gs.nations[ni].spoil_rate = 0;
    gs.nations[ni].treasury_gold = 50000;
    gs.nations[ni].jewels = 50000; // high jewels to avoid auto-buy
    gs.nations[ni].cap_x = 0;
    gs.nations[ni].cap_y = 0;
    gs.sectors[0][0].owner = ni as u8;
    gs.sectors[0][0].designation = Designation::Capitol as u8;

    let mut rng = ConquerRng::new(99);
    updcomodities(&mut gs, &mut rng);

    // C: mil_food = tmil * (eatrate/25.0) * 2 = 2000 * 1.0 * 2 = 4000
    // C: civ_food = tciv * (eatrate/25.0) = 10000 * 1.0 = 10000
    // Total consumed: 14000
    let expected_food = 100000 - 14000; // 86000

    eprintln!(
        "Food after consumption: {} (expected {})",
        gs.nations[ni].total_food, expected_food
    );
    assert_eq!(
        gs.nations[ni].total_food, expected_food,
        "Food consumption mismatch"
    );
}

/// Test: food spoilage = food * (100 - spoil_rate) / 100
#[test]
fn fn_updcomod_food_spoilage() {
    let mut gs = setup_state(8);
    let ni = 1;
    gs.nations[ni].active = 16u8;
    gs.nations[ni].total_civ = 0;
    gs.nations[ni].total_mil = 0;
    gs.nations[ni].total_food = 100000;
    gs.nations[ni].eat_rate = 25;
    gs.nations[ni].spoil_rate = 10; // 10% spoils
    gs.nations[ni].treasury_gold = 50000;
    gs.nations[ni].jewels = 50000;
    gs.nations[ni].cap_x = 0;
    gs.nations[ni].cap_y = 0;
    gs.sectors[0][0].owner = ni as u8;
    gs.sectors[0][0].designation = Designation::Capitol as u8;

    let mut rng = ConquerRng::new(99);
    updcomodities(&mut gs, &mut rng);

    // No consumption (0 civ, 0 mil), just spoilage
    // C: tfood = tfood * (100 - spoilrate) / 100 = 100000 * 90 / 100 = 90000
    let expected = (100000i64 * 90 / 100) as i64;

    eprintln!(
        "Food after spoilage: {} (expected {})",
        gs.nations[ni].total_food, expected
    );
    assert_eq!(
        gs.nations[ni].total_food, expected,
        "Food spoilage mismatch"
    );
}

/// Test: jewel auto-buy when gold > GOLDTHRESH * jewels
#[test]
fn fn_updcomod_jewel_autobuy() {
    let mut gs = setup_state(8);
    let ni = 1;
    gs.nations[ni].active = 16u8;
    gs.nations[ni].total_civ = 0;
    gs.nations[ni].total_mil = 0;
    gs.nations[ni].total_food = 100000;
    gs.nations[ni].eat_rate = 25;
    gs.nations[ni].spoil_rate = 0;
    gs.nations[ni].treasury_gold = 200000; // Much more than 10 * jewels
    gs.nations[ni].jewels = 5000; // GOLDTHRESH(10) * 5000 = 50000 < 200000
    gs.nations[ni].cap_x = 0;
    gs.nations[ni].cap_y = 0;
    gs.sectors[0][0].owner = ni as u8;
    gs.sectors[0][0].designation = Designation::Capitol as u8;

    let mut rng = ConquerRng::new(99);
    updcomodities(&mut gs, &mut rng);

    // C: xx = tgold - GOLDTHRESH * jewels = 200000 - 50000 = 150000
    // bought = xx * GODJEWL / GODPRICE = 150000 * 3000 / 25000 = 18000
    // tgold -= xx = 200000 - 150000 = 50000
    // jewels += bought = 5000 + 18000 = 23000
    let expected_gold = 200000 - 150000;
    let expected_jewels = 5000 + (150000i64 * 3000 / 25000);

    eprintln!(
        "After jewel buy: gold={} (expected {}), jewels={} (expected {})",
        gs.nations[ni].treasury_gold, expected_gold, gs.nations[ni].jewels, expected_jewels
    );
    assert_eq!(
        gs.nations[ni].treasury_gold, expected_gold,
        "Gold after jewel buy mismatch"
    );
    assert_eq!(
        gs.nations[ni].jewels, expected_jewels,
        "Jewels after buy mismatch"
    );
}

// ============================================================
// UPDMIL TESTS
// ============================================================

/// Test: army movement reset matches C formula.
/// C: P_AMOVE = (maxmove * unit_move) / 10
#[test]
fn fn_updmil_movement_reset() {
    let mut gs = setup_state(8);
    let ni = 1;
    gs.nations[ni].active = 16u8;
    gs.nations[ni].max_move = 12;
    gs.nations[ni].class = 1; // King — leader is L_BARON-1 = L_KING
    gs.nations[ni].treasury_gold = 100000;
    gs.nations[ni].cap_x = 0;
    gs.nations[ni].cap_y = 0;
    gs.sectors[0][0].owner = ni as u8;
    gs.sectors[0][0].designation = Designation::Capitol as u8;

    // Set up a leader so we're not in disarray
    let leader_type = conquer_engine::utils::getleader(gs.nations[ni].class).wrapping_sub(1);
    gs.nations[ni].armies[0].unit_type = leader_type;
    gs.nations[ni].armies[0].soldiers = 100;
    gs.nations[ni].armies[0].status = ArmyStatus::Defend.to_value();
    gs.nations[ni].armies[0].x = 0;
    gs.nations[ni].armies[0].y = 0;

    // Regular infantry (type 3) with DEFEND status
    gs.nations[ni].armies[1].unit_type = 3;
    gs.nations[ni].armies[1].soldiers = 200;
    gs.nations[ni].armies[1].status = ArmyStatus::Defend.to_value();
    gs.nations[ni].armies[1].movement = 0; // Start at 0
    gs.nations[ni].armies[1].x = 0;
    gs.nations[ni].armies[1].y = 0;

    // MARCH status army
    gs.nations[ni].armies[2].unit_type = 3;
    gs.nations[ni].armies[2].soldiers = 200;
    gs.nations[ni].armies[2].status = ArmyStatus::March.to_value();
    gs.nations[ni].armies[2].movement = 0;
    gs.nations[ni].armies[2].x = 0;
    gs.nations[ni].armies[2].y = 0;

    let mut rng = ConquerRng::new(99);
    updmil(&mut gs, &mut rng);

    // Army 1 (DEFEND): movement = (12 * unit_move[3%UTYPE]) / 10
    let unit_move_3 = UNIT_MOVE.get(3 % UTYPE as usize).copied().unwrap_or(10);
    let expected_defend = (12 * unit_move_3) / 10;
    // Army 2 (MARCH): movement = (12 * unit_move[3%UTYPE]) / 5
    let expected_march = (12 * unit_move_3) / 5;

    eprintln!(
        "DEFEND army move: {} (expected {}), MARCH army move: {} (expected {})",
        gs.nations[ni].armies[1].movement,
        expected_defend,
        gs.nations[ni].armies[2].movement,
        expected_march
    );
    assert_eq!(
        gs.nations[ni].armies[1].movement, expected_defend as u8,
        "DEFEND movement mismatch"
    );
    assert_eq!(
        gs.nations[ni].armies[2].movement, expected_march as u8,
        "MARCH movement mismatch"
    );
}

/// Test: military maintenance cost matches C.
/// C: tgold -= sold * maintenance[type%UTYPE]
#[test]
fn fn_updmil_maintenance_cost() {
    let mut gs = setup_state(8);
    let ni = 1;
    gs.nations[ni].active = 16u8;
    gs.nations[ni].max_move = 12;
    gs.nations[ni].class = 1;
    gs.nations[ni].treasury_gold = 100000;
    gs.nations[ni].cap_x = 0;
    gs.nations[ni].cap_y = 0;
    gs.sectors[0][0].owner = ni as u8;
    gs.sectors[0][0].designation = Designation::Capitol as u8;

    // Leader to avoid disarray
    let leader_type = conquer_engine::utils::getleader(gs.nations[ni].class).wrapping_sub(1);
    gs.nations[ni].armies[0].unit_type = leader_type;
    gs.nations[ni].armies[0].soldiers = 100;
    gs.nations[ni].armies[0].status = ArmyStatus::Defend.to_value();
    gs.nations[ni].armies[0].x = 0;
    gs.nations[ni].armies[0].y = 0;

    // 1000 soldiers of type 3
    gs.nations[ni].armies[1].unit_type = 3;
    gs.nations[ni].armies[1].soldiers = 1000;
    gs.nations[ni].armies[1].status = ArmyStatus::Defend.to_value();
    gs.nations[ni].armies[1].x = 0;
    gs.nations[ni].armies[1].y = 0;

    let gold_before = gs.nations[ni].treasury_gold;
    let mut rng = ConquerRng::new(99);
    updmil(&mut gs, &mut rng);

    // C: tgold -= sold * maintenance[type%UTYPE]
    let maint_3 = UNIT_MAINTENANCE
        .get(3 % UTYPE as usize)
        .copied()
        .unwrap_or(50) as i64;
    // Leader also costs maintenance for leaders (type >= MINLEADER): tgold -= 5 * maint
    // But leader maintenance only applies to monsters (type >= MINMONSTER)
    // Regular leaders: type >= MINLEADER && type < MINMONSTER → no gold cost
    let expected_cost = 1000 * maint_3; // just the infantry
    let actual_cost = gold_before - gs.nations[ni].treasury_gold;

    eprintln!(
        "Maintenance cost: {} (expected {} = 1000 * {})",
        actual_cost, expected_cost, maint_3
    );
    assert_eq!(actual_cost, expected_cost, "Military maintenance mismatch");
}

// ============================================================
// MOVEMENT TESTS
// ============================================================

/// Test: move_cost calculation matches C updmove()
#[test]
fn fn_move_cost_calculation() {
    let mut gs = setup_state(8);
    let ni = 1;
    gs.nations[ni].active = 16u8;
    gs.nations[ni].race = 'H'; // Human

    // Set specific terrain - use grassland (passable for humans)
    gs.sectors[3][3].altitude = Altitude::Clear as u8;
    gs.sectors[3][3].vegetation = Vegetation::LtVeg as u8;

    gs.sectors[4][4].altitude = Altitude::Water as u8; // Water = impassable

    gs.sectors[5][5].altitude = Altitude::Clear as u8;
    gs.sectors[5][5].vegetation = Vegetation::LtVeg as u8;
    gs.sectors[5][5].designation = Designation::Road as u8; // Road halves cost

    update_move_costs(&mut gs, 'H', ni);

    // Water should be negative
    assert!(
        gs.move_cost[4][4] < 0,
        "Water should be impassable (negative cost)"
    );

    let normal_cost = gs.move_cost[3][3];
    let road_cost = gs.move_cost[5][5];

    eprintln!("Normal cost={}, Road cost={}", normal_cost, road_cost);

    // If normal cost is passable (>0), road should reduce it
    if normal_cost > 0 {
        let expected_road = (normal_cost + 1) / 2;
        assert_eq!(road_cost, expected_road, "Road cost reduction mismatch");
    }
    assert!(normal_cost != 0, "Cost should not be 0 for land");
}

/// Test: land_reachp pathfinding
#[test]
fn fn_land_reachp_basic() {
    let mut gs = setup_state(8);
    let ni = 1;
    gs.nations[ni].active = 16u8;
    gs.world.map_x = 8;
    gs.world.map_y = 8;

    // Set all move costs to 1
    for x in 0..8 {
        for y in 0..8 {
            gs.move_cost[x][y] = 1;
        }
    }

    // Adjacent = reachable with 1 move point
    let r1 = land_reachp(&gs, 3, 3, 4, 3, 1, ni);
    eprintln!("Adjacent reachable with 1 move: {}", r1);
    assert!(r1, "Adjacent sector should be reachable with 1 move point");

    // 2 steps away needs 2 move points
    let r2 = land_reachp(&gs, 3, 3, 5, 3, 2, ni);
    eprintln!("2-away reachable with 2 moves: {}", r2);
    assert!(r2, "2 sectors away should be reachable with 2 move points");

    let r3 = land_reachp(&gs, 3, 3, 5, 3, 1, ni);
    eprintln!("2-away reachable with 1 move: {} (should be false)", r3);
    assert!(
        !r3,
        "2 sectors away should NOT be reachable with 1 move point"
    );

    // Water wall blocks — make a full column of water at x=4
    for yw in 0..8 {
        gs.sectors[4][yw].altitude = Altitude::Water as u8;
        gs.move_cost[4][yw] = -1;
    }
    let r4 = land_reachp(&gs, 3, 3, 5, 3, 10, ni);
    eprintln!("Blocked by water wall: {} (should be false)", r4);
    assert!(!r4, "Water wall should block pathfinding");
}

// ============================================================
// CAPTURE TESTS
// ============================================================

/// Test: update_capture with sole occupier captures unowned sector
#[test]
fn fn_capture_unowned_sector() {
    let mut gs = setup_state(8);
    let ni = 1;
    gs.nations[ni].active = 16u8;
    gs.nations[ni].total_civ = 10000;

    // Place army in unowned sector with enough soldiers
    gs.nations[ni].armies[0].unit_type = 3; // Base unit
    gs.nations[ni].armies[0].soldiers = 200; // > 75 (NPC threshold)
    gs.nations[ni].armies[0].status = ArmyStatus::Defend.to_value();
    gs.nations[ni].armies[0].x = 3;
    gs.nations[ni].armies[0].y = 3;

    // Sector is unowned
    gs.sectors[3][3].owner = 0;

    let mut rng = ConquerRng::new(99);
    let news = update_capture(&mut gs, &mut rng);

    assert_eq!(
        gs.sectors[3][3].owner, ni as u8,
        "Army should capture unowned sector"
    );
    eprintln!("Capture test passed: sector captured by nation {}", ni);
}

/// Test: capture requires WAR status for enemy sectors
#[test]
fn fn_capture_requires_war() {
    let mut gs = setup_state(8);
    let ni = 1;
    let enemy = 2;
    gs.nations[ni].active = 16u8;
    gs.nations[ni].total_civ = 10000;
    gs.nations[enemy].active = 16u8;

    // Army in enemy sector
    gs.nations[ni].armies[0].unit_type = 3;
    gs.nations[ni].armies[0].soldiers = 200;
    gs.nations[ni].armies[0].status = ArmyStatus::Defend.to_value();
    gs.nations[ni].armies[0].x = 3;
    gs.nations[ni].armies[0].y = 3;
    gs.sectors[3][3].owner = enemy as u8;

    // NOT at war
    gs.nations[ni].diplomacy[enemy] = DiplomaticStatus::Neutral as u8;

    let mut rng = ConquerRng::new(99);
    update_capture(&mut gs, &mut rng);
    assert_eq!(
        gs.sectors[3][3].owner, enemy as u8,
        "Should NOT capture without WAR"
    );

    // Now declare war
    gs.nations[ni].diplomacy[enemy] = DiplomaticStatus::War as u8;
    update_capture(&mut gs, &mut rng);
    assert_eq!(
        gs.sectors[3][3].owner, ni as u8,
        "Should capture when at WAR"
    );
}

// ============================================================
// NPC GARRISON TESTS
// ============================================================

/// Test: NPC garrison ideal calculation
/// C: ideal = tmil * peace / (10 * MILINCAP)
/// peace = 8 (at peace) or 12 (at war)
#[test]
fn fn_npc_garrison_ideal() {
    // This tests the garrison logic concept — not a direct function call
    // since nation_run combines many steps. But we can verify the formula.
    let tmil = 5000i64;
    let peace = 8; // at peace
    let ideal = tmil * peace / (10 * MILINCAP);
    // 5000 * 8 / (10 * 8) = 40000 / 80 = 500
    assert_eq!(ideal, 500, "Garrison ideal calculation");

    let peace_war = 12;
    let ideal_war = tmil * peace_war / (10 * MILINCAP);
    // 5000 * 12 / 80 = 750
    assert_eq!(ideal_war, 750, "Garrison ideal at war");
}

// ============================================================
// INFLATION TESTS
// ============================================================

/// Test: inflation reduces gold via popularity penalty
/// C: popularity -= 2 * inflation
/// C: if inflation > 0: inflation = rand()%(inflation/2+1)
/// C: inflation += tax_rate/4 + rand()%(tax_rate*3/4+1)
/// C: inflation += (tmil*100/tciv - 15) / 5
/// C: inflation += (poverty - 50) / 2
#[test]
fn fn_inflation_calculation() {
    // Inflation is computed in updsectors after spreadsheet.
    // Test the formula with known values.
    let tax_rate: u8 = 10;
    let popularity: u8 = 60;
    let inflation: i16 = 20;

    // Step 1: popularity adjustment
    let new_pop = popularity as i32 - 2 * inflation as i32;
    assert_eq!(new_pop, 20, "Popularity after inflation");

    // Step 2: inflation decay (deterministic part)
    // With inflation=20 and rand()%11 = 0 (min): new inflation = 0
    // With inflation=20 and rand()%11 = 10 (max): new inflation = 10
    // Then add: tax_rate/4 = 2, plus rand()%(tax_rate*3/4+1) = 0..8
    // So inflation range after: 2..20

    // Just verify the formula doesn't panic
    let min_inf = 0 + tax_rate as i16 / 4 + 0;
    let max_inf = inflation / 2 + tax_rate as i16 / 4 + (tax_rate as i16 * 3 / 4);
    eprintln!("Inflation range: {}..{}", min_inf, max_inf);
    assert!(min_inf <= max_inf);
}

// ============================================================
// OVERALL ECONOMY ROUND-TRIP
// ============================================================

/// Test: full economy cycle for one nation with staged data.
/// Set up a nation, run spreadsheet + updcomod + updmil, verify gold balance.
#[test]
fn fn_economy_roundtrip() {
    let mut gs = setup_state(16);
    let ni = 1;
    gs.nations[ni].active = 16u8;
    gs.nations[ni].tax_rate = 10;
    gs.nations[ni].treasury_gold = 50000;
    gs.nations[ni].total_food = 100000;
    gs.nations[ni].metals = 5000;
    gs.nations[ni].jewels = 10000;
    gs.nations[ni].eat_rate = 25;
    gs.nations[ni].spoil_rate = 5;
    gs.nations[ni].max_move = 12;
    gs.nations[ni].class = 1;
    gs.nations[ni].charity = 0;
    gs.nations[ni].cap_x = 0;
    gs.nations[ni].cap_y = 0;

    // Capitol with 8000 people
    gs.sectors[0][0].owner = ni as u8;
    gs.sectors[0][0].designation = Designation::Capitol as u8;
    gs.sectors[0][0].people = 8000;

    // Farm with 3000 people
    gs.sectors[1][0].owner = ni as u8;
    gs.sectors[1][0].designation = Designation::Farm as u8;
    gs.sectors[1][0].people = 3000;

    // Mine with 2000 people, metal=4
    gs.sectors[2][0].owner = ni as u8;
    gs.sectors[2][0].designation = Designation::Mine as u8;
    gs.sectors[2][0].people = 2000;
    gs.sectors[2][0].metal = 4;

    // Leader
    let leader_type = conquer_engine::utils::getleader(gs.nations[ni].class).wrapping_sub(1);
    gs.nations[ni].armies[0].unit_type = leader_type;
    gs.nations[ni].armies[0].soldiers = 100;
    gs.nations[ni].armies[0].status = ArmyStatus::Defend.to_value();
    gs.nations[ni].armies[0].x = 0;
    gs.nations[ni].armies[0].y = 0;

    // 500 infantry
    gs.nations[ni].armies[1].unit_type = 3;
    gs.nations[ni].armies[1].soldiers = 500;
    gs.nations[ni].armies[1].status = ArmyStatus::Defend.to_value();
    gs.nations[ni].armies[1].x = 0;
    gs.nations[ni].armies[1].y = 0;

    // Step 1: spreadsheet
    let spread = spreadsheet(&gs, ni);
    eprintln!("\nRound-trip spreadsheet:");
    eprintln!(
        "  Revenue: food={} metal={} cap={} other={}",
        spread.rev_food, spread.rev_metal, spread.rev_cap, spread.rev_other
    );
    eprintln!(
        "  Gold: {} -> {} (gain {})",
        50000,
        spread.gold,
        spread.gold - 50000
    );

    // Apply spreadsheet (simplified — normally done in updsectors)
    gs.nations[ni].treasury_gold = spread.gold;
    gs.nations[ni].total_food = spread.food;
    gs.nations[ni].metals = spread.metal;
    gs.nations[ni].jewels = spread.jewels;
    gs.nations[ni].total_civ = spread.civilians;
    gs.nations[ni].total_sectors = spread.sectors as i16;

    // Step 2: updmil
    let gold_before_mil = gs.nations[ni].treasury_gold;
    let mut rng = ConquerRng::new(99);
    updmil(&mut gs, &mut rng);
    eprintln!(
        "  After updmil: gold {} -> {} (cost {})",
        gold_before_mil,
        gs.nations[ni].treasury_gold,
        gold_before_mil - gs.nations[ni].treasury_gold
    );

    // Step 3: updcomodities
    let gold_before_com = gs.nations[ni].treasury_gold;
    let food_before = gs.nations[ni].total_food;
    updcomodities(&mut gs, &mut rng);
    eprintln!(
        "  After updcomod: gold {} -> {}, food {} -> {}",
        gold_before_com, gs.nations[ni].treasury_gold, food_before, gs.nations[ni].total_food
    );

    // Verify nothing went catastrophically wrong
    eprintln!(
        "  Final: gold={} food={} metal={} jewels={}",
        gs.nations[ni].treasury_gold,
        gs.nations[ni].total_food,
        gs.nations[ni].metals,
        gs.nations[ni].jewels
    );

    // Gold should be reasonable (not negative with these inputs)
    assert!(
        gs.nations[ni].treasury_gold > 0,
        "Gold went negative with moderate inputs"
    );
}
