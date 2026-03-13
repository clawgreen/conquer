use bitflags::bitflags;

bitflags! {
    /// Nation powers bitmask — matches C `long powers` field in `s_nation`.
    /// Each power is a single bit in a 32-bit field.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
    pub struct Power: i64 {
        // Military Powers (S_MIL = 0, E_MIL = 11)
        const WARRIOR   = 0x00000001;
        const CAPTAIN   = 0x00000002;
        const WARLORD   = 0x00000004;
        const ARCHER    = 0x00000008;
        const CAVALRY   = 0x00000010;
        const SAPPER    = 0x00000020;
        const ARMOR     = 0x00000040;
        const AVIAN     = 0x00000080;
        const MI_MONST  = 0x00000100;
        const AV_MONST  = 0x00000200;
        const MA_MONST  = 0x00000400;

        // Civilian Powers (S_CIV = 11, E_CIV = 13)
        const SLAVER    = 0x00000800;
        const DERVISH   = 0x00001000;
        const HIDDEN    = 0x00002000;
        const ARCHITECT = 0x00004000;
        const RELIGION  = 0x00008000;
        const MINER     = 0x00010000;
        const BREEDER   = 0x00020000;
        const URBAN     = 0x00040000;
        const STEEL     = 0x00080000;
        const NINJA     = 0x00100000;
        const SAILOR    = 0x00200000;
        const DEMOCRACY = 0x00400000;
        const ROADS     = 0x00800000;

        // Magical Powers (S_MGK = 24, E_MGK = 7)
        const THE_VOID  = 0x01000000;
        const KNOWALL   = 0x02000000;
        const DESTROYER = 0x04000000;
        const VAMPIRE   = 0x08000000;
        const SUMMON    = 0x10000000;
        const WYZARD    = 0x20000000;
        const SORCERER  = 0x40000000;
    }
}

/// Total number of powers
pub const MAXPOWER: usize = 31;

/// Power category boundaries
pub const S_MIL: usize = 0;
pub const E_MIL: usize = 11;
pub const S_CIV: usize = 11;
pub const E_CIV: usize = 13; // number of civilian powers
pub const S_MGK: usize = 24;
pub const E_MGK: usize = 7; // number of magical powers

/// Power type categories
pub const M_MIL: u8 = 1;
pub const M_CIV: u8 = 2;
pub const M_MGK: u8 = 3;
pub const M_TECH: u8 = 4;
pub const M_ALL: u8 = 5;

impl Power {
    /// Check if a nation has a specific power (matches C macro `magic(NATION, POWER)`)
    pub fn has_power(nation_powers: i64, power: Power) -> bool {
        (nation_powers & power.bits()) != 0
    }

    /// Get power by index (0-30), matching the C `powers[]` array
    pub fn from_index(index: usize) -> Option<Power> {
        POWERS_ARRAY.get(index).copied()
    }

    /// Get the display name for this power
    pub fn name_by_index(index: usize) -> &'static str {
        POWER_NAMES.get(index).copied().unwrap_or("ERROR")
    }
}

/// The `powers[]` array from C — maps index to power bitmask
pub const POWERS_ARRAY: [Power; 31] = [
    Power::WARRIOR,   // 0
    Power::CAPTAIN,   // 1
    Power::WARLORD,   // 2
    Power::ARCHER,    // 3
    Power::CAVALRY,   // 4
    Power::SAPPER,    // 5
    Power::ARMOR,     // 6
    Power::AVIAN,     // 7
    Power::MI_MONST,  // 8
    Power::AV_MONST,  // 9
    Power::MA_MONST,  // 10
    Power::SLAVER,    // 11
    Power::DERVISH,   // 12
    Power::HIDDEN,    // 13
    Power::ARCHITECT, // 14
    Power::RELIGION,  // 15
    Power::MINER,     // 16
    Power::BREEDER,   // 17
    Power::URBAN,     // 18
    Power::STEEL,     // 19
    Power::NINJA,     // 20
    Power::SAILOR,    // 21
    Power::DEMOCRACY, // 22
    Power::ROADS,     // 23
    Power::THE_VOID,  // 24
    Power::KNOWALL,   // 25
    Power::DESTROYER, // 26
    Power::VAMPIRE,   // 27
    Power::SUMMON,    // 28
    Power::WYZARD,    // 29
    Power::SORCERER,  // 30
];

/// Power names matching C `pwrname[]` array
pub const POWER_NAMES: [&str; 32] = [
    "WARRIOR",
    "CAPTAIN",
    "WARLORD",
    "ARCHER",
    "CAVALRY",
    "SAPPER",
    "ARMOR",
    "AVIAN",
    "MI_MONST",
    "AV_MONST",
    "MA_MONST",
    "SLAVER",
    "DERVISH",
    "HIDDEN",
    "ARCHITECT",
    "RELIGION",
    "MINER",
    "BREEDER",
    "URBAN",
    "STEEL",
    "NINJA",
    "SAILOR",
    "DEMOCRACY",
    "ROADS",
    "THE_VOID",
    "KNOWALL",
    "DESTROYER",
    "VAMPIRE",
    "SUMMON",
    "WYZARD",
    "SORCERER",
    "ERROR",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_values_match_c() {
        assert_eq!(Power::WARRIOR.bits(), 0x00000001);
        assert_eq!(Power::CAPTAIN.bits(), 0x00000002);
        assert_eq!(Power::WARLORD.bits(), 0x00000004);
        assert_eq!(Power::ARCHER.bits(), 0x00000008);
        assert_eq!(Power::CAVALRY.bits(), 0x00000010);
        assert_eq!(Power::SAPPER.bits(), 0x00000020);
        assert_eq!(Power::ARMOR.bits(), 0x00000040);
        assert_eq!(Power::AVIAN.bits(), 0x00000080);
        assert_eq!(Power::MI_MONST.bits(), 0x00000100);
        assert_eq!(Power::AV_MONST.bits(), 0x00000200);
        assert_eq!(Power::MA_MONST.bits(), 0x00000400);
        assert_eq!(Power::SLAVER.bits(), 0x00000800);
        assert_eq!(Power::DERVISH.bits(), 0x00001000);
        assert_eq!(Power::HIDDEN.bits(), 0x00002000);
        assert_eq!(Power::ARCHITECT.bits(), 0x00004000);
        assert_eq!(Power::RELIGION.bits(), 0x00008000);
        assert_eq!(Power::MINER.bits(), 0x00010000);
        assert_eq!(Power::BREEDER.bits(), 0x00020000);
        assert_eq!(Power::URBAN.bits(), 0x00040000);
        assert_eq!(Power::STEEL.bits(), 0x00080000);
        assert_eq!(Power::NINJA.bits(), 0x00100000);
        assert_eq!(Power::SAILOR.bits(), 0x00200000);
        assert_eq!(Power::DEMOCRACY.bits(), 0x00400000);
        assert_eq!(Power::ROADS.bits(), 0x00800000);
        assert_eq!(Power::THE_VOID.bits(), 0x01000000);
        assert_eq!(Power::KNOWALL.bits(), 0x02000000);
        assert_eq!(Power::DESTROYER.bits(), 0x04000000);
        assert_eq!(Power::VAMPIRE.bits(), 0x08000000);
        assert_eq!(Power::SUMMON.bits(), 0x10000000);
        assert_eq!(Power::WYZARD.bits(), 0x20000000);
        assert_eq!(Power::SORCERER.bits(), 0x40000000);
    }

    #[test]
    fn test_has_power() {
        let p = Power::WARRIOR.bits() | Power::SORCERER.bits();
        assert!(Power::has_power(p, Power::WARRIOR));
        assert!(Power::has_power(p, Power::SORCERER));
        assert!(!Power::has_power(p, Power::CAPTAIN));
    }

    #[test]
    fn test_powers_array_matches() {
        assert_eq!(POWERS_ARRAY[0], Power::WARRIOR);
        assert_eq!(POWERS_ARRAY[10], Power::MA_MONST);
        assert_eq!(POWERS_ARRAY[11], Power::SLAVER);
        assert_eq!(POWERS_ARRAY[23], Power::ROADS);
        assert_eq!(POWERS_ARRAY[24], Power::THE_VOID);
        assert_eq!(POWERS_ARRAY[30], Power::SORCERER);
    }
}
