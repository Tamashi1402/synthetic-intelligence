//! si-store — engineering/01 §4–§13, 00 §5–§6, 04 §24–§25, 07 §2–§13.
//! Storage does not think: one append-only lattice, the interner, the frozen
//! loader entry with its two guards and its `engine`-only body, the sealed
//! genome format, the converter, and the container codec.

#[cfg(feature = "engine")]
pub mod build;
pub mod arena;
#[cfg(feature = "engine")]
pub mod frozen_engine;
pub mod genome_file;
pub mod interner;
pub mod layers_frozen;
pub mod opens;
pub mod points;
pub mod snapshot;
pub mod store;
