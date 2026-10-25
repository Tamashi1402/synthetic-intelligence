//! engineering/04 §18; 05 §12 G7. RE-PARSE: the decoder maturing retroactively.
//!
//! Eligibility is the absence of a child, never a cursor, so a second pass adds
//! zero rows by construction (G7.b.1). The pass walks S-R rows in
//! `(prov_step, row_id)` order, so the derivation sequence is the teaching
//! sequence and not the arena's. A derivation keeps the parent's slot identity
//! and goes back through the ordinary gate at step 4, where it can conflict and
//! arbitrate like any candidate — but it takes the charge rule instead of a
//! salience read: `bake = 0`, inherited stamps `src = 3` (G7.charge.1–2).
//!
//! The crate owns no vocabulary, so the parse test is injected: `Parser` is the
//! seam the session fills with `si-front`'s reflex set. This module folds keys
//! with the same `fold_key` the point table uses and nothing else.

use anyhow::Result;

use si_affect::params::Params;
use si_core::addr::{SHELF_FACTS, SHELF_S_R_PAIRS};
use si_store::interner::fold_key;
use si_store::store::Lattice;

use crate::gate::{self, Candidate, GateStats, Outcome, WriteContext, WriteReport};

/// What a matured parse says a taught surface means. Words, not handles: the
/// derivation must land on points the teaching already allocated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DerivedSlot {
    pub pattern_id: u16,
    /// `(pattern_id << 8) | target_slot` of the decoded frame.
    pub key_field: u16,
    pub key: String,
    pub value: String,
    pub target_slot: u16,
}

/// The reflex set, as seen by one re-parse pass. The session implements this
/// over `si-front`; `None` is "still not decodable", which is not an error.
pub trait Parser {
    fn slot_of(&self, stimulus: &str, response: &str) -> Option<DerivedSlot>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PassOutcome {
    Derived { sr_row: u32, row_id: u32 },
    Duplicate { sr_row: u32 },
    Rejected { sr_row: u32, reason: &'static str },
    Undecodable { sr_row: u32 },
}

/// 04 §18.1: the parent carries no child pointer in the schema, so "no derived
/// child" is read through the children's `derived_from` lists. Superseded
/// children count: a row that was parsed once is not re-derived after its
/// derived view is corrected (that is what makes the pass idempotent).
pub fn has_derived_child(store: &Lattice, sr_id: u32) -> Result<bool> {
    for (_, record) in store.rows()? {
        if record.envelope.shelf_x != SHELF_FACTS {
            continue;
        }
        let parents =
            store.handles_at(record.envelope.derived_off, record.envelope.derived_len)?;
        if parents.contains(&sr_id) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// One eligible derivation, holding the parent's ids so the candidate can reuse
/// them verbatim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Derivation {
    pub sr_row: u32,
    pub sr_point: u32,
    pub seq: (i32, u32),
    pub slot: DerivedSlot,
    pub key_point: u32,
    pub val_point: u32,
    pub stim_signature: u32,
    pub stim_surface: u32,
    pub resp_surface: u32,
    pub text_ref: u32,
    pub terms: Vec<u32>,
}

/// The eligibility test, step-ordered. `None` inside the loop is an ineligible
/// row (already derived, undecodable, or a parse whose key is not the parent's).
pub fn eligible(store: &Lattice, parser: &dyn Parser) -> Result<Vec<Derivation>> {
    let mut rows = Vec::new();
    for (_, record) in store.rows()? {
        if record.envelope.shelf_x == SHELF_S_R_PAIRS && record.envelope.is_live() {
            rows.push(record);
        }
    }
    rows.sort_by_key(|record| (record.envelope.prov_step, record.envelope.row_id));

    let mut out = Vec::new();
    for record in rows {
        if has_derived_child(store, record.envelope.row_id)? {
            continue;
        }
        let Some(stimulus) = store.interner().get(record.body.stim_surface) else {
            continue;
        };
        let Some(response) = store.interner().get(record.body.resp_surface) else {
            continue;
        };
        let Some(slot) = parser.slot_of(&stimulus.verbatim, &response.verbatim) else {
            continue;
        };
        let key = fold_key(&slot.key);
        let value = fold_key(&slot.value);
        let Some(key_point) = store.points().by_key().get(&key).and_then(|set| set.first()) else {
            continue;
        };
        // The typed view is a view *of this row*: a parse that names a different
        // entity is a different fact, not a derivation of this one.
        if *key_point != record.body.key_point || slot.key_field != record.body.key_field {
            continue;
        }
        let Some(val_point) = store.points().by_key().get(&value).and_then(|set| set.first())
        else {
            continue;
        };
        let terms = store.handles_at(record.envelope.terms_off, record.envelope.terms_len)?;
        out.push(Derivation {
            sr_row: record.envelope.row_id,
            sr_point: record.body.key_point,
            seq: (record.envelope.prov_step, record.envelope.row_id),
            slot,
            key_point: *key_point,
            val_point: *val_point,
            stim_signature: record.body.stim_signature,
            stim_surface: record.body.stim_surface,
            resp_surface: record.body.resp_surface,
            text_ref: record.envelope.text_ref,
            terms,
        });
    }
    Ok(out)
}

/// 04 §18 steps 2–6. Runs only from the idle tick (the session owns that call,
/// so `si-walk ↛ si-learn` survives RE-PARSE), and only at stage ≥ S1.
pub fn pass(
    store: &mut Lattice,
    parser: &dyn Parser,
    ctx: &WriteContext,
    params: &Params,
    stats: &mut GateStats,
) -> Result<Vec<PassOutcome>> {
    let mut outcomes = Vec::new();
    for derivation in eligible(store, parser)? {
        let candidate = Candidate {
            shelf: SHELF_FACTS,
            pattern_id: derivation.slot.pattern_id,
            key_field: derivation.slot.key_field,
            key_point: derivation.key_point,
            val_point: derivation.val_point,
            stim_signature: derivation.stim_signature,
            stim_surface: derivation.stim_surface,
            resp_surface: derivation.resp_surface,
            target_slot: derivation.slot.target_slot,
            prov_kind: gate::PROV_REPARSE,
            prov_speaker: gate::SPEAKER_SYSTEM_INFERRED,
            bind_output_id: 0,
            question_id: 0,
            text_ref: derivation.text_ref,
            prov_step: derivation.seq.0,
            marker: false,
            attempts_at_slot: 0,
            terms: derivation.terms.clone(),
            derived_from: vec![derivation.sr_row],
            stimulus: derivation.slot.key.clone(),
            response: derivation.slot.value.clone(),
        };
        let scoped = *ctx;
        let report: WriteReport = gate::write(store, &candidate, &scoped, params, stats)?;
        outcomes.push(match report.outcome {
            Outcome::Committed { row_id, .. } => {
                PassOutcome::Derived { sr_row: derivation.sr_row, row_id }
            }
            Outcome::Inert { reason } => {
                PassOutcome::Rejected { sr_row: derivation.sr_row, reason }
            }
            Outcome::Rejected { reason } => {
                PassOutcome::Rejected { sr_row: derivation.sr_row, reason }
            }
            Outcome::Asked { .. } | Outcome::Confirmed { .. } => {
                PassOutcome::Duplicate { sr_row: derivation.sr_row }
            }
            // A derivation is not a teaching act and carries no speaker, so the
            // class arm skips it (`prov_kind = REPARSE`); this arm exists because
            // the gate's outcome set is now five-wide (04 §12 step 0b).
            Outcome::Refused { .. } => PassOutcome::Rejected {
                sr_row: derivation.sr_row,
                reason: "derived_refused",
            },
        });
    }
    Ok(outcomes)
}
