use crate::constants::*;
use crate::enums::*;
use serde::{Deserialize, Serialize};

// ============================================================
// World (T070)
// ============================================================

/// Matches C `struct s_world`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct World {
    pub map_x: i16,
    pub map_y: i16,
    pub nations: i16,
    pub other_nations: i16,
    pub turn: i16,
    pub merc_mil: i64,
    pub merc_aplus: i16,
    pub merc_dplus: i16,
    pub world_jewels: i64,
    pub world_gold: i64,
    pub world_food: i64,
    pub world_metal: i64,
    pub world_civ: i64,
    pub world_mil: i64,
    pub world_sectors: i64,
    pub score: i64,
}

impl Default for World {
    fn default() -> Self {
        World {
            map_x: 0,
            map_y: 0,
            nations: 0,
            other_nations: 0,
            turn: 0,
            merc_mil: 0,
            merc_aplus: 0,
            merc_dplus: 0,
            world_jewels: 0,
            world_gold: 0,
            world_food: 0,
            world_metal: 0,
            world_civ: 0,
            world_mil: 0,
            world_sectors: 0,
            score: 0,
        }
    }
}

// ============================================================
// Sector (T071)
// ============================================================

/// Matches C `struct s_sector`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sector {
    pub designation: u8,
    pub altitude: u8,
    pub vegetation: u8,
    pub owner: u8,
    pub people: i64,
    pub initial_people: i16,
    pub jewels: u8,
    pub fortress: u8,
    pub metal: u8,
    pub trade_good: u8,
}

impl Default for Sector {
    fn default() -> Self {
        Sector {
            designation: 0,
            altitude: 0,
            vegetation: 0,
            owner: 0,
            people: 0,
            initial_people: 0,
            jewels: 0,
            fortress: 0,
            metal: 0,
            trade_good: TradeGood::None as u8,
        }
    }
}

impl Sector {
    pub fn designation_enum(&self) -> Option<Designation> {
        Designation::from_index(self.designation)
    }

    pub fn altitude_enum(&self) -> Option<Altitude> {
        Altitude::from_index(self.altitude)
    }

    pub fn vegetation_enum(&self) -> Option<Vegetation> {
        Vegetation::from_index(self.vegetation)
    }

    pub fn trade_good_enum(&self) -> Option<TradeGood> {
        TradeGood::from_value(self.trade_good)
    }
}

// ============================================================
// Army (T072)
// ============================================================

/// Matches C `struct army`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Army {
    pub unit_type: u8,
    pub x: u8,
    pub y: u8,
    pub movement: u8,
    pub soldiers: i64,
    pub status: u8,
}

impl Default for Army {
    fn default() -> Self {
        Army {
            unit_type: 0,
            x: 0,
            y: 0,
            movement: 0,
            soldiers: 0,
            status: 0,
        }
    }
}

impl Army {
    pub fn unit_type_enum(&self) -> UnitType {
        UnitType(self.unit_type)
    }

    pub fn status_enum(&self) -> ArmyStatus {
        ArmyStatus::from_value(self.status)
    }

    pub fn is_alive(&self) -> bool {
        self.soldiers > 0
    }
}

// ============================================================
// Navy (T073)
// ============================================================

/// Matches C `struct navy`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Navy {
    pub warships: u16,
    pub merchant: u16,
    pub galleys: u16,
    pub x: u8,
    pub y: u8,
    pub movement: u8,
    pub crew: u8,
    pub people: u8,
    pub commodity: u8,
    pub army_num: u8,
}

impl Default for Navy {
    fn default() -> Self {
        Navy {
            warships: 0,
            merchant: 0,
            galleys: 0,
            x: 0,
            y: 0,
            movement: 0,
            crew: 0,
            people: 0,
            commodity: 0,
            army_num: 0,
        }
    }
}

impl Navy {
    /// Get light/medium/heavy warship count using SHIPS() macro
    pub fn war_ships(&self, size: NavalSize) -> i16 {
        NavalSize::ships(self.warships, size)
    }

    /// Get light/medium/heavy merchant count
    pub fn mer_ships(&self, size: NavalSize) -> i16 {
        NavalSize::ships(self.merchant, size)
    }

    /// Get light/medium/heavy galley count
    pub fn gal_ships(&self, size: NavalSize) -> i16 {
        NavalSize::ships(self.galleys, size)
    }

    /// Check if this fleet has any ships
    pub fn has_ships(&self) -> bool {
        self.warships > 0 || self.merchant > 0 || self.galleys > 0
    }
}

// ============================================================
// Nation (T074)
// ============================================================

/// Matches C `struct s_nation`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Nation {
    pub name: String,
    pub password: String,
    pub leader: String,
    pub race: char,
    pub location: char,
    pub mark: char,
    pub cap_x: u8,
    pub cap_y: u8,
    pub active: u8,
    pub max_move: u8,
    pub repro: i8,
    pub score: i64,
    pub treasury_gold: i64,
    pub jewels: i64,
    pub total_mil: i64,
    pub total_civ: i64,
    pub metals: i64,
    pub total_food: i64,
    pub powers: i64,
    pub class: i16,
    pub attack_plus: i16,
    pub defense_plus: i16,
    pub spell_points: i16,
    pub total_sectors: i16,
    pub total_ships: i16,
    pub inflation: i16,
    pub charity: u8,
    pub armies: Vec<Army>,
    pub navies: Vec<Navy>,
    pub diplomacy: Vec<u8>,
    pub tax_rate: u8,
    pub prestige: u8,
    pub popularity: u8,
    pub power: u8,
    pub communications: u8,
    pub wealth: u8,
    pub eat_rate: u8,
    pub spoil_rate: u8,
    pub knowledge: u8,
    pub farm_ability: u8,
    pub mine_ability: u8,
    pub poverty: u8,
    pub terror: u8,
    pub reputation: u8,
}

impl Default for Nation {
    fn default() -> Self {
        Nation {
            name: String::new(),
            password: String::new(),
            leader: String::new(),
            race: '?',
            location: ' ',
            mark: ' ',
            cap_x: 0,
            cap_y: 0,
            active: NationStrategy::Inactive as u8,
            max_move: 0,
            repro: 0,
            score: 0,
            treasury_gold: 0,
            jewels: 0,
            total_mil: 0,
            total_civ: 0,
            metals: 0,
            total_food: 0,
            powers: 0,
            class: 0,
            attack_plus: 0,
            defense_plus: 0,
            spell_points: 0,
            total_sectors: 0,
            total_ships: 0,
            inflation: 0,
            charity: 0,
            armies: vec![Army::default(); MAXARM],
            navies: vec![Navy::default(); MAXNAVY],
            diplomacy: vec![DiplomaticStatus::Unmet as u8; NTOTAL],
            tax_rate: 0,
            prestige: 0,
            popularity: 0,
            power: 0,
            communications: 0,
            wealth: 0,
            eat_rate: 0,
            spoil_rate: 0,
            knowledge: 0,
            farm_ability: 0,
            mine_ability: 0,
            poverty: 0,
            terror: 0,
            reputation: 0,
        }
    }
}

impl Nation {
    pub fn race_enum(&self) -> Race {
        Race::from_char(self.race)
    }

    pub fn strategy(&self) -> Option<NationStrategy> {
        NationStrategy::from_value(self.active)
    }

    pub fn class_enum(&self) -> Option<NationClass> {
        NationClass::from_value(self.class)
    }

    pub fn has_power(&self, power: crate::powers::Power) -> bool {
        crate::powers::Power::has_power(self.powers, power)
    }

    pub fn is_active(&self) -> bool {
        self.active != NationStrategy::Inactive as u8
    }

    /// Number of alive armies
    pub fn alive_armies(&self) -> usize {
        self.armies.iter().filter(|a| a.is_alive()).count()
    }
}

// ============================================================
// Spreadsheet (T075)
// ============================================================

/// Matches C `struct sprd_sht`
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Spreadsheet {
    pub food: i64,
    pub gold: i64,
    pub jewels: i64,
    pub metal: i64,
    pub rev_food: i64,
    pub rev_jewels: i64,
    pub rev_metal: i64,
    pub rev_cap: i64,
    pub rev_city: i64,
    pub rev_other: i64,
    pub in_gold: i64,
    pub in_metal: i64,
    pub in_farm: i64,
    pub in_city: i64,
    pub in_cap: i64,
    pub in_other: i64,
    pub civilians: i64,
    pub sectors: i32,
}

// ============================================================
// GameState (T076)
// ============================================================

/// Single owner of all game state — replaces all C globals
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameState {
    pub world: World,
    pub nations: Vec<Nation>,
    /// Sectors indexed as [x][y]
    pub sectors: Vec<Vec<Sector>>,
    /// Sector occupation map: NTOTAL+1 if contested
    pub occupied: Vec<Vec<i8>>,
    /// Movement cost grid
    pub move_cost: Vec<Vec<i16>>,
}

impl GameState {
    pub fn new(map_x: usize, map_y: usize) -> Self {
        GameState {
            world: World {
                map_x: map_x as i16,
                map_y: map_y as i16,
                ..Default::default()
            },
            nations: (0..NTOTAL).map(|_| Nation::default()).collect(),
            sectors: vec![vec![Sector::default(); map_y]; map_x],
            occupied: vec![vec![0i8; map_y]; map_x],
            move_cost: vec![vec![0i16; map_y]; map_x],
        }
    }

    /// Check if coordinates are on the map: `ONMAP(x,y)`
    pub fn on_map(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.world.map_x as i32 && y < self.world.map_y as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_default() {
        let w = World::default();
        assert_eq!(w.map_x, 0);
        assert_eq!(w.turn, 0);
        assert_eq!(w.score, 0);
    }

    #[test]
    fn test_nation_default() {
        let n = Nation::default();
        assert_eq!(n.armies.len(), MAXARM);
        assert_eq!(n.navies.len(), MAXNAVY);
        assert_eq!(n.diplomacy.len(), NTOTAL);
        assert!(!n.is_active());
    }

    #[test]
    fn test_game_state_creation() {
        let gs = GameState::new(32, 32);
        assert_eq!(gs.sectors.len(), 32);
        assert_eq!(gs.sectors[0].len(), 32);
        assert_eq!(gs.nations.len(), NTOTAL);
        assert!(gs.on_map(0, 0));
        assert!(gs.on_map(31, 31));
        assert!(!gs.on_map(32, 0));
        assert!(!gs.on_map(-1, 0));
    }

    #[test]
    fn test_navy_ship_unpacking() {
        let mut n = Navy::default();
        // Pack: 3 light, 2 medium, 1 heavy warships
        n.warships = NavalSize::set_ships(0, NavalSize::Light, 3);
        n.warships = NavalSize::set_ships(n.warships, NavalSize::Medium, 2);
        n.warships = NavalSize::set_ships(n.warships, NavalSize::Heavy, 1);

        assert_eq!(n.war_ships(NavalSize::Light), 3);
        assert_eq!(n.war_ships(NavalSize::Medium), 2);
        assert_eq!(n.war_ships(NavalSize::Heavy), 1);
    }

    #[test]
    fn test_army_status() {
        let mut a = Army::default();
        a.status = 3; // Garrison
        assert_eq!(a.status_enum(), ArmyStatus::Garrison);
        a.status = 20; // Group 3
        assert_eq!(a.status_enum(), ArmyStatus::Group(3));
    }

    #[test]
    fn test_sector_enums() {
        let mut s = Sector::default();
        s.designation = Designation::Capitol as u8;
        s.altitude = Altitude::Hill as u8;
        s.vegetation = Vegetation::Good as u8;
        s.trade_good = TradeGood::Wine as u8;

        assert_eq!(s.designation_enum(), Some(Designation::Capitol));
        assert_eq!(s.altitude_enum(), Some(Altitude::Hill));
        assert_eq!(s.vegetation_enum(), Some(Vegetation::Good));
        assert_eq!(s.trade_good_enum(), Some(TradeGood::Wine));
    }
}
