//! The stage table of engineering/11 §4: which static checks each stage turns
//! on. A check whose subject does not exist yet is disabled and listed, never
//! green (11 §4).

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    B0,
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
    B8,
    B9,
    B14,
    B15,
    B16,
    B17,
    B18,
    B10,
    B11,
    B12,
    B13,
    B20,
}

impl Stage {
    /// engineering/12 §8's five arc stages sit between B9 and B10: the night run
    /// builds them first (`12 §8`), so the table follows the build order.
    pub const ALL: [Stage; 20] = [
        Stage::B0,
        Stage::B1,
        Stage::B2,
        Stage::B3,
        Stage::B4,
        Stage::B5,
        Stage::B6,
        Stage::B7,
        Stage::B8,
        Stage::B9,
        Stage::B14,
        Stage::B15,
        Stage::B16,
        Stage::B17,
        Stage::B18,
        Stage::B10,
        Stage::B11,
        Stage::B12,
        Stage::B13,
        Stage::B20,
    ];

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|stage| stage.label().eq_ignore_ascii_case(text))
    }

    /// The stage whose prefix is the whole set: 11 §4 puts **B20** last precisely so
    /// `check --all` runs every id while each earlier stage's prefix stays exactly
    /// what it was when that stage shipped.
    pub const fn union() -> Self {
        Stage::B20
    }

    pub const fn label(self) -> &'static str {
        match self {
            Stage::B0 => "B0",
            Stage::B1 => "B1",
            Stage::B2 => "B2",
            Stage::B3 => "B3",
            Stage::B4 => "B4",
            Stage::B5 => "B5",
            Stage::B6 => "B6",
            Stage::B7 => "B7",
            Stage::B8 => "B8",
            Stage::B9 => "B9",
            Stage::B14 => "B14",
            Stage::B15 => "B15",
            Stage::B16 => "B16",
            Stage::B17 => "B17",
            Stage::B18 => "B18",
            Stage::B10 => "B10",
            Stage::B11 => "B11",
            Stage::B12 => "B12",
            Stage::B13 => "B13",
            Stage::B20 => "B20",
        }
    }

    /// Every check that is live at this stage, in registry order.
    pub fn checks(self) -> Vec<&'static str> {
        crate::harness::checks::CHECK_IDS
            .into_iter()
            .filter(|id| crate::harness::checks::enable_stage(id) <= self)
            .collect()
    }

    /// Checks whose subject first exists at this stage.
    pub fn introduced(self) -> Vec<&'static str> {
        crate::harness::checks::CHECK_IDS
            .into_iter()
            .filter(|id| crate::harness::checks::enable_stage(id) == self)
            .collect()
    }
}
