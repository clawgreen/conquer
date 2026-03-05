// conquer-engine/src/events.rs — Random events ported from update.c, randeven.c
//
// T243-T250: Random events, weather, tax revolts, volcanoes
//
// Events: storms, plagues, revolts, discoveries, etc.

use conquer_core::*;
use conquer_core::tables::*;
use crate::rng::ConquerRng;

/// Event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    None,
    Storm,        // Storm damages fleets
    Plague,      // Population plague
    Revolt,      // Tax revolt
    Discovery,   // Gold/metal discovery
    Volcano,     // Volcano eruption
    Flood,       // Flood damages crops
    Drought,     // Drought reduces food
    Bounty,      // Unexpected bounty
    Raid,        // Barbarian raid
    TradeWind,   // Good trade winds
}

/// Event result
#[derive(Debug, Clone)]
pub struct EventResult {
    pub event_type: EventType,
    pub affected_nation: Option<u8>,
    pub affected_x: Option<u8>,
    pub affected_y: Option<u8>,
    pub damage: i64,
    pub message: String,
}

/// Chance constants
pub const STORM_CHANCE: i32 = 3;      // 3% chance per turn
pub const VOLCANO_CHANCE: i32 = 20;   // 20% chance (defined in header.h)
pub const REVOLT_CHANCE: i32 = 25;    // 25% /turn (PREVOLT in header.h)

/// Generate random event
/// Matches C: randomevent() function
pub fn generate_random_event(
    rng: &mut ConquerRng,
    nation: &Nation,
    nation_idx: usize,
    x: u8,
    y: u8,
    sector: &Sector,
) -> Option<EventResult> {
    // Roll for event (RANEVENT must be defined in C)
    let roll = rng.rand() % 100;
    
    // Storm (for navies in water)
    if sector.altitude == Altitude::Water as u8 {
        if rng.rand() % 100 < STORM_CHANCE {
            return Some(EventResult {
                event_type: EventType::Storm,
                affected_nation: Some(nation_idx as u8),
                affected_x: Some(x as u8),
                affected_y: Some(y as u8),
                damage: 0,
                message: format!("Storm strikes fleet at ({}, {})!", x, y),
            });
        }
    }
    
    // Volcano (random sector check)
    if sector.vegetation == Vegetation::Volcano as u8 {
        if rng.rand() % 100 < VOLCANO_CHANCE {
            return Some(EventResult {
                event_type: EventType::Volcano,
                affected_nation: None,
                affected_x: Some(x as u8),
                affected_y: Some(y as u8),
                damage: 0,
                message: format!("Volcano erupts at ({}, {})!", x, y),
            });
        }
    }
    
    // Revolt check (based on PREVOLT)
    // This is done at nation level, not per sector
    
    // Tax revolt chance
    // Done in updmil() in C
    
    None
}

/// Check for tax revolt in a nation
/// Matches C: logic in update.c
pub fn check_tax_revolt(
    rng: &mut ConquerRng,
    nation: &Nation,
    gold_in_treasury: i64,
    population: i64,
) -> bool {
    // Check if nation has enough gold for population
    // Revolt happens if gold < needed
    // Base formula: need 10 gold per 100 people per turn
    
    let needed_gold = population * 10 / 100;
    
    // High treasury reduces revolt chance
    let treasury_ratio = if needed_gold > 0 {
        (gold_in_treasury * 100) / needed_gold
    } else {
        100
    };
    
    // If treasury is very low (< 25% of needed), high chance
    // Otherwise roll against PREVOLT (25%)
    if treasury_ratio < 25 {
        // Certain revolt
        return true;
    }
    
    // Roll for revolt
    let revolt_roll = rng.rand() % 100;
    revolt_roll < REVOLT_CHANCE
}

/// Process revolt damage
/// Returns amount of gold lost
pub fn process_revolt(
    nation: &mut Nation,
    nation_idx: usize,
    gold_lost: i64,
) -> EventResult {
    nation.treasury_gold = nation.treasury_gold.saturating_sub(gold_lost);
    
    EventResult {
        event_type: EventType::Revolt,
        affected_nation: Some(nation_idx as u8),
        affected_x: None,
        affected_y: None,
        damage: gold_lost,
        message: format!(
            "TAX REVOLT! {} loses {} gold in riots!",
            nation.name, gold_lost
        ),
    }
}

/// Storm damage calculation
/// Matches C: STORMS logic
pub fn calculate_storm_damage(
    navy: &Navy,
    is_in_harbor: bool,
) -> i32 {
    if is_in_harbor {
        return 0; // Safe in harbor
    }
    
    // Calculate ship losses based on ship type
    let mut damage = 0;
    
    // Warships more susceptible
    damage += navy.warships as i32 * 2;
    damage += navy.galleys as i32 * 2;
    damage += navy.merchant as i32;
    
    // Crew may be lost
    // Would need to update navy.crew
    
    damage
}

/// Process storm on a fleet
pub fn process_storm(
    navy: &mut Navy,
    is_in_harbor: bool,
) -> EventResult {
    if is_in_harbor {
        return EventResult {
            event_type: EventType::Storm,
            affected_nation: None,
            affected_x: Some(navy.x),
            affected_y: Some(navy.y),
            damage: 0,
            message: "Fleet rides out storm in harbor.".to_string(),
        };
    }
    
    let damage = calculate_storm_damage(navy, false);
    
    EventResult {
        event_type: EventType::Storm,
        affected_nation: None,
        affected_x: Some(navy.x),
        affected_y: Some(navy.y),
        damage: damage as i64,
        message: format!("Storm damages fleet! {} ships lost!", damage),
    }
}

/// Volcano damage to adjacent sectors
pub fn volcano_damage(
    sectors: &mut [[Sector; MAPY as usize]; MAPX as usize],
    vx: i32,
    vy: i32,
) -> Vec<EventResult> {
    let mut results = Vec::new();
    
    // Check 3x3 area around volcano
    for dx in -1..=1 {
        for dy in -1..=1 {
            let x = vx + dx;
            let y = vy + dy;
            
            if x < 0 || y < 0 || x >= MAPX as i32 || y >= MAPY as i32 {
                continue;
            }
            
            let sector = &mut sectors[x as usize][y as usize];
            
            // Lava destroys everything
            if sector.owner != 0 {
                let old_owner = sector.owner;
                sector.owner = 0;
                sector.people = 0;
                
                results.push(EventResult {
                    event_type: EventType::Volcano,
                    affected_nation: Some(old_owner),
                    affected_x: Some(x as u8),
                    affected_y: Some(y as u8),
                    damage: 0,
                    message: format!(
                        "Volcanic eruption destroys sector ({}, {})!",
                        x, y
                    ),
                });
            }
            
            // Vegetation destroyed
            if sector.vegetation != Vegetation::Volcano as u8 {
                sector.vegetation = Vegetation::Desert as u8;
            }
        }
    }
    
    results
}

/// Population plague effect
pub fn plague_effect(
    sector: &mut Sector,
    mortality_rate: i32,  // 0-100 percentage
) -> i64 {
    if sector.people <= 0 {
        return 0;
    }
    
    let deaths = (sector.people as i64 * mortality_rate as i64) / 100;
    sector.people = sector.people.saturating_sub(deaths as i64);
    
    deaths
}

/// Random gold/metal discovery in a sector
pub fn random_discovery(
    rng: &mut ConquerRng,
    sector: &Sector,
) -> Option<(i32, i32)> {
    // Check for discovery chance (FINDPERCENT = 1%)
    if rng.rand() % 100 >= 1 {
        return None;
    }
    
    // Check if sector has potential for discovery
    // Mountains and hills may have metal
    // Specific veg types may have gold
    
    let mut metal_found = 0;
    let mut gold_found = 0;
    
    // Based on altitude and vegetation
    let alt = sector.altitude;
    if alt == Altitude::Mountain as u8 || alt == Altitude::Hill as u8 {
        metal_found = rng.rand() % 10 + 1;
    }
    
    // Check for gold based on trade good
    if sector.trade_good > 0 {
        gold_found = rng.rand() % 5 + 1;
    }
    
    if metal_found > 0 || gold_found > 0 {
        Some((metal_found, gold_found))
    } else {
        None
    }
}

/// Calculate food production bonus from good weather
pub fn weather_bonus(
    season: Season,
    vegetation: u8,
) -> i32 {
    // Summer bonus for farms
    if season == Season::Summer {
        if vegetation == Vegetation::Good as u8 
            || vegetation == Vegetation::Wood as u8 {
            return 20; // 20% bonus
        }
    }
    
    // Spring good for planting
    if season == Season::Spring {
        return 10;
    }
    
    0
}

/// Random barbarian raid
pub fn barbarian_raid(
    rng: &mut ConquerRng,
    sector: &mut Sector,
    nation_idx: u8,
    sector_x: u8,
    sector_y: u8,
) -> Option<EventResult> {
    // Check if sector can be raided (must have population or be near border)
    if sector.people < 100 {
        return None;
    }
    
    // 10% chance of raid per turn for vulnerable sectors
    if rng.rand() % 100 >= 10 {
        return None;
    }
    
    // Calculate raid damage
    let stolen_gold = sector.people / 10;
    let stolen_food = sector.people / 20;
    
    // Reduce sector population
    sector.people = sector.people.saturating_sub(stolen_gold / 10);
    
    Some(EventResult {
        event_type: EventType::Raid,
        affected_nation: Some(nation_idx),
        affected_x: Some(sector_x),
        affected_y: Some(sector_y),
        damage: stolen_gold,
        message: format!(
            "Barbarians raid sector ({}, {})! {} gold stolen!",
            sector_x, sector_y, stolen_gold
        ),
    })
}

/// Process random events for a nation
/// Called during turn update
pub fn process_nation_events(
    rng: &mut ConquerRng,
    nation: &mut Nation,
    nation_idx: usize,
    sectors: &mut [[Sector; MAPY as usize]; MAPX as usize],
) -> Vec<EventResult> {
    let mut results = Vec::new();
    
    // Check for tax revolt
    if check_tax_revolt(rng, nation, nation.treasury_gold, nation.total_civ) {
        // Calculate gold lost (half of treasury)
        let gold_lost = nation.treasury_gold / 2;
        results.push(process_revolt(nation, nation_idx, gold_lost));
    }
    
    // Process each sector for random events
    for x in 0..MAPX {
        for y in 0..MAPY {
            let sector = &mut sectors[x as usize][y as usize];
            
            // Skip unowned sectors
            if sector.owner != nation_idx as u8 {
                continue;
            }
            
            // Check for barbarian raid
            if let Some(raid_result) = barbarian_raid(rng, sector, nation_idx as u8, x as u8, y as u8) {
                continue;
            }
            
            // Check for random discovery
            if let Some((metal, gold)) = random_discovery(rng, sector) {
                if metal > 0 {
                    sector.metal = sector.metal.saturating_add(metal as u8);
                }
                if gold > 0 {
                    sector.jewels = sector.jewels.saturating_add(gold as u8);
                }
                
                results.push(EventResult {
                    event_type: EventType::Discovery,
                    affected_nation: Some(nation_idx as u8),
                    affected_x: Some(x as u8),
                    affected_y: Some(y as u8),
                    damage: 0,
                    message: format!(
                        "Discovery! +{} metal, +{} jewels at ({}, {})",
                        metal, gold, x, y
                    ),
                });
            }
            
            // Check for barbarian raid
            if let Some(raid_result) = barbarian_raid(rng, sector, nation_idx as u8, x as u8, y as u8) {
                results.push(raid_result);
            }
        }
    }
    
    // Process navies for storms
    for navy in &mut nation.navies {
        if navy.has_ships() {
            // Check if in harbor (need sector info)
            let is_harbor = is_sector_harbor(sectors, navy.x, navy.y);
            if !is_harbor && rng.rand() % 100 < STORM_CHANCE {
                results.push(process_storm(navy, false));
            }
        }
    }
    
    results
}

/// Check if a sector is a harbor (next to water)
fn is_sector_harbor(
    sectors: &[[Sector; MAPY as usize]; MAPX as usize],
    x: u8,
    y: u8,
) -> bool {
    let x = x as i32;
    let y = y as i32;
    
    for dx in -1..=1 {
        for dy in -1..=1 {
            let nx = x + dx;
            let ny = y + dy;
            
            if nx < 0 || ny < 0 || nx >= MAPX as i32 || ny >= MAPY as i32 {
                continue;
            }
            
            if sectors[nx as usize][ny as usize].altitude == Altitude::Water as u8 {
                return true;
            }
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storm_damage_no_harbor() {
        let navy = Navy {
            warships: 5,
            merchant: 2,
            galleys: 3,
            ..Default::default()
        };
        
        let damage = calculate_storm_damage(&navy, false);
        assert!(damage > 0);
    }

    #[test]
    fn test_storm_damage_harbor() {
        let navy = Navy {
            warships: 5,
            ..Default::default()
        };
        
        let damage = calculate_storm_damage(&navy, true);
        assert_eq!(damage, 0);
    }

    #[test]
    fn test_plague_effect() {
        let mut sector = Sector {
            people: 1000,
            ..Default::default()
        };
        
        let deaths = plague_effect(&mut sector, 50);
        assert_eq!(deaths, 500);
        assert_eq!(sector.people, 500);
    }
}
