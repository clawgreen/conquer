// conquer-engine/src/turn.rs — Turn pipeline ported from update.c
//
// T261-T270: Turn processing, end-of-turn updates
//
// Main update function and supporting functions.
// T10: Refactored to use &mut GameState instead of fixed-size arrays.
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

/// Execute one full turn update using GameState (dynamic Vecs, no fixed arrays).
/// Matches C: update() function ordering exactly:
///   updexecs -> monster -> combat -> updcapture -> uptrade -> updmil -> randomevent
///   -> updsectors -> move_people -> updcomodities -> updleader -> destroy check
///   -> score -> cheat -> att_bonus
pub fn update_turn(
    state: &mut GameState,
    rng: &mut ConquerRng,
) -> TurnResult {
    let mut events = Vec::new();
    let nation_updates = Vec::new();

    let current_turn = state.world.turn;

    // 1. updexecs: Run each NPC nation in random order
    let mut nation_order: Vec<usize> = (1..NTOTAL).collect();
    shuffle_array_usize(rng, &mut nation_order);

    for &nation_idx in &nation_order {
        let active = state.nations[nation_idx].active;
        if active == 0 { continue; }
        let strat = NationStrategy::from_value(active);
        if strat.map_or(false, |s| s.is_npc()) {
            let _news = crate::npc::nation_run(state, nation_idx, rng);
        }
        events.push(format!("Nation {} updated", state.nations[nation_idx].name));
    }

    // 2. monster() - monster nation updates
    let _monster_news = crate::monster::update_monsters(state, rng);

    // 3. combat()
    events.push("Running combat...".to_string());
    let _results = run_combat(state, rng);

    // 4. updcapture()
    events.push("Capturing unoccupied sectors...".to_string());
    let capture_news = crate::movement::update_capture(state, rng);
    events.extend(capture_news);

    // 5. uptrade()
    events.push("Processing trades...".to_string());
    let _trade_news = crate::trade::process_trades_gs(state);

    // 6. updmil() - reset military, movement, maintenance, recount tmil
    events.push("Resetting military...".to_string());
    updmil(state, rng);

    // 7. randomevent()
    events.push("Random events...".to_string());
    let event_news = crate::events::process_events_gs(state, rng);
    events.extend(event_news);

    // 8. updsectors() - population growth, spreadsheet, inflation, poverty
    events.push("Updating sectors...".to_string());
    updsectors(state, rng);

    // 9. move_people() - civilian migration between sectors
    move_people_gs(state);

    // 10. updcomodities() - food consumption, spoilage, jewel balancing
    events.push("Updating commodities...".to_string());
    updcomodities(state, rng);

    // 11. updleader() - leader births
    events.push("Updating leaders...".to_string());
    update_leaders(state, rng);

    // 12. Check for destroyed nations — C uses isntn() which excludes monsters (active > 16)
    for i in 1..NTOTAL {
        let active = state.nations[i].active;
        if active != 0 && active <= 16 {
            if state.nations[i].total_civ < 100
                && state.nations[i].total_mil < takesector(state.nations[i].total_civ)
            {
                events.push(format!("Nation {} has been destroyed!", state.nations[i].name));
                destroy_nation_gs(state, i);
            }
        }
    }

    // 13. score()
    calculate_scores_gs(state);

    // 14. cheat() — NPC bonus (skipped here; enabled via game settings in store.rs)

    // 15. att_base() + att_bonus()
    att_base_gs(state, rng);
    att_bonus_gs(state);

    // Mercenary increase (5% chance)
    if rng.rand() % 20 == 0 {
        state.world.merc_aplus += 1;
        state.world.merc_dplus += 1;
        events.push("Mercenary bonuses increased!".to_string());
    }

    // Increase turn
    state.world.turn += 1;

    TurnResult {
        turn: current_turn as i32,
        nation_updates,
        events,
        new_turn: state.world.turn as i32,
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
    score / 10
}

/// Shuffle usize slice using Fisher-Yates
fn shuffle_array_usize(rng: &mut ConquerRng, arr: &mut [usize]) {
    let len = arr.len();
    if len <= 1 { return; }
    for i in (1..len).rev() {
        let j = (rng.rand() as usize) % (i + 1);
        arr.swap(i, j);
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
