//! docs/03 §3.2 / 03 §8: the closed innate pattern table. Nine named frames plus
//! `Undecoded`; a tenth named frame or a synonym table is a design change.

pub const PATTERN_TABLE_SIZE: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PatternId {
    PAttrib,
    PDef,
    PQuestA,
    PQuestD,
    PQuestW,
    POfRel,
    PCorrect,
    PCmdC,
    PTeach,
    Undecoded,
}

impl PatternId {
    pub const fn name(self) -> &'static str {
        match self {
            PatternId::PAttrib => "P_ATTRIB",
            PatternId::PDef => "P_DEF",
            PatternId::PQuestA => "P_QUEST_A",
            PatternId::PQuestD => "P_QUEST_D",
            PatternId::PQuestW => "P_QUEST_W",
            PatternId::POfRel => "P_OF_REL",
            PatternId::PCorrect => "P_CORRECT",
            PatternId::PCmdC => "P_CMD_C",
            PatternId::PTeach => "P_TEACH",
            PatternId::Undecoded => "UNDECODED",
        }
    }

    pub const fn index(self) -> u8 {
        match self {
            PatternId::PAttrib => 0,
            PatternId::PDef => 1,
            PatternId::PQuestA => 2,
            PatternId::PQuestD => 3,
            PatternId::PQuestW => 4,
            PatternId::POfRel => 5,
            PatternId::PCorrect => 6,
            PatternId::PCmdC => 7,
            PatternId::PTeach => 8,
            PatternId::Undecoded => 9,
        }
    }

    /// The reverse of `index`, for a row whose `pattern_id` is read back from
    /// the lattice. `None` for an ordinal outside the closed table.
    pub const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(PatternId::PAttrib),
            1 => Some(PatternId::PDef),
            2 => Some(PatternId::PQuestA),
            3 => Some(PatternId::PQuestD),
            4 => Some(PatternId::PQuestW),
            5 => Some(PatternId::POfRel),
            6 => Some(PatternId::PCorrect),
            7 => Some(PatternId::PCmdC),
            8 => Some(PatternId::PTeach),
            9 => Some(PatternId::Undecoded),
            _ => None,
        }
    }
}

pub const PATTERN_TABLE: [PatternId; PATTERN_TABLE_SIZE] = [
    PatternId::PAttrib,
    PatternId::PDef,
    PatternId::PQuestA,
    PatternId::PQuestD,
    PatternId::PQuestW,
    PatternId::POfRel,
    PatternId::PCorrect,
    PatternId::PCmdC,
    PatternId::PTeach,
    PatternId::Undecoded,
];

pub fn table_is_closed() -> bool {
    PATTERN_TABLE.len() == PATTERN_TABLE_SIZE
        && PATTERN_TABLE.contains(&PatternId::PTeach)
        && PATTERN_TABLE.contains(&PatternId::Undecoded)
        && PATTERN_TABLE.iter().filter(|p| **p != PatternId::Undecoded).count() == 9
}
