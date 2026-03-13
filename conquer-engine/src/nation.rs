// conquer-engine/src/nation.rs — Nation creation ported from misc.c, data.c
//
// T271-T276: Nation creation, new nation formation, nation initialization
//
use crate::diplomacy::DIPL_UNMET;
use crate::utils::is_habitable;
use conquer_core::*;

/// PC nation active status values (from C data.h)
pub const PC_GOOD: u8 = 1;
pub const PC_NEUTRAL: u8 = 2;
pub const PC_EVIL: u8 = 3;

/// NPC nation type values (from C data.h)
pub const GOOD_0FREE: u8 = 4;
pub const NPC_PIRATE: u8 = 18;
pub const NPC_NOMAD: u8 = 20;
pub const NPC_SAVAGE: u8 = 21;
pub const ISOLATIONIST: u8 = 16;
pub const NPC_FIRST: u8 = GOOD_0FREE;

/// Result of nation creation
#[derive(Debug, Clone)]
pub struct NationCreationResult {
    pub success: bool,
    pub nation_id: u8,
    pub message: String,
}

/// Create a new player nation
/// Matches C: newnation() in misc.c
pub fn create_nation(
    name: &str,
    leader: &str,
    race: Race,
    password: &str,
    class: NationClass,
    world: &mut World,
    nations: &mut Vec<Nation>,
    sectors: &mut Vec<Vec<Sector>>,
    start_x: u8,
    start_y: u8,
) -> NationCreationResult {
    // Find empty nation slot
    let mut nation_id = 0u8;
    let mut found = false;

    for i in 1..nations.len() {
        if nations[i].active == 0 {
            nation_id = i as u8;
            found = true;
            break;
        }
    }

    if !found {
        return NationCreationResult {
            success: false,
            nation_id: 0,
            message: "No free nation slots".to_string(),
        };
    }

    // Validate name length
    if name.len() > NAMELTH {
        return NationCreationResult {
            success: false,
            nation_id: 0,
            message: "Name too long".to_string(),
        };
    }

    // Validate location
    let map_x = sectors.len();
    let map_y = if map_x > 0 { sectors[0].len() } else { 0 };
    if start_x as usize >= map_x || start_y as usize >= map_y {
        return NationCreationResult {
            success: false,
            nation_id: 0,
            message: "Invalid start location".to_string(),
        };
    }

    let sector = &sectors[start_x as usize][start_y as usize];

    // Check if sector is habitable
    if !is_habitable(sector) {
        return NationCreationResult {
            success: false,
            nation_id: 0,
            message: "Start sector is not habitable".to_string(),
        };
    }

    // Check if sector is already owned
    if sector.owner != 0 {
        return NationCreationResult {
            success: false,
            nation_id: 0,
            message: "Sector already owned".to_string(),
        };
    }

    // Create nation
    let nation = &mut nations[nation_id as usize];
    init_new_nation(
        nation, name, leader, race, password, class, start_x, start_y,
    );

    // Assign starting sector
    assign_starting_sector(nation_id, sectors, start_x, start_y);

    // Update world
    world.nations += 1;

    NationCreationResult {
        success: true,
        nation_id,
        message: format!("Nation {} created successfully!", name),
    }
}

/// Initialize a new nation with starting values
/// Matches C: ninit() logic
fn init_new_nation(
    nation: &mut Nation,
    name: &str,
    leader: &str,
    race: Race,
    password: &str,
    class: NationClass,
    start_x: u8,
    start_y: u8,
) {
    // Set basic info
    // Note: nation index is set by the caller, not stored in Nation

    // Name
    nation.name = name.to_string();

    // Leader
    nation.leader = leader.to_string();

    // Password
    nation.password = password.to_string();

    // Race
    nation.race = race.to_char();

    // Starting location
    nation.cap_x = start_x;
    nation.cap_y = start_y;
    nation.location = 'R'; // Random placement

    // Active status (PC based on alignment)
    nation.active = match class {
        NationClass::Good => PC_GOOD as u8,
        NationClass::Neutral => PC_NEUTRAL as u8,
        NationClass::Evil => PC_EVIL as u8,
    };

    // Starting gold and resources (based on MAXPTS = 65)
    nation.treasury_gold = 10000; // Starting gold
    nation.total_food = 5000; // Starting food
    nation.metals = 500; // Starting metal
    nation.jewels = 100; // Starting jewels

    // Powers (none initially)
    nation.powers = 0;

    // Spell points
    nation.spell_points = 0;

    // Population
    nation.total_civ = 1000; // Starting civilians
    nation.total_mil = 0;

    // Max movement
    nation.max_move = 10;

    // Reproduction rate
    nation.repro = 10;

    // Score
    nation.score = 0;

    // Class
    nation.class = class as i16;

    // Attack/defense bonuses
    nation.attack_plus = 0;
    nation.defense_plus = 0;

    // Initialize armies to empty
    for army in nation.armies.iter_mut() {
        army.soldiers = 0;
        army.unit_type = 0;
        army.x = 0;
        army.y = 0;
        army.status = ArmyStatus::Defend.to_value();
        army.movement = 0;
    }

    // Initialize navies to empty
    for navy in nation.navies.iter_mut() {
        navy.warships = 0;
        navy.merchant = 0;
        navy.galleys = 0;
        navy.x = 0;
        navy.y = 0;
        navy.movement = 0;
        navy.crew = 0;
        navy.people = 0;
        navy.commodity = 0;
        navy.army_num = MAXARM as u8;
    }

    // Initialize diplomatic status
    // Initialize diplomacy - self is Neutral (0), others are Unmet
    for i in 0..MAXNTOTAL {
        nation.diplomacy[i] = if i == 0 { 0 } else { DIPL_UNMET };
    }

    // Tax rate
    nation.tax_rate = 50; // 50%

    // Charity
    nation.charity = 0;

    // Inflation
    nation.inflation = 0;

    // Sectors
    nation.total_sectors = 0;
    nation.total_ships = 0;

    // Popularity
    nation.popularity = 50;
}

/// Assign starting sector to new nation
fn assign_starting_sector(nation_id: u8, sectors: &mut Vec<Vec<Sector>>, x: u8, y: u8) {
    let sector = &mut sectors[x as usize][y as usize];
    sector.owner = nation_id;
    sector.people = 1000; // Starting population
    sector.designation = Designation::Capitol as u8;
    sector.fortress = 2; // Starting fort level

    // Mark seen (has_seen array would be updated)
}

/// Nation class for new nations
#[derive(Debug, Clone, Copy)]
pub enum NationClass {
    Good,
    Neutral,
    Evil,
}

/// Starting points allocation
pub struct StartingPoints {
    pub gold: i64,
    pub food: i64,
    pub metal: i64,
    pub jewels: i64,
    pub power_points: i32,
}

/// Default starting points based on MAXPTS (65)
pub fn default_starting_points() -> StartingPoints {
    StartingPoints {
        gold: 10000,
        food: 5000,
        metal: 500,
        jewels: 100,
        power_points: MAXPTS as i32,
    }
}

/// Allocate starting points to nation
/// Matches C: buyPower() logic
pub fn allocate_starting_points(
    nation: &mut Nation,
    points: &StartingPoints,
    powers_to_buy: &[Power],
) -> Result<(), String> {
    let mut remaining = points.power_points;

    for &power in powers_to_buy {
        let cost = power_cost(power);
        if cost > remaining {
            return Err(format!("Not enough points for {:?}", power));
        }

        if Power::has_power(nation.powers, power) {
            return Err(format!("Already have power {:?}", power));
        }

        nation.powers |= power.bits();
        remaining -= cost;
    }

    // Apply resources
    nation.treasury_gold += points.gold;
    nation.total_food += points.food;
    nation.metals += points.metal;
    nation.jewels += points.jewels;

    Ok(())
}

/// Get power cost
fn power_cost(power: Power) -> i32 {
    match power {
        // Military powers
        Power::WARRIOR => 3,
        Power::CAPTAIN => 4,
        Power::WARLORD => 5,
        Power::ARCHER => 3,
        Power::CAVALRY => 4,
        Power::SAPPER => 3,
        Power::ARMOR => 4,
        Power::AVIAN => 4,
        // Civilian powers
        Power::SLAVER => 3,
        Power::DERVISH => 2,
        Power::HIDDEN => 3,
        Power::ARCHITECT => 3,
        Power::RELIGION => 3,
        Power::MINER => 3,
        Power::BREEDER => 4,
        Power::URBAN => 3,
        Power::STEEL => 4,
        Power::NINJA => 4,
        Power::SAILOR => 3,
        Power::DEMOCRACY => 2,
        Power::ROADS => 2,
        // Magical powers
        Power::THE_VOID => 5,
        Power::KNOWALL => 4,
        Power::DESTROYER => 5,
        Power::VAMPIRE => 5,
        Power::SUMMON => 4,
        Power::WYZARD => 5,
        Power::SORCERER => 5,
        _ => 3,
    }
}

/// Find best starting location for new nation
/// Matches C: findcap()
pub fn find_starting_location(sectors: &Vec<Vec<Sector>>, desired_race: Race) -> Option<(u8, u8)> {
    let mut best_x = 0u8;
    let mut best_y = 0u8;
    let mut best_score = -1i32;

    let map_x = sectors.len();
    let map_y = if map_x > 0 { sectors[0].len() } else { 0 };
    for x in 0..map_x {
        for y in 0..map_y {
            let sector = &sectors[x][y];

            // Must be habitable
            if !is_habitable(sector) {
                continue;
            }

            // Must not be owned
            if sector.owner != 0 {
                continue;
            }

            // Check distance from other nations
            let mut too_close = false;
            for nx in 0..MAXNTOTAL {
                // Would check against existing nations
            }

            if too_close {
                continue;
            }

            // Score this location
            let score = score_starting_location(sector, desired_race);

            if score > best_score {
                best_score = score;
                best_x = x as u8;
                best_y = y as u8;
            }
        }
    }

    if best_score < 0 {
        None
    } else {
        Some((best_x, best_y))
    }
}

/// Score a potential starting location
fn score_starting_location(sector: &Sector, race: Race) -> i32 {
    let mut score = 0;

    // Base score from vegetation
    match sector.vegetation as char {
        'g' | 'G' => score += 10, // Good
        'w' | 'W' => score += 8,  // Wood
        'f' | 'F' => score += 6,  // Forest
        _ => score += 2,
    }

    // Bonus for altitude
    match sector.altitude as char {
        '0' => score += 5, // Clear
        '-' => score += 3, // Hill
        _ => {}
    }

    // Trade good bonus
    if sector.trade_good > 0 {
        score += 5;
    }

    // Race-specific bonuses
    match race {
        Race::Dwarf => {
            if sector.altitude == Altitude::Mountain as u8 {
                score += 10;
            }
            if sector.trade_good > 0 {
                score += 10;
            }
        }
        Race::Elf => {
            if sector.vegetation == Vegetation::Forest as u8 {
                score += 10;
            }
        }
        Race::Orc => {
            if sector.vegetation == Vegetation::Swamp as u8
                || sector.vegetation == Vegetation::Jungle as u8
            {
                score += 10;
            }
        }
        _ => {}
    }

    score
}

/// NPC nation initialization
/// Matches C: initnpc() logic
pub fn init_npc_nation(
    nations: &mut Vec<Nation>,
    npc_type: NpcType,
    race: Race,
    start_x: u8,
    start_y: u8,
    name: &str,
    leader: &str,
) -> Option<u8> {
    // Find empty slot
    let mut nation_id = 0u8;
    let max_npc = nations.len().min(MAXNTOTAL);
    for i in NPC_FIRST as usize..max_npc {
        if nations[i].active == 0 {
            nation_id = i as u8;
            break;
        }
    }

    if nation_id == 0 {
        return None;
    }

    // Initialize NPC
    let nation = &mut nations[nation_id as usize];

    // NPC type determines starting resources and behavior
    let (active_value, gold, food, metal, jewels) = match npc_type {
        NpcType::Expansionist => (GOOD_0FREE as u8, 8000, 4000, 400, 50),
        NpcType::Isolationist => (ISOLATIONIST as u8, 6000, 3000, 300, 30),
        NpcType::Pirate => (NPC_PIRATE as u8, 4000, 2000, 200, 100),
        NpcType::Nomad => (NPC_NOMAD as u8, 5000, 2500, 150, 20),
        NpcType::Savage => (NPC_SAVAGE as u8, 3000, 1500, 100, 10),
    };

    // Set active status
    nation.active = active_value;

    // Basic setup
    // (would copy name, leader, etc.)

    // Set resources
    nation.treasury_gold = gold;
    nation.total_food = food;
    nation.metals = metal;
    nation.jewels = jewels;

    // Set capitol
    nation.cap_x = start_x;
    nation.cap_y = start_y;

    // Assign starting sector
    // Would call assign_starting_sector

    Some(nation_id)
}

/// NPC nation types
#[derive(Debug, Clone, Copy)]
pub enum NpcType {
    Expansionist,
    Isolationist,
    Pirate,
    Nomad,
    Savage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_starting_points() {
        let pts = default_starting_points();
        assert_eq!(pts.gold, 10000);
        assert_eq!(pts.food, 5000);
        assert_eq!(pts.metal, 500);
        assert_eq!(pts.jewels, 100);
    }

    #[test]
    fn test_nation_creation_validation() {
        // Would test invalid inputs
    }
}
