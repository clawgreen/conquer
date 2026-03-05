use serde::{Deserialize, Serialize};

// ============================================================
// Race (T052)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Race {
    God,
    Orc,
    Elf,
    Dwarf,
    Lizard,
    Human,
    Pirate,
    Savage,
    Nomad,
    Unknown,
}

impl Race {
    pub fn from_char(c: char) -> Self {
        match c {
            '-' => Race::God,
            'O' => Race::Orc,
            'E' => Race::Elf,
            'D' => Race::Dwarf,
            'L' => Race::Lizard,
            'H' => Race::Human,
            'P' => Race::Pirate,
            'S' => Race::Savage,
            'N' => Race::Nomad,
            _ => Race::Unknown,
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Race::God => '-',
            Race::Orc => 'O',
            Race::Elf => 'E',
            Race::Dwarf => 'D',
            Race::Lizard => 'L',
            Race::Human => 'H',
            Race::Pirate => 'P',
            Race::Savage => 'S',
            Race::Nomad => 'N',
            Race::Unknown => '?',
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Race::God => "GOD",
            Race::Orc => "ORC",
            Race::Elf => "ELF",
            Race::Dwarf => "DWARF",
            Race::Lizard => "LIZARD",
            Race::Human => "HUMAN",
            Race::Pirate => "PIRATE",
            Race::Savage => "SAVAGE",
            Race::Nomad => "NOMAD",
            Race::Unknown => "UNKNOWN",
        }
    }
}

impl From<char> for Race {
    fn from(c: char) -> Self {
        Race::from_char(c)
    }
}

impl From<Race> for char {
    fn from(r: Race) -> char {
        r.to_char()
    }
}

// ============================================================
// Designation (T053)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Designation {
    Town = 0,
    City = 1,
    Mine = 2,
    Farm = 3,
    Devastated = 4,
    GoldMine = 5,
    Fort = 6,
    Ruin = 7,
    Stockade = 8,
    Capitol = 9,
    Special = 10,
    LumberYard = 11,
    Blacksmith = 12,
    Road = 13,
    Mill = 14,
    Granary = 15,
    Church = 16,
    University = 17,
    NoDesig = 18,
    BaseCamp = 19,
}

/// The `des` character array from C: "tcmfx$!&sC?lb+*g=u-P0"
const DES_CHARS: &[u8] = b"tcmfx$!&sC?lb+*g=u-P";

impl Designation {
    pub fn from_char(c: char) -> Option<Self> {
        DES_CHARS.iter().position(|&ch| ch == c as u8).and_then(|i| Self::from_index(i as u8))
    }

    pub fn to_char(self) -> char {
        DES_CHARS.get(self as usize).copied().unwrap_or(b'0') as char
    }

    pub fn from_index(i: u8) -> Option<Self> {
        match i {
            0 => Some(Designation::Town),
            1 => Some(Designation::City),
            2 => Some(Designation::Mine),
            3 => Some(Designation::Farm),
            4 => Some(Designation::Devastated),
            5 => Some(Designation::GoldMine),
            6 => Some(Designation::Fort),
            7 => Some(Designation::Ruin),
            8 => Some(Designation::Stockade),
            9 => Some(Designation::Capitol),
            10 => Some(Designation::Special),
            11 => Some(Designation::LumberYard),
            12 => Some(Designation::Blacksmith),
            13 => Some(Designation::Road),
            14 => Some(Designation::Mill),
            15 => Some(Designation::Granary),
            16 => Some(Designation::Church),
            17 => Some(Designation::University),
            18 => Some(Designation::NoDesig),
            19 => Some(Designation::BaseCamp),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Designation::Town => "TOWN",
            Designation::City => "CITY",
            Designation::Mine => "MINE",
            Designation::Farm => "FARM",
            Designation::Devastated => "DEVASTATED",
            Designation::GoldMine => "GOLDMINE",
            Designation::Fort => "FORT",
            Designation::Ruin => "RUIN",
            Designation::Stockade => "STOCKADE",
            Designation::Capitol => "CAPITOL",
            Designation::Special => "SPECIAL",
            Designation::LumberYard => "LUMBERYD",
            Designation::Blacksmith => "BLKSMITH",
            Designation::Road => "ROAD",
            Designation::Mill => "MILL",
            Designation::Granary => "GRANARY",
            Designation::Church => "CHURCH",
            Designation::University => "UNIVERSITY",
            Designation::NoDesig => "NODESIG",
            Designation::BaseCamp => "BASE CAMP",
        }
    }

    /// Returns true if this designation is a city-like structure (for defense bonuses etc.)
    pub fn is_city(self) -> bool {
        matches!(self, Designation::City | Designation::Capitol | Designation::Fort | Designation::Town)
    }
}

// ============================================================
// Vegetation (T054)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Vegetation {
    Volcano = 0,
    Desert = 1,
    Tundra = 2,
    Barren = 3,
    LtVeg = 4,
    Good = 5,
    Wood = 6,
    Forest = 7,
    Jungle = 8,
    Swamp = 9,
    Ice = 10,
    None = 11,
}

/// The `veg` character array from C: "vdtblgwfjsi~0"
const VEG_CHARS: &[u8] = b"vdtblgwfjsi~";

impl Vegetation {
    pub fn from_char(c: char) -> Option<Self> {
        VEG_CHARS.iter().position(|&ch| ch == c as u8).and_then(|i| Self::from_index(i as u8))
    }

    pub fn to_char(self) -> char {
        VEG_CHARS.get(self as usize).copied().unwrap_or(b'0') as char
    }

    pub fn from_index(i: u8) -> Option<Self> {
        match i {
            0 => Some(Vegetation::Volcano),
            1 => Some(Vegetation::Desert),
            2 => Some(Vegetation::Tundra),
            3 => Some(Vegetation::Barren),
            4 => Some(Vegetation::LtVeg),
            5 => Some(Vegetation::Good),
            6 => Some(Vegetation::Wood),
            7 => Some(Vegetation::Forest),
            8 => Some(Vegetation::Jungle),
            9 => Some(Vegetation::Swamp),
            10 => Some(Vegetation::Ice),
            11 => Some(Vegetation::None),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Vegetation::Volcano => "VOLCANO",
            Vegetation::Desert => "DESERT",
            Vegetation::Tundra => "TUNDRA",
            Vegetation::Barren => "BARREN",
            Vegetation::LtVeg => "LT VEG",
            Vegetation::Good => "GOOD",
            Vegetation::Wood => "WOOD",
            Vegetation::Forest => "FOREST",
            Vegetation::Jungle => "JUNGLE",
            Vegetation::Swamp => "SWAMP",
            Vegetation::Ice => "ICE",
            Vegetation::None => "NONE",
        }
    }

    /// Food value from the vegfood array: "0004697400000"
    pub fn food_value(self) -> i32 {
        const VEGFOOD: [i32; 12] = [0, 0, 0, 4, 6, 9, 7, 4, 0, 0, 0, 0];
        VEGFOOD[self as usize]
    }
}

// ============================================================
// Altitude (T055)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Altitude {
    Water = 0,
    Peak = 1,
    Mountain = 2,
    Hill = 3,
    Clear = 4,
}

/// The `ele` character array from C: "~#^%-0"
const ELE_CHARS: &[u8] = b"~#^%-";

impl Altitude {
    pub fn from_char(c: char) -> Option<Self> {
        ELE_CHARS.iter().position(|&ch| ch == c as u8).and_then(|i| Self::from_index(i as u8))
    }

    pub fn to_char(self) -> char {
        ELE_CHARS.get(self as usize).copied().unwrap_or(b'0') as char
    }

    pub fn from_index(i: u8) -> Option<Self> {
        match i {
            0 => Some(Altitude::Water),
            1 => Some(Altitude::Peak),
            2 => Some(Altitude::Mountain),
            3 => Some(Altitude::Hill),
            4 => Some(Altitude::Clear),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Altitude::Water => "WATER",
            Altitude::Peak => "PEAK",
            Altitude::Mountain => "MOUNTAIN",
            Altitude::Hill => "HILL",
            Altitude::Clear => "FLAT",
        }
    }
}

// ============================================================
// DiplomaticStatus (T056)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum DiplomaticStatus {
    Unmet = 0,
    Treaty = 1,
    Allied = 2,
    Friendly = 3,
    Neutral = 4,
    Hostile = 5,
    War = 6,
    Jihad = 7,
}

impl DiplomaticStatus {
    pub fn from_value(v: u8) -> Option<Self> {
        match v {
            0 => Some(DiplomaticStatus::Unmet),
            1 => Some(DiplomaticStatus::Treaty),
            2 => Some(DiplomaticStatus::Allied),
            3 => Some(DiplomaticStatus::Friendly),
            4 => Some(DiplomaticStatus::Neutral),
            5 => Some(DiplomaticStatus::Hostile),
            6 => Some(DiplomaticStatus::War),
            7 => Some(DiplomaticStatus::Jihad),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            DiplomaticStatus::Unmet => "UNMET",
            DiplomaticStatus::Treaty => "TREATY",
            DiplomaticStatus::Allied => "ALLIED",
            DiplomaticStatus::Friendly => "FRIENDLY",
            DiplomaticStatus::Neutral => "NEUTRAL",
            DiplomaticStatus::Hostile => "HOSTILE",
            DiplomaticStatus::War => "WAR",
            DiplomaticStatus::Jihad => "JIHAD",
        }
    }
}

// ============================================================
// ArmyStatus (T057)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ArmyStatus {
    March = 1,
    Scout = 2,
    Garrison = 3,
    Traded = 4,
    Militia = 5,
    Flight = 6,
    Defend = 7,
    MagDef = 8,
    Attack = 9,
    MagAtt = 10,
    General = 11,
    Sortie = 12,
    Siege = 13,
    Sieged = 14,
    OnBoard = 15,
    Rule = 16,
    /// Group membership: value = NUMSTATUS + group_id
    /// Groups are always in attack mode and may not be magicked
    Group(u8),
}

/// Number of individual statuses (group IDs start at NUMSTATUS)
pub const NUMSTATUS: u8 = 17;

impl ArmyStatus {
    pub fn from_value(v: u8) -> Self {
        match v {
            1 => ArmyStatus::March,
            2 => ArmyStatus::Scout,
            3 => ArmyStatus::Garrison,
            4 => ArmyStatus::Traded,
            5 => ArmyStatus::Militia,
            6 => ArmyStatus::Flight,
            7 => ArmyStatus::Defend,
            8 => ArmyStatus::MagDef,
            9 => ArmyStatus::Attack,
            10 => ArmyStatus::MagAtt,
            11 => ArmyStatus::General,
            12 => ArmyStatus::Sortie,
            13 => ArmyStatus::Siege,
            14 => ArmyStatus::Sieged,
            15 => ArmyStatus::OnBoard,
            16 => ArmyStatus::Rule,
            v if v >= NUMSTATUS => ArmyStatus::Group(v - NUMSTATUS),
            _ => ArmyStatus::March, // 0 or invalid defaults to March
        }
    }

    pub fn to_value(self) -> u8 {
        match self {
            ArmyStatus::March => 1,
            ArmyStatus::Scout => 2,
            ArmyStatus::Garrison => 3,
            ArmyStatus::Traded => 4,
            ArmyStatus::Militia => 5,
            ArmyStatus::Flight => 6,
            ArmyStatus::Defend => 7,
            ArmyStatus::MagDef => 8,
            ArmyStatus::Attack => 9,
            ArmyStatus::MagAtt => 10,
            ArmyStatus::General => 11,
            ArmyStatus::Sortie => 12,
            ArmyStatus::Siege => 13,
            ArmyStatus::Sieged => 14,
            ArmyStatus::OnBoard => 15,
            ArmyStatus::Rule => 16,
            ArmyStatus::Group(g) => NUMSTATUS + g,
        }
    }

    pub fn is_group(self) -> bool {
        matches!(self, ArmyStatus::Group(_))
    }

    pub fn name(self) -> &'static str {
        match self {
            ArmyStatus::March => "MARCH",
            ArmyStatus::Scout => "SCOUT",
            ArmyStatus::Garrison => "GARRISON",
            ArmyStatus::Traded => "TRADED",
            ArmyStatus::Militia => "MILITIA",
            ArmyStatus::Flight => "FLYING",
            ArmyStatus::Defend => "DEFEND",
            ArmyStatus::MagDef => "MAG_DEF",
            ArmyStatus::Attack => "ATTACK",
            ArmyStatus::MagAtt => "MAG_ATT",
            ArmyStatus::General => "GENERAL",
            ArmyStatus::Sortie => "SORTIE",
            ArmyStatus::Siege => "SIEGE",
            ArmyStatus::Sieged => "BESIEGED",
            ArmyStatus::OnBoard => "ON_BOARD",
            ArmyStatus::Rule => "RULE",
            ArmyStatus::Group(_) => "GROUP",
        }
    }
}

// ============================================================
// UnitType (T058)
// ============================================================

/// Unit types with three ranges:
/// - Base units: 0-26 (A_MILITIA through A_SCOUT)
/// - Leaders: 27+UTYPE(75) = 102..119 (L_KING through L_NAZGUL)
/// - Monsters: 45+TWOUTYPE(150) = 195..209 (SPIRIT through DRAGON)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitType(pub u8);

// Base unit types (0-26)
impl UnitType {
    pub const MILITIA: UnitType = UnitType(0);
    pub const GOBLIN: UnitType = UnitType(1);
    pub const ORC: UnitType = UnitType(2);
    pub const INFANTRY: UnitType = UnitType(3);
    pub const SAILOR: UnitType = UnitType(4);
    pub const MARINES: UnitType = UnitType(5);
    pub const ARCHER: UnitType = UnitType(6);
    pub const URUK: UnitType = UnitType(7);
    pub const NINJA: UnitType = UnitType(8);
    pub const PHALANX: UnitType = UnitType(9);
    pub const OLOG: UnitType = UnitType(10);
    pub const LEGION: UnitType = UnitType(11);
    pub const DRAGOON: UnitType = UnitType(12);
    pub const MERCENARY: UnitType = UnitType(13);
    pub const TROLL: UnitType = UnitType(14);
    pub const ELITE: UnitType = UnitType(15);
    pub const LT_CAV: UnitType = UnitType(16);
    pub const CAVALRY: UnitType = UnitType(17);
    pub const CATAPULT: UnitType = UnitType(18);
    pub const SIEGE_UNIT: UnitType = UnitType(19);
    pub const ROC: UnitType = UnitType(20);
    pub const KNIGHT: UnitType = UnitType(21);
    pub const GRIFFON: UnitType = UnitType(22);
    pub const ELEPHANT: UnitType = UnitType(23);
    pub const ZOMBIE: UnitType = UnitType(24);
    pub const SPY: UnitType = UnitType(25);
    pub const SCOUT: UnitType = UnitType(26);

    /// Number of base unit types (not counting leaders/monsters)
    pub const NUM_UNIT_TYPES: u8 = 26;

    // Leader types (27 + UTYPE=75 = 102..119)
    pub const L_KING: UnitType = UnitType(27 + 75);
    pub const L_BARON: UnitType = UnitType(28 + 75);
    pub const L_EMPEROR: UnitType = UnitType(29 + 75);
    pub const L_PRINCE: UnitType = UnitType(30 + 75);
    pub const L_WIZARD: UnitType = UnitType(31 + 75);
    pub const L_MAGI: UnitType = UnitType(32 + 75);
    pub const L_APOSTLE: UnitType = UnitType(33 + 75);
    pub const L_BISHOP: UnitType = UnitType(34 + 75);
    pub const L_ADMIRAL: UnitType = UnitType(35 + 75);
    pub const L_CAPTAIN: UnitType = UnitType(36 + 75);
    pub const L_WARLORD: UnitType = UnitType(37 + 75);
    pub const L_LORD: UnitType = UnitType(38 + 75);
    pub const L_DEMON: UnitType = UnitType(39 + 75);
    pub const L_DEVIL: UnitType = UnitType(40 + 75);
    pub const L_DRAGON: UnitType = UnitType(41 + 75);
    pub const L_WYRM: UnitType = UnitType(42 + 75);
    pub const L_SHADOW: UnitType = UnitType(43 + 75);
    pub const L_NAZGUL: UnitType = UnitType(44 + 75);

    /// Minimum value for a leader type (MINLEADER in C is 27+UTYPE)
    pub const MIN_LEADER: u8 = 27 + 75;

    // Monster types (45 + TWOUTYPE=150 = 195..209)
    pub const SPIRIT: UnitType = UnitType(45 + 150);
    pub const ASSASSIN: UnitType = UnitType(46 + 150);
    pub const DJINNI: UnitType = UnitType(47 + 150);
    pub const GARGOYLE: UnitType = UnitType(48 + 150);
    pub const WRAITH: UnitType = UnitType(49 + 150);
    pub const HERO: UnitType = UnitType(50 + 150);
    pub const CENTAUR: UnitType = UnitType(51 + 150);
    pub const GIANT: UnitType = UnitType(52 + 150);
    pub const SUPERHERO: UnitType = UnitType(53 + 150);
    pub const MUMMY: UnitType = UnitType(54 + 150);
    pub const ELEMENTAL: UnitType = UnitType(55 + 150);
    pub const MINOTAUR: UnitType = UnitType(56 + 150);
    pub const DEMON: UnitType = UnitType(57 + 150);
    pub const BALROG: UnitType = UnitType(58 + 150);
    pub const DRAGON: UnitType = UnitType(59 + 150);

    /// Minimum value for a monster type
    pub const MIN_MONSTER: u8 = 45 + 150;
    /// Maximum value for a monster type
    pub const MAX_MONSTER: u8 = 59 + 150;

    pub fn is_base_unit(self) -> bool {
        self.0 <= Self::NUM_UNIT_TYPES
    }

    pub fn is_leader(self) -> bool {
        self.0 >= Self::MIN_LEADER && self.0 < Self::MIN_MONSTER
    }

    pub fn is_monster(self) -> bool {
        self.0 >= Self::MIN_MONSTER && self.0 <= Self::MAX_MONSTER
    }

    /// Get the index into the unit stats arrays.
    /// Base units: 0-26 map directly
    /// Leaders: subtract UTYPE to get index 27-44
    /// Monsters: subtract TWOUTYPE to get index 45-59
    pub fn stats_index(self) -> Option<usize> {
        if self.0 <= Self::NUM_UNIT_TYPES {
            Some(self.0 as usize)
        } else if self.is_leader() {
            Some((self.0 - 75) as usize)
        } else if self.is_monster() {
            Some((self.0 - 150) as usize)
        } else {
            None
        }
    }

    /// Get unit display name
    pub fn name(self) -> &'static str {
        self.stats_index()
            .and_then(|i| crate::tables::UNIT_TYPE_NAMES.get(i))
            .copied()
            .unwrap_or("Unknown")
    }
}

// ============================================================
// Season (T059)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Season {
    Winter = 0,
    Spring = 1,
    Summer = 2,
    Fall = 3,
}

impl Season {
    pub fn from_turn(turn: i16) -> Self {
        match (turn % 4) as u8 {
            0 => Season::Winter,
            1 => Season::Spring,
            2 => Season::Summer,
            3 => Season::Fall,
            _ => unreachable!(),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Season::Winter => "Winter",
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Fall => "Fall",
        }
    }
}

/// Calculate year from turn: (turn + 3) / 4
pub fn year_from_turn(turn: i16) -> i32 {
    ((turn as i32) + 3) / 4
}

// ============================================================
// NationClass (T060)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum NationClass {
    Npc = 0,
    King = 1,
    Emperor = 2,
    Wizard = 3,
    Priest = 4,
    Pirate = 5,
    Trader = 6,
    Warlord = 7,
    Demon = 8,
    Dragon = 9,
    Shadow = 10,
}

impl NationClass {
    pub fn from_value(v: i16) -> Option<Self> {
        match v {
            0 => Some(NationClass::Npc),
            1 => Some(NationClass::King),
            2 => Some(NationClass::Emperor),
            3 => Some(NationClass::Wizard),
            4 => Some(NationClass::Priest),
            5 => Some(NationClass::Pirate),
            6 => Some(NationClass::Trader),
            7 => Some(NationClass::Warlord),
            8 => Some(NationClass::Demon),
            9 => Some(NationClass::Dragon),
            10 => Some(NationClass::Shadow),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            NationClass::Npc => "monster",
            NationClass::King => "king",
            NationClass::Emperor => "emperor",
            NationClass::Wizard => "wizard",
            NationClass::Priest => "priest",
            NationClass::Pirate => "pirate",
            NationClass::Trader => "trader",
            NationClass::Warlord => "warlord",
            NationClass::Demon => "demon",
            NationClass::Dragon => "dragon",
            NationClass::Shadow => "shadow",
        }
    }
}

// ============================================================
// NationStrategy (T061)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum NationStrategy {
    Inactive = 0,
    PcGood = 1,
    PcNeutral = 2,
    PcEvil = 3,
    Good0Free = 4,
    Good2Free = 5,
    Good4Free = 6,
    Good6Free = 7,
    Neutral0Free = 8,
    Neutral2Free = 9,
    Neutral4Free = 10,
    Neutral6Free = 11,
    Evil0Free = 12,
    Evil2Free = 13,
    Evil4Free = 14,
    Evil6Free = 15,
    Isolationist = 16,
    NpcPeasant = 17,
    NpcPirate = 18,
    NpcLizard = 19,
    NpcNomad = 20,
    NpcSavage = 21,
}

impl NationStrategy {
    pub fn from_value(v: u8) -> Option<Self> {
        match v {
            0 => Some(NationStrategy::Inactive),
            1 => Some(NationStrategy::PcGood),
            2 => Some(NationStrategy::PcNeutral),
            3 => Some(NationStrategy::PcEvil),
            4 => Some(NationStrategy::Good0Free),
            5 => Some(NationStrategy::Good2Free),
            6 => Some(NationStrategy::Good4Free),
            7 => Some(NationStrategy::Good6Free),
            8 => Some(NationStrategy::Neutral0Free),
            9 => Some(NationStrategy::Neutral2Free),
            10 => Some(NationStrategy::Neutral4Free),
            11 => Some(NationStrategy::Neutral6Free),
            12 => Some(NationStrategy::Evil0Free),
            13 => Some(NationStrategy::Evil2Free),
            14 => Some(NationStrategy::Evil4Free),
            15 => Some(NationStrategy::Evil6Free),
            16 => Some(NationStrategy::Isolationist),
            17 => Some(NationStrategy::NpcPeasant),
            18 => Some(NationStrategy::NpcPirate),
            19 => Some(NationStrategy::NpcLizard),
            20 => Some(NationStrategy::NpcNomad),
            21 => Some(NationStrategy::NpcSavage),
            _ => None,
        }
    }

    pub fn to_value(self) -> u8 {
        self as u8
    }

    /// Is this a player-controlled nation?
    /// `ispc(x) = (x==PC_GOOD || x==PC_EVIL || x==PC_NEUTRAL)`
    pub fn is_pc(self) -> bool {
        matches!(self, NationStrategy::PcGood | NationStrategy::PcNeutral | NationStrategy::PcEvil)
    }

    /// Is this a regular NPC nation (not monster, not PC)?
    /// `isnpc(x) = (x >= GOOD_0FREE && x <= ISOLATIONIST)`
    pub fn is_npc(self) -> bool {
        let v = self as u8;
        v >= 4 && v <= 16
    }

    /// Is this a monster nation?
    /// `ismonst(x) = (x >= NPC_PEASANT)`
    pub fn is_monster(self) -> bool {
        (self as u8) >= 17
    }

    /// Is this nation active (not inactive)?
    /// `isactive(x) = (x != INACTIVE)`
    pub fn is_active(self) -> bool {
        self != NationStrategy::Inactive
    }

    /// Is this a peasant revolt?
    /// `ispeasant(x) = (x == NPC_PEASANT)`
    pub fn is_peasant(self) -> bool {
        self == NationStrategy::NpcPeasant
    }

    /// Is this a regular nation (active and not monster except peasant)?
    /// `isntn(x) = (x != INACTIVE && x <= ISOLATIONIST)`
    pub fn is_nation(self) -> bool {
        let v = self as u8;
        v != 0 && v <= 16
    }

    /// Is this a nation or peasant?
    /// `isntnorp(x) = (x != INACTIVE && x <= NPC_PEASANT)`
    pub fn is_nation_or_peasant(self) -> bool {
        let v = self as u8;
        v != 0 && v <= 17
    }

    /// Is this not a PC (but not inactive either)?
    /// `isnotpc(x) = (x >= GOOD_0FREE && x != INACTIVE)`
    pub fn is_not_pc(self) -> bool {
        let v = self as u8;
        v >= 4 && v != 0
    }

    /// NPC type classification (for alignment).
    /// `npctype(x) = ispc(x) ? x : (ismonst(x) ? 0 : x/4)`
    pub fn npc_type(self) -> u8 {
        if self.is_pc() {
            self as u8
        } else if self.is_monster() {
            0
        } else {
            (self as u8) / 4
        }
    }

    /// Is this a good-aligned nation?
    /// `isgood(x) = (npctype(x) == 1)`
    pub fn is_good(self) -> bool {
        self.npc_type() == 1
    }

    /// Is this a neutral-aligned nation?
    /// `isneutral(x) = (npctype(x) == 2)`
    pub fn is_neutral(self) -> bool {
        self.npc_type() == 2
    }

    /// Is this an evil-aligned nation?
    /// `isevil(x) = (npctype(x) == 3)`
    pub fn is_evil(self) -> bool {
        self.npc_type() == 3
    }
}

// ============================================================
// Direction (T062)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Direction {
    Centered = 0,
    North = 1,
    NorthEast = 2,
    East = 3,
    SouthEast = 4,
    South = 5,
    SouthWest = 6,
    West = 7,
    NorthWest = 8,
}

impl Direction {
    pub fn name(self) -> &'static str {
        match self {
            Direction::Centered => "here",
            Direction::North => "north",
            Direction::NorthEast => "northeast",
            Direction::East => "east",
            Direction::SouthEast => "southeast",
            Direction::South => "south",
            Direction::SouthWest => "southwest",
            Direction::West => "west",
            Direction::NorthWest => "northwest",
        }
    }
}

// ============================================================
// NavalSize (T063)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum NavalSize {
    Light = 0,
    Medium = 1,
    Heavy = 2,
}

impl NavalSize {
    /// Extract ship count from a packed u16 field.
    /// Matches C macro: `SHIPS(x,y) = (short)((x & (N_MASK << (y*N_BITSIZE))) >> (y*N_BITSIZE))`
    pub fn ships(packed: u16, size: NavalSize) -> i16 {
        let shift = (size as u32) * crate::constants::N_BITSIZE;
        ((packed & (crate::constants::N_MASK << shift as u16)) >> shift as u16) as i16
    }

    /// Set ship count in a packed u16 field for a given size.
    pub fn set_ships(packed: u16, size: NavalSize, count: u16) -> u16 {
        let shift = (size as u32) * crate::constants::N_BITSIZE;
        let mask = (crate::constants::N_MASK as u16) << shift;
        (packed & !mask) | ((count & crate::constants::N_MASK) << shift)
    }
}

// ============================================================
// TradeGood (T064)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum TradeGood {
    Furs = 0,
    Wool = 1,
    Beer = 2,
    Cloth = 3,
    Wine = 4,
    Mules = 5,
    Horses = 6,
    Pigeons = 7,
    Griffons = 8,
    Corn = 9,
    Fish = 10,
    Sugar = 11,
    Honey = 12,
    Fruit = 13,
    Rice = 14,
    Wheat = 15,
    Dairy = 16,
    Peas = 17,
    Bread = 18,
    Cereal = 19,
    Pottery = 20,
    Salt = 21,
    Timber = 22,
    Granite = 23,
    Pine = 24,
    Oak = 25,
    Nails = 26,
    Papyrus = 27,
    Math = 28,
    Library = 29,
    Drama = 30,
    Paper = 31,
    Literature = 32,
    Law = 33,
    Philosophy = 34,
    Irrigation = 35,
    Oxen = 36,
    Plows = 37,
    Stones = 38,
    Herbs = 39,
    Medicine = 40,
    Torture = 41,
    Prison = 42,
    Bronze = 43,
    Copper = 44,
    Lead = 45,
    Tin = 46,
    Iron = 47,
    Steel = 48,
    Mithral = 49,
    Adamantine = 50,
    Spice = 51,
    Silver = 52,
    Pearls = 53,
    Dye = 54,
    Silk = 55,
    Gold = 56,
    Rubys = 57,
    Ivory = 58,
    Diamonds = 59,
    Platinum = 60,
    None = 61,
}

// Category boundaries
pub const END_POPULARITY: u8 = 4;
pub const END_COMMUNICATION: u8 = 8;
pub const END_EATRATE: u8 = 19;
pub const END_SPOILRATE: u8 = 26;
pub const END_KNOWLEDGE: u8 = 34;
pub const END_FARM: u8 = 37;
pub const END_SPELL: u8 = 38;
pub const END_HEALTH: u8 = 40;
pub const END_TERROR: u8 = 42;
pub const END_NORMAL: u8 = 42;
pub const END_MINE: u8 = 50;
pub const END_WEALTH: u8 = 60;

impl TradeGood {
    pub fn from_value(v: u8) -> Option<Self> {
        if v <= 61 {
            // SAFETY: repr(u8) and all values 0-61 are valid variants
            Some(unsafe { std::mem::transmute(v) })
        } else {
            None
        }
    }

    pub fn is_mine_good(self) -> bool {
        let v = self as u8;
        v > END_NORMAL && v <= END_MINE
    }

    pub fn is_wealth_good(self) -> bool {
        let v = self as u8;
        v > END_MINE && v <= END_WEALTH
    }
}

// ============================================================
// HighlightMode (T066)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum HighlightMode {
    Own = 0,
    Army = 1,
    None = 2,
    YourArmy = 3,
    Move = 4,
    Good = 5,
}

// ============================================================
// DisplayMode (T067)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum DisplayMode {
    Vegetation = 1,
    Designation = 2,
    Contour = 3,
    Food = 4,
    Nation = 5,
    Race = 6,
    Move = 7,
    Defense = 8,
    People = 9,
    Gold = 10,
    Metal = 11,
    Items = 12,
}

// ============================================================
// NationPlacement (T068)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NationPlacement {
    Great,
    Fair,
    Random,
    Oops,
}

impl NationPlacement {
    pub fn from_char(c: char) -> Self {
        match c {
            'G' => NationPlacement::Great,
            'F' => NationPlacement::Fair,
            'R' => NationPlacement::Random,
            _ => NationPlacement::Oops,
        }
    }

    pub fn to_char(self) -> char {
        match self {
            NationPlacement::Great => 'G',
            NationPlacement::Fair => 'F',
            NationPlacement::Random => 'R',
            NationPlacement::Oops => 'X',
        }
    }
}

// ============================================================
// MailStatus (T069)
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i8)]
pub enum MailStatus {
    DoneMail = -3,
    NewsMail = -2,
    AbortMail = -1,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_race_roundtrip() {
        let races = [
            (Race::God, '-'),
            (Race::Orc, 'O'),
            (Race::Elf, 'E'),
            (Race::Dwarf, 'D'),
            (Race::Lizard, 'L'),
            (Race::Human, 'H'),
            (Race::Pirate, 'P'),
            (Race::Savage, 'S'),
            (Race::Nomad, 'N'),
            (Race::Unknown, '?'),
        ];
        for (race, ch) in &races {
            assert_eq!(race.to_char(), *ch);
            assert_eq!(Race::from_char(*ch), *race);
        }
    }

    #[test]
    fn test_designation_roundtrip() {
        for i in 0..20u8 {
            let d = Designation::from_index(i).unwrap();
            let ch = d.to_char();
            assert_eq!(Designation::from_char(ch), Some(d), "index {} char {}", i, ch);
        }
    }

    #[test]
    fn test_vegetation_roundtrip() {
        for i in 0..12u8 {
            let v = Vegetation::from_index(i).unwrap();
            let ch = v.to_char();
            assert_eq!(Vegetation::from_char(ch), Some(v));
        }
    }

    #[test]
    fn test_altitude_roundtrip() {
        for i in 0..5u8 {
            let a = Altitude::from_index(i).unwrap();
            let ch = a.to_char();
            assert_eq!(Altitude::from_char(ch), Some(a));
        }
    }

    #[test]
    fn test_diplomatic_status_values() {
        assert_eq!(DiplomaticStatus::Unmet as u8, 0);
        assert_eq!(DiplomaticStatus::Treaty as u8, 1);
        assert_eq!(DiplomaticStatus::Allied as u8, 2);
        assert_eq!(DiplomaticStatus::Friendly as u8, 3);
        assert_eq!(DiplomaticStatus::Neutral as u8, 4);
        assert_eq!(DiplomaticStatus::Hostile as u8, 5);
        assert_eq!(DiplomaticStatus::War as u8, 6);
        assert_eq!(DiplomaticStatus::Jihad as u8, 7);
    }

    #[test]
    fn test_army_status_roundtrip() {
        for v in 1..=16u8 {
            let s = ArmyStatus::from_value(v);
            assert_eq!(s.to_value(), v);
        }
        // Test group
        let g = ArmyStatus::from_value(20);
        assert_eq!(g, ArmyStatus::Group(3));
        assert_eq!(g.to_value(), 20);
    }

    #[test]
    fn test_unit_type_ranges() {
        assert!(UnitType::MILITIA.is_base_unit());
        assert!(UnitType::SCOUT.is_base_unit());
        assert!(!UnitType::L_KING.is_base_unit());
        assert!(UnitType::L_KING.is_leader());
        assert!(UnitType::L_NAZGUL.is_leader());
        assert!(!UnitType::L_KING.is_monster());
        assert!(UnitType::SPIRIT.is_monster());
        assert!(UnitType::DRAGON.is_monster());
        assert!(!UnitType::DRAGON.is_leader());

        // Verify exact values
        assert_eq!(UnitType::L_KING.0, 102); // 27+75
        assert_eq!(UnitType::L_NAZGUL.0, 119); // 44+75
        assert_eq!(UnitType::SPIRIT.0, 195); // 45+150
        assert_eq!(UnitType::DRAGON.0, 209); // 59+150
    }

    #[test]
    fn test_unit_type_stats_index() {
        assert_eq!(UnitType::MILITIA.stats_index(), Some(0));
        assert_eq!(UnitType::SCOUT.stats_index(), Some(26));
        assert_eq!(UnitType::L_KING.stats_index(), Some(27)); // 102-75=27
        assert_eq!(UnitType::SPIRIT.stats_index(), Some(45)); // 195-150=45
        assert_eq!(UnitType::DRAGON.stats_index(), Some(59)); // 209-150=59
    }

    #[test]
    fn test_season_from_turn() {
        assert_eq!(Season::from_turn(0), Season::Winter);
        assert_eq!(Season::from_turn(1), Season::Spring);
        assert_eq!(Season::from_turn(2), Season::Summer);
        assert_eq!(Season::from_turn(3), Season::Fall);
        assert_eq!(Season::from_turn(4), Season::Winter);
        assert_eq!(Season::from_turn(7), Season::Fall);
    }

    #[test]
    fn test_nation_strategy_macros() {
        // PC checks
        assert!(NationStrategy::PcGood.is_pc());
        assert!(NationStrategy::PcNeutral.is_pc());
        assert!(NationStrategy::PcEvil.is_pc());
        assert!(!NationStrategy::Good0Free.is_pc());

        // NPC checks
        assert!(NationStrategy::Good0Free.is_npc());
        assert!(NationStrategy::Isolationist.is_npc());
        assert!(!NationStrategy::NpcPeasant.is_npc());
        assert!(!NationStrategy::PcGood.is_npc());

        // Monster checks
        assert!(NationStrategy::NpcPeasant.is_monster());
        assert!(NationStrategy::NpcPirate.is_monster());
        assert!(NationStrategy::NpcSavage.is_monster());
        assert!(!NationStrategy::Isolationist.is_monster());

        // Alignment checks
        assert!(NationStrategy::PcGood.is_good());
        assert!(NationStrategy::Good0Free.is_good());
        assert!(NationStrategy::Good6Free.is_good());
        assert!(NationStrategy::PcNeutral.is_neutral());
        assert!(NationStrategy::Neutral0Free.is_neutral());
        assert!(NationStrategy::PcEvil.is_evil());
        assert!(NationStrategy::Evil0Free.is_evil());
    }

    #[test]
    fn test_naval_ship_packing() {
        // Pack 3 light, 2 medium, 1 heavy into a u16
        let mut packed = 0u16;
        packed = NavalSize::set_ships(packed, NavalSize::Light, 3);
        packed = NavalSize::set_ships(packed, NavalSize::Medium, 2);
        packed = NavalSize::set_ships(packed, NavalSize::Heavy, 1);

        assert_eq!(NavalSize::ships(packed, NavalSize::Light), 3);
        assert_eq!(NavalSize::ships(packed, NavalSize::Medium), 2);
        assert_eq!(NavalSize::ships(packed, NavalSize::Heavy), 1);
    }

    #[test]
    fn test_vegetation_food_values() {
        assert_eq!(Vegetation::Volcano.food_value(), 0);
        assert_eq!(Vegetation::Desert.food_value(), 0);
        assert_eq!(Vegetation::Barren.food_value(), 4);
        assert_eq!(Vegetation::LtVeg.food_value(), 6);
        assert_eq!(Vegetation::Good.food_value(), 9);
        assert_eq!(Vegetation::Wood.food_value(), 7);
        assert_eq!(Vegetation::Forest.food_value(), 4);
    }

    #[test]
    fn test_trade_good_categories() {
        assert!(TradeGood::Bronze.is_mine_good());
        assert!(TradeGood::Adamantine.is_mine_good());
        assert!(!TradeGood::Furs.is_mine_good());
        assert!(TradeGood::Spice.is_wealth_good());
        assert!(TradeGood::Platinum.is_wealth_good());
        assert!(!TradeGood::Bronze.is_wealth_good());
    }

    #[test]
    fn test_designation_is_city() {
        assert!(Designation::City.is_city());
        assert!(Designation::Capitol.is_city());
        assert!(Designation::Fort.is_city());
        assert!(Designation::Town.is_city());
        assert!(!Designation::Mine.is_city());
        assert!(!Designation::Farm.is_city());
    }
}
