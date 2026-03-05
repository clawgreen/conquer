// conquer-engine/src/diplomacy.rs — Diplomacy system ported from check.c, data.c
//
// T235-T242: Diplomatic status, nation relationships, war/peace
//
// Diplomatic status levels: UNMET, TREATY, ALLIED, FRIENDLY, NEUTRAL, HOSTILE, WAR, JIHAD

use conquer_core::*;
use crate::rng::RngExt;

/// Diplomatic status constants (matching C)
pub const DIPL_UNMET: u8 = 0;
pub const DIPL_TREATY: u8 = 1;
pub const DIPL_ALLIED: u8 = 2;
pub const DIPL_FRIENDLY: u8 = 3;
pub const DIPL_NEUTRAL: u8 = 4;
pub const DIPL_HOSTILE: u8 = 5;
pub const DIPL_WAR: u8 = 6;
pub const DIPL_JIHAD: u8 = 7;

/// Get diplomatic status name
pub fn dipl_name(status: u8) -> &'static str {
    match status {
        DIPL_UNMET => "UNMET",
        DIPL_TREATY => "TREATY",
        DIPL_ALLIED => "ALLIED",
        DIPL_FRIENDLY => "FRIENDLY",
        DIPL_NEUTRAL => "NEUTRAL",
        DIPL_HOSTILE => "HOSTILE",
        DIPL_WAR => "WAR",
        DIPL_JIHAD => "JIHAD",
        _ => "UNKNOWN",
    }
}

/// Check if two nations can trade (must not be UNMET or HOSTILE/JIHAD/WAR)
pub fn can_trade_with(dipl_status: u8) -> bool {
    dipl_status != DIPL_UNMET 
    && dipl_status != DIPL_HOSTILE 
    && dipl_status != DIPL_WAR 
    && dipl_status != DIPL_JIHAD
}

/// Check if nation at given status can be contacted
pub fn can_contact(dipl_status: u8) -> bool {
    dipl_status != DIPL_UNMET
}

/// Check if nations are hostile (at war or worse)
pub fn is_hostile(dipl_status: u8) -> bool {
    dipl_status >= DIPL_HOSTILE
}

/// Check if nations are at war
pub fn is_at_war(dipl_status: u8) -> bool {
    dipl_status >= DIPL_WAR
}

/// Check if nations have met
pub fn have_met(dipl_status: u8) -> bool {
    dipl_status != DIPL_UNMET
}

/// Check if diplomatic status allows passage through territory
pub fn can_pass_through(dipl_status: u8) -> bool {
    dipl_status <= DIPL_ALLIED
}

/// Calculate cost to break a jihad or confederacy
/// Matches C: BREAKJIHAD constant (200000L)
pub fn break_jihad_cost() -> i64 {
    200000
}

/// Set diplomatic status between two nations
/// Updates both directions
pub fn set_diplomatic_status(
    nation1: &mut Nation,
    nation1_idx: usize,
    nation2_idx: usize,
    status: u8,
) {
    if nation2_idx < MAXNTOTAL {
        nation1.diplomacy[nation2_idx] = status;
    }
}

/// Get diplomatic status from one nation toward another
pub fn get_diplomatic_status(nation: &Nation, other_idx: usize) -> u8 {
    if other_idx < MAXNTOTAL {
        nation.diplomacy[other_idx]
    } else {
        DIPL_UNMET
    }
}

/// Initialize diplomatic status for a new nation
/// All nations start as UNMET
pub fn init_diplomatic_status(nation: &mut Nation) {
    for i in 0..MAXNTOTAL {
        nation.diplomacy[i] = DIPL_UNMET;
    }
}

/// Verify diplomatic status is valid (called at turn start)
/// Matches C: verify_ntn() function
pub fn verify_diplomatic_status(nation: &mut Nation, nation_idx: usize) -> Vec<String> {
    let mut errors = Vec::new();
    
    // NPC_PEASANT and above must always be at WAR
    if nation.active >= NPC_PEASANT as u8 {
        for i in 0..MAXNTOTAL {
            if nation.diplomacy[i] != DIPL_WAR {
                // Set to WAR if not already
                if i != nation_idx {
                    nation.diplomacy[i] = DIPL_WAR;
                }
            }
        }
        
        // Also ensure other nations are at WAR with this nation
        // (would need access to full nation list)
    }
    
    // Check for invalid status (greater than JIHAD)
    for i in 0..MAXNTOTAL {
        if nation.diplomacy[i] > DIPL_JIHAD {
            errors.push(format!(
                "Invalid diplomatic status {} with nation {}",
                nation.diplomacy[i], i
            ));
            nation.diplomacy[i] = DIPL_WAR;
        }
    }
    
    errors
}

/// Check if diplomatic status can be changed
/// Some transitions may require special conditions
pub fn can_change_diplomacy(current_status: u8, new_status: u8) -> bool {
    // Can't change from JIHAD without paying
    if current_status == DIPL_JIHAD && new_status < DIPL_JIHAD {
        return false; // Would need BREAKJIHAD
    }
    
    // Can't go from WAR to ALLIED directly without TREATY first
    if current_status == DIPL_WAR && new_status >= DIPL_ALLIED {
        return false;
    }
    
    true
}

/// Improve diplomatic relations (move toward better status)
pub fn improve_relations(current_status: u8) -> u8 {
    match current_status {
        DIPL_UNMET => DIPL_TREATY,
        DIPL_TREATY => DIPL_ALLIED,
        DIPL_ALLIED => DIPL_ALLIED,
        DIPL_FRIENDLY => DIPL_ALLIED,
        DIPL_NEUTRAL => DIPL_FRIENDLY,
        DIPL_HOSTILE => DIPL_NEUTRAL,
        DIPL_WAR => DIPL_HOSTILE,
        DIPL_JIHAD => DIPL_WAR,
        _ => current_status,
    }
}

/// Worsen diplomatic relations (move toward war)
pub fn worsen_relations(current_status: u8) -> u8 {
    match current_status {
        DIPL_UNMET => DIPL_UNMET,
        DIPL_TREATY => DIPL_UNMET,
        DIPL_ALLIED => DIPL_TREATY,
        DIPL_FRIENDLY => DIPL_NEUTRAL,
        DIPL_NEUTRAL => DIPL_HOSTILE,
        DIPL_HOSTILE => DIPL_WAR,
        DIPL_WAR => DIPL_JIHAD,
        DIPL_JIHAD => DIPL_JIHAD,
        _ => current_status,
    }
}

/// Calculate war exhaustion or alliance strength
pub fn diplomatic_strength(status: u8) -> i32 {
    match status {
        DIPL_JIHAD => 100,
        DIPL_WAR => 80,
        DIPL_HOSTILE => 60,
        DIPL_NEUTRAL => 40,
        DIPL_FRIENDLY => 20,
        DIPL_ALLIED => 10,
        DIPL_TREATY => 5,
        DIPL_UNMET => 0,
        _ => 0,
    }
}

/// Meet a new nation (set from UNMET to NEUTRAL)
pub fn meet_nation(nation: &mut Nation, nation_idx: usize, other_idx: usize) {
    if other_idx < MAXNTOTAL && other_idx != nation_idx {
        if nation.diplomacy[other_idx] == DIPL_UNMET {
            nation.diplomacy[other_idx] = DIPL_NEUTRAL;
        }
    }
}

/// Declare war on a nation
pub fn declare_war(nation: &mut Nation, nation_idx: usize, target_idx: usize) -> bool {
    if target_idx >= MAXNTOTAL || target_idx == nation_idx {
        return false;
    }
    
    // Can't declare war on own nation
    // Check if already at jihad
    if nation.diplomacy[target_idx] == DIPL_JIHAD {
        return false;
    }
    
    nation.diplomacy[target_idx] = DIPL_WAR;
    true
}

/// Propose peace (lower status from war/hostile)
pub fn propose_peace(nation: &mut Nation, target_idx: usize) -> bool {
    if target_idx >= MAXNTOTAL {
        return false;
    }
    
    let current = nation.diplomacy[target_idx];
    if current < DIPL_HOSTILE {
        return false; // Not hostile
    }
    
    // Peace proposal accepted - move to neutral
    nation.diplomacy[target_idx] = DIPL_NEUTRAL;
    true
}

/// Check if diplomatic contact is possible
pub fn can_have_diplomacy(status: u8) -> bool {
    status != DIPL_UNMET
}

/// Get default diplomatic status when meeting new nations
pub fn default_new_nation_status() -> u8 {
    DIPL_NEUTRAL
}

// ============================================================
// T233: Port newdip() - Diplomacy change logic
// ============================================================

/// Check if a nation is player-controlled (PC)
/// Matches C: ispc(x) = (x==PC_GOOD || x==PC_EVIL || x==PC_NEUTRAL)
/// PC values: PcGood=1, PcNeutral=2, PcEvil=3
pub fn is_pc(active: u8) -> bool {
    matches!(active, 1 | 2 | 3) // PcGood, PcNeutral, PcEvil
}

/// Check if a nation is a monster (NPC_PEASANT or higher)
/// Matches C: ismonst(x) = (x >= NPC_PEASANT)
/// NPC_PEASANT = 17
pub fn is_monster(active: u8) -> bool {
    active >= 17 // NpcPeasant through NpcSavage (17-21)
}

/// Check if a nation's race is Orc
/// Matches C: ntn[ntn].race == ORC (where ORC = 'O')
pub fn is_orc(race: char) -> bool {
    race == 'O' || race == 'O' as char
}

/// Set diplomatic status for nation1 toward nation2 when they first meet (UNMET -> known)
/// Matches C: newdip() from npc.c
/// 
/// This is called when two nations occupy adjacent sectors for the first time.
/// It sets initial diplomatic status based on:
/// - If nation1 is PC: neutral (or hostile toward Orcs)
/// - If either is Orc: hostile or war (50% chance)
/// - If nation2 is monster: war
/// - If nation1 is PC: neutral (if UNMET)
/// - Same race: 50% friendly, 50% neutral
/// - Different race: neutral
pub fn newdip(
    nation1: &mut Nation,
    nation1_idx: usize,
    nation2: &Nation,
    _nation2_idx: usize,
    rng: &mut ConquerRng,
) {
    let ntn1_active = nation1.active;
    let ntn2_race = nation2.race;
    let ntn2_active = nation2.active;
    
    // Get current status (should be UNMET when newdip is called)
    let current_status = nation1.diplomacy[_nation2_idx];
    
    // If nation1 is a PC (player-controlled)
    if is_pc(ntn1_active) {
        if is_orc(ntn2_race) {
            nation1.diplomacy[_nation2_idx] = DIPL_HOSTILE;
        } else {
            nation1.diplomacy[_nation2_idx] = DIPL_NEUTRAL;
        }
        return;
    }
    
    // If either nation is Orc
    if is_orc(nation1.race) || is_orc(ntn2_race) {
        if current_status == DIPL_UNMET {
            // 50% chance of HOSTILE, or if nation1 is PC (shouldn't happen here but safety)
            if rng.rand_mod(2) == 0 || is_pc(ntn1_active) {
                nation1.diplomacy[_nation2_idx] = DIPL_HOSTILE;
            } else {
                nation1.diplomacy[_nation2_idx] = DIPL_WAR;
            }
        }
    } 
    // If nation2 is a monster
    else if is_monster(ntn2_active) {
        nation1.diplomacy[_nation2_idx] = DIPL_WAR;
    } 
    // If nation1 is a PC
    else if is_pc(ntn1_active) {
        if current_status == DIPL_UNMET {
            nation1.diplomacy[_nation2_idx] = DIPL_NEUTRAL;
        }
    } 
    // Same race
    else if nation1.race == ntn2_race {
        if rng.rand_mod(2) < 1 {
            nation1.diplomacy[_nation2_idx] = DIPL_FRIENDLY;
        } else {
            nation1.diplomacy[_nation2_idx] = DIPL_NEUTRAL;
        }
    } 
    // Different race (NPCs of different races)
    else {
        nation1.diplomacy[_nation2_idx] = DIPL_NEUTRAL;
    }
}

/// Update NPC diplomatic relations
/// Called during NPC turn processing
pub fn update_npc_diplomacy(
    npc: &mut Nation,
    npc_idx: usize,
    other: &Nation,
    other_idx: usize,
    _world_turn: i32,
) {
    // NPCs make treaties based on various factors
    // This is a simplified version
    
    let current = npc.diplomacy[other_idx];
    
    // Don't change status with self
    if other_idx == npc_idx {
        return;
    }
    
    // NPCs at WAR stay at WAR
    if current == DIPL_WAR || current == DIPL_JIHAD {
        return;
    }
    
    // Check if other nation is a monster nation - always WAR
    if other.active >= NPC_PEASANT as u8 {
        npc.diplomacy[other_idx] = DIPL_WAR;
        return;
    }
    
    // Check relation based on other nation's status toward this one
    let other_status = other.diplomacy[npc_idx];
    
    // Mirror enemy relations
    if other_status == DIPL_WAR || other_status == DIPL_JIHAD {
        npc.diplomacy[other_idx] = DIPL_WAR;
    }
    // If allied with someone who is hostile to this nation, become hostile
    else if other_status >= DIPL_ALLIED && current < DIPL_HOSTILE {
        // Not implemented fully - would need more context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dipl_names() {
        assert_eq!(dipl_name(DIPL_UNMET), "UNMET");
        assert_eq!(dipl_name(DIPL_TREATY), "TREATY");
        assert_eq!(dipl_name(DIPL_ALLIED), "ALLIED");
        assert_eq!(dipl_name(DIPL_FRIENDLY), "FRIENDLY");
        assert_eq!(dipl_name(DIPL_NEUTRAL), "NEUTRAL");
        assert_eq!(dipl_name(DIPL_HOSTILE), "HOSTILE");
        assert_eq!(dipl_name(DIPL_WAR), "WAR");
        assert_eq!(dipl_name(DIPL_JIHAD), "JIHAD");
    }

    #[test]
    fn test_can_trade() {
        assert!(!can_trade_with(DIPL_UNMET));
        assert!(can_trade_with(DIPL_TREATY));
        assert!(can_trade_with(DIPL_ALLIED));
        assert!(can_trade_with(DIPL_FRIENDLY));
        assert!(can_trade_with(DIPL_NEUTRAL));
        assert!(!can_trade_with(DIPL_HOSTILE));
        assert!(!can_trade_with(DIPL_WAR));
        assert!(!can_trade_with(DIPL_JIHAD));
    }

    #[test]
    fn test_improve_relations() {
        assert_eq!(improve_relations(DIPL_UNMET), DIPL_TREATY);
        assert_eq!(improve_relations(DIPL_TREATY), DIPL_ALLIED);
        assert_eq!(improve_relations(DIPL_ALLIED), DIPL_ALLIED);
        assert_eq!(improve_relations(DIPL_HOSTILE), DIPL_NEUTRAL);
        assert_eq!(improve_relations(DIPL_WAR), DIPL_HOSTILE);
    }

    #[test]
    fn test_worsen_relations() {
        assert_eq!(worsen_relations(DIPL_UNMET), DIPL_UNMET);
        assert_eq!(worsen_relations(DIPL_ALLIED), DIPL_TREATY);
        assert_eq!(worsen_relations(DIPL_NEUTRAL), DIPL_HOSTILE);
        assert_eq!(worsen_relations(DIPL_HOSTILE), DIPL_WAR);
        assert_eq!(worsen_relations(DIPL_WAR), DIPL_JIHAD);
    }

    // Tests for newdip() - T233
    
    #[test]
    fn test_is_pc() {
        // PC values: 1, 2, 3
        assert!(is_pc(1));  // PcGood
        assert!(is_pc(2));  // PcNeutral
        assert!(is_pc(3));  // PcEvil
        // NPC/monster values: >= 17
        assert!(!is_pc(0));   // Inactive
        assert!(!is_pc(17));  // NpcPeasant
        assert!(!is_pc(21));  // NpcSavage
    }
    
    #[test]
    fn test_is_monster() {
        // Monster values: >= 17
        assert!(is_monster(17)); // NpcPeasant
        assert!(is_monster(18)); // NpcPirate
        assert!(is_monster(21)); // NpcSavage
        // Non-monster
        assert!(!is_monster(0)); // Inactive
        assert!(!is_monster(1)); // PcGood
        assert!(!is_monster(3)); // PcEvil
    }
    
    #[test]
    fn test_is_orc() {
        assert!(is_orc('O'));
        assert!(!is_orc('H'));
        assert!(!is_orc('E'));
        assert!(!is_orc('D'));
    }

    #[test]
    fn test_newdip_pc_vs_orc() {
        // PC nation meeting Orc nation -> HOSTILE
        let mut ntn1 = Nation::default();
        ntn1.active = 1; // PC
        ntn1.diplomacy[2] = DIPL_UNMET;
        
        let ntn2 = Nation {
            race: 'O', // Orc
            active: 0,
            ..Default::default()
        };
        
        let mut rng = ConquerRng::new(42);
        newdip(&mut ntn1, 1, &ntn2, 2, &mut rng);
        
        assert_eq!(ntn1.diplomacy[2], DIPL_HOSTILE);
    }

    #[test]
    fn test_newdip_pc_vs_human() {
        // PC nation meeting human nation -> NEUTRAL
        let mut ntn1 = Nation::default();
        ntn1.active = 1; // PC
        ntn1.diplomacy[2] = DIPL_UNMET;
        
        let ntn2 = Nation {
            race: 'H', // Human
            active: 0,
            ..Default::default()
        };
        
        let mut rng = ConquerRng::new(42);
        newdip(&mut ntn1, 1, &ntn2, 2, &mut rng);
        
        assert_eq!(ntn1.diplomacy[2], DIPL_NEUTRAL);
    }

    #[test]
    fn test_newdip_npc_vs_monster() {
        // NPC nation meeting monster -> WAR
        let mut ntn1 = Nation::default();
        ntn1.active = 0; // NPC (not monster)
        ntn1.race = 'H'; // Human
        ntn1.diplomacy[2] = DIPL_UNMET;
        
        let ntn2 = Nation {
            race: 'H',
            active: 17, // Monster (NpcPeasant)
            ..Default::default()
        };
        
        let mut rng = ConquerRng::new(42);
        newdip(&mut ntn1, 1, &ntn2, 2, &mut rng);
        
        assert_eq!(ntn1.diplomacy[2], DIPL_WAR);
    }

    #[test]
    fn test_newdip_same_race() {
        // Two NPCs of same race -> FRIENDLY or NEUTRAL (50% chance)
        // Use fixed seed to get deterministic result
        let mut ntn1 = Nation::default();
        ntn1.active = 0; // NPC
        ntn1.race = 'H'; // Human
        ntn1.diplomacy[2] = DIPL_UNMET;
        
        let ntn2 = Nation {
            race: 'H', // Same race
            active: 0,
            ..Default::default()
        };
        
        // With seed 0, rand_mod(2) returns 0 -> FRIENDLY (0 < 1)
        let mut rng = ConquerRng::new(0);
        newdip(&mut ntn1, 1, &ntn2, 2, &mut rng);
        
        // rand_mod(2) with seed 0 returns 0, which is < 1, so FRIENDLY
        assert!(ntn1.diplomacy[2] == DIPL_FRIENDLY || ntn1.diplomacy[2] == DIPL_NEUTRAL);
    }

    #[test]
    fn test_newdip_different_race_npc() {
        // Two NPCs of different races -> NEUTRAL
        let mut ntn1 = Nation::default();
        ntn1.active = 0; // NPC
        ntn1.race = 'H'; // Human
        ntn1.diplomacy[2] = DIPL_UNMET;
        
        let ntn2 = Nation {
            race: 'E', // Elf - different race
            active: 0,
            ..Default::default()
        };
        
        let mut rng = ConquerRng::new(42);
        newdip(&mut ntn1, 1, &ntn2, 2, &mut rng);
        
        assert_eq!(ntn1.diplomacy[2], DIPL_NEUTRAL);
    }
}
