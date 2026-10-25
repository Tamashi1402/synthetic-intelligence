//! si-supporter — engineering/02 §8, 04 §23; docs/21.
//! The seam (process C's one route into SI) and the speak gate (the two-layer
//! mouth filter that sits **upstream of sense**).
//!
//! Stage B9 installs both. The crate may depend on `si-core` and `si-store` and
//! nothing else: `si-supporter ↛ si-affect` is a missing edge in the build graph,
//! which is the mechanical form of rule (vi) — neither the seam nor the filter
//! can read `a`, charge, `bake` or the median because no module they can reach
//! holds them ([00 §2](../../../engineering/00-stack-workspace.md), R-34).

pub mod seam;
pub mod speakgate;

pub use seam::{Admission, GateLog, Seam, SeamState};
pub use speakgate::{Lexicon, Verdict, check, gate, layer1, layer2, lexicon, token_keys};
