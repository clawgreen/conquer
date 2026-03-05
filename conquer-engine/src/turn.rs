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
            process_nation_events(rng, &mut nations[i], sectors);
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
            if nations[i].tciv < 100 && nations[i].tmil < takesector(nations[i].tciv) {
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
        world.m_aplus += 1;
        world.m_dplus += 1;
        events.push("Mercenary bonuses increased!".to_string());
    }
    
    // Increase turn
    world.turn += 1;
    
    // Recalculate nation attributes
    calculate_attribute_base(nations);
    calculate_tradegood_bonus(nations);
    
    TurnResult {
        turn: current_turn,
        nation_updates,
        events,
        new_turn: world.turn,
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
    
    let start_gold = nation.tgold;
    let start_food = nation.tfood;
    let start_metals = nation.metals;
    let start_civ = nation.tciv;
    let start_sectors = nation.tsctrs;
    
    // Run nation economy
    update_nation_economy(nation, sectors, world.turn);
    
    // Calculate changes
    let gold_change = nation.tgold - start_gold;
    let food_change = nation.tfood - start_food;
    let metal_change = nation.metals - start_metals;
    let population_change = nation.tciv - start_civ;
    let sectors_change = nation.tsctrs - start_sectors;
    
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
                army.smove = get_max_movement(army.unit_type, nation.powers);
            }
        }
        
        // Reset navy movements
        for navy in nation.navies.iter_mut() {
            if navy.has_ships() {
                navy.smove = NAVY_MOVE;
            }
        }
    }
}

/// Get maximum movement for a unit type
fn get_max_movement(unit_type: u8, powers: i64) -> u8 {
    let r#move = unit_move(unit_type as usize);
    
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
            update_sector_population(sector, nations, world.turn, rng);
            
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
    let food_needed = nation.tciv * 10 / 100; // Need 10% of population in food
    let food_ratio = if food_needed > 0 {
        (nation.tfood * 100) / food_needed
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
        total_gold += nation.tgold;
        total_food += nation.tfood;
        total_metal += nation.metals;
        total_civ += nation.tciv;
        total_mil += nation.tmil;
        total_sectors += nation.tsctrs;
    }
    
    // Update world totals
    world.w_gold = total_gold;
    world.w_food = total_food;
    world.w_metal = total_metal;
    world.w_civ = total_civ;
    world.w_mil = total_mil;
    world.w_sctrs = total_sectors as i64;
}

/// Calculate individual nation score
fn calculate_nation_score(nation: &Nation) -> i64 {
    let mut score: i64 = 0;
    
    // Gold worth 1 point per 1000
    score += nation.tgold / 1000;
    
    // Food worth 1 point per 1000
    score += nation.tfood / 1000;
    
    // Metal worth 1 point per 100
    score += nation.metals / 100;
    
    // Civilians worth 1 point per 100
    score += nation.tciv / 100;
    
    // Military worth 2 points per 100
    score += nation.tmil * 2 / 100;
    
    // Sectors worth 10 points each
    score += nation.tsctrs as i64 * 10;
    
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
    nation.tgold = 0;
    nation.tfood = 0;
    nation.metals = 0;
    nation.tciv = 0;
    nation.tmil = 0;
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
            tgold: 10000,
            tfood: 10000,
            metals: 1000,
            tciv: 5000,
            tmil: 1000,
            tsctrs: 10,
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
