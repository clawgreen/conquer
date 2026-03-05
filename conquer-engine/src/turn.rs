// conquer-engine/src/turn.rs — Turn pipeline ported from update.c
//
// T261-T270: Turn processing, end-of-turn updates
//
// Main update function and supporting functions
use conquer_core::*;
use crate::rng::ConquerRng;
use crate::economy::*;
use crate::combat::*;
use crate::diplomacy::*;
use crate::events::*;
use crate::trade::*;
use crate::utils::is_habitable;

/// Wrapper for run_combat that accepts separate parameters
/// Matches C: combat()
fn run_world_combat(
    world: &mut World,
    nations: &mut [Nation; MAXNTOTAL],
    sectors: &mut [[Sector; MAPY as usize]; MAPX as usize],
    rng: &mut ConquerRng,
) {
    // Convert to GameState format for run_combat
    let map_x = world.map_x as usize;
    let map_y = world.map_y as usize;
    
    let mut state = GameState::new(map_x, map_y);
    state.world = world.clone();
    
    // Convert [Nation; MAXNTOTAL] to Vec<Nation>
    state.nations = nations.iter().cloned().collect();
    
    // Convert [[Sector; MAPY]; MAPX] to Vec<Vec<Sector>>
    state.sectors = sectors.iter()
        .map(|row| row.iter().cloned().collect())
        .collect();
    
    // Run combat
    let _results = run_combat(&mut state, rng);
    
    // Copy back
    *world = state.world;
    for (i, nation) in state.nations.iter().enumerate() {
        if i < MAXNTOTAL {
            nations[i] = nation.clone();
        }
    }
    for (i, row) in state.sectors.iter().enumerate() {
        for (j, sector) in row.iter().enumerate() {
            if i < MAPX as usize && j < MAPY as usize {
                sectors[i][j] = sector.clone();
            }
        }
    }
}

/// Navy movement default (based on C: ~4 for armies, navies vary)
pub const NAVY_MOVE: u8 = 4;

/// Turn update result
#[derive(Debug, Clone)]
pub struct TurnResult {
    pub turn: i32,
    pub nation_updates: Vec<NationUpdate>,
    pub events: Vec<String>,
    pub new_turn: i32,
}

#[derive(Debug, Clone)]
pub struct NationUpdate {
    pub nation_id: u8,
    pub gold_change: i64,
    pub food_change: i64,
    pub metal_change: i64,
    pub population_change: i64,
    pub sectors_lost: i32,
    pub sectors_gained: i32,
    pub armies_lost: i32,
    pub armies_gained: i32,
    pub message: String,
}

/// Execute one full turn update
/// Matches C: update() function
pub fn update_turn(
    world: &mut World,
    nations: &mut [Nation; MAXNTOTAL],
    sectors: &mut [[Sector; MAPY as usize]; MAPX as usize],
    rng: &mut ConquerRng,
) -> TurnResult {
    let mut events = Vec::new();
    let mut nation_updates = Vec::new();
    
    let current_turn = world.turn;
    
    // Run each nation in random order (updexecs in C)
    let mut nation_order: Vec<u8> = (1..MAXNTOTAL as u8).collect();
    shuffle_array(rng, &mut nation_order);
    
    // Process each nation
    for &nation_id in &nation_order {
        let nation_idx = nation_id as usize;
        if !is_nation_active(&nations[nation_idx]) {
            continue;
        }
        
        // Update nation
        let update = update_nation(
            nation_id,
            world,
            nations,
            sectors,
            rng,
        );
        
        events.push(format!("Nation {} updated", nations[nation_idx].name));
        nation_updates.push(update);
    }
    
    // Combat (combat() in C)
    events.push("Running combat...".to_string());
    run_world_combat(world, nations, sectors, rng);
    
    // Capture unowned sectors (updcapture in C)
    events.push("Capturing unoccupied sectors...".to_string());
    capture_unoccupied_sectors(nations, sectors);
    
    // Trade update
    events.push("Processing trade...".to_string());
    // Would process trade deals
    
    // Reset military (updmil in C)
    events.push("Resetting military movements...".to_string());
    reset_military_movements(nations);
    
    // Random events (randomevent in C)
    events.push("Generating random events...".to_string());
    for i in 1..MAXNTOTAL {
        if is_nation_active(&nations[i]) {
            process_nation_events(rng, &mut nations[i], i, sectors);
        }
    }
    
    // Update sectors
    events.push("Updating sectors...".to_string());
    update_all_sectors(world, nations, sectors, rng);
    
    // Update leaders
    events.push("Updating leaders...".to_string());
    // Would update leaders
    
    // Check for destroyed nations
    for i in 1..MAXNTOTAL {
        if is_nation_active(&nations[i]) {
            if nations[i].total_civ < 100 && nations[i].total_mil < takesector(nations[i].total_civ) {
                // Destroy nation
                events.push(format!("Nation {} has been destroyed!", nations[i].name));
                destroy_nation(i, nations, sectors);
            }
        }
    }
    
    // Score calculation (score() in C)
    calculate_scores(world, nations);
    
    // Mercenary increase (5% chance)
    if rng.rand() % 20 == 0 {
        world.merc_aplus += 1;
        world.merc_dplus += 1;
        events.push("Mercenary bonuses increased!".to_string());
    }
    
    // Increase turn
    world.turn += 1;
    
    // Recalculate nation attributes
    calculate_attribute_base(nations);
    calculate_tradegood_bonus(nations);
    
    TurnResult {
        turn: current_turn as i32,
        nation_updates,
        events,
        new_turn: world.turn as i32,
    }
}

/// Update a single nation
/// Matches C: updexecs() logic
fn update_nation(
    nation_id: u8,
    world: &mut World,
    nations: &mut [Nation; MAXNTOTAL],
    sectors: &mut [[Sector; MAPY as usize]; MAPX as usize],
    rng: &mut ConquerRng,
) -> NationUpdate {
    let nation_idx = nation_id as usize;
    let nation = &mut nations[nation_idx];
    
    let start_gold = nation.treasury_gold;
    let start_food = nation.total_food;
    let start_metals = nation.metals;
    let start_civ = nation.total_civ;
    let start_sectors = nation.total_sectors;
    
    // Run nation economy
    update_nation_economy(nation_idx, nation, sectors, world.turn);
    
    // Calculate changes
    let gold_change = nation.treasury_gold - start_gold;
    let food_change = nation.total_food - start_food;
    let metal_change = nation.metals - start_metals;
    let population_change = nation.total_civ - start_civ;
    let sectors_change = nation.total_sectors - start_sectors;
    
    NationUpdate {
        nation_id,
        gold_change,
        food_change,
        metal_change,
        population_change,
        sectors_lost: 0,
        sectors_gained: 0,
        armies_lost: 0,
        armies_gained: 0,
        message: format!("{} updated", nation.name),
    }
}

/// Capture unoccupied sectors
/// Matches C: updcapture()
fn capture_unoccupied_sectors(
    nations: &mut [Nation; MAXNTOTAL],
    sectors: &mut [[Sector; MAPY as usize]; MAPX as usize],
) {
    for x in 0..MAPX {
        for y in 0..MAPY {
            let sector = &mut sectors[x as usize][y as usize];
            
            // Skip if owned or not habitable
            if sector.owner != 0 {
                continue;
            }
            
            if !is_habitable(sector) {
                continue;
            }
            
            // Check if any nation can capture
            for i in 1..MAXNTOTAL {
                if !is_nation_active(&nations[i]) {
                    continue;
                }
                
                // Would check adjacency and move in
                // Simplified: capture if has armies nearby
            }
        }
    }
}

/// Reset military movements
/// Matches C: updmil()
fn reset_military_movements(nations: &mut [Nation; MAXNTOTAL]) {
    for nation in nations.iter_mut() {
        if !is_nation_active(nation) {
            continue;
        }
        
        // Reset army movements
        for army in nation.armies.iter_mut() {
            if army.soldiers > 0 {
                army.movement = get_max_movement(army.unit_type, nation.powers);
            }
        }
        
        // Reset navy movements
        for navy in nation.navies.iter_mut() {
            if navy.has_ships() {
                navy.movement = NAVY_MOVE;
            }
        }
    }
}

/// Update a single nation's economy
/// Simplified version - applies basic economy processing
fn update_nation_economy(
    nation_idx: usize,
    nation: &mut Nation,
    sectors: &mut [[Sector; MAPY as usize]; MAPX as usize],
    turn: i16,
) {
    // Process each sector the nation owns
    for x in 0..MAPX {
        for y in 0..MAPY {
            let sector = &mut sectors[x as usize][y as usize];
            
            if sector.owner as usize != nation_idx {
                continue;
            }
            
            // Process based on designation
            match sector.designation as char {
                'F' | 'f' => {
                    // Farm - produce food
                    let food_prod = sector.people * 5 / 100; // 5% production
                    nation.total_food += food_prod;
                }
                'M' | 'm' => {
                    // Mine - produce metal
                    if sector.metal > 0 {
                        let metal_prod = sector.people * sector.metal as i64 / 10;
                        nation.metals += metal_prod;
                    }
                }
                'C' | 'c' => {
                    // City - produce gold
                    let gold_prod = sector.people * 10 / 100;
                    nation.treasury_gold += gold_prod;
                }
                'T' | 't' => {
                    // Town - produce gold
                    let gold_prod = sector.people * 5 / 100;
                    nation.treasury_gold += gold_prod;
                }
                _ => {}
            }
        }
    }
    
    // Apply military upkeep
    let mil_cost = nation.total_mil * 2;
    nation.treasury_gold -= mil_cost;
    
    // Apply food consumption
    let food_needed = nation.total_civ * 10 / 100 + nation.total_mil * 5 / 100;
    if nation.total_food >= food_needed {
        nation.total_food -= food_needed;
    } else {
        nation.total_food = 0;
        // Starvation - lose civilians
        nation.total_civ = (nation.total_civ * 9) / 10;
    }
    
    // Inflation
    nation.inflation = ((nation.inflation as i64 + 100) * 101 / 100) as i16;
}

/// Get maximum movement for a unit type
fn get_max_movement(unit_type: u8, powers: i64) -> u8 {
    use conquer_core::tables::UNIT_MOVE;
    let r#move = UNIT_MOVE[unit_type as usize];
    
    // CAVALRY power bonus
    if Power::has_power(powers, Power::CAVALRY) {
        return (r#move * 3 / 2) as u8;
    }
    
    // AV_MONST power bonus
    if Power::has_power(powers, Power::AV_MONST) {
        return (r#move * 2) as u8;
    }
    
    r#move as u8
}

/// Update all sectors
/// Matches C: updsectors()
fn update_all_sectors(
    world: &World,
    nations: &mut [Nation; MAXNTOTAL],
    sectors: &mut [[Sector; MAPY as usize]; MAPX as usize],
    rng: &mut ConquerRng,
) {
    for x in 0..MAPX {
        for y in 0..MAPY {
            let sector = &mut sectors[x as usize][y as usize];
            
            if sector.owner == 0 {
                continue;
            }
            
            // Update population
            update_sector_population(sector, nations, world.turn as i32, rng);
            
            // Update production
            update_sector_production(sector, nations);
            
            // Check for events
            // (volcano, etc.)
        }
    }
}

/// Update sector population
fn update_sector_population(
    sector: &mut Sector,
    nations: &[Nation; MAXNTOTAL],
    turn: i32,
    rng: &mut ConquerRng,
) {
    let owner_idx = sector.owner as usize;
    if owner_idx >= MAXNTOTAL {
        return;
    }
    
    let nation = &nations[owner_idx];
    
    // Base reproduction
    let base_growth = 10i64; // 10% base
    
    // Food affects growth
    let food_needed = nation.total_civ * 10 / 100; // Need 10% of population in food
    let food_ratio = if food_needed > 0 {
        (nation.total_food * 100) / food_needed
    } else {
        100
    };
    
    let growth_rate = if food_ratio >= 100 {
        base_growth
    } else if food_ratio >= 50 {
        base_growth / 2
    } else {
        -base_growth // Starvation
    };
    
    // Apply growth
    let change = (sector.people as i64 * growth_rate) / 100;
    sector.people = (sector.people as i64 + change) as i64;
    
    // Cap at ABSMAXPEOPLE
    if sector.people > ABSMAXPEOPLE as i64 {
        sector.people = ABSMAXPEOPLE as i64;
    }
    if sector.people < 0 {
        sector.people = 0;
    }
    
    // Check forTOO MANY PEOPLE
    if sector.people > TOMANYPEOPLE as i64 {
        // Half reproduction, half production
        sector.people = TOMANYPEOPLE as i64;
    }
}

/// Update sector production
fn update_sector_production(
    sector: &mut Sector,
    nations: &[Nation; MAXNTOTAL],
) {
    let owner_idx = sector.owner as usize;
    if owner_idx >= MAXNTOTAL {
        return;
    }
    
    // Production depends on designation and trade goods
    // This is handled in economy module
}

/// Calculate scores
/// Matches C: score()
fn calculate_scores(
    world: &mut World,
    nations: &mut [Nation; MAXNTOTAL],
) {
    let mut total_gold: i64 = 0;
    let mut total_food: i64 = 0;
    let mut total_metal: i64 = 0;
    let mut total_civ: i64 = 0;
    let mut total_mil: i64 = 0;
    let mut total_sectors: i32 = 0;
    
    for nation in nations.iter_mut() {
        if !is_nation_active(nation) {
            continue;
        }
        
        // Calculate nation score
        let score = calculate_nation_score(nation);
        nation.score = score;
        
        // Add to world totals
        total_gold += nation.treasury_gold;
        total_food += nation.total_food;
        total_metal += nation.metals;
        total_civ += nation.total_civ;
        total_mil += nation.total_mil;
        total_sectors += nation.total_sectors as i32;
    }
    
    // Update world totals
    world.world_gold = total_gold;
    world.world_food = total_food;
    world.world_metal = total_metal;
    world.world_civ = total_civ;
    world.world_mil = total_mil;
    world.world_sectors = total_sectors as i64;
}

/// Calculate individual nation score
fn calculate_nation_score(nation: &Nation) -> i64 {
    let mut score: i64 = 0;
    
    // Gold worth 1 point per 1000
    score += nation.treasury_gold / 1000;
    
    // Food worth 1 point per 1000
    score += nation.total_food / 1000;
    
    // Metal worth 1 point per 100
    score += nation.metals / 100;
    
    // Civilians worth 1 point per 100
    score += nation.total_civ / 100;
    
    // Military worth 2 points per 100
    score += nation.total_mil * 2 / 100;
    
    // Sectors worth 10 points each
    score += nation.total_sectors as i64 * 10;
    
    // Active military units worth 5 points each
    for army in &nation.armies {
        if army.soldiers > 0 {
            score += 5;
        }
    }
    
    // Ships worth 10 points each
    for navy in &nation.navies {
        if navy.has_ships() {
            score += 10;
        }
    }
    
    score
}

/// Destroy a nation
/// Matches C: destroy()
fn destroy_nation(
    nation_idx: usize,
    nations: &mut [Nation; MAXNTOTAL],
    sectors: &mut [[Sector; MAPY as usize]; MAPX as usize],
) {
    let nation = &mut nations[nation_idx];
    
    // Remove ownership of all sectors
    for x in 0..MAPX {
        for y in 0..MAPY {
            if sectors[x as usize][y as usize].owner == nation_idx as u8 {
                sectors[x as usize][y as usize].owner = 0;
            }
        }
    }
    
    // Clear all armies
    for army in nation.armies.iter_mut() {
        army.soldiers = 0;
    }
    
    // Clear all navies
    for navy in nation.navies.iter_mut() {
        navy.warships = 0;
        navy.merchant = 0;
        navy.galleys = 0;
    }
    
    // Mark as inactive
    nation.active = 0;
    nation.treasury_gold = 0;
    nation.total_food = 0;
    nation.metals = 0;
    nation.total_civ = 0;
    nation.total_mil = 0;
}

/// Check if nation is active
fn is_nation_active(nation: &Nation) -> bool {
    nation.active != 0
}

/// Shuffle array using Fisher-Yates
fn shuffle_array(rng: &mut ConquerRng, arr: &mut [u8]) {
    let len = arr.len();
    if len <= 1 {
        return;
    }
    
    for i in (1..len).rev() {
        let j = (rng.rand() as usize) % (i + 1);
        arr.swap(i, j);
    }
}

/// Calculate base nation attributes
/// Matches C: att_base()
fn calculate_attribute_base(nations: &mut [Nation; MAXNTOTAL]) {
    for nation in nations.iter_mut() {
        if !is_nation_active(nation) {
            continue;
        }
        
        // Would calculate base attributes based on race, etc.
    }
}

/// Calculate tradegood bonus
/// Matches C: att_bonus()
fn calculate_tradegood_bonus(nations: &mut [Nation; MAXNTOTAL]) {
    for nation in nations.iter_mut() {
        if !is_nation_active(nation) {
            continue;
        }
        
        // Would calculate bonus based on trade goods
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_nation_score() {
        let nation = Nation {
            treasury_gold: 10000,
            total_food: 10000,
            metals: 1000,
            total_civ: 5000,
            total_mil: 1000,
            total_sectors: 10,
            ..Default::default()
        };
        
        let score = calculate_nation_score(&nation);
        
        // 10000/1000 = 10 (gold)
        // 10000/1000 = 10 (food)
        // 1000/100 = 10 (metal)
        // 5000/100 = 50 (civ)
        // 1000*2/100 = 20 (mil)
        // 10*10 = 100 (sectors)
        // = 200
        assert_eq!(score, 200);
    }
}
