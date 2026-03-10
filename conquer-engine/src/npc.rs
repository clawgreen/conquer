#![allow(unused_variables, unused_imports, unused_assignments)]
// conquer-engine/src/npc.rs — NPC AI ported from npc.c
//
// T211-T230: nationrun, getdstatus, newdip, redomil, n_redes,
// sector redesignation, attractiveness calculation, army drafting,
// diplomacy updates, tax/charity management.

use conquer_core::*;

use conquer_core::rng::ConquerRng;
use conquer_core::tables::*;
use crate::utils::*;
use crate::magic;
use crate::movement;

/// NPC operating range
#[derive(Debug, Clone, Copy)]
pub struct NpcRange {
    pub stx: i32,
    pub sty: i32,
    pub endx: i32,
    pub endy: i32,
}

/// Calculate NPC operating range from capitol.
/// Matches C npc.c stx/sty/endx/endy setup.
pub fn npc_range(nation: &Nation, map_x: i32, map_y: i32) -> NpcRange {
    let strat = NationStrategy::from_value(nation.active);
    let is_pc = strat.map_or(false, |s| s.is_pc());

    if is_pc {
        return NpcRange {
            stx: 0,
            sty: 0,
            endx: map_x,
            endy: map_y,
        };
    }

    let cap_x = nation.cap_x as i32;
    let cap_y = nation.cap_y as i32;

    NpcRange {
        stx: if cap_x > NPCTOOFAR { cap_x - NPCTOOFAR } else { 0 },
        sty: if cap_y > NPCTOOFAR { cap_y - NPCTOOFAR } else { 0 },
        endx: if cap_x + NPCTOOFAR < map_x { cap_x + NPCTOOFAR } else { map_x },
        endy: if cap_y + NPCTOOFAR < map_y { cap_y + NPCTOOFAR } else { map_y },
    }
}

/// newdip(ntn1, ntn2) — set initial diplomacy when nations first meet.
/// Matches C newdip() exactly.
pub fn new_diplomacy(
    state: &mut GameState,
    ntn1: usize,
    ntn2: usize,
    rng: &mut ConquerRng,
) {
    let strat1 = NationStrategy::from_value(state.nations[ntn1].active);
    let strat2 = NationStrategy::from_value(state.nations[ntn2].active);

    if strat1.map_or(false, |s| s.is_pc()) {
        if state.nations[ntn2].race == 'O' {
            state.nations[ntn1].diplomacy[ntn2] = DiplomaticStatus::Hostile as u8;
        } else {
            state.nations[ntn1].diplomacy[ntn2] = DiplomaticStatus::Neutral as u8;
        }
        return;
    }

    let race1 = state.nations[ntn1].race;
    let race2 = state.nations[ntn2].race;

    if race1 == 'O' || race2 == 'O' {
        if state.nations[ntn1].diplomacy[ntn2] == DiplomaticStatus::Unmet as u8 {
            if rng.rand() % 2 == 0 || strat1.map_or(false, |s| s.is_pc()) {
                state.nations[ntn1].diplomacy[ntn2] = DiplomaticStatus::Hostile as u8;
            } else {
                state.nations[ntn1].diplomacy[ntn2] = DiplomaticStatus::War as u8;
            }
        }
    } else if strat2.map_or(false, |s| s.is_monster()) {
        state.nations[ntn1].diplomacy[ntn2] = DiplomaticStatus::War as u8;
    } else if strat1.map_or(false, |s| s.is_pc()) {
        if state.nations[ntn1].diplomacy[ntn2] == DiplomaticStatus::Unmet as u8 {
            state.nations[ntn1].diplomacy[ntn2] = DiplomaticStatus::Neutral as u8;
        }
    } else if race1 == race2 {
        if rng.rand() % 2 < 1 {
            state.nations[ntn1].diplomacy[ntn2] = DiplomaticStatus::Friendly as u8;
        } else {
            state.nations[ntn1].diplomacy[ntn2] = DiplomaticStatus::Neutral as u8;
        }
    } else {
        state.nations[ntn1].diplomacy[ntn2] = DiplomaticStatus::Neutral as u8;
    }
}

/// getdstatus() — NPC diplomacy update. Matches C getdstatus() exactly.
pub fn get_diplomatic_status(
    state: &mut GameState,
    nation_idx: usize,
    rng: &mut ConquerRng,
) -> Vec<String> {
    let mut news = Vec::new();
    let nation = &state.nations[nation_idx];
    let strat = NationStrategy::from_value(nation.active);

    if !strat.map_or(false, |s| s.is_npc()) {
        return news;
    }

    // Calculate base hostility
    let svhostile: i32 = match nation.active {
        x if x == NationStrategy::Good6Free as u8
            || x == NationStrategy::Isolationist as u8
            || x == NationStrategy::Neutral6Free as u8
            || x == NationStrategy::Evil6Free as u8 =>
        {
            5
        }
        x if x == NationStrategy::Good4Free as u8
            || x == NationStrategy::Neutral4Free as u8
            || x == NationStrategy::Evil4Free as u8 =>
        {
            10
        }
        x if x == NationStrategy::Good2Free as u8
            || x == NationStrategy::Neutral2Free as u8
            || x == NationStrategy::Evil2Free as u8 =>
        {
            20
        }
        x if x == NationStrategy::Good0Free as u8
            || x == NationStrategy::Neutral0Free as u8
            || x == NationStrategy::Evil0Free as u8 =>
        {
            35
        }
        _ => 5,
    };

    let mut old_stats = vec![0u8; NTOTAL];

    let range = npc_range(&state.nations[nation_idx], state.world.map_x as i32, state.world.map_y as i32);

    for x in 1..NTOTAL {
        let x_strat = NationStrategy::from_value(state.nations[x].active);
        if !x_strat.map_or(false, |s| s.is_nation()) {
            continue;
        }

        let mut hostile = svhostile;
        let npc_type_cur = NationStrategy::from_value(state.nations[nation_idx].active)
            .map_or(0, |s| s.npc_type());
        let npc_type_x = NationStrategy::from_value(state.nations[x].active)
            .map_or(0, |s| s.npc_type());

        if npc_type_cur != npc_type_x {
            hostile += 20;
        }
        let mut friendly = 60 - hostile;

        if state.nations[nation_idx].active == NationStrategy::Isolationist as u8 {
            friendly -= 20;
        }

        let x_is_neutral = x_strat.map_or(false, |s| s.is_neutral());
        if x_is_neutral {
            friendly -= 10;
            hostile -= 10;
        }

        if state.nations[x].race == state.nations[nation_idx].race {
            friendly += 10;
            hostile -= 10;
        }

        let cur_is_neutral = NationStrategy::from_value(state.nations[nation_idx].active)
            .map_or(false, |s| s.is_neutral());
        if cur_is_neutral {
            friendly -= 20;
            hostile -= 20;
        }

        // Capitol adjacency
        let cap_x = state.nations[nation_idx].cap_x as i32;
        let cap_y = state.nations[nation_idx].cap_y as i32;
        for cx in (cap_x - 1)..=(cap_x + 1) {
            for cy in (cap_y - 1)..=(cap_y + 1) {
                if state.on_map(cx, cy) {
                    if state.sectors[cx as usize][cy as usize].owner as usize == x {
                        friendly -= 10;
                        hostile += 10;
                    }
                }
            }
        }

        if friendly < 0 {
            friendly = 0;
        }
        if hostile < 0 {
            hostile = 0;
        }

        old_stats[x] = state.nations[nation_idx].diplomacy[x];

        // Break bad treaties
        if state.nations[nation_idx].diplomacy[x] == DiplomaticStatus::Treaty as u8 {
            if state.nations[x].diplomacy[nation_idx] >= DiplomaticStatus::War as u8 {
                state.nations[nation_idx].diplomacy[x] = DiplomaticStatus::Jihad as u8;
            }
            continue;
        }

        if state.nations[nation_idx].diplomacy[x] == DiplomaticStatus::Jihad as u8
            || state.nations[nation_idx].diplomacy[x] == DiplomaticStatus::Unmet as u8
            || strat.map_or(false, |s| s.is_pc())
        {
            continue;
        }

        // Bigger nation pressure
        if state.nations[x].total_mil > 4 * state.nations[nation_idx].total_mil
            && state.nations[x].score > 4 * state.nations[nation_idx].score
        {
            if state.nations[nation_idx].diplomacy[x] < DiplomaticStatus::War as u8 {
                if (rng.rand() % 100) as i32 <= hostile {
                    state.nations[nation_idx].diplomacy[x] += 1;
                }
            }
        }

        // Mirror diplomacy
        let our_status = state.nations[nation_idx].diplomacy[x];
        let their_status = state.nations[x].diplomacy[nation_idx];

        if our_status == DiplomaticStatus::War as u8
            && their_status < DiplomaticStatus::War as u8
        {
            if (rng.rand() % 100) as i32 <= friendly {
                state.nations[nation_idx].diplomacy[x] -= 1;
            }
        }

        if our_status < DiplomaticStatus::War as u8
            && our_status > DiplomaticStatus::Allied as u8
        {
            if their_status > 1 + our_status {
                if (rng.rand() % 100) as i32 <= hostile {
                    state.nations[nation_idx].diplomacy[x] += 1;
                }
            } else if their_status + 1 < our_status {
                if (rng.rand() % 100) as i32 <= friendly {
                    state.nations[nation_idx].diplomacy[x] -= 1;
                }
            }
        }

        // Random hostile/friendly drift
        if (rng.rand() % 100) as i32 <= hostile {
            let d = state.nations[nation_idx].diplomacy[x];
            if d != DiplomaticStatus::Jihad as u8 && d != DiplomaticStatus::Treaty as u8 {
                state.nations[nation_idx].diplomacy[x] += 1;
            }
        }
        if (rng.rand() % 100) as i32 <= friendly {
            let d = state.nations[nation_idx].diplomacy[x];
            if d != DiplomaticStatus::Treaty as u8
                && d != DiplomaticStatus::Jihad as u8
                && d != DiplomaticStatus::War as u8
            {
                state.nations[nation_idx].diplomacy[x] -= 1;
            }
        }
    }

    // Ceasefire and war declarations
    for x in 1..NTOTAL {
        let x_strat = NationStrategy::from_value(state.nations[x].active);
        if !x_strat.map_or(false, |s| s.is_nation()) {
            continue;
        }

        let old = old_stats[x];
        let new = state.nations[nation_idx].diplomacy[x];

        if rng.rand() % 5 == 0
            && state.nations[x].diplomacy[nation_idx] == DiplomaticStatus::War as u8
            && new == DiplomaticStatus::War as u8
        {
            state.nations[x].diplomacy[nation_idx] = DiplomaticStatus::Hostile as u8;
            state.nations[nation_idx].diplomacy[x] = DiplomaticStatus::Hostile as u8;
            news.push(format!(
                "nation {} and {} announce ceasefire",
                state.nations[nation_idx].name, state.nations[x].name
            ));
        } else if old == DiplomaticStatus::War as u8 && new == DiplomaticStatus::War as u8 {
            news.push(format!(
                "nation {} stays at war with {}",
                state.nations[nation_idx].name, state.nations[x].name
            ));
        } else if old < DiplomaticStatus::War as u8 && new == DiplomaticStatus::War as u8 {
            news.push(format!(
                "nation {} goes to war with {}",
                state.nations[nation_idx].name, state.nations[x].name
            ));
        } else if old != DiplomaticStatus::Jihad as u8 && new == DiplomaticStatus::Jihad as u8 {
            news.push(format!(
                "nation {} announces a jihad with {}",
                state.nations[nation_idx].name, state.nations[x].name
            ));
        }
    }

    news
}

/// n_redes — NPC sector redesignation logic.
/// Matches C n_redes() exactly.
pub fn npc_redesignate_sector(
    sct: &mut Sector,
    nation: &Nation,
    nation_idx: usize,
    spread: &Spreadsheet,
    goldthresh: i32,
    metalthresh: i32,
    citythresh: i32,
    hunger: f32,
    rng: &mut ConquerRng,
) {
    let des = sct.designation;

    if des == Designation::Capitol as u8 || des == Designation::City as u8 {
        return;
    }

    let eat_rate = nation.eat_rate as f32 / 25.0;

    // Large enough for a city?
    if (sct.people > spread.civilians / CITYLIMIT
        || (spread.civilians < 30000 && sct.people > 1000))
        && hunger > eat_rate * 1.5
        && spread.in_city + spread.in_cap < spread.civilians * CITYPERCENT / 100
        && spread.sectors > 10
        && sct.trade_good == TradeGood::None as u8
    {
        sct.designation = Designation::Town as u8;
        return;
    }

    // Town with not enough food -> farm
    if des == Designation::Town as u8
        && hunger < eat_rate
        && tofood(sct, Some(nation)) > citythresh
    {
        sct.designation = Designation::Farm as u8;
        return;
    }

    // Too many in cities -> farm
    if des == Designation::Town as u8
        && spread.in_city + spread.in_cap > spread.civilians * CITYPERCENT / 66
    {
        sct.designation = Designation::Farm as u8;
        return;
    }

    // Non-city redesignation
    if des != Designation::Town as u8
        && des != Designation::City as u8
        && des != Designation::Capitol as u8
    {
        if sct.trade_good != TradeGood::None as u8 && tg_ok(nation, sct) {
            if metalthresh + goldthresh > 8
                || (sct.metal < metalthresh as u8 && sct.metal != 0)
                || (sct.jewels < goldthresh as u8 && sct.jewels != 0)
            {
                sct.designation = Designation::Farm as u8;
            } else {
                // Use trade good sector type
                let tg_idx = sct.trade_good as usize;
                let tg_bytes = TG_SECTOR_TYPE.as_bytes();
                if tg_idx < tg_bytes.len() {
                    let pref_char = tg_bytes[tg_idx];
                    let des_chars_bytes = DES_CHARS.as_bytes();
                    let mut found = false;
                    for (i, &dc) in des_chars_bytes.iter().enumerate() {
                        if dc == pref_char {
                            sct.designation = i as u8;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        sct.designation = Designation::Farm as u8;
                    }
                }
            }

            // Metal override
            if sct.metal < metalthresh as u8 && sct.metal != 0 {
                sct.designation = Designation::Blacksmith as u8;
            }

            // Invalid designation fix
            if sct.designation == Designation::Devastated as u8
                || (sct.designation == Designation::City as u8 && sct.people < 1000)
            {
                sct.designation = Designation::Farm as u8;
            }
        } else if tofood(sct, Some(nation)) >= 4 {
            sct.designation = Designation::Farm as u8;
        } else {
            sct.designation = Designation::Stockade as u8;
        }
    }

    // Farm with surplus food -> specialized
    if sct.designation == Designation::Farm as u8
        && hunger > eat_rate * 1.5
        && tofood(sct, Some(nation)) <= 6
    {
        if rng.rand() % 2 == 0 && nation.mine_ability < 30 {
            sct.designation = Designation::Blacksmith as u8;
        } else if rng.rand() % 2 == 0 && sct.people < 100 && nation.spoil_rate > 15 {
            sct.designation = Designation::Granary as u8;
        } else if rng.rand() % 2 == 0 && nation.popularity < 50 {
            sct.designation = Designation::Church as u8;
        } else if sct.people > 1000 {
            sct.designation = Designation::Town as u8;
        }
    }
}

/// redomil() — NPC military management (garrison, drafting, sizing, militia).
/// Matches C redomil() — simplified to just the key logic.
pub fn redo_military(
    state: &mut GameState,
    nation_idx: usize,
    peace: i32,
    range: &NpcRange,
    rng: &mut ConquerRng,
) {
    let nation = &state.nations[nation_idx];

    // Check ship crews
    for nvynum in 1..MAXNAVY {
        let nvy = &state.nations[nation_idx].navies[nvynum];
        if nvy.merchant != 0 || nvy.warships != 0 || nvy.galleys != 0 {
            if rng.rand() % 2 == 0 {
                state.nations[nation_idx].navies[nvynum].crew = SHIPCREW as u8;
            }
        }
    }

    // Calculate total military
    let mut tmil: i64 = 0;
    for armynum in 1..MAXARM {
        let army = &state.nations[nation_idx].armies[armynum];
        if army.soldiers > 0
            && army.unit_type < UnitType::MIN_LEADER
            && army.soldiers != UnitType::MILITIA.0 as i64
        // BUG-COMPAT: C compares P_ASOLD != A_MILITIA which is a type/value mismatch
        {
            tmil += army.soldiers;
        }
    }
    state.nations[nation_idx].total_mil = tmil;

    // Find garrison army (army 0 or first non-leader)
    let mut gar_army = 0usize;
    while gar_army < MAXARM {
        let army = &state.nations[nation_idx].armies[gar_army];
        if army.unit_type >= UnitType::MIN_LEADER && army.soldiers > 0 {
            gar_army += 1;
        } else {
            break;
        }
    }
    if gar_army >= MAXARM {
        gar_army = 0;
    }

    // Setup garrison
    let def_unit = defaultunit(&state.nations[nation_idx]);
    state.nations[nation_idx].armies[gar_army].unit_type = def_unit;
    state.nations[nation_idx].armies[gar_army].status = ArmyStatus::Garrison.to_value();
    state.nations[nation_idx].armies[gar_army].x = state.nations[nation_idx].cap_x;
    state.nations[nation_idx].armies[gar_army].y = state.nations[nation_idx].cap_y;

    // Ideal garrison = tmil * peace / (10 * MILINCAP)
    let mut ideal = tmil * peace as i64 / (10 * MILINCAP);
    if state.nations[nation_idx].treasury_gold < 0 {
        ideal /= 2;
    }

    let gar_sold = state.nations[nation_idx].armies[gar_army].soldiers;
    let unit_idx = UnitType(def_unit).stats_index().unwrap_or(0);
    let en_metal = UNIT_ENLIST_METAL.get(unit_idx).copied().unwrap_or(0) as i64;
    let en_cost = UNIT_ENLIST_COST.get(unit_idx).copied().unwrap_or(0) as i64;

    if gar_sold * 10 < 9 * ideal {
        // Too few garrison men
        let mut diff = ideal - gar_sold;
        if en_metal > 0 {
            diff = std::cmp::min(diff, state.nations[nation_idx].metals / en_metal);
        }
        let cap_x = state.nations[nation_idx].cap_x as usize;
        let cap_y = state.nations[nation_idx].cap_y as usize;
        diff = std::cmp::min(diff, state.sectors[cap_x][cap_y].people / 2);

        if (state.nations[nation_idx].treasury_gold < 0
            || state.nations[nation_idx].metals < 0)
            && diff > 0
        {
            diff = 0;
        }
        if state.sectors[cap_x][cap_y].owner as usize != nation_idx {
            diff = 0;
        }

        if diff > 0 {
            state.sectors[cap_x][cap_y].people -= diff;
            state.nations[nation_idx].armies[gar_army].soldiers += diff;
            state.nations[nation_idx].total_civ -= diff;
            state.nations[nation_idx].total_mil += diff;

            let has_warrior = Power::has_power(state.nations[nation_idx].powers, Power::WARRIOR);
            if has_warrior {
                state.nations[nation_idx].treasury_gold -= diff * en_cost / 2;
            } else {
                state.nations[nation_idx].treasury_gold -= diff * en_cost;
            }
            state.nations[nation_idx].metals -= diff * en_metal;
        }
    } else if gar_sold * 4 > 5 * ideal {
        // Too many garrison men — demobilize
        let diff = (4 * gar_sold - 5 * ideal) / 4;
        if diff > 0 {
            state.nations[nation_idx].armies[gar_army].soldiers -= diff;
            state.nations[nation_idx].total_mil -= diff;
            state.nations[nation_idx].total_civ += diff;
            let cap_x = state.nations[nation_idx].cap_x as usize;
            let cap_y = state.nations[nation_idx].cap_y as usize;
            state.sectors[cap_x][cap_y].people += diff;
            state.nations[nation_idx].metals += diff * en_metal;
            let has_warrior = Power::has_power(state.nations[nation_idx].powers, Power::WARRIOR);
            if has_warrior {
                state.nations[nation_idx].treasury_gold += diff * en_cost / 2;
            } else {
                state.nations[nation_idx].treasury_gold += diff * en_cost;
            }
        }
    }

    // Find and position leader
    let leader_type = getleader(state.nations[nation_idx].class) - 1;
    for armynum in 0..MAXARM {
        if state.nations[nation_idx].armies[armynum].unit_type == leader_type {
            state.nations[nation_idx].armies[armynum].status = ArmyStatus::Rule.to_value();
            state.nations[nation_idx].armies[armynum].x = state.nations[nation_idx].cap_x;
            state.nations[nation_idx].armies[armynum].y = state.nations[nation_idx].cap_y;
            break;
        }
    }

    // Draft new armies if under ideal
    let ideal_total = state.nations[nation_idx].total_civ * peace as i64 / (10 * MILRATIO);
    if state.nations[nation_idx].total_mil < 4 * ideal_total / 5 {
        for armynum in 1..MAXARM {
            if state.nations[nation_idx].armies[armynum].soldiers == 0 {
                let def_unit = defaultunit(&state.nations[nation_idx]);
                let unit_idx = UnitType(def_unit).stats_index().unwrap_or(0);
                let en_metal = UNIT_ENLIST_METAL.get(unit_idx).copied().unwrap_or(0) as i64;
                let en_cost = UNIT_ENLIST_COST.get(unit_idx).copied().unwrap_or(0) as i64;

                let mut new_sold = ideal_total - state.nations[nation_idx].total_mil;
                if en_metal > 0 {
                    new_sold = std::cmp::min(new_sold, state.nations[nation_idx].metals / en_metal);
                }
                let cap_x = state.nations[nation_idx].cap_x as usize;
                let cap_y = state.nations[nation_idx].cap_y as usize;
                new_sold = std::cmp::min(new_sold, state.sectors[cap_x][cap_y].people / 2);
                if en_cost > 0 {
                    new_sold = std::cmp::min(
                        new_sold,
                        state.nations[nation_idx].treasury_gold / en_cost,
                    );
                }

                if new_sold > 0 {
                    state.nations[nation_idx].metals -= new_sold * en_metal;
                    state.nations[nation_idx].armies[armynum].x = cap_x as u8;
                    state.nations[nation_idx].armies[armynum].y = cap_y as u8;
                    state.nations[nation_idx].armies[armynum].unit_type = def_unit;
                    state.nations[nation_idx].armies[armynum].soldiers = new_sold;
                    state.nations[nation_idx].armies[armynum].status =
                        ArmyStatus::Defend.to_value();
                    state.nations[nation_idx].armies[armynum].movement = 0;
                    state.nations[nation_idx].total_mil += new_sold;
                    state.nations[nation_idx].total_civ -= new_sold;
                    state.sectors[cap_x][cap_y].people -= new_sold;

                    let has_warrior =
                        Power::has_power(state.nations[nation_idx].powers, Power::WARRIOR);
                    if has_warrior {
                        state.nations[nation_idx].treasury_gold -= new_sold * en_cost / 2;
                    } else {
                        state.nations[nation_idx].treasury_gold -= new_sold * en_cost;
                    }
                }
                break; // Only draft one army per turn
            }
        }
    }

    // Militia in cities
    if state.nations[nation_idx].treasury_gold > 0 {
        for x in range.stx..range.endx {
            for y in range.sty..range.endy {
                if !state.on_map(x, y) {
                    continue;
                }
                let sct = &state.sectors[x as usize][y as usize];
                if sct.owner as usize != nation_idx {
                    continue;
                }
                let des = sct.designation;
                if des != Designation::Town as u8
                    && des != Designation::City as u8
                    && des != Designation::Capitol as u8
                {
                    continue;
                }

                // Check if militia already exists
                let mut has_militia = false;
                let mut militia_idx = None;
                for armynum in 0..MAXARM {
                    let army = &state.nations[nation_idx].armies[armynum];
                    if army.soldiers > 0
                        && army.x as i32 == x
                        && army.y as i32 == y
                        && army.unit_type == UnitType::MILITIA.0
                    {
                        let _ = has_militia; has_militia = true;
                        militia_idx = Some(armynum);
                        break;
                    }
                }

                if !has_militia {
                    // Find free army slot
                    for armynum in 0..MAXARM {
                        if state.nations[nation_idx].armies[armynum].soldiers == 0 {
                            state.nations[nation_idx].armies[armynum].x = x as u8;
                            state.nations[nation_idx].armies[armynum].y = y as u8;
                            state.nations[nation_idx].armies[armynum].unit_type =
                                UnitType::MILITIA.0;
                            militia_idx = Some(armynum);
                            let _ = has_militia; has_militia = true;
                            break;
                        }
                    }
                }

                if let Some(aidx) = militia_idx {
                    let ideal = std::cmp::max(sct.people / MILINCITY, 50);
                    let en_cost = UNIT_ENLIST_COST[UnitType::MILITIA.0 as usize] as i64;
                    let old_sold = state.nations[nation_idx].armies[aidx].soldiers;
                    let has_warrior =
                        Power::has_power(state.nations[nation_idx].powers, Power::WARRIOR);
                    if has_warrior {
                        state.nations[nation_idx].treasury_gold -= (ideal - old_sold) * en_cost / 2;
                    } else {
                        state.nations[nation_idx].treasury_gold -= (ideal - old_sold) * en_cost;
                    }
                    state.nations[nation_idx].armies[aidx].soldiers = ideal;
                    state.nations[nation_idx].armies[aidx].status = ArmyStatus::Militia.to_value();
                }
            }
        }
    }

    // Set default unit types
    let def_unit = defaultunit(&state.nations[nation_idx]);
    for armynum in 1..MAXARM {
        let army = &state.nations[nation_idx].armies[armynum];
        if army.soldiers > 0
            && army.unit_type != UnitType::MILITIA.0
            && army.unit_type < UnitType::MIN_LEADER
        {
            state.nations[nation_idx].armies[armynum].unit_type = def_unit;
        }
    }
}

/// Main NPC nation turn execution. Matches C nationrun() structure.
pub fn nation_run(
    state: &mut GameState,
    nation_idx: usize,
    rng: &mut ConquerRng,
) -> Vec<String> {
    let mut news = Vec::new();

    let range = npc_range(
        &state.nations[nation_idx],
        state.world.map_x as i32,
        state.world.map_y as i32,
    );

    // Fix capitol designation
    let cap_x = state.nations[nation_idx].cap_x as usize;
    let cap_y = state.nations[nation_idx].cap_y as usize;
    if state.sectors[cap_x][cap_y].owner as usize == nation_idx
        && state.sectors[cap_x][cap_y].designation != Designation::Capitol as u8
    {
        state.sectors[cap_x][cap_y].designation = Designation::Capitol as u8;
    }

    // Diplomacy
    let dip_news = get_diplomatic_status(state, nation_idx, rng);
    news.extend(dip_news);

    // T2: find_avg_sector() — compute global averages for attractiveness
    let avg = find_avg_sector(state, nation_idx);

    // Determine peace/war status
    let mut peace = 0i32;
    for i in 1..NTOTAL {
        let i_strat = NationStrategy::from_value(state.nations[i].active);
        if !i_strat.map_or(false, |s| s.is_nation()) {
            continue;
        }
        let d = state.nations[nation_idx].diplomacy[i] as i32;
        if d > peace {
            peace = d;
            if peace >= DiplomaticStatus::War as u8 as i32 {
                break;
            }
        }
    }

    let peace_val = if peace < DiplomaticStatus::War as u8 as i32 {
        8i32
    } else {
        12i32
    };

    // T20: Update move costs for this nation's race before movement
    let race = state.nations[nation_idx].race as char;
    movement::update_move_costs(state, race, nation_idx);

    // T1: Create attractiveness grid
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;
    let mut attr = create_attr_grid(map_x, map_y);

    let strat = NationStrategy::from_value(state.nations[nation_idx].active);

    // T15: Attacker/defender decision + attractiveness calculation
    if strat.map_or(false, |s| s.is_not_pc()) {
        if peace_val < 12 {
            // At peace: just expand
            pceattr(state, nation_idx, &range, &avg, &mut attr);
        } else {
            // At war: decide attack vs defend per enemy nation
            if state.nations[nation_idx].total_mil == 0 {
                defattr(state, nation_idx, &range, &avg, &mut attr);
            } else {
                for enemy in 1..NTOTAL {
                    let e_strat = NationStrategy::from_value(state.nations[enemy].active);
                    if !e_strat.map_or(false, |s| s.is_nation()) {
                        continue;
                    }
                    let their_status = state.nations[nation_idx].diplomacy[enemy];
                    if (their_status as i32) <= DiplomaticStatus::Hostile as u8 as i32 {
                        continue;
                    }
                    // Compare attack/defense strengths (C: tmil*(aplus+100))
                    let our_tmil = state.nations[nation_idx].total_mil as i64;
                    let our_aplus = state.nations[nation_idx].attack_plus as i64 + 100;
                    let their_tmil = state.nations[enemy].total_mil as i64;
                    let their_dplus = state.nations[enemy].defense_plus as i64 + 100;
                    let our_str = our_tmil * our_aplus;
                    let their_str = their_tmil * their_dplus;
                    let total = our_str + their_str;
                    let pct = if total > 0 { 100 * our_str / total } else { 50 };

                    if pct > rng.rand() as i64 % 100 {
                        // Attacker: set eligible armies to ATTACK
                        for armynum in 1..MAXARM {
                            let a = &state.nations[nation_idx].armies[armynum];
                            if a.soldiers > 0
                                && a.unit_type != UnitType::MILITIA.0
                                && a.status != ArmyStatus::OnBoard.to_value()
                                && a.status != ArmyStatus::Traded.to_value()
                                && a.status < NUMSTATUS
                                && a.status != ArmyStatus::General.to_value()
                            {
                                state.nations[nation_idx].armies[armynum].status =
                                    ArmyStatus::Attack.to_value();
                            }
                        }
                        atkattr(state, nation_idx, &range, &avg, &mut attr);
                    } else {
                        // Defender: small armies defend, big ones attack
                        for armynum in 1..MAXARM {
                            let a = &state.nations[nation_idx].armies[armynum];
                            if a.soldiers > 0
                                && a.unit_type != UnitType::MILITIA.0
                                && a.status != ArmyStatus::OnBoard.to_value()
                                && a.status != ArmyStatus::Traded.to_value()
                                && a.status < NUMSTATUS
                                && a.status != ArmyStatus::General.to_value()
                            {
                                let new_status = if a.soldiers < 350 {
                                    ArmyStatus::Defend.to_value()
                                } else {
                                    ArmyStatus::Attack.to_value()
                                };
                                state.nations[nation_idx].armies[armynum].status = new_status;
                            }
                        }
                        defattr(state, nation_idx, &range, &avg, &mut attr);
                    }
                }
            }
        }
    }

    // T16: Move infantry, then leaders/monsters
    let mut loop_count = 0i32;
    n_people(state, nation_idx, &range, true, &mut attr);

    for armynum in 1..MAXARM {
        if state.nations[nation_idx].armies[armynum].soldiers > 0
            && state.nations[nation_idx].armies[armynum].unit_type < UnitType::MIN_LEADER
        {
            loop_count +=
                movement::npc_army_move(state, nation_idx, armynum, &attr, rng);
        }
    }

    n_people(state, nation_idx, &range, false, &mut attr);

    for armynum in 1..MAXARM {
        if state.nations[nation_idx].armies[armynum].soldiers > 0
            && state.nations[nation_idx].armies[armynum].unit_type >= UnitType::MIN_LEADER
        {
            loop_count +=
                movement::npc_army_move(state, nation_idx, armynum, &attr, rng);
        }
    }

    // T17: Update NPC active status based on movement count
    let active_strat = NationStrategy::from_value(state.nations[nation_idx].active);
    if active_strat.map_or(false, |s| s.is_npc())
        && active_strat != Some(NationStrategy::Isolationist)
    {
        let new_active = if active_strat.map_or(false, |s| s.is_good()) {
            if loop_count <= 1 { NationStrategy::Good0Free as u8 }
            else if loop_count >= 6 { NationStrategy::Good6Free as u8 }
            else if loop_count >= 4 { NationStrategy::Good4Free as u8 }
            else { NationStrategy::Good2Free as u8 }
        } else if active_strat.map_or(false, |s| s.is_neutral()) {
            if loop_count <= 1 { NationStrategy::Neutral0Free as u8 }
            else if loop_count >= 6 { NationStrategy::Neutral6Free as u8 }
            else if loop_count >= 4 { NationStrategy::Neutral4Free as u8 }
            else { NationStrategy::Neutral2Free as u8 }
        } else if active_strat.map_or(false, |s| s.is_evil()) {
            if loop_count <= 1 { NationStrategy::Evil0Free as u8 }
            else if loop_count >= 6 { NationStrategy::Evil6Free as u8 }
            else if loop_count >= 4 { NationStrategy::Evil4Free as u8 }
            else { NationStrategy::Evil2Free as u8 }
        } else {
            state.nations[nation_idx].active // unchanged for monsters/etc
        };
        state.nations[nation_idx].active = new_active;
    }

    // Charity
    if state.nations[nation_idx].treasury_gold > state.nations[nation_idx].total_civ {
        state.nations[nation_idx].charity = 10;
    } else {
        state.nations[nation_idx].charity = 0;
    }

    // Tax rate
    let strat = NationStrategy::from_value(state.nations[nation_idx].active);
    if strat.map_or(false, |s| s.is_not_pc()) {
        if state.nations[nation_idx].total_sectors < 20
            || state.nations[nation_idx].score < 20
        {
            if state.nations[nation_idx].tax_rate < 10 {
                state.nations[nation_idx].tax_rate = 10;
            }
        } else {
            let prestige = state.nations[nation_idx].prestige as i32;
            let pop = state.nations[nation_idx].popularity as i32;
            let terror = state.nations[nation_idx].terror as i32;
            let charity = state.nations[nation_idx].charity as i32;

            let rate1 = prestige / 5;
            let rate2 = (pop + terror + 3 * charity) / 10;
            let mut rate = std::cmp::min(rate1, rate2);
            rate = std::cmp::min(rate, 20);
            if rate < 4 {
                rate = 4;
            }
            state.nations[nation_idx].tax_rate = rate as u8;
        }
    }

    // Military management
    if strat.map_or(false, |s| s.is_not_pc()) {
        redo_military(state, nation_idx, peace_val, &range, rng);
    }

    // Buy magic/weapons
    let powers = magic::npc_buy_magic(state, nation_idx, rng);
    for p in &powers {
        news.push(format!(
            "nation {} gets power {:?}",
            state.nations[nation_idx].name, p
        ));
    }
    magic::npc_buy_weapons(state, nation_idx, rng);

    // Fort building
    let strat = NationStrategy::from_value(state.nations[nation_idx].active);
    if strat.map_or(false, |s| s.is_not_pc()) {
        for x in range.stx..range.endx {
            for y in range.sty..range.endy {
                if !state.on_map(x, y) {
                    continue;
                }
                let sct = &state.sectors[x as usize][y as usize];
                if sct.owner as usize != nation_idx {
                    continue;
                }
                let des = sct.designation;
                if (des == Designation::Town as u8
                    || des == Designation::City as u8
                    || des == Designation::Capitol as u8
                    || des == Designation::Fort as u8)
                    && sct.fortress < 10
                    && state.nations[nation_idx].treasury_gold > 10000
                    && rng.rand() % 5 == 0
                    && (sct.fortress as i64) < sct.people % 1000
                {
                    state.sectors[x as usize][y as usize].fortress += 1;
                }
            }
        }
    }

    // T18: Don't allow ATTACK status from own fortified city
    for armynum in 0..MAXARM {
        if state.nations[nation_idx].armies[armynum].soldiers <= 0 {
            continue;
        }
        if state.nations[nation_idx].armies[armynum].status != ArmyStatus::Attack.to_value() {
            continue;
        }
        let ax = state.nations[nation_idx].armies[armynum].x as usize;
        let ay = state.nations[nation_idx].armies[armynum].y as usize;
        let sct_owner = state.sectors[ax][ay].owner as usize;
        let nation_powers = state.nations[nation_idx].powers;
        let fv = fort_val(&state.sectors[ax][ay], nation_powers);
        if sct_owner == nation_idx && fv > 0 {
            state.nations[nation_idx].armies[armynum].status = if rng.rand() % 2 == 0 {
                ArmyStatus::Defend.to_value()
            } else {
                ArmyStatus::Garrison.to_value()
            };
        }
    }

    news
}

// ============================================================
// T1: Attractiveness grid infrastructure
// ============================================================

/// Create a zero-initialized attractiveness grid matching map dimensions.
pub fn create_attr_grid(map_x: usize, map_y: usize) -> Vec<Vec<i32>> {
    vec![vec![0i32; map_y]; map_x]
}

/// Zero out an existing attractiveness grid (reuse allocation).
pub fn clear_attr_grid(attr: &mut Vec<Vec<i32>>) {
    for row in attr.iter_mut() {
        for v in row.iter_mut() {
            *v = 0;
        }
    }
}

// ============================================================
// T2: NPC averages — find_avg_sector()
// ============================================================

/// Pre-computed world averages used by attractiveness functions.
/// C: original/npc.c static Avg_food, Avg_tradegood, Avg_soldiers[]
pub struct NpcAverages {
    pub avg_food: i32,
    pub avg_tradegood: i32,
    pub avg_soldiers: [i64; NTOTAL],
}

/// find_avg_sector() — calculates global average food/tradegood values
/// and per-nation average soldiers per occupied sector.
/// C: original/npc.c:980
pub fn find_avg_sector(state: &GameState, nation_idx: usize) -> NpcAverages {
    let map_x = state.world.map_x as usize;
    let map_y = state.world.map_y as usize;
    let mut total_food: i64 = 0;
    let mut total_tg: i64 = 0;
    let mut useable_land: i64 = 0;
    let nation = &state.nations[nation_idx];

    for x in 0..map_x {
        for y in 0..map_y {
            let sct = &state.sectors[x][y];
            if sct.altitude != Altitude::Water as u8 && sct.altitude != Altitude::Peak as u8 {
                useable_land += 1;
                total_food += tofood(sct, Some(nation)) as i64;
                if sct.trade_good != TradeGood::None as u8 {
                    if sct.metal != 0 {
                        total_tg += 500;
                    } else if sct.jewels != 0 {
                        total_tg += 500;
                    } else {
                        total_tg += 300;
                    }
                }
            }
        }
    }

    let (avg_food, avg_tradegood) = if useable_land > 0 {
        (
            (total_food / useable_land) as i32,
            (total_tg / useable_land) as i32,
        )
    } else {
        (0, 0)
    };

    let mut avg_soldiers = [0i64; NTOTAL];
    for n in 1..NTOTAL {
        let n_strat = NationStrategy::from_value(state.nations[n].active);
        if !n_strat.map_or(false, |s| s.is_nation()) {
            continue;
        }
        // Count unique sectors occupied by nation n's armies
        let mut total_sectors: i64 = 0;
        'army_loop: for armynum in 1..MAXARM {
            let arm = &state.nations[n].armies[armynum];
            if arm.soldiers <= 0 {
                continue;
            }
            let ax = arm.x;
            let ay = arm.y;
            // Ensure we only count each unique sector once
            for i in 1..armynum {
                let prev = &state.nations[n].armies[i];
                if prev.soldiers > 0 && prev.x == ax && prev.y == ay {
                    continue 'army_loop;
                }
            }
            total_sectors += 1;
        }
        if total_sectors > 0 {
            avg_soldiers[n] = state.nations[n].total_mil / total_sectors;
        }
    }

    NpcAverages { avg_food, avg_tradegood, avg_soldiers }
}

// ============================================================
// T3: n_unowned() — attract toward unowned land
// ============================================================

/// n_unowned() — score sectors for unowned/unclaimed land.
/// C: original/npc.c:1355
fn n_unowned(
    state: &GameState,
    nation_idx: usize,
    range: &NpcRange,
    avg: &NpcAverages,
    attr: &mut Vec<Vec<i32>>,
) {
    let cap_x = state.nations[nation_idx].cap_x as i32;
    let cap_y = state.nations[nation_idx].cap_y as i32;
    let nation = &state.nations[nation_idx];

    // Around capitol (within 4): +450 for unowned
    for x in (cap_x - 4)..=(cap_x + 4) {
        for y in (cap_y - 4)..=(cap_y + 4) {
            if state.on_map(x, y) && state.sectors[x as usize][y as usize].owner == 0 {
                attr[x as usize][y as usize] += 450;
            }
        }
    }

    // Range scan
    for x in range.stx..range.endx {
        for y in range.sty..range.endy {
            if !state.on_map(x, y) {
                continue;
            }
            let sct = &state.sectors[x as usize][y as usize];

            // Trade goods (always visible in our simplified version)
            if sct.trade_good != TradeGood::None as u8 {
                if sct.metal != 0 {
                    attr[x as usize][y as usize] += 500;
                } else if sct.jewels != 0 {
                    attr[x as usize][y as usize] += 500;
                } else {
                    attr[x as usize][y as usize] += 300;
                }
            }

            // Unowned / nomad land
            let owner = sct.owner as usize;
            if owner == 0 {
                attr[x as usize][y as usize] += 300;
            } else {
                let o_strat = NationStrategy::from_value(state.nations[owner].active);
                if o_strat == Some(NationStrategy::NpcNomad) {
                    attr[x as usize][y as usize] += 100;
                }
            }

            // Food value (visible: use actual; else: avg)
            attr[x as usize][y as usize] += 50 * tofood(sct, Some(nation));

            // Not habitable: divide by 5
            if !is_habitable(sct) {
                attr[x as usize][y as usize] /= 5;
            }
        }
    }
}

// ============================================================
// T4: n_trespass() — avoid foreign territory
// ============================================================

/// n_trespass() — set attr=1 for foreign non-allied sectors we're not at war with.
/// C: original/npc.c:1327
fn n_trespass(
    state: &GameState,
    nation_idx: usize,
    range: &NpcRange,
    attr: &mut Vec<Vec<i32>>,
) {
    let cap_x = state.nations[nation_idx].cap_x as i32;
    let cap_y = state.nations[nation_idx].cap_y as i32;

    for x in range.stx..range.endx {
        for y in range.sty..range.endy {
            if !state.on_map(x, y) {
                continue;
            }
            let owner = state.sectors[x as usize][y as usize].owner as usize;
            if owner == nation_idx || owner == 0 {
                continue;
            }
            // More than 2 from capitol in both axes
            if (x - cap_x).abs() <= 2 || (y - cap_y).abs() <= 2 {
                continue;
            }
            let our_status = state.nations[nation_idx].diplomacy[owner];
            let their_status = state.nations[owner].diplomacy[nation_idx];
            // Not at war with owner, not allied (status > ALLIED)
            if our_status >= DiplomaticStatus::War as u8 {
                continue;
            }
            if their_status >= DiplomaticStatus::War as u8 {
                continue;
            }
            if our_status <= DiplomaticStatus::Allied as u8 {
                continue;
            }
            attr[x as usize][y as usize] = 1;
        }
    }
}

// ============================================================
// T5: n_toofar() — stay near capitol
// ============================================================

/// n_toofar() — set attr=1 for all sectors outside NPC operating range.
/// C: original/npc.c:1344
fn n_toofar(
    state: &GameState,
    range: &NpcRange,
    attr: &mut Vec<Vec<i32>>,
) {
    let map_x = state.world.map_x as i32;
    let map_y = state.world.map_y as i32;
    for x in 0..map_x {
        for y in 0..map_y {
            if x < range.stx || y < range.sty || x >= range.endx || y >= range.endy {
                attr[x as usize][y as usize] = 1;
            }
        }
    }
}

// ============================================================
// T6: n_people() — population attractiveness
// ============================================================

/// n_people() — add or subtract people/4 to owned habitable sectors.
/// Called with doadd=true before infantry moves, false after (before leaders).
/// C: original/npc.c:1527
fn n_people(
    state: &GameState,
    nation_idx: usize,
    range: &NpcRange,
    doadd: bool,
    attr: &mut Vec<Vec<i32>>,
) {
    for x in range.stx..range.endx {
        for y in range.sty..range.endy {
            if !state.on_map(x, y) {
                continue;
            }
            let sct = &state.sectors[x as usize][y as usize];
            if sct.owner as usize == nation_idx && is_habitable(sct) {
                let delta = (sct.people / 4) as i32;
                if doadd {
                    attr[x as usize][y as usize] += delta;
                } else {
                    attr[x as usize][y as usize] -= delta;
                }
            }
        }
    }
}

// ============================================================
// T7: n_survive() — capitol defense urgency
// ============================================================

/// n_survive() — add urgency to defend capitol if under threat.
/// C: original/npc.c:1579
fn n_survive(
    state: &GameState,
    nation_idx: usize,
    avg: &NpcAverages,
    attr: &mut Vec<Vec<i32>>,
) {
    let cap_x = state.nations[nation_idx].cap_x as usize;
    let cap_y = state.nations[nation_idx].cap_y as usize;

    // If we've lost our capitol, urgently reclaim it
    if state.sectors[cap_x][cap_y].owner as usize != nation_idx {
        attr[cap_x][cap_y] = 1000;
    }

    // Defend against nearby war enemies
    for nation in 1..NTOTAL {
        let n_strat = NationStrategy::from_value(state.nations[nation].active);
        if !n_strat.map_or(false, |s| s.is_nation()) {
            continue;
        }
        let our_status = state.nations[nation].diplomacy[nation_idx];
        let their_status = state.nations[nation_idx].diplomacy[nation];
        if our_status < DiplomaticStatus::War as u8
            && their_status < DiplomaticStatus::War as u8
        {
            continue;
        }

        let cx = cap_x as i32;
        let cy = cap_y as i32;

        // Count armies visible (simplified: always visible)
        let mut seen: Vec<(i32, i32)> = Vec::new();
        for armynum in 1..MAXARM {
            let arm = &state.nations[nation].armies[armynum];
            if arm.soldiers <= 0 {
                continue;
            }
            let ax = arm.x as i32;
            let ay = arm.y as i32;
            if ax < cx - 2 || ax > cx + 2 || ay < cy - 2 || ay > cy + 2 {
                continue;
            }
            // Deduplicate by sector
            if seen.iter().any(|&(sx, sy)| sx == ax && sy == ay) {
                continue;
            }
            seen.push((ax, ay));

            let soldiers = arm.soldiers as i32;
            if ax == cx as i32 && ay == cy as i32 {
                attr[cap_x][cap_y] += 2 * soldiers;
            } else {
                attr[ax as usize][ay as usize] += soldiers;
            }
        }
    }
}

// ============================================================
// T8: n_defend() — defensive sector scoring
// ============================================================

/// n_defend() — score own sectors for defensive value against enemy nation.
/// C: original/npc.c:1471
fn n_defend(
    state: &GameState,
    nation_idx: usize,
    enemy: usize,
    range: &NpcRange,
    avg: &NpcAverages,
    attr: &mut Vec<Vec<i32>>,
) {
    // Add 1/10th of enemy soldiers in our sectors
    for armynum in 1..MAXARM {
        let arm = &state.nations[enemy].armies[armynum];
        if arm.soldiers <= 0 {
            continue;
        }
        let ax = arm.x as usize;
        let ay = arm.y as usize;
        if state.sectors[ax][ay].owner as usize == nation_idx {
            attr[ax][ay] += (arm.soldiers / 10) as i32;
        }
    }

    // +80 near capitol (C bug: x loop uses capy+1 instead of capx+1 — port faithfully)
    let cap_x = state.nations[nation_idx].cap_x as i32;
    let cap_y = state.nations[nation_idx].cap_y as i32;
    for x in (cap_x - 1)..=(cap_y + 1) {
        for y in (cap_y - 1)..=(cap_y + 1) {
            if state.on_map(x, y) {
                attr[x as usize][y as usize] += 80;
            }
        }
    }

    // Movement cost and population scoring
    let total_civ = state.nations[nation_idx].total_civ;
    for x in range.stx..range.endx {
        for y in range.sty..range.endy {
            if !state.on_map(x, y) {
                continue;
            }
            let mc = state.move_cost[x as usize][y as usize];
            if mc == 1 {
                attr[x as usize][y as usize] += 50;
            } else if mc <= 3 {
                attr[x as usize][y as usize] += 20;
            } else if mc <= 5 {
                attr[x as usize][y as usize] += 10;
            }

            let sct = &state.sectors[x as usize][y as usize];
            if sct.owner as usize == nation_idx {
                let des = sct.designation;
                if des == Designation::Town as u8
                    || des == Designation::City as u8
                    || des == Designation::Capitol as u8
                {
                    attr[x as usize][y as usize] += 50;
                }
                if total_civ > 0 {
                    attr[x as usize][y as usize] +=
                        (3000 * sct.people / total_civ) as i32;
                }
            }
        }
    }
}

// ============================================================
// T9: n_attack() — offensive targeting
// ============================================================

/// n_attack() — score enemy cities as attack targets.
/// C: original/npc.c:1510
fn n_attack(
    state: &GameState,
    nation_idx: usize,
    enemy: usize,
    range: &NpcRange,
    avg: &NpcAverages,
    attr: &mut Vec<Vec<i32>>,
) {
    for x in range.stx..range.endx {
        for y in range.sty..range.endy {
            if !state.on_map(x, y) {
                continue;
            }
            if state.sectors[x as usize][y as usize].owner as usize != enemy {
                continue;
            }

            let des = state.sectors[x as usize][y as usize].designation;
            if des == Designation::City as u8
                || des == Designation::Capitol as u8
                || des == Designation::Town as u8
            {
                // Count our soldiers within 1 of this sector
                let mut our_solds: i64 = 0;
                for armynum in 1..MAXARM {
                    let arm = &state.nations[nation_idx].armies[armynum];
                    if arm.soldiers <= 0 {
                        continue;
                    }
                    let ax = arm.x as i32;
                    let ay = arm.y as i32;
                    if (ax - x).abs() <= 1 && (ay - y).abs() <= 1 {
                        our_solds += arm.soldiers;
                    }
                }

                // Visible: check actual enemy strength
                let enemy_solds = solds_in_sector(&state.nations[enemy], x as u8, y as u8);
                if enemy_solds * 2 < 3 * our_solds {
                    attr[x as usize][y as usize] += 500;
                }
            } else {
                // Unseen: give some value (UNS_CITY_VALUE = 10)
                attr[x as usize][y as usize] += 10;
            }
        }
    }
}

// ============================================================
// T10: n_undefended() — target empty enemy sectors
// ============================================================

/// n_undefended() — score undefended enemy sectors.
/// C: original/npc.c:1542
fn n_undefended(
    state: &GameState,
    nation_idx: usize,
    enemy: usize,
    range: &NpcRange,
    attr: &mut Vec<Vec<i32>>,
) {
    for x in range.stx..range.endx {
        for y in range.sty..range.endy {
            if !state.on_map(x, y) {
                continue;
            }
            let sct = &state.sectors[x as usize][y as usize];
            if sct.owner as usize != enemy {
                continue;
            }
            if !is_habitable(sct) {
                attr[x as usize][y as usize] += 30;
            } else if state.occupied[x as usize][y as usize] == 0 {
                attr[x as usize][y as usize] += 100;
            } else {
                attr[x as usize][y as usize] += 60;
            }
        }
    }
}

// ============================================================
// T11: n_between() — strategic blocking
// ============================================================

/// n_between() — +60 for sectors in bounding box between two capitols.
/// C: original/npc.c:1544
fn n_between(
    state: &GameState,
    nation_idx: usize,
    enemy: usize,
    attr: &mut Vec<Vec<i32>>,
) {
    // Always visible in simplified version
    let my_cap_x = state.nations[nation_idx].cap_x as i32;
    let my_cap_y = state.nations[nation_idx].cap_y as i32;
    let en_cap_x = state.nations[enemy].cap_x as i32;
    let en_cap_y = state.nations[enemy].cap_y as i32;

    let x1 = my_cap_x.min(en_cap_x);
    let x2 = my_cap_x.max(en_cap_x);
    let y1 = my_cap_y.min(en_cap_y);
    let y2 = my_cap_y.max(en_cap_y);

    for x in x1..=x2 {
        for y in y1..=y2 {
            if state.on_map(x, y) {
                attr[x as usize][y as usize] += 60;
            }
        }
    }
}

// ============================================================
// T12: pceattr() — peacetime attractiveness
// ============================================================

/// pceattr() — calculate peacetime sector attractiveness.
/// C: original/npc.c:1708 — n_unowned() ×3, n_trespass(), n_toofar(), n_survive()
fn pceattr(
    state: &GameState,
    nation_idx: usize,
    range: &NpcRange,
    avg: &NpcAverages,
    attr: &mut Vec<Vec<i32>>,
) {
    n_unowned(state, nation_idx, range, avg, attr);
    n_unowned(state, nation_idx, range, avg, attr);
    n_unowned(state, nation_idx, range, avg, attr);
    n_trespass(state, nation_idx, range, attr);
    n_toofar(state, range, attr);
    n_survive(state, nation_idx, avg, attr);
}

// ============================================================
// T13: atkattr() — attack attractiveness
// ============================================================

/// atkattr() — calculate attack-mode sector attractiveness.
/// C: original/npc.c:1674
fn atkattr(
    state: &GameState,
    nation_idx: usize,
    range: &NpcRange,
    avg: &NpcAverages,
    attr: &mut Vec<Vec<i32>>,
) {
    n_unowned(state, nation_idx, range, avg, attr);

    for nation in 1..NTOTAL {
        let n_strat = NationStrategy::from_value(state.nations[nation].active);
        if !n_strat.map_or(false, |s| s.is_nation()) {
            continue;
        }
        let our_status = state.nations[nation_idx].diplomacy[nation];
        if our_status == DiplomaticStatus::War as u8 {
            n_between(state, nation_idx, nation, attr);
            n_undefended(state, nation_idx, nation, range, attr);
            n_attack(state, nation_idx, nation, range, avg, attr);
        } else if our_status == DiplomaticStatus::Jihad as u8 {
            // ×4 attack, ×2 between, ×2 undefended (C calls them 4, 2, 2 times)
            n_attack(state, nation_idx, nation, range, avg, attr);
            n_attack(state, nation_idx, nation, range, avg, attr);
            n_between(state, nation_idx, nation, attr);
            n_undefended(state, nation_idx, nation, range, attr);
            n_attack(state, nation_idx, nation, range, avg, attr);
            n_between(state, nation_idx, nation, attr);
            n_undefended(state, nation_idx, nation, range, attr);
            n_attack(state, nation_idx, nation, range, avg, attr);
        }
    }

    n_toofar(state, range, attr);
    n_trespass(state, nation_idx, range, attr);
    n_survive(state, nation_idx, avg, attr);
}

// ============================================================
// T14: defattr() — defensive attractiveness
// ============================================================

/// defattr() — calculate defensive-mode sector attractiveness.
/// C: original/npc.c:1650
fn defattr(
    state: &GameState,
    nation_idx: usize,
    range: &NpcRange,
    avg: &NpcAverages,
    attr: &mut Vec<Vec<i32>>,
) {
    n_unowned(state, nation_idx, range, avg, attr);

    for nation in 1..NTOTAL {
        let n_strat = NationStrategy::from_value(state.nations[nation].active);
        if !n_strat.map_or(false, |s| s.is_nation()) {
            continue;
        }
        if state.nations[nation_idx].diplomacy[nation] >= DiplomaticStatus::War as u8 {
            n_defend(state, nation_idx, nation, range, avg, attr);
            n_between(state, nation_idx, nation, attr);
            n_undefended(state, nation_idx, nation, range, attr);
        }
    }

    n_trespass(state, nation_idx, range, attr);
    n_toofar(state, range, attr);
    n_survive(state, nation_idx, avg, attr);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npc_range() {
        let mut nation = Nation::default();
        nation.cap_x = 20;
        nation.cap_y = 20;
        nation.active = NationStrategy::Good4Free as u8;

        let range = npc_range(&nation, 40, 40);
        assert_eq!(range.stx, 5); // 20 - 15
        assert_eq!(range.sty, 5);
        assert_eq!(range.endx, 35); // 20 + 15
        assert_eq!(range.endy, 35);
    }

    #[test]
    fn test_npc_range_pc() {
        let mut nation = Nation::default();
        nation.active = NationStrategy::PcGood as u8;

        let range = npc_range(&nation, 40, 40);
        assert_eq!(range.stx, 0);
        assert_eq!(range.sty, 0);
        assert_eq!(range.endx, 40);
        assert_eq!(range.endy, 40);
    }

    #[test]
    fn test_new_diplomacy_orc_vs_human() {
        let mut state = GameState::new(16, 16);
        state.nations[1].race = 'O';
        state.nations[1].active = NationStrategy::Evil4Free as u8;
        state.nations[2].race = 'H';
        state.nations[2].active = NationStrategy::Good4Free as u8;
        state.nations[1].diplomacy[2] = DiplomaticStatus::Unmet as u8;

        let mut rng = ConquerRng::new(42);
        new_diplomacy(&mut state, 1, 2, &mut rng);

        // Orc vs non-orc: should be Hostile or War
        let d = state.nations[1].diplomacy[2];
        assert!(
            d == DiplomaticStatus::Hostile as u8 || d == DiplomaticStatus::War as u8,
            "Orc diplomacy should be hostile or war, got {}",
            d
        );
    }

    #[test]
    fn test_new_diplomacy_same_race() {
        let mut state = GameState::new(16, 16);
        state.nations[1].race = 'H';
        state.nations[1].active = NationStrategy::Good4Free as u8;
        state.nations[2].race = 'H';
        state.nations[2].active = NationStrategy::Good4Free as u8;

        let mut rng = ConquerRng::new(42);
        new_diplomacy(&mut state, 1, 2, &mut rng);

        // Same race: should be Friendly or Neutral
        let d = state.nations[1].diplomacy[2];
        assert!(
            d == DiplomaticStatus::Friendly as u8 || d == DiplomaticStatus::Neutral as u8,
            "Same race diplomacy should be friendly/neutral, got {}",
            d
        );
    }
}
