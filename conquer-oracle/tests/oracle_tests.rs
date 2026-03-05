use conquer_core::rng::ConquerRng;
use conquer_oracle::OracleSnapshot;
use std::fs;
use std::path::PathBuf;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

// ============================================================
// T020b: RNG Cross-Validation (10,000 numbers, seed 42)
// ============================================================

#[test]
fn test_rng_cross_validation_seed42() {
    let c_values_path = project_root().join("oracle/snapshots/rng_seed42_10000.txt");
    if !c_values_path.exists() {
        eprintln!("Skipping RNG cross-validation: {} not found", c_values_path.display());
        return;
    }

    let c_output = fs::read_to_string(&c_values_path).unwrap();
    let c_values: Vec<i32> = c_output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.trim().parse::<i32>().unwrap())
        .collect();

    assert_eq!(c_values.len(), 10000, "Expected 10,000 C RNG values");

    let mut rng = ConquerRng::new(42);
    for (i, &expected) in c_values.iter().enumerate() {
        let actual = rng.rand();
        assert_eq!(
            actual, expected,
            "RNG mismatch at index {}: Rust={} C={}",
            i, actual, expected
        );
    }
}

// ============================================================
// T109-T110: Oracle JSON Loader & Verification
// ============================================================

#[test]
fn test_load_oracle_snapshot() {
    let snap_path = project_root().join("oracle/snapshots/seed42_init.json");
    if !snap_path.exists() {
        eprintln!("Skipping oracle snapshot test: {} not found", snap_path.display());
        return;
    }

    let json = fs::read_to_string(&snap_path).unwrap();
    let snapshot = OracleSnapshot::from_json(&json).unwrap();
    let gs = snapshot.to_game_state();

    // Verify world data
    assert_eq!(gs.world.map_x, 32);
    assert_eq!(gs.world.map_y, 32);
    assert!(gs.world.turn > 0);
    assert!(gs.world.score > 0);
    assert!(gs.world.world_gold > 0);
    assert!(gs.world.world_food > 0);

    // Verify nations
    assert_eq!(gs.nations.len(), 35);
    assert_eq!(gs.nations[0].name, "unowned");
    assert_eq!(gs.nations[0].race, '-');

    // Verify at least some nations are active
    let active_count = gs.nations.iter().filter(|n| n.is_active()).count();
    assert!(active_count > 0, "Expected at least one active nation");

    // Verify at least some armies exist
    let total_armies: usize = gs.nations.iter()
        .map(|n| n.alive_armies())
        .sum();
    assert!(total_armies > 0, "Expected at least one army");

    // Verify sectors
    assert_eq!(gs.sectors.len(), 32);
    assert_eq!(gs.sectors[0].len(), 32);
}

// ============================================================
// T111: Enum Round-Trip Verification Against Oracle Data
// ============================================================

#[test]
fn test_enum_roundtrips_against_oracle() {
    let snap_path = project_root().join("oracle/snapshots/seed42_init.json");
    if !snap_path.exists() {
        eprintln!("Skipping enum roundtrip test: {} not found", snap_path.display());
        return;
    }

    let json = fs::read_to_string(&snap_path).unwrap();
    let snapshot = OracleSnapshot::from_json(&json).unwrap();
    let gs = snapshot.to_game_state();

    // Verify all nation strategies round-trip
    for n in &gs.nations {
        let strategy = conquer_core::NationStrategy::from_value(n.active);
        assert!(strategy.is_some(), "Invalid strategy value: {}", n.active);
        assert_eq!(strategy.unwrap().to_value(), n.active);
    }

    // Verify all army statuses round-trip
    for n in &gs.nations {
        for a in &n.armies {
            if a.soldiers > 0 {
                let status = conquer_core::ArmyStatus::from_value(a.status);
                assert_eq!(status.to_value(), a.status,
                    "Army status roundtrip failed: {}", a.status);
            }
        }
    }

    // Verify all unit types have valid stats indices
    for n in &gs.nations {
        for a in &n.armies {
            if a.soldiers > 0 {
                let ut = conquer_core::UnitType(a.unit_type);
                assert!(ut.stats_index().is_some(),
                    "Invalid unit type: {} for nation {}", a.unit_type, n.name);
            }
        }
    }

    // Verify race chars round-trip
    for n in &gs.nations {
        if n.is_active() {
            let race = conquer_core::Race::from_char(n.race);
            assert_eq!(race.to_char(), n.race,
                "Race roundtrip failed for '{}' in nation {}", n.race, n.name);
        }
    }
}

// ============================================================
// T112: Data Table Values Match C Arrays
// ============================================================

#[test]
fn test_data_tables_match_c() {
    // Verify designation chars match C des[] = "tcmfx$!&sC?lb+*g=u-P0"
    let c_des = "tcmfx$!&sC?lb+*g=u-P";
    for (i, c) in c_des.chars().enumerate() {
        let d = conquer_core::Designation::from_index(i as u8).unwrap();
        assert_eq!(d.to_char(), c, "Designation char mismatch at index {}: expected '{}' got '{}'", i, c, d.to_char());
    }

    // Verify vegetation chars match C veg[] = "vdtblgwfjsi~0"
    let c_veg = "vdtblgwfjsi~";
    for (i, c) in c_veg.chars().enumerate() {
        let v = conquer_core::Vegetation::from_index(i as u8).unwrap();
        assert_eq!(v.to_char(), c, "Vegetation char mismatch at index {}: expected '{}' got '{}'", i, c, v.to_char());
    }

    // Verify altitude chars match C ele[] = "~#^%-0"
    let c_ele = "~#^%-";
    for (i, c) in c_ele.chars().enumerate() {
        let a = conquer_core::Altitude::from_index(i as u8).unwrap();
        assert_eq!(a.to_char(), c, "Altitude char mismatch at index {}: expected '{}' got '{}'", i, c, a.to_char());
    }

    // Verify vegfood matches "0004697400000"
    let c_vegfood = "0004697400000";
    for (i, c) in c_vegfood.chars().enumerate().take(12) {
        let expected = c.to_digit(10).unwrap() as i32;
        if let Some(v) = conquer_core::Vegetation::from_index(i as u8) {
            assert_eq!(v.food_value(), expected,
                "VegFood mismatch at index {}: expected {} got {}", i, expected, v.food_value());
        }
    }

    // Verify power bitmask values
    assert_eq!(conquer_core::powers::POWERS_ARRAY[0].bits(), 0x00000001); // WARRIOR
    assert_eq!(conquer_core::powers::POWERS_ARRAY[11].bits(), 0x00000800); // SLAVER
    assert_eq!(conquer_core::powers::POWERS_ARRAY[24].bits(), 0x01000000); // THE_VOID
    assert_eq!(conquer_core::powers::POWERS_ARRAY[30].bits(), 0x40000000); // SORCERER

    // Verify unit stats array lengths
    assert_eq!(conquer_core::tables::UNIT_ATTACK.len(), 60);
    assert_eq!(conquer_core::tables::UNIT_DEFEND.len(), 60);
    assert_eq!(conquer_core::tables::UNIT_MOVE.len(), 60);
    assert_eq!(conquer_core::tables::UNIT_ENLIST_METAL.len(), 60);
    assert_eq!(conquer_core::tables::UNIT_ENLIST_COST.len(), 60);
    assert_eq!(conquer_core::tables::UNIT_MAINTENANCE.len(), 60);
    assert_eq!(conquer_core::tables::UNIT_MIN_STRENGTH.len(), 60);

    // Spot-check specific unit values from data.c
    // Militia (index 0): attack=-40, defend=-25, move=0
    assert_eq!(conquer_core::tables::UNIT_ATTACK[0], -40);
    assert_eq!(conquer_core::tables::UNIT_DEFEND[0], -25);
    assert_eq!(conquer_core::tables::UNIT_MOVE[0], 0);

    // Knight (index 21): attack=40, defend=40, move=20
    assert_eq!(conquer_core::tables::UNIT_ATTACK[21], 40);
    assert_eq!(conquer_core::tables::UNIT_DEFEND[21], 40);
    assert_eq!(conquer_core::tables::UNIT_MOVE[21], 20);

    // Dragon (monster, index 59): attack=50, defend=50, move=20, minsth=1000, maint=10000
    assert_eq!(conquer_core::tables::UNIT_ATTACK[59], 50);
    assert_eq!(conquer_core::tables::UNIT_DEFEND[59], 50);
    assert_eq!(conquer_core::tables::UNIT_MOVE[59], 20);
    assert_eq!(conquer_core::tables::UNIT_MIN_STRENGTH[59], 1000);
    assert_eq!(conquer_core::tables::UNIT_MAINTENANCE[59], 10000);

    // Trade good counts
    assert_eq!(conquer_core::tables::TG_NAMES.len(), 62);
    assert_eq!(conquer_core::tables::TG_NAMES[0], "furs");
    assert_eq!(conquer_core::tables::TG_NAMES[61], "none");
}
