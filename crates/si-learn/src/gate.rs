//! engineering/04 §12–§17; 09 §4–§6. The write gate, ten steps in the order the
//! design fixes: privilege is an input (L-A), a teach short-circuits before any
//! pending-ask test (L-B), the pending match is owner-gated (L-C), the novelty
//! delta precedes consistency (L-D), arbitration sits between consistency and commit
//! (L-E), and `drive.salience` is called once at the commit instant and nowhere
//! else (L-F). The ledger is written last because it needs the row id.
//!
//! The gate owns no vocabulary and no shape test: it receives a `Candidate`
//! already keyed by the decoded pattern. A candidate that did not decode never
//! arrives, which is why "the row key is the decoded pattern" is structural here
//! rather than a check.

use std::collections::BTreeSet;

use anyhow::{bail, Result};

use si_affect::drive::{self, Formation};
use si_affect::params::Params;
use si_affect::signal::{Signal, SignalKind};
use si_core::addr::{
    EDGE_KIND_OWNED_ATTRIBUTE, EDGE_KIND_OWNED_DEFINITION, SHELF_FACTS, SHELF_LINKS,
    SHELF_QUESTIONS, SHELF_S_R_PAIRS,
};
use si_core::consts;
use si_core::patterns::PatternId;
use si_core::record::{Body, Envelope, Record, Stamp, AROUSAL_NONE};
use si_core::strings::Source;
use si_core::CONF_MAX;
use si_store::store::{EdgeDraft, Lattice};

// ---------------------------------------------------------------------------
// The closed provenance / ledger enum (01 §4.1, 09 §4, 09 §6).
// ---------------------------------------------------------------------------

pub const PROV_CORRECTION: u8 = 0;
pub const PROV_STATEMENT: u8 = 1;
pub const PROV_REPARSE: u8 = 2;
pub const PROV_ANSWER_TO_ASK: u8 = 3;
pub const PROV_OWNER_TEACH: u8 = 4;

pub const SPEAKER_OWNER: u8 = 0;
pub const SPEAKER_GRIMM: u8 = 1;
pub const SPEAKER_SYSTEM_INFERRED: u8 = 2;

pub const GAIN_ASSERTED: u8 = 0;
pub const GAIN_DEMONSTRATED: u8 = 1;

/// 01 §4.2 `status`: `0 open · 1 resolved · 2 expired`, changed by a new
/// version row and never in place (06 §2's closed mutable set).
pub const Q_OPEN: u8 = 0;
pub const Q_RESOLVED: u8 = 1;
pub const Q_EXPIRED: u8 = 2;

/// 01 §3's Q12 grid: `1.0 = 4096`. A fresh OWNED edge is born at weight 1.0;
/// the bake is added onto it at the commit instant (09 §4's `z[0]` row).
pub const EDGE_WEIGHT_INIT_Q12: u16 = 4096;

/// `key_field`'s high byte on a `questions` row: the gap kind that asked.
/// `GapKind` ordinals follow `si_read::gaps` (G1 = 0 … G4 = 3).
pub const ASK_ORIGIN_SENSE: u8 = 0;
pub const ASK_ORIGIN_IDLE: u8 = 1;
/// engineering/12 §2.4: the arc adds one origin value in `key_field`'s low byte
/// and two gap kinds in its high byte — no new column, no envelope byte.
pub const ASK_ORIGIN_NOMINATION: u8 = 2;
pub const GAP_CLUSTER_NOMINATION: u8 = 5;
pub const GAP_MOVE_NOMINATION: u8 = 6;

/// Event-log kinds. The three signal kinds keep their numbers because the audit
/// reads them as signals (04 §6); everything that is a gate note rather than a
/// signal uses the note kind, so a commit line can never be mistaken for a
/// novelty spike.
pub const EV_NOVELTY: u8 = 0;
pub const EV_GAIN: u8 = 1;
pub const EV_CONTRADICTION: u8 = 2;
pub const EV_NOTE: u8 = 32;

// ---------------------------------------------------------------------------
// Inputs.
// ---------------------------------------------------------------------------

/// One decoded teaching act, keyed by its pattern. The session builds it from
/// `si-front`'s decode; the gate never re-decodes and never reads the raw line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub shelf: u16,
    pub pattern_id: u16,
    /// `(pattern_id << 8) | target_slot` (01 §4.2).
    pub key_field: u16,
    pub key_point: u32,
    pub val_point: u32,
    pub stim_signature: u32,
    pub stim_surface: u32,
    pub resp_surface: u32,
    pub target_slot: u16,
    pub prov_kind: u8,
    pub prov_speaker: u8,
    pub bind_output_id: u32,
    pub question_id: u32,
    pub text_ref: u32,
    pub prov_step: i32,
    /// `P_CORRECT`'s marker shape (04 §6.4's `marker-correction` branch).
    pub marker: bool,
    /// Prior bare statements at this slot, held by the session: 0 on the first,
    /// 1 on the repeat that arbitrates as `insistence`.
    pub attempts_at_slot: u32,
    pub terms: Vec<u32>,
    pub derived_from: Vec<u32>,
    /// The row's normalized surface, for the mind log only.
    pub stimulus: String,
    pub response: String,
}

impl Candidate {
    pub fn key_field_for(pattern: PatternId, target_slot: u16) -> u16 {
        ((pattern.index() as u16) << 8) | (target_slot & 0x00ff)
    }
}

/// Everything about the instant that is not the candidate: the clock, the
/// speaker (an input, never a computation), the stage, and the `a` the salience
/// call will read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteContext {
    pub t: i32,
    pub prov_session: u32,
    pub source: Source,
    /// 04 §12 step 0b (B18): `world::class` evaluated at the line's generation and
    /// handed in as an **argument**. It is derived at the call site and stored
    /// nowhere — a `WriteContext` is RAM, and no container, row or export may
    /// carry this field (`check_speaker_class_derived`, OD-123).
    pub class: si_core::world::SpeakerClass,
    pub stage: u8,
    pub a_formation: u8,
    pub thought_id: u32,
    pub signal_ref: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GateStats {
    pub salience_calls: u32,
    pub commits: u32,
    pub confirmations: u32,
    pub asks: u32,
    pub supersessions: u32,
    pub serves: u32,
}

// ---------------------------------------------------------------------------
// Outputs.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoveltyDelta {
    pub novelty_q12: u16,
    pub coverage_q12: u16,
    pub delta_q12: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Branch {
    Commit,
    Tier3,
    MarkerCorrection,
    OwnerTeach,
    Insistence,
    OwnerWins,
    Recency,
}

impl Branch {
    pub fn as_str(self) -> &'static str {
        match self {
            Branch::Commit => "commit",
            Branch::Tier3 => "tier3",
            Branch::MarkerCorrection => "marker-correction",
            // the provenance enum literal, single-sourced from si-core (03 §3.2)
            Branch::OwnerTeach => si_core::TEACH_LITERALS[2],
            Branch::Insistence => "insistence",
            Branch::OwnerWins => "owner-wins",
            Branch::Recency => "recency",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Committed {
        row_id: u32,
        shelf: u16,
        branch: Branch,
        superseded: Vec<(u16, u32)>,
        resolved_asks: Vec<u32>,
    },
    Confirmed {
        row_id: u32,
        shelf: u16,
    },
    Asked {
        question_id: u32,
    },
    Rejected {
        reason: &'static str,
    },
    Inert {
        reason: &'static str,
    },
    /// 04 §12 step 0b / §33 AE.2 (B18): a belief-row write whose **derived** class
    /// is not `Owner`. This is a different outcome from `Inert`'s tag refusal and
    /// from `Rejected`'s tier refusal: it writes zero bytes *because the class has
    /// no honest `prov_speaker` encoding yet* (**OD-135**), and G19(c) grades the
    /// refusal itself as its pass condition.
    Refused {
        class: si_core::world::SpeakerClass,
        reason: &'static str,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateEvent {
    pub kind: u8,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteReport {
    pub outcome: Outcome,
    pub novelty: NoveltyDelta,
    /// The signals this act fires, in gather order (class 3 then 4). The session
    /// hands them to `drive.update` **after** the commit, which is why the
    /// salience call sees the pre-spike `a` (OD-56's cold-first-row litmus).
    pub signals: Vec<Signal>,
    pub events: Vec<GateEvent>,
    /// Exactly 1 for a commit and 0 for every other outcome (L-F's instrument).
    pub salience_calls: u32,
}

// ---------------------------------------------------------------------------
// Step 4 — novelty − coverage, the write-time delta (04 §13, docs/06 §6.2).
// ---------------------------------------------------------------------------

/// `matching` is *identical content* at the slot: same key, same value, same
/// normalized signature. It is the only thing that cancels novelty, which is
/// 04 §13.3's "the owner teaching the same thing in different words … commits"
/// read as a rule: a same-slot, same-value, different-surface act is a new
/// version, not a boring repeat.
pub fn novelty_delta(store: &Lattice, c: &Candidate, now: i32) -> Result<NoveltyDelta> {
    let mut identical = false;
    let mut same_value = false;
    for (_, record) in store.active_at(c.shelf, now)? {
        if record.body.key_field != c.key_field || record.body.key_point != c.key_point {
            continue;
        }
        if record.body.val_point != c.val_point {
            continue;
        }
        same_value = true;
        if record.body.stim_signature == c.stim_signature {
            identical = true;
        }
    }
    let coverage_q12 = if identical { si_core::NOVELTY_TIERS_Q12[2] } else { 0 };
    let novelty_q12 = if identical {
        si_core::NOVELTY_TIERS_Q12[0]
    } else if same_value {
        si_core::NOVELTY_TIERS_Q12[1]
    } else {
        si_core::NOVELTY_TIERS_Q12[2]
    };
    Ok(NoveltyDelta {
        novelty_q12,
        coverage_q12,
        delta_q12: novelty_q12.saturating_sub(coverage_q12),
    })
}

fn identical_row(store: &Lattice, c: &Candidate, now: i32) -> Result<Option<u32>> {
    let mut found = None;
    for (_, record) in store.active_at(c.shelf, now)? {
        if record.body.key_field == c.key_field
            && record.body.key_point == c.key_point
            && record.body.val_point == c.val_point
            && record.body.stim_signature == c.stim_signature
        {
            found = Some(match found {
                Some(previous) if previous < record.envelope.row_id => previous,
                _ => record.envelope.row_id,
            });
        }
    }
    Ok(found)
}

// ---------------------------------------------------------------------------
// Step 6 — consistency (04 §14, docs/06 §6.3): five rules, no sixth.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub shelf: u16,
    pub row_id: u32,
    pub val_point: u32,
    pub prov_kind: u8,
    pub prov_speaker: u8,
    pub confidence: u16,
    pub written_at: i32,
    pub prov_step: i32,
}

/// Rule 1 ("same slot, different active value") over the three belief shelves.
/// Rules 2–3 are instances of it (a link is a row on `links`; an inferred row is
/// a tier, not a separate flag). Rule 4 is CONFIG ENGINE's and does not exist in
/// this configuration. Rule 5 closes the set.
pub fn consistency(store: &Lattice, c: &Candidate, now: i32) -> Result<Vec<Conflict>> {
    let mut out = Vec::new();
    for shelf in [SHELF_S_R_PAIRS, SHELF_FACTS, SHELF_LINKS] {
        for (_, record) in store.active_at(shelf, now)? {
            if record.body.key_field != c.key_field || record.body.key_point != c.key_point {
                continue;
            }
            if record.body.val_point == c.val_point {
                continue;
            }
            out.push(Conflict {
                shelf,
                row_id: record.envelope.row_id,
                val_point: record.body.val_point,
                prov_kind: record.envelope.prov_kind,
                prov_speaker: record.envelope.prov_speaker,
                confidence: record.envelope.confidence,
                written_at: record.envelope.written_at_t,
                prov_step: record.envelope.prov_step,
            });
        }
    }
    out.sort_by_key(|conflict| (conflict.shelf, conflict.row_id));
    Ok(out)
}

// ---------------------------------------------------------------------------
// Step 7 — arbitration (04 §15, docs/06 §6.4).
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Commit,
    Supersede { branch: Branch, losers: Vec<Conflict> },
    Ask,
    Reject { reason: &'static str },
}

/// T3 owner `/correct` and answer-to-ask > T2 owner statement and `/teach` >
/// T1 system-inferred (including RE-PARSE). The teach sits in T2's row but
/// carries owner-tier confidence — **OD-41**'s split, and the reason a teach
/// loses to a demonstrated correction without being demoted a tier.
pub fn tier(prov_kind: u8) -> u8 {
    match prov_kind {
        PROV_CORRECTION | PROV_ANSWER_TO_ASK => 3,
        PROV_STATEMENT | PROV_OWNER_TEACH => 2,
        _ => 1,
    }
}

/// 04 §15 row 5 (B18): the class axis sits **strictly above** the tier axis and
/// orders authority, never strength. It scopes the conflict list *before* §15's
/// tier table reads it, so the table's rows 1–4 do not move. An `Owner` candidate
/// keeps §15 exactly as built.
pub fn class_scope(
    class: si_core::world::SpeakerClass,
    conflicts: &[Conflict],
) -> Vec<Conflict> {
    use si_core::world::SpeakerClass;
    match class {
        SpeakerClass::Owner => conflicts.to_vec(),
        // AE.3's sentence is "supersedes **Trusted rows** only". No row can be one:
        // a class is stored nowhere (OD-123) and the three `prov_speaker` values
        // have ruled scopes that do not include a granted teacher (**OD-135**), so
        // the scope of a trusted correction is the empty set — which is also the
        // mechanical form of "it may never age an Owner row".
        SpeakerClass::Trusted => Vec::new(),
        // A Grimm- or Stranger-class line never reaches arbitration: step 0's tag
        // guard or step 0b's class arm has already answered it with zero bytes.
        _ => Vec::new(),
    }
}

pub fn arbitrate(c: &Candidate, now: i32, conflicts: &[Conflict]) -> Verdict {    if conflicts.is_empty() {
        return Verdict::Commit;
    }
    let candidate_tier = tier(c.prov_kind);
    let highest = conflicts.iter().map(|conflict| tier(conflict.prov_kind)).max().unwrap_or(0);
    match candidate_tier {
        3 => Verdict::Supersede { branch: Branch::Tier3, losers: conflicts.to_vec() },
        2 => {
            if c.marker {
                if highest >= 3 {
                    return Verdict::Reject { reason: "marker_below_tier3" };
                }
                return Verdict::Supersede {
                    branch: Branch::MarkerCorrection,
                    losers: conflicts.to_vec(),
                };
            }
            if c.prov_kind == PROV_OWNER_TEACH {
                if highest >= 3 {
                    // 09 §5.1: the teach loses on rank and the disagreement is
                    // recorded, never silently overwritten.
                    return Verdict::Reject { reason: "teach_below_tier3" };
                }
                return Verdict::Supersede {
                    branch: Branch::OwnerTeach,
                    losers: conflicts.to_vec(),
                };
            }
            if highest <= 1 {
                return Verdict::Supersede { branch: Branch::OwnerWins, losers: conflicts.to_vec() };
            }
            if c.attempts_at_slot >= 1 {
                return Verdict::Supersede {
                    branch: Branch::Insistence,
                    losers: conflicts.to_vec(),
                };
            }
            Verdict::Ask
        }
        _ => {
            if highest >= 2 {
                return Verdict::Reject { reason: "owner_wins" };
            }
            let loses = conflicts.iter().any(|conflict| {
                conflict.written_at > now
                    || (conflict.written_at == now
                        && (conflict.confidence > conf_init(c.prov_kind)
                            || (conflict.confidence == conf_init(c.prov_kind)
                                && conflict.prov_step <= c.prov_step)))
            });
            if loses {
                Verdict::Reject { reason: "recency_tie" }
            } else {
                Verdict::Supersede { branch: Branch::Recency, losers: conflicts.to_vec() }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Step 8 — commit.
// ---------------------------------------------------------------------------

fn law(name: &str) -> u16 {
    consts::row(name)
        .and_then(|row| row.law_value)
        .unwrap_or_else(|| panic!("{name} is a LAW row with a value"))
        .clamp(0, u16::MAX as i64) as u16
}


pub fn conf_init(prov_kind: u8) -> u16 {
    match prov_kind {
        PROV_CORRECTION => law("CONF_INIT_T3_CORRECT"),
        PROV_ANSWER_TO_ASK => law("CONF_INIT_ANSWER_TO_ASK"),
        PROV_OWNER_TEACH => law("CONF_INIT_OWNER_TEACH"),
        PROV_STATEMENT => law("CONF_INIT_T2_STATEMENT"),
        _ => law("CONF_INIT_T1_INFERRED"),
    }
}

pub fn bump_exact_repeat() -> u16 {
    law("CONF_BUMP_EXACT_REPEAT")
}

pub fn bump_served() -> u16 {
    law("CONF_BUMP_SERVED")
}

pub fn gain_kind_for(prov_kind: u8) -> u8 {
    match prov_kind {
        PROV_CORRECTION | PROV_ANSWER_TO_ASK => GAIN_DEMONSTRATED,
        _ => GAIN_ASSERTED,
    }
}

fn primary_edge_kind(pattern_id: u16) -> u8 {
    if pattern_id == PatternId::PDef.index() as u16
        || pattern_id == PatternId::PQuestD.index() as u16
    {
        EDGE_KIND_OWNED_DEFINITION
    } else {
        EDGE_KIND_OWNED_ATTRIBUTE
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Committed {
    row_id: u32,
    salience_calls: u32,
    bake: u16,
    confidence: u16,
}

/// 04 §18.4 / G7.charge.1: a derived row inherits its parents' stamps as
/// history only. The list is copied and every stamp is re-tagged `src = 3`, so
/// the derivation can neither manufacture a fresh boost nor enter the median.
fn inherited_stamps(store: &Lattice, parents: &[u32]) -> Result<Vec<Stamp>> {
    let mut out = Vec::new();
    for parent in parents {
        let Some(record) = store.row_at(SHELF_S_R_PAIRS, *parent)? else {
            continue;
        };
        for edge in store.edges_of(&record)? {
            let stamps = store.stamps_of(&edge)?;
            out.extend(stamp_src_inherited(&stamps));
        }
    }
    Ok(out)
}

fn commit(
    store: &mut Lattice,
    c: &Candidate,
    ctx: &WriteContext,
    novelty: &NoveltyDelta,
    params: &Params,
    stats: &mut GateStats,
) -> Result<Committed> {
    let derived = c.prov_kind == PROV_REPARSE;
    // L-F: the one and only salience read, at this instant. A derivation is not
    // a sensed event, so it takes the charge rule instead (bake = 0).
    let (bake, stamps, salience_calls) = if derived {
        if c.derived_from.is_empty() {
            bail!("a reparse candidate without derived_from has no charge rule to obey");
        }
        (0u16, inherited_stamps(store, &c.derived_from)?, 0u32)
    } else {
        let formation = Formation {
            at_t: ctx.t,
            thought_id: ctx.thought_id,
            signal_ref: ctx.signal_ref,
            signal: SignalKind::NoveltySpike,
            magnitude: novelty.delta_q12,
        };
        let salience = drive::salience(ctx.a_formation, &formation, params);
        stats.salience_calls += 1;
        (salience.bake, vec![salience.stamp], 1u32)
    };

    let confidence = conf_init(c.prov_kind).saturating_add(bake).min(CONF_MAX);
    let mut envelope = Envelope::live(c.shelf, ctx.t, ctx.prov_session, c.prov_step);
    envelope.repeats = 1;
    envelope.confidence = confidence;
    envelope.gain_exact = 0;
    envelope.gain_kind = gain_kind_for(c.prov_kind);
    envelope.prov_kind = c.prov_kind;
    envelope.prov_speaker = c.prov_speaker;
    envelope.stage = ctx.stage;
    envelope.bind_output_id = c.bind_output_id;
    envelope.question_id = c.question_id;
    envelope.text_ref = c.text_ref;
    let body = Body {
        pattern_id: c.pattern_id,
        key_field: c.key_field,
        key_point: c.key_point,
        val_point: c.val_point,
        stim_signature: c.stim_signature,
        stim_surface: c.stim_surface,
        resp_surface: c.resp_surface,
        target_slot: c.target_slot,
        arousal_at_ask: AROUSAL_NONE,
        status: 0,
    };
    let edge = EdgeDraft {
        to_point: c.val_point,
        kind: primary_edge_kind(c.pattern_id),
        flags: 0,
        weight: EDGE_WEIGHT_INIT_Q12.saturating_add(bake),
        bake,
        stamps,
    };
    let row_id = store.append_row(envelope, body, vec![edge], &c.derived_from, &c.terms)?;
    stats.commits += 1;
    Ok(Committed { row_id, salience_calls, bake, confidence })
}

// ---------------------------------------------------------------------------
// Steps 2 / 9 — the `questions` shelf: open asks, dedupe, resolution.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestionSpec {
    pub pattern_id: u16,
    pub gap_kind: u8,
    pub origin: u8,
    pub key_point: u32,
    pub target_slot: u16,
    pub text_ref: u32,
    pub prov_step: i32,
    pub arousal_at_ask: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskRow {
    pub row_id: u32,
    pub pattern_id: u16,
    pub key_point: u32,
    pub target_slot: u16,
    pub asked_at: i32,
    pub status: u8,
    pub record: Record,
}

impl AskRow {
    /// The id a resolving row is stored under (`resolved_by` rides
    /// `bind_output_id` on the resolution's version row, 06 §2).
    pub fn resolved_by(&self) -> u32 {
        self.record.envelope.bind_output_id
    }

    /// `key_field`'s low byte — the lane, derived and never stored separately
    /// (12 §2.4: OD-128's "lane field" is this half).
    pub fn origin(&self) -> u8 {
        (self.record.body.key_field & 0x00FF) as u8
    }

    /// `key_field`'s high byte — the gap kind that asked.
    pub fn gap_kind(&self) -> u8 {
        (self.record.body.key_field >> 8) as u8
    }
}

pub fn append_question(
    store: &mut Lattice,
    spec: &QuestionSpec,
    ctx: &WriteContext,
) -> Result<u32> {
    let mut envelope = Envelope::live(SHELF_QUESTIONS, ctx.t, ctx.prov_session, spec.prov_step);
    envelope.repeats = 0;
    envelope.confidence = 0;
    envelope.gain_exact = 0;
    envelope.gain_kind = GAIN_ASSERTED;
    envelope.prov_kind = PROV_STATEMENT;
    envelope.prov_speaker = SPEAKER_OWNER;
    envelope.stage = ctx.stage;
    envelope.text_ref = spec.text_ref;
    let body = Body {
        pattern_id: spec.pattern_id,
        key_field: ((spec.gap_kind as u16) << 8) | (spec.origin as u16),
        key_point: spec.key_point,
        val_point: 0,
        stim_signature: 0,
        stim_surface: spec.text_ref,
        resp_surface: 0,
        target_slot: spec.target_slot,
        arousal_at_ask: spec.arousal_at_ask,
        status: Q_OPEN,
    };
    store.append_row(envelope, body, Vec::new(), &[], &[])
}

/// The one row per slot group with the highest `row_id` carries the status. The
/// group is `(pattern_id, key_point, target_slot)`: a resolution must find the
/// ask by the slot the answering row covers, not by the ask's gap kind.
pub fn ask_at(
    store: &Lattice,
    now: i32,
    pattern_id: u16,
    key_point: u32,
    target_slot: u16,
) -> Result<Option<AskRow>> {
    let mut latest: Option<AskRow> = None;
    let mut asked_at: Option<i32> = None;
    for (_, record) in store.active_at(SHELF_QUESTIONS, now)? {
        if record.body.pattern_id != pattern_id
            || record.body.key_point != key_point
            || record.body.target_slot != target_slot
        {
            continue;
        }
        asked_at = Some(match asked_at {
            Some(previous) => previous.min(record.envelope.written_at_t),
            None => record.envelope.written_at_t,
        });
        if latest.as_ref().is_none_or(|row| row.row_id < record.envelope.row_id) {
            latest = Some(AskRow {
                row_id: record.envelope.row_id,
                pattern_id: record.body.pattern_id,
                key_point: record.body.key_point,
                target_slot: record.body.target_slot,
                asked_at: record.envelope.written_at_t,
                status: record.body.status,
                record,
            });
        }
    }
    Ok(latest.map(|mut row| {
        row.asked_at = asked_at.unwrap_or(row.asked_at);
        row
    }))
}

pub fn open_asks(store: &Lattice, now: i32) -> Result<Vec<AskRow>> {
    let mut groups: Vec<AskRow> = Vec::new();
    for (_, record) in store.active_at(SHELF_QUESTIONS, now)? {
        let key = (
            record.body.pattern_id,
            record.body.key_point,
            record.body.target_slot,
        );
        match groups.iter_mut().find(|row| {
            (row.pattern_id, row.key_point, row.target_slot) == key
        }) {
            Some(row) => {
                if record.envelope.row_id > row.row_id {
                    let asked_at = row.asked_at.min(record.envelope.written_at_t);
                    *row = AskRow {
                        row_id: record.envelope.row_id,
                        pattern_id: record.body.pattern_id,
                        key_point: record.body.key_point,
                        target_slot: record.body.target_slot,
                        asked_at,
                        status: record.body.status,
                        record,
                    };
                } else {
                    row.asked_at = row.asked_at.min(record.envelope.written_at_t);
                }
            }
            None => groups.push(AskRow {
                row_id: record.envelope.row_id,
                pattern_id: record.body.pattern_id,
                key_point: record.body.key_point,
                target_slot: record.body.target_slot,
                asked_at: record.envelope.written_at_t,
                status: record.body.status,
                record,
            }),
        }
    }
    let mut open: Vec<AskRow> =
        groups.into_iter().filter(|row| row.status == Q_OPEN).collect();
    open.sort_by_key(|row| row.row_id);
    Ok(open)
}

pub fn latest_open_ask(store: &Lattice, now: i32) -> Result<Option<AskRow>> {
    let open = open_asks(store, now)?;
    // OD-128 / 12 §2.4: an `origin = 2` nomination ask is answered only by the
    // owner lane that mints its point (B17). Until that lane exists it must not
    // capture a bare reply that was answering an ordinary ask, or a detector's
    // proposal would silently satisfy itself.
    Ok(open
        .into_iter()
        .filter(|row| row.origin() != ASK_ORIGIN_NOMINATION)
        .max_by_key(|row| (row.asked_at, row.row_id)))
}

/// 12 §2.4 / OD-128: the owner lane's own lookup. `latest_open_ask` deliberately
/// never returns an `origin = 2` row, so the nomination lane is a separate read of
/// the same shelf — a kind-6 `move-nomination` ask is answerable here and nowhere
/// else, which is what `check_nomination_owner_lane` greps for.
pub fn latest_open_nomination(
    store: &Lattice,
    now: i32,
    gap_kind: u8,
) -> Result<Option<AskRow>> {
    Ok(open_asks(store, now)?
        .into_iter()
        .filter(|row| row.origin() == ASK_ORIGIN_NOMINATION && row.gap_kind() == gap_kind)
        .filter(|row| row.status == Q_OPEN)
        .max_by_key(|row| (row.asked_at, row.row_id)))
}

/// `status` moves by a new version row (06 §2's closed mutable set), so the old
/// row stays citable at its own interval and nothing is rewritten.
pub fn resolve_ask(
    store: &mut Lattice,
    ask: &AskRow,
    resolved_by: u32,
    ctx: &WriteContext,
    prov_step: i32,
) -> Result<u32> {
    if ask.status != Q_OPEN {
        bail!("ask {} is not open", ask.row_id);
    }
    let mut envelope =
        Envelope::live(SHELF_QUESTIONS, ctx.t, ctx.prov_session, prov_step);
    envelope.repeats = 0;
    envelope.confidence = 0;
    envelope.gain_exact = 0;
    envelope.gain_kind = GAIN_ASSERTED;
    envelope.prov_kind = ask.record.envelope.prov_kind;
    envelope.prov_speaker = ask.record.envelope.prov_speaker;
    envelope.stage = ctx.stage;
    envelope.bind_output_id = resolved_by;
    envelope.text_ref = ask.record.envelope.text_ref;
    let mut body = ask.record.body;
    body.status = Q_RESOLVED;
    store.append_row(envelope, body, Vec::new(), &[], &[])
}

// ---------------------------------------------------------------------------
// The serve path (docs/08 §8.2–§8.3): the only promotion the ledger has.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServeReport {
    pub row_id: u32,
    pub shelf: u16,
    pub promoted: bool,
    pub gain_exact: i16,
    pub repeats: u32,
    pub confidence: u16,
    pub magnitude_q12: u16,
}

pub fn serve(
    store: &mut Lattice,
    shelf: u16,
    row_id: u32,
    at_t: i32,
    stats: &mut GateStats,
) -> Result<ServeReport> {
    let record = store
        .row_at(shelf, row_id)?
        .ok_or_else(|| anyhow::anyhow!("no row {row_id} on shelf {shelf}"))?;
    if !record.envelope.is_live() {
        bail!("row {row_id} on shelf {shelf} is superseded; a serve cannot cite it");
    }
    let stamp_magnitude = primary_stamp_magnitude(store, &record)?;
    let promoted = store.promote_gain(shelf, row_id)?;
    let gain_exact = store.adjust_gain(shelf, row_id, 1)?;
    store.confirm(shelf, row_id, at_t, bump_served())?;
    let after = store
        .row_at(shelf, row_id)?
        .ok_or_else(|| anyhow::anyhow!("row {row_id} vanished under a serve"))?;
    stats.serves += 1;
    Ok(ServeReport {
        row_id,
        shelf,
        promoted,
        gain_exact,
        repeats: after.envelope.repeats,
        confidence: after.envelope.confidence,
        magnitude_q12: stamp_magnitude,
    })
}

fn primary_stamp_magnitude(store: &Lattice, record: &Record) -> Result<u16> {
    let edges = store.edges_of(record)?;
    for edge in &edges {
        if edge.is_live() {
            let stamps = store.stamps_of(edge)?;
            if let Some(stamp) = stamps.iter().find(|stamp| stamp.src == 0) {
                return Ok(stamp.magnitude);
            }
        }
    }
    Ok(0)
}

// ---------------------------------------------------------------------------
// The orchestrator.
// ---------------------------------------------------------------------------

pub fn write(
    store: &mut Lattice,
    c: &Candidate,
    ctx: &WriteContext,
    params: &Params,
    stats: &mut GateStats,
) -> Result<WriteReport> {
    // Step 0b — the class arm (04 §12, §33 AE.2; B18). A **granted** line is the
    // only class that reaches here: `grants` is empty at birth (OD-125), and a
    // `Grimm`- or `Stranger`-class line is answered by step 0's tag guard below,
    // which is B9's ruled law with G12's assertions on its reason string. The arm
    // therefore runs ahead of that guard rather than after it, because a granted
    // teacher's line is `source != owner` and step 0 would swallow the very
    // refusal AE.2 names as G19(c)'s pass condition. Tier (b) has no honest
    // `prov_speaker` value, so the build refuses and reports instead of minting a
    // fourth value (**OD-135**): one log line, zero bytes, no step 1–10.
    if ctx.class == si_core::world::SpeakerClass::Trusted && c.prov_kind != PROV_REPARSE {
        return Ok(WriteReport {
            outcome: Outcome::Refused {
                class: ctx.class,
                reason: "SpeakerNotRepresentable",
            },
            novelty: NoveltyDelta { novelty_q12: 0, coverage_q12: 0, delta_q12: 0 },
            signals: Vec::new(),
            events: vec![note(format!(
                "{{\"event\":\"refuse\",\"arm\":\"class\",\"class\":{},\"reason\":\"SpeakerNotRepresentable\",\"row\":{}}}",
                json_string(ctx.class.literal()),
                c.key_point
            ))],
            salience_calls: 0,
        });
    }

    if ctx.source != Source::Owner && c.prov_kind != PROV_REPARSE {
        // All three teaching channels are owner-privileged (09 §1, G12(d)); a
        // derivation is not a channel and carries no source at all.
        return Ok(WriteReport {
            outcome: Outcome::Inert { reason: "source_not_owner" },
            novelty: NoveltyDelta { novelty_q12: 0, coverage_q12: 0, delta_q12: 0 },
            signals: Vec::new(),
            events: vec![note(format!(
                "{{\"event\":\"refuse\",\"reason\":\"source_not_owner\",\"row\":{}}}",
                c.key_point
            ))],
            salience_calls: 0,
        });
    }

    let s = novelty_delta(store, c, ctx.t)?;
    let mut events = vec![GateEvent {
        kind: EV_NOVELTY,
        body: format!(
            "{{\"event\":\"novelty\",\"shelf\":{},\"pattern\":{},\"key_point\":{},\"target_slot\":{},\"novelty\":{},\"coverage\":{},\"delta\":{},\"stimulus\":{},\"response\":{}}}",
            c.shelf,
            c.pattern_id,
            c.key_point,
            c.target_slot,
            s.novelty_q12,
            s.coverage_q12,
            s.delta_q12,
            json_string(&c.stimulus),
            json_string(&c.response)
        ),
    }];
    let mut signals = Vec::new();

    if s.delta_q12 == 0 {
        if c.prov_kind == PROV_REPARSE {
            // The derivation adds nothing the lattice does not already say. It
            // is not a boring repeat by an owner, so it writes no bytes.
            return Ok(WriteReport {
                outcome: Outcome::Inert { reason: "derived_duplicate" },
                novelty: s,
                signals,
                events,
                salience_calls: 0,
            });
        }
        let Some(row_id) = identical_row(store, c, ctx.t)? else {
            return Ok(WriteReport {
                outcome: Outcome::Rejected { reason: "boring_without_row" },
                novelty: s,
                signals,
                events,
                salience_calls: 0,
            });
        };
        store.confirm(c.shelf, row_id, ctx.t, bump_exact_repeat())?;
        let after = store
            .row_at(c.shelf, row_id)?
            .ok_or_else(|| anyhow::anyhow!("row {row_id} vanished under a confirmation"))?;
        stats.confirmations += 1;
        events.push(note(format!(
            "{{\"event\":\"confirm\",\"row\":{row_id},\"shelf\":{},\"repeats\":{},\"confidence\":{},\"last_confirmed_at\":{}}}",
            c.shelf, after.envelope.repeats, after.envelope.confidence, after.envelope.written_at_t.max(ctx.t)
        )));
        return Ok(WriteReport {
            outcome: Outcome::Confirmed { row_id, shelf: c.shelf },
            novelty: s,
            signals,
            events,
            salience_calls: 0,
        });
    }

    if c.prov_kind != PROV_REPARSE {
        // A derivation is not a sensation: it fires no novelty spike (04 §18.4's
        // "may not manufacture a fresh boost", read at the affect door too).
        signals.push(Signal::new(SignalKind::NoveltySpike, s.delta_q12));
    }

    // §15 row 5: the class axis scopes the conflict list *before* the tier table
    // reads it. For the only class that reaches here today this is the identity.
    let conflicts = class_scope(ctx.class, &consistency(store, c, ctx.t)?);
    if !conflicts.is_empty() {
        events.push(GateEvent {
            kind: EV_CONTRADICTION,
            body: format!(
                "{{\"event\":\"conflict\",\"count\":{},\"losers\":[{}]}}",
                conflicts.len(),
                conflicts
                    .iter()
                    .map(|conflict| format!(
                        "{{\"shelf\":{},\"row\":{},\"prov_kind\":{},\"confidence\":{}}}",
                        conflict.shelf, conflict.row_id, conflict.prov_kind, conflict.confidence
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        });
        signals.push(Signal::new(
            SignalKind::Contradiction,
            (conflicts.len() as u16).saturating_mul(EDGE_WEIGHT_INIT_Q12),
        ));
    }

    let verdict = arbitrate(c, ctx.t, &conflicts);
    let (branch, losers) = match verdict {
        Verdict::Commit => (Branch::Commit, Vec::new()),
        Verdict::Ask => {
            let existing = ask_at(store, ctx.t, c.pattern_id, c.key_point, c.target_slot)?;
            let question_id = match existing {
                Some(row) if row.status == Q_OPEN => row.row_id,
                _ => append_question(
                    store,
                    &QuestionSpec {
                        pattern_id: c.pattern_id,
                        gap_kind: 3, // G4: a conflict is pending resolution
                        origin: ASK_ORIGIN_SENSE,
                        key_point: c.key_point,
                        target_slot: c.target_slot,
                        text_ref: c.text_ref,
                        prov_step: c.prov_step,
                        arousal_at_ask: ctx.a_formation,
                    },
                    ctx,
                )?,
            };
            stats.asks += 1;
            events.push(note(format!(
                "{{\"event\":\"ask\",\"branch\":\"arbitration\",\"question\":{question_id},\"row_written\":0}}"
            )));
            return Ok(WriteReport {
                outcome: Outcome::Asked { question_id },
                novelty: s,
                signals,
                events,
                salience_calls: 0,
            });
        }
        Verdict::Reject { reason } => {
            events.push(note(format!(
                "{{\"event\":\"disagreement\",\"reason\":\"{reason}\",\"branch\":\"reject\",\"row_written\":0,\"losers\":[{}]}}",
                conflicts
                    .iter()
                    .map(|conflict| conflict.row_id.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            )));
            return Ok(WriteReport {
                outcome: Outcome::Rejected { reason },
                novelty: s,
                signals,
                events,
                salience_calls: 0,
            });
        }
        Verdict::Supersede { branch, losers: list } => {
            events.push(note(format!(
                "{{\"event\":\"agreement\",\"branch\":\"{}\"}}",
                branch.as_str()
            )));
            (branch, list)
        }
    };

    let committed = commit(store, c, ctx, &s, params, stats)?;
    let mut superseded = Vec::new();
    for loser in &losers {
        store.age(loser.shelf, loser.row_id, ctx.t, committed.row_id)?;
        if gain_kind_for(loser.prov_kind) == GAIN_DEMONSTRATED
            && tier(c.prov_kind) == 3
        {
            // docs/08 §8.2: a correction that displaces a demonstrated row takes
            // one correction-unit back off it. FLUID column, in place.
            let _ = store.adjust_gain(loser.shelf, loser.row_id, -1)?;
        }
        superseded.push((loser.shelf, loser.row_id));
        stats.supersessions += 1;
        events.push(note(format!(
            "{{\"event\":\"supersede\",\"branch\":\"{}\",\"loser\":{},\"loser_shelf\":{},\"winner\":{},\"at\":{}}}",
            branch.as_str(),
            loser.row_id,
            loser.shelf,
            committed.row_id,
            ctx.t
        )));
    }

    let mut resolved_asks = Vec::new();
    if let Some(ask) =
        ask_at(store, ctx.t, c.pattern_id, c.key_point, c.target_slot)?
    {
        if ask.status == Q_OPEN {
            let version = resolve_ask(store, &ask, committed.row_id, ctx, c.prov_step)?;
            resolved_asks.push(ask.row_id);
            events.push(note(format!(
                "{{\"event\":\"resolve\",\"question\":{},\"version_row\":{},\"resolved_by\":{},\"at\":{},\"asked_at\":{}}}",
                ask.row_id, version, committed.row_id, ctx.t, ask.asked_at
            )));
            signals.push(Signal::new(SignalKind::RealizedGain, s.delta_q12));
        }
    }

    events.push(note(format!(
        "{{\"event\":\"commit\",\"row\":{},\"shelf\":{},\"prov_kind\":{},\"prov_speaker\":{},\"confidence\":{},\"bake\":{},\"gain_kind\":{},\"gain_exact\":0,\"question_id\":{},\"bind_output_id\":{},\"branch\":\"{}\",\"stage\":{}}}",
        committed.row_id,
        c.shelf,
        c.prov_kind,
        c.prov_speaker,
        committed.confidence,
        committed.bake,
        gain_kind_for(c.prov_kind),
        c.question_id,
        c.bind_output_id,
        branch.as_str(),
        ctx.stage
    )));

    Ok(WriteReport {
        outcome: Outcome::Committed {
            row_id: committed.row_id,
            shelf: c.shelf,
            branch,
            superseded,
            resolved_asks,
        },
        novelty: s,
        signals,
        events,
        salience_calls: committed.salience_calls,
    })
}

// ---------------------------------------------------------------------------
// Stage derivation (docs/11 §11.1, STAGE_THRESHOLDS).
// ---------------------------------------------------------------------------

pub fn stage_of(store: &Lattice) -> Result<u8> {
    let rows = store.rows()?;
    let mut sr_rows = 0usize;
    let mut sr_superseded = false;
    let mut fact_rows = 0usize;
    let mut entities: BTreeSet<u32> = BTreeSet::new();
    let mut links = 0usize;
    for (_, record) in &rows {
        match record.envelope.shelf_x {
            SHELF_S_R_PAIRS => {
                sr_rows += 1;
                if !record.envelope.is_live() {
                    sr_superseded = true;
                }
            }
            SHELF_FACTS => {
                if record.envelope.is_live() {
                    fact_rows += 1;
                    entities.insert(record.body.key_point);
                }
            }
            SHELF_LINKS => {
                if record.envelope.is_live() {
                    links += 1;
                }
            }
            _ => {}
        }
    }
    let s0_exit = sr_rows >= 5 && sr_superseded;
    let s1_exit = fact_rows >= 3 && entities.len() >= 2;
    Ok(if s1_exit && links >= 1 {
        2
    } else if s0_exit {
        1
    } else {
        0
    })
}

// ---------------------------------------------------------------------------
// Small helpers.
// ---------------------------------------------------------------------------

fn note(body: String) -> GateEvent {
    GateEvent { kind: EV_NOTE, body }
}

fn json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// The interned normalized key a boring test compares (01 §4.2). One function so
/// the session and the gate cannot fold differently.
pub fn intern_signature(store: &mut Lattice, stimulus: &str, response: &str) -> u32 {
    let fold = si_store::interner::fold_key;
    let signature = format!("{}=>{}", fold(stimulus), fold(response));
    store.interner_mut().intern(&signature)
}

/// The primary edge of a live row, if it has one; the serve path reads its
/// stamp as the realized-gain signal's own value.
pub fn primary_edge_bake(store: &Lattice, record: &Record) -> Result<u16> {
    for edge in store.edges_of(record)? {
        if edge.is_live() {
            return Ok(edge.bake);
        }
    }
    Ok(0)
}

pub fn stamp_src_inherited(stamps: &[Stamp]) -> Vec<Stamp> {
    stamps
        .iter()
        .map(|stamp| Stamp { src: 3, ..*stamp })
        .collect()
}
