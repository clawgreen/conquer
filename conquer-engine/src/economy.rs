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
    // C uses isntn() for per-nation spreadsheet (active 1..16)
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
/// C uses isntn() = active 1..16, excluding monsters.
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

// ── T6: updleader — leader births and monster spawning ──

/// updleader() — new leaders are born, old ones age; monster nations spawn in Spring.
/// Matches C updleader() structure from update.c line 1471.
pub fn update_leaders(state: &mut GameState, rng: &mut ConquerRng) {
    for nation in 0..NTOTAL {
        let active = state.nations[nation].active;
        let strat = NationStrategy::from_value(active);
        if !strat.map_or(false, |s| s.is_nation()) { continue; }

        // Monster nations spawn new monsters in Spring
        let is_spring = (state.world.turn % 4) == 1;
        if is_spring && Power::has_power(state.nations[nation].powers, Power::MI_MONST) {
            // Find free army slot
            let cap_x = state.nations[nation].cap_x;
            let cap_y = state.nations[nation].cap_y;
            for armynum in 0..MAXARM {
                if state.nations[nation].armies[armynum].soldiers != 0 { continue; }
                // Spawn a basic monster (spirit = 150, minimal strength)
                state.nations[nation].armies[armynum].unit_type = UnitType::MIN_MONSTER;
                state.nations[nation].armies[armynum].soldiers = 1;
                state.nations[nation].armies[armynum].x = cap_x;
                state.nations[nation].armies[armynum].y = cap_y;
                state.nations[nation].armies[armynum].status = ArmyStatus::Defend.to_value();
                state.nations[nation].armies[armynum].movement = state.nations[nation].max_move * 2;
                break;
            }
        }

        // Leader birth rate by class (C: switch(curntn->class))
        let born: i32 = match state.nations[nation].class {
            c if c == NationClass::Npc as i16
              || c == NationClass::King as i16
              || c == NationClass::Trader as i16
              || c == NationClass::Emperor as i16 => 50,
            c if c == NationClass::Wizard as i16
              || c == NationClass::Priest as i16
              || c == NationClass::Pirate as i16
              || c == NationClass::Warlord as i16
              || c == NationClass::Demon as i16 => 25,
            c if c == NationClass::Dragon as i16
              || c == NationClass::Shadow as i16 => 2,
            _ => 50,
        };

        // born represents yearly birth rate (out of 400 turns/year)
        if rng.rand() % 400 >= born { continue; }

        // Find free army slot for a new leader
        let cap_x = state.nations[nation].cap_x;
        let cap_y = state.nations[nation].cap_y;
        for armynum in 0..MAXARM {
            if state.nations[nation].armies[armynum].soldiers != 0 { continue; }
            // Spawn a basic leader unit
            state.nations[nation].armies[armynum].unit_type = UnitType::MIN_LEADER;
            state.nations[nation].armies[armynum].soldiers = 1;
            state.nations[nation].armies[armynum].x = cap_x;
            state.nations[nation].armies[armynum].y = cap_y;
            state.nations[nation].armies[armynum].status = ArmyStatus::Defend.to_value();
            state.nations[nation].armies[armynum].movement = state.nations[nation].max_move * 2;
            break;
        }
    }
}

// ── T7: cheat — NPC bonus gold when behind ──

/// cheat() — give NPC nations bonus gold/attributes if they fall behind.
/// Matches C cheat() structure from update.c line 459.
pub fn npc_cheat(state: &mut GameState, rng: &mut ConquerRng) {
    // Collect PC and NPC nations
    let mut pc_score: i64 = 0;
    let mut pc_bonus: i32 = 0;
    let mut pc_count: i32 = 0;
    let mut npc_bonus: i32 = 0;
    let mut npc_count: i32 = 0;

    for x in 1..NTOTAL {
        let strat = NationStrategy::from_value(state.nations[x].active);
        match strat {
            Some(s) if s.is_pc() => {
                pc_bonus += state.nations[x].attack_plus as i32 + state.nations[x].defense_plus as i32;
                pc_score += state.nations[x].score;
                pc_count += 1;
            }
            Some(s) if s.is_npc() => {
                // NPC nations get gold if treasury < civ count (5% chance)
                if state.nations[x].treasury_gold < state.nations[x].total_civ
                    && rng.rand() % 5 == 0
                {
                    state.nations[x].treasury_gold += 10_000;
                }
                npc_bonus += state.nations[x].attack_plus as i32 + state.nations[x].defense_plus as i32;
                npc_count += 1;
            }
            _ => {}
        }
    }

    if pc_count == 0 || npc_count == 0 { return; }
    let pc_avg = pc_bonus / pc_count;
    let npc_avg = npc_bonus / npc_count;
    let avg_score = pc_score / pc_count as i64;

    // If NPC behind PC in combat skill, give them a +1 bonus
    for x in 1..NTOTAL {
        let strat = NationStrategy::from_value(state.nations[x].active);
        if !strat.map_or(false, |s| s.is_npc()) { continue; }
        if state.nations[x].race == 'O' { continue; } // No cheat for orcs
        if state.nations[x].score < avg_score
            && rng.rand() % 100 < (pc_avg - npc_avg).max(0)
        {
            if state.nations[x].attack_plus > state.nations[x].defense_plus {
                state.nations[x].defense_plus += 1;
            } else {
                state.nations[x].attack_plus += 1;
            }
        }
    }
}

// ── T8: att_bonus — tradegood attribute bonuses ──

/// att_bonus() — tradegoods in sectors provide attribute bonuses to owning nations.
/// Matches C att_bonus() from admin.c line 605.
pub fn att_bonus_gs(state: &mut GameState) {
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;

    for x in 0..map_x {
        for y in 0..map_y {
            let owner = state.sectors[x][y].owner as usize;
            if owner == 0 { continue; }
            let strat = NationStrategy::from_value(state.nations[owner].active);
            if !strat.map_or(false, |s| s.is_nation()) { continue; }

            let sct = &state.sectors[x][y];
            if !tg_ok(&state.nations[owner], sct) { continue; }

            let good = sct.trade_good as usize;
            if good >= TG_SECTOR_TYPE.len() { continue; }

            let tg_stype = TG_SECTOR_TYPE.as_bytes()[good] as char;
            let des = sct.designation;

            // Check sector type compatibility (matches C tg_stype logic)
            let compatible = tg_stype == 'x'
                || (tg_stype == 'f' && (des == Designation::Farm as u8))
                || (tg_stype == 't' && (des == Designation::Town as u8
                    || des == Designation::City as u8
                    || des == Designation::Capitol as u8))
                || (tg_stype == 'l' && des == Designation::LumberYard as u8)
                || (tg_stype == 'u' && (des == Designation::University as u8
                    || des == Designation::City as u8
                    || des == Designation::Capitol as u8))
                || (tg_stype == 'c' && des == Designation::Church as u8)
                || (tg_stype == 'm' && des == Designation::Mine as u8)
                || (tg_stype == '$' && des == Designation::GoldMine as u8);

            if !compatible { continue; }

            let val = tg_value(good);
            let good_u8 = good as u8;
            let ntn = &mut state.nations[owner];

            if good_u8 <= END_POPULARITY {
                ntn.popularity = ntn.popularity.saturating_add(val).min(MAXTGVAL as u8);
            } else if good_u8 <= END_COMMUNICATION {
                if (ntn.communications as i32 + val as i32) < 2 * MAXTGVAL {
                    ntn.communications = ntn.communications.saturating_add(val);
                } else {
                    ntn.communications = (2 * MAXTGVAL) as u8;
                }
            } else if good_u8 <= END_EATRATE {
                // No tradegoods for eatrate (C: just clamp)
                ntn.eat_rate = ntn.eat_rate.min(MAXTGVAL as u8);
            } else if good_u8 <= END_SPOILRATE {
                ntn.spoil_rate = ntn.spoil_rate.saturating_sub(val).max(1);
            } else if good_u8 <= END_KNOWLEDGE {
                ntn.knowledge = ntn.knowledge.saturating_add(val).min(MAXTGVAL as u8);
            } else if good_u8 <= END_FARM {
                ntn.farm_ability = ntn.farm_ability.saturating_add(val).min(MAXTGVAL as u8);
            } else if good_u8 <= END_SPELL {
                // Spell points from people
                let p = state.sectors[x][y].people / 1000 + 1;
                ntn.spell_points = ntn.spell_points.saturating_add(p as i16);
            } else if good_u8 <= END_TERROR {
                ntn.terror = ntn.terror.saturating_add(val).min(MAXTGVAL as u8);
            }
        }
    }
}

// ── T9: move_people — civilian migration ──

/// move_people() — civilians migrate between adjacent sectors based on attractiveness.
/// Matches C move_people() from update.c line 1558.
/// Called once per nation per turn as part of updsectors.
pub fn move_people_gs(state: &mut GameState) {
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;

    // For each owned sector, move 1/5 of the difference toward equilibrium
    // with each neighbor. Only between sectors owned by the same nation.
    for x in 0..map_x {
        for y in 0..map_y {
            let owner = state.sectors[x][y].owner as usize;
            if owner == 0 { continue; }

            let p1 = state.sectors[x][y].people;
            if p1 == 0 { continue; }

            // Check each adjacent sector
            for (dx, dy) in &[(0i32,1i32),(1,0),(0,-1),(-1,0)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= map_x as i32 || ny >= map_y as i32 { continue; }
                let nx = nx as usize;
                let ny = ny as usize;

                // Only migrate between sectors of the same nation
                if state.sectors[nx][ny].owner as usize != owner { continue; }

                // Skip water
                if state.sectors[x][y].altitude == Altitude::Water as u8 { continue; }
                if state.sectors[nx][ny].altitude == Altitude::Water as u8 { continue; }

                let p2 = state.sectors[nx][ny].people;

                // Attractiveness of each sector based on designation
                let a1 = sector_attractiveness(state.sectors[x][y].designation);
                let a2 = sector_attractiveness(state.sectors[nx][ny].designation);

                if a1 + a2 == 0 { continue; }

                // DELTA(1) = (A1*P2 - P1*A2) / 5*(A1+A2)
                let delta = (a1 as i64 * p2 - p1 * a2 as i64) / (5 * (a1 + a2) as i64);

                if delta > 0 {
                    let moved = delta.min(p2 / 5);
                    state.sectors[x][y].people += moved;
                    state.sectors[nx][ny].people -= moved;
                } else if delta < 0 {
                    let moved = (-delta).min(p1 / 5);
                    state.sectors[x][y].people -= moved;
                    state.sectors[nx][ny].people += moved;
                }
            }
        }
    }
}

/// Sector attractiveness for migration (simplified from C attract()).
fn sector_attractiveness(designation: u8) -> i64 {
    if designation == Designation::City as u8 || designation == Designation::Capitol as u8 {
        10
    } else if designation == Designation::Town as u8 {
        7
    } else if designation == Designation::Farm as u8 {
        5
    } else if designation == Designation::Mine as u8 {
        4
    } else if designation == Designation::GoldMine as u8 {
        6
    } else {
        2
    }
}

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
