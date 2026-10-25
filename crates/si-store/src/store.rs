//! 01 §4–§11 / 02 §2: the L3 lattice — three append-only record arenas plus the
//! handle arena the `terms_*`/`derived_*` lists address, the interner, and the
//! derived RAM (out-degree, composite tally) that is never serialized or hashed.

use std::collections::BTreeMap;

use anyhow::{bail, Result};

use si_core::addr::{edge_kind_is_owned, EDGE_KIND_OWNED_LAST, SHELF_CONTEXT};
use si_core::hash::HashInputs;
use si_core::record::{
    Body, Edge, Envelope, Record, Stamp, EDGE_BYTES, RECORD_BYTES, ROW_VERSION_V1, STAMP_BYTES,
    SUPERSEDED_LIVE,
};
use si_core::CONF_MAX;

use crate::arena::Arena;
use crate::interner::Interner;
use crate::layers_frozen::Frozen;
use crate::points::Points;

pub const HANDLE_BYTES: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdgeDraft {
    pub to_point: u32,
    pub kind: u8,
    pub flags: u8,
    pub weight: u16,
    pub bake: u16,
    pub stamps: Vec<Stamp>,
}

impl EdgeDraft {
    pub fn owned(to_point: u32, weight: u16, bake: u16) -> Self {
        Self {
            to_point,
            kind: si_core::addr::EDGE_KIND_OWNED_ATTRIBUTE,
            flags: 0,
            weight,
            bake,
            stamps: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Lattice {
    rows: Arena,
    edges: Arena,
    stamps: Arena,
    handles: Arena,
    interner: Interner,
    points: Points,
    degrees: BTreeMap<u32, u32>,
    composites: BTreeMap<u32, u64>,
    /// 01 §8's rehearsal pair's second half: `edge_off → last_touched_at`. The
    /// recency value is derived on read (04 §10), never stored. RAM only, like
    /// `degrees`; sleep's P1 rebuilds it from `context` rows.
    touched: BTreeMap<u32, i32>,
    /// 07 §9 W.1's loaded layer, attached at a CONFIG ENGINE wake and read by
    /// `si_walk::Graph::build` as the web's second run. RAM only, like `degrees`
    /// and `touched`: the genome is *not* stored bytes, and an `Rc` to a read-only
    /// mapping is a reference, not the copy §9's load budget forbids. `None` is
    /// CONFIG SI, where nothing is attached and the web is the live one.
    frozen: Option<std::rc::Rc<Frozen>>,
}

impl Default for Lattice {
    fn default() -> Self {
        Self::new()
    }
}

impl Lattice {
    pub fn new() -> Self {
        let mut stamps = Arena::new(STAMP_BYTES);
        // 01 §4.1/§5.1: an edge's `charge_off = 0` encodes "no stamp list", so
        // offset 0 can never address a real list. Slot 0 is reserved (one zero
        // stamp) and no edge ever points at it; every list starts at 16 or later.
        stamps.append(&[0u8; STAMP_BYTES]).expect("the reserved stamp slot is one record");
        Self {
            rows: Arena::new(RECORD_BYTES),
            edges: Arena::new(EDGE_BYTES),
            stamps,
            handles: Arena::new(HANDLE_BYTES),
            interner: Interner::new(),
            points: Points::new(),
            degrees: BTreeMap::new(),
            composites: BTreeMap::new(),
            touched: BTreeMap::new(),
            frozen: None,
        }
    }

    pub fn from_parts(
        rows: Arena,
        edges: Arena,
        stamps: Arena,
        handles: Arena,
        interner: Interner,
        points: Points,
    ) -> Result<Self> {
        if rows.record_bytes() != RECORD_BYTES
            || edges.record_bytes() != EDGE_BYTES
            || stamps.record_bytes() != STAMP_BYTES
            || handles.record_bytes() != HANDLE_BYTES
        {
            bail!("arena widths do not match 112/16/16/4");
        }
        if stamps.len() >= STAMP_BYTES && stamps.bytes()[..STAMP_BYTES].iter().any(|byte| *byte != 0) {
            bail!("stamp slot 0 is reserved for `charge_off = 0`; a loaded arena has a list there");
        }
        let mut lattice = Self {
            rows,
            edges,
            stamps,
            handles,
            interner,
            points,
            degrees: BTreeMap::new(),
            composites: BTreeMap::new(),
            touched: BTreeMap::new(),
            frozen: None,
        };
        lattice.rebuild_degrees()?;
        Ok(lattice)
    }

    pub fn rows_arena(&self) -> &Arena {
        &self.rows
    }

    pub fn edges_arena(&self) -> &Arena {
        &self.edges
    }

    pub fn stamps_arena(&self) -> &Arena {
        &self.stamps
    }

    pub fn handles_arena(&self) -> &Arena {
        &self.handles
    }

    pub fn interner(&self) -> &Interner {
        &self.interner
    }

    pub fn interner_mut(&mut self) -> &mut Interner {
        &mut self.interner
    }

    pub fn points(&self) -> &Points {
        &self.points
    }

    pub fn points_mut(&mut self) -> &mut Points {
        &mut self.points
    }

    /// 07 §9's W.2 + W.3 attach, at a CONFIG ENGINE wake and at its resume and
    /// nowhere else. It seeds the allocation base under the layer the web is
    /// merged from, so the two halves of the merge cannot be applied apart; on a
    /// resume the base is already in the decoded table and is *checked* against
    /// this genome rather than replaced. CONFIG SI never calls it: `load` has
    /// already refused.
    pub fn attach_frozen(&mut self, layer: std::rc::Rc<Frozen>) -> Result<()> {
        if self.frozen.is_some() {
            bail!("a frozen layer is already attached to this lattice");
        }
        let base = layer.genome_points();
        if self.points.is_empty() {
            self.points = Points::with_base(base);
        } else if self.points.base() != base {
            bail!(
                "the resumed point table was allocated from base {} but this genome owns {}",
                self.points.base(),
                base
            );
        }
        self.frozen = Some(layer);
        Ok(())
    }

    /// The loaded layer, for whoever builds the web. `None` is CONFIG SI.
    pub fn frozen(&self) -> Option<&std::rc::Rc<Frozen>> {
        self.frozen.as_ref()
    }

    /// The handle the next arrival takes: `0` in CONFIG SI, `P_gen` under ENGINE.
    pub fn point_base(&self) -> u32 {
        self.points.base()
    }

    pub fn last_touched(&self, edge_off: u32) -> Option<i32> {
        self.touched.get(&edge_off).copied()
    }

    /// 04 §19 S.7's may-column: the idle tick may reset an edge's
    /// rehearsal/decay curve. This is the whole of that permission — the pair's
    /// second half moves, no arena byte is written, no confidence or `weight`
    /// bump is bought (docs/08 §8.4: "his repetition is rehearsal, which touches
    /// §8.4's decay curve and buys no durable bump"), and the map is not a hash
    /// input, so the durable store is byte-identical across the reset (G10(b)).
    /// It is named for the byte it moves, not for a behavior: docs/19 §19.2
    /// demotes "rehearsal" to a readout of which weight dominated, so there is
    /// no `rehearse()` entry point in this build.
    pub fn touch_edge_curve(&mut self, edge_off: u32, at_t: i32) {
        self.touched.insert(edge_off, at_t);
    }

    /// 10 §9 (a) / 06 §3.4 / G13(b): the dimming pass's **only** write. An
    /// OWNED edge's `weight`, rewritten in place at its existing address. The
    /// two weight bytes are spliced into the arena's own current record, so the
    /// other 14 bytes cannot move by construction — the write path is the proof,
    /// not a check that runs afterwards. `Err` when the caller asks to brighten
    /// (10 §9: "no row ever gets better because of sleep"; Z.5 makes it a
    /// contract error, not a policy branch) or to zero a live edge, which is
    /// supersession's job (`age`) and never sleep's. Returns whether a byte moved.
    pub fn dim_edge_weight(&mut self, edge_off: u32, weight: u16) -> Result<bool> {
        let mut bytes = [0u8; EDGE_BYTES];
        bytes.copy_from_slice(self.edges.read(edge_off)?);
        let before = Edge::decode(&bytes)?;
        if weight > before.weight {
            bail!(
                "dimming would brighten edge {edge_off} from {} to {weight} (10 §9, 04 §28 Z.5)",
                before.weight
            );
        }
        if weight == 0 && before.weight != 0 {
            bail!(
                "dimming would zero live edge {edge_off}; only supersession sets a weight to 0 (10 §6 D.3)"
            );
        }
        if before.weight == weight {
            return Ok(false);
        }
        // `weight` is the fourth field of the 16-byte record: `to_point` (0..4),
        // `kind` (4), `flags` (5), `weight` (6..8), `bake` (8..10), pad (10..12),
        // `charge_off` (12..16). Splicing only that window is the whole mutation.
        bytes[6..8].copy_from_slice(&weight.to_le_bytes());
        let after = Edge::decode(&bytes)?;
        if after.to_point != before.to_point
            || after.kind != before.kind
            || after.flags != before.flags
            || after.bake != before.bake
            || after.charge_off != before.charge_off
        {
            bail!("the weight splice moved a byte outside 6..8 of edge {edge_off}");
        }
        self.edges.write(edge_off, &bytes)?;
        Ok(true)
    }

    /// 04 §12 step 5 / 06 §2: the boring repeat writes **no new bytes**. Exactly
    /// the FLUID columns move: `repeats +1`, `last_confirmed_at = now`, confidence
    /// `+bump` (cap `CONF_MAX`), `gain_exact` re-encoded at its current value, the
    /// primary edge's `weight +bump`, and the rehearsal pair's `last_touched_at`.
    /// Owner-gated by the caller (`source: owner`, R-35).
    pub fn confirm(&mut self, shelf: u16, row_id: u32, at_t: i32, bump: u16) -> Result<u32> {
        let (offset, mut record) = self
            .find_row(shelf, row_id)?
            .ok_or_else(|| anyhow::anyhow!("no row {row_id} on shelf {shelf}"))?;
        if !record.envelope.is_live() {
            bail!("row {row_id} on shelf {shelf} is superseded; a repeat cannot resurrect it");
        }
        record.envelope.repeats = record.envelope.repeats.saturating_add(1);
        record.envelope.last_confirmed_at = at_t;
        record.envelope.confidence =
            record.envelope.confidence.saturating_add(bump).min(CONF_MAX);
        self.rows.write(offset, &record.encode())?;

        let edges = self.edges_of(&record)?;
        if let Some((index, edge)) = edges.iter().enumerate().find(|(_, edge)| edge.is_live()) {
            let mut bumped = *edge;
            bumped.weight = bumped.weight.saturating_add(bump).min(u16::MAX);
            let edge_off = record.envelope.edges_off + (index * EDGE_BYTES) as u32;
            self.edges.write(edge_off, &bumped.encode())?;
            self.touched.insert(edge_off, at_t);
        }
        Ok(row_id)
    }

    /// 09 §6: the ledger's one promotion — the first recall that serves an
    /// asserted row and is not corrected flips `gain_kind` to demonstrated.
    /// Returns true when the promotion actually happened.
    pub fn promote_gain(&mut self, shelf: u16, row_id: u32) -> Result<bool> {
        let (offset, mut record) = self
            .find_row(shelf, row_id)?
            .ok_or_else(|| anyhow::anyhow!("no row {row_id} on shelf {shelf}"))?;
        if record.envelope.gain_kind != 0 {
            return Ok(false);
        }
        record.envelope.gain_kind = 1;
        self.rows.write(offset, &record.encode())?;
        Ok(true)
    }

    /// docs/08 §8.2: `gain_exact` is FLUID (06 §2), so a serve adds one
    /// demonstrated unit in place and a displacing correction takes one back.
    /// Returns the new value. `gain_kind` is untouched — only `promote_gain`
    /// moves it.
    pub fn adjust_gain(&mut self, shelf: u16, row_id: u32, delta: i16) -> Result<i16> {
        let (offset, mut record) = self
            .find_row(shelf, row_id)?
            .ok_or_else(|| anyhow::anyhow!("no row {row_id} on shelf {shelf}"))?;
        record.envelope.gain_exact = record.envelope.gain_exact.saturating_add(delta);
        self.rows.write(offset, &record.encode())?;
        Ok(record.envelope.gain_exact)
    }

    /// The record a row id names, without decoding the whole arena at a call
    /// site. `None` is the absent router's answer, never an error.
    pub fn row_at(&self, shelf: u16, row_id: u32) -> Result<Option<Record>> {
        Ok(self.find_row(shelf, row_id)?.map(|(_, record)| record))
    }

    /// 02 §2: the only write primitive. The store allocates `row_id`; a caller's
    /// non-zero id or a live-shelf violation is a contract error.
    pub fn append_row(
        &mut self,
        mut envelope: Envelope,
        body: Body,
        edges: Vec<EdgeDraft>,
        derived: &[u32],
        terms: &[u32],
    ) -> Result<u32> {
        if !si_core::shelves::is_l3_live(envelope.shelf_x) {
            bail!(
                "shelf {} is not an append-only L3 shelf (01 §2's registry is the only answer)",
                envelope.shelf_x
            );
        }
        if envelope.row_version != ROW_VERSION_V1 {
            bail!("row_version {:#06x} is not v1", envelope.row_version);
        }
        if envelope.row_id != 0 {
            bail!("row_id is allocated by the store, not by the caller");
        }
        if envelope.written_at_t < 0 {
            bail!("a live row's t is a committed logical step, never negative");
        }
        if envelope.superseded_at != SUPERSEDED_LIVE {
            bail!("a new row is live; supersession is this store's `age`");
        }
        if envelope.confidence > CONF_MAX {
            bail!("confidence {} is above the representable cap {CONF_MAX}", envelope.confidence);
        }
        if envelope.shelf_x == SHELF_CONTEXT && (!edges.is_empty() || envelope.charge_off != 0) {
            bail!("context carries no charge and no edges (01 §6)");
        }
        if edges.len() > u16::MAX as usize
            || derived.len() > u16::MAX as usize
            || terms.len() > u16::MAX as usize
        {
            bail!("edge/derived/term list exceeds a u16 length");
        }
        for edge in &edges {
            if !edge_kind_is_owned(edge.kind) {
                bail!("a live row's edge kind {} is not an OWNED kind (0..={EDGE_KIND_OWNED_LAST})", edge.kind);
            }
        }

        // Stamps first: each edge's `charge_off` is a byte offset in the stamp
        // arena (01 §5.3).
        let mut encoded_edges = Vec::with_capacity(edges.len());
        for draft in edges {
            let charge_off = self.append_stamps(&draft.stamps)?;
            encoded_edges.push(Edge {
                to_point: draft.to_point,
                kind: draft.kind,
                flags: draft.flags,
                weight: draft.weight,
                bake: draft.bake,
                charge_off,
            });
        }

        let edges_off = if encoded_edges.is_empty() {
            0
        } else {
            let mut first = 0;
            for (index, edge) in encoded_edges.iter().enumerate() {
                let offset = self.edges.append(&edge.encode())?;
                // 01 §8's rehearsal pair is per edge, so it exists from
                // formation: the edge has been touched exactly once, by the
                // write that made it. RAM only — never a hash input.
                self.touched.insert(offset, envelope.written_at_t);
                if index == 0 {
                    first = offset;
                }
            }
            first
        };
        let charge_off = encoded_edges.first().map(|edge| edge.charge_off).unwrap_or(0);
        let (derived_off, derived_len) = self.append_handles(derived)?;
        let (terms_off, terms_len) = self.append_handles(terms)?;

        let source = body.key_point;
        let row_id = self.next_row_id(envelope.shelf_x);
        envelope.row_id = row_id;
        envelope.edges_off = edges_off;
        envelope.edges_len = encoded_edges.len() as u16;
        envelope.charge_off = charge_off;
        envelope.derived_off = derived_off;
        envelope.derived_len = derived_len;
        envelope.terms_off = terms_off;
        envelope.terms_len = terms_len;
        let record = Record { envelope, body };
        self.rows.append(&record.encode())?;

        // 06 §4 trap 1: the counter the walk's hub damp reads is the source
        // point's **live outgoing** edge count, maintained at append/age.
        for edge in &encoded_edges {
            if edge.is_live() {
                *self.degrees.entry(source).or_insert(0) += 1;
            }
        }
        Ok(row_id)
    }

    /// 02 §2: supersession is aging, not deletion. The loser's stamps and `bake`
    /// stay; its OWNED live weights go to zero and leave `deg` (I4, 06 §4).
    pub fn age(&mut self, shelf: u16, row_id: u32, at_t: i32, by_id: u32) -> Result<()> {
        if by_id == 0 {
            bail!("age needs a successor row id; 0 is the absent sentinel");
        }
        let (offset, mut record) = self
            .find_row(shelf, row_id)?
            .ok_or_else(|| anyhow::anyhow!("no row {row_id} on shelf {shelf}"))?;
        if !record.envelope.is_live() {
            bail!("row {row_id} on shelf {shelf} is already superseded");
        }
        if at_t < record.envelope.written_at_t {
            bail!("superseding at {at_t} precedes the row's write at {}", record.envelope.written_at_t);
        }
        let edges = self.edges_of(&record)?;
        for (index, edge) in edges.iter().enumerate() {
            if edge.is_owned() && edge.weight != 0 {
                if let Some(count) = self.degrees.get_mut(&record.body.key_point) {
                    *count = count.saturating_sub(1);
                }
                let mut zeroed = *edge;
                zeroed.weight = 0;
                self.edges.write(record.envelope.edges_off + (index * EDGE_BYTES) as u32, &zeroed.encode())?;
            }
        }
        record.envelope.superseded_at = at_t;
        record.envelope.superseded_by = by_id;
        self.rows.write(offset, &record.encode())?;
        Ok(())
    }

    pub fn rows(&self) -> Result<Vec<(u32, Record)>> {
        self.rows
            .records()
            .enumerate()
            .map(|(index, bytes)| Ok(((index * RECORD_BYTES) as u32, Record::decode(bytes)?)))
            .collect()
    }

    pub fn active_at(&self, shelf: u16, anchor: i32) -> Result<Vec<(u32, Record)>> {
        let mut out = Vec::new();
        for (offset, record) in self.rows()? {
            if record.envelope.shelf_x == shelf && record.envelope.active_at(anchor) {
                out.push((offset, record));
            }
        }
        Ok(out)
    }

    pub fn now(&self) -> i32 {
        self.rows()
            .map(|rows| rows.into_iter().map(|(_, row)| row.envelope.written_at_t).max().unwrap_or(0))
            .unwrap_or(0)
    }

    pub fn edges_of(&self, record: &Record) -> Result<Vec<Edge>> {
        let len = record.envelope.edges_len as usize;
        if len == 0 {
            return Ok(Vec::new());
        }
        let bytes = self.edges.read_list(record.envelope.edges_off, len)?;
        bytes.chunks_exact(EDGE_BYTES).map(Edge::decode).collect()
    }

    pub fn stamps_of(&self, edge: &Edge) -> Result<Vec<Stamp>> {
        if edge.charge_off == 0 {
            return Ok(Vec::new());
        }
        // The edge schema carries no stamp count, so a list's end is the next
        // list's start (every start is recorded in some edge's `charge_off`),
        // or the end of the stamp arena. Lists are contiguous by allocation.
        let mut starts = Vec::new();
        for bytes in self.edges.records() {
            let candidate = Edge::decode(bytes)?;
            if candidate.charge_off != 0 {
                starts.push(candidate.charge_off);
            }
        }
        starts.sort_unstable();
        starts.dedup();
        let start = edge.charge_off;
        let end = match starts.binary_search(&start) {
            Ok(index) => starts.get(index + 1).copied().unwrap_or(self.stamps.len() as u32),
            Err(_) => bail!("stamp list at {start} is not the start of any edge's list"),
        };
        if end <= start || (end - start) % STAMP_BYTES as u32 != 0 {
            bail!("stamp list at {start}..{end} is not a whole number of stamps");
        }
        let bytes = self.stamps.read_list(start, ((end - start) / STAMP_BYTES as u32) as usize)?;
        bytes.chunks_exact(STAMP_BYTES).map(Stamp::decode).collect()
    }

    pub fn handles_at(&self, offset: u32, len: u16) -> Result<Vec<u32>> {
        if len == 0 {
            return Ok(Vec::new());
        }
        let bytes = self.handles.read_list(offset, len as usize)?;
        Ok(bytes
            .chunks_exact(HANDLE_BYTES)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("4 bytes")))
            .collect())
    }

    pub fn degrees(&self) -> &BTreeMap<u32, u32> {
        &self.degrees
    }

    pub fn out_degree(&self, point: u32) -> u32 {
        self.degrees.get(&point).copied().unwrap_or(0)
    }

    pub fn composite_count(&self, point: u32) -> u64 {
        self.composites.get(&point).copied().unwrap_or(0)
    }

    /// 04 §21 U.4: one entry per distinct composite per thought, folded in as
    /// the settle-set is logged. RAM only; never serialized, never hashed.
    pub fn observe_settle(&mut self, points: &[u32]) {
        let mut seen = std::collections::BTreeSet::new();
        for point in points {
            if seen.insert(*point) {
                *self.composites.entry(*point).or_insert(0) += 1;
            }
        }
    }

    pub fn rebuild_degrees(&mut self) -> Result<()> {
        self.degrees.clear();
        for (_, record) in self.rows()? {
            for edge in self.edges_of(&record)? {
                if edge.is_live() {
                    *self.degrees.entry(record.body.key_point).or_insert(0) += 1;
                }
            }
        }
        Ok(())
    }

    /// 01 §10 items 1–4. Items 5–9 belong to the session.
    pub fn hash_inputs(&self) -> Result<HashInputs> {
        let mut live: Vec<Record> = self
            .rows()?
            .into_iter()
            .filter(|(_, record)| record.envelope.is_live())
            .map(|(_, record)| record)
            .collect();
        live.sort_by_key(|record| (record.envelope.shelf_x, record.envelope.row_id));
        let mut live_rows = Vec::with_capacity(live.len() * RECORD_BYTES);
        for record in live {
            live_rows.extend_from_slice(&record.encode());
        }

        let mut edge_records: Vec<&[u8]> = self.edges.records().collect();
        edge_records.sort_unstable();
        let mut edges = Vec::with_capacity(edge_records.len() * EDGE_BYTES);
        for record in edge_records {
            edges.extend_from_slice(record);
        }

        // Slot 0 is the reserved null list, not a charge stamp (01 §10 item 3
        // hashes stamps, and no stamp lives there); skipping it keeps the birth
        // digest equal to the pinned empty-state value.
        let mut stamp_records: Vec<&[u8]> = self.stamps.records().skip(1).collect();
        stamp_records.sort_unstable();
        let mut stamps = Vec::with_capacity(stamp_records.len() * STAMP_BYTES);
        for record in stamp_records {
            stamps.extend_from_slice(record);
        }

        Ok(HashInputs {
            live_rows,
            edges,
            stamps,
            interner: self.interner.canonical_bytes(),
            ..HashInputs::default()
        })
    }

    fn find_row(&self, shelf: u16, row_id: u32) -> Result<Option<(u32, Record)>> {
        for (offset, record) in self.rows()? {
            if record.envelope.shelf_x == shelf && record.envelope.row_id == row_id {
                return Ok(Some((offset, record)));
            }
        }
        Ok(None)
    }

    fn next_row_id(&self, shelf: u16) -> u32 {
        self.rows()
            .map(|rows| {
                rows.into_iter()
                    .filter(|(_, record)| record.envelope.shelf_x == shelf)
                    .map(|(_, record)| record.envelope.row_id)
                    .max()
                    .unwrap_or(0)
                    + 1
            })
            .unwrap_or(1)
    }

    fn append_stamps(&mut self, stamps: &[Stamp]) -> Result<u32> {
        if stamps.is_empty() {
            return Ok(0);
        }
        let mut first = 0;
        for (index, stamp) in stamps.iter().enumerate() {
            let offset = self.stamps.append(&stamp.encode())?;
            if index == 0 {
                first = offset;
            }
        }
        Ok(first)
    }

    fn append_handles(&mut self, handles: &[u32]) -> Result<(u32, u16)> {
        if handles.is_empty() {
            return Ok((0, 0));
        }
        let mut first = 0;
        for (index, handle) in handles.iter().enumerate() {
            let offset = self.handles.append(&handle.to_le_bytes())?;
            if index == 0 {
                first = offset;
            }
        }
        Ok((first, handles.len() as u16))
    }
}
