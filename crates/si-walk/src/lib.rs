//! si-walk — engineering/04 §5, §10, §19; 03 §2, §5; 02 §4.
//!
//! One thinking engine: the eleven-step round, the settle test, and the idle
//! tick that calls the same walk. This crate reads the lattice and never writes
//! it: no store import beyond the immutable borrow `Graph::build` takes, and no
//! `si-learn` edge at all (I9, R-25). Every modulation value is injected by the
//! caller; the swept `MODCurve` is an artifact, not a constant here.

#![forbid(unsafe_code)]

pub mod curve;
pub mod graph;
pub mod hub;
pub mod idle;
pub mod walk;

pub use curve::{Curve, CurvePoint};
pub use graph::{Arc, Arcs, Graph};
// 00 §2: `si-sleep ──► core, store, walk` only, yet a sleep walk borrows the
// session's integrator. The type reaches si-sleep through this re-export rather
// than through an si-affect edge in si-sleep's manifest.
pub use si_affect::{params::LagUnit, Integrator, Params};
pub use idle::{
    cascade, idle_tick, ignite, Agenda, IdleAction, IdleInputs, IdleWeights, QuestionRef,
};
pub use walk::{
    walk, Activation, Contribution, Lit, OscillationRule, Relight, RoundTrace, StopReason, Thought,
    WalkConfig,
};

/// 04 §4 / 01 §3: the round-0 activation quantum is one unit on the walk's Q12
/// grid (`1.0 = 4096`). No registry row names a different magnitude; the grid
/// row is the only definition the design supplies.
pub const LIGHT_QUANTUM: i32 = 4096;
