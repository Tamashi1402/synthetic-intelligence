//! si-front — engineering/04 §2–§4, §20, §21; 09 §2–§3; 02 §7.
//!
//! Stage B4: R1–R3 tokenizer, the nine named `P_*` frames plus `Undecoded`, the
//! `/teach` split with its malformation table, the copy-not-create renderer and
//! the demote-only claim labeler. The crate carries the `source` tag through and
//! never infers it from text (04 §21.7); it may not import affect or learn.

#![forbid(unsafe_code)]

pub mod decode;
pub mod label;
pub mod render;
pub mod teach;
pub mod tokenize;

pub use decode::{decode, Decoded, Pattern, Slot, QUESTION_FRAMES};
pub use label::{label, Caps, ClaimLabel, Flavor, LabelInput, Labeled, Reason, SourceTag, HOP_CAP_ROW};
pub use render::{render_monologue, reverse_decode, Rendered};
pub use teach::{decide, split_line, LineKind, TeachDecision, TeachOutcome, LEFT_ACCEPT_FRAMES};
pub use tokenize::{keys, tokenize, Token};
