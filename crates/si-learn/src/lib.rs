//! si-learn — engineering/04 §11–§18; 09 §4–§7.
//!
//! Two surfaces: the write gate (ten steps, six ordering laws, the gain ledger)
//! and RE-PARSE (the decoder maturing retroactively, on idle ticks). The crate
//! owns no vocabulary and no shape test; it receives decoded candidates.

#![forbid(unsafe_code)]

pub mod gate;
pub mod reparse;

pub use gate::{
    append_question, arbitrate, ask_at, conf_init, consistency, latest_open_ask, open_asks,
    resolve_ask, serve, stage_of, novelty_delta, tier, write, AskRow, Branch, Candidate, Conflict,
    GateEvent, GateStats, NoveltyDelta, Outcome, QuestionSpec, ServeReport, Verdict, WriteContext,
    WriteReport,
};
pub use reparse::{eligible, has_derived_child, pass as reparse_pass, DerivedSlot, Parser};
