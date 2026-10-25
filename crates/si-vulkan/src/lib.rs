//! si-vulkan — engineering/08, 00 §2 (core only; displays, never decides).
//!
//! Process B of [00 §7]: it owns **zero** SI state and has exactly one writable
//! file, [`WRITABLE_FILE`]. Everything else this crate does is a read of an
//! artifact the loop already wrote, which is what the three laws of 08 §1 make
//! into a crate graph rather than a review note: the missing edges
//! `si-vulkan ↛ si-learn / si-read / si-walk` ([02 §10]) are stated here as
//! [`CONTROL_CRATES`] and checked by `tests/laws.rs` against the real sources.
//!
//! The pure half — [`frames`], [`layout`], [`panels`], [`atlas`], [`batch`] —
//! performs no Vulkan call and no arithmetic over the lattice: it decodes
//! published numbers and turns them into draw lists. The [`surface`] half is the
//! window, and a device loss there degrades to headless without the loop
//! noticing.

pub mod atlas;
pub mod batch;
pub mod frames;
pub mod layout;
pub mod panels;
pub mod spirv;
#[cfg(feature = "vulkan")]
pub mod surface;

/// 08 §1's OBSERVER-ONLY law as a path name: the one file process B may write,
/// and every frame it appends there carries `source: owner` because it is the
/// owner's own keystroke ([00 §7]).
pub const WRITABLE_FILE: &str = "sense_inbox.siro";

/// The crates this one may never import ([02 §10], [00 §2]'s "core ONLY").
/// Vulkan displays; it does not decide what SI knows (**OD-44**).
pub const CONTROL_CRATES: [&str; 3] = ["si-learn", "si-read", "si-walk"];

/// The crate's single dependency edge. Named so `tests/laws.rs` can compare the
/// real `Cargo.toml` against it instead of trusting a comment.
pub const ALLOWED_SI_DEPENDENCY: &str = "si-core";

/// [00 §7]'s privilege table read as bytes: the `source` field is minted by the
/// seam, and process B may only ever mint the owner's own byte. A keystroke from
/// this window is the owner speaking, so [`frames::sense_frame`] writes
/// [`SOURCE_OWNER`] and nothing else — writing `1` would be the viewer putting
/// words in the supporter's mouth, which §9 lists as a refused widget.
pub const SOURCE_OWNER: u8 = 0;

/// The supporter's byte, present so the law above is checkable rather than
/// implied: this crate reads it in a lane ([`panels::source_tag`]) and never
/// writes it.
pub const SOURCE_SUPPORTER: u8 = 1;

/// 08 §3.1 / §9: the viewer holds no label word, no emotion word, no temperament
/// adjective and no content-word list. It is not a missing dependency —
/// `si-audit` and `si-core::temperament` are reachable only through the banned
/// edges above, and `derive` does live in `si-core`, so 08 §11's fingerprint
/// refusal ("the panel does not re-derive; it forwards kind 20") is a **rule**.
pub const NO_VOCABULARY: &str =
    "the only English in this crate is chrome; every classified string is a forwarded payload";
