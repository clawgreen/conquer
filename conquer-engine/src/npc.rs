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
        8
    } else {
        12
    };

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

        // Charity
        if state.nations[nation_idx].treasury_gold > state.nations[nation_idx].total_civ {
            state.nations[nation_idx].charity = 10;
        } else {
            state.nations[nation_idx].charity = 0;
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

    news
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
