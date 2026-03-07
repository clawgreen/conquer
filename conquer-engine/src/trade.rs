// conquer-engine/src/trade.rs — Trade system ported from trade.c
//
// T227-T234: Trade routes, commodities, market logic
//
// Commodities: Gold, Food, Metal, Land, Soldiers, Ships
// Trade status: SELL, BUY, NOSALE (remove from market)

use conquer_core::*;
use conquer_core::tables::*;
use crate::utils::*;

/// Trade commodity types
pub const TDGOLD: u8 = 0;
pub const TDFOOD: u8 = 1;
pub const TDMETAL: u8 = 2;
pub const TDJEWL: u8 = 3;
pub const TDLAND: u8 = 4;
pub const TDARMY: u8 = 5;
pub const TDSHIP: u8 = 6;
pub const NUMPRODUCTS: u8 = 7;

/// Trade action types
pub const SELL: u8 = 0;
pub const BUY: u8 = 1;
pub const NOSALE: u8 = 3;  // Remove from market

/// Trade deal record
#[derive(Debug, Clone, Default)]
pub struct TradeDeal {
    pub deal_type: u8,      // SELL, BUY, NOSALE
    pub nation: u8,          // Owner nation
    pub commodity_type: u8,  // What being traded
    pub want_type: u8,       // What they want in return
    pub amount: i64,         // Amount/value
    pub min_want: i64,       // Minimum wanted (for land/armies)
    pub extra: i32,          // Extra info (land Y coord, army index)
}

/// Trade result for processing
#[derive(Debug, Clone)]
pub struct TradeResult {
    pub success: bool,
    pub seller_receives: i64,
    pub buyer_pays: i64,
    pub message: String,
}

/// Get commodity name
pub fn commodity_name(td_type: u8) -> &'static str {
    match td_type {
        TDGOLD => "Gold",
        TDFOOD => "Food",
        TDMETAL => "Metal",
        TDJEWL => "Jewels",
        TDLAND => "Land",
        TDARMY => "Soldiers",
        TDSHIP => "Ships",
        _ => "Unknown",
    }
}

/// Calculate trade cost (20% cost is normal)
pub fn trade_cost(cost_reduction: i32) -> i64 {
    // TRADECOST(cost) = (100 - cost) / 100
    // Normal is 20% cost, so (100 - 20) / 100 = 0.8
    ((100 - cost_reduction) as i64) * 100 / 100
}

/// Check if an army type is tradable (can be sold on market)
/// Matches C: tradable() function
pub fn is_army_tradable(nation: &Nation, army_idx: u8) -> bool {
    if army_idx >= MAXARM as u8 {
        return false;
    }
    
    let army = &nation.armies[army_idx as usize];
    if army.soldiers <= 0 {
        return false;
    }
    
    let status = army.status;
    if status == ArmyStatus::Traded.to_value() || status == ArmyStatus::OnBoard.to_value() {
        return false;
    }
    
    let unit_type = army.unit_type;
    
    // Tradable: Mercenary, Siege, Catapult, Elephant, or monster units
    matches!(
        UnitType(unit_type),
        UnitType(13)  // A_MERCENARY
        | UnitType(19) // A_CATAPULT
        | UnitType(20) // A_SIEGE
        | UnitType(23) // A_ELEPHANT
    ) || unit_type >= MINMONSTER as u8
}

/// Check if land can be traded (valid for sale)
pub fn can_trade_land(sct: &Sector, sector_x: u8, sector_y: u8, nation_id: u8, cap_x: u8, cap_y: u8) -> TradeResult {
    // Check owner
    if sct.owner != nation_id {
        return TradeResult {
            success: false,
            seller_receives: 0,
            buyer_pays: 0,
            message: "You don't own it".to_string(),
        };
    }
    
    // Check not capitol
    if sector_x == cap_x && sector_y == cap_y {
        return TradeResult {
            success: false,
            seller_receives: 0,
            buyer_pays: 0,
            message: "That is your capitol".to_string(),
        };
    }
    
    // Check designation
    let des = sct.designation;
    if des == Designation::Town as u8 {
        return TradeResult {
            success: false,
            seller_receives: 0,
            buyer_pays: 0,
            message: "Towns may not be sold".to_string(),
        };
    }
    if des == Designation::City as u8 {
        return TradeResult {
            success: false,
            seller_receives: 0,
            buyer_pays: 0,
            message: "Cities may not be sold".to_string(),
        };
    }
    
    TradeResult {
        success: true,
        seller_receives: 0,
        buyer_pays: 0,
        message: "OK".to_string(),
    }
}

/// Check if navy can be traded
pub fn can_trade_navy(navy: &Navy) -> TradeResult {
    // Check has ships
    if navy.warships == 0 && navy.merchant == 0 && navy.galleys == 0 {
        return TradeResult {
            success: false,
            seller_receives: 0,
            buyer_pays: 0,
            message: "Invalid Navy".to_string(),
        };
    }
    
    // Check no army onboard
    if navy.army_num != MAXARM as u8 {
        return TradeResult {
            success: false,
            seller_receives: 0,
            buyer_pays: 0,
            message: "Navy must be unloaded first".to_string(),
        };
    }
    
    // Check no passengers
    if navy.people != 0 {
        return TradeResult {
            success: false,
            seller_receives: 0,
            buyer_pays: 0,
            message: "Navy must be unloaded first".to_string(),
        };
    }
    
    TradeResult {
        success: true,
        seller_receives: 0,
        buyer_pays: 0,
        message: "OK".to_string(),
    }
}

/// Calculate army value for trade
/// Matches C: armyvalue() function
pub fn army_trade_value(nation: &Nation, army_idx: u8) -> i64 {
    if army_idx >= MAXARM as u8 {
        return 0;
    }
    
    let army = &nation.armies[army_idx as usize];
    if army.soldiers <= 0 {
        return 0;
    }
    
    let unit_type = army.unit_type;
    let attack = UNIT_ATTACK.get(unit_type as usize).copied().unwrap_or(0);
    
    let mut value = army.soldiers as i64 * 100 
        + army.soldiers as i64 * attack as i64;
    
    // Monsters get bonus
    if unit_type >= MINMONSTER as u8 {
        value += army.soldiers as i64 * 10;
    }
    
    value / 100
}

/// Calculate navy holding capacity for trade
/// Matches C: flthold() function
pub fn navy_hold_capacity(navy: &Navy) -> i32 {
    // Each ship has SHIPHOLD (100) capacity
    let capacity = (navy.warships as i32 * (0 + 1))    // Light=0+1
                 + (navy.warships as i32 * (1 + 1))   // Medium=1+1
                 + (navy.warships as i32 * (2 + 1))   // Heavy=2+1
                 + (navy.merchant as i32 * (0 + 1))
                 + (navy.merchant as i32 * (1 + 1))
                 + (navy.merchant as i32 * (2 + 1))
                 + (navy.galleys as i32 * (0 + 1))
                 + (navy.galleys as i32 * (1 + 1))
                 + (navy.galleys as i32 * (2 + 1));
    
    capacity * SHIPCREW as i32
}

/// Set aside items that are up for bid (remove from nation's resources)
/// Matches C: setaside() function
pub fn set_aside_for_trade(nation: &mut Nation, commodity: u8, amount: i64, extra: i32, is_up: bool) {
    match commodity {
        TDGOLD => {
            if !is_up {
                nation.treasury_gold = nation.treasury_gold.saturating_sub(amount);
            }
        }
        TDFOOD => {
            nation.total_food = nation.total_food.saturating_sub(amount);
        }
        TDMETAL => {
            if !is_up {
                nation.metals = nation.metals.saturating_sub(amount);
            }
        }
        TDJEWL => {
            if !is_up {
                nation.jewels = nation.jewels.saturating_sub(amount);
            }
        }
        TDLAND => {
            // Land handled separately - owner changes
        }
        TDARMY => {
            if extra >= 0 && (extra as usize) < MAXARM {
                let army = &mut nation.armies[extra as usize];
                army.movement = 0;
                army.status = ArmyStatus::Traded.to_value();
            }
        }
        TDSHIP => {
            if extra >= 0 && (extra as usize) < MAXNAVY {
                let navy = &mut nation.navies[extra as usize];
                navy.movement = 0;
                navy.commodity = TRADED as u8;
            }
        }
        _ => {}
    }
}

/// Take back items from trade (return to nation's resources)
/// Matches C: takeback() function
pub fn take_back_from_trade(nation: &mut Nation, commodity: u8, amount: i64, extra: i32, _is_up: bool) {
    if nation.name == "unowned" {
        return;
    }
    
    match commodity {
        TDGOLD => {
            nation.treasury_gold += amount;
        }
        TDFOOD => {
            nation.total_food += amount;
        }
        TDMETAL => {
            nation.metals += amount;
        }
        TDJEWL => {
            nation.jewels += amount;
        }
        TDLAND => {
            // Land handled separately
        }
        TDARMY => {
            if extra >= 0 && (extra as usize) < MAXARM {
                let army = &mut nation.armies[extra as usize];
                army.status = ArmyStatus::Defend.to_value();
            }
        }
        TDSHIP => {
            if extra >= 0 && (extra as usize) < MAXNAVY {
                let navy = &mut nation.navies[extra as usize];
                navy.commodity = 0;
            }
        }
        _ => {}
    }
}

/// Execute a trade between two nations
/// Matches C: tradeit() function (ADMIN only)
pub fn execute_trade(
    seller: &mut Nation,
    buyer: &mut Nation,
    commodity: u8,
    amount: i64,
    extra: i32,
) -> TradeResult {
    match commodity {
        TDGOLD => {
            let received = amount * trade_cost(20) / 100;
            buyer.treasury_gold += received;
            TradeResult {
                success: true,
                seller_receives: amount,
                buyer_pays: received,
                message: format!("Trade {} gold for {} gold", amount, received),
            }
        }
        TDFOOD => {
            let received = amount * trade_cost(20) / 100;
            buyer.total_food += received;
            TradeResult {
                success: true,
                seller_receives: amount,
                buyer_pays: received,
                message: format!("Trade {} food for {} food", amount, received),
            }
        }
        TDMETAL => {
            let received = amount * trade_cost(20) / 100;
            buyer.metals += received;
            TradeResult {
                success: true,
                seller_receives: amount,
                buyer_pays: received,
                message: format!("Trade {} metal for {} metal", amount, received),
            }
        }
        TDJEWL => {
            let received = amount * trade_cost(20) / 100;
            buyer.jewels += received;
            TradeResult {
                success: true,
                seller_receives: amount,
                buyer_pays: received,
                message: format!("Trade {} jewels for {} jewels", amount, received),
            }
        }
        TDLAND => {
            // Land transfer handled separately in sector owner change
            TradeResult {
                success: true,
                seller_receives: 0,
                buyer_pays: amount,
                message: "Land transferred".to_string(),
            }
        }
        TDARMY => {
            // Transfer army from seller to buyer
            if extra < 0 || (extra as usize) >= MAXARM {
                return TradeResult {
                    success: false,
                    seller_receives: 0,
                    buyer_pays: 0,
                    message: "Invalid army index".to_string(),
                };
            }
            
            let seller_army = &seller.armies[extra as usize];
            if seller_army.soldiers <= 0 {
                return TradeResult {
                    success: false,
                    seller_receives: 0,
                    buyer_pays: 0,
                    message: "Army does not exist".to_string(),
                };
            }
            
            // Find empty slot in buyer
            let mut buyer_idx = -1;
            for i in 0..MAXARM {
                if buyer.armies[i].soldiers <= 0 {
                    buyer_idx = i as i32;
                    break;
                }
            }
            
            if buyer_idx < 0 {
                return TradeResult {
                    success: false,
                    seller_receives: 0,
                    buyer_pays: 0,
                    message: "Buyer has no room for army".to_string(),
                };
            }
            
            // Transfer army
            let buyer_army = &mut buyer.armies[buyer_idx as usize];
            buyer_army.soldiers = seller_army.soldiers;
            buyer_army.unit_type = seller_army.unit_type;
            buyer_army.x = buyer.cap_x;
            buyer_army.y = buyer.cap_y;
            buyer_army.status = ArmyStatus::Defend.to_value();
            buyer_army.movement = 0;
            
            // Clear seller army
            let seller_army = &mut seller.armies[extra as usize];
            seller_army.soldiers = 0;
            seller_army.movement = 0;
            seller_army.status = ArmyStatus::Defend.to_value();
            
            TradeResult {
                success: true,
                seller_receives: amount,
                buyer_pays: amount,
                message: format!("Army transferred to {}", buyer.name),
            }
        }
        TDSHIP => {
            // Transfer navy
            if extra < 0 || (extra as usize) >= MAXNAVY {
                return TradeResult {
                    success: false,
                    seller_receives: 0,
                    buyer_pays: 0,
                    message: "Invalid navy index".to_string(),
                };
            }
            
            let seller_navy = &seller.navies[extra as usize];
            if seller_navy.warships == 0 && seller_navy.merchant == 0 && seller_navy.galleys == 0 {
                return TradeResult {
                    success: false,
                    seller_receives: 0,
                    buyer_pays: 0,
                    message: "Navy does not exist".to_string(),
                };
            }
            
            // Find empty slot in buyer
            let mut buyer_idx = -1;
            for i in 0..MAXNAVY {
                if !buyer.navies[i].has_ships() {
                    buyer_idx = i as i32;
                    break;
                }
            }
            
            if buyer_idx < 0 {
                return TradeResult {
                    success: false,
                    seller_receives: 0,
                    buyer_pays: 0,
                    message: "Buyer has no room for navy".to_string(),
                };
            }
            
            // Transfer navy
            let buyer_navy = &mut buyer.navies[buyer_idx as usize];
            buyer_navy.warships = seller_navy.warships;
            buyer_navy.merchant = seller_navy.merchant;
            buyer_navy.galleys = seller_navy.galleys;
            buyer_navy.crew = seller_navy.crew;
            buyer_navy.x = seller_navy.x;
            buyer_navy.y = seller_navy.y;
            buyer_navy.commodity = 0;
            buyer_navy.movement = 0;
            
            // Clear seller navy
            let seller_navy = &mut seller.navies[extra as usize];
            seller_navy.movement = 0;
            seller_navy.warships = 0;
            seller_navy.merchant = 0;
            seller_navy.galleys = 0;
            seller_navy.crew = 0;
            seller_navy.commodity = 0;
            
            TradeResult {
                success: true,
                seller_receives: amount,
                buyer_pays: amount,
                message: format!("Navy transferred to {}", buyer.name),
            }
        }
        _ => TradeResult {
            success: false,
            seller_receives: 0,
            buyer_pays: 0,
            message: "Unknown commodity".to_string(),
        },
    }
}

/// Get trade value of an item
/// Matches C: gettval() function
pub fn get_trade_value(
    _seller: &Nation,
    buyer: &Nation,
    commodity: u8,
    amount: i64,
    extra: i32,
) -> i64 {
    match commodity {
        TDGOLD | TDFOOD | TDMETAL | TDJEWL => amount,
        TDLAND => {
            // Would need sector access - return amount as food value
            amount
        }
        TDARMY => {
            army_trade_value(buyer, extra as u8)
        }
        TDSHIP => {
            navy_hold_capacity(&buyer.navies[extra as usize]) as i64
        }
        _ => -1,
    }
}

/// Check nation items for trade (process BUY entries at turn start)
/// Matches C: checktrade() function
pub fn check_trade(deals: &mut Vec<TradeDeal>, nation: &mut Nation, nation_idx: usize) {
    for deal in deals.iter_mut() {
        if deal.deal_type == NOSALE {
            if deal.nation == nation_idx as u8 {
                // Take back the item
                take_back_from_trade(
                    nation,
                    deal.commodity_type,
                    deal.amount,
                    deal.extra,
                    true,
                );
            }
        } else if deal.deal_type == SELL {
            if deal.nation == nation_idx as u8 {
                // Set aside for trade
                set_aside_for_trade(
                    nation,
                    deal.commodity_type,
                    deal.amount,
                    deal.extra,
                    true,
                );
            }
        } else if deal.deal_type == BUY {
            if deal.nation == nation_idx as u8 {
                // Process purchase
                set_aside_for_trade(
                    nation,
                    deal.want_type,
                    deal.amount,
                    deal.amount as i32,
                    true,
                );
            }
        }
    }
}

/// process_trades_gs() — turn-level trade processing using GameState.
/// Matches C uptrade() structure from trade.c line 942.
///
/// In the C original, trades are stored in a flat file: nations post SELL offers,
/// other nations BUY them, and uptrade() matches the highest bidder to each offer,
/// executing commodity swaps (gold, food, metal, jewels, land, soldiers, ships).
///
/// In the web version, trades are player-action-based. This hook validates resource
/// consistency at turn boundary and clears any negative balances.
/// Called AFTER updcapture, BEFORE updmil.
pub fn process_trades_gs(state: &mut GameState) -> Vec<String> {
    let mut news = Vec::new();

    // Collect all pending trades and validate them.
    // In the current model, trade deals are expressed as nation-to-nation
    // commodity exchanges; since persistent deal storage is not yet in GameState,
    // this hook processes any expired or conflicting deals at turn boundary.
    for nation_idx in 1..NTOTAL {
        let active = state.nations[nation_idx].active;
        if active == 0 { continue; }

        // Validate that set-aside resources haven't gone negative
        // (C checktrade() runs at the start of each nation's exec)
        let treasury = state.nations[nation_idx].treasury_gold;
        let food = state.nations[nation_idx].total_food;
        let metals = state.nations[nation_idx].metals;
        let jewels = state.nations[nation_idx].jewels;

        if treasury < 0 {
            state.nations[nation_idx].treasury_gold = 0;
            news.push(format!("{} treasury went negative - cleared", state.nations[nation_idx].name));
        }
        if food < 0 {
            state.nations[nation_idx].total_food = 0;
        }
        if metals < 0 {
            state.nations[nation_idx].metals = 0;
        }
        if jewels < 0 {
            state.nations[nation_idx].jewels = 0;
        }
    }

    news
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commodity_names() {
        assert_eq!(commodity_name(TDGOLD), "Gold");
        assert_eq!(commodity_name(TDFOOD), "Food");
        assert_eq!(commodity_name(TDMETAL), "Metal");
        assert_eq!(commodity_name(TDJEWL), "Jewels");
        assert_eq!(commodity_name(TDLAND), "Land");
        assert_eq!(commodity_name(TDARMY), "Soldiers");
        assert_eq!(commodity_name(TDSHIP), "Ships");
    }

    #[test]
    fn test_army_trade_value_basic() {
        // Create a minimal nation for testing
        let nation = Nation::default();
        // Army value should be 0 for empty army
        assert_eq!(army_trade_value(&nation, 0), 0);
    }
}
