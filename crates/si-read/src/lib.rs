//! si-read — engineering/04 §2–§4, §7, §11, §20, §21; docs/07; 02 §6.
//!
//! Stage B4: lighting, retrieval-as-the-walk, the gap predicates, the ask
//! engine and the nonce. The crate reads the lattice and never writes it: there
//! is no store mutation here and no `si-learn`/`si-audit` edge (I7).
//!
//! `Thought`, `Activation` and `StopReason` are re-exported from `si-walk`
//! because `si-front` may not take a `si-walk` edge (00 §2): the labeler names
//! the settle result through this crate.

#![forbid(unsafe_code)]

pub mod ask;
pub mod event;
pub mod gaps;
pub mod guess;
pub mod light;
pub mod retrieve;

pub use ask::{ask_back, AskBack};
pub use event::{event_view_index, event_views};
pub use gaps::{gaps, Gap, GapContext, GapKind};
pub use guess::guess_seed;
pub use light::{light, LightRequest, Lighting, Resolve};
pub use retrieve::{
    primary_kind, retrieve, route_counts, Cited, ReadRequest, Retrieval, RouteCounts, SlotKey,
};
pub use si_walk::{Activation, StopReason, Thought};
