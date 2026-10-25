//! si-audit — engineering/04 §21–§22, 02 §9. The label stream over the written log.
//!
//! [`classify`] is the audit classifier of 04 §22 V: six predicates run over the
//! **persisted event log**, never live state. It is imported by nothing in the
//! control path (R-24): the label stream joins the owner-facing line in the
//! exporter/viewer, so no label can steer a later step. Deleting this crate
//! changes no state byte except the label strings in the log (G9(c)).
//!
//! Reproducibility is the acceptance condition (G9(b)): every emitted label is a
//! pure function of the [`Signal`], [`APoint`] and [`Act`] rows already logged,
//! recomputable byte-for-byte on replay. The predicates read **structural
//! presence, order and count** over those rows, never an invented magnitude: the
//! rule-set parameters are `AUDIT_THRESHOLDS` (OPEN-22 owns them, and they carry
//! no value yet), so a label fires on the shape of the log, not on a number this
//! crate is not allowed to make up. Where the design names a threshold qualitatively
//! ("above its floor", "repeated", "within a tick or two"), it is read at its
//! structural minimum: a non-zero `a`, a count of two, a window of two ticks.

use serde::Serialize;

/// The closed installed vocabulary of 02 §9 / docs/18 §18.2. A seventh word is an
/// OPEN-22 ruling, not a pull request, and this is the only crate that names them.
pub const LABELS: [&'static str; 6] = [
    "curiosity",
    "surprise",
    "satisfaction",
    "caution",
    "boredom",
    "frustration",
];

/// Signal kinds of 04 §6 / docs/18 §18.1, as they appear in the log. classify
/// reads the ones the written log actually carries: novelty, realized-gain,
/// contradiction. The rest are behavioural, read through [`Act`].
pub const KIND_NOVELTY: u8 = 0;
pub const KIND_REALIZED_GAIN: u8 = 1;
pub const KIND_CONTRADICTION: u8 = 2;

/// "repeated" is a count of two — the smallest reading that means more than once.
pub const REPEAT: usize = 2;
/// "a tick or two" is a window of two.
pub const DECAY_WINDOW_TICKS: i32 = 2;

/// One signal as the log records it: the event's own sequence, tick and thought,
/// its 04 §6 kind, and its magnitude already quantized in the row. classify never
/// invents the magnitude; it only tests presence and order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Signal {
    pub seq: u64,
    pub t: i32,
    pub thought: i64,
    pub kind: u8,
    pub magnitude: u16,
}

/// One arousal sample as the log records it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct APoint {
    pub seq: u64,
    pub t: i32,
    pub thought: i64,
    pub a: u8,
}

/// A behavioural atom: the walk/output acts a predicate reads that are not signal
/// firings — an ask, an ASK-branch arbitration, a closed question, a settle outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ActKind {
    Ask,
    ArbitrationAsk,
    ClosedAsk,
    SettleTauMiss,
    SettleMaxK,
    Settle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Act {
    pub seq: u64,
    pub t: i32,
    pub thought: i64,
    pub kind: ActKind,
}

/// One label with the logged event sequences it stands on. An empty citation list
/// is never emitted: a label that names no row is the uncitable label G9(b) refuses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Label {
    pub t: i32,
    pub name: &'static str,
    pub citations: Vec<u64>,
}

pub type LabelStream = Vec<Label>;

/// `a` above its floor is a non-zero arousal: the structural "did it move at all",
/// not a swept threshold. OPEN-22 owns any real floor value and none is installed.
fn above_floor(a: u8) -> bool {
    a > 0
}

fn seqs<I: Iterator<Item = u64>>(iter: I, limit: usize) -> Vec<u64> {
    let mut out: Vec<u64> = iter.take(limit).collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// The six predicates, in the closed vocabulary's order, over the three already-
/// logged streams. Each is a conjunction of structural facts; the citations are the
/// rows that satisfied it. A conjunct with no witness yields no label, so nothing
/// here is theater: a quiet log produces a quiet (or empty) stream, which 04 §22
/// blesses as a finding rather than a fabricated affect.
pub fn classify(signals: &[Signal], a_trajectory: &[APoint], behavior: &[Act]) -> LabelStream {
    let mut out = LabelStream::new();

    let peak_arousal = a_trajectory.iter().map(|p| p.a).max().unwrap_or(0);
    let aroused = a_trajectory.iter().any(|p| above_floor(p.a));

    let novelty: Vec<&Signal> = signals
        .iter()
        .filter(|s| s.kind == KIND_NOVELTY)
        .collect();
    let gain: Vec<&Signal> = signals
        .iter()
        .filter(|s| s.kind == KIND_REALIZED_GAIN)
        .collect();
    let contradiction: Vec<&Signal> = signals
        .iter()
        .filter(|s| s.kind == KIND_CONTRADICTION)
        .collect();

    let has = |kind: ActKind| behavior.iter().any(|a| a.kind == kind);

    // curiosity — arousal up ∧ a kind-0 spike ∧ an ask emitted (docs/18 §18.2).
    if aroused && !novelty.is_empty() && has(ActKind::Ask) {
        let cites = seqs(
            novelty
                .iter()
                .map(|s| s.seq)
                .chain(behavior.iter().filter(|a| a.kind == ActKind::Ask).map(|a| a.seq))
                .chain(a_trajectory.iter().filter(|p| above_floor(p.a)).map(|p| p.seq)),
            usize::MAX,
        );
        push_label(&mut out, a_trajectory, "curiosity", cites);
    }

    // surprise — a kind-0 spike at a write that decays within a tick or two: the
    // arousal after the spike steps back down inside the window. The spike alone is
    // not enough; without a later, lower sample the log has not shown it decay.
    for spike in &novelty {
        let rising = a_trajectory
            .iter()
            .filter(|p| p.t >= spike.t && p.t <= spike.t + DECAY_WINDOW_TICKS)
            .map(|p| p.a)
            .max()
            .unwrap_or(0);
        let decayed = a_trajectory.iter().find(|p| {
            p.seq > spike.seq && p.t <= spike.t + DECAY_WINDOW_TICKS && p.a < rising
        });
        if let (true, Some(after)) = (above_floor(rising), decayed) {
            push_label(
                &mut out,
                a_trajectory,
                "surprise",
                vec![spike.seq, after.seq],
            );
            break;
        }
    }

    // satisfaction — a kind-1 realized-gain row ∧ a pending question closed.
    if !gain.is_empty() && has(ActKind::ClosedAsk) {
        let cites = seqs(
            gain.iter()
                .map(|s| s.seq)
                .chain(behavior.iter().filter(|a| a.kind == ActKind::ClosedAsk).map(|a| a.seq)),
            usize::MAX,
        );
        push_label(&mut out, a_trajectory, "satisfaction", cites);
    }

    // caution — a kind-2 contradiction present ∧ a recent ASK-branch arbitration.
    if !contradiction.is_empty() && has(ActKind::ArbitrationAsk) {
        let cites = seqs(
            contradiction
                .iter()
                .map(|s| s.seq)
                .chain(
                    behavior
                        .iter()
                        .filter(|a| a.kind == ActKind::ArbitrationAsk)
                        .map(|a| a.seq),
                ),
            usize::MAX,
        );
        push_label(&mut out, a_trajectory, "caution", cites);
    }

    // boredom — sustained low kind-0 ∧ low spikes ∧ low open-question pressure.
    // The structural minimum is "none of these appear"; the witnesses are the flat,
    // low arousal samples the log did record, so the label still cites real rows.
    if novelty.is_empty() && !has(ActKind::Ask) {
        let cites = seqs(a_trajectory.iter().map(|p| p.seq), REPEAT);
        if !cites.is_empty() {
            push_label(&mut out, a_trajectory, "boredom", cites);
        }
    }

    // frustration — high `a` ∧ repeated settle failure or repeated contradiction.
    // "repeated" is a count of two; the two failure directions share one leg.
    if above_floor(peak_arousal) {
        let settle_failures: Vec<u64> = behavior
            .iter()
            .filter(|a| matches!(a.kind, ActKind::SettleTauMiss | ActKind::SettleMaxK))
            .map(|a| a.seq)
            .collect();
        let conflict_seqs: Vec<u64> = contradiction.iter().map(|s| s.seq).collect();
        let cites = if settle_failures.len() >= REPEAT {
            seqs(settle_failures.into_iter(), usize::MAX)
        } else if conflict_seqs.len() >= REPEAT {
            seqs(conflict_seqs.into_iter(), usize::MAX)
        } else {
            Vec::new()
        };
        if !cites.is_empty() {
            push_label(&mut out, a_trajectory, "frustration", cites);
        }
    }

    // Each predicate pushes at most once and the branches run in `LABELS` order, so
    // the stream is already in the closed vocabulary's order — a replay lands on the
    // same bytes without a re-sort.
    out
}

fn push_label(
    out: &mut LabelStream,
    a_trajectory: &[APoint],
    name: &'static str,
    citations: Vec<u64>,
) {
    if citations.is_empty() {
        return;
    }
    let t = citations
        .iter()
        .filter_map(|seq| a_trajectory.iter().find(|p| &p.seq == seq).map(|p| p.t))
        .next()
        .or_else(|| a_trajectory.first().map(|p| p.t))
        .unwrap_or(0);
    out.push(Label {
        t,
        name,
        citations,
    });
}

/// An observed co-occurrence, as data (02 §9: output-only). This never becomes a
/// wiring rule — an adjacency that steered control would fail the gate that
/// produced it (§18.3, R-24). The harness serializes it and writes
/// `coupling_observed.json`; this crate holds no file handle and names no artifact path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CouplingRow {
    pub label: &'static str,
    pub present: bool,
    pub novelty: usize,
    pub realized_gain: usize,
    pub contradiction: usize,
    pub peak_arousal: u8,
    pub citations: Vec<u64>,
}

/// The co-occurrence counts behind each label: which signal kinds and what peak
/// arousal stood beside it when it fired, and zero rows when it did not. Reported,
/// never acted on — this is the file that "reports a finding" when the emotions
/// do not appear (04 §22).
pub fn coupling(labels: &LabelStream, signals: &[Signal], a_trajectory: &[APoint]) -> Vec<CouplingRow> {
    let peak = a_trajectory.iter().map(|p| p.a).max().unwrap_or(0);
    let novelty = signals.iter().filter(|s| s.kind == KIND_NOVELTY).count();
    let gain = signals.iter().filter(|s| s.kind == KIND_REALIZED_GAIN).count();
    let contradiction = signals.iter().filter(|s| s.kind == KIND_CONTRADICTION).count();
    LABELS
        .iter()
        .map(|name| {
            let label = labels.iter().find(|label| label.name == *name);
            CouplingRow {
                label: name,
                present: label.is_some(),
                novelty: if label.is_some() { novelty } else { 0 },
                realized_gain: if label.is_some() { gain } else { 0 },
                contradiction: if label.is_some() { contradiction } else { 0 },
                peak_arousal: if label.is_some() { peak } else { 0 },
                citations: label.map(|label| label.citations.clone()).unwrap_or_default(),
            }
        })
        .collect()
}
