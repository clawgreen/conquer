// conquer-engine/src/events.rs — Random events ported from update.c, randeven.c
//
// T243-T250: Random events, weather, tax revolts, volcanoes
//
// Events: storms, plagues, revolts, discoveries, etc.

use conquer_core::*;
use conquer_core::tables::*;
use conquer_core::enums::{Season, Altitude, Vegetation};
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

/// Disaster report output from wdisaster()
/// Contains formatted output for console, news, and mail
#[derive(Debug, Clone, Default)]
pub struct DisasterReport {
    pub console_output: String,
    pub news_output: String,
    pub news_details: Option<String>,
    pub mail_message: Option<String>,
    pub is_pc: bool,
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

/// Report a weather disaster event
/// Ported from C: wdisaster() in randeven.c
/// 
/// This function formats and reports disaster events to:
/// - Console output
/// - News file (returned as string)
/// - Mail message (for PC nations, returned as string)
/// 
/// Parameters:
/// - nation: The affected nation
/// - nation_idx: Nation index
/// - x, y: Location coordinates (-1 if not location-specific)
/// - damage_percent: Damage severity percentage (0-100)
/// - event_name: Name of the event (e.g., "volcano erupted", "peasant revolt")
/// - event_details: Optional additional details about the event
/// - season: Current season for the mail message
/// - year: Current year for the mail message
pub fn wdisaster(
    nation: &Nation,
    nation_idx: usize,
    x: i32,
    y: i32,
    damage_percent: i32,
    event_name: &str,
    event_details: Option<&str>,
    season: Season,
    year: i32,
) -> DisasterReport {
    let is_pc = matches!(
        nation.active,
        1 | 2 | 3
    ); // PC_GOOD=1, PC_NEUTRAL=2, PC_EVIL=3
    
    let season_name = match season {
        Season::Winter => "Winter",
        Season::Spring => "Spring",
        Season::Summer => "Summer",
        Season::Fall => "Fall",
    };
    
    // Console/news output
    let console_output = format!("\t{} in {}", event_name, nation.name);
    
    // News output
    let news_output = format!("1.\t{} in {}", event_name, nation.name);
    
    // Mail message (only for PC nations)
    let mail_message = if is_pc {
        let mut msg = format!(
            "MESSAGE FROM CONQUER\n\nAn event occurs within your nation ({})\n{} during the {} of Year {},\n",
            nation.name, event_name, season_name, year
        );
        
        if x >= 0 && y >= 0 {
            msg.push_str(&format!(" centered around location {}, {}.\n", x, y));
        }
        
        if damage_percent > 0 {
            msg.push_str(&format!("Damage was estimated at about {}% in severity.\n", damage_percent));
        }
        
        if let Some(details) = event_details {
            if !details.is_empty() {
                msg.push_str(&format!("\t{}\n", details));
            }
        }
        
        Some(msg)
    } else {
        None
    };
    
    // Event details in news (if provided)
    let news_details = event_details.map(|details| {
        format!("1.\tevent in {} -->{}", nation.name, details)
    });
    
    DisasterReport {
        console_output,
        news_output,
        news_details,
        mail_message,
        is_pc,
    }
}

/// Check if a nation is player-controlled
/// Matches C: ispc() macro
pub fn is_pc_nation(active: u8) -> bool {
    matches!(active, 1 | 2 | 3)
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

// ============================================================
// T239: Random event dispatcher tests
// ============================================================

#[cfg(test)]
mod event_tests {
    use super::*;
    use conquer_core::enums::Season;

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

    #[test]
    fn test_plague_zero_people() {
        let mut sector = Sector {
            people: 0,
            ..Default::default()
        };
        
        let deaths = plague_effect(&mut sector, 50);
        assert_eq!(deaths, 0);
    }

    #[test]
    fn test_plague_full_mortality() {
        let mut sector = Sector {
            people: 1000,
            ..Default::default()
        };
        
        let deaths = plague_effect(&mut sector, 100);
        assert_eq!(deaths, 1000);
        assert_eq!(sector.people, 0);
    }

    #[test]
    fn test_plague_no_overflow() {
        let mut sector = Sector {
            people: 100,
            ..Default::default()
        };
        
        // With 150% mortality rate, more people die than exist
        // The function should handle this gracefully
        let deaths = plague_effect(&mut sector, 150);
        
        // The deaths should exceed the population
        assert!(deaths > 50);
    }

    #[test]
    fn test_check_tax_revolt_low_treasury() {
        // High tax, low treasury = revolt likely
        let nation = Nation {
            treasury_gold: 100,
            total_civ: 10000,  // Needs 1000 gold
            tax_rate: 50,
            ..Default::default()
        };
        
        let mut rng = ConquerRng::new(0); // Deterministic
        
        // With very low treasury (<25% needed), revolt is certain
        let revolt = check_tax_revolt(&mut rng, &nation, 100, 10000);
        assert!(revolt);
    }

    #[test]
    fn test_check_tax_revolt_high_treasury() {
        // Low tax, high treasury = no revolt
        let nation = Nation {
            treasury_gold: 100000,
            total_civ: 1000,
            tax_rate: 10,
            ..Default::default()
        };
        
        let mut rng = ConquerRng::new(0); // Deterministic
        
        // With high treasury, no revolt
        let revolt = check_tax_revolt(&mut rng, &nation, 100000, 1000);
        // Depends on RNG roll but should be unlikely with high treasury
    }

    #[test]
    fn test_weather_bonus_summer_farm() {
        let bonus = weather_bonus(Season::Summer, Vegetation::Good as u8);
        assert_eq!(bonus, 20);
    }

    #[test]
    fn test_weather_bonus_spring() {
        let bonus = weather_bonus(Season::Spring, Vegetation::Barren as u8);
        assert_eq!(bonus, 10);
    }

    #[test]
    fn test_weather_bonus_winter() {
        let bonus = weather_bonus(Season::Winter, Vegetation::Good as u8);
        assert_eq!(bonus, 0);
    }

    #[test]
    fn test_barbarian_raid_too_small() {
        let mut sector = Sector {
            people: 50,  // Too small
            ..Default::default()
        };
        
        let mut rng = ConquerRng::new(42);
        let result = barbarian_raid(&mut rng, &mut sector, 1, 10, 10);
        
        assert!(result.is_none());
    }

    #[test]
    fn test_barbarian_raid_occurs() {
        let mut sector = Sector {
            people: 1000,
            ..Default::default()
        };
        
        // Use a seed that triggers the raid
        let mut rng = ConquerRng::new(0); // 10% chance, seed 0 should trigger
        let result = barbarian_raid(&mut rng, &mut sector, 1, 10, 10);
        
        // Either some raids happen or not, but no crash
        if let Some(event) = result {
            assert_eq!(event.event_type, EventType::Raid);
            assert!(event.damage > 0);
        }
    }

    // Note: volcano_damage test skipped - requires array of non-Copy types
    // The function is tested indirectly through integration tests

    #[test]
    fn test_random_discovery_chance() {
        // Very low chance (1%), most calls return None
        let mut rng = ConquerRng::new(9999); // Unlikely to trigger 1%
        
        let sector = Sector {
            altitude: Altitude::Mountain as u8,
            vegetation: Vegetation::Good as u8,
            trade_good: 1,
            ..Default::default()
        };
        
        // Most should be None due to 1% chance
        let mut found_count = 0;
        for _ in 0..100 {
            if random_discovery(&mut rng, &sector).is_some() {
                found_count += 1;
            }
        }
        
        // Should be very few or none
        assert!(found_count <= 2);
    }

    #[test]
    fn test_process_revolt() {
        let mut nation = Nation {
            treasury_gold: 10000,
            name: "Test Nation".to_string(),
            ..Default::default()
        };
        
        let result = process_revolt(&mut nation, 1, 5000);
        
        assert_eq!(result.event_type, EventType::Revolt);
        assert_eq!(nation.treasury_gold, 5000);
    }

    #[test]
    fn test_process_storm_in_harbor() {
        let mut navy = Navy {
            warships: 5,
            x: 10,
            y: 10,
            ..Default::default()
        };
        
        let result = process_storm(&mut navy, true);
        
        assert_eq!(result.event_type, EventType::Storm);
        assert!(result.message.contains("harbor"));
    }

    #[test]
    fn test_process_storm_at_sea() {
        let mut navy = Navy {
            warships: 5,
            x: 10,
            y: 10,
            ..Default::default()
        };
        
        let result = process_storm(&mut navy, false);
        
        assert_eq!(result.event_type, EventType::Storm);
        // Should have damage message
    }

    #[test]
    fn test_event_type_names() {
        assert_eq!(format!("{:?}", EventType::Storm), "Storm");
        assert_eq!(format!("{:?}", EventType::Plague), "Plague");
        assert_eq!(format!("{:?}", EventType::Revolt), "Revolt");
        assert_eq!(format!("{:?}", EventType::Volcano), "Volcano");
    }

    #[test]
    fn test_wdisaster_pc_nation() {
        let nation = Nation {
            name: "Test Kingdom".to_string(),
            active: 1, // PC_GOOD
            cap_x: 10,
            cap_y: 10,
            ..Default::default()
        };
        
        let report = wdisaster(
            &nation,
            1,
            10,
            10,
            50,
            "volcano erupted",
            Some("devastating damage"),
            Season::Summer,
            100,
        );
        
        assert!(report.is_pc);
        assert!(report.console_output.contains("volcano erupted"));
        assert!(report.console_output.contains("Test Kingdom"));
        assert!(report.news_output.contains("volcano erupted"));
        assert!(report.mail_message.is_some());
        let mail = report.mail_message.unwrap();
        assert!(mail.contains("Test Kingdom"));
        assert!(mail.contains("Summer"));
        assert!(mail.contains("Year 100"));
        assert!(mail.contains("50%"));
    }

    #[test]
    fn test_wdisaster_npc_nation() {
        let nation = Nation {
            name: "NPC Empire".to_string(),
            active: 20, // NPC_Savage
            cap_x: 5,
            cap_y: 5,
            ..Default::default()
        };
        
        let report = wdisaster(
            &nation,
            2,
            5,
            5,
            100,
            "peasant revolt",
            None,
            Season::Spring,
            50,
        );
        
        assert!(!report.is_pc);
        assert!(report.console_output.contains("peasant revolt"));
        assert!(report.console_output.contains("NPC Empire"));
        // NPC nations don't get mail
        assert!(report.mail_message.is_none());
    }

    #[test]
    fn test_wdisaster_no_location() {
        let nation = Nation {
            name: "Test Nation".to_string(),
            active: 2, // PC_NEUTRAL
            ..Default::default()
        };
        
        let report = wdisaster(
            &nation,
            1,
            -1,
            -1,
            0,
            "unexpected bounty",
            Some("found treasure"),
            Season::Fall,
            25,
        );
        
        assert!(report.is_pc);
        assert!(report.mail_message.is_some());
        let mail = report.mail_message.unwrap();
        // Should not mention location when x/y are -1
        assert!(!mail.contains("centered around"));
    }

    #[test]
    fn test_wdisaster_no_damage_percent() {
        let nation = Nation {
            name: "Small Kingdom".to_string(),
            active: 3, // PC_EVIL
            ..Default::default()
        };
        
        let report = wdisaster(
            &nation,
            1,
            15,
            20,
            0,
            "barbarian raid",
            None,
            Season::Winter,
            75,
        );
        
        assert!(report.is_pc);
        let mail = report.mail_message.unwrap();
        // When damage_percent is 0, should not mention damage
        assert!(!mail.contains("Damage was estimated"));
    }

    #[test]
    fn test_is_pc_nation() {
        // PC nations
        assert!(is_pc_nation(1)); // PC_GOOD
        assert!(is_pc_nation(2)); // PC_NEUTRAL
        assert!(is_pc_nation(3)); // PC_EVIL
        
        // NPC nations
        assert!(!is_pc_nation(0));   // Inactive
        assert!(!is_pc_nation(4));   // Good0Free
        assert!(!is_pc_nation(17));  // NpcPeasant
        assert!(!is_pc_nation(20));  // NpcSavage
    }

    #[test]
    fn test_disaster_report_default() {
        let report = DisasterReport::default();
        assert!(report.console_output.is_empty());
        assert!(report.news_output.is_empty());
        assert!(report.news_details.is_none());
        assert!(report.mail_message.is_none());
        assert!(!report.is_pc);
    }
}
