//! 04 §5 step 7 / 03 §2: `hub_damp(deg) = deg^(−HUB_DAMP_EXPONENT)`.
//!
//! The exponent is Q12 and the result is Q12, so the power is built from a Q32
//! `ln`/`exp` pair in integer arithmetic — no float exists in the build
//! (01 §3, §16.6). One table per graph, over the degrees the graph actually
//! reaches (06 §4 trap 1: the counter says which), never per point per round.

use std::collections::{BTreeMap, BTreeSet};

use si_core::quant::Q12_ONE;

const FRAC: u32 = 32;
const ONE: i128 = 1 << FRAC;
/// round(ln 2 · 2³²).
const LN2: i128 = 2_977_044_472;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HubDamp {
    table: BTreeMap<u32, u16>,
}

impl HubDamp {
    pub(crate) fn build(degrees: BTreeSet<u32>, exponent_q12: u16) -> Self {
        let table = degrees
            .into_iter()
            .map(|degree| (degree, damp_q12(degree, exponent_q12)))
            .collect();
        Self { table }
    }

    pub(crate) fn at(&self, degree: u32) -> u16 {
        self.table.get(&degree).copied().unwrap_or(Q12_ONE)
    }
}

fn damp_q12(degree: u32, exponent_q12: u16) -> u16 {
    if degree <= 1 {
        return Q12_ONE;
    }
    let ln = ln_q32(degree as u64);
    let z = (exponent_q12 as i128 * ln) / Q12_ONE as i128;
    let value = exp_neg_q32(z);
    ((value + (1 << 19)) >> 20).clamp(0, Q12_ONE as i128) as u16
}

fn ln_q32(x: u64) -> i128 {
    debug_assert!(x >= 1);
    if x == 1 {
        return 0;
    }
    let shift = 63 - x.leading_zeros() as i32;
    let mantissa = if shift <= FRAC as i32 {
        (x as i128) << (FRAC as i32 - shift)
    } else {
        (x as i128) >> (shift - FRAC as i32)
    };
    let z = ((mantissa - ONE) << FRAC) / (mantissa + ONE);
    let z2 = (z * z) >> FRAC;
    let mut term = z;
    let mut sum = z;
    for k in 1..=24 {
        term = (term * z2) >> FRAC;
        let add = term / (2 * k + 1) as i128;
        sum += add;
        if add == 0 {
            break;
        }
    }
    (shift as i128) * LN2 + 2 * sum
}

fn exp_neg_q32(z: i128) -> i128 {
    if z <= 0 {
        return ONE;
    }
    let quotient = (z / LN2) as u32;
    let remainder = z - (quotient as i128) * LN2;
    let mut term = ONE;
    let mut sum = ONE;
    for n in 1..=32 {
        term = -(term * remainder) / (n as i128 * ONE);
        sum += term;
        if term == 0 {
            break;
        }
    }
    if quotient >= FRAC {
        0
    } else {
        sum >> quotient
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_four_degrees_are_exact_for_a_unit_exponent() {
        assert_eq!(damp_q12(1, 4096), 4096);
        assert_eq!(damp_q12(2, 4096), 2048);
        assert_eq!(damp_q12(4, 4096), 1024);
        assert_eq!(damp_q12(8, 4096), 512);
    }

    #[test]
    fn a_half_exponent_is_a_square_root_and_monotone() {
        let two = damp_q12(2, 2048);
        assert!((2896..=2897).contains(&two), "2^-0.5 on Q12 is {two}");
        let mut previous = 4096;
        for degree in 1..=64 {
            let value = damp_q12(degree, 4096);
            assert!(value <= previous, "damping never brightens");
            previous = value;
        }
    }
}
