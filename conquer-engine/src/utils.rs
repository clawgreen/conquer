// conquer-engine/src/utils.rs — Core utility functions ported from misc.c
//
// T115-T134: is_habitable, tofood, solds_in_sector, units_in_sector,
// fort_val, avian, defaultunit, flightcost, num_powers, etc.

use conquer_core::*;
use conquer_core::powers::*;
use conquer_core::tables::*;

/// is_habitable(x, y) — returns true if sector is habitable land.
/// Matches C: altitude not WATER/PEAK, vegetation is BARREN/LT_VEG/GOOD/WOOD/FOREST.
pub fn is_habitable(sct: &Sector) -> bool {
    let alt = sct.altitude;
    let veg = sct.vegetation;

    // Must not be water or peak
    if alt == Altitude::Water as u8 || alt == Altitude::Peak as u8 {
        return false;
    }

    // Vegetation must be one of the habitable types
    matches!(
        Vegetation::from_index(veg),
        Some(Vegetation::Barren)
            | Some(Vegetation::LtVeg)
            | Some(Vegetation::Good)
            | Some(Vegetation::Wood)
            | Some(Vegetation::Forest)
    )
}

/// ONMAP(x, y) — bounds check
pub fn on_map(x: i32, y: i32, map_x: i32, map_y: i32) -> bool {
    x >= 0 && y >= 0 && x < map_x && y < map_y
}

/// tofood(sptr, country) — returns food value of a sector.
/// Matches C exactly including DERVDESG, elf bonuses, and trade good bonuses.
pub fn tofood(sct: &Sector, nation: Option<&Nation>) -> i32 {
    let veg_idx = sct.vegetation as usize;
    let veg_food = VEG_FOOD.get(veg_idx).copied().unwrap_or(0);
    let mut foodvalue = veg_food;

    if let Some(ntn) = nation {
        if foodvalue == 0 {
            // DERVDESG: dervish/destroyer can farm desert/ice
            let has_dervish = Power::has_power(ntn.powers, Power::DERVISH);
            let has_destroyer = Power::has_power(ntn.powers, Power::DESTROYER);
            let is_desert = sct.vegetation == Vegetation::Desert as u8;
            let is_ice = sct.vegetation == Vegetation::Ice as u8;
            if (has_dervish || has_destroyer) && (is_desert || is_ice) {
                return 6;
            }
            return 0;
        }

        // Elf bonuses
        if ntn.race == 'E' {
            if sct.vegetation == Vegetation::Forest as u8 {
                foodvalue += 3;
            } else if sct.vegetation == Vegetation::Barren as u8 {
                foodvalue -= 1;
            }
        }
    }

    // Trade good food bonus: trade goods between END_COMMUNICATION and END_EATRATE
    let tg = sct.trade_good;
    if tg > END_COMMUNICATION && tg <= END_EATRATE {
        foodvalue += tg_value(tg as usize) as i32;
    }

    foodvalue
}

/// solds_in_sector(x, y, nation) — count total soldiers of a nation in a sector.
pub fn solds_in_sector(nation: &Nation, x: u8, y: u8) -> i64 {
    let mut total: i64 = 0;
    for army in &nation.armies {
        if army.soldiers <= 0 {
            continue;
        }
        if army.x == x && army.y == y {
            total += army.soldiers;
        }
    }
    total
}

/// units_in_sector(x, y, nation) — count number of units (armies + navies) at location.
pub fn units_in_sector(nation: &Nation, x: u8, y: u8) -> i32 {
    let mut count = 0;
    for army in &nation.armies {
        if army.soldiers > 0 && army.x == x && army.y == y {
            count += 1;
        }
    }
    for navy in &nation.navies {
        if navy.has_ships() && navy.x == x && navy.y == y {
            count += 1;
        }
    }
    count
}

/// avian(unit_type) — check if unit type can fly.
/// Matches C: ROC, GRIFFON, SPIRIT, DJINNI, DEMON, DRAGON.
pub fn avian(typ: u8) -> bool {
    matches!(
        UnitType(typ),
        UnitType(20)    // A_ROC
        | UnitType(22)  // A_GRIFFON
        | UnitType(195) // SPIRIT (45+150)
        | UnitType(197) // DJINNI (47+150)
        | UnitType(207) // DEMON  (57+150)
        | UnitType(209) // DRAGON (59+150)
    )
}

/// defaultunit(nation) — returns the default army type for a given nation.
/// Matches C exactly.
pub fn defaultunit(nation: &Nation) -> u8 {
    if Power::has_power(nation.powers, Power::VAMPIRE) {
        return UnitType::ZOMBIE.0;
    }
    if Power::has_power(nation.powers, Power::AV_MONST) {
        if Power::has_power(nation.powers, Power::BREEDER) {
            return UnitType::OLOG.0;
        } else {
            return UnitType::URUK.0;
        }
    }
    if Power::has_power(nation.powers, Power::ARCHER) {
        return UnitType::ARCHER.0;
    }
    if Power::has_power(nation.powers, Power::MI_MONST) {
        return UnitType::ORC.0;
    }
    if nation.active == NationStrategy::NpcNomad as u8 {
        return UnitType::LT_CAV.0;
    }
    UnitType::INFANTRY.0
}

/// fort_val(sector) — compute the fortification value of a sector.
/// Matches C exactly, including ARCHITECT power doubling.
pub fn fort_val(sct: &Sector, owner_powers: i64) -> i32 {
    let des = sct.designation;
    let fort = sct.fortress as i32;
    let has_architect = Power::has_power(owner_powers, Power::ARCHITECT);

    if des == Designation::Stockade as u8 {
        return DEF_BASE;
    }
    if des == Designation::Fort as u8 {
        if has_architect {
            return DEF_BASE + 2 * FORTSTR * fort;
        } else {
            return DEF_BASE + FORTSTR * fort;
        }
    }
    if des == Designation::Town as u8 {
        if has_architect {
            return DEF_BASE + 2 * TOWNSTR * fort;
        } else {
            return DEF_BASE + TOWNSTR * fort;
        }
    }
    if des == Designation::Capitol as u8 || des == Designation::City as u8 {
        if has_architect {
            return 2 * DEF_BASE + 2 * CITYSTR * fort;
        } else {
            return 2 * DEF_BASE + CITYSTR * fort;
        }
    }
    0
}

/// flightcost(sector) — cost of flying over a sector.
/// Matches C misc.c flightcost(). Returns -1 if unenterable.
pub fn flightcost(sct: &Sector) -> i32 {
    let alt_idx = sct.altitude as usize;
    let veg_idx = sct.vegetation as usize;

    let f_ele = F_ELE_COST.as_bytes();
    let f_veg = F_VEG_COST.as_bytes();

    // Find altitude cost
    let ele_cost = if alt_idx < f_ele.len() - 1 {
        // '/' means impassable
        let c = f_ele[alt_idx];
        if c == b'/' { return -1; }
        (c as i32) - ('0' as i32)
    } else {
        return -1;
    };

    // Find vegetation cost
    let veg_cost = if veg_idx < f_veg.len() - 1 {
        let c = f_veg[veg_idx];
        if c == b'/' { return -1; }
        (c as i32) - ('0' as i32)
    } else {
        return -1;
    };

    if ele_cost == -1 || veg_cost == -1 {
        -1
    } else {
        ele_cost + veg_cost
    }
}

/// num_powers(nation_powers, power_type) — count the number of powers of a given type.
/// Matches C exactly. type is M_MIL=1, M_CIV=2, M_MGK=3, M_ALL=5.
pub fn num_powers(nation_powers: i64, power_type: u8) -> i32 {
    let (start, count) = match power_type {
        M_MIL => (S_MIL, E_MIL),
        M_CIV => (S_CIV, E_CIV),
        M_MGK => (S_MGK, E_MGK),
        M_ALL => (S_MIL, E_MIL + E_CIV + E_MGK), // BUG-COMPAT: C loops S_MIL to E_MGK 
        _ => return 0,
    };

    let mut count_magic = 0;
    for i in start..(start + count) {
        if let Some(power) = Power::from_index(i) {
            if Power::has_power(nation_powers, power) {
                count_magic += 1;
            }
        }
    }
    count_magic
}

/// getmgkcost(power_type, nation) — compute cost of next magic power.
/// Matches C exactly: base * 2^(npowers/2).
pub fn getmgkcost(power_type: u8, nation: &Nation) -> i64 {
    let race = nation.race;
    let mut base: i64 = BASEMAGIC;
    let npowers: i32;

    match power_type {
        M_MGK => {
            if race == 'D' { base = DWFMAGIC; }
            else if race == 'H' { base = HUMMAGIC; }
            else if race == 'O' { base = ORCMAGIC; }
            let raw = num_powers(nation.powers, M_CIV)
                + num_powers(nation.powers, M_MIL)
                + 1
                + 2 * num_powers(nation.powers, M_MGK);
            npowers = raw / 2;
        }
        M_CIV => {
            if race == 'D' { base = DWFCIVIL; }
            else if race == 'H' { base = HUMCIVIL; }
            else if race == 'O' { base = ORCCIVIL; }
            let raw = num_powers(nation.powers, M_MGK)
                + num_powers(nation.powers, M_MIL)
                + 1
                + 2 * num_powers(nation.powers, M_CIV);
            npowers = raw / 2;
        }
        M_MIL => {
            if race == 'D' { base = DWFMILIT; }
            else if race == 'O' { base = ORCMILIT; }
            let raw = num_powers(nation.powers, M_CIV)
                + num_powers(nation.powers, M_MGK)
                + 1
                + 2 * num_powers(nation.powers, M_MIL);
            npowers = raw / 2;
        }
        _ => return -1,
    }

    let mut cost = base;
    for _ in 1..npowers {
        cost <<= 1;
        if cost > BIG {
            return BIG / 2;
        }
    }
    cost
}

/// ISCITY(designation) — check if designation is city-like (town/city/capitol/fort).
pub fn is_city(des: u8) -> bool {
    des == Designation::City as u8
        || des == Designation::Capitol as u8
        || des == Designation::Fort as u8
        || des == Designation::Town as u8
}

/// getleader(class) — get the leader unit type for a nation class.
/// Matches C exactly. Returns the unit type value.
pub fn getleader(class: i16) -> u8 {
    match class {
        0 | 1 | 6 => UnitType::L_BARON.0,  // NPC, King, Trader
        2 => UnitType::L_PRINCE.0,          // Emperor
        3 => UnitType::L_MAGI.0,            // Wizard
        4 => UnitType::L_BISHOP.0,          // Priest
        5 => UnitType::L_CAPTAIN.0,         // Pirate
        7 => UnitType::L_LORD.0,            // Warlord
        8 => UnitType::L_DEVIL.0,           // Demon
        9 => UnitType::L_WYRM.0,            // Dragon
        10 => UnitType::L_NAZGUL.0,         // Shadow
        _ => UnitType::L_BARON.0,
    }
}

/// getmetal(sector, rng) — assign metal trade good to sector.
/// Matches C exactly.
pub fn getmetal(sct: &mut Sector, rng: &mut conquer_core::rng::ConquerRng) {
    if sct.trade_good != TradeGood::None as u8 && sct.trade_good != 0 {
        return;
    }
    let randval = rng.rand() % 100;
    if randval < 20 {
        sct.trade_good = TradeGood::Copper as u8;
        sct.metal = (rng.rand() % 2 + 1) as u8;
    } else if randval < 30 {
        sct.trade_good = TradeGood::Lead as u8;
        sct.metal = (rng.rand() % 4 + 1) as u8;
    } else if randval < 40 {
        sct.trade_good = TradeGood::Tin as u8;
        sct.metal = (rng.rand() % 4 + 2) as u8;
    } else if randval < 55 {
        sct.trade_good = TradeGood::Bronze as u8;
        sct.metal = (rng.rand() % 4 + 2) as u8;
    } else if randval < 80 {
        sct.trade_good = TradeGood::Iron as u8;
        sct.metal = (rng.rand() % 7 + 2) as u8;
    } else if randval < 95 {
        sct.trade_good = TradeGood::Steel as u8;
        sct.metal = (rng.rand() % 8 + 3) as u8;
    } else if randval < 99 {
        sct.trade_good = TradeGood::Mithral as u8;
        sct.metal = (rng.rand() % 11 + 5) as u8;
    } else {
        sct.trade_good = TradeGood::Adamantine as u8;
        sct.metal = (rng.rand() % 13 + 8) as u8;
    }
}

/// getjewel(sector, rng) — assign jewel trade good to sector.
/// Matches C exactly.
pub fn getjewel(sct: &mut Sector, rng: &mut conquer_core::rng::ConquerRng) {
    if sct.trade_good != TradeGood::None as u8 && sct.trade_good != 0 {
        return;
    }
    let randval = rng.rand() % 100;
    if randval < 20 {
        sct.trade_good = TradeGood::Spice as u8;
        sct.jewels = (rng.rand() % 2 + 1) as u8;
    } else if randval < 40 {
        sct.trade_good = TradeGood::Silver as u8;
        sct.jewels = (rng.rand() % 3 + 1) as u8;
    } else if randval < 48 {
        sct.trade_good = TradeGood::Pearls as u8;
        sct.jewels = (rng.rand() % 3 + 1) as u8;
    } else if randval < 56 {
        sct.trade_good = TradeGood::Dye as u8;
        sct.jewels = (rng.rand() % 5 + 1) as u8;
    } else if randval < 64 {
        sct.trade_good = TradeGood::Silk as u8;
        sct.jewels = (rng.rand() % 5 + 1) as u8;
    } else if randval < 84 {
        sct.trade_good = TradeGood::Gold as u8;
        sct.jewels = (rng.rand() % 6 + 1) as u8;
    } else if randval < 91 {
        sct.trade_good = TradeGood::Rubys as u8;
        sct.jewels = (rng.rand() % 6 + 1) as u8;
    } else if randval < 96 {
        sct.trade_good = TradeGood::Ivory as u8;
        sct.jewels = (rng.rand() % 7 + 2) as u8;
    } else if randval < 99 {
        sct.trade_good = TradeGood::Diamonds as u8;
        sct.jewels = (rng.rand() % 11 + 2) as u8;
    } else {
        sct.trade_good = TradeGood::Platinum as u8;
        sct.jewels = (rng.rand() % 17 + 4) as u8;
    }
}

/// tg_ok(nation, sector) — check if a trade good can be seen/used by the nation.
/// Matches C exactly.
pub fn tg_ok(nation: &Nation, sct: &Sector) -> bool {
    let tg = sct.trade_good;
    match tg {
        x if x == TradeGood::Lead as u8 => { if nation.mine_ability < 8 { return false; } }
        x if x == TradeGood::Tin as u8 => { if nation.mine_ability < 11 { return false; } }
        x if x == TradeGood::Bronze as u8 => { if nation.mine_ability < 15 { return false; } }
        x if x == TradeGood::Iron as u8 => { if nation.mine_ability < 25 { return false; } }
        x if x == TradeGood::Steel as u8 => { if nation.mine_ability < 30 { return false; } }
        x if x == TradeGood::Mithral as u8 => { if nation.mine_ability < 30 { return false; } }
        x if x == TradeGood::Adamantine as u8 => { if nation.mine_ability < 40 { return false; } }
        x if x == TradeGood::Spice as u8 => {}
        x if x == TradeGood::Silver as u8 => {}
        x if x == TradeGood::Pearls as u8 => {}
        x if x == TradeGood::Dye as u8 => { if nation.wealth < 5 { return false; } }
        x if x == TradeGood::Silk as u8 => { if nation.wealth < 5 { return false; } }
        x if x == TradeGood::Gold as u8 => { if nation.wealth < 8 { return false; } }
        x if x == TradeGood::Rubys as u8 => { if nation.wealth < 8 { return false; } }
        x if x == TradeGood::Ivory as u8 => { if nation.wealth < 15 { return false; } }
        x if x == TradeGood::Diamonds as u8 => { if nation.wealth < 20 { return false; } }
        x if x == TradeGood::Platinum as u8 => { if nation.wealth < 25 { return false; } }
        _ => {}
    }

    tofood(sct, Some(nation)) >= DESFOOD
}

/// attract(x, y, race, sct, nation) — how attractive is sector to civilians.
/// Matches C update.c attract() exactly.
pub fn attract(
    _x: i32,
    _y: i32,
    sct: &Sector,
    nation: &Nation,
    move_cost: i16,
) -> i32 {
    let des = sct.designation;
    let race = nation.race;
    let mut attr = 0i32;

    // Trade good bonus
    if sct.trade_good != TradeGood::None as u8 {
        let tg_stype_bytes = TG_SECTOR_TYPE.as_bytes();
        let tg_idx = sct.trade_good as usize;
        if tg_idx < tg_stype_bytes.len() {
            // Compare trade good's preferred sector type with actual designation
            // The C code compares the char value directly
            let des_chars = DES_CHARS.as_bytes();
            let pref_char = tg_stype_bytes[tg_idx];
            if des < des_chars.len() as u8 && pref_char == des_chars[des as usize] {
                if des != Designation::Mine as u8 && des != Designation::GoldMine as u8 {
                    attr += (tg_value(tg_idx) as i32) * TGATTR;
                }
            }
        }
    }

    if des == Designation::GoldMine as u8 {
        if sct.jewels >= 6 {
            attr += GOLDATTR * sct.jewels as i32 * 2;
        } else {
            attr += GOLDATTR * sct.jewels as i32;
        }
    } else if des == Designation::Farm as u8 {
        // BUG-COMPAT: C uses ntn[sptr->owner] which may differ from `nation`
        let need_food = nation.total_food * 250 <= (nation.eat_rate as i64) * (nation.total_civ * 11);
        if need_food {
            attr += 50 * FARMATTR;
        } else {
            attr += tofood(sct, Some(nation)) * FARMATTR;
        }
    } else if des == Designation::City as u8 {
        attr += CITYATTR;
    } else if des == Designation::Capitol as u8 {
        attr += CITYATTR;
    } else if des == Designation::Town as u8 {
        attr += TOWNATTR;
    } else if des == Designation::Mine as u8 {
        if sct.metal > 6 {
            attr += MINEATTR * sct.metal as i32 * 2;
        } else {
            attr += MINEATTR * sct.metal as i32;
        }
    } else if des != Designation::Road as u8
        && des != Designation::NoDesig as u8
        && des != Designation::Devastated as u8
        && is_habitable(sct)
    {
        attr += OTHRATTR;
    }

    // Race-specific bonuses
    match race {
        'D' => {
            if des == Designation::GoldMine as u8 && sct.jewels > 3 {
                attr += DGOLDATTR;
            } else if des == Designation::Mine as u8 && sct.metal > 3 {
                attr += DMINEATTR;
            } else if des == Designation::Town as u8 {
                attr += DTOWNATTR;
            } else if des == Designation::City as u8 || des == Designation::Capitol as u8 {
                attr += DCITYATTR;
            }
            if sct.vegetation == Vegetation::Wood as u8 { attr += DWOODATTR; }
            else if sct.vegetation == Vegetation::Forest as u8 { attr += DFOREATTR; }
            if sct.altitude == Altitude::Mountain as u8 { attr += DMNTNATTR; }
            else if sct.altitude == Altitude::Hill as u8 { attr += DHILLATTR; }
            else if sct.altitude == Altitude::Clear as u8 { attr += DCLERATTR; }
            else { attr = 0; }
        }
        'E' => {
            if des == Designation::GoldMine as u8 && sct.jewels > 3 {
                attr += EGOLDATTR;
            } else if des == Designation::Mine as u8 && sct.metal > 3 {
                attr += EMINEATTR;
            } else if des == Designation::Town as u8
                || des == Designation::City as u8
                || des == Designation::Capitol as u8
            {
                attr += ECITYATTR;
            }
            if sct.vegetation == Vegetation::Wood as u8 { attr += EWOODATTR; }
            else if sct.vegetation == Vegetation::Forest as u8 { attr += EFOREATTR; }
            if sct.altitude == Altitude::Mountain as u8 { attr += EMNTNATTR; }
            else if sct.altitude == Altitude::Hill as u8 { attr += EHILLATTR; }
            else if sct.altitude == Altitude::Clear as u8 { attr += ECLERATTR; }
            else { attr = 0; }
        }
        'H' => {
            if des == Designation::GoldMine as u8 && sct.jewels > 3 {
                attr += HGOLDATTR;
            } else if des == Designation::Mine as u8 && sct.metal > 3 {
                attr += HMINEATTR;
            } else if des == Designation::Town as u8
                || des == Designation::City as u8
                || des == Designation::Capitol as u8
            {
                attr += HCITYATTR;
            }
            if sct.vegetation == Vegetation::Wood as u8 { attr += HWOODATTR; }
            else if sct.vegetation == Vegetation::Forest as u8 { attr += HFOREATTR; }
            if sct.altitude == Altitude::Mountain as u8 { attr += HMNTNATTR; }
            else if sct.altitude == Altitude::Hill as u8 { attr += HHILLATTR; }
            else if sct.altitude == Altitude::Clear as u8 { attr += HCLERATTR; }
            else { attr = 0; }
        }
        'O' => {
            if des == Designation::GoldMine as u8 && sct.jewels > 3 {
                attr += OGOLDATTR;
            } else if des == Designation::Mine as u8 && sct.metal > 3 {
                attr += OMINEATTR;
            } else if des == Designation::Town as u8 {
                attr += OTOWNATTR;
            } else if des == Designation::City as u8 || des == Designation::Capitol as u8 {
                attr += OCITYATTR;
            }
            if sct.vegetation == Vegetation::Wood as u8 { attr += OWOODATTR; }
            else if sct.vegetation == Vegetation::Forest as u8 { attr += OFOREATTR; }
            if sct.altitude == Altitude::Mountain as u8 { attr += OMNTNATTR; }
            else if sct.altitude == Altitude::Hill as u8 { attr += OHILLATTR; }
            else if sct.altitude == Altitude::Clear as u8 { attr += OCLERATTR; }
            else { attr = 0; }
        }
        _ => {}
    }

    if des == Designation::Devastated as u8 || attr < 0 || move_cost < 0 {
        attr = 0;
    }

    attr
}

/// markok(mark) — check if a character can be used as a nation mark.
/// Simplified version (no curses error output).
pub fn markok(mark: char, nations: &[Nation]) -> bool {
    if !mark.is_ascii_graphic() || mark.is_ascii_whitespace() {
        return false;
    }

    // Check altitude chars
    for &c in ELE_CHARS.as_bytes() {
        if c == b'0' { break; }
        if mark as u8 == c { return false; }
    }

    // Check vegetation chars
    for &c in VEG_CHARS.as_bytes() {
        if c == b'0' { break; }
        if mark as u8 == c { return false; }
    }

    // Check existing nation marks
    for (i, ntn) in nations.iter().enumerate() {
        if i == 0 { continue; }
        if NationStrategy::from_value(ntn.active).map_or(false, |s| s.is_active()) && ntn.mark == mark {
            return false;
        }
    }

    if mark == '*' { return false; }
    if !mark.is_ascii_alphabetic() { return false; }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_habitable() {
        let mut s = Sector::default();
        // Water = not habitable
        s.altitude = Altitude::Water as u8;
        s.vegetation = Vegetation::Good as u8;
        assert!(!is_habitable(&s));

        // Peak = not habitable
        s.altitude = Altitude::Peak as u8;
        assert!(!is_habitable(&s));

        // Clear + Good = habitable
        s.altitude = Altitude::Clear as u8;
        s.vegetation = Vegetation::Good as u8;
        assert!(is_habitable(&s));

        // Mountain + Forest = habitable
        s.altitude = Altitude::Mountain as u8;
        s.vegetation = Vegetation::Forest as u8;
        assert!(is_habitable(&s));

        // Hill + Desert = NOT habitable
        s.altitude = Altitude::Hill as u8;
        s.vegetation = Vegetation::Desert as u8;
        assert!(!is_habitable(&s));

        // Hill + Jungle = NOT habitable
        s.vegetation = Vegetation::Jungle as u8;
        assert!(!is_habitable(&s));

        // Hill + Barren = habitable
        s.vegetation = Vegetation::Barren as u8;
        assert!(is_habitable(&s));
    }

    #[test]
    fn test_tofood_basic() {
        let mut s = Sector::default();
        s.vegetation = Vegetation::Good as u8;
        assert_eq!(tofood(&s, None), 9);

        s.vegetation = Vegetation::Forest as u8;
        assert_eq!(tofood(&s, None), 4);

        // Volcano has 0 food
        s.vegetation = Vegetation::Volcano as u8;
        assert_eq!(tofood(&s, None), 0);
    }

    #[test]
    fn test_avian() {
        assert!(avian(UnitType::ROC.0));
        assert!(avian(UnitType::GRIFFON.0));
        assert!(avian(UnitType::DRAGON.0));
        assert!(!avian(UnitType::INFANTRY.0));
        assert!(!avian(UnitType::KNIGHT.0));
    }

    #[test]
    fn test_fort_val_basic() {
        let mut s = Sector::default();
        s.fortress = 5;

        // Stockade: DEF_BASE only
        s.designation = Designation::Stockade as u8;
        assert_eq!(fort_val(&s, 0), DEF_BASE);

        // Fort: DEF_BASE + FORTSTR * fortress
        s.designation = Designation::Fort as u8;
        assert_eq!(fort_val(&s, 0), DEF_BASE + FORTSTR * 5);

        // City: 2*DEF_BASE + CITYSTR * fortress
        s.designation = Designation::City as u8;
        assert_eq!(fort_val(&s, 0), 2 * DEF_BASE + CITYSTR * 5);
    }

    #[test]
    fn test_is_city() {
        assert!(is_city(Designation::City as u8));
        assert!(is_city(Designation::Capitol as u8));
        assert!(is_city(Designation::Fort as u8));
        assert!(is_city(Designation::Town as u8));
        assert!(!is_city(Designation::Mine as u8));
        assert!(!is_city(Designation::Farm as u8));
    }
}
