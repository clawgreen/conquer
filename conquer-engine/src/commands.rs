// conquer-engine/src/commands.rs — Player commands ported from commands.c
//
// T251-T260: Player commands (form, attack, designate, construct, draft, etc.)
//
// Core command handlers that process player input
use conquer_core::*;
use conquer_core::tables::*;
use crate::utils::*;

/// Command result
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub gold_cost: i64,
    pub metal_cost: i64,
}

/// Validate if a designation is allowed
/// Matches C: desg_ok() function
pub fn validate_designation(
    nation: &Nation,
    nation_idx: usize,
    sector: &Sector,
    new_designation: u8,
    print_error: bool,
) -> CommandResult {
    let country = nation_idx as u8;
    
    // Check vegetation requirement
    if new_designation != Designation::NoDesig as u8
        && new_designation != Designation::Road as u8
        && new_designation != Designation::Fort as u8
        && new_designation != Designation::Stockade as u8
    {
        if tofood(sector, Some(nation)) < DESFOOD {
            return CommandResult {
                success: false,
                message: "vegetation too sparse".to_string(),
                gold_cost: 0,
                metal_cost: 0,
            };
        }
    }
    
    // Don't allow same designation
    if new_designation == sector.designation {
        return CommandResult {
            success: false,
            message: "Hey, get your act together! There is already one there.".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    // Check for city/capitol being made into something else
    if new_designation != Designation::Ruin as u8 {
        if (new_designation != Designation::Capitol as u8 && sector.designation == Designation::City as u8)
            || sector.designation == Designation::Capitol as u8
        {
            if new_designation != Designation::Ruin as u8 {
                return CommandResult {
                    success: false,
                    message: format!(
                        "Must first burn down city/capitol (designate as '{}')",
                        Designation::Ruin.to_char()
                    ),
                    gold_cost: 0,
                    metal_cost: 0,
                };
            }
        }
    }
    
    // Check population requirement for city/town/capitol
    if sector.people < 500 {
        if new_designation == Designation::Capitol as u8
            || new_designation == Designation::City as u8
            || new_designation == Designation::Town as u8
        {
            return CommandResult {
                success: false,
                message: "Need 500 people to build a city or town".to_string(),
                gold_cost: 0,
                metal_cost: 0,
            };
        }
    }
    
    // Only god may create pirate base
    if new_designation == Designation::BaseCamp as u8 {
        return CommandResult {
            success: false,
            message: "A Pirate Cove?? Are you serious?!".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    // Check for ruin
    if new_designation == Designation::Ruin as u8 {
        if sector.designation != Designation::City as u8
            && sector.designation != Designation::Capitol as u8
        {
            return CommandResult {
                success: false,
                message: "Ruins may only come from cities or capitols".to_string(),
                gold_cost: 0,
                metal_cost: 0,
            };
        }
    }
    
    // Check for special designation (requires magic)
    if new_designation == Designation::Special as u8 {
        if !Power::has_power(nation.powers, Power::SUMMON) {
            return CommandResult {
                success: false,
                message: "You are gonna need SUMMON power to use those stones!".to_string(),
                gold_cost: 0,
                metal_cost: 0,
            };
        }
    }
    
    CommandResult {
        success: true,
        message: "OK".to_string(),
        gold_cost: 0,
        metal_cost: 0,
    }
}

/// Calculate cost for redesignation
/// Matches C: SADJDES and related calculations
pub fn redesignation_cost(
    current_designation: u8,
    new_designation: u8,
) -> (i64, i64) {
    let mut gold_cost = DESCOST;
    let mut metal_cost = 0i64;
    
    // Towns and forts have different costs
    if new_designation == Designation::Town as u8 || new_designation == Designation::Fort as u8 {
        metal_cost = DESCOST;
    }
    // City or Capitol
    else if new_designation == Designation::City as u8 
        || new_designation == Designation::Capitol as u8 
    {
        metal_cost = 5 * DESCOST;
    }
    // From ruin costs extra
    if current_designation == Designation::Ruin as u8 {
        gold_cost += 10 * DESCOST;
        metal_cost += metal_cost / 2;
    } else {
        gold_cost += 20 * DESCOST;
    }
    
    (gold_cost, metal_cost)
}

/// Draft soldiers in a city/town/capitol
/// Matches C: draft() function
pub fn draft_unit(
    nation: &mut Nation,
    sector_x: u8,
    sector_y: u8,
    sector: &Sector,
    unit_type: u8,
    num_soldiers: i64,
) -> CommandResult {
    // Must be in valid location
    let des = sector.designation;
    if des != Designation::Town as u8
        && des != Designation::City as u8
        && des != Designation::Capitol as u8
    {
        return CommandResult {
            success: false,
            message: "must raise in towns/cities/capitols".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    // Check gold
    let cost = enlist_cost(unit_type) * num_soldiers;
    if nation.treasury_gold < cost {
        return CommandResult {
            success: false,
            message: "Not enough gold".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    // Find empty army slot
    let mut army_idx = -1;
    for i in 0..MAXARM {
        if nation.armies[i].soldiers <= 0 {
            army_idx = i as i32;
            break;
        }
    }
    
    if army_idx < 0 {
        return CommandResult {
            success: false,
            message: "No free army slots".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    // Create army
    let army = &mut nation.armies[army_idx as usize];
    army.soldiers = num_soldiers;
    army.unit_type = unit_type;
    army.x = sector_x;
    army.y = sector_y;
    army.status = ArmyStatus::Defend.to_value();
    army.movement = UNIT_MOVE.get(unit_type as usize).copied().unwrap_or(0) as u8;
    
    // Deduct gold
    nation.treasury_gold -= cost;
    
    CommandResult {
        success: true,
        message: format!("Drafted {} soldiers", num_soldiers),
        gold_cost: cost,
        metal_cost: 0,
    }
}

/// Calculate enlist cost for a unit type
/// Matches C: u_encost table
pub fn enlist_cost(unit_type: u8) -> i64 {
    let idx = unit_type as usize;
    if idx >= UNIT_ENLIST_COST.len() {
        return 100; // Default
    }
    UNIT_ENLIST_COST[idx] as i64
}

/// Construct a fort in a sector
/// Matches C: construct() with fort option
pub fn construct_fort(
    nation: &mut Nation,
    sector: &mut Sector,
) -> CommandResult {
    // Must be in town, city, fort, or capitol
    let des = sector.designation;
    if des != Designation::Town as u8
        && des != Designation::City as u8
        && des != Designation::Fort as u8
        && des != Designation::Capitol as u8
    {
        return CommandResult {
            success: false,
            message: "Must construct in town, city, or fortress".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    // Check max fortress level
    if sector.fortress >= 12 {
        return CommandResult {
            success: false,
            message: "That sector is as impregnable as you can make it".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    // Calculate cost (doubles with each level)
    let mut cost = FORTCOST;
    for _ in 0..sector.fortress {
        cost *= 2;
    }
    
    // Check if can afford (debt limit based on jewels)
    let max_debt = nation.jewels * 10;
    if nation.treasury_gold - cost < -max_debt {
        return CommandResult {
            success: false,
            message: "you may not spend that much".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    // Build fort
    nation.treasury_gold -= cost;
    sector.fortress = sector.fortress.saturating_add(1);
    
    CommandResult {
        success: true,
        message: format!("Built fort (+{}%)", fort_bonus(sector, nation.powers)),
        gold_cost: cost,
        metal_cost: 0,
    }
}

/// Calculate fortification bonus
/// Matches C: FORTSTR, TOWNSTR, CITYSTR
pub fn fort_bonus(sector: &Sector, powers: i64) -> i32 {
    let base = match sector.designation {
        d if d == Designation::Town as u8 => 5,
        d if d == Designation::Fort as u8 => 5,
        d if d == Designation::City as u8 => 8,
        d if d == Designation::Capitol as u8 => 8,
        _ => 0,
    };
    
    let mut bonus = base * sector.fortress as i32;
    
    // ARCHITECT power doubles
    if Power::has_power(powers, Power::ARCHITECT) {
        bonus *= 2;
    }
    
    bonus
}

/// Check if sector is next to water (for ship building)
pub fn is_next_to_water(sectors: &Vec<Vec<Sector>>, x: u8, y: u8) -> bool {
    let x = x as i32;
    let y = y as i32;
    let map_x = sectors.len() as i32;
    let map_y = if map_x > 0 { sectors[0].len() as i32 } else { 0 };
    
    for dx in -1..=1 {
        for dy in -1..=1 {
            let nx = x + dx;
            let ny = y + dy;
            
            if nx < 0 || ny < 0 || nx >= map_x || ny >= map_y {
                continue;
            }
            
            if sectors[nx as usize][ny as usize].altitude == Altitude::Water as u8 {
                return true;
            }
        }
    }
    
    false
}

/// Build/repair ships
/// Matches C: construct() with ship option
pub fn construct_ship(
    nation: &mut Nation,
    sector: &mut Sector,
    navy_idx: i32,
    ship_type: ShipType,
    ship_class: ShipClass,
    num_ships: i32,
) -> CommandResult {
    // Check if next to water
    // Would need sectors access
    
    // Check population
    let crew_needed = num_ships * (ship_class as i32 + 1) * SHIPCREW as i32;
    if sector.people < crew_needed as i64 {
        return CommandResult {
            success: false,
            message: "NOT ENOUGH CIVILIANS IN SECTOR".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    // Calculate cost
    let base_cost = match ship_type {
        ShipType::Warship => WARSHPCOST,
        ShipType::Merchant => MERSHPCOST,
        ShipType::Galley => GALSHPCOST,
    };
    let size_mod = (ship_class as i32 + 1);
    let cost = num_ships as i64 * size_mod as i64 * base_cost;
    
    // SAILOR power halves cost
    let cost = if Power::has_power(nation.powers, Power::SAILOR) {
        cost / 2
    } else {
        cost
    };
    
    // Check gold
    if nation.treasury_gold < cost {
        return CommandResult {
            success: false,
            message: "sorry - not enough talons".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    // Build ships (simplified)
    // Would need navy management
    
    nation.treasury_gold -= cost;
    sector.people = sector.people.saturating_sub(crew_needed as i64);
    
    CommandResult {
        success: true,
        message: format!("Built {} ships", num_ships),
        gold_cost: cost,
        metal_cost: 0,
    }
}

/// Ship types
#[derive(Debug, Clone, Copy)]
pub enum ShipType {
    Warship,
    Merchant,
    Galley,
}

/// Ship classes
#[derive(Debug, Clone, Copy)]
pub enum ShipClass {
    Light = 0,
    Medium = 1,
    Heavy = 2,
}

/// Execute a road building command
/// Matches C: redesignate() with DROAD
pub fn build_road(
    nation: &mut Nation,
    nation_idx: usize,
    sector: &Sector,
    x: u8,
    y: u8,
) -> CommandResult {
    // Must own both sectors
    if sector.owner != nation_idx as u8 {
        return CommandResult {
            success: false,
            message: "You don't own that sector".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    // Need 100+ people
    if sector.people < 100 {
        return CommandResult {
            success: false,
            message: "Need 100+ people to build a road!".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    // Cost
    let cost = DESCOST;
    if nation.treasury_gold < cost {
        return CommandResult {
            success: false,
            message: "Not enough gold".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    nation.treasury_gold -= cost;
    
    CommandResult {
        success: true,
        message: format!("Road built to ({}, {})", x, y),
        gold_cost: cost,
        metal_cost: 0,
    }
}

/// Process a complete redesignation command
pub fn execute_designation(
    nation: &mut Nation,
    nation_idx: usize,
    sector_x: u8,
    sector_y: u8,
    sector: &mut Sector,
    new_designation: u8,
    is_god: bool,
) -> CommandResult {
    // Validate
    let validation = validate_designation(nation, nation_idx, sector, new_designation, true);
    if !validation.success {
        return validation;
    }
    
    // Calculate costs
    let (gold_cost, metal_cost) = redesignation_cost(sector.designation, new_designation);
    
    // Check metal
    if nation.metals < metal_cost {
        return CommandResult {
            success: false,
            message: "Not enough metal for city, town, or fort".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    // Check gold (god is free)
    let actual_gold_cost = if is_god { 0 } else { gold_cost };
    if nation.treasury_gold < actual_gold_cost {
        return CommandResult {
            success: false,
            message: "Not enough gold".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    // Execute
    nation.treasury_gold -= actual_gold_cost;
    nation.metals = nation.metals.saturating_sub(metal_cost);
    sector.designation = new_designation;
    
    // Special handling for capitol
    if new_designation == Designation::Capitol as u8 {
        // Demolish old capitol
        let old_cap_x = nation.cap_x;
        let old_cap_y = nation.cap_y;
        // Would need sectors access
        
        // Set new capitol
        nation.cap_x = sector_x;
        nation.cap_y = sector_y;
    }
    
    CommandResult {
        success: true,
        message: format!("Designated as {}", new_designation),
        gold_cost: actual_gold_cost,
        metal_cost,
    }
}

/// Transfer gold between nations (tribute)
pub fn send_tribute(
    from: &mut Nation,
    to_idx: usize,
    amount: i64,
) -> CommandResult {
    if to_idx >= MAXNTOTAL {
        return CommandResult {
            success: false,
            message: "Invalid nation".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    if from.treasury_gold < amount {
        return CommandResult {
            success: false,
            message: "Not enough gold".to_string(),
            gold_cost: 0,
            metal_cost: 0,
        };
    }
    
    from.treasury_gold -= amount;
    // Would add to recipient
    
    CommandResult {
        success: true,
        message: format!("Sent {} gold tribute", amount),
        gold_cost: amount,
        metal_cost: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fort_bonus() {
        let sector = Sector {
            designation: Designation::Fort as u8,
            fortress: 3,
            ..Default::default()
        };
        
        let bonus = fort_bonus(&sector, 0);
        assert_eq!(bonus, 15); // 5 * 3
    }

    #[test]
    fn test_enlist_cost() {
        // Infantry (unit type 3) costs 100
        assert_eq!(enlist_cost(3), 100);
    }
}
