//! 02 §6 / 04 §7.3 / §20 T: retrieval is the walk settling.
//!
//! The read path lights the decoded pattern, runs the one walk, and cites the
//! live rows the thought actually reached. The activation trace is the route
//! report; `route_counts` is derived from it at this call and never stored.
//!
//! A row is citable only through `si_core::record::answerable_meta`: a
//! `6 cooccur-weak` candidate or a `confidence = 0` row is never an answer's
//! source (05 §20 `check_candidate_unanswerable`, OD-58 either way).
//!
//! `anchor` is `NOW` or `t'` (04 §20 T.2): the same call at a different anchor is
//! the history walk, because the graph and the row set are filtered by
//! `active_at(anchor)` and nothing about the walk changes.

use std::collections::BTreeSet;

use anyhow::Result;

use si_affect::Integrator;
use si_core::addr::{EDGE_KIND_COOCCUR_WEAK, EDGE_KIND_OWNED_LAST, SHELF_SKILLS};
use si_core::patterns::PatternId;
use si_core::record::{answerable_meta, Edge, Record, PATTERN_NAMELESS};
use si_store::interner::Interner;
use si_store::store::Lattice;
use si_walk::{walk, Graph, Thought, WalkConfig};

use crate::light::{light, LightRequest, Resolve};

/// The decoded question's addressable slot: the frame it decoded to and the
/// term handle the answer is keyed on. The handle is supplied by the caller;
/// the point allocator is B5's (B4 report).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotKey {
    pub frame: PatternId,
    pub point: u32,
}

/// One citable row, with everything the labeler and renderer need without a
/// second store read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cited {
    pub row_id: u32,
    /// The kind of the live edge that cites the row; `6 cooccur-weak` when the
    /// row carries no live edge, so it can never be a claim.
    pub kind: u8,
    pub confidence: u16,
    pub active_at_anchor: bool,
    pub key_point: u32,
    pub val_point: u32,
    /// `resp_surface` verbatim; what the copier echoes.
    pub surface: String,
    /// 04 §21 U.1 / 01 §11: the row carries `pattern_id == NAMELESS`. Such a row
    /// is citable, enumerable by handles and reason-over-able, and **not a
    /// surface** — `si-front` refuses to print it or to lend it tokens.
    pub nameless: bool,
}

impl Cited {
    pub fn contains_all(&self, content: &[String]) -> bool {
        let surface = folded(&self.surface);
        content.iter().all(|key| surface.contains(key))
    }
}

/// The renderer's content test uses the one tokenizer; duplicated here as a
/// three-line fold call because `si-read` may not import `si-front` (00 §2).
fn folded(surface: &str) -> Vec<String> {
    surface
        .split(|c: char| !c.is_alphanumeric())
        .filter(|piece| !piece.is_empty())
        .map(si_store::interner::fold_key)
        .collect()
}

/// 04 §21 U.2 / 05 §20 `check_hop_cap_is_claim_side`: the route's claim edges,
/// counted from the trace. `hops` is the claim-side depth the labeler caps;
/// nothing in the walk reads it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RouteCounts {
    pub owned: u32,
    pub base: u32,
    pub hops: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Retrieval {
    pub thought: Thought,
    /// Citable rows reached by this thought, ranked by the value point's
    /// activation, ties by row id.
    pub cited: Vec<Cited>,
    /// Index into `cited` of the answer row, when one fired.
    pub answer: Option<usize>,
    /// The named dark slot: `Some` when decode named a slot and no answer row
    /// fired (docs/07 §7.1). `Some` ⇒ ask-back; `None` ⇒ nonce fallback.
    pub named: Option<SlotKey>,
    pub route: RouteCounts,
    pub novel: Vec<usize>,
}

pub struct ReadRequest {
    /// Folded term keys in word order.
    pub keys: Vec<String>,
    pub frame_point: Option<u32>,
    pub slot: Option<SlotKey>,
    pub anchor: i32,
    pub thought_id: u32,
}

pub fn retrieve(
    lattice: &Lattice,
    interner: &Interner,
    resolve: &dyn Resolve,
    integrator: &mut Integrator,
    config: &WalkConfig<'_>,
    request: &ReadRequest,
) -> Result<Retrieval> {
    let graph = Graph::build(lattice, request.anchor)?;
    let lighting = light(
        &LightRequest { keys: request.keys.clone(), frame_point: request.frame_point },
        resolve,
    );
    let thought = walk(
        &graph,
        &lighting.lit,
        integrator,
        request.thought_id,
        &[],
        &[],
        config,
    )?;
    let settled: BTreeSet<u32> = thought.settle_set.iter().map(|entry| entry.point).collect();
    let activation = |point: u32| {
        thought
            .settle_set
            .iter()
            .find(|entry| entry.point == point)
            .map(|entry| entry.activation)
            .unwrap_or(0)
    };

    let mut rows: Vec<(u8, Record)> = Vec::new();
    for (_, record) in lattice.rows()? {
        let active = record.envelope.active_at(request.anchor);
        if !active {
            continue;
        }
        // 12 §2.5: a skill row is a preference, never an answer. It sits on a
        // settled handle and carries one owned edge, so `answerable_meta` would
        // happily cite it — the exclusion is explicit, and it is the read-side
        // half of OD-118's "no path from a skill to a commit".
        if record.envelope.shelf_x == SHELF_SKILLS {
            continue;
        }
        let point = record.body.key_point;
        if !settled.contains(&point) {
            continue;
        }
        if let Some(slot) = &request.slot {
            if point != slot.point {
                continue;
            }
        }
        let kind = primary_kind(lattice, &record)?;
        if !answerable_meta(record.envelope.confidence, kind, active) {
            continue;
        }
        rows.push((kind, record));
    }
    rows.sort_by(|left, right| {
        activation(right.1.body.val_point)
            .cmp(&activation(left.1.body.val_point))
            .then(left.1.envelope.row_id.cmp(&right.1.envelope.row_id))
    });

    let cited: Vec<Cited> = rows
        .into_iter()
        .map(|(kind, record)| Cited {
            row_id: record.envelope.row_id,
            kind,
            confidence: record.envelope.confidence,
            active_at_anchor: true,
            key_point: record.body.key_point,
            val_point: record.body.val_point,
            surface: interner
                .get(record.body.resp_surface)
                .map(|entry| entry.verbatim.clone())
                .unwrap_or_default(),
            nameless: record.body.pattern_id == PATTERN_NAMELESS,
        })
        .collect();

    let answer = if cited.is_empty() { None } else { Some(0) };
    let named = match &request.slot {
        Some(slot) if cited.is_empty() => {
            let lit = thought.lit_round0.iter().any(|entry| entry.point == slot.point);
            if lit {
                Some(slot.clone())
            } else {
                None
            }
        }
        _ => None,
    };
    let route = route_counts(&thought, lattice)?;
    Ok(Retrieval { thought, cited, answer, named, route, novel: lighting.novel })
}

/// The kind of the live edge that would cite this row: the smallest live
/// owned kind. No live edge ⇒ `6 cooccur-weak`, which is never answerable.
pub fn primary_kind(lattice: &Lattice, record: &Record) -> Result<u8> {
    let mut kind = EDGE_KIND_COOCCUR_WEAK;
    for edge in lattice.edges_of(record)? {
        if edge.is_live() && edge.kind < kind {
            kind = edge.kind;
        }
    }
    Ok(kind)
}

pub fn route_counts(thought: &Thought, lattice: &Lattice) -> Result<RouteCounts> {
    let mut owned = BTreeSet::new();
    let mut base = BTreeSet::new();
    for round in &thought.rounds {
        for contribution in &round.contributions {
            // 07 §9's two record shapes: a genome arc is not in the edges arena,
            // so its address is never read there. It is a BASE hop by birth, and
            // G6(b.1) is the leg that needs both webs counted, not one merged.
            if contribution.genome {
                base.insert(contribution.edge_off);
                continue;
            }
            let bytes = lattice.edges_arena().read(contribution.edge_off)?;
            let kind = Edge::decode(bytes)?.kind;
            if kind == EDGE_KIND_COOCCUR_WEAK {
                continue;
            }
            if kind <= EDGE_KIND_OWNED_LAST {
                owned.insert(contribution.edge_off);
            } else {
                base.insert(contribution.edge_off);
            }
        }
    }
    let owned = owned.len() as u32;
    Ok(RouteCounts { owned, base: base.len() as u32, hops: owned })
}
