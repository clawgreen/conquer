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

// ============================================================
// Oracle JSON structures (match the C oracle output exactly)
// ============================================================

#[derive(Debug, Deserialize)]
pub struct OracleSnapshot {
    pub world: Option<OracleWorld>,
    pub nations: Option<Vec<OracleNation>>,
    pub armies: Option<Vec<OracleArmy>>,
    pub sectors: Option<Vec<OracleSector>>,
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
