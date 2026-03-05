// conquer-engine/src/economy.rs — Economy & production ported from update.c, misc.c
//
// T145-T157: updsectors, produce, spreadsheet, updcomodities, updmil,
// taxation, population growth, food consumption, inflation, depletion.

use conquer_core::*;
use conquer_core::rng::ConquerRng;
use conquer_core::powers::Power;
use conquer_core::tables::*;
use crate::utils::*;

/// Spreadsheet calculation — matches C spreadsheet() exactly.
/// Computes food, gold, metal, jewels production for a nation.
pub fn spreadsheet(
    state: &GameState,
    nation_id: usize,
) -> Spreadsheet {
    let ntn = &state.nations[nation_id];
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;
    let turn = state.world.turn;

    let mut spread = Spreadsheet::default();
    spread.food = ntn.total_food;
    spread.gold = ntn.treasury_gold;
    spread.metal = ntn.metals;
    spread.jewels = ntn.jewels;
    spread.sectors = 0;
    spread.civilians = 0;

    for x in 0..map_x {
        for y in 0..map_y {
            let sct = &state.sectors[x][y];
            if sct.owner as usize != nation_id {
                continue;
            }

            spread.sectors += 1;
            spread.civilians += sct.people;

            let des = sct.designation;
            let mut product: i64 = 0;

            if des == Designation::Mine as u8 {
                // Check tg_ok
                if !tg_ok(ntn, sct) { continue; }
                spread.in_metal += sct.people;
                if sct.people > TOMANYPEOPLE {
                    product = sct.metal as i64 * TOMANYPEOPLE;
                    product += sct.metal as i64 * (sct.people - TOMANYPEOPLE) / 2;
                } else {
                    product = sct.metal as i64 * sct.people;
                }
                if Power::has_power(ntn.powers, Power::MINER) { product *= 2; }
                if Power::has_power(ntn.powers, Power::STEEL) { product *= 2; }
                spread.metal += product;
                spread.rev_metal += product * TAXMETAL * ntn.tax_rate as i64 / 100;
            } else if des == Designation::Farm as u8 {
                spread.in_farm += sct.people;
                let food_val = tofood(sct, Some(ntn)) as i64;
                if sct.people > TOMANYPEOPLE {
                    product = food_val * TOMANYPEOPLE;
                    product += food_val * (sct.people - TOMANYPEOPLE) / 2;
                } else {
                    product = food_val * sct.people;
                }

                // Seasonal adjustment
                match (turn % 4) as u8 {
                    1 => { product /= 2; }  // SPRING
                    2 => {}                  // SUMMER (full)
                    3 => { product = product * 5 / 2; }  // FALL
                    0 => { product = 0; }    // WINTER
                    _ => {}
                }

                // Mill bonus: search for neighboring mills
                let mut _foundmill = false;
                'mill_search: for i in (x as i32 - 1)..=(x as i32 + 1) {
                    for j in (y as i32 - 1)..=(y as i32 + 1) {
                        if on_map(i, j, map_x as i32, map_y as i32) {
                            let ns = &state.sectors[i as usize][j as usize];
                            if ns.owner == sct.owner
                                && ns.designation == Designation::Mill as u8
                                && ns.people >= MILLSIZE
                            {
                                product = product * 12 / 10;
                                _foundmill = true;
                                break 'mill_search;
                            }
                        }
                    }
                }
                spread.food += product;
                spread.rev_food += product * TAXFOOD * ntn.tax_rate as i64 / 100;
            } else if des == Designation::GoldMine as u8 {
                if !tg_ok(ntn, sct) { continue; }
                spread.in_gold += sct.people;
                if sct.people > TOMANYPEOPLE {
                    product = sct.jewels as i64 * TOMANYPEOPLE;
                    product += sct.jewels as i64 * (sct.people - TOMANYPEOPLE) / 2;
                } else {
                    product = sct.jewels as i64 * sct.people;
                }
                if Power::has_power(ntn.powers, Power::MINER) { product *= 2; }
                spread.jewels += product;
                spread.rev_jewels += product * TAXGOLD * ntn.tax_rate as i64 / 100;
            } else if des == Designation::City as u8 || des == Designation::Capitol as u8 {
                let mut cap_pop = sct.people;
                spread.in_cap += cap_pop;
                if Power::has_power(ntn.powers, Power::ARCHITECT) {
                    cap_pop *= 2;
                }
                spread.rev_cap += cap_pop * TAXCITY * ntn.tax_rate as i64 / 100;
            } else if des == Designation::Town as u8 {
                spread.in_city += sct.people;
                let mut city_pop = sct.people;
                if Power::has_power(ntn.powers, Power::ARCHITECT) {
                    city_pop *= 2;
                }
                spread.rev_city += city_pop * TAXTOWN * ntn.tax_rate as i64 / 100;
            } else {
                // Other sectors
                spread.in_other += sct.people;
                let food_val = tofood(sct, Some(ntn)) as i64;
                if sct.people > TOMANYPEOPLE {
                    product = food_val * TOMANYPEOPLE;
                    product += food_val * (sct.people - TOMANYPEOPLE) / 2;
                } else {
                    product = food_val * sct.people;
                }
                spread.rev_other += product * TAXOTHR * ntn.tax_rate as i64 / 100;
            }
        }
    }

    spread.gold += spread.rev_food + spread.rev_jewels + spread.rev_metal
        + spread.rev_city + spread.rev_cap + spread.rev_other;

    spread
}

/// updsectors() — update all sectors: population growth, mining depletion, diplomacy.
/// Matches C updsectors() logic for population/growth/depletion.
pub fn updsectors(state: &mut GameState, rng: &mut ConquerRng) {
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;
    let turn = state.world.turn;

    for x in 0..map_x {
        for y in 0..map_y {
            let owner = state.sectors[x][y].owner as usize;
            if owner == 0 { continue; }

            // Random resource discovery
            if rng.rand() % 100 < FINDPERCENT {
                if state.sectors[x][y].trade_good == TradeGood::None as u8 {
                    if rng.rand() % 2 == 0 {
                        getmetal(&mut state.sectors[x][y], rng);
                    } else {
                        getjewel(&mut state.sectors[x][y], rng);
                    }
                }
            }

            // Calculate reproduction per season
            let repro = state.nations[owner].repro;
            let mut rephold = repro as i32 / 4;
            let season = (turn % 4) as u8;
            if season != 0 /* not winter */ && season <= (repro % 4) as u8 {
                rephold += 1;
            }

            // Only one capitol per nation
            if state.sectors[x][y].designation == Designation::Capitol as u8 {
                let ntn = &state.nations[owner];
                if ntn.cap_x as usize != x || ntn.cap_y as usize != y {
                    state.sectors[x][y].designation = Designation::City as u8;
                }
            }

            let sct = &mut state.sectors[x][y];
            let people = sct.people;
            let des = sct.designation;

            // Population growth
            if people >= ABSMAXPEOPLE {
                sct.people = ABSMAXPEOPLE;
                // Mining depletion at max pop
                if sct.people * sct.metal as i64 > 2 * (rng.rand() as i64 % 100) * TOMUCHMINED {
                    if des == Designation::Mine as u8 && sct.metal > 0 {
                        sct.metal -= 1;
                    }
                }
                if sct.people * sct.jewels as i64 > 2 * (rng.rand() as i64 % 100) * TOMUCHMINED {
                    if des == Designation::GoldMine as u8 && sct.jewels > 0 {
                        sct.jewels -= 1;
                    }
                }
            } else if people > TOMANYPEOPLE
                && des != Designation::Town as u8
                && des != Designation::Capitol as u8
                && des != Designation::City as u8
            {
                sct.people += (rephold as i64 * people) / 200;
                if sct.people > ABSMAXPEOPLE {
                    sct.people = ABSMAXPEOPLE;
                }
                // Mining depletion
                if sct.people * sct.metal as i64 > 2 * (rng.rand() as i64 % 100) * TOMUCHMINED {
                    if des == Designation::Mine as u8 && sct.metal > 0 {
                        sct.metal -= 1;
                    }
                }
                if sct.people * sct.jewels as i64 > 2 * (rng.rand() as i64 % 100) * TOMUCHMINED {
                    if des == Designation::GoldMine as u8 && sct.jewels > 0 {
                        sct.jewels -= 1;
                    }
                }
            } else if people < 100 {
                sct.people += people / 10;
            } else {
                sct.people += (rephold as i64 * people) / 100;
                // Mining depletion
                if sct.people * sct.metal as i64 > (rng.rand() as i64 % 100) * TOMUCHMINED {
                    if des == Designation::Mine as u8 && sct.metal > 0 {
                        sct.metal -= 1;
                    }
                }
                if sct.people * sct.jewels as i64 > (rng.rand() as i64 % 100) * TOMUCHMINED {
                    if des == Designation::GoldMine as u8 && sct.jewels > 0 {
                        sct.jewels -= 1;
                    }
                }
            }

            // If depleted, devastate
            if (sct.designation == Designation::GoldMine as u8 && sct.jewels == 0)
                || (sct.designation == Designation::Mine as u8 && sct.metal == 0)
            {
                sct.trade_good = TradeGood::None as u8;
                sct.designation = Designation::Devastated as u8;
            }

            // Desert sector reverts to no-designation
            if tofood(sct, None) < DESFOOD {
                if sct.designation != Designation::Stockade as u8
                    && sct.designation != Designation::Fort as u8
                    && sct.designation != Designation::Road as u8
                {
                    sct.designation = Designation::NoDesig as u8;
                }
            }
        }
    }

    // Post-sector update: run spreadsheet for each nation, calculate poverty/inflation
    for country in 1..NTOTAL {
        let ntn = &state.nations[country];
        if !NationStrategy::from_value(ntn.active).map_or(false, |s| s.is_nation()) {
            continue;
        }

        let spread = spreadsheet(state, country);
        let ntn = &mut state.nations[country];

        // Popularity adjustment for inflation
        let pop_val = ntn.popularity as i32 - 2 * ntn.inflation as i32;
        if pop_val < MAXTGVAL {
            ntn.popularity = pop_val.max(0) as u8;
        } else {
            ntn.popularity = MAXTGVAL as u8;
        }

        ntn.total_sectors = spread.sectors as i16;
        ntn.total_civ = spread.civilians;
        ntn.total_food = spread.food;

        // Charity
        let charity_amount = ((spread.gold - ntn.treasury_gold) * ntn.charity as i64) / 100;
        let charity = charity_amount.max(0);

        ntn.treasury_gold = spread.gold - charity;

        let charity_per_civ = if ntn.total_civ > 0 { charity / ntn.total_civ } else { 0 };

        // Calculate poverty base — matches C exactly
        if ntn.treasury_gold < 0 {
            ntn.poverty = 95;
        } else if ntn.total_civ < 100 {
            ntn.poverty = 20;
        } else {
            let ratio = ntn.treasury_gold / ntn.total_civ;
            if ratio < 30 {
                ntn.poverty = (95 - ratio) as u8;
            } else if ratio < 80 {
                ntn.poverty = (65 - (ratio - 30) / 2) as u8;
            } else if ratio < 120 {
                ntn.poverty = (40 - (ratio - 80) / 4) as u8;
            } else if ratio < 200 {
                ntn.poverty = (30 - (ratio - 120) / 8) as u8;
            } else {
                ntn.poverty = 20;
            }
        }

        // Charity popularity increase
        ntn.popularity = (ntn.popularity as i64 + 5 * charity_per_civ).min(MAXTGVAL as i64) as u8;

        // Charity poverty reduction
        let charity_reduce = ((charity_per_civ + 1) / 2) as u8;
        if ntn.poverty < charity_reduce {
            ntn.poverty = 0;
        } else {
            ntn.poverty -= charity_reduce;
        }

        // Calculate inflation — matches C exactly
        if ntn.inflation > 0 {
            ntn.inflation = (rng.rand() % (ntn.inflation as i32 / 2 + 1)) as i16;
        } else {
            ntn.inflation = 0;
        }
        ntn.inflation += (ntn.tax_rate as i16 / 4)
            + (rng.rand() % (ntn.tax_rate as i32 * 3 / 4 + 1)) as i16;

        // Military adjustment to inflation
        if spread.civilians > 0 {
            ntn.inflation += ((ntn.total_mil * 100 / spread.civilians - 15) / 5) as i16;
        }
        // Poverty adjustment
        ntn.inflation += (ntn.poverty as i16 - 50) / 2;

        // Apply inflation to gold — matches C exactly
        if ntn.treasury_gold > 1_000_000 {
            ntn.treasury_gold = (ntn.treasury_gold / (400 + ntn.inflation as i64)) * 400;
        } else {
            ntn.treasury_gold = (ntn.treasury_gold * 400) / (400 + ntn.inflation as i64);
        }

        // Update resource totals from spreadsheet
        ntn.metals = spread.metal;
        ntn.jewels = spread.jewels;
    }
}

/// updcomodities() — food consumption, spoilage, starvation, jewel balancing.
/// Matches C updcomodities() exactly.
pub fn updcomodities(state: &mut GameState, _rng: &mut ConquerRng) {
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;

    for country in 1..NTOTAL {
        let ntn = &state.nations[country];
        if !NationStrategy::from_value(ntn.active).map_or(false, |s| s.is_nation()) {
            continue;
        }

        // P_EATRATE = ((float)curntn->eatrate) / 25.0
        let eat_rate_f = ntn.eat_rate as f64 / 25.0;

        // Soldiers eat 2x as much
        let mil_food = (ntn.total_mil as f64 * eat_rate_f * 2.0) as i64;
        let civ_food = (ntn.total_civ as f64 * eat_rate_f) as i64;

        let ntn = &mut state.nations[country];
        ntn.total_food -= mil_food;
        ntn.total_food -= civ_food;

        // Starve people if food < 0
        if ntn.total_food < 0 {
            for x in 0..map_x {
                for y in 0..map_y {
                    let sct = &mut state.sectors[x][y];
                    if sct.owner as usize != country { continue; }
                    if ntn.total_food >= 0 { break; }

                    let des = sct.designation;
                    if des == Designation::Town as u8
                        || des == Designation::Capitol as u8
                        || des == Designation::City as u8
                    {
                        if ntn.total_food < 0 {
                            if sct.people < -ntn.total_food {
                                // BUG-COMPAT: C adds tfood/3 to people (negative, so subtracts)
                                sct.people += ntn.total_food / 3;
                                ntn.total_food = 0;
                            } else {
                                ntn.total_food += sct.people;
                                let dead = sct.people / 3;
                                sct.people -= dead;
                            }
                        }
                    }
                }
            }
        }

        let ntn = &mut state.nations[country];
        // Food floor at 0
        if ntn.total_food < 0 {
            ntn.total_food = 0;
        }

        // Spoilage
        let spoil_rate = ntn.spoil_rate as f64;
        ntn.total_food = ((ntn.total_food as f64) * (100.0 - spoil_rate) / 100.0) as i64;

        // Jewel balancing: if gold > GOLDTHRESH * jewels, auto-buy jewels
        if (ntn.treasury_gold as f64) - (GOLDTHRESH as f64) * (ntn.jewels as f64) > 0.0 {
            let xx = ntn.treasury_gold - GOLDTHRESH * ntn.jewels;
            // BUG-COMPAT: C uses dtol((double)xx*GODJEWL/GODPRICE)
            let bought = (xx as f64 * GODJEWL as f64 / GODPRICE as f64) as i64;
            ntn.jewels += bought;
            ntn.treasury_gold -= xx;
        }

        // Fix overflow
        if ntn.treasury_gold < -BIG { ntn.treasury_gold = BIG; }
        if ntn.total_food < -BIG { ntn.total_food = BIG; }
        if ntn.jewels < -BIG { ntn.jewels = BIG; }
        if ntn.metals < -BIG { ntn.metals = BIG; }
    }
}

/// updmil() — reset military movement, upkeep, siege mechanics.
/// Simplified version matching core C logic for movement and maintenance.
pub fn updmil(state: &mut GameState, rng: &mut ConquerRng) {
    for country in 1..NTOTAL {
        let ntn = &state.nations[country];
        if !NationStrategy::from_value(ntn.active).map_or(false, |s| s.is_nation()) {
            continue;
        }

        // Check if leader exists (disarray check)
        let leader_type = getleader(ntn.class);
        let mut disarray = true;
        for army in &ntn.armies {
            if army.unit_type == leader_type.wrapping_sub(1) && army.soldiers > 0 {
                disarray = false;
                break;
            }
        }

        let ntn = &mut state.nations[country];
        ntn.total_mil = 0;
        ntn.total_ships = 0;

        // Spell point decay: 25% chance to halve
        if rng.rand() % 4 == 0 {
            ntn.spell_points /= 2;
        }

        // Spell point gains from powers
        if Power::has_power(ntn.powers, Power::SUMMON) {
            ntn.spell_points += 4;
            if Power::has_power(ntn.powers, Power::WYZARD) { ntn.spell_points += 3; }
            if Power::has_power(ntn.powers, Power::SORCERER) { ntn.spell_points += 3; }
        }
        if Power::has_power(ntn.powers, Power::MA_MONST) { ntn.spell_points += 2; }
        if Power::has_power(ntn.powers, Power::AV_MONST) { ntn.spell_points += 1; }
        if Power::has_power(ntn.powers, Power::MI_MONST) && rng.rand() % 2 == 0 {
            ntn.spell_points += 1;
        }

        // Process armies
        for armynum in 0..MAXARM {
            if ntn.armies[armynum].soldiers <= 0 { continue; }

            let at = ntn.armies[armynum].unit_type;
            let stat = ntn.armies[armynum].status;
            let soldiers = ntn.armies[armynum].soldiers;

            // Count military
            if at < UnitType::MIN_LEADER {
                ntn.total_mil += soldiers;
                if at == UnitType::MILITIA.0 {
                    ntn.armies[armynum].status = ArmyStatus::Militia.to_value();
                }
            }

            // Set movement
            if disarray {
                ntn.armies[armynum].movement = 0;
            } else {
                let move_idx = (at % UTYPE) as usize;
                let unit_move = UNIT_MOVE.get(move_idx).copied().unwrap_or(10) as u8;

                match ArmyStatus::from_value(stat) {
                    ArmyStatus::March => {
                        ntn.armies[armynum].movement = (ntn.max_move * unit_move) / 5;
                    }
                    ArmyStatus::Militia | ArmyStatus::OnBoard => {
                        ntn.armies[armynum].movement = 0;
                    }
                    _ => {
                        ntn.armies[armynum].movement = (ntn.max_move * unit_move) / 10;
                    }
                }
            }

            // Flight
            if avian(at)
                && ArmyStatus::from_value(stat) != ArmyStatus::OnBoard
                && stat < NUMSTATUS
            {
                ntn.armies[armynum].status = ArmyStatus::Flight.to_value();
            }

            // Military maintenance
            let maint_idx = (at % UTYPE) as usize;
            let maint = UNIT_MAINTENANCE.get(maint_idx).copied().unwrap_or(50) as i64;

            let has_sapper = Power::has_power(ntn.powers, Power::SAPPER);
            if has_sapper
                && (at == UnitType::CATAPULT.0 || at == UnitType::SIEGE_UNIT.0)
            {
                ntn.treasury_gold -= soldiers * maint / 2;
            } else if at < UnitType::MIN_LEADER {
                ntn.treasury_gold -= soldiers * maint;
            } else if at >= UnitType::MIN_MONSTER {
                ntn.treasury_gold -= 5 * maint;
                if ntn.jewels > maint {
                    ntn.jewels -= maint;
                } else {
                    // Monster leaves due to lack of jewels
                    ntn.armies[armynum].soldiers = 0;
                }
            }
        }

        // Navy maintenance
        for nvynum in 0..MAXNAVY {
            let has_ships = ntn.navies[nvynum].warships != 0
                || ntn.navies[nvynum].merchant != 0
                || ntn.navies[nvynum].galleys != 0;
            if has_ships {
                // Fleet speed and crew
                if !disarray {
                    let speed = fleet_speed(&ntn.navies[nvynum]);
                    let crew = ntn.navies[nvynum].crew;
                    let has_sailor = Power::has_power(ntn.powers, Power::SAILOR);
                    let mut mv = ((speed as u32 * crew as u32) / SHIPCREW as u32) as u8;
                    if has_sailor { mv *= 2; }
                    ntn.navies[nvynum].movement = mv;
                } else {
                    ntn.navies[nvynum].movement = 0;
                }

                // Ship count and maintenance
                let total_ships = fleet_ships(&ntn.navies[nvynum]);
                ntn.total_ships += total_ships as i16;
                let hold = fleet_hold(&ntn.navies[nvynum]);
                ntn.treasury_gold -= hold as i64 * SHIPMAINT;
            }
        }
    }
}

/// fleet_speed(navy) — compute fleet speed based on ship types.
/// Matches C fltspeed() logic.
fn fleet_speed(navy: &Navy) -> i32 {
    let mut speed = 0;
    let mut count = 0;

    // Warships
    for size in [NavalSize::Light, NavalSize::Medium, NavalSize::Heavy] {
        let n = NavalSize::ships(navy.warships, size) as i32;
        if n > 0 {
            let bonus = match size {
                NavalSize::Light => N_SIZESPD * 2,
                NavalSize::Medium => N_SIZESPD,
                NavalSize::Heavy => 0,
            };
            speed += n * (N_WSPD + bonus);
            count += n;
        }
    }

    // Galleys
    for size in [NavalSize::Light, NavalSize::Medium, NavalSize::Heavy] {
        let n = NavalSize::ships(navy.galleys, size) as i32;
        if n > 0 {
            let bonus = match size {
                NavalSize::Light => N_SIZESPD * 2,
                NavalSize::Medium => N_SIZESPD,
                NavalSize::Heavy => 0,
            };
            speed += n * (N_GSPD + bonus);
            count += n;
        }
    }

    // Merchants
    for size in [NavalSize::Light, NavalSize::Medium, NavalSize::Heavy] {
        let n = NavalSize::ships(navy.merchant, size) as i32;
        if n > 0 {
            let bonus = match size {
                NavalSize::Light => N_SIZESPD * 2,
                NavalSize::Medium => N_SIZESPD,
                NavalSize::Heavy => 0,
            };
            speed += n * (N_MSPD + bonus);
            count += n;
        }
    }

    if count > 0 { speed / count } else { 0 }
}

/// fleet_ships(navy) — count total ships in fleet.
fn fleet_ships(navy: &Navy) -> i32 {
    let mut total = 0;
    for size in [NavalSize::Light, NavalSize::Medium, NavalSize::Heavy] {
        total += NavalSize::ships(navy.warships, size) as i32;
        total += NavalSize::ships(navy.merchant, size) as i32;
        total += NavalSize::ships(navy.galleys, size) as i32;
    }
    total
}

/// fleet_hold(navy) — compute total hold capacity.
fn fleet_hold(navy: &Navy) -> i32 {
    let mut total = 0;
    for size in [NavalSize::Light, NavalSize::Medium, NavalSize::Heavy] {
        total += NavalSize::ships(navy.warships, size) as i32;
        total += NavalSize::ships(navy.merchant, size) as i32;
        total += NavalSize::ships(navy.galleys, size) as i32;
    }
    total
}

/// MAXTGVAL constant (used in economy calcs).
const MAXTGVAL: i32 = 100;

/// GODJEWL / GODPRICE constants for jewel buying
const GODJEWL: i64 = 3000;
const GODPRICE: i64 = 25000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spreadsheet_empty_nation() {
        let state = GameState::new(32, 32);
        let spread = spreadsheet(&state, 1);
        assert_eq!(spread.sectors, 0);
        assert_eq!(spread.civilians, 0);
    }

    #[test]
    fn test_spreadsheet_farm_production() {
        let mut state = GameState::new(32, 32);
        state.nations[1].active = NationStrategy::PcGood as u8;
        state.nations[1].tax_rate = 10;
        state.nations[1].race = 'H';
        state.world.turn = 2; // SUMMER

        // Set up a farm sector
        state.sectors[5][5].owner = 1;
        state.sectors[5][5].designation = Designation::Farm as u8;
        state.sectors[5][5].altitude = Altitude::Clear as u8;
        state.sectors[5][5].vegetation = Vegetation::Good as u8;
        state.sectors[5][5].people = 1000;

        let spread = spreadsheet(&state, 1);
        assert_eq!(spread.sectors, 1);
        assert_eq!(spread.civilians, 1000);
        // Food = 9 (Good veg) * 1000 people = 9000 (Summer, no modifier)
        assert_eq!(spread.food, 9000);
        assert!(spread.gold > 0);
    }

    #[test]
    fn test_inflation_calculation() {
        // Just verify it doesn't panic
        let mut state = GameState::new(32, 32);
        let mut rng = ConquerRng::new(42);
        state.nations[1].active = NationStrategy::PcGood as u8;
        state.nations[1].tax_rate = 15;
        state.nations[1].race = 'H';
        state.nations[1].repro = 4;
        state.nations[1].treasury_gold = 100000;
        state.nations[1].total_food = 50000;
        state.nations[1].total_civ = 5000;
        state.nations[1].total_mil = 500;
        state.nations[1].eat_rate = 25; // P_EATRATE = 1.0
        state.nations[1].spoil_rate = 10;
        state.world.turn = 2;

        updsectors(&mut state, &mut rng);
    }
}
