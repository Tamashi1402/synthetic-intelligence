//! 07 §4 / §11: the sealed canonical file — its sections, their fixed order and
//! the byte codec both the converter and the loader read. This file performs no
//! I/O: it turns bytes into records and records into bytes, which is what makes
//! 00 §6's two guards testable without a disk.
//!
//! The canonical form is one byte string, so a section list is the whole format:
//! a `u32` count, then per section a `u32` byte length and that many bytes. The
//! order below is part of the seal — reordering two sections is a re-version.

use anyhow::{bail, Result};
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

use si_core::container::{Container, Format};
use si_core::genome::{
    CsrElement, DictRow, GenomeHeader, PointRow, RelRow, RuleRow, StringTable, SurfaceEntry,
    CSR_ELEMENT_BYTES, DICT_BYTES, POINT_BYTES, REL_BYTES, RULE_BYTES, SURFACE_ENTRY_BYTES,
};

pub const SECTION_COUNT: usize = 16;
/// The sealed order. Section 0–2 are the three shelf runs 07 §M.9 hashes; the
/// rest are the aux tables of §11, including the interner — which is why a
/// relation *name* changing moves the version tag.
pub const SECTIONS: [&str; SECTION_COUNT] = [
    "dict_run",
    "rel_run",
    "rule_run",
    "point_table",
    "csr_starts",
    "csr_elements",
    "surface_entries",
    "surface_starts",
    "surface_aliases",
    "surface_keys",
    "forms_blob",
    "rule_tokens",
    "kind_table",
    "tier_buckets",
    "tier_tops",
    "interner",
];

/// One bucket row of the closed step table (07 §8): authored in the manifest and
/// sealed here, so editing a boundary is a re-version rather than a rebuild.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BucketRow {
    pub source: u32,
    pub min: u32,
    pub max: u32,
    pub tier: u8,
}

/// A source's top tier: the weight its unweighted rows take (§8's no-drop rule).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TopRow {
    pub source: u32,
    pub tier: u8,
}

impl BucketRow {
    pub fn encode(&self) -> [u8; 16] {
        let mut out = [0u8; 16];
        out[0..4].copy_from_slice(&self.source.to_le_bytes());
        out[4..8].copy_from_slice(&self.min.to_le_bytes());
        out[8..12].copy_from_slice(&self.max.to_le_bytes());
        out[12] = self.tier;
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 16 {
            bail!("tier bucket row is {} bytes, want 16", bytes.len());
        }
        if bytes[13..16] != [0, 0, 0] {
            bail!("tier bucket row carries a non-zero reserved tail (A6)");
        }
        Ok(Self {
            source: u32::from_le_bytes(bytes[0..4].try_into().expect("4 bytes")),
            min: u32::from_le_bytes(bytes[4..8].try_into().expect("4 bytes")),
            max: u32::from_le_bytes(bytes[8..12].try_into().expect("4 bytes")),
            tier: bytes[12],
        })
    }
}

impl TopRow {
    pub fn encode(&self) -> [u8; 8] {
        let mut out = [0u8; 8];
        out[0..4].copy_from_slice(&self.source.to_le_bytes());
        out[4] = self.tier;
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 8 {
            bail!("top tier row is {} bytes, want 8", bytes.len());
        }
        if bytes[5..8] != [0, 0, 0] {
            bail!("top tier row carries a non-zero reserved tail (A6)");
        }
        Ok(Self {
            source: u32::from_le_bytes(bytes[0..4].try_into().expect("4 bytes")),
            tier: bytes[4],
        })
    }
}

/// One sealed provenance batch (07 §10): the license is stamped here, never per
/// row, which is what keeps a `REL` record at 16 bytes.
///
/// It rides in the container header rather than in the hashed payload, and that
/// split is deliberate: §4 lists "the manifest's URLs" among what is *not* in the
/// canonical bytes, while §10 requires the batch table *inside* the artifact. A
/// sealed file that hashed its own URLs could not satisfy both.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub batch_id: u8,
    pub source: String,
    pub version: String,
    pub license: String,
    pub url: String,
    pub sha256: String,
    pub dict_rows: u32,
    pub rel_rows: u32,
    pub rule_rows: u32,
}

/// The container header, read as the two things it holds.
pub fn split_header(value: &serde_json::Value) -> Result<(GenomeHeader, Vec<Provenance>)> {
    let mut value = value.clone();
    let object = value
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("a container header must be an object"))?;
    let provenance = object.remove("provenance").ok_or_else(|| {
        anyhow::anyhow!("the artifact carries no batch table, so no row is attributable (A5)")
    })?;
    let header = GenomeHeader::from_json(&value)?;
    let batches: Vec<Provenance> = serde_json::from_value(provenance)?;
    if batches.len() > si_core::genome::GENOME_BATCH_SLOTS {
        bail!(
            "{} batches are past the sealed {}-slot space",
            batches.len(),
            si_core::genome::GENOME_BATCH_SLOTS
        );
    }
    for (index, batch) in batches.iter().enumerate() {
        if batch.batch_id as usize != index {
            bail!("batch {index} is sealed as id {}", batch.batch_id);
        }
    }
    Ok((header, batches))
}

/// The three shelf runs of an already-encoded canonical file: §M.9's narrower
/// "sorted row file", kept checkable beside §4's wider sealed-file reading.
pub fn rows_bytes(canonical: &[u8]) -> Result<Vec<u8>> {
    let slices = section_slices(canonical)?;
    let mut out = Vec::new();
    for slice in &slices[..3] {
        out.extend_from_slice(slice);
    }
    Ok(out)
}

/// The payload as its sections, in sealed order.
pub fn section_slices(bytes: &[u8]) -> Result<Vec<&[u8]>> {
    if bytes.len() < 4 {
        bail!("the canonical file is shorter than its section count");
    }
    let count = u32::from_le_bytes(bytes[0..4].try_into().expect("4 bytes")) as usize;
    if count != SECTION_COUNT {
        bail!("the canonical file declares {count} sections, want {SECTION_COUNT}");
    }
    let mut slices = Vec::with_capacity(SECTION_COUNT);
    let mut off = 4usize;
    for index in 0..SECTION_COUNT {
        if off + 4 > bytes.len() {
            bail!("section {} is truncated at its length", SECTIONS[index]);
        }
        let len = u32::from_le_bytes(bytes[off..off + 4].try_into().expect("4 bytes")) as usize;
        off += 4;
        let end =
            off.checked_add(len).ok_or_else(|| anyhow::anyhow!("section length overflows"))?;
        if end > bytes.len() {
            bail!("section {} runs past the end of the file", SECTIONS[index]);
        }
        slices.push(&bytes[off..end]);
        off = end;
    }
    if off != bytes.len() {
        bail!("the canonical file has {} bytes after its last section", bytes.len() - off);
    }
    Ok(slices)
}

/// The decoded canonical file. Every vector is in sealed order, so an index here
/// is the `(0, x, y)` address a citation prints and a CSR `rel_row` names.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GenomeFile {
    pub dict: Vec<DictRow>,
    pub rel: Vec<RelRow>,
    pub rule: Vec<RuleRow>,
    pub points: Vec<PointRow>,
    pub csr_starts: Vec<u32>,
    pub csr: Vec<CsrElement>,
    pub surface_entries: Vec<SurfaceEntry>,
    pub surface_starts: Vec<u32>,
    pub surface_aliases: Vec<u32>,
    pub surface_keys: Vec<Vec<u8>>,
    pub forms: Vec<u8>,
    pub rule_tokens: Vec<u32>,
    pub kind_ids: Vec<u32>,
    pub buckets: Vec<BucketRow>,
    pub tops: Vec<TopRow>,
    pub strings: StringTable,
}

impl GenomeFile {
    /// The canonical byte string the version tag is taken over.
    pub fn encode(&self) -> Vec<u8> {
        let parts = self.sections();
        let mut out = Vec::new();
        out.extend_from_slice(&(SECTION_COUNT as u32).to_le_bytes());
        for bytes in parts {
            out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            out.extend_from_slice(&bytes);
        }
        out
    }

    /// The payload's sections, in sealed order.
    pub fn sections(&self) -> [Vec<u8>; SECTION_COUNT] {
        let mut dict = Vec::with_capacity(self.dict.len() * DICT_BYTES);
        for row in &self.dict {
            dict.extend_from_slice(&row.encode());
        }
        let mut rel = Vec::with_capacity(self.rel.len() * REL_BYTES);
        for row in &self.rel {
            rel.extend_from_slice(&row.encode());
        }
        let mut rule = Vec::with_capacity(self.rule.len() * RULE_BYTES);
        for row in &self.rule {
            rule.extend_from_slice(&row.encode());
        }
        let mut points = Vec::with_capacity(self.points.len() * POINT_BYTES);
        for row in &self.points {
            points.extend_from_slice(&row.encode());
        }
        let u32s = |values: &[u32]| {
            let mut bytes = Vec::with_capacity(values.len() * 4);
            for value in values {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            bytes
        };
        let mut csr = Vec::with_capacity(self.csr.len() * CSR_ELEMENT_BYTES);
        for element in &self.csr {
            csr.extend_from_slice(&element.encode());
        }
        let mut entries = Vec::with_capacity(self.surface_entries.len() * SURFACE_ENTRY_BYTES);
        for entry in &self.surface_entries {
            entries.extend_from_slice(&entry.encode());
        }
        let mut keys = Vec::new();
        for key in &self.surface_keys {
            keys.extend_from_slice(&(key.len() as u32).to_le_bytes());
            keys.extend_from_slice(key);
        }
        let mut tokens = Vec::with_capacity(self.rule_tokens.len() * 4);
        for token in &self.rule_tokens {
            tokens.extend_from_slice(&token.to_le_bytes());
        }
        let mut buckets = Vec::with_capacity(self.buckets.len() * 16);
        for row in &self.buckets {
            buckets.extend_from_slice(&row.encode());
        }
        let mut tops = Vec::with_capacity(self.tops.len() * 8);
        for row in &self.tops {
            tops.extend_from_slice(&row.encode());
        }
        [
            dict,
            rel,
            rule,
            points,
            u32s(&self.csr_starts),
            csr,
            entries,
            u32s(&self.surface_starts),
            u32s(&self.surface_aliases),
            keys,
            self.forms.clone(),
            tokens,
            u32s(&self.kind_ids),
            buckets,
            tops,
            self.strings.canonical_bytes(),
        ]
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let slices = section_slices(bytes)?;
        let dict = fixed("the DICT run", slices[0], DICT_BYTES)?
            .iter()
            .map(|row| DictRow::decode(row))
            .collect::<Result<Vec<_>>>()?;
        let rel = fixed("the REL run", slices[1], REL_BYTES)?
            .iter()
            .map(|row| RelRow::decode(row))
            .collect::<Result<Vec<_>>>()?;
        let rule = fixed("the RULE run", slices[2], RULE_BYTES)?
            .iter()
            .map(|row| RuleRow::decode(row))
            .collect::<Result<Vec<_>>>()?;
        let points = fixed("the point table", slices[3], POINT_BYTES)?
            .iter()
            .map(|row| PointRow::decode(row))
            .collect::<Result<Vec<_>>>()?;
        let csr_starts = read_u32s(slices[4])?;
        let csr = fixed("the CSR elements", slices[5], CSR_ELEMENT_BYTES)?
            .iter()
            .map(|row| CsrElement::decode(row))
            .collect::<Result<Vec<_>>>()?;
        let surface_entries = fixed("the surface index", slices[6], SURFACE_ENTRY_BYTES)?
            .iter()
            .map(|row| SurfaceEntry::decode(row))
            .collect::<Result<Vec<_>>>()?;
        let surface_starts = read_u32s(slices[7])?;
        let surface_aliases = read_u32s(slices[8])?;
        let mut surface_keys = Vec::new();
        let mut cursor = 0usize;
        while cursor < slices[9].len() {
            if cursor + 4 > slices[9].len() {
                bail!("a surface key is truncated at its length");
            }
            let len =
                u32::from_le_bytes(slices[9][cursor..cursor + 4].try_into().expect("4 bytes"))
                    as usize;
            cursor += 4;
            let end = cursor + len;
            if end > slices[9].len() {
                bail!("a surface key runs past its section");
            }
            surface_keys.push(slices[9][cursor..end].to_vec());
            cursor = end;
        }
        let rule_tokens = read_u32s(slices[11])?;
        let kind_ids = read_u32s(slices[12])?;
        let buckets = fixed("the tier buckets", slices[13], 16)?
            .iter()
            .map(|row| BucketRow::decode(row))
            .collect::<Result<Vec<_>>>()?;
        let tops = fixed("the top tiers", slices[14], 8)?
            .iter()
            .map(|row| TopRow::decode(row))
            .collect::<Result<Vec<_>>>()?;
        let strings = StringTable::decode(slices[15])?;
        let out = Self {
            dict,
            rel,
            rule,
            points,
            csr_starts,
            csr,
            surface_entries,
            surface_starts,
            surface_aliases,
            surface_keys,
            forms: slices[10].to_vec(),
            rule_tokens,
            kind_ids,
            buckets,
            tops,
            strings,
        };
        out.check_shape()?;
        Ok(out)
    }

    /// The shape invariants that are not per-record: the CSR array bounds and the
    /// one-edge-per-`REL`-row identity of §11's index space.
    pub fn check_shape(&self) -> Result<()> {
        if self.csr_starts.len() != self.points.len() + 1 {
            bail!(
                "the CSR start array has {} entries for {} points, want {} (07 §11)",
                self.csr_starts.len(),
                self.points.len(),
                self.points.len() + 1
            );
        }
        if *self.csr_starts.last().unwrap_or(&0) as usize != self.csr.len() {
            bail!("the CSR start array's sentinel is not the element count");
        }
        if self.csr.len() != self.rel.len() {
            bail!(
                "the index covers {} edges but shelf 2 holds {} rows: a genome edge is \
                 counted once, in its `a_point` row (07 §11)",
                self.csr.len(),
                self.rel.len()
            );
        }
        if self.points.len() != self.dict.len() {
            bail!("{} points for {} DICT rows: one dictionary row per point (07 §11)", self.points.len(), self.dict.len());
        }
        if self.surface_starts.len() != self.surface_entries.len() + 1 {
            bail!("the alias start array does not bound the surface index");
        }
        if *self.surface_starts.last().unwrap_or(&0) as usize > self.surface_aliases.len() {
            bail!("the alias start array points past the alias list");
        }
        if self.kind_ids.len() as u64 + si_core::genome::GENOME_KIND_FIRST as u64
            > si_core::genome::GENOME_KIND_SLOTS as u64
        {
            bail!("{} kind slots are past the sealed space (07 §6)", self.kind_ids.len());
        }
        Ok(())
    }

    /// The CSR row of one point, in sealed order. W.3's second cursor reads this.
    pub fn row(&self, point: u32) -> &[CsrElement] {
        let Some(start) = self.csr_starts.get(point as usize) else {
            return &[];
        };
        let Some(end) = self.csr_starts.get(point as usize + 1) else {
            return &[];
        };
        let (start, end) = (*start as usize, *end as usize);
        self.csr.get(start..end).unwrap_or(&[])
    }

    /// The CSR index this point's row starts at — the base of the addresses a
    /// merged walk prints for its arcs (07 §11: genome edges are indices).
    pub fn row_start(&self, point: u32) -> u32 {
        self.csr_starts.get(point as usize).copied().unwrap_or(0)
    }

    /// §5's retrieval space: the folded key reaches every point that speaks it,
    /// first the index entry's own and then its aliases. Binary search over the
    /// sealed key order, which is byte-lexicographic like §4's runs.
    pub fn points_for_folded(&self, key: &[u8]) -> Vec<u32> {
        let Some(index) = self.surface_keys.binary_search(&key.to_vec()).ok() else {
            return Vec::new();
        };
        let entry = match self.surface_entries.get(index) {
            Some(entry) => *entry,
            None => return Vec::new(),
        };
        let start = entry.alias_off as usize;
        let end = self
            .surface_starts
            .get(index + 1)
            .map(|value| *value as usize)
            .unwrap_or(start);
        let mut out = vec![entry.point];
        out.extend(self.surface_aliases.get(start..end).unwrap_or(&[]).iter().copied());
        out
    }

    /// The sealed kind name behind an `edge_kind` slot, or `None` for a slot the
    /// table does not hold (A3's refusal, and the citation string §6 prints).
    pub fn kind_name(&self, edge_kind: u8) -> Option<&str> {
        let offset = edge_kind.checked_sub(si_core::genome::GENOME_KIND_FIRST)? as usize;
        self.kind_ids.get(offset).and_then(|id| self.strings.get(*id))
    }

    /// The forms a DICT row lists, read from the sealed payload blob.
    pub fn forms_of(&self, row: &DictRow) -> Result<Vec<&str>> {
        let start = row.forms_off as usize;
        let end = start + row.forms_len as usize;
        let blob = self
            .forms
            .get(start..end)
            .ok_or_else(|| anyhow::anyhow!("a DICT row's forms payload runs past the blob (A1)"))?;
        let mut out = Vec::new();
        let mut cursor = 0usize;
        while cursor < blob.len() {
            if cursor + 4 > blob.len() {
                bail!("a form is truncated at its length");
            }
            let len = u32::from_le_bytes(blob[cursor..cursor + 4].try_into().expect("4 bytes"))
                as usize;
            cursor += 4;
            let stop = cursor + len;
            if stop > blob.len() {
                bail!("a form runs past its payload");
            }
            out.push(
                std::str::from_utf8(&blob[cursor..stop])
                    .map_err(|e| anyhow::anyhow!("a form is not UTF-8: {e}"))?,
            );
            cursor = stop;
        }
        Ok(out)
    }

    /// The three shelf runs, §M.9's narrower hash input.
    pub fn rows_bytes(&self) -> Vec<u8> {
        let parts = self.sections();
        let mut out = Vec::new();
        for part in &parts[..3] {
            out.extend_from_slice(part);
        }
        out
    }

    /// A parked paradigm table's token lists (07 §7). Rules are citable and never
    /// conducted, so these exist for the cycle check and for an export.
    pub fn triggers_of(&self, row: &RuleRow) -> Result<Vec<u32>> {
        self.tokens_of(row.triggers_off, row.triggers_len)
    }

    pub fn results_of(&self, row: &RuleRow) -> Result<Vec<u32>> {
        self.tokens_of(row.results_off, row.results_len)
    }

    fn tokens_of(&self, off: u32, len: u16) -> Result<Vec<u32>> {
        let start = off as usize;
        let end = start + len as usize;
        let slice = self
            .rule_tokens
            .get(start..end)
            .ok_or_else(|| anyhow::anyhow!("a RULE row's token list runs past the array (A1)"))?;
        Ok(slice.to_vec())
    }

    /// The point table's surface bytes for a point, verbatim. §11 gives one
    /// payload blob to both the forms and the surface bytes, so a point's
    /// `(surface_off, surface_len)` addresses the same blob a DICT row's
    /// `(forms_off, forms_len)` does.
    pub fn surface_of(&self, point: usize) -> Result<&str> {
        let row = self
            .points
            .get(point)
            .ok_or_else(|| anyhow::anyhow!("point {point} is not allocated"))?;
        let start = row.surface_off as usize;
        let end = start + row.surface_len as usize;
        let bytes = self
            .forms
            .get(start..end)
            .ok_or_else(|| anyhow::anyhow!("a point's surface bytes run past the payload"))?;
        std::str::from_utf8(bytes).map_err(|e| anyhow::anyhow!("a surface is not UTF-8: {e}"))
    }

    /// The highest CSR row length, which is what A7 compares to the header's
    /// declared reciprocal-table range.
    pub fn max_degree(&self) -> u32 {
        self.points
            .iter()
            .enumerate()
            .map(|(point, _)| self.row(point as u32).len() as u32)
            .max()
            .unwrap_or(0)
    }

    /// The degree a hub-damp table would be built over (W.4's seed): every row
    /// length, in ascending order, so the table is one per graph and the counter
    /// says which degrees the graph actually reaches.
    pub fn degrees(&self) -> Vec<u32> {
        (0..self.points.len())
            .map(|point| self.row(point as u32).len() as u32)
            .collect()
    }
}

/// Split a section into whole records of `width` bytes, refusing a section whose
/// length is not a multiple of the declared width (07 §11).
fn fixed<'a>(name: &str, bytes: &'a [u8], width: usize) -> Result<Vec<&'a [u8]>> {
    if bytes.len() % width != 0 {
        bail!(
            "{name} is {} bytes, which is not a whole number of {width}-byte rows",
            bytes.len()
        );
    }
    Ok(bytes.chunks(width).collect())
}

fn read_u32s(bytes: &[u8]) -> Result<Vec<u32>> {
    if bytes.len() % 4 != 0 {
        bail!("a u32 array section is {} bytes, not a multiple of four", bytes.len());
    }
    Ok(bytes.chunks_exact(4).map(|chunk| u32::from_le_bytes(chunk.try_into().expect("4 bytes"))).collect())
}

/// `md-5`, hex, lowercase — the seal's own digits (OD-49: identity, not security).
pub fn md5_hex(bytes: &[u8]) -> String {
    let digest = Md5::digest(bytes);
    let mut out = String::with_capacity(32);
    for byte in digest {
        out.push(char::from(b"0123456789abcdef"[byte as usize >> 4]));
        out.push(char::from(b"0123456789abcdef"[byte as usize & 15]));
    }
    out
}

/// The seal, in the two forms the spec asks for. OD-49 fixes `md-5` as identity,
/// not security; the tag is the seal's first eight bytes and `rows_md5` keeps
/// §M.9's narrower "sorted row file" reading checkable beside §4's wider one.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Seal {
    pub seal_md5: String,
    pub rows_md5: String,
    pub version_tag: u64,
}

impl Seal {
    pub fn compute(canonical: &[u8], rows: &[u8]) -> Result<Self> {
        let seal_md5 = md5_hex(canonical);
        Ok(Self {
            seal_md5: seal_md5.clone(),
            rows_md5: md5_hex(rows),
            version_tag: GenomeHeader::tag_u64(&seal_md5)?,
        })
    }
}

/// The read side of A10, and the loader's first act (W.1): the sealed bytes are
/// hashed again and compared to what the header claims.
pub fn check_seal(artifact: &[u8]) -> Result<(GenomeHeader, Vec<Provenance>, GenomeFile)> {
    let container = Container::decode(artifact)?;
    if container.format != Format::Genome {
        bail!("the artifact is a {:?} container, not the frozen genome", container.format);
    }
    let (header, provenance) = split_header(&container.header)?;
    let seal = Seal::compute(&container.payload, &rows_bytes(&container.payload)?)?;
    if seal.seal_md5 != header.seal_md5 {
        bail!(
            "A10: the artifact's own bytes hash to {}, the header seals {}; a file edited after \
             sealing is not loaded (07 §13)",
            seal.seal_md5,
            header.seal_md5
        );
    }
    if seal.rows_md5 != header.rows_md5 {
        bail!(
            "A10: the three shelf runs hash to {}, the header seals {}",
            seal.rows_md5,
            header.rows_md5
        );
    }
    if seal.version_tag != header.version_tag {
        bail!("A10: the header tag {:016x} is not the seal's {:016x}", header.version_tag, seal.version_tag);
    }
    let file = GenomeFile::decode(&container.payload)?;
    Ok((header, provenance, file))
}

#[cfg(test)]
mod tests {
    use si_core::genome::{DictRow, StringTable, GENOME_KIND_FIRST};

    use super::*;

    fn sample() -> GenomeFile {
        let mut strings = StringTable::new();
        strings.intern("warm").unwrap();
        strings.intern("IsA").unwrap();
        let dict = vec![
            DictRow { hyperpoint_id: 1, lemma: 1, forms_off: 0, forms_len: 0, pos: b'a', prov_batch: 0, synset_id: 1, version_tag: 7 },
            DictRow { hyperpoint_id: 2, lemma: 1, forms_off: 0, forms_len: 0, pos: b's', prov_batch: 0, synset_id: 1, version_tag: 7 },
        ];
        let rel = vec![RelRow {
            a_point: 0,
            b_point: 1,
            edge_kind: GENOME_KIND_FIRST,
            weight: 125,
            prov_batch: 0,
            version_tag: 7,
        }];
        GenomeFile {
            dict,
            rel,
            points: vec![
                PointRow { hyperpoint_id: 1, dict_row: 0, surface_off: 0, surface_len: 0, batch: 0 },
                PointRow { hyperpoint_id: 2, dict_row: 1, surface_off: 0, surface_len: 0, batch: 0 },
            ],
            csr_starts: vec![0, 1, 1],
            csr: vec![CsrElement { to_point: 1, edge_kind: GENOME_KIND_FIRST, weight: 125, rel_row: 0 }],
            kind_ids: vec![2],
            strings,
            // The surface index is empty here, and an empty index still carries
            // its one sentinel, exactly as the CSR array does over zero points.
            surface_starts: vec![0],
            ..Default::default()
        }
    }

    #[test]
    fn the_section_list_is_the_sealed_format() {
        assert_eq!(SECTIONS.len(), SECTION_COUNT);
        assert_eq!(&SECTIONS[..3], ["dict_run", "rel_run", "rule_run"]);
        assert_eq!(SECTIONS[SECTION_COUNT - 1], "interner");
    }

    #[test]
    fn a_canonical_file_round_trips_through_its_sections() {
        let file = sample();
        let bytes = file.encode();
        let read = GenomeFile::decode(&bytes).unwrap();
        assert_eq!(read, file);
        assert_eq!(read.row(0).len(), 1);
        assert_eq!(read.max_degree(), 1);
        assert_eq!(read.kind_name(GENOME_KIND_FIRST), Some("IsA"));
        assert_eq!(read.kind_name(0), None);
    }

    #[test]
    fn a_truncated_or_extended_file_is_refused() {
        let bytes = sample().encode();
        assert!(GenomeFile::decode(&bytes[..bytes.len() - 1]).is_err());
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(GenomeFile::decode(&extra).is_err());
    }

    #[test]
    fn the_shape_checks_refuse_an_index_that_lies() {
        let mut file = sample();
        file.csr.clear();
        assert!(file.check_shape().is_err());
        let mut file = sample();
        file.points.pop();
        assert!(file.check_shape().is_err());
    }

    #[test]
    fn a_folded_key_reaches_every_point_that_shares_it() {
        let mut file = sample();
        file.surface_keys = vec![b"warm".to_vec()];
        file.surface_entries = vec![SurfaceEntry { point: 0, alias_off: 0 }];
        file.surface_starts = vec![0, 1];
        file.surface_aliases = vec![1];
        assert_eq!(file.points_for_folded(b"warm"), vec![0, 1]);
        assert!(file.points_for_folded(b"cold").is_empty());
    }

    #[test]
    fn the_tier_rows_refuse_a_non_zero_pad() {
        let row = BucketRow { source: 1, min: 0, max: 10, tier: 40 };
        let bytes = row.encode();
        assert_eq!(BucketRow::decode(&bytes).unwrap(), row);
        let mut mutated = bytes;
        mutated[14] = 1;
        assert!(BucketRow::decode(&mutated).is_err());

        let top = TopRow { source: 1, tier: 40 };
        let bytes = top.encode();
        assert_eq!(TopRow::decode(&bytes).unwrap(), top);
        let mut mutated = bytes;
        mutated[6] = 1;
        assert!(TopRow::decode(&mutated).is_err());
    }

    #[test]
    fn the_seal_digest_is_the_lowercase_hex_of_the_bytes() {
        assert_eq!(md5_hex(b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(md5_hex(b"abc"), "900150983cd24fb0d6963f7d28e17f72");
    }
}
