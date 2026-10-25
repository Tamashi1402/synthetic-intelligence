//! 04 §19 S / 02 §4: the idle tick — fixed precedence, one seeded sampler,
//! and a cascade that is the same walk with its lighting drawn instead of read.
//!
//! The tick may not write: it reads the live web (through `Graph`, an immutable
//! borrow) and the caller's own quiet-time statistics, and returns an action.
//! The cascade borrows `Integrator` (session state) and nothing from the store.
//! `si-walk` has no `si-learn` edge, which is I9's structural guarantee.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;

use si_affect::drive::PointMedian;
use si_affect::{Integrator, Params};
use si_core::clock;
use si_core::quant::{mul_q12_u16, u8_to_q12, Q12_ONE};
use si_core::rng::Rng;

use crate::graph::Graph;
use crate::walk::{walk, Lit, Thought, WalkConfig};
use crate::LIGHT_QUANTUM;

/// The five ignition weights. The first four are 03 §5's TEMPERAMENT draws (the
/// caller supplies the individual's own values; nothing here is a value); the
/// fifth is `IGN_WEIGHT_SILT`, a SPECIES row that ships unarmed at 0 (OD-108,
/// engineering/12 §5 §30), so the arithmetic below is unchanged at that default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdleWeights {
    pub recency_q12: u16,
    pub bake_q12: u16,
    pub decay_deficit_q12: u16,
    pub contradiction_q12: u16,
    pub silt_q12: u16,
}

/// A queue row the tick may resolve. Resolving it is `si-session`'s bookkeeping
/// of a `questions` row, never a belief write (04 §19 S.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuestionRef {
    pub question_id: u32,
    pub asked_at: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Agenda {
    pub open_questions: Vec<QuestionRef>,
    pub reparse_due: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdleAction {
    Revisit { question_id: u32 },
    Reparse { row_id: u32 },
    Ignition { point: u32 },
    Nothing,
}

/// What the tick reads. `last_lit` is session state; `decay_deficit` and
/// `contradiction_warmth` are the caller's read-side statistics — the tick
/// combines them, it does not invent them. `idles_this_window` and
/// `per_window` implement `IDLE_PER_HOUR` in logical ticks (§S.6); the caller
/// owns the window's start because the design defines no wall-clock hour (I5).
pub struct IdleInputs<'a> {
    pub senses_quiet: bool,
    pub agenda: &'a Agenda,
    pub idles_this_window: u32,
    pub per_window: u16,
    pub last_lit: &'a BTreeMap<u32, i32>,
    pub decay_deficit: &'a BTreeMap<u32, u16>,
    pub contradiction_warmth: &'a BTreeSet<u32>,
    /// `silt::view(conversation)` — the fifth term's only input. It is an
    /// argument, never an edge: `si-walk` does not depend on `si-silt` (12 §1.1).
    pub silt: &'a BTreeMap<u32, u16>,
}

/// 04 §19 S.1–S.3. Precedence is revisit → due RE-PARSE → ignition; ignition is
/// only reached when the agenda is empty and the rate allows it. One call is
/// one quiet tick, so ≤ `IDLE_PER_TICK = 1` idle thought is structural.
pub fn idle_tick(
    graph: &Graph,
    now: i32,
    inputs: &IdleInputs<'_>,
    weights: &IdleWeights,
    params: &Params,
    rng: &mut Rng,
) -> IdleAction {
    if !inputs.senses_quiet {
        return IdleAction::Nothing;
    }
    if let Some(question) = inputs
        .agenda
        .open_questions
        .iter()
        .min_by_key(|question| (question.asked_at, question.question_id))
    {
        return IdleAction::Revisit { question_id: question.question_id };
    }
    if let Some(row_id) = inputs.agenda.reparse_due {
        return IdleAction::Reparse { row_id };
    }
    if inputs.idles_this_window >= inputs.per_window as u32 {
        return IdleAction::Nothing;
    }
    match ignite(
        graph,
        now,
        inputs.last_lit,
        inputs.decay_deficit,
        inputs.contradiction_warmth,
        inputs.silt,
        weights,
        params,
        rng,
    ) {
        Some(point) => IdleAction::Ignition { point },
        None => IdleAction::Nothing,
    }
}

/// 04 §19 S.3 / §30 AB.2: five weights summed on Q12, scanned in ascending point
/// handle, one draw from the seeded stream R. It reads the stored `bake` and the
/// caller's silt view and writes nothing (OPEN-32 closed); `bake <= B_max` bounds
/// its share by construction, and `silt_level = 0` for an absent point.
#[allow(clippy::too_many_arguments)]
pub fn ignite(
    graph: &Graph,
    now: i32,
    last_lit: &BTreeMap<u32, i32>,
    decay_deficit: &BTreeMap<u32, u16>,
    contradiction_warmth: &BTreeSet<u32>,
    silt: &BTreeMap<u32, u16>,
    weights: &IdleWeights,
    params: &Params,
    rng: &mut Rng,
) -> Option<u32> {
    let bake_max = graph.bake_touch_max();
    let candidates: Vec<u32> = graph.points().collect();
    let mut cumulative: Vec<(u32, u64)> = Vec::with_capacity(candidates.len());
    let mut total: u64 = 0;
    for point in candidates {
        let recency = last_lit
            .get(&point)
            .map(|last| clock::recency(now, *last))
            .unwrap_or(0);
        let recency_term = mul_q12_u16(u8_to_q12(recency), weights.recency_q12);

        let bake = bake_max.get(&point).copied().unwrap_or(0);
        let bake_term = mul_q12_u16(bake_fraction_q12(bake, params.b_max), weights.bake_q12);

        let deficit = decay_deficit.get(&point).copied().unwrap_or(0).min(Q12_ONE);
        let deficit_term = mul_q12_u16(deficit, weights.decay_deficit_q12);

        let warmth = if contradiction_warmth.contains(&point) { Q12_ONE } else { 0 };
        let warmth_term = mul_q12_u16(warmth, weights.contradiction_q12);

        // §30 AB.2's one arithmetic change: at the shipped `IGN_WEIGHT_SILT = 0`
        // this addend is 0 for every point and the Q12 sum is numerically
        // identical to B3's four-term scan.
        let silt_level = silt.get(&point).copied().unwrap_or(0);
        let silt_term = mul_q12_u16(weights.silt_q12, silt_level);

        let weight = recency_term as u64
            + bake_term as u64
            + deficit_term as u64
            + warmth_term as u64
            + silt_term as u64;
        if weight == 0 {
            continue;
        }
        total = total.saturating_add(weight);
        cumulative.push((point, total));
    }
    if total == 0 {
        return None;
    }
    let fraction = rng.next_q12() as u64;
    let target = (total * fraction) >> 12;
    cumulative
        .iter()
        .find(|(_, sum)| target < *sum)
        .map(|(point, _)| *point)
        .or_else(|| cumulative.last().map(|(point, _)| *point))
}

/// 04 §19 S.4: the sampled point is the round-0 lighting and §5 runs unchanged.
/// The cascade borrows the graph immutably and mutates only the session's
/// integrator; it cannot form an edge because no write path is in scope.
pub fn cascade(
    graph: &Graph,
    point: u32,
    integrator: &mut Integrator,
    thought_id: u32,
    medians: &[PointMedian],
    config: &WalkConfig<'_>,
) -> Result<Thought> {
    walk(
        graph,
        &[Lit { point, activation: LIGHT_QUANTUM }],
        integrator,
        thought_id,
        &[],
        medians,
        config,
    )
}

/// `bake / B_max` on Q12. The registry's legal `B_MAX` range starts at 1; a
/// zero would divide by zero, so it contributes no weight rather than panicking.
fn bake_fraction_q12(bake: u16, b_max: u16) -> u16 {
    if b_max == 0 {
        return 0;
    }
    let bake = bake.min(b_max) as u64;
    let reciprocal = (1u64 << 32) / b_max as u64;
    let scaled = bake.saturating_mul(reciprocal);
    (((scaled + (1 << 19)) >> 20) as u16).min(Q12_ONE)
}
