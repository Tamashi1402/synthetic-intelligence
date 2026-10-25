//! si-affect — engineering/04 §6–§8, §16, §10; 03 §3–§4.
//!
//! The tissue and its six doors. Its public surface is `drive`'s six functions;
//! everything here consumes anonymous signal numbers and returns numbers. It
//! never sees a row, a point, a word or a source, and it contains no float.

#![forbid(unsafe_code)]

pub mod arousal;
pub mod bake;
pub mod drive;
pub mod params;
pub mod signal;

pub use arousal::{Integrator, Lagged};
pub use bake::bake_q12;
pub use drive::{conduction, emit, prioritize, salience, update, budget, Budget};
pub use params::Params;
pub use signal::{Signal, SignalKind};
