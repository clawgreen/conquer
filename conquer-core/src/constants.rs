// ============================================================
// Core Game Limits (from header.h)
// ============================================================

/// Max number of nations (player + NPC + monster)
pub const NTOTAL: usize = 35;
/// Max number of nations - alias for array sizing (from C: MAXNTOTAL)
pub const MAXNTOTAL: usize = NTOTAL;
/// Points for players to buy stuff with at start
pub const MAXPTS: i16 = 65;
/// Maximum number of armies per nation
pub const MAXARM: usize = 50;
/// Maximum number of fleets per nation
pub const MAXNAVY: usize = 10;
/// % of armies/sectors depleted without Capitol
pub const PDEPLETE: i32 = 30;
/// Percentage chance for capturing scouts
pub const PFINDSCOUT: i32 = 50;

// ============================================================
// Game Features
// ============================================================

/// Percent of sectors with exotic trade goods
pub const TRADEPCT: i32 = 75;
/// Percent of tradegoods that are metals
pub const METALPCT: i32 = 33;
/// Percent of tradegoods that are luxury items
pub const JEWELPCT: i32 = 33;
/// Allow for this many revolts in nation list
pub const REVSPACE: i32 = 5;
/// Last turn players may join without password
pub const LASTADD: i16 = 5;

// ============================================================
// Monster and NPC Configuration
// ============================================================

/// Sectors of land per pirate/savage/nomad nation
pub const MONSTER: i32 = 45;
/// Sectors of land per non-player character nation
pub const NPC: i32 = 45;

// ============================================================
// Environmental Effects
// ============================================================

/// % chance of volcanic eruption each round
pub const PVULCAN: i32 = 20;
/// Jewel cost for orc takeover
pub const ORCTAKE: i64 = 100_000;
/// Cost per move/screen
pub const MOVECOST_GOLD: i64 = 20;
/// Spell points for orc takeover
pub const TAKEPOINTS: i32 = 10;
/// % of land that is mountains
pub const PMOUNT: i32 = 40;
/// % chance storm strikes fleet
pub const PSTORM: i32 = 3;

// ============================================================
// Random Events
// ============================================================

/// Enable random events (% chance)
pub const RANEVENT: i32 = 15;
/// Percent for weather disasters
pub const PWEATHER: i32 = 0;
/// %/turn that a revolt actually occurs
pub const PREVOLT: i32 = 25;

// ============================================================
// Vision and Movement Ranges
// ============================================================

/// How far you can see from your land
pub const LANDSEE: i32 = 2;
/// How far navies can see
pub const NAVYSEE: i32 = 1;
/// How far armies can see
pub const ARMYSEE: i32 = 2;
/// How far pirates roam from basecamp
pub const PRTZONE: i32 = 3;
/// How close nations must be to adjust status
pub const MEETNTN: i32 = 2;

// ============================================================
// Taxation Rates (in gold talons per unit)
// ============================================================

/// Per food unit
pub const TAXFOOD: i64 = 5;
/// Per metal unit
pub const TAXMETAL: i64 = 8;
/// Per gold unit
pub const TAXGOLD: i64 = 8;
/// Per food point equivalent (other)
pub const TAXOTHR: i64 = 3;
/// Per person in city
pub const TAXCITY: i64 = 100;
/// Per person in town
pub const TAXTOWN: i64 = 80;

// ============================================================
// Economic Parameters
// ============================================================

/// Ship maintenance cost
pub const SHIPMAINT: i64 = 4000;
/// Overpopulation threshold
pub const TOMANYPEOPLE: i64 = 4000;
/// Absolute max people in any sector
pub const ABSMAXPEOPLE: i64 = 50_000;
/// Min people to work a mill
pub const MILLSIZE: i64 = 500;
/// Units mined for 100% depletion chance
pub const TOMUCHMINED: i64 = 50_000;
/// Min food value to redesignate sector
pub const DESFOOD: i32 = 4;
/// Number of news files stored
pub const MAXNEWS: i32 = 5;
/// Navy trip length for 100% attrition
pub const LONGTRIP: i32 = 100;

// ============================================================
// Combat and Military
// ============================================================

/// Maximum % of men lost in 1:1 battle
pub const MAXLOSS: i32 = 60;
/// Percent chance to find gold/metal
pub const FINDPERCENT: i32 = 1;
/// Cost to redesignate + metal cost for cities
pub const DESCOST: i64 = 2000;
/// Cost to build a fort point
pub const FORTCOST: i64 = 1000;
/// Cost to build a stockade
pub const STOCKCOST: i64 = 3000;
/// Cost to remove a ruin
pub const REBUILDCOST: i64 = 3000;

// ============================================================
// Naval Configuration
// ============================================================

/// Cost to build one light warship
pub const WARSHPCOST: i64 = 20_000;
/// Cost to build one light merchant
pub const MERSHPCOST: i64 = 25_000;
/// Cost to build one light galley
pub const GALSHPCOST: i64 = 25_000;
/// Movement lost in (un)loading in cities
pub const N_CITYCOST: u8 = 4;
/// Full strength crew on a ship
pub const SHIPCREW: i32 = 100;
/// Storage space of a ship unit
pub const SHIPHOLD: i64 = 100;
/// Bitfield size for ship counts (light/medium/heavy)
pub const N_BITSIZE: u32 = 5;
/// Bitmask for extracting ship count from packed field
pub const N_MASK: u16 = 0x001f;

/// Speed of warships
pub const N_WSPD: i32 = 20;
/// Speed of galleys
pub const N_GSPD: i32 = 18;
/// Speed of merchants
pub const N_MSPD: i32 = 15;
/// No ships no speed
pub const N_NOSPD: i32 = 0;
/// Bonus speed for lighter ships
pub const N_SIZESPD: i32 = 3;

// ============================================================
// NPC Behavior Parameters
// ============================================================

/// % of NPC pop in sector before => city
pub const CITYLIMIT: i64 = 8;
/// % of NPC pop able to be in cities
pub const CITYPERCENT: i64 = 20;
/// Ratio civ:mil for NPCs
pub const MILRATIO: i64 = 8;
/// Ratio (mil in cap):mil for NPCs
pub const MILINCAP: i64 = 8;
/// Militia = people/MILINCITY in city/cap
pub const MILINCITY: i64 = 10;
/// NPCs shouldn't go this far from capitol
pub const NPCTOOFAR: i32 = 15;
/// Gold/1000 men to bribe
pub const BRIBE: i64 = 50_000;
/// Metal/soldier needed for +1% weapons
pub const METALORE: i64 = 7;

// ============================================================
// Defense Values
// ============================================================

/// Base defense value, 2x in city/caps
pub const DEF_BASE: i32 = 10;
/// Percent per fortress point in forts
pub const FORTSTR: i32 = 5;
/// Percent per fortress point in towns
pub const TOWNSTR: i32 = 5;
/// Percent per fortress point in cities
pub const CITYSTR: i32 = 8;
/// New player gets 1 point / LATESTART turns
pub const LATESTART: i32 = 2;

// ============================================================
// Starting Mercenary Values
// ============================================================

/// Starting mercenary pool
pub const ST_MMEN: i64 = (NTOTAL as i64) * 500;
/// Mercenary attack bonus
pub const ST_MATT: i16 = 40;
/// Mercenary defense bonus
pub const ST_MDEF: i16 = 40;

// ============================================================
// Magic/Civilian/Military Power Costs by Race
// ============================================================

pub const BASEMAGIC: i64 = 50_000;
pub const DWFMAGIC: i64 = 80_000;
pub const HUMMAGIC: i64 = 100_000;
pub const ORCMAGIC: i64 = 150_000;
pub const DWFCIVIL: i64 = 40_000;
pub const ORCCIVIL: i64 = 75_000;
pub const HUMCIVIL: i64 = 25_000;
pub const DWFMILIT: i64 = 40_000;
pub const ORCMILIT: i64 = 45_000;

// ============================================================
// Attractiveness Constants (General)
// ============================================================

pub const GOLDATTR: i32 = 9;
pub const FARMATTR: i32 = 7;
pub const MINEATTR: i32 = 9;
pub const TOWNATTR: i32 = 150;
pub const CITYATTR: i32 = 300;
pub const TGATTR: i32 = 10;
pub const OTHRATTR: i32 = 50;

// ============================================================
// Attractiveness Constants by Race
// ============================================================

// Dwarf
pub const DMNTNATTR: i32 = 40;
pub const DHILLATTR: i32 = 20;
pub const DCLERATTR: i32 = 0;
pub const DCITYATTR: i32 = -20;
pub const DTOWNATTR: i32 = -20;
pub const DGOLDATTR: i32 = 40;
pub const DMINEATTR: i32 = 40;
pub const DFOREATTR: i32 = -20;
pub const DWOODATTR: i32 = -10;

// Elf
pub const EMNTNATTR: i32 = -40;
pub const EHILLATTR: i32 = -20;
pub const ECLERATTR: i32 = 0;
pub const ECITYATTR: i32 = -50;
pub const ETOWNATTR: i32 = -50;
pub const EGOLDATTR: i32 = 0;
pub const EMINEATTR: i32 = 0;
pub const EFOREATTR: i32 = 40;
pub const EWOODATTR: i32 = 40;

// Orc
pub const OMNTNATTR: i32 = 30;
pub const OHILLATTR: i32 = 20;
pub const OCLERATTR: i32 = 0;
pub const OCITYATTR: i32 = 50;
pub const OTOWNATTR: i32 = 25;
pub const OGOLDATTR: i32 = 20;
pub const OMINEATTR: i32 = 20;
pub const OFOREATTR: i32 = -40;
pub const OWOODATTR: i32 = -20;

// Human
pub const HMNTNATTR: i32 = -10;
pub const HHILLATTR: i32 = 0;
pub const HCLERATTR: i32 = 30;
pub const HCITYATTR: i32 = 50;
pub const HTOWNATTR: i32 = 40;
pub const HGOLDATTR: i32 = 10;
pub const HMINEATTR: i32 = 10;
pub const HFOREATTR: i32 = -20;
pub const HWOODATTR: i32 = 0;

// ============================================================
// Trade Good Value Thresholds
// ============================================================

pub const GOLDTHRESH: i64 = 10;
pub const MAXTGVAL: i32 = 100;

// ============================================================
// God Market Constants
// ============================================================

pub const GODFOOD: i64 = 8000;
pub const GODMETAL: i64 = 2000;
pub const GODJEWL: i64 = 3000;
pub const GODPRICE: i64 = 25_000;

// ============================================================
// String/Field Lengths
// ============================================================

pub const PASSLTH: usize = 7;
pub const NAMELTH: usize = 9;
pub const LEADERLTH: usize = 9;
pub const FILELTH: usize = 80;
pub const NUMCLASS: usize = 11;

// ============================================================
// Unit Type Offsets
// ============================================================

/// Offset added to base unit index for leader types
pub const UTYPE: u8 = 75;
/// Two times UTYPE — offset for monster types
pub const TWOUTYPE: u8 = 150;

/// Minimum value of a monster unit type (from C: MINMONSTER = 45+TWOUTYPE)
pub const MINMONSTER: u8 = 45 + TWOUTYPE as i32 as u8;
/// Maximum value of a monster unit type
pub const MAXMONSTER: u8 = 199;

// ============================================================
// Army/Navy Status Types
// ============================================================

/// Indicates an army that has been traded (from C: TRADED = 4)
pub const TRADED: u8 = 4;

// ============================================================
// Nation Status Types
// ============================================================

/// PEASANT REVOLT TYPE NATIONS (from C: NPC_PEASANT = 17)
pub const NPC_PEASANT: u8 = 17;

// ============================================================
// Seasons
// ============================================================

/// Spring season (from C: SPRING = 1)
pub const SPRING: u8 = 1;
/// Summer season (from C: SUMMER = 2)
pub const SUMMER: u8 = 2;
/// Fall season (from C: FALL = 3)
pub const FALL: u8 = 3;
/// Winter season (from C: WINTER = 4)
pub const WINTER: u8 = 4;

// ============================================================
// Map Dimensions
// ============================================================

/// Map X dimension - set at world creation (runtime value)
pub const MAPX: usize = 32;
/// Map Y dimension - set at world creation (runtime value)
pub const MAPY: usize = 32;

// ============================================================
// Miscellaneous
// ============================================================

pub const BIG: i64 = 500_000_000;
pub const MAXHELP: i32 = 6;
pub const BREAKJIHAD: i64 = 200_000;

/// National attributes gained from power
pub const PWR_NA: i32 = 10;
/// National attributes gained from class
pub const CLA_NA: i32 = 30;

/// Cost to break jihad / confederacy
pub const SALT: &str = "aa";
