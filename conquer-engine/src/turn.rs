// conquer-engine/src/turn.rs — Turn pipeline ported from update.c
//
// T261-T270: Turn processing, end-of-turn updates
//
// Main update function and supporting functions
use conquer_core::*;
use crate::rng::ConquerRng;
use crate::economy::*;
use crate::combat::*;

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
/// Matches C: update() function ordering exactly:
///   updexecs -> monster -> combat -> updcapture -> updmil -> randomevent
///   -> updsectors -> updcomodities -> updleader -> destroy check -> score
pub fn update_turn(
    world: &mut World,
    nations: &mut [Nation; MAXNTOTAL],
    sectors: &mut [[Sector; MAPY]; MAPX],
    rng: &mut ConquerRng,
) -> TurnResult {
    let mut events = Vec::new();
    let mut nation_updates = Vec::new();
    
    let current_turn = world.turn;
    
    // Convert to GameState for economy functions that need it
    let mut state = arrays_to_gamestate(world, nations, sectors);
    
    // 1. updexecs: Run each nation in random order
    let mut nation_order: Vec<u8> = (1..MAXNTOTAL as u8).collect();
    shuffle_array(rng, &mut nation_order);
    
    for &nation_id in &nation_order {
        let nation_idx = nation_id as usize;
        if !is_nation_active_gs(&state.nations[nation_idx]) {
            continue;
        }
        events.push(format!("Nation {} updated", state.nations[nation_idx].name));
    }
    
    // 2. monster() - monster nation updates (lizard growth etc)
    // (simplified - monsters handled via normal combat/movement)
    
    // 3. combat()
    events.push("Running combat...".to_string());
    let _results = run_combat(&mut state, rng);
    
    // 4. updcapture()
    events.push("Capturing unoccupied sectors...".to_string());
    // (simplified - armies capture sectors during movement)
    
    // 5. updmil() - reset military, movement, maintenance, recount tmil
    events.push("Resetting military...".to_string());
    updmil(&mut state, rng);
    
    // 6. randomevent()
    events.push("Random events...".to_string());
    // (simplified - random events not critical for economy parity)
    
    // 7. updsectors() - population growth, spreadsheet, inflation, poverty
    events.push("Updating sectors...".to_string());
    updsectors(&mut state, rng);
    
    // 8. updcomodities() - food consumption, spoilage, jewel balancing
    events.push("Updating commodities...".to_string());
    updcomodities(&mut state, rng);
    
    // 9. updleader() - leader births (simplified)
    events.push("Updating leaders...".to_string());
    
    // 10. Check for destroyed nations — C uses isntn() which excludes monsters (active > 16)
    for i in 1..NTOTAL {
        let active = state.nations[i].active;
        if active != 0 && active <= 16 {
            // C: isntn() check — only regular nations, not monsters
            if state.nations[i].total_civ < 100 
                && state.nations[i].total_mil < takesector(state.nations[i].total_civ) 
            {
                events.push(format!("Nation {} has been destroyed!", state.nations[i].name));
                destroy_nation_gs(&mut state, i);
            }
        }
    }
    
    // 11. score()
    calculate_scores_gs(&mut state);
    
    // Mercenary increase (5% chance)
    if rng.rand() % 20 == 0 {
        state.world.merc_aplus += 1;
        state.world.merc_dplus += 1;
        events.push("Mercenary bonuses increased!".to_string());
    }
    
    // Increase turn
    state.world.turn += 1;
    
    // Recalculate nation attributes (att_base + att_bonus)
    calculate_attribute_base_gs(&mut state);
    
    // Copy back to fixed-size arrays
    gamestate_to_arrays(&state, world, nations, sectors);
    
    TurnResult {
        turn: current_turn as i32,
        nation_updates,
        events,
        new_turn: state.world.turn as i32,
    }
}

// ── Conversion helpers: fixed-size arrays <-> GameState ──

fn arrays_to_gamestate(
    world: &World,
    nations: &[Nation; MAXNTOTAL],
    sectors: &[[Sector; MAPY]; MAPX],
) -> GameState {
    let map_x = world.map_x as usize;
    let map_y = world.map_y as usize;
    let mut state = GameState::new(map_x, map_y);
    state.world = world.clone();
    state.nations = nations.iter().cloned().collect();
    state.sectors = sectors.iter()
        .take(map_x)
        .map(|row| row.iter().take(map_y).cloned().collect())
        .collect();
    state
}

fn gamestate_to_arrays(
    state: &GameState,
    world: &mut World,
    nations: &mut [Nation; MAXNTOTAL],
    sectors: &mut [[Sector; MAPY]; MAPX],
) {
    *world = state.world.clone();
    for (i, nation) in state.nations.iter().enumerate() {
        if i < MAXNTOTAL {
            nations[i] = nation.clone();
        }
    }
    for (i, row) in state.sectors.iter().enumerate() {
        for (j, sector) in row.iter().enumerate() {
            if i < MAPX && j < MAPY {
                sectors[i][j] = sector.clone();
            }
        }
    }
}

fn is_nation_active_gs(nation: &Nation) -> bool {
    nation.active != 0
}

/// Destroy a nation (GameState version)
fn destroy_nation_gs(state: &mut GameState, nation_idx: usize) {
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;
    
    for x in 0..map_x {
        for y in 0..map_y {
            if state.sectors[x][y].owner == nation_idx as u8 {
                state.sectors[x][y].owner = 0;
            }
        }
    }
    
    let nation = &mut state.nations[nation_idx];
    for army in nation.armies.iter_mut() {
        army.soldiers = 0;
    }
    for navy in nation.navies.iter_mut() {
        navy.warships = 0;
        navy.merchant = 0;
        navy.galleys = 0;
    }
    nation.active = 0;
    nation.treasury_gold = 0;
    nation.total_food = 0;
    nation.metals = 0;
    nation.total_civ = 0;
    nation.total_mil = 0;
}

/// Score all nations (GameState version)
pub fn calculate_scores_gs(state: &mut GameState) {
    for i in 1..NTOTAL {
        if !is_nation_active_gs(&state.nations[i]) { continue; }
        state.nations[i].score += calculate_nation_score(&state.nations[i]);
    }
}

/// Calculate individual nation score
/// Matches C: score() - incremental per-turn score
fn calculate_nation_score(nation: &Nation) -> i64 {
    let mut score: i64 = 0;
    
    // Gold worth 1 point per 1000
    if nation.treasury_gold > 0 {
        score += nation.treasury_gold / 1000;
    }
    
    // Civilians worth 1 point per 100
    score += nation.total_civ / 100;
    
    // Military worth 2 points per 100
    score += nation.total_mil * 2 / 100;
    
    // Sectors worth 10 points each
    score += nation.total_sectors as i64 * 10;
    
    // Ships
    score += nation.total_ships as i64 * 5;
    
    // Score is incremental in C (score += each turn)
    // Divide by a factor since C accumulates smaller per-turn amounts
    score / 10
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

/// Calculate base nation attributes (GameState version)
/// Matches C: att_base() + att_setup()
fn calculate_attribute_base_gs(state: &mut GameState) {
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;
    
    for country in 1..NTOTAL {
        if !is_nation_active_gs(&state.nations[country]) { continue; }
        
        // Recalculate spoilrate based on granaries + cities
        let mut ncities: i64 = 0;
        let mut ngranary: i64 = 0;
        
        for x in 0..map_x {
            for y in 0..map_y {
                if state.sectors[x][y].owner as usize != country { continue; }
                match state.sectors[x][y].designation {
                    d if d == Designation::City as u8 => ncities += 1,
                    d if d == Designation::Capitol as u8 => ncities += 3,
                    d if d == Designation::Granary as u8 => ngranary += 1,
                    _ => {}
                }
            }
        }
        
        if 30 <= 1 + ngranary + ncities {
            state.nations[country].spoil_rate = 1;
        } else {
            state.nations[country].spoil_rate = (30 - ngranary - ncities) as u8;
        }
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
        // Incremental score calculation
        assert!(score > 0);
    }
}
