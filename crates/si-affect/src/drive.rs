//! 02 §3 / 03 §4 / 04 §6–§8, §11 K.3–K.5, §16: the drive interface, and it is
//! exactly six doors. The walk makes two call sites (`update`, `conduction`);
//! the read path makes the other four. None of them takes a level it should not:
//! `budget` and `emit` read no `a` (OD-22).

use si_core::clock;
use si_core::quant::{mul_q12_u16, u8_to_q12};
use si_core::record::Stamp;

use crate::arousal::{Integrator, Lagged};
use crate::bake::bake_q12;
use crate::params::Params;
use crate::signal::{Signal, SignalKind};

/// The closed door list. `check_drive_doors` reads this and the module's `pub fn`
/// count; a seventh door is a design change, not a helper.
pub const DOORS: [&str; 6] = ["prioritize", "emit", "budget", "salience", "update", "conduction"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    pub cap_per_input: u8,
    pub dedupe_window: u16,
    pub salience_floor: u16,
}

/// 04 §11 K.4: pure queue arithmetic. No `a`, no level, no signal (OD-22).
pub fn budget(params: &Params) -> Budget {
    Budget {
        cap_per_input: params.ask_cap_per_input,
        dedupe_window: params.dedupe_window,
        salience_floor: params.gap_salience_floor,
    }
}

/// A gap is informational: how much bears on the slot, when it was last pulsed,
/// and how often. None of it is affective (K.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gap {
    pub slot: u32,
    pub salience_q12: u16,
    pub last_pulsed_at: i32,
    pub usage_q12: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ranked {
    pub gap: Gap,
    pub weight_q12: u16,
}

/// 04 §11 K.3: salience × recency × usage, ties by slot ascending.
pub fn prioritize(gaps: &[Gap], now: i32, params: &Params) -> Vec<Ranked> {
    let mut ranked: Vec<Ranked> = gaps
        .iter()
        .map(|gap| {
            let recency = u8_to_q12(clock::recency(now, gap.last_pulsed_at));
            let salience = mul_q12_u16(gap.salience_q12, params.priority[0]);
            let recency = mul_q12_u16(recency, params.priority[1]);
            let usage = mul_q12_u16(gap.usage_q12, params.priority[2]);
            let weight = mul_q12_u16(mul_q12_u16(salience, recency), usage);
            Ranked { gap: *gap, weight_q12: weight }
        })
        .collect();
    ranked.sort_by(|a, b| b.weight_q12.cmp(&a.weight_q12).then(a.gap.slot.cmp(&b.gap.slot)));
    ranked
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ask {
    pub slot: u32,
}

/// 04 §11 K.5: floor, then the per-input ceiling. A suppressed gap is still a
/// gap; this door only decides what is spoken now.
pub fn emit(ranked: &[Ranked], params: &Params) -> Vec<Ask> {
    ranked
        .iter()
        .filter(|entry| entry.gap.salience_q12 >= params.gap_salience_floor)
        .take(params.ask_cap_per_input as usize)
        .map(|entry| Ask { slot: entry.gap.slot })
        .collect()
}

/// The formation instant's inputs. The stamp needs the firing kind and its own
/// value; the bake law may not read either of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Formation {
    pub at_t: i32,
    pub thought_id: u32,
    pub signal_ref: u32,
    pub signal: SignalKind,
    pub magnitude: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Salience {
    pub bake: u16,
    pub stamp: Stamp,
}

/// 04 §16 P: the only assignment of `bake` in the build. The scalar argument is
/// `a_formation`; nothing categorizes the event (OD-31, R-21(c)).
pub fn salience(a_formation: u8, formation: &Formation, params: &Params) -> Salience {
    let bake = bake_q12(a_formation, params.b_max, params.k_tanh);
    let stamp = Stamp {
        thought_id: formation.thought_id,
        signal_ref: formation.signal_ref,
        magnitude: formation.magnitude,
        a_at: a_formation,
        signal: formation.signal as u8,
        src: 0,
    };
    Salience { bake, stamp }
}

/// 04 §5 step 2 / §7: integrate the round's signals and return the value the
/// lagged pointer will conduct with (step 3).
pub fn update(
    integrator: &mut Integrator,
    signals: &[Signal],
    round: u16,
    thought: u32,
    params: &Params,
    trajectory: &mut Vec<(u16, u8)>,
) -> Lagged {
    integrator.update(signals, round, thought, params, trajectory)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeCharge {
    pub edge_off: u32,
    pub to_point: u32,
    pub bake: u16,
    pub a_at: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointMedian {
    pub point: u32,
    pub median_q12: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeBias {
    pub edge_off: u32,
    pub sigma_q12: u16,
    pub median_term_q12: u16,
    pub bias_q12: u16,
    pub a_now: u8,
    pub a_at: u8,
    pub median_used: u16,
}

/// 04 §8: `β·σ + γ·m`, clipped **once** against `BIAS_CAP_SHARE` of the round's
/// total conduction. `σ`'s falloff shape is OPEN-18's; it lives behind this
/// function so the sweep replaces no caller.
pub fn conduction(
    a_now: u8,
    edges: &[EdgeCharge],
    medians: &[PointMedian],
    base_total_q12: u16,
    params: &Params,
) -> Vec<EdgeBias> {
    let mut sorted = edges.to_vec();
    sorted.sort_by_key(|edge| edge.edge_off);
    let mut raw = Vec::with_capacity(sorted.len());
    let mut total_raw: u64 = 0;
    for edge in &sorted {
        let median = medians
            .iter()
            .find(|entry| entry.point == edge.to_point)
            .map(|entry| entry.median_q12)
            .unwrap_or(0);
        let diff = a_now.abs_diff(edge.a_at) as u16;
        let falloff_q12 = (255 - diff) << 4;
        let sigma = mul_q12_u16(mul_q12_u16(edge.bake, falloff_q12), params.beta);
        let median_term = mul_q12_u16(median, params.gamma);
        let sum = sigma.saturating_add(median_term);
        total_raw += sum as u64;
        raw.push((edge, sigma, median_term, sum, median));
    }
    let cap_total = mul_q12_u16(base_total_q12, params.bias_cap_share) as u64;
    let reciprocal = if total_raw > cap_total && total_raw > 0 {
        ((cap_total << 32) / total_raw) as u64
    } else {
        1u64 << 32
    };
    raw.into_iter()
        .map(|(edge, sigma, median_term, sum, median)| {
            let scaled = ((sum as u64 * reciprocal) >> 32).min(u16::MAX as u64) as u16;
            EdgeBias {
                edge_off: edge.edge_off,
                sigma_q12: sigma,
                median_term_q12: median_term,
                bias_q12: scaled,
                a_now,
                a_at: edge.a_at,
                median_used: median,
            }
        })
        .collect()
}
