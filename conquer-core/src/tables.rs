/// Data tables ported from data.c — all values match the C arrays exactly.

// ============================================================
// Altitude (T079)
// ============================================================

/// Altitude display characters: "~#^%-0"
pub const ELE_CHARS: &str = "~#^%-0";

/// Altitude names
pub const ELE_NAMES: [&str; 6] = ["WATER", "PEAK", "MOUNTAIN", "HILL", "FLAT", "ERROR"];

// ============================================================
// Vegetation (T080)
// ============================================================

/// Vegetation display characters: "vdtblgwfjsi~0"
pub const VEG_CHARS: &str = "vdtblgwfjsi~0";

/// Food value per vegetation type (index into string "0004697400000")
pub const VEG_FOOD: [i32; 13] = [0, 0, 0, 4, 6, 9, 7, 4, 0, 0, 0, 0, 0];

/// Vegetation names
pub const VEG_NAMES: [&str; 12] = [
    "VOLCANO", "DESERT", "TUNDRA", "BARREN", "LT VEG", "GOOD", "WOOD", "FOREST", "JUNGLE", "SWAMP",
    "ICE", "NONE",
];

// Movement costs by race × vegetation (index chars in vegcost strings)
// Format: char-'0' gives movement cost; '/' = impassable

/// Human vegetation movement costs: "63210001332//"
pub const H_VEG_COST: &str = "63210001332//";
/// Orc vegetation movement costs: "43100022527//"
pub const O_VEG_COST: &str = "43100022527//";
/// Elf vegetation movement costs: "86221000027//"
pub const E_VEG_COST: &str = "86221000027//";
/// Dwarf vegetation movement costs: "47100013577//"
pub const D_VEG_COST: &str = "47100013577//";
/// Flight vegetation movement costs: "410000001000/"
pub const F_VEG_COST: &str = "410000001000/";

// Movement costs by race × altitude

/// Human altitude movement costs: "//521/"
pub const H_ELE_COST: &str = "//521/";
/// Orc altitude movement costs: "//222/"
pub const O_ELE_COST: &str = "//222/";
/// Elf altitude movement costs: "//631/"
pub const E_ELE_COST: &str = "//631/";
/// Dwarf altitude movement costs: "//311/"
pub const D_ELE_COST: &str = "//311/";
/// Flight altitude movement costs: "16211/"
pub const F_ELE_COST: &str = "16211/";

// ============================================================
// Designation (T081)
// ============================================================

/// Designation display characters: "tcmfx$!&sC?lb+*g=u-P0"
pub const DES_CHARS: &str = "tcmfx$!&sC?lb+*g=u-P0";

/// Designation names
pub const DES_NAMES: [&str; 21] = [
    "TOWN",
    "CITY",
    "MINE",
    "FARM",
    "DEVASTATED",
    "GOLDMINE",
    "FORT",
    "RUIN",
    "STOCKADE",
    "CAPITOL",
    "SPECIAL",
    "LUMBERYD",
    "BLKSMITH",
    "ROAD",
    "MILL",
    "GRANARY",
    "CHURCH",
    "UNIVERSITY",
    "NODESIG",
    "BASE CAMP",
    "ERROR",
];

// ============================================================
// Unit Types (T082-T084)
// ============================================================

/// Full unit type names (60 entries: 27 base + 18 leaders + 15 monsters)
pub const UNIT_TYPE_NAMES: [&str; 60] = [
    // Base units (0-26)
    "Militia",
    "Goblins",
    "Orcs",
    "Infantry",
    "Sailors",
    "Marines",
    "Archers",
    "Uruk-Hai",
    "Ninjas",
    "Phalanx",
    "Olog-Hai",
    "Legionaries",
    "Dragoons",
    "Mercenaries",
    "Trolls",
    "Elite",
    "Lt_Cavalry",
    "Hv_Cavalry",
    "Catapults",
    "Siege",
    "Rocs",
    "Knights",
    "Gryfins",
    "Elephants",
    "Zombies",
    "Spy",
    "Scout",
    // Leaders (27-44)
    "King",
    "Baron",
    "Emperor",
    "Prince",
    "Wizard",
    "Mage",
    "Pope",
    "Bishop",
    "Admiral",
    "Captain",
    "Warlord",
    "Lord",
    "Demon",
    "Devil",
    "Dragyn",
    "Wyrm",
    "Shadow",
    "Nazgul",
    // Monsters (45-59)
    "Spirit",
    "Assasin",
    "Efreet",
    "Gargoyl",
    "Wraith",
    "Hero",
    "Centaur",
    "Giant",
    "Suphero",
    "Mummy",
    "Elmentl",
    "Mintaur",
    "Daemon",
    "Balrog",
    "Dragon",
];

/// Short unit type names (for display)
pub const SHORT_UNIT_TYPE_NAMES: [&str; 60] = [
    "mlta", "Gob", "Orc", "Inf", "Sail", "XMar", "Arch", "Uruk", "Ninj", "Phax", "olog", "Legn",
    "Drag", "Merc", "Trol", "Elt", "lCav", "hCav", "cat", "sge", "Roc", "Kni", "grif", "ele",
    "zom", "Spy", "Scout", "King", "Bar", "Emp", "Prin", "Wizd", "Magi", "Apos", "Bish", "Admi",
    "Capt", "Warl", "Lord", "Demn", "Devl", "Drag", "Wyrm", "Shad", "Nazg", "spir", "Assn", "efr",
    "Garg", "Wra", "Hero", "Cent", "gt", "Shro", "Mumm", "Elem", "mino", "daem", "Bal", "Drgn",
];

/// Minimum strength of each unit type
pub const UNIT_MIN_STRENGTH: [i32; 60] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 100, 50, 100,
    50, 250, 50, 100, 50, 100, 50, 250, 125, 250, 50, 500, 100, 250, 125, 50, 50, 50, 75, 75, 75,
    50, 150, 150, 150, 175, 150, 500, 500, 1000,
];

/// Unit attack bonuses
pub const UNIT_ATTACK: [i32; 60] = [
    -40, -15, 0, 0, 0, 5, 0, 5, 20, 10, 15, 20, 10, 0, 25, 20, 20, 30, -20, -20, 20, 40, 40, 50,
    -15, -30, -30, 30, 20, 30, 20, 30, 20, 30, 20, 30, 20, 30, 30, 50, 20, 50, 40, 50, 40, 0, 20,
    10, 10, 10, 0, 10, 0, 15, 15, 5, 20, 50, 40, 50,
];

/// Unit defense bonuses
pub const UNIT_DEFEND: [i32; 60] = [
    -25, -15, 0, 0, 0, 0, 10, 5, 0, 10, 15, 20, 10, 0, 15, 20, 20, 30, 20, 20, 30, 40, 50, 50, -15,
    -30, -30, 30, 20, 30, 20, 30, 20, 30, 20, 30, 20, 30, 30, 50, 20, 50, 40, 50, 40, 0, 20, 10,
    10, 10, 0, 10, 0, 15, 15, 5, 20, 50, 40, 50,
];

/// Unit movement rates (×10)
pub const UNIT_MOVE: [i32; 60] = [
    0, 10, 10, 10, 0, 0, 10, 10, 10, 10, 10, 10, 20, 10, 10, 13, 20, 20, 5, 5, 10, 20, 15, 5, 10,
    10, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 10, 10, 15, 10,
    10, 10, 15, 10, 10, 10, 15, 10, 10, 15, 20,
];

/// Metal cost for enlisting a unit
pub const UNIT_ENLIST_METAL: [i32; 60] = [
    0, 80, 80, 100, 100, 100, 100, 150, 150, 150, 150, 150, 100, 0, 200, 200, 100, 300, 1000, 1000,
    300, 600, 400, 600, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Gold cost for enlisting a unit (spell points for monsters)
pub const UNIT_ENLIST_COST: [i32; 60] = [
    50, 70, 85, 100, 100, 100, 100, 125, 125, 150, 180, 180, 300, 225, 225, 225, 300, 450, 600,
    600, 600, 600, 800, 600, 100, 10000, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    2, 2, 2, 2, 2, 2, 2, 5, 5, 5, 5, 5, 10, 10, 15,
];

/// Maintenance cost per unit (gold for base/leaders, jewels/turn for monsters)
pub const UNIT_MAINTENANCE: [i32; 60] = [
    20, 20, 50, 50, 50, 50, 50, 50, 50, 50, 75, 75, 200, 100, 100, 100, 175, 225, 250, 250, 250,
    250, 250, 250, 0, 2000, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1000, 400,
    400, 450, 450, 200, 200, 2100, 450, 1000, 1900, 2100, 6000, 6000, 10000,
];

// ============================================================
// Class, Race, Diplomacy Names (T085)
// ============================================================

/// Nation class names
pub const CLASS_NAMES: [&str; 11] = [
    "monster", "king", "emperor", "wizard", "priest", "pirate", "trader", "warlord", "demon",
    "dragon", "shadow",
];

/// Race names
pub const RACE_NAMES: [&str; 10] = [
    "GOD", "ORC", "ELF", "DWARF", "LIZARD", "HUMAN", "PIRATE", "SAVAGE", "NOMAD", "UNKNOWN",
];

/// Diplomacy status names
pub const DIPLO_NAMES: [&str; 8] = [
    "UNMET", "TREATY", "ALLIED", "FRIENDLY", "NEUTRAL", "HOSTILE", "WAR", "JIHAD",
];

// ============================================================
// Trade Goods (T086)
// ============================================================

/// Trade good sector-type compatibility string from C:
/// "xffffttttffffffffffftxlxllttuuctcccfff?xtccmmmmmmmm$$$$$$$$$$0"
/// x=any, f=farm-like, t=town, l=lumber, u=university, c=church, m=mine, $=goldmine, ?=special
pub const TG_SECTOR_TYPE: &str = "xffffttttffffffffffftxlxllttuuctcccfff?xtccmmmmmmmm$$$$$$$$$$0";

/// Trade good values (as string from C):
/// "13335157911433442331131135734567789123937571111111111111111110"
pub const TG_VALUE_STR: &str = "13335157911433442331131135734567789123937571111111111111111110";

/// Parse a trade good value by index
pub fn tg_value(index: usize) -> u8 {
    TG_VALUE_STR
        .as_bytes()
        .get(index)
        .map(|b| b - b'0')
        .unwrap_or(0)
}

/// Trade good names (62 entries including "none")
pub const TG_NAMES: [&str; 62] = [
    "furs",
    "wool",
    "beer",
    "cloth",
    "wine",
    "mules",
    "horses",
    "pigeons",
    "griffons",
    "corn",
    "fish",
    "sugar",
    "honey",
    "fruit",
    "rice",
    "wheat",
    "dairy",
    "peas",
    "bread",
    "cereal",
    "pottery",
    "salt",
    "timber",
    "granite",
    "pine",
    "oak",
    "nails",
    "papyrus",
    "math",
    "library",
    "drama",
    "paper",
    "literature",
    "law",
    "philosophy",
    "irrigation",
    "oxen",
    "plows",
    "stones",
    "herbs",
    "medicine",
    "torture",
    "prison",
    "bronze",
    "copper",
    "lead",
    "tin",
    "iron",
    "steel",
    "mithral",
    "adamantine",
    "spice",
    "silver",
    "pearls",
    "dye",
    "silk",
    "gold",
    "rubys",
    "ivory",
    "diamonds",
    "platinum",
    "none",
];

// ============================================================
// Season/Direction/Alignment Strings (T088)
// ============================================================

/// Season names
pub const SEASON_NAMES: [&str; 4] = ["Winter", "Spring", "Summer", "Fall"];

/// Direction names
pub const DIRECTION_NAMES: [&str; 9] = [
    "here",
    "north",
    "northeast",
    "east",
    "southeast",
    "south",
    "southwest",
    "west",
    "northwest",
];

/// Alignment names
pub const ALIGNMENT_NAMES: [&str; 5] = ["Other", "Good", "Neutral", "Evil", "Other"];

/// Army status display names (index 0 is "?" for invalid, 1-16 match MARCH..RULE)
pub const SOLD_NAMES: [&str; 17] = [
    "?", "MARCH", "SCOUT", "GARRISON", "TRADED", "MILITIA", "FLYING", "DEFEND", "MAG_DEF",
    "ATTACK", "MAG_ATT", "GENERAL", "SORTIE", "SIEGE", "BESIEGED", "ON_BOARD", "RULE",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_arrays_lengths() {
        assert_eq!(UNIT_TYPE_NAMES.len(), 60);
        assert_eq!(SHORT_UNIT_TYPE_NAMES.len(), 60);
        assert_eq!(UNIT_MIN_STRENGTH.len(), 60);
        assert_eq!(UNIT_ATTACK.len(), 60);
        assert_eq!(UNIT_DEFEND.len(), 60);
        assert_eq!(UNIT_MOVE.len(), 60);
        assert_eq!(UNIT_ENLIST_METAL.len(), 60);
        assert_eq!(UNIT_ENLIST_COST.len(), 60);
        assert_eq!(UNIT_MAINTENANCE.len(), 60);
    }

    #[test]
    fn test_specific_unit_values() {
        // Militia
        assert_eq!(UNIT_ATTACK[0], -40);
        assert_eq!(UNIT_DEFEND[0], -25);
        assert_eq!(UNIT_MOVE[0], 0);
        // Infantry
        assert_eq!(UNIT_ATTACK[3], 0);
        assert_eq!(UNIT_DEFEND[3], 0);
        assert_eq!(UNIT_MOVE[3], 10);
        // Knight (index 21)
        assert_eq!(UNIT_ATTACK[21], 40);
        assert_eq!(UNIT_DEFEND[21], 40);
        assert_eq!(UNIT_MOVE[21], 20);
        // Dragon (monster, index 59)
        assert_eq!(UNIT_ATTACK[59], 50);
        assert_eq!(UNIT_DEFEND[59], 50);
        assert_eq!(UNIT_MOVE[59], 20);
        assert_eq!(UNIT_MIN_STRENGTH[59], 1000);
        assert_eq!(UNIT_MAINTENANCE[59], 10000);
    }

    #[test]
    fn test_veg_food_values() {
        // "0004697400000"
        assert_eq!(VEG_FOOD[0], 0); // Volcano
        assert_eq!(VEG_FOOD[3], 4); // Barren
        assert_eq!(VEG_FOOD[4], 6); // LtVeg
        assert_eq!(VEG_FOOD[5], 9); // Good
        assert_eq!(VEG_FOOD[6], 7); // Wood
        assert_eq!(VEG_FOOD[7], 4); // Forest
    }

    #[test]
    fn test_tg_values() {
        // First few: "13335..."
        assert_eq!(tg_value(0), 1); // furs
        assert_eq!(tg_value(1), 3); // wool
        assert_eq!(tg_value(2), 3); // beer
        assert_eq!(tg_value(3), 3); // cloth
        assert_eq!(tg_value(4), 5); // wine
    }

    #[test]
    fn test_des_chars() {
        // "tcmfx$!&sC?lb+*g=u-P0"
        assert_eq!(DES_CHARS.as_bytes()[0], b't'); // Town
        assert_eq!(DES_CHARS.as_bytes()[1], b'c'); // City
        assert_eq!(DES_CHARS.as_bytes()[9], b'C'); // Capitol
    }

    #[test]
    fn test_tg_names_count() {
        assert_eq!(TG_NAMES.len(), 62);
        assert_eq!(TG_NAMES[0], "furs");
        assert_eq!(TG_NAMES[61], "none");
    }

    #[test]
    fn test_class_names() {
        assert_eq!(CLASS_NAMES[0], "monster");
        assert_eq!(CLASS_NAMES[1], "king");
        assert_eq!(CLASS_NAMES[10], "shadow");
    }
}
