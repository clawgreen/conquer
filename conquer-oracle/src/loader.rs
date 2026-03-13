/// Oracle JSON loader — reads C oracle JSON dumps into Rust types.
///
/// The C oracle outputs JSON in this format:
/// - world: {mapx, mapy, turn, score, gold, food, jewels, metal}
/// - nations: [{id, name, leader, active, race, mark, tgold, tfood, tciv, tmil, tsctrs, score, metals, jewels, capx, capy}]
/// - armies: [{nation, army, xloc, yloc, sold, type, stat}]
/// - sectors: [{x, y, owner, des, alt, veg, people, metal, jewels}]

use serde::Deserialize;
use conquer_core::structs::*;
use conquer_core::constants::*;
use conquer_core::enums::*;
use conquer_core::tables::*;

// ============================================================
// Oracle JSON structures (match the C oracle output exactly)
// ============================================================

#[derive(Debug, Deserialize)]
pub struct OracleSnapshot {
    pub world: Option<OracleWorld>,
    pub nations: Option<Vec<OracleNation>>,
    pub armies: Option<Vec<OracleArmy>>,
    pub sectors: Option<Vec<OracleSector>>,
    pub navies: Option<Vec<OracleNavy>>,
}

#[derive(Debug, Deserialize)]
pub struct OracleWorld {
    pub mapx: i16,
    pub mapy: i16,
    pub turn: i64,
    pub score: i64,
    pub gold: i64,
    pub food: i64,
    pub jewels: i64,
    pub metal: i64,
}

#[derive(Debug, Deserialize)]
pub struct OracleNation {
    pub id: usize,
    pub name: String,
    pub leader: String,
    pub active: u8,
    pub race: String,
    pub mark: String,
    pub tgold: i64,
    pub tfood: i64,
    pub tciv: i64,
    pub tmil: i64,
    pub tsctrs: i64,
    pub score: i64,
    pub metals: i64,
    pub jewels: i64,
    pub capx: u8,
    pub capy: u8,
    // Extended fields (v2 oracle)
    #[serde(default)]
    pub class: Option<i16>,
    #[serde(default)]
    pub maxmove: Option<u8>,
    #[serde(default)]
    pub repro: Option<i8>,
    #[serde(default)]
    pub powers: Option<i64>,
    #[serde(default)]
    pub aplus: Option<i16>,
    #[serde(default)]
    pub dplus: Option<i16>,
    #[serde(default)]
    pub spellpts: Option<i16>,
    #[serde(default)]
    pub tships: Option<i16>,
    #[serde(default)]
    pub inflation: Option<i16>,
    #[serde(default)]
    pub charity: Option<u8>,
    #[serde(default)]
    pub tax_rate: Option<u8>,
    #[serde(default)]
    pub prestige: Option<u8>,
    #[serde(default)]
    pub popularity: Option<u8>,
    #[serde(default)]
    pub power: Option<u8>,
    #[serde(default)]
    pub communications: Option<u8>,
    #[serde(default)]
    pub wealth: Option<u8>,
    #[serde(default)]
    pub eatrate: Option<u8>,
    #[serde(default)]
    pub spoilrate: Option<u8>,
    #[serde(default)]
    pub knowledge: Option<u8>,
    #[serde(default)]
    pub farm_ability: Option<u8>,
    #[serde(default)]
    pub mine_ability: Option<u8>,
    #[serde(default)]
    pub poverty: Option<u8>,
    #[serde(default)]
    pub terror: Option<u8>,
    #[serde(default)]
    pub reputation: Option<u8>,
    #[serde(default)]
    pub dstatus: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
pub struct OracleArmy {
    pub nation: usize,
    pub army: usize,
    pub xloc: u8,
    pub yloc: u8,
    pub sold: i64,
    #[serde(rename = "type")]
    pub unit_type: u8,
    pub stat: u8,
    #[serde(default)]
    pub smove: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct OracleSector {
    pub x: usize,
    pub y: usize,
    pub owner: u8,
    pub des: String,
    pub alt: u8,
    pub veg: u8,
    pub people: i64,
    pub metal: u8,
    pub jewels: u8,
    #[serde(default)]
    pub fortress: Option<u8>,
    #[serde(default)]
    pub tradegood: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct OracleNavy {
    pub nation: usize,
    pub navy: usize,
    pub xloc: u8,
    pub yloc: u8,
    pub warships: u16,
    pub merchant: u16,
    pub galleys: u16,
    pub crew: u8,
    pub people: u8,
    pub smove: u8,
}

// ============================================================
// Conversion from oracle format to Rust game types
// ============================================================

impl OracleSnapshot {
    /// Load a snapshot from a JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Convert oracle snapshot to a partial GameState.
    /// Populates world, nations (with armies), and sectors from oracle data.
    pub fn to_game_state(&self) -> GameState {
        let world_data = self.world.as_ref().expect("oracle snapshot missing world");
        let map_x = world_data.mapx as usize;
        let map_y = world_data.mapy as usize;

        let mut gs = GameState::new(map_x, map_y);

        // World
        gs.world.map_x = world_data.mapx;
        gs.world.map_y = world_data.mapy;
        gs.world.turn = world_data.turn as i16;
        gs.world.score = world_data.score;
        gs.world.world_gold = world_data.gold;
        gs.world.world_food = world_data.food;
        gs.world.world_jewels = world_data.jewels;
        gs.world.world_metal = world_data.metal;

        // Nations
        if let Some(nations) = &self.nations {
            for on in nations {
                if on.id >= NTOTAL { continue; }
                let n = &mut gs.nations[on.id];
                n.name = on.name.clone();
                n.leader = on.leader.clone();
                n.active = on.active;
                n.race = on.race.chars().next().unwrap_or('?');
                n.mark = on.mark.chars().next().unwrap_or('?');
                n.treasury_gold = on.tgold;
                n.total_food = on.tfood;
                n.total_civ = on.tciv;
                n.total_mil = on.tmil;
                n.total_sectors = on.tsctrs as i16;
                n.score = on.score;
                n.metals = on.metals;
                n.jewels = on.jewels;
                n.cap_x = on.capx;
                n.cap_y = on.capy;
                // Extended fields (v2 oracle)
                if let Some(v) = on.class { n.class = v; }
                if let Some(v) = on.maxmove { n.max_move = v; }
                if let Some(v) = on.repro { n.repro = v; }
                if let Some(v) = on.powers { n.powers = v; }
                if let Some(v) = on.aplus { n.attack_plus = v; }
                if let Some(v) = on.dplus { n.defense_plus = v; }
                if let Some(v) = on.spellpts { n.spell_points = v; }
                if let Some(v) = on.tships { n.total_ships = v; }
                if let Some(v) = on.inflation { n.inflation = v; }
                if let Some(v) = on.charity { n.charity = v; }
                if let Some(v) = on.tax_rate { n.tax_rate = v; }
                if let Some(v) = on.prestige { n.prestige = v; }
                if let Some(v) = on.popularity { n.popularity = v; }
                if let Some(v) = on.power { n.power = v; }
                if let Some(v) = on.communications { n.communications = v; }
                if let Some(v) = on.wealth { n.wealth = v; }
                if let Some(v) = on.eatrate { n.eat_rate = v; }
                if let Some(v) = on.spoilrate { n.spoil_rate = v; }
                if let Some(v) = on.knowledge { n.knowledge = v; }
                if let Some(v) = on.farm_ability { n.farm_ability = v; }
                if let Some(v) = on.mine_ability { n.mine_ability = v; }
                if let Some(v) = on.poverty { n.poverty = v; }
                if let Some(v) = on.terror { n.terror = v; }
                if let Some(v) = on.reputation { n.reputation = v; }
                if let Some(ref ds) = on.dstatus {
                    for (i, &val) in ds.iter().enumerate() {
                        if i < NTOTAL {
                            n.diplomacy[i] = val;
                        }
                    }
                }
            }
        }

        // Armies
        if let Some(armies) = &self.armies {
            for oa in armies {
                if oa.nation >= NTOTAL || oa.army >= MAXARM { continue; }
                let a = &mut gs.nations[oa.nation].armies[oa.army];
                a.x = oa.xloc;
                a.y = oa.yloc;
                a.soldiers = oa.sold;
                a.unit_type = oa.unit_type;
                a.status = oa.stat;
                if let Some(mv) = oa.smove {
                    a.movement = mv;
                }
            }
        }

        // Sectors
        if let Some(sectors) = &self.sectors {
            for os in sectors {
                if os.x >= map_x || os.y >= map_y { continue; }
                let s = &mut gs.sectors[os.x][os.y];
                // des is a char like 't', 'c', '-', etc.
                s.designation = os.des.bytes().next().unwrap_or(0);
                s.altitude = os.alt;
                s.vegetation = os.veg;
                s.owner = os.owner;
                s.people = os.people;
                s.metal = os.metal;
                s.jewels = os.jewels;
                if let Some(v) = os.fortress { s.fortress = v; }
                if let Some(v) = os.tradegood { s.trade_good = v; }
            }
        }

        // Navies
        if let Some(navies) = &self.navies {
            for on in navies {
                if on.nation >= NTOTAL { continue; }
                let navy_idx = on.navy;
                if navy_idx >= MAXNAVY { continue; }
                let nvy = &mut gs.nations[on.nation].navies[navy_idx];
                nvy.x = on.xloc;
                nvy.y = on.yloc;
                nvy.warships = on.warships;
                nvy.merchant = on.merchant;
                nvy.galleys = on.galleys;
                nvy.crew = on.crew;
                nvy.people = on.people;
                nvy.movement = on.smove;
            }
        }

        // Post-load: set army movement and max_move when oracle didn't provide them.
        // v2 oracle includes smove directly; v1 needs reconstruction.
        let has_smove = self.armies.as_ref().map_or(false, |a| a.first().map_or(false, |a| a.smove.is_some()));
        for country in 1..NTOTAL {
            let strat = NationStrategy::from_value(gs.nations[country].active);
            if !strat.map_or(false, |s| s.is_nation()) && !strat.map_or(false, |s| s.is_monster()) {
                continue;
            }

            // Fallback max_move if not provided by oracle
            if gs.nations[country].max_move == 0 {
                gs.nations[country].max_move = 12;
            }

            // Only reconstruct movement if oracle didn't provide smove
            if !has_smove {
                let max_move = gs.nations[country].max_move;
                for armynum in 0..MAXARM {
                    let a = &gs.nations[country].armies[armynum];
                    if a.soldiers <= 0 { continue; }
                    let at = a.unit_type as usize;
                    let unit_move_idx = at % (UTYPE as usize);
                    let unit_move = UNIT_MOVE.get(unit_move_idx).copied().unwrap_or(10);
                    gs.nations[country].armies[armynum].movement =
                        ((max_move as i32 * unit_move) / 10) as u8;
                }
            }
        }

        gs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_snapshot() {
        let json = r#"{
            "world": {"mapx": 32, "mapy": 32, "turn": 1, "score": 0, "gold": 100, "food": 200, "jewels": 50, "metal": 75},
            "nations": [
                {"id": 0, "name": "unowned", "leader": "god", "active": 0, "race": "-", "mark": "-", "tgold": 0, "tfood": 0, "tciv": 0, "tmil": 0, "tsctrs": 0, "score": 0, "metals": 0, "jewels": 0, "capx": 0, "capy": 0},
                {"id": 1, "name": "argos", "leader": "The_Ed", "active": 16, "race": "H", "mark": "A", "tgold": 74787, "tfood": 49691, "tciv": 26419, "tmil": 2854, "tsctrs": 14, "score": 48, "metals": 0, "jewels": 15969, "capx": 24, "capy": 7}
            ],
            "armies": [
                {"nation": 1, "army": 0, "xloc": 24, "yloc": 7, "sold": 109, "type": 3, "stat": 3}
            ],
            "sectors": [
                {"x": 0, "y": 0, "owner": 0, "des": "-", "alt": 0, "veg": 11, "people": 0, "metal": 0, "jewels": 0}
            ]
        }"#;

        let snap = OracleSnapshot::from_json(json).unwrap();
        let gs = snap.to_game_state();

        assert_eq!(gs.world.map_x, 32);
        assert_eq!(gs.world.map_y, 32);
        assert_eq!(gs.world.turn, 1);
        assert_eq!(gs.world.world_gold, 100);

        assert_eq!(gs.nations[1].name, "argos");
        assert_eq!(gs.nations[1].race, 'H');
        assert_eq!(gs.nations[1].treasury_gold, 74787);
        assert_eq!(gs.nations[1].cap_x, 24);

        assert_eq!(gs.nations[1].armies[0].soldiers, 109);
        assert_eq!(gs.nations[1].armies[0].unit_type, 3);
        assert_eq!(gs.nations[1].armies[0].status, 3);
    }
}
