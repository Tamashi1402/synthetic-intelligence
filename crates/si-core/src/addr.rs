//! 01 §1: the address `(t, x, y, z)` and hyperpoint identity.
//! 01 §5.2: the closed edge-kind enum.

use md5::{Digest, Md5};

pub type PointHandle = u32;

pub const SHELF_DICTIONARY: u16 = 1;
pub const SHELF_RELATIONS: u16 = 2;
pub const SHELF_RULES: u16 = 3;
pub const SHELF_S_R_PAIRS: u16 = 4;
pub const SHELF_FACTS: u16 = 5;
pub const SHELF_LINKS: u16 = 6;
pub const SHELF_QUESTIONS: u16 = 7;
pub const SHELF_CONTEXT: u16 = 8;
pub const SHELF_SKILLS: u16 = 9;
pub const SHELF_STRATEGIES: u16 = 10;
pub const SHELF_EVENTS: u16 = 11;

pub const EDGE_KIND_OWNED_ATTRIBUTE: u8 = 0;
pub const EDGE_KIND_OWNED_DEFINITION: u8 = 1;
pub const EDGE_KIND_LINK_MEANS: u8 = 2;
pub const EDGE_KIND_LINK_SAME_QUESTION: u8 = 3;
pub const EDGE_KIND_LINK_FRAME_SYNONYM: u8 = 4;
pub const EDGE_KIND_LINK_SLOT_UNIQUE: u8 = 5;
pub const EDGE_KIND_COOCCUR_WEAK: u8 = 6;
pub const EDGE_KIND_GENOME_FIRST: u8 = 7;
pub const EDGE_KIND_OWNED_LAST: u8 = 6;

pub const EDGE_FLAG_BORN_THIS_STEP: u8 = 1 << 0;
pub const EDGE_FLAG_FROZEN_GENOME: u8 = 1 << 1;
pub const EDGE_FLAG_HISTORY_ONLY: u8 = 1 << 2;

/// 01 §1: `(t, x, y)`; `z` is the edge arena offset held by the record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Addr {
    pub t: i32,
    pub x: u16,
    pub y: u32,
}

impl Addr {
    pub const fn new(t: i32, x: u16, y: u32) -> Self {
        Self { t, x, y }
    }

    /// 01 §2: the append-only layer, read out of the shelf registry rather than
    /// re-declared here — 12 §2.2 opens shelf 9 and this follows it.
    pub const fn is_live_shelf(&self) -> bool {
        crate::shelves::is_l3_live(self.x)
    }
}

/// 01 §1: the first eight bytes of md5 over the canonical key string.
/// Identity, not security: two records that hash equal are the same point.
pub fn hyperpoint_id(key: &str) -> u64 {
    let digest = Md5::digest(key.as_bytes());
    let mut out = [0u8; 8];
    out.copy_from_slice(&digest[..8]);
    u64::from_le_bytes(out)
}

/// 03 §8: the genome slots are indices into a sealed kind table, capped at 256.
pub const fn edge_kind_is_owned(kind: u8) -> bool {
    kind <= EDGE_KIND_OWNED_LAST
}

pub const fn edge_kind_is_genome(kind: u8) -> bool {
    kind >= EDGE_KIND_GENOME_FIRST
}

/// docs/03 §3.2: the claim kinds are 0..=5. `edge_kind_is_owned`'s last slot *is*
/// `6 cooccur-weak` — it is the record-shape predicate, which columns the edge has
/// — so the semantic question "is this a claim" needs its own name (**OD-186**).
pub const fn edge_kind_is_claim(kind: u8) -> bool {
    kind <= EDGE_KIND_OWNED_LAST && kind != EDGE_KIND_COOCCUR_WEAK
}
