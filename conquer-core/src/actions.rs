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
}

impl Action {
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
