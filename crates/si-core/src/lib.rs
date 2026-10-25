//! si-core — the kernel of engineering/00 §2.
//!
//! Modules implement 01 §1–§3, §9–§10 (address, shelves, grids, containers, hash),
//! 01 §6 / 07 §11 (the packed frozen records), 07 §M.1 (the manifest's source
//! pins), 03 §1–§8 (the constants registry and its OD-86 placeholders), 04 §10
//! (the one recency curve), 10 §3–§4 (the birth seed's derivation arithmetic) and
//! 13 §5 / 04 §34 (the tongue's own drawn order).
#![forbid(unsafe_code)]

pub mod addr;
pub mod clock;
pub mod consts;
pub mod container;
pub mod encode;
pub mod episodic;
pub mod eventlog;
pub mod frame;
pub mod genome;
pub mod hash;
pub mod patterns;
pub mod quant;
pub mod record;
pub mod rng;
pub mod seed;
pub mod sha256;
pub mod shelves;
pub mod strings;
pub mod temperament;
pub mod tongue;
pub mod world;

/// 01 §4–§5: the record widths live beside their encoders in `record`.
pub use record::{EDGE_BYTES as EDGE_RECORD_BYTES, RECORD_BYTES as RECORD_FIXED_BYTES, STAMP_BYTES};
/// 01 §6: a frozen genome record is packed, not the 112 B envelope.
pub const GENOME_REC_BYTES: [usize; 3] = [32, 16, 16];
/// 03 §8: eleven registered shelves, nine real and two reserved.
pub const SHELF_COUNT: usize = 11;
/// 03 §8: six raw signals, six labels, six doors, six organs.
pub const SIGNAL_KIND_COUNT: u8 = 6;
pub const LABEL_COUNT: u8 = 6;
pub const DOOR_COUNT: u8 = 6;
pub const ORGAN_COUNT: u8 = 6;
/// 03 §8: the closed innate pattern table (nine named frames plus `Undecoded`).
pub const PATTERN_TABLE_SIZE: usize = 10;
/// 03 §8: `CONF_CAP` is 99; 100 is unrepresentable.
pub const CONF_MAX: u16 = 99;
/// 03 §6: three discrete novelty tiers on the Q12 grid.
pub const NOVELTY_TIERS_Q12: [u16; 3] = [0, 2048, 4096];

/// docs/03 §3.2: the two `source` enum literals, metadata and not vocabulary.
pub const SOURCE_LITERALS: [&str; 2] = ["owner", "grimm"];
/// docs/03 §3.2 / engineering/09: the three TEACH literals, form metadata only.
pub const TEACH_LITERALS: [&str; 3] = ["/teach", "=>", "owner-teach"];

/// 03 §8: every record length is a multiple of four.
pub const fn aligned(bytes: usize) -> bool {
    bytes % 4 == 0
}

pub const fn layout_is_valid() -> bool {
    aligned(RECORD_FIXED_BYTES)
        && aligned(EDGE_RECORD_BYTES)
        && aligned(STAMP_BYTES)
        && aligned(GENOME_REC_BYTES[0])
        && aligned(GENOME_REC_BYTES[1])
        && aligned(GENOME_REC_BYTES[2])
        && RECORD_FIXED_BYTES == 80 + 32
}
