//! 04 §5: the eleven ordered steps, once per round, and the settle test.
//!
//! The round is fixed: gather → `drive.update` → advance the lagged pointer →
//! read the curve → `drive.conduction` → spread → hub damp → sum/decay →
//! normalize → trim → test τ. `si-walk` calls exactly those two doors and no
//! other drive surface (02 §4, G8's call-site leg).
//!
//! Two build-fixed readings the step table needs and the design leaves to the
//! build (04 §27's pattern):
//! * Step 3's pointer is `a_used(k) = a produced at k − MOD_LAG_ROUNDS`, and
//!   every round at or before the lag conducts with the thought's starting
//!   value. The two sentences of 04 §5 step 3 agree at the owner-lean lag of 1;
//!   above it, "the value round k + MOD_LAG_ROUNDS conducts with" is the only
//!   operational one.
//! * The round's total conduction is the frontier's plain `Σ act·W·damp`; the
//!   conduction door clips `β·σ + γ·m` against it, so the normalization of step
//!   9 happens before any activation exists and therefore before the τ test.
//! * A settle-failure's own value is the round's `diff − τ` shortfall, a
//!   quantity of the round, never of `a` (OD-22).
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Result};

use si_affect::drive::{self, EdgeBias, EdgeCharge, PointMedian};
use si_affect::{Integrator, Params, Signal, SignalKind};
use si_core::episodic::OriginTug;
use si_core::frame::{CoLight, FrameLog, Ghost, Role, RoleGraph};
use si_core::quant::{mul_q12_u16, u8_to_q12};

use crate::curve::Curve;
use crate::graph::{Arc, Graph};
use crate::hub::HubDamp;

/// A round-0 lighting. The activation is already on the Q12 grid (04 §4.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lit {
    pub point: u32,
    pub activation: i32,
}

/// 04 §5.1 / OPEN-11: both rules end a 2-cycle in the same uncertainty readout;
/// the trace records that the period check fired, the stop reason stays `MaxK`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OscillationRule {
    MaxKOnly,
    PeriodCheck,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    Settled,
    TauMiss,
    MaxK,
    NothingLit,
}

/// One lit point in a round's vector. The stored settle-set is sorted by point
/// id; display order is `Thought::display` (04 §5.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Activation {
    pub point: u32,
    pub activation: i32,
    pub first_lit: u16,
}

/// One traversed edge, in contribution order (04 §5.4's route).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Contribution {
    pub source: u32,
    pub to_point: u32,
    /// A live edge's arena byte offset, or a genome edge's CSR index — two
    /// namespaces that overlap numerically. `genome` says which, and it is the
    /// only thing standing between a dimming pass and a corpus row (G13(e)).
    pub edge_off: u32,
    pub amount: i32,
    pub genome: bool,
    /// 08 §2.1 row 3's `weight_eff_q12`: the effective weight this traversal
    /// actually paid, kept alongside the transfer it produced. Scratch, like the
    /// rest of `RoundTrace` — [01 §10]'s nine hashed classes carry the settle set
    /// and the `a` trajectory, not the route — so a panel can show *why* a round
    /// reached what it reached without the viewer re-deriving a weight.
    pub weight_eff_q12: i32,
}

/// 04 §31 AC.2: one origin re-light, in `(event_id, point)` order. It is the
/// frame's only record of *why* a round-0 light it did not read exists, and it is
/// scratch — hashed nowhere, written nowhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Relight {
    pub event_id: u32,
    pub point: u32,
    pub activation: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoundTrace {
    pub round: u16,
    pub a_used: u8,
    pub a_after: u8,
    pub reach: u16,
    pub decay_q12: u16,
    pub tau_q12: u16,
    pub base_total_q12: u16,
    pub bias_total_q12: u16,
    pub diff_q12: i64,
    /// `h_k`, ascending point id.
    pub activations: Vec<Activation>,
    /// The propagation set kept by the trim, in trim order.
    pub frontier: Vec<Activation>,
    /// Every traversed edge, in contribution order.
    pub contributions: Vec<Contribution>,
    /// `drive.conduction`'s attribution, ascending edge offset (04 §8).
    pub biases: Vec<EdgeBias>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thought {
    /// Stored/hashed order: point id ascending (01 §10 item 5).
    pub settle_set: Vec<Activation>,
    pub rounds: Vec<RoundTrace>,
    pub a_trajectory: Vec<(u16, u8)>,
    pub stop_reason: StopReason,
    pub lit_round0: Vec<Lit>,
    /// 04 §31 AC.2: the frame's origin re-lights. Empty when the traversal is off,
    /// and empty again at settle's exit when no tug fired.
    pub relit: Vec<Relight>,
    pub oscillated: bool,
    /// engineering/12 §2.6: the frame scratch, exported as an observation and
    /// hashed nowhere. Dead at settle; the readers receive it as an argument.
    pub frame_log: FrameLog,
}

impl Thought {
    /// Display order: activation desc, first-lit round asc, point id asc
    /// (04 §5.2's one tie-break convention).
    pub fn display(&self, top_k: usize) -> Vec<Activation> {
        let mut shown = self.settle_set.clone();
        shown.sort_by(|left, right| {
            right
                .activation
                .cmp(&left.activation)
                .then(left.first_lit.cmp(&right.first_lit))
                .then(left.point.cmp(&right.point))
        });
        shown.truncate(top_k);
        shown
    }

    /// The address-free behavioural trace: what two runs of the same graph must
    /// agree on. Arena offsets are citations, not behaviour.
    pub fn behavioural_trace(&self) -> Vec<(u16, Vec<(u32, i32, u16)>)> {
        self.rounds
            .iter()
            .map(|round| {
                (
                    round.round,
                    round
                        .activations
                        .iter()
                        .map(|entry| (entry.point, entry.activation, entry.first_lit))
                        .collect(),
                )
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WalkConfig<'a> {
    pub params: &'a Params,
    pub curve: &'a Curve,
    pub k_max: u16,
    pub tau_settle: u16,
    pub decay_base: u8,
    pub hub_exponent_q12: u16,
    pub frontier_cap: usize,
    pub oscillation: OscillationRule,
    /// 12 §3.1's two frame-scratch bounds; `0` builds none, which is the
    /// registry floor and the pre-arc arithmetic.
    pub ghost_max: u16,
    pub shape_max: u16,
    /// 04 §4 note 7 / §31 AC.1: step 5b's derived origin links. Empty is the
    /// shipped `EPISODIC_TRAVERSAL = off`, and an empty slice is a no-op.
    pub tugs: &'a [OriginTug],
    /// §31 AC.2's per-frame cap. `0` is the registry floor below `RELIGHT_MAX`'s
    /// legal range, so `walk` refuses no call and re-lights nothing.
    pub relight_max: u16,
    /// 04 §32 AD.5 / OD-118's buildable half: the installed skills' root handles,
    /// as a **candidate-iteration order**. It reorders only which arcs survive a
    /// source's reach truncation — no weight, no threshold, no sampler, no RNG,
    /// and it cannot reach a commit. Empty is `SKILL_COMPOSITION = off`, and an
    /// empty hint reduces the comparator to the shipped arithmetic bit for bit.
    pub composition_hint: &'a [u32],
}

/// 04 §5 steps 1–11. `graph` is anchored; `integrator` is the session's one
/// integrator; `medians` come from `si-affect` when a point has a reading, and
/// are empty at birth (§9, §20.3). The trajectory is the hashed artifact, not
/// just its endpoint (04 §7).
pub fn walk(
    graph: &Graph,
    lit: &[Lit],
    integrator: &mut Integrator,
    thought_id: u32,
    initial_signals: &[Signal],
    medians: &[PointMedian],
    config: &WalkConfig<'_>,
) -> Result<Thought> {
    if config.k_max == 0 {
        bail!("K_MAX_ROUNDS is 0; a thought has at least one round (03 §2)");
    }
    if config.frontier_cap == 0 {
        bail!("FRONTIER_CAP is 0; the trim would drop the lighting itself (03 §2)");
    }
    for entry in lit {
        if entry.activation < 0 {
            bail!("lit point {} carries a negative activation", entry.point);
        }
    }

    let mut h: BTreeMap<u32, i32> = BTreeMap::new();
    let mut first_lit: BTreeMap<u32, u16> = BTreeMap::new();
    for entry in lit {
        if entry.activation == 0 {
            continue;
        }
        let value = h.entry(entry.point).or_insert(0);
        *value = value.saturating_add(entry.activation);
        first_lit.entry(entry.point).or_insert(0);
    }
    if h.is_empty() {
        return Ok(Thought {
            settle_set: Vec::new(),
            rounds: Vec::new(),
            a_trajectory: Vec::new(),
            stop_reason: StopReason::NothingLit,
            lit_round0: lit.to_vec(),
            relit: Vec::new(),
            oscillated: false,
            frame_log: FrameLog::default(),
        });
    }
    // 04 §4 note 7 / §31 AC.2: step 5b's tugs light after the token lights and
    // before round 1, in `(event_id, point)` order, capped at `RELIGHT_MAX` over
    // the frame. The value is on the grid already, the point joins round 0 as an
    // ordinary light, and nothing is written. The early return above is why an
    // empty-token line stays `NothingLit` even when a tug exists.
    let mut relit: Vec<Relight> = Vec::new();
    let mut tugged: BTreeSet<u32> = BTreeSet::new();
    let tug_cap = config.relight_max as usize;
    if tug_cap > 0 && !config.tugs.is_empty() {
        let mut pairs: Vec<(u32, u32, u16)> = Vec::new();
        for tug in config.tugs {
            for point in &tug.points {
                pairs.push((tug.event_id, *point, tug.weight_q12));
            }
        }
        pairs.sort_by_key(|pair| (pair.0, pair.1));
        for (event_id, point, weight) in pairs {
            if relit.len() >= tug_cap {
                break;
            }
            if weight == 0 {
                continue;
            }
            let activation = i32::from(weight);
            let value = h.entry(point).or_insert(0);
            *value = value.saturating_add(activation);
            first_lit.entry(point).or_insert(0);
            tugged.insert(point);
            relit.push(Relight { event_id, point, activation });
        }
    }
    // engineering/12 §2.6: round 0's lighting is the frame's seed pulse. A point
    // the index re-lit carries `primed`, OD-105's tag for a frame a synthetic
    // light brought into being; the token lights do not.
    let mut lights: Vec<CoLight> = h
        .keys()
        .map(|point| CoLight {
            point: *point,
            role: Role::Seed,
            edge_kind: 0,
            round: 0,
            primed: tugged.contains(point),
        })
        .collect();
    let mut source_rounds: BTreeMap<u32, u8> = BTreeMap::new();

    let degrees: BTreeSet<u32> = graph.points().map(|point| graph.out_degree(point)).collect();
    let hub = HubDamp::build(degrees, config.hub_exponent_q12);

    let mut frontier: BTreeSet<u32> = trim(&h, &first_lit, config.frontier_cap)
        .into_iter()
        .map(|entry| entry.point)
        .collect();
    let mut history: Vec<BTreeMap<u32, i32>> = vec![h.clone()];
    // Step 3's pointer: the `a` produced in round k is the value round
    // k + MOD_LAG_ROUNDS conducts with, so every round at or before the lag
    // conducts with the value the thought began with (birth 0, or a restored a).
    let a_start = integrator.a();
    let mut schedule: BTreeMap<u16, u8> = BTreeMap::new();
    for seeded in 1..=config.params.lag_rounds.min(config.k_max) {
        schedule.insert(seeded, a_start);
    }
    let mut trajectory: Vec<(u16, u8)> = Vec::new();
    let mut rounds: Vec<RoundTrace> = Vec::new();
    let mut pending: Option<Signal> = None;
    let mut stopped: Option<(StopReason, bool)> = None;

    for round in 1..=config.k_max {
        // Step 1: the previous round's events, in the §6 class order — the
        // injected tick-start/lighting signals first, then a round-end miss.
        let mut signals: Vec<Signal> = Vec::new();
        if round == 1 {
            signals.extend_from_slice(initial_signals);
        }
        if let Some(signal) = pending.take() {
            signals.push(signal);
        }

        // Steps 2–3: integrate, then let round k conduct with the value the
        // pointer reaches this round; the value produced here applies L later.
        let a_used = schedule.remove(&round).unwrap_or(a_start);
        let lagged = drive::update(
            integrator,
            &signals,
            round,
            thought_id,
            config.params,
            &mut trajectory,
        );
        if lagged.applies_at > round {
            schedule.insert(lagged.applies_at, lagged.a);
        }
        let a_after = lagged.a;

        // Step 4: the round's modulation, read for the value it conducts with.
        let point = config.curve.at(a_used);
        let decay_q12 = mul_q12_u16(u8_to_q12(config.decay_base), point.decay_q12);
        let tau_q12 = (config.tau_settle as i64 + point.settle_shift_q12 as i64)
            .clamp(0, u16::MAX as i64) as u16;

        // Step 5: the frontier's two conduction terms, one cap.
        let mut charges: Vec<EdgeCharge> = Vec::new();
        let mut base_total: u64 = 0;
        for source in &frontier {
            let activation = h.get(source).copied().unwrap_or(0);
            if activation <= 0 {
                continue;
            }
            let damp = hub.at(graph.out_degree(*source)) as i32;
            for arc in graph.arcs(*source) {
                let plain = q12_mul(activation, arc.weight as i32);
                base_total += q12_mul(plain, damp).max(0) as u64;
                // 07 §9's two record shapes: a genome edge has no `bake` and no
                // stamp, so it conducts no `σ` — and `γ·m` is not a stand-in for
                // one, which is exactly why the clean-room law is a type fact
                // rather than a weight that happens to be 0 (G6(c), G9(d)).
                if arc.is_genome() {
                    continue;
                }
                charges.push(EdgeCharge {
                    edge_off: arc.edge_off,
                    to_point: arc.to_point,
                    bake: arc.bake,
                    a_at: arc.a_at,
                });
            }
        }
        let base_total_q12 = base_total.min(u16::MAX as u64) as u16;
        let biases = drive::conduction(a_used, &charges, medians, base_total_q12, config.params);
        let bias_total_q12 = biases
            .iter()
            .map(|bias| bias.bias_q12 as u32)
            .sum::<u32>()
            .min(u16::MAX as u32) as u16;
        let by_edge: BTreeMap<u32, EdgeBias> =
            biases.iter().map(|bias| (bias.edge_off, *bias)).collect();

        // Steps 6–7: ascending source, ascending offset, per-source reach cap
        // by weight_eff (ties by offset), contribution quantized to Q12 before
        // it is added, hub damp by the source's live out-degree.
        let reach = point.reach as usize;
        let mut spread: BTreeMap<u32, i32> = BTreeMap::new();
        let mut contributions: Vec<Contribution> = Vec::new();
        let mut round_carriers: BTreeSet<u32> = BTreeSet::new();
        let mut round_fillers: BTreeMap<u32, u16> = BTreeMap::new();
        for source in &frontier {
            let activation = h.get(source).copied().unwrap_or(0);
            if activation <= 0 {
                continue;
            }
            let arcs = graph.arcs(*source);
            if arcs.is_empty() {
                continue;
            }
            let damp = hub.at(graph.out_degree(*source)) as i32;
            let weighted: Vec<(i32, Arc)> = arcs
                .map(|arc| {
                    // A genome arc never entered `charges`, so a CSR index that
                    // numerically aliases another source's arena offset must not
                    // borrow its bias. The shape guard is what keeps the two
                    // address spaces apart (07 §9, §11).
                    let bias = if arc.is_genome() {
                        0
                    } else {
                        by_edge.get(&arc.edge_off).map(|entry| entry.bias_q12).unwrap_or(0)
                    };
                    let factor = 4096i32.saturating_add(bias as i32);
                    (q12_mul(arc.weight as i32, factor), arc)
                })
                .collect();
            let mut selected: Vec<usize> = (0..weighted.len()).collect();
            if reach < selected.len() {
                // AD.5: the hint is the first key, so a preferred filler is the
                // one that survives the truncation. An empty hint makes that key
                // a tie for every arc, which is the shipped comparator unchanged.
                let hint = config.composition_hint;
                selected.sort_by(|left, right| {
                    let preferred = |index: usize| hint.contains(&weighted[index].1.to_point);
                    preferred(*right)
                        .cmp(&preferred(*left))
                        .then(weighted[*right].0.cmp(&weighted[*left].0))
                        .then(weighted[*left].1.edge_off.cmp(&weighted[*right].1.edge_off))
                });
                selected.truncate(reach);
                selected.sort_unstable();
            }
            for index in selected {
                let (weight_eff, arc) = weighted[index];
                let contribution = q12_mul(q12_mul(activation, weight_eff), damp);
                if contribution == 0 {
                    continue;
                }
                round_carriers.insert(*source);
                round_fillers.entry(arc.to_point).or_insert(arc.kind as u16);
                let target = spread.entry(arc.to_point).or_insert(0);
                *target = target.saturating_add(contribution);
                contributions.push(Contribution {
                    source: *source,
                    to_point: arc.to_point,
                    edge_off: arc.edge_off,
                    amount: contribution,
                    genome: arc.is_genome(),
                    weight_eff_q12: weight_eff,
                });
            }
        }

        // 12 §2.6: this round's pulse, one entry per light. A source keeps the
        // role it first lit with; a point first lit later as an edge target is
        // the filler a member carried. `primed` is B15's tag and false until then.
        for point_id in &round_carriers {
            let role = match first_lit.get(point_id) {
                Some(0) | None => Role::Seed,
                Some(_) => {
                    if source_rounds.contains_key(point_id) {
                        Role::LitB
                    } else {
                        Role::LitA
                    }
                }
            };
            source_rounds.entry(*point_id).or_insert(round as u8);
            lights.push(CoLight {
                point: *point_id,
                role,
                edge_kind: 0,
                round: round as u8,
                primed: false,
            });
        }
        for (point_id, edge_kind) in &round_fillers {
            lights.push(CoLight {
                point: *point_id,
                role: Role::SharedFiller,
                edge_kind: *edge_kind,
                round: round as u8,
                primed: false,
            });
        }

        // Step 8: h_k = decay · (h_{k−1} + spread), then step 9's normalization
        // is already inside the conduction door that produced every bias above.
        let mut h_new: BTreeMap<u32, i32> = BTreeMap::new();
        let mut touched: BTreeSet<u32> = h.keys().copied().collect();
        touched.extend(spread.keys().copied());
        for point_id in touched {
            let combined = h
                .get(&point_id)
                .copied()
                .unwrap_or(0)
                .saturating_add(spread.get(&point_id).copied().unwrap_or(0));
            let value = q12_mul(combined, decay_q12 as i32);
            if value > 0 {
                h_new.insert(point_id, value);
            }
        }
        for point_id in h_new.keys() {
            first_lit.entry(*point_id).or_insert(round);
        }

        // Step 10: trim the propagation set, never the settle-set.
        let kept = trim(&h_new, &first_lit, config.frontier_cap);
        let next_frontier: BTreeSet<u32> = kept.iter().map(|entry| entry.point).collect();

        // Step 11: L1 over the union of the two lit sets, on the grid.
        let mut union: BTreeSet<u32> = h.keys().copied().collect();
        union.extend(h_new.keys().copied());
        let diff: i64 = union
            .iter()
            .map(|point_id| {
                (h_new.get(point_id).copied().unwrap_or(0) as i64
                    - h.get(point_id).copied().unwrap_or(0) as i64)
                    .abs()
            })
            .sum();
        let settled = diff < tau_q12 as i64;
        let cycle = config.oscillation == OscillationRule::PeriodCheck
            && round >= 2
            && history
                .get(round as usize - 2)
                .is_some_and(|old| *old == h_new);
        let reason = if settled {
            Some(StopReason::Settled)
        } else if cycle {
            Some(StopReason::MaxK)
        } else if next_frontier.is_empty() {
            Some(StopReason::TauMiss)
        } else if round >= config.k_max {
            Some(StopReason::MaxK)
        } else {
            None
        };

        rounds.push(RoundTrace {
            round,
            a_used,
            a_after,
            reach: point.reach,
            decay_q12,
            tau_q12,
            base_total_q12,
            bias_total_q12,
            diff_q12: diff,
            activations: vector(&h_new, &first_lit),
            frontier: kept,
            contributions,
            biases,
        });
        history.push(h_new.clone());
        h = h_new;
        frontier = next_frontier;

        if let Some(reason) = reason {
            stopped = Some((reason, cycle));
            break;
        }
        let miss = (diff - tau_q12 as i64).clamp(0, u16::MAX as i64) as u16;
        pending = Some(Signal::new(SignalKind::SettleFailure, miss));
    }

    let (stop_reason, oscillated) = stopped.unwrap_or((StopReason::MaxK, false));
    let frame_log = frame_scratch(lights, config);
    Ok(Thought {
        settle_set: vector(&h, &first_lit),
        rounds,
        a_trajectory: trajectory,
        stop_reason,
        lit_round0: lit.to_vec(),
        relit,
        oscillated,
        frame_log,
    })
}

/// docs/22 §22.2 / 12 §2.6: ghosts and role graphs are built from this frame's
/// own pulse log and die with it. Over `GHOST_MAX`/`SHAPE_MAX` the frame builds
/// none — never an eviction game (OD-99).
fn frame_scratch(lights: Vec<CoLight>, config: &WalkConfig<'_>) -> FrameLog {
    let seeds = {
        let mut points: Vec<u32> = lights
            .iter()
            .filter(|light| light.role == Role::Seed && light.round == 0)
            .map(|light| light.point)
            .collect();
        points.sort_unstable();
        points.dedup();
        points
    };
    let mut ghosts = Vec::new();
    if config.ghost_max > 0 && seeds.len() >= 2 && lights.iter().any(|light| light.round > 0) {
        ghosts.push(Ghost {
            members: seeds.clone(),
            formed_in_frame: 1,
            frame_seed: u64::from(seeds[0]),
        });
        ghosts.truncate(config.ghost_max as usize);
    }
    let mut role_graphs = Vec::new();
    if config.shape_max > 0 && !lights.is_empty() {
        let mut roles: Vec<u8> = lights.iter().map(|light| light.role.byte()).collect();
        roles.sort_unstable();
        roles.dedup();
        let mut traversed_kinds: Vec<u16> = lights
            .iter()
            // 04 §32 AD.2: the kinds *actually traversed*. A filler light always
            // carries the kind it was reached by, and kind 0 is
            // `EDGE_KIND_OWNED_ATTRIBUTE`, not "no edge" — dropping it here would
            // silently delete the half of the template that separates an
            // attribute-shaped move from a definitional one.
            .filter(|light| light.role == Role::SharedFiller)
            .map(|light| light.edge_kind)
            .collect();
        traversed_kinds.sort_unstable();
        traversed_kinds.dedup();
        let mut filler_rounds: Vec<u8> = lights
            .iter()
            .filter(|light| light.role == Role::SharedFiller)
            .map(|light| light.round)
            .collect();
        filler_rounds.sort_unstable();
        filler_rounds.dedup();
        role_graphs.push(RoleGraph {
            roles,
            traversed_kinds,
            filler_sets: filler_rounds.len().min(u16::MAX as usize) as u16,
        });
        role_graphs.truncate(config.shape_max as usize);
    }
    FrameLog { lights, ghosts, role_graphs }
}

fn vector(h: &BTreeMap<u32, i32>, first_lit: &BTreeMap<u32, u16>) -> Vec<Activation> {
    h.iter()
        .map(|(point, activation)| Activation {
            point: *point,
            activation: *activation,
            first_lit: first_lit.get(point).copied().unwrap_or(0),
        })
        .collect()
}

/// (activation desc, first-lit round asc, point id asc), at most `cap` entries.
fn trim(
    h: &BTreeMap<u32, i32>,
    first_lit: &BTreeMap<u32, u16>,
    cap: usize,
) -> Vec<Activation> {
    let mut ranked = vector(h, first_lit);
    ranked.sort_by(|left, right| {
        right
            .activation
            .cmp(&left.activation)
            .then(left.first_lit.cmp(&right.first_lit))
            .then(left.point.cmp(&right.point))
    });
    ranked.truncate(cap);
    ranked
}

/// Q12 × Q12, round-half-up, saturating at the i32 grid. No float reaches a
/// stored activation (01 §3, §16.6).
fn q12_mul(left: i32, right: i32) -> i32 {
    let product = left as i64 * right as i64;
    let rounded = if product >= 0 {
        (product + 2048) >> 12
    } else {
        -(((-product) + 2048) >> 12)
    };
    rounded.clamp(i32::MIN as i64, i32::MAX as i64) as i32
}
