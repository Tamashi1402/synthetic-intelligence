//! 13 §5 / 04 §34 AF.1–AF.2: the voice's own derivation order.
//!
//! Five Q12 leans, drawn at wake from the **same** `birth_seed` over a **separate**
//! sealed order, so `DERIV_ORDER`'s eighteen rows, `SEED_FORMAT` and
//! `temperament_digest` do not move (**OD-165**). Like the 18, the vector is never
//! stored and never hashed — only its digest is read, and only into a seed mix
//! (P.3, the same discipline [10 §4](../../engineering/10-temperament-sleep.md)
//! states for temperament).
//!
//! The stream is salted by [`TONGUE_DOMAIN`] rather than a continuation of stream
//! T: `birth_streams` hands back two clones of one keying, so continuing T would
//! make `STYLE_SPREAD` a function of `DERIV_ORDER`'s *length* — one future
//! OD-71 flip would silently re-voice every individual. Stream R is not read here
//! at all: a voice must not move the ignition history.

use md5::{Digest, Md5};

use crate::consts::{self, Class};
use crate::quant::map_fraction;
use crate::rng::Rng;

/// `"TONG"` plus a format tag. A constant, so the derivation stays a pure function
/// of `(birth_seed, DERIV_ORDER_TONGUE)`.
pub const TONGUE_DOMAIN: u64 = 0x544f_4e47_0001_0001;
pub const STYLE_ORDER_LEN: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StyleId {
    Spread,
    Depth,
    Lean,
    Recency,
    Proximity,
}

impl StyleId {
    /// The registry row this draw fills, by the registry's own name.
    pub const fn registry_name(self) -> &'static str {
        match self {
            StyleId::Spread => "STYLE_SPREAD",
            StyleId::Depth => "STYLE_DEPTH",
            StyleId::Lean => "STYLE_LEAN",
            StyleId::Recency => "STYLE_RECENCY",
            StyleId::Proximity => "STYLE_PROXIMITY",
        }
    }
}

/// The five `TONGUE`-class rows in the registry's reading order (03 §5.4.1).
pub const DERIV_ORDER_TONGUE: [StyleId; STYLE_ORDER_LEN] = [
    StyleId::Spread,
    StyleId::Depth,
    StyleId::Lean,
    StyleId::Recency,
    StyleId::Proximity,
];

/// The five drawn leans. Ten bytes, `u16le` in the sealed order, exactly the
/// [`crate::temperament::Temperament`] shape one class over.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    values: [u16; STYLE_ORDER_LEN],
}

impl Style {
    pub const fn new(values: [u16; STYLE_ORDER_LEN]) -> Self {
        Self { values }
    }

    pub const fn values(&self) -> &[u16; STYLE_ORDER_LEN] {
        &self.values
    }

    pub fn get(&self, id: StyleId) -> u16 {
        let index = DERIV_ORDER_TONGUE
            .iter()
            .position(|d| *d == id)
            .expect("sealed style id");
        self.values[index]
    }

    /// 13 §34.3 step 1: the budget's price is drawn, its ceiling is the species row.
    pub fn spread(&self) -> u16 {
        self.get(StyleId::Spread)
    }

    /// Fixed order, u16le per value — the canonical byte string, ten bytes.
    pub fn canonical_bytes(&self) -> [u8; STYLE_ORDER_LEN * 2] {
        let mut out = [0u8; STYLE_ORDER_LEN * 2];
        for (i, v) in self.values.iter().enumerate() {
            out[i * 2..i * 2 + 2].copy_from_slice(&v.to_le_bytes());
        }
        out
    }

    /// The first eight MD5 bytes, little-endian: the 18's own digest rule.
    pub fn digest(&self) -> u64 {
        let digest = Md5::digest(self.canonical_bytes());
        let mut out = [0u8; 8];
        out.copy_from_slice(&digest[..8]);
        u64::from_le_bytes(out)
    }
}

/// The salted stream, keyed and discarded per call: the derivation is a pure
/// function, so a wake re-derives rather than stores.
pub fn style_stream(birth_seed: u64) -> Rng {
    Rng::seeded(birth_seed ^ TONGUE_DOMAIN)
}

/// P.2's rule, one class over: one draw per row, mapped onto the row's own range
/// and grid. A row outside `Class::Tongue` is a registry contradiction, not a
/// fallback — `style_order_is_closed` is the mechanical half of that claim.
pub fn derive_style(birth_seed: u64) -> Style {
    let mut stream = style_stream(birth_seed);
    let mut values = [0u16; STYLE_ORDER_LEN];
    for (index, id) in DERIV_ORDER_TONGUE.iter().enumerate() {
        let row = consts::row(id.registry_name())
            .unwrap_or_else(|| panic!("sealed style draw {} has no registry row", id.registry_name()));
        if row.class != Class::Tongue {
            panic!("{} is drawn but is not a TONGUE row", row.name);
        }
        values[index] = map_fraction(stream.next_q12(), row.lo, row.hi, row.grid) as u16;
    }
    Style::new(values)
}

/// What a caller outside the kernel may read: the digest, never the vector.
pub fn style_digest(birth_seed: u64) -> u64 {
    derive_style(birth_seed).digest()
}

/// **OD-183** / 04 §34 AF.2: three operands and nothing else. The state hash is
/// *not* one of them — item 3 of the nine hashed items is the stamp arena, a stamp
/// carries `a_at`, so a seed mixed from the hash would let arousal choose the words
/// through the back door. That is what **OD-168** bans and **G20(e)** grades.
pub fn render_seed(birth_seed: u64, style_digest: u64, thought_id: u32) -> u64 {
    let mut buffer = [0u8; 20];
    buffer[0..8].copy_from_slice(&birth_seed.to_le_bytes());
    buffer[8..16].copy_from_slice(&style_digest.to_le_bytes());
    buffer[16..20].copy_from_slice(&thought_id.to_le_bytes());
    let digest = Md5::digest(buffer);
    let mut out = [0u8; 8];
    out.copy_from_slice(&digest[..8]);
    u64::from_le_bytes(out)
}

/// `check_tongue_closed` reads this: the style draw count is a schema fact, and the
/// 18 keep their own.
pub fn style_order_is_closed() -> bool {
    DERIV_ORDER_TONGUE.len() == STYLE_ORDER_LEN
        && DERIV_ORDER_TONGUE.iter().all(|id| {
            consts::row(id.registry_name()).map(|r| r.class == Class::Tongue).unwrap_or(false)
        })
        && STYLE_ORDER_LEN == consts::tongue_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_style_order_is_closed_and_separate_from_the_18() {
        assert!(style_order_is_closed());
        assert_eq!(crate::temperament::DERIV_ORDER_LEN, 18);
        assert_eq!(STYLE_ORDER_LEN, 5);
    }

    #[test]
    fn derive_style_is_a_pure_function_of_the_seed() {
        for seed in [0u64, 1, 7, u64::MAX] {
            assert_eq!(derive_style(seed), derive_style(seed), "seed {seed} drew twice");
        }
    }

    #[test]
    fn the_style_draw_is_not_the_temperament_draw() {
        // The salt is the whole argument: an unsalted continuation of stream T
        // would make these two sequences overlap, so the drawn vector is pinned
        // rather than compared. Change the salt or the order and this is red.
        let drawn = derive_style(0).values().to_vec();
        assert_eq!(drawn, STYLE_GOLDEN_SEED_ZERO, "seed 0's voice moved: {drawn:?}");
    }

    /// `derive_style(0)`, pinned the way `EMPTY_STATE_MD5_SEED_ZERO` is: the salted
    /// stream plus the sealed order are a schema fact, not a preference.
    const STYLE_GOLDEN_SEED_ZERO: [u16; STYLE_ORDER_LEN] = [1408, 1856, 768, 1872, 240];

    #[test]
    fn a_style_lands_inside_its_row_box() {
        for seed in 0u64..64 {
            let style = derive_style(seed);
            for value in style.values() {
                assert!(*value <= 2048, "{seed} drew {value} past the box");
                assert_eq!(value % 16, 0, "{seed} drew {value} off the grid");
            }
        }
    }

    #[test]
    fn both_box_corners_are_reachable() {
        // 13 §3.1: `hi = 2048` is chosen so OD-161's corner rule holds — a span of
        // at most 2048 is the largest box a Q12 draw can land on both ends of.
        let mut low = None;
        let mut high = None;
        for seed in 0u64..4096 {
            for (index, value) in derive_style(seed).values().iter().enumerate() {
                if *value == 0 {
                    low.get_or_insert(index);
                }
                if *value == 2048 {
                    high.get_or_insert(index);
                }
            }
        }
        assert!(low.is_some(), "no seed drew the floor");
        assert!(high.is_some(), "no seed drew the ceiling");
    }

    #[test]
    fn canonical_bytes_is_ten_bytes_and_digest_is_stable() {
        let style = Style::new([0, 16, 2048, 1024, 512]);
        assert_eq!(style.canonical_bytes().len(), 10);
        assert_eq!(style.digest(), Style::new([0, 16, 2048, 1024, 512]).digest());
        assert_ne!(style.digest(), Style::new([16, 0, 2048, 1024, 512]).digest());
    }

    #[test]
    fn render_seed_is_a_function_of_three_operands() {
        let digest = Style::new([0, 16, 2048, 1024, 512]).digest();
        assert_eq!(render_seed(9, digest, 3), render_seed(9, digest, 3));
        assert_ne!(render_seed(9, digest, 3), render_seed(9, digest, 4));
        assert_ne!(render_seed(9, digest, 3), render_seed(10, digest, 3));
        assert_ne!(render_seed(9, digest, 3), render_seed(9, digest + 1, 3));
    }
}
