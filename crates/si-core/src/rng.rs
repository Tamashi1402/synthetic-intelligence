//! 10 §4 P.1: PCG32 and the serializable `RngState{state, inc, spare}` that is
//! hash item 7. No host entropy is reachable from this module.

use serde::{Deserialize, Serialize};

const MULTIPLIER: u64 = 6364136223846793005;
/// `spare` has no draw in v1; the sentinel keeps the hashed field unambiguous.
pub const NO_SPARE: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RngState {
    pub state: u64,
    pub inc: u64,
    pub spare: u32,
}

impl RngState {
    /// Hash item 7 reads exactly these bytes, little-endian.
    pub fn canonical_bytes(&self) -> [u8; 20] {
        let mut out = [0u8; 20];
        out[0..8].copy_from_slice(&self.state.to_le_bytes());
        out[8..16].copy_from_slice(&self.inc.to_le_bytes());
        out[16..20].copy_from_slice(&self.spare.to_le_bytes());
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng {
    state: u64,
    inc: u64,
    spare: u32,
}

impl Rng {
    /// Keys PCG32 from the birth seed as `tesseract-core/src/rng.rs` does:
    /// zero state, odd increment from the seed, two warm-up advances.
    pub fn seeded(birth_seed: u64) -> Self {
        let mut rng = Self { state: 0, inc: (birth_seed << 1) | 1, spare: NO_SPARE };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(birth_seed);
        rng.next_u32();
        rng
    }

    pub fn from_state(state: RngState) -> Self {
        Self { state: state.state, inc: state.inc, spare: state.spare }
    }

    pub fn state(&self) -> RngState {
        RngState { state: self.state, inc: self.inc, spare: self.spare }
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(MULTIPLIER).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    pub fn next_u64(&mut self) -> u64 {
        let hi = self.next_u32() as u64;
        let lo = self.next_u32() as u64;
        (hi << 32) | lo
    }

    /// One draw, no rejection: the top twelve bits are a Q12 fraction in `0..4096`.
    pub fn next_q12(&mut self) -> u16 {
        (self.next_u32() >> 20) as u16
    }
}

/// 10 §4 P.1: stream T (temperament draws) and stream R (ignition).
/// Both are keyed from the birth seed; R is cloned before T draws, so its
/// sequence does not depend on `DERIV_ORDER`'s length.
pub fn birth_streams(birth_seed: u64) -> (Rng, Rng) {
    let temperament = Rng::seeded(birth_seed);
    let runtime = temperament.clone();
    (temperament, runtime)
}
