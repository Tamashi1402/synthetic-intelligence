//! Fixtures for the B3 legs: a three-point chain lattice, a params literal and
//! a flat fixture curve. Every number here is a scenario constant, not a
//! registry value; no gate verdict may depend on one (11 §1 D5).

#![allow(dead_code)]

use si_affect::params::LagUnit;
use si_affect::Params;
use si_core::record::{Body, Envelope, Stamp, SUPERSEDED_LIVE};
use si_store::store::{EdgeDraft, Lattice};
use si_walk::{Curve, CurvePoint, OscillationRule, WalkConfig};

pub const A: u32 = 1;
pub const B: u32 = 2;
pub const C: u32 = 3;

/// A seed-format-legal stamp; `a_at` is the level the formation instant read.
pub fn stamp(a_at: u8) -> Stamp {
    Stamp { thought_id: 7, signal_ref: 9, magnitude: 4096, a_at, signal: 1, src: 0 }
}

/// One shelf-4 row whose key point carries the given outgoing edges.
pub fn add_row(lattice: &mut Lattice, source: u32, edges: &[(u32, u16, u16, u8)]) -> u32 {
    let drafts: Vec<EdgeDraft> = edges
        .iter()
        .map(|(to, weight, bake, a_at)| {
            let mut draft = EdgeDraft::owned(*to, *weight, *bake);
            draft.stamps.push(stamp(*a_at));
            draft
        })
        .collect();
    let mut body = Body::default();
    body.pattern_id = 1;
    body.key_point = source;
    lattice.append_row(Envelope::live(4, 0, 1, 0), body, drafts, &[], &[]).unwrap()
}

/// `A → B → C`, both edges weight 1.0, no bake.
pub fn chain() -> Lattice {
    let mut lattice = Lattice::new();
    add_row(&mut lattice, A, &[(B, 4096, 0, 128)]);
    add_row(&mut lattice, B, &[(C, 4096, 0, 128)]);
    lattice
}

pub fn params() -> Params {
    Params {
        lambda_a: 128,
        gain: [0; 6],
        beta: 4096,
        gamma: 4096,
        ask_cap_per_input: 1,
        priority: [4096, 4096, 4096],
        gap_salience_floor: 0,
        dedupe_window: 0,
        bias_cap_share: 4096,
        b_max: 4096,
        k_tanh: 1,
        median_min_n: 1,
        median_shape: 0,
        lag_rounds: 1,
        lag_unit: LagUnit::PerRound,
    }
}

pub fn params_with_lag(lag_rounds: u16) -> Params {
    Params { lag_rounds, ..params() }
}

/// Every level the same row: `reach` edges per source, `retention` of the
/// activation kept by the curve, no settle shift.
pub fn flat_curve(reach: u16, retention_q12: u16) -> Curve {
    Curve::fixture(CurvePoint { reach, decay_q12: retention_q12, settle_shift_q12: 0 })
}

/// `decay_base = 128` is 0.5 on the 1/255 grid, so the fixture curve's
/// retention is the only other factor in `decay`.
pub fn config<'a>(
    params: &'a Params,
    curve: &'a Curve,
    k_max: u16,
    tau_settle: u16,
) -> WalkConfig<'a> {
    WalkConfig {
        params,
        curve,
        k_max,
        tau_settle,
        decay_base: 128,
        hub_exponent_q12: 4096,
        frontier_cap: 16,
        oscillation: OscillationRule::MaxKOnly,
        ghost_max: 0,
        shape_max: 0,
        // 04 §31 AC.1: a caller with no index passes the shipped off state.
        tugs: &[],
        relight_max: 0,
        composition_hint: &[],
    }
}

/// The "live" sentinel, re-exported so a test can build an aged row without
/// pulling the whole record module in.
pub const LIVE: i32 = SUPERSEDED_LIVE;
