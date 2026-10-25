//! 04 §11 K.2 / docs/07 §7.1: the four gap predicates, first match per slot.
//!
//! Read at settle, a dark spot is a nameable slot; the same spot read live is an
//! arousal signal and is not this function's business (I7, §20.1). Detection is
//! not decision: the set is handed to the drive and nothing here emits.
//!
//! `evidence` is the predicate's own count (referencing rows, candidate edges,
//! owner pulses). Turning it into the drive's prioritization pass is the
//! drive's arithmetic and is not invented here (B4 report).

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;

use si_core::patterns::PatternId;
use si_store::store::Lattice;
use si_walk::{Graph, Thought};

use crate::retrieve::SlotKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GapKind {
    /// A term referenced by an active row has no defining row (01 §7's null).
    G1,
    /// The entity lit, its field/frame neighborhood stayed dark, and it has at
    /// least two candidate edges: an ambiguous field needing a taught link.
    G2,
    /// A term pulsed in ≥ 2 **owner** utterances and present in no row.
    G3,
    /// A conflict is pending resolution (the write path's ASK branch).
    G4,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gap {
    pub kind: GapKind,
    pub slot: Option<SlotKey>,
    pub evidence: u32,
}

pub struct GapContext<'a> {
    pub slot: Option<&'a SlotKey>,
    /// Decode named a slot and the walk died at it: the gap is addressable.
    pub named: bool,
    pub frame_point: Option<u32>,
    /// Owner utterances only; a supporter mention is never evidence (04 §21.4).
    pub owner_pulses: &'a BTreeMap<u32, u32>,
    pub pending_conflicts: &'a [u32],
}

pub fn gaps(
    lattice: &Lattice,
    anchor: i32,
    thought: &Thought,
    context: &GapContext<'_>,
) -> Result<Vec<Gap>> {
    let Some(slot) = context.slot else {
        return Ok(Vec::new());
    };
    let rows = lattice.rows()?;
    let mut referenced: BTreeMap<u32, u32> = BTreeMap::new();
    let mut defined: BTreeSet<u32> = BTreeSet::new();
    for (_, record) in &rows {
        if !record.envelope.active_at(anchor) {
            continue;
        }
        *referenced.entry(record.body.key_point).or_insert(0) += 1;
        *referenced.entry(record.body.val_point).or_insert(0) += 1;
        if record.body.pattern_id == PatternId::PDef.index() as u16 {
            defined.insert(record.body.key_point);
        }
    }

    let point = slot.point;
    let settle: BTreeSet<u32> = thought.settle_set.iter().map(|entry| entry.point).collect();
    let graph = Graph::build(lattice, anchor)?;

    // K.2's first-match order; one slot, at most one gap.
    let found = if referenced.contains_key(&point) && !defined.contains(&point) {
        Some(Gap { kind: GapKind::G1, slot: None, evidence: referenced[&point] })
    } else if settle.contains(&point)
        && context.frame_point.is_none_or(|frame| !settle.contains(&frame))
        && graph.arcs(point).len() >= 2
    {
        Some(Gap { kind: GapKind::G2, slot: None, evidence: graph.arcs(point).len() as u32 })
    } else if context.owner_pulses.get(&point).copied().unwrap_or(0) >= 2
        && !referenced.contains_key(&point)
    {
        Some(Gap {
            kind: GapKind::G3,
            slot: None,
            evidence: context.owner_pulses.get(&point).copied().unwrap_or(0),
        })
    } else if context.pending_conflicts.contains(&point) {
        Some(Gap { kind: GapKind::G4, slot: None, evidence: 1 })
    } else {
        None
    };

    Ok(found
        .map(|gap| {
            let slot = if context.named { Some(slot.clone()) } else { None };
            Gap { slot, ..gap }
        })
        .into_iter()
        .collect())
}
