//! 04 §5 step 6 / §5.3 / 06 §4 trap 1: the live web the walk spreads over.
//!
//! Built once per thought from the rows active at the anchor, then reused by
//! every round: iterating a source's live edges is a slice walk in ascending
//! arena-offset order, and hub damp reads a per-point live **outgoing** count.
//! A superseded edge is weight 0 in storage and never enters the index, which
//! is the NOW filter (§5.3) — no query clause, a structural absence.
//!
//! Under CONFIG ENGINE the same index carries the frozen genome's adjacency too
//! (engineering/07 §9 W.3/W.4): the `frozen` arc is a *reference*, never a copy —
//! the merged edge arena is exactly the byte-forbidden materialization (§9's load
//! budget), so a genome arc is assembled per query from the mapped CSR row.
//! Under CONFIG SI there is no genome and the graph is the live one.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use anyhow::Result;

use si_core::addr::SHELF_SKILLS;
use si_core::genome::GENOME_KIND_FIRST;
use si_core::record::{Stamp, EDGE_BYTES};
use si_store::layers_frozen::Frozen;
use si_store::store::Lattice;

/// One traversable edge, addressed by its tie-break key. For a live edge that
/// key is its arena byte offset; for a genome edge it is its index in the sealed
/// CSR run, and the two namespaces alias numerically, which is why every
/// consumer that hands an address to the store first asks [`Arc::is_genome`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arc {
    pub edge_off: u32,
    pub to_point: u32,
    /// The edge's kind byte (01 §5), for the frame log's `CoLight.edge_kind`
    /// (engineering/12 §2.6). The walk's arithmetic never reads it.
    pub kind: u8,
    pub weight: u16,
    pub bake: u16,
    /// The formation stamp's level: `σ` compares against it, never multiplies
    /// by it (04 §8, OD-33). `0` when the edge carries no stamp.
    pub a_at: u8,
}

impl Arc {
    /// 07 §6's closed kind space is also the record-shape marker: slots `0..=6`
    /// are OWNED-only, so a kind at or past slot 7 is a genome element with no
    /// `bake`, no `a_at` and no `charge_off` column at all (07 §9). Reading it
    /// is the type fact, not a kind-keyed gain: the weight arithmetic below
    /// never consults it.
    pub const fn is_genome(&self) -> bool {
        self.kind >= GENOME_KIND_FIRST
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Graph {
    out: BTreeMap<u32, Vec<Arc>>,
    degrees: BTreeMap<u32, u32>,
    points: BTreeSet<u32>,
    /// W.1's loaded layer, held by reference rather than copied — and by `Rc`,
    /// not a shared atomic handle: the shipped spread check keeps this file
    /// single-threaded, and a genome the walk can neither send to a thread nor
    /// mutate is the same law one import further along. `None` is CONFIG SI: no
    /// genome, so no merged cursor and no seeded degree.
    frozen: Option<Rc<Frozen>>,
}

/// W.3's two-cursor run over one source: the live `z` list first, in ascending
/// arena offset, then the mapped CSR row in sealed order. That order *is*
/// engineering/07 §16.2's OWNED-over-BASE weighting law, so the spread reads one
/// sequence and branches on no source class.
#[derive(Debug)]
pub struct Arcs<'g> {
    live: std::slice::Iter<'g, Arc>,
    genome: Option<GenomeCursor<'g>>,
}

#[derive(Debug)]
struct GenomeCursor<'g> {
    index: u32,
    row: &'g [si_core::genome::CsrElement],
}

impl<'g> Iterator for Arcs<'g> {
    type Item = Arc;

    fn next(&mut self) -> Option<Arc> {
        if let Some(arc) = self.live.next() {
            return Some(*arc);
        }
        let cursor = self.genome.as_mut()?;
        let element = cursor.row.first().copied()?;
        cursor.row = &cursor.row[1..];
        let edge_off = cursor.index;
        cursor.index += 1;
        Some(Arc {
            edge_off,
            to_point: element.to_point,
            kind: element.edge_kind,
            weight: element.weight_q12(),
            // 07 §9/§11: a genome edge has no `bake` column and no stamp to read
            // `a_at` from. Both fields are the absence, spelled.
            bake: 0,
            a_at: 0,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rest = self.len();
        (rest, Some(rest))
    }
}

/// W.4: the merged degree is a sum read off both runs, never a stored copy, so
/// the cursor count is exact and `arcs(point).len()` answers a gap probe.
impl ExactSizeIterator for Arcs<'_> {
    fn len(&self) -> usize {
        self.live.len() + self.genome.as_ref().map_or(0, |cursor| cursor.row.len())
    }
}

impl Arcs<'_> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Graph {
    /// The web as the lattice knows it: CONFIG SI has no attached layer, so this
    /// is the live index alone and is byte-identical to the graph every shipped
    /// gate walked before this stage.
    pub fn build(lattice: &Lattice, anchor: i32) -> Result<Self> {
        let frozen = lattice.frozen().cloned();
        Self::build_merged(lattice, anchor, frozen)
    }

    /// 07 §9's W.2–W.4 merge. The live index is built first and byte-for-byte as
    /// it always was, so a `None` layer leaves this function's output identical
    /// to [`Graph::build`]'s; the genome arcs are never copied into `out`, only
    /// pointed at.
    pub fn build_merged(
        lattice: &Lattice,
        anchor: i32,
        frozen: Option<Rc<Frozen>>,
    ) -> Result<Self> {
        let mut out: BTreeMap<u32, Vec<Arc>> = BTreeMap::new();
        let mut points = BTreeSet::new();
        for (_, record) in lattice.rows()? {
            if !record.envelope.active_at(anchor) {
                continue;
            }
            // 12 §2.5 / OD-118: a skill row is a *preference*, not a place the
            // spread can go. Its one owned edge — to the absent handle `0` —
            // never enters the index, so "no path to the sampler" is a structural
            // absence here rather than a sentence in a doc: `bake_touch_max` reads
            // this same map, so excluding shelf 9 excludes it from ignition too.
            if record.envelope.shelf_x == SHELF_SKILLS {
                continue;
            }
            let source = record.body.key_point;
            let edges = lattice.edges_of(&record)?;
            for (index, edge) in edges.iter().enumerate() {
                if !edge.is_live() {
                    continue;
                }
                let edge_off = record.envelope.edges_off + (index * EDGE_BYTES) as u32;
                // The list's first stamp is the formation stamp; later re-stamps
                // do not re-price a write-once bake (04 §16, 01 §5.3).
                let a_at = if edge.charge_off == 0 {
                    0
                } else {
                    Stamp::decode(lattice.stamps_arena().read(edge.charge_off)?)?.a_at
                };
                out.entry(source).or_default().push(Arc {
                    edge_off,
                    to_point: edge.to_point,
                    kind: edge.kind,
                    weight: edge.weight,
                    bake: edge.bake,
                    a_at,
                });
                points.insert(source);
                points.insert(edge.to_point);
            }
        }
        let mut degrees = BTreeMap::new();
        for (source, arcs) in out.iter_mut() {
            arcs.sort_by_key(|arc| arc.edge_off);
            degrees.insert(*source, arcs.len() as u32);
        }
        // W.2: the genome owns handles `0 … P_gen−1` of the same `u32` space, so
        // its points are endpoints of the web the wake walks and the hub table
        // has to cover their degrees. Under CONFIG SI the layer is absent and
        // neither loop runs.
        if let Some(layer) = &frozen {
            for point in 0..layer.genome_points() {
                points.insert(point);
            }
        }
        Ok(Self { out, degrees, points, frozen })
    }

    /// A web from arcs alone — no lattice, no store read. The fixture path a
    /// read-only leaf's own tests take: 13 §6.2's `check_tongue_is_leaf` forbids
    /// `si-tongue` from naming a lattice at all, and a walker that could only be
    /// tested through a session could not be tested under that law (11 §4's B20
    /// DoD 2). Same index shape `build_merged` produces: ascending arena offset,
    /// degrees from the grouped list, both endpoints in the point set.
    pub fn from_arcs(entries: impl IntoIterator<Item = (u32, Arc)>) -> Self {
        let mut out: BTreeMap<u32, Vec<Arc>> = BTreeMap::new();
        let mut points = BTreeSet::new();
        for (source, arc) in entries {
            points.insert(source);
            points.insert(arc.to_point);
            out.entry(source).or_default().push(arc);
        }
        let mut degrees = BTreeMap::new();
        for (source, arcs) in out.iter_mut() {
            arcs.sort_by_key(|arc| arc.edge_off);
            degrees.insert(*source, arcs.len() as u32);
        }
        Self { out, degrees, points, frozen: None }
    }

    /// W.3: the source's merged arcs — live first, ascending arena offset, then
    /// the sealed CSR row.
    pub fn arcs(&self, point: u32) -> Arcs<'_> {
        let live = self.out.get(&point).map(Vec::as_slice).unwrap_or(&[]);
        let genome = self.frozen.as_ref().and_then(|layer| {
            let row = layer.row(point);
            if row.is_empty() {
                return None;
            }
            Some(GenomeCursor { index: layer.row_start(point), row })
        });
        Arcs { live: live.iter(), genome }
    }

    /// The source's live outgoing edge count plus its genome row length: W.4's
    /// seed, read off the mapped index instead of stored twice.
    pub fn out_degree(&self, point: u32) -> u32 {
        let live = self.degrees.get(&point).copied().unwrap_or(0);
        live + self.frozen.as_ref().map_or(0, |layer| layer.degree_seed(point))
    }

    /// Every endpoint of a live edge, ascending handle.
    pub fn points(&self) -> impl Iterator<Item = u32> + '_ {
        self.points.iter().copied()
    }

    /// The largest stored `bake` over edges touching each point, in either
    /// direction. Ignition reads it; the sampler never computes a bake.
    pub fn bake_touch_max(&self) -> BTreeMap<u32, u16> {
        let mut max: BTreeMap<u32, u16> = self.points.iter().map(|point| (*point, 0)).collect();
        for (source, arcs) in &self.out {
            for arc in arcs {
                max.entry(*source).and_modify(|value| *value = (*value).max(arc.bake));
                max.entry(arc.to_point).and_modify(|value| *value = (*value).max(arc.bake));
            }
        }
        max
    }
}
