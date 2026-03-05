#![allow(unused_variables, unused_imports, unused_assignments)]
// conquer-engine/src/navy.rs — Naval system ported from navy.c & update.c
//
// T246-T260: fleet management (add/sub ships), fleet hold/speed calculations,
// loading/unloading, storms, naval movement, civilian attrition.

use conquer_core::*;
use conquer_core::powers::Power;
use conquer_core::rng::ConquerRng;

/// Add warships to a fleet. Matches C addwships() exactly.
/// Returns true on success, false if invalid or overflow.
pub fn add_warships(nvy: &mut Navy, size: NavalSize, count: i16) -> bool {
    let current = NavalSize::ships(nvy.warships, size);
    let new_count = current + count;
    if new_count < 0 || new_count > N_MASK as i16 {
        return false;
    }
    nvy.warships = NavalSize::set_ships(nvy.warships, size, new_count as u16);
    true
}

/// Add merchant ships. Matches C addmships() exactly.
pub fn add_merchants(nvy: &mut Navy, size: NavalSize, count: i16) -> bool {
    let current = NavalSize::ships(nvy.merchant, size);
    let new_count = current + count;
    if new_count < 0 || new_count > N_MASK as i16 {
        return false;
    }
    nvy.merchant = NavalSize::set_ships(nvy.merchant, size, new_count as u16);
    true
}

/// Add galleys. Matches C addgships() exactly.
pub fn add_galleys(nvy: &mut Navy, size: NavalSize, count: i16) -> bool {
    let current = NavalSize::ships(nvy.galleys, size);
    let new_count = current + count;
    if new_count < 0 || new_count > N_MASK as i16 {
        return false;
    }
    nvy.galleys = NavalSize::set_ships(nvy.galleys, size, new_count as u16);
    true
}

/// Remove warships. Matches C subwships() exactly.
pub fn sub_warships(nvy: &mut Navy, size: NavalSize, count: i16) {
    let current = NavalSize::ships(nvy.warships, size);
    let new_count = current - count;
    if new_count < 0 {
        return;
    }
    nvy.warships = NavalSize::set_ships(nvy.warships, size, new_count as u16);
}

/// Remove merchant ships. Matches C submships() exactly.
pub fn sub_merchants(nvy: &mut Navy, size: NavalSize, count: i16) {
    let current = NavalSize::ships(nvy.merchant, size);
    let new_count = current - count;
    if new_count < 0 {
        return;
    }
    nvy.merchant = NavalSize::set_ships(nvy.merchant, size, new_count as u16);
}

/// Remove galleys. Matches C subgships() exactly.
pub fn sub_galleys(nvy: &mut Navy, size: NavalSize, count: i16) {
    let current = NavalSize::ships(nvy.galleys, size);
    let new_count = current - count;
    if new_count < 0 {
        return;
    }
    nvy.galleys = NavalSize::set_ships(nvy.galleys, size, new_count as u16);
}

/// Total ships in fleet. Matches C fltships() exactly.
pub fn fleet_ships(nvy: &Navy) -> i32 {
    let mut total = 0i32;
    for i in 0..=2u8 {
        let size = match i {
            0 => NavalSize::Light,
            1 => NavalSize::Medium,
            _ => NavalSize::Heavy,
        };
        total += NavalSize::ships(nvy.warships, size) as i32;
        total += NavalSize::ships(nvy.merchant, size) as i32;
        total += NavalSize::ships(nvy.galleys, size) as i32;
    }
    total
}

/// Fleet speed (slowest ship). Matches C fltspeed() exactly.
pub fn fleet_speed(nvy: &Navy) -> u16 {
    let mut hold = 99i32;

    for i in 0..=2u8 {
        let size = match i {
            0 => NavalSize::Light,
            1 => NavalSize::Medium,
            _ => NavalSize::Heavy,
        };
        if hold > N_WSPD && NavalSize::ships(nvy.warships, size) > 0 {
            hold = N_WSPD + (2 - i as i32) * N_SIZESPD;
        }
    }
    for i in 0..=2u8 {
        let size = match i {
            0 => NavalSize::Light,
            1 => NavalSize::Medium,
            _ => NavalSize::Heavy,
        };
        if hold > N_MSPD && NavalSize::ships(nvy.merchant, size) > 0 {
            hold = N_MSPD + (2 - i as i32) * N_SIZESPD;
        }
    }
    for i in 0..=2u8 {
        let size = match i {
            0 => NavalSize::Light,
            1 => NavalSize::Medium,
            _ => NavalSize::Heavy,
        };
        if hold > N_GSPD && NavalSize::ships(nvy.galleys, size) > 0 {
            hold = N_GSPD + (2 - i as i32) * N_SIZESPD;
        }
    }

    if hold == 99 {
        N_NOSPD as u16
    } else {
        hold as u16
    }
}

/// Fleet total hold. Matches C flthold() exactly.
pub fn fleet_hold(nvy: &Navy) -> i32 {
    let mut hold = 0i32;
    for i in 0..=2u8 {
        let size = match i {
            0 => NavalSize::Light,
            1 => NavalSize::Medium,
            _ => NavalSize::Heavy,
        };
        hold += (i as i32 + 1) * NavalSize::ships(nvy.warships, size) as i32;
        hold += (i as i32 + 1) * NavalSize::ships(nvy.merchant, size) as i32;
        hold += (i as i32 + 1) * NavalSize::ships(nvy.galleys, size) as i32;
    }
    hold
}

/// Warship hold only. Matches C fltwhold() exactly.
pub fn fleet_warship_hold(nvy: &Navy) -> i32 {
    let mut hold = 0i32;
    for i in 0..=2u8 {
        let size = match i {
            0 => NavalSize::Light,
            1 => NavalSize::Medium,
            _ => NavalSize::Heavy,
        };
        hold += (i as i32 + 1) * NavalSize::ships(nvy.warships, size) as i32;
    }
    hold
}

/// Galley hold only. Matches C fltghold() exactly.
pub fn fleet_galley_hold(nvy: &Navy) -> i32 {
    let mut hold = 0i32;
    for i in 0..=2u8 {
        let size = match i {
            0 => NavalSize::Light,
            1 => NavalSize::Medium,
            _ => NavalSize::Heavy,
        };
        hold += (i as i32 + 1) * NavalSize::ships(nvy.galleys, size) as i32;
    }
    hold
}

/// Merchant hold only. Matches C fltmhold() exactly.
pub fn fleet_merchant_hold(nvy: &Navy) -> i32 {
    let mut hold = 0i32;
    for i in 0..=2u8 {
        let size = match i {
            0 => NavalSize::Light,
            1 => NavalSize::Medium,
            _ => NavalSize::Heavy,
        };
        hold += (i as i32 + 1) * NavalSize::ships(nvy.merchant, size) as i32;
    }
    hold
}

/// Check if army can be loaded onto a ship.
/// Matches C loadstat() exactly.
pub fn can_load_army(status: u8) -> bool {
    let s = ArmyStatus::from_value(status);
    !matches!(
        s,
        ArmyStatus::Traded
            | ArmyStatus::General
            | ArmyStatus::Militia
            | ArmyStatus::Garrison
            | ArmyStatus::OnBoard
    )
}

/// Storm damage to a fleet. Matches C storms logic from update.c.
pub fn storm_damage(
    nvy: &mut Navy,
    rng: &mut ConquerRng,
) -> bool {
    if rng.rand() % 100 >= PSTORM {
        return false;
    }

    // Kill some ships
    for i in 0..=2u8 {
        let size = match i {
            0 => NavalSize::Light,
            1 => NavalSize::Medium,
            _ => NavalSize::Heavy,
        };

        // Warships: 20% chance to lose one
        if NavalSize::ships(nvy.warships, size) > 0 && rng.rand() % 5 == 0 {
            sub_warships(nvy, size, 1);
        }
        // Merchants: 25% chance
        if NavalSize::ships(nvy.merchant, size) > 0 && rng.rand() % 4 == 0 {
            sub_merchants(nvy, size, 1);
        }
        // Galleys: 33% chance (weaker)
        if NavalSize::ships(nvy.galleys, size) > 0 && rng.rand() % 3 == 0 {
            sub_galleys(nvy, size, 1);
        }
    }

    // People loss
    if nvy.people > 0 {
        nvy.people = nvy.people * 3 / 4;
    }

    true
}

/// Civilian attrition during long trips.
/// Matches C move.c mvused calculation exactly.
pub fn civilian_attrition(
    nvy: &mut Navy,
    move_used: i32,
    has_sailor: bool,
    rng: &mut ConquerRng,
) {
    let mut mvused = move_used;
    if mvused > LONGTRIP {
        mvused = LONGTRIP;
    }
    if has_sailor {
        mvused /= 2;
    }
    if mvused != 0 {
        mvused = rng.rand() % mvused;
    }
    nvy.people = (nvy.people as i32 * (LONGTRIP - mvused) / LONGTRIP) as u8;
}

/// Load an army onto a fleet. Returns true on success.
pub fn load_army(
    nation: &mut Nation,
    army_idx: usize,
    navy_idx: usize,
    sct: &Sector,
    nation_idx: usize,
) -> Result<(), &'static str> {
    let nvy = &nation.navies[navy_idx];
    if nvy.army_num < MAXARM as u8 {
        return Err("Fleet already carrying an army");
    }

    let army = &nation.armies[army_idx];
    if army.soldiers <= 0 {
        return Err("No soldiers");
    }
    if !can_load_army(army.status) {
        return Err("Invalid army status for loading");
    }
    if army.x != nvy.x || army.y != nvy.y {
        return Err("Army not in same sector");
    }

    let ghold = fleet_galley_hold(nvy) as i64 * SHIPHOLD;
    if army.soldiers > ghold
        && !UnitType(army.unit_type).is_leader()
        && !UnitType(army.unit_type).is_monster()
    {
        return Err("Army too large for fleet");
    }

    nation.armies[army_idx].status = ArmyStatus::OnBoard.to_value();
    nation.armies[army_idx].movement = 0;
    nation.navies[navy_idx].army_num = army_idx as u8;

    // City/harbor cost
    if (sct.designation == Designation::City as u8
        || sct.designation == Designation::Capitol as u8)
        && sct.owner as usize == nation_idx
    {
        if nation.navies[navy_idx].movement >= N_CITYCOST {
            nation.navies[navy_idx].movement -= N_CITYCOST;
        } else {
            nation.navies[navy_idx].movement = 0;
        }
    } else {
        nation.navies[navy_idx].movement = 0;
    }

    Ok(())
}

/// Unload an army from a fleet. Returns true on success.
pub fn unload_army(
    nation: &mut Nation,
    navy_idx: usize,
    sct: &Sector,
    nation_idx: usize,
) -> Result<(), &'static str> {
    let army_idx = nation.navies[navy_idx].army_num as usize;
    if army_idx >= MAXARM {
        return Err("No army on board");
    }

    let atype = nation.armies[army_idx].unit_type;

    // Check disembarkation rules
    if sct.owner == 0
        && atype != UnitType::MARINES.0
        && atype != UnitType::SAILOR.0
    {
        return Err("Only sailors or marines may disembark in unowned land");
    }
    if sct.owner as usize != nation_idx
        && sct.owner != 0
        && atype != UnitType::MARINES.0
    {
        return Err("Only marines may disembark in someone else's land");
    }

    nation.armies[army_idx].status = ArmyStatus::Defend.to_value();
    nation.navies[navy_idx].army_num = MAXARM as u8;

    // Movement cost
    if (sct.designation == Designation::City as u8
        || sct.designation == Designation::Capitol as u8)
        && sct.owner as usize == nation_idx
        && nation.navies[navy_idx].movement >= N_CITYCOST
    {
        nation.navies[navy_idx].movement -= N_CITYCOST;
    } else {
        nation.navies[navy_idx].movement = 0;
    }

    Ok(())
}

/// Load people onto a fleet. Returns actual amount loaded.
pub fn load_people(
    nation: &mut Nation,
    navy_idx: usize,
    sct: &mut Sector,
    amount: i64,
    nation_idx: usize,
) -> Result<i64, &'static str> {
    if sct.owner as usize != nation_idx {
        return Err("The people refuse to board");
    }

    let mhold = fleet_merchant_hold(&nation.navies[navy_idx]);
    let capacity = mhold as i64 * (SHIPHOLD - nation.navies[navy_idx].people as i64);

    if amount > capacity {
        return Err("Not enough room on fleet");
    }
    if sct.people < amount {
        return Err("Not enough people in sector");
    }
    if amount <= 0 {
        return Err("Invalid amount");
    }

    sct.people -= amount;
    nation.navies[navy_idx].people += (amount / mhold as i64) as u8;

    // Movement cost
    if (sct.designation == Designation::City as u8
        || sct.designation == Designation::Capitol as u8)
        && sct.owner as usize == nation_idx
        && nation.navies[navy_idx].movement >= N_CITYCOST
    {
        nation.navies[navy_idx].movement -= N_CITYCOST;
    } else {
        nation.navies[navy_idx].movement = 0;
    }

    Ok(amount)
}

/// Unload people from a fleet.
pub fn unload_people(
    nation: &mut Nation,
    navy_idx: usize,
    sct: &mut Sector,
    amount: i64,
) -> Result<i64, &'static str> {
    let mhold = fleet_merchant_hold(&nation.navies[navy_idx]);
    let on_board = nation.navies[navy_idx].people as i64 * mhold as i64;

    if amount > on_board {
        return Err("Not that many on board");
    }
    if amount <= 0 {
        return Err("Invalid amount");
    }

    sct.people += amount;
    nation.navies[navy_idx].people =
        ((on_board - amount) / mhold as i64) as u8;

    Ok(amount)
}

/// NPC fleet update — move fleets and handle maintenance.
/// Matches the navy-related portion of nationrun() in npc.c.
pub fn npc_fleet_update(
    state: &mut GameState,
    nation_idx: usize,
    rng: &mut ConquerRng,
) {
    // Set fleet speeds
    for nvynum in 0..MAXNAVY {
        let nvy = &state.nations[nation_idx].navies[nvynum];
        if fleet_ships(nvy) == 0 {
            continue;
        }
        let speed = fleet_speed(nvy);
        state.nations[nation_idx].navies[nvynum].movement = speed as u8;
    }

    // Storm check
    for nvynum in 0..MAXNAVY {
        if fleet_ships(&state.nations[nation_idx].navies[nvynum]) == 0 {
            continue;
        }
        let nvy_x = state.nations[nation_idx].navies[nvynum].x as usize;
        let nvy_y = state.nations[nation_idx].navies[nvynum].y as usize;
        if state.sectors[nvy_x][nvy_y].altitude == Altitude::Water as u8 {
            storm_damage(&mut state.nations[nation_idx].navies[nvynum], rng);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_remove_warships() {
        let mut nvy = Navy::default();
        assert!(add_warships(&mut nvy, NavalSize::Light, 3));
        assert_eq!(NavalSize::ships(nvy.warships, NavalSize::Light), 3);
        sub_warships(&mut nvy, NavalSize::Light, 1);
        assert_eq!(NavalSize::ships(nvy.warships, NavalSize::Light), 2);
    }

    #[test]
    fn test_add_overflow() {
        let mut nvy = Navy::default();
        assert!(!add_warships(&mut nvy, NavalSize::Light, 32)); // N_MASK = 31
    }

    #[test]
    fn test_fleet_ships() {
        let mut nvy = Navy::default();
        add_warships(&mut nvy, NavalSize::Light, 3);
        add_merchants(&mut nvy, NavalSize::Medium, 2);
        add_galleys(&mut nvy, NavalSize::Heavy, 1);
        assert_eq!(fleet_ships(&nvy), 6);
    }

    #[test]
    fn test_fleet_speed() {
        let mut nvy = Navy::default();
        add_warships(&mut nvy, NavalSize::Light, 3);
        // Light warships: speed = N_WSPD + 2*N_SIZESPD = 20+6 = 26
        assert_eq!(fleet_speed(&nvy), 26);

        add_merchants(&mut nvy, NavalSize::Heavy, 1);
        // Heavy merchants: speed = N_MSPD + 0*N_SIZESPD = 15
        assert_eq!(fleet_speed(&nvy), 15);
    }

    #[test]
    fn test_fleet_hold() {
        let mut nvy = Navy::default();
        add_warships(&mut nvy, NavalSize::Light, 3); // 3*1 = 3
        add_merchants(&mut nvy, NavalSize::Medium, 2); // 2*2 = 4
        add_galleys(&mut nvy, NavalSize::Heavy, 1); // 1*3 = 3
        assert_eq!(fleet_hold(&nvy), 10);
    }

    #[test]
    fn test_can_load() {
        assert!(can_load_army(ArmyStatus::Defend.to_value()));
        assert!(can_load_army(ArmyStatus::Attack.to_value()));
        assert!(!can_load_army(ArmyStatus::OnBoard.to_value()));
        assert!(!can_load_army(ArmyStatus::Militia.to_value()));
        assert!(!can_load_army(ArmyStatus::Garrison.to_value()));
        assert!(!can_load_army(ArmyStatus::Traded.to_value()));
        assert!(!can_load_army(ArmyStatus::General.to_value()));
    }

    #[test]
    fn test_civilian_attrition() {
        let mut nvy = Navy::default();
        nvy.people = 100;
        let mut rng = ConquerRng::new(42);
        civilian_attrition(&mut nvy, 50, false, &mut rng);
        // Should lose some people
        assert!(nvy.people <= 100);
    }
}
