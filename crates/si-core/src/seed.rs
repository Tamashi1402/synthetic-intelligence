//! 10 §4: one birth seed, two streams, a fixed draw order. The derivation is a
//! pure function of `(birth_seed, DERIV_ORDER)`; no lattice, transcript, host,
//! wall clock or OS entropy enters it.

use crate::consts;
use crate::quant::map_fraction;
use crate::rng::{birth_streams, Rng};
use crate::temperament::{DerivId, Temperament, DERIV_ORDER, DERIV_ORDER_LEN};

pub const SEED_FORMAT: u16 = 0x0001;

/// P.2: one draw per row, unconditional, mapped onto the row's own range and
/// grid. Stream R is returned untouched so its sequence depends on nothing but
/// the seed.
pub fn derive_temperament(birth_seed: u64) -> Temperament {
    let (mut stream_t, _stream_r) = birth_streams(birth_seed);
    let mut values = [0u16; DERIV_ORDER_LEN];
    for (index, id) in DERIV_ORDER.iter().enumerate() {
        let row = consts::row(id.registry_name())
            .unwrap_or_else(|| panic!("sealed draw {} has no registry row", id.registry_name()));
        let value = map_fraction(stream_t.next_q12(), row.lo, row.hi, row.grid);
        values[index] = value as u16;
    }
    Temperament::new(values)
}

/// P.3: what is recorded is the digest, never the vector.
pub fn temperament_digest(birth_seed: u64) -> u64 {
    derive_temperament(birth_seed).digest()
}

/// The draw count is a schema fact; `check_temperament_closed` reads this.
pub fn order_is_closed() -> bool {
    DERIV_ORDER.len() == DERIV_ORDER_LEN
        && DERIV_ORDER
            .iter()
            .all(|id| consts::row(id.registry_name()).map(|r| r.class == consts::Class::Temperament).unwrap_or(false))
        && DERIV_ORDER.len() == consts::temperament_count()
}

/// `Rng` is exposed only so a caller can inspect the untouched runtime stream
/// in tests; the derivation itself discards stream T (P.3).
pub fn runtime_stream(birth_seed: u64) -> Rng {
    birth_streams(birth_seed).1
}

pub const _GAIN_IDS: [DerivId; 6] = [
    DerivId::Gain0,
    DerivId::Gain1,
    DerivId::Gain2,
    DerivId::Gain3,
    DerivId::Gain4,
    DerivId::Gain5,
];
