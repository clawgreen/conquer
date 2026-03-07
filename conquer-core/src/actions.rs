use serde::{Deserialize, Serialize};

/// Execute-file action type codes matching C defines (XASTAT, XAMEN, etc.)
pub mod codes {
    pub const XASTAT: i32 = 1;
    pub const XAMEN: i32 = 2;
    pub const XBRIBE: i32 = 3;
    pub const XALOC: i32 = 4;
    pub const XNLOC: i32 = 5;
    pub const XNAMER: i32 = 6;
    pub const XNACREW: i32 = 7;
    pub const XECNAME: i32 = 8;
    pub const XECPAS: i32 = 9;
    pub const EDSPL: i32 = 10;
    pub const XSADES: i32 = 11;
    pub const XSACIV: i32 = 12;
    pub const XSIFORT: i32 = 13;
    pub const XNAGOLD: i32 = 14;
    pub const XAMOV: i32 = 15;
    pub const XNMOV: i32 = 16;
    pub const XSAOWN: i32 = 17;
    pub const EDADJ: i32 = 18;
    pub const XNARGOLD: i32 = 19;
    pub const XNAMETAL: i32 = 20;
    // 21 is unused
    pub const INCAPLUS: i32 = 22;
    pub const INCDPLUS: i32 = 23;
    pub const CHG_MGK: i32 = 24;
    pub const DESTRY: i32 = 25;
    pub const MSETA: i32 = 26;
    pub const MSETB: i32 = 27;
    pub const NTAX: i32 = 28;
    pub const XNAWAR: i32 = 29;
    pub const XNAGAL: i32 = 30;
    pub const XNAHOLD: i32 = 31;
    pub const NPOP: i32 = 32;
    pub const XSACIV3: i32 = 33;
}

/// Typed action enum replacing the C execute-file format.
/// Each variant corresponds to one C execute-file macro.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Action {
    /// AADJSTAT / XASTAT — set army status
    AdjustArmyStat { nation: i16, army: i32, status: u8 },

    /// AADJMEN / XAMEN — set army soldiers and type
    AdjustArmyMen { nation: i16, army: i16, soldiers: i64, unit_type: u8 },

    /// BRIBENATION / XBRIBE — bribe a nation
    BribeNation { nation: i32, cost: i64, target: i32 },

    /// AADJLOC / XALOC — move army
    MoveArmy { nation: i16, army: i32, x: i32, y: i32 },

    /// NADJLOC / XNLOC — move navy
    MoveNavy { nation: i16, fleet: i32, x: i32, y: i32 },

    /// NADJMER / XNAMER — adjust navy merchant ships
    AdjustNavyMerchant { nation: i16, fleet: i32, merchant: i16 },

    /// NADJCRW / XNACREW — adjust navy crew and army
    AdjustNavyCrew { nation: i16, fleet: i16, crew: i32, army_num: i32 },

    /// ECHGNAME / XECNAME — change nation name
    ChangeName { nation: i16, name: String },

    /// ECHGPAS / XECPAS — change password
    ChangePassword { nation: i16, password: String },

    /// EDECSPL / EDSPL — adjust spell points
    AdjustSpellPoints { nation: i16, cost: i32 },

    /// SADJDES / XSADES — redesignate sector
    DesignateSector { nation: i16, x: i32, y: i32, designation: char },

    /// SADJCIV / XSACIV — set sector civilian count
    AdjustSectorCiv { nation: i16, people: i64, x: i32, y: i32 },

    /// SADJCIV3 / XSACIV3 — add civilians to sector
    AddSectorCiv { nation: i16, people: i64, x: i32, y: i32 },

    /// INCFORT / XSIFORT — increase fortress level
    IncreaseFort { nation: i16, x: i32, y: i32 },

    /// XNAGOLD — adjust navy gold (nation treasury adjustment)
    AdjustNavyGold { nation: i16, gold: i64 },

    /// AADJMOV / XAMOV — adjust army movement points
    AdjustArmyMove { nation: i16, army: i32, movement: i32 },

    /// NADJMOV / XNMOV — adjust navy movement points
    AdjustNavyMove { nation: i16, fleet: i32, movement: i32 },

    /// SADJOWN / XSAOWN — take sector ownership
    TakeSectorOwnership { nation: i16, x: i32, y: i32 },

    /// EADJDIP / EDADJ — adjust diplomacy
    AdjustDiplomacy { nation_a: i16, nation_b: i32, status: i32 },

    /// NADJWAR / XNAWAR — adjust navy warships
    AdjustNavyWarships { nation: i16, fleet: i32, warships: i16 },

    /// NADJGAL / XNAGAL — adjust navy galleys
    AdjustNavyGalleys { nation: i16, fleet: i32, galleys: i16 },

    /// NADJHLD / XNAHOLD — adjust navy hold (army + people)
    AdjustNavyHold { nation: i16, fleet: i32, army_num: i16, people: i32 },

    /// NADJNTN2 / NPOP — adjust population stats
    AdjustPopulation { nation: i16, popularity: i32, terror: i32, reputation: i32 },

    /// NADJNTN / NTAX — adjust tax rate, active status, charity
    AdjustTax { nation: i16, tax_rate: i32, active: i32, charity: i32 },

    /// I_APLUS / INCAPLUS — increase attack bonus
    IncreaseAttack { nation: i16 },

    /// I_DPLUS / INCDPLUS — increase defense bonus
    IncreaseDefense { nation: i16 },

    /// CHGMGK / CHG_MGK — change magic powers
    ChangeMagic { nation: i16, powers: i64, new_power: i64 },

    /// DESTROY / DESTRY — destroy a nation
    DestroyNation { target: i16, by: i16 },

    /// AADJMERC / MSETA — hire mercenaries
    HireMercenaries { nation: i32, men: i64 },

    /// AADJDISB / MSETB — disband to mercenary pool
    DisbandToMerc { nation: i32, men: i64, attack: i32, defense: i32 },

    // ============================================================
    // Sprint: Commands Parity — new action variants
    // ============================================================

    /// T1: Split soldiers from an army into a new army at same location
    SplitArmy { nation: i16, army: i32, soldiers: i64 },

    /// T2: Combine army2 into army1 (same location, compatible types)
    CombineArmies { nation: i16, army1: i32, army2: i32 },

    /// T4: Divide army equally in half
    DivideArmy { nation: i16, army: i32 },

    /// T5: Draft/enlist soldiers in a sector
    DraftUnit { nation: i16, x: i32, y: i32, unit_type: u8, count: i64 },

    /// T6: Construct fortification in a sector
    ConstructFort { nation: i16, x: i32, y: i32 },

    /// T7: Build a road in a sector
    BuildRoad { nation: i16, x: i32, y: i32 },

    /// T8: Construct ships at a coastal sector
    ConstructShip { nation: i16, x: i32, y: i32, ship_type: u8, ship_size: u8, count: i32 },

    /// T9: Load army onto fleet
    LoadArmyOnFleet { nation: i16, army: i32, fleet: i32 },

    /// T9: Unload army from fleet
    UnloadArmyFromFleet { nation: i16, fleet: i32 },

    /// T9: Load civilians onto fleet
    LoadPeopleOnFleet { nation: i16, fleet: i32, x: i32, y: i32, amount: i64 },

    /// T9: Unload civilians from fleet
    UnloadPeople { nation: i16, fleet: i32, x: i32, y: i32, amount: i64 },

    /// T10: Cast a spell
    CastSpell { nation: i16, spell_type: u8, target_x: i32, target_y: i32, target_nation: i16 },

    /// T11: Buy a magic power
    BuyMagicPower { nation: i16, power_type: u8 },

    /// T12: Propose a trade
    ProposeTrade { nation: i16, target_nation: i16, offer_type: u8, offer_amount: i64, request_type: u8, request_amount: i64 },

    /// T12: Accept a trade
    AcceptTrade { nation: i16, trade_id: u32 },

    /// T12: Reject a trade
    RejectTrade { nation: i16, trade_id: u32 },

    /// T17: Send tribute (gold, food, metal, jewels)
    SendTribute { nation: i16, target: i16, gold: i64, food: i64, metal: i64, jewels: i64 },
}

impl Action {
    /// Returns true if this action can be submitted by a player via the REST API.
    /// Engine-only actions (used by NPC AI and turn processing) return false.
    pub fn is_player_action(&self) -> bool {
        match self {
            // Player actions — can be submitted via REST API
            Action::MoveArmy { .. } => true,
            Action::MoveNavy { .. } => true,
            Action::AdjustArmyStat { .. } => true,
            Action::SplitArmy { .. } => true,
            Action::CombineArmies { .. } => true,
            Action::DivideArmy { .. } => true,
            Action::DraftUnit { .. } => true,
            Action::ConstructFort { .. } => true,
            Action::BuildRoad { .. } => true,
            Action::ConstructShip { .. } => true,
            Action::LoadArmyOnFleet { .. } => true,
            Action::UnloadArmyFromFleet { .. } => true,
            Action::LoadPeopleOnFleet { .. } => true,
            Action::UnloadPeople { .. } => true,
            Action::CastSpell { .. } => true,
            Action::BuyMagicPower { .. } => true,
            Action::ProposeTrade { .. } => true,
            Action::AcceptTrade { .. } => true,
            Action::RejectTrade { .. } => true,
            Action::AdjustDiplomacy { .. } => true,
            Action::BribeNation { .. } => true,
            Action::SendTribute { .. } => true,
            Action::HireMercenaries { .. } => true,
            Action::DisbandToMerc { .. } => true,
            Action::DesignateSector { .. } => true,
            Action::AdjustTax { .. } => true,
            Action::AdjustPopulation { .. } => true,
            Action::ChangeName { .. } => true,
            Action::ChangePassword { .. } => true,
            Action::IncreaseAttack { .. } => true,
            Action::IncreaseDefense { .. } => true,

            // Engine-only actions — blocked from player API
            Action::AdjustArmyMen { .. } => false,
            Action::AdjustArmyMove { .. } => false,
            Action::AdjustNavyMove { .. } => false,
            Action::AdjustNavyGold { .. } => false,
            Action::AdjustNavyMerchant { .. } => false,
            Action::AdjustNavyWarships { .. } => false,
            Action::AdjustNavyGalleys { .. } => false,
            Action::AdjustNavyHold { .. } => false,
            Action::AdjustNavyCrew { .. } => false,
            Action::AddSectorCiv { .. } => false,
            Action::AdjustSectorCiv { .. } => false,
            Action::TakeSectorOwnership { .. } => false,
            Action::IncreaseFort { .. } => false,
            Action::ChangeMagic { .. } => false,
            Action::AdjustSpellPoints { .. } => false,
            Action::DestroyNation { .. } => false,
        }
    }

    /// Get the execute-file code for this action
    pub fn code(&self) -> i32 {
        match self {
            Action::AdjustArmyStat { .. } => codes::XASTAT,
            Action::AdjustArmyMen { .. } => codes::XAMEN,
            Action::BribeNation { .. } => codes::XBRIBE,
            Action::MoveArmy { .. } => codes::XALOC,
            Action::MoveNavy { .. } => codes::XNLOC,
            Action::AdjustNavyMerchant { .. } => codes::XNAMER,
            Action::AdjustNavyCrew { .. } => codes::XNACREW,
            Action::ChangeName { .. } => codes::XECNAME,
            Action::ChangePassword { .. } => codes::XECPAS,
            Action::AdjustSpellPoints { .. } => codes::EDSPL,
            Action::DesignateSector { .. } => codes::XSADES,
            Action::AdjustSectorCiv { .. } => codes::XSACIV,
            Action::AddSectorCiv { .. } => codes::XSACIV3,
            Action::IncreaseFort { .. } => codes::XSIFORT,
            Action::AdjustNavyGold { .. } => codes::XNAGOLD,
            Action::AdjustArmyMove { .. } => codes::XAMOV,
            Action::AdjustNavyMove { .. } => codes::XNMOV,
            Action::TakeSectorOwnership { .. } => codes::XSAOWN,
            Action::AdjustDiplomacy { .. } => codes::EDADJ,
            Action::AdjustNavyWarships { .. } => codes::XNAWAR,
            Action::AdjustNavyGalleys { .. } => codes::XNAGAL,
            Action::AdjustNavyHold { .. } => codes::XNAHOLD,
            Action::AdjustPopulation { .. } => codes::NPOP,
            Action::AdjustTax { .. } => codes::NTAX,
            Action::IncreaseAttack { .. } => codes::INCAPLUS,
            Action::IncreaseDefense { .. } => codes::INCDPLUS,
            Action::ChangeMagic { .. } => codes::CHG_MGK,
            Action::DestroyNation { .. } => codes::DESTRY,
            Action::HireMercenaries { .. } => codes::MSETA,
            Action::DisbandToMerc { .. } => codes::MSETB,
            // Sprint commands — use synthetic codes above 100
            Action::SplitArmy { .. } => 101,
            Action::CombineArmies { .. } => 102,
            Action::DivideArmy { .. } => 103,
            Action::DraftUnit { .. } => 104,
            Action::ConstructFort { .. } => 105,
            Action::BuildRoad { .. } => 106,
            Action::ConstructShip { .. } => 107,
            Action::LoadArmyOnFleet { .. } => 108,
            Action::UnloadArmyFromFleet { .. } => 109,
            Action::LoadPeopleOnFleet { .. } => 110,
            Action::UnloadPeople { .. } => 111,
            Action::CastSpell { .. } => 112,
            Action::BuyMagicPower { .. } => 113,
            Action::ProposeTrade { .. } => 114,
            Action::AcceptTrade { .. } => 115,
            Action::RejectTrade { .. } => 116,
            Action::SendTribute { .. } => 117,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_codes() {
        let a = Action::AdjustArmyStat { nation: 1, army: 0, status: 3 };
        assert_eq!(a.code(), codes::XASTAT);

        let b = Action::MoveArmy { nation: 1, army: 0, x: 10, y: 20 };
        assert_eq!(b.code(), codes::XALOC);
    }

    #[test]
    fn test_action_serde_roundtrip() {
        let actions = vec![
            Action::AdjustArmyStat { nation: 1, army: 0, status: 3 },
            Action::MoveArmy { nation: 2, army: 5, x: 10, y: 20 },
            Action::ChangeName { nation: 3, name: "TestNation".to_string() },
            Action::ChangeMagic { nation: 1, powers: 0xFF, new_power: 0x100 },
        ];

        for action in &actions {
            let json = serde_json::to_string(action).unwrap();
            let restored: Action = serde_json::from_str(&json).unwrap();
            assert_eq!(*action, restored);
        }
    }
}
