//! engineering/11 §4 DoD 2: the generated table of swept values.
//!
//! A row appears here **only** when a sweep artifact publishes a winner for it.
//! Each entry names the artifact that chose the value, the artifact's sha256 and
//! the version tag the sweep bumped, so a value can never be traced to a hand
//! edit: `check_registry_size` re-hashes the artifact and refuses a mismatch, a
//! missing file, a value outside the row's box, or a name that is not a swept
//! scalar SPECIES row.
//!
//! The file is written by `si-harness sweep` and is never edited by hand. It is
//! empty at this stage because every sweep runner publishes a named refusal: no
//! enumeration rule for the open-ended boxes is ruled, so no artifact can name a
//! winner (OD-192). An empty table means OD-86's placeholders are still the
//! arithmetic — which `--strict-consts` refuses to run against.

/// One published value: the row it replaces, the number, and the artifact that
/// chose it (05 §4 steps 4–5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SweptValue {
    pub name: &'static str,
    pub value: i64,
    pub artifact: &'static str,
    pub sha256: &'static str,
    pub version_tag: &'static str,
}

/// The published values, in registry order.
pub const SWEPT: &[SweptValue] = &[];

pub fn value(name: &str) -> Option<&'static SweptValue> {
    SWEPT.iter().find(|entry| entry.name == name)
}
