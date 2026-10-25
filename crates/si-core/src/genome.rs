//! 07 §11 / 01 §6: the packed frozen records, the sealed string table and the
//! the frozen genome container's header. These are **not** the live 112 B envelope: at `t = 0` every
//! mutable column of §4 is constant or meaningless, and a genome row has no
//! `bake`, no `a_at` and no `charge_off` field at all — so stamping a corpus edge
//! is a missing field rather than a policy violation (07 §9, G6(c)).
//!
//! Every encoder here is the one the sealed bytes are made of, so the field
//! order below *is* the canonical order 07 §4 sorts on. Decoding refuses a
//! non-zero reserved byte is refused rather than ignored (A6).

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

pub const DICT_BYTES: usize = 32;
pub const REL_BYTES: usize = 16;
pub const RULE_BYTES: usize = 16;
/// 07 §11's aux point table: one entry per genome point, sealed order.
pub const POINT_BYTES: usize = 24;
/// 07 §11: the CSR row start, one `u32` per point plus a terminal sentinel.
pub const CSR_START_BYTES: usize = 4;
/// 07 §11: a surface index entry — the point it reaches and where its alias list
/// starts in the point table.
pub const SURFACE_ENTRY_BYTES: usize = 8;
/// A CSR genome element: the far point plus the kind slot and tier.
pub const CSR_ELEMENT_BYTES: usize = 12;
/// 07 §6/§10: the sealed kind table and the sealed batch table are both 256-slot
/// spaces, and both overflow as a hard fail at M.5, never as a clamp.
pub const GENOME_KIND_SLOTS: usize = 256;
pub const GENOME_BATCH_SLOTS: usize = 256;
/// 07 §6: slots `0..=6` are OWNED-only, so the genome's own vocabulary starts at
/// `EDGE_KIND_GENOME_FIRST` and has this many names available.
pub const GENOME_KIND_FIRST: u8 = 7;
pub const GENOME_KIND_CAPACITY: usize = GENOME_KIND_SLOTS - GENOME_KIND_FIRST as usize;
/// 07 §8: the derived ceiling on the `u8` corpus grid. `125 << 4 = 2000 ≤ 2007`,
/// `126 << 4 = 2016` is over the weakest owned birth value, so A4 fails it.
pub const GENOME_WEIGHT_CEILING: u8 = 125;

/// `DICT` — shelf 1, the dictionary row. 07 §11's field list sums to 28, and the
/// declared width is 32, so the last four bytes are reserved-zero padding; that
/// is the only reading that satisfies both the width and A6.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DictRow {
    pub hyperpoint_id: u64,
    /// Interned lemma (`StringTable` id).
    pub lemma: u32,
    pub forms_off: u32,
    pub forms_len: u16,
    /// The source's own part-of-speech byte, verbatim; no mapping exists here
    /// because a mapping is relabeling (07 §5).
    pub pos: u8,
    pub prov_batch: u8,
    /// A **local** synset id: the canonical key string lives in the interner and
    /// this is one `u32` into it (07 §11).
    pub synset_id: u32,
    /// The source's version pin from the manifest. This is *not* the seal: the
    /// content hash cannot be stored inside the bytes it hashes (07 §4, §13).
    pub version_tag: u32,
}

impl DictRow {
    pub fn encode(&self) -> [u8; DICT_BYTES] {
        let mut out = [0u8; DICT_BYTES];
        out[0..8].copy_from_slice(&self.hyperpoint_id.to_le_bytes());
        out[8..12].copy_from_slice(&self.lemma.to_le_bytes());
        out[12..16].copy_from_slice(&self.forms_off.to_le_bytes());
        out[16..18].copy_from_slice(&self.forms_len.to_le_bytes());
        out[18] = self.pos;
        out[19] = self.prov_batch;
        out[20..24].copy_from_slice(&self.synset_id.to_le_bytes());
        out[24..28].copy_from_slice(&self.version_tag.to_le_bytes());
        // 28..32 stays zero: the reserved tail.
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != DICT_BYTES {
            bail!("DICT record is {} bytes, want {DICT_BYTES}", bytes.len());
        }
        if bytes[28..32] != [0, 0, 0, 0] {
            bail!("DICT record carries a non-zero reserved tail (A6)");
        }
        Ok(Self {
            hyperpoint_id: u64::from_le_bytes(bytes[0..8].try_into().expect("8 bytes")),
            lemma: u32::from_le_bytes(bytes[8..12].try_into().expect("4 bytes")),
            forms_off: u32::from_le_bytes(bytes[12..16].try_into().expect("4 bytes")),
            forms_len: u16::from_le_bytes(bytes[16..18].try_into().expect("2 bytes")),
            pos: bytes[18],
            prov_batch: bytes[19],
            synset_id: u32::from_le_bytes(bytes[20..24].try_into().expect("4 bytes")),
            version_tag: u32::from_le_bytes(bytes[24..28].try_into().expect("4 bytes")),
        })
    }
}

/// `REL` — shelf 2, one corpus correlation. The 16th byte is reserved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RelRow {
    pub a_point: u32,
    pub b_point: u32,
    /// A slot in the sealed kind table (07 §6), never a name.
    pub edge_kind: u8,
    /// The `u8` corpus grid; the walk converts by shift (`<< 4`) to Q12 (07 §8).
    pub weight: u8,
    pub prov_batch: u8,
    pub version_tag: u32,
}

impl RelRow {
    pub fn encode(&self) -> [u8; REL_BYTES] {
        let mut out = [0u8; REL_BYTES];
        out[0..4].copy_from_slice(&self.a_point.to_le_bytes());
        out[4..8].copy_from_slice(&self.b_point.to_le_bytes());
        out[8] = self.edge_kind;
        out[9] = self.weight;
        out[10] = self.prov_batch;
        out[11] = 0;
        out[12..16].copy_from_slice(&self.version_tag.to_le_bytes());
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != REL_BYTES {
            bail!("REL record is {} bytes, want {REL_BYTES}", bytes.len());
        }
        if bytes[11] != 0 {
            bail!("REL record carries a non-zero reserved byte (A6)");
        }
        Ok(Self {
            a_point: u32::from_le_bytes(bytes[0..4].try_into().expect("4 bytes")),
            b_point: u32::from_le_bytes(bytes[4..8].try_into().expect("4 bytes")),
            edge_kind: bytes[8],
            weight: bytes[9],
            prov_batch: bytes[10],
            version_tag: u32::from_le_bytes(bytes[12..16].try_into().expect("4 bytes")),
        })
    }

    /// 07 §8: the shift is the whole conversion, so no rounding table exists.
    pub const fn weight_q12(&self) -> u16 {
        (self.weight as u16) << 4
    }
}

/// `RULE` — shelf 3, the parked paradigm tables (OD-63): sealed, loaded,
/// citable, never conducted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuleRow {
    pub triggers_off: u32,
    pub triggers_len: u16,
    pub results_off: u32,
    pub results_len: u16,
    /// The source's own operator byte, verbatim.
    pub operator: u8,
    pub prov_batch: u8,
}

impl RuleRow {
    pub fn encode(&self) -> [u8; RULE_BYTES] {
        let mut out = [0u8; RULE_BYTES];
        out[0..4].copy_from_slice(&self.triggers_off.to_le_bytes());
        out[4..6].copy_from_slice(&self.triggers_len.to_le_bytes());
        out[6..10].copy_from_slice(&self.results_off.to_le_bytes());
        out[10..12].copy_from_slice(&self.results_len.to_le_bytes());
        out[12] = self.operator;
        out[13] = self.prov_batch;
        out[14..16].copy_from_slice(&[0, 0]);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != RULE_BYTES {
            bail!("RULE record is {} bytes, want {RULE_BYTES}", bytes.len());
        }
        if bytes[14..16] != [0, 0] {
            bail!("RULE record carries a non-zero reserved tail (A6)");
        }
        Ok(Self {
            triggers_off: u32::from_le_bytes(bytes[0..4].try_into().expect("4 bytes")),
            triggers_len: u16::from_le_bytes(bytes[4..6].try_into().expect("2 bytes")),
            results_off: u32::from_le_bytes(bytes[6..10].try_into().expect("4 bytes")),
            results_len: u16::from_le_bytes(bytes[10..12].try_into().expect("2 bytes")),
            operator: bytes[12],
            prov_batch: bytes[13],
        })
    }
}

/// The aux point table entry: the genome half of the shared `u32` handle space
/// (W.2). 07 §11's fields sum to 20 and the declared width is 24, so the tail is
/// reserved padding, checked on read like every other reserved byte.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PointRow {
    pub hyperpoint_id: u64,
    /// The `DICT` row index this point speaks.
    pub dict_row: u32,
    pub surface_off: u32,
    pub surface_len: u16,
    pub batch: u8,
}

impl PointRow {
    pub fn encode(&self) -> [u8; POINT_BYTES] {
        let mut out = [0u8; POINT_BYTES];
        out[0..8].copy_from_slice(&self.hyperpoint_id.to_le_bytes());
        out[8..12].copy_from_slice(&self.dict_row.to_le_bytes());
        out[12..16].copy_from_slice(&self.surface_off.to_le_bytes());
        out[16..18].copy_from_slice(&self.surface_len.to_le_bytes());
        out[18] = self.batch;
        // 19..24 stays zero: the declared pad plus the reserved tail.
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != POINT_BYTES {
            bail!("point record is {} bytes, want {POINT_BYTES}", bytes.len());
        }
        if bytes[19..24] != [0, 0, 0, 0, 0] {
            bail!("point record carries a non-zero reserved tail (A6)");
        }
        Ok(Self {
            hyperpoint_id: u64::from_le_bytes(bytes[0..8].try_into().expect("8 bytes")),
            dict_row: u32::from_le_bytes(bytes[8..12].try_into().expect("4 bytes")),
            surface_off: u32::from_le_bytes(bytes[12..16].try_into().expect("4 bytes")),
            surface_len: u16::from_le_bytes(bytes[16..18].try_into().expect("2 bytes")),
            batch: bytes[18],
        })
    }
}

/// A surface index entry: one folded key reaches this point, and its alias list
/// starts at `alias_off` in the point table (07 §5 — the fold may alias, never
/// merge).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SurfaceEntry {
    pub point: u32,
    pub alias_off: u32,
}

impl SurfaceEntry {
    pub fn encode(&self) -> [u8; SURFACE_ENTRY_BYTES] {
        let mut out = [0u8; SURFACE_ENTRY_BYTES];
        out[0..4].copy_from_slice(&self.point.to_le_bytes());
        out[4..8].copy_from_slice(&self.alias_off.to_le_bytes());
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != SURFACE_ENTRY_BYTES {
            bail!("surface entry is {} bytes, want {SURFACE_ENTRY_BYTES}", bytes.len());
        }
        Ok(Self {
            point: u32::from_le_bytes(bytes[0..4].try_into().expect("4 bytes")),
            alias_off: u32::from_le_bytes(bytes[4..8].try_into().expect("4 bytes")),
        })
    }
}

/// A CSR genome element: 12 bytes, so it keeps the `% 4` discipline of §11, and
/// it carries the shelf-2 row address `rel_row` — the `(0, 2, y)` citation a BASE
/// trace line prints (07 §7's citability, which the walk never needs).
/// 07 §11 counts genome edges as **indices**, not byte offsets, which is what
/// lifts the edge ceiling to 4.29·10⁹.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CsrElement {
    pub to_point: u32,
    pub edge_kind: u8,
    pub weight: u8,
    pub rel_row: u32,
}

impl CsrElement {
    pub fn encode(&self) -> [u8; CSR_ELEMENT_BYTES] {
        let mut out = [0u8; CSR_ELEMENT_BYTES];
        out[0..4].copy_from_slice(&self.to_point.to_le_bytes());
        out[4] = self.edge_kind;
        out[5] = self.weight;
        // 6..8 stays zero: the reserved pair.
        out[8..12].copy_from_slice(&self.rel_row.to_le_bytes());
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != CSR_ELEMENT_BYTES {
            bail!("CSR element is {} bytes, want {CSR_ELEMENT_BYTES}", bytes.len());
        }
        if bytes[6..8] != [0, 0] {
            bail!("CSR element carries a non-zero reserved pair (A6)");
        }
        Ok(Self {
            to_point: u32::from_le_bytes(bytes[0..4].try_into().expect("4 bytes")),
            edge_kind: bytes[4],
            weight: bytes[5],
            rel_row: u32::from_le_bytes(bytes[8..12].try_into().expect("4 bytes")),
        })
    }

    pub const fn weight_q12(&self) -> u16 {
        (self.weight as u16) << 4
    }
}

/// The sealed string table. `07 §4` puts the interned strings inside the hashed
/// bytes and orders them by id, so a relation name changing is a version change.
/// Id 0 is the empty string, the same convention the live interner uses.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StringTable {
    entries: Vec<String>,
    index: std::collections::BTreeMap<String, u32>,
}

impl StringTable {
    pub fn new() -> Self {
        Self { entries: vec![String::new()], index: Default::default() }
    }

    pub fn intern(&mut self, text: &str) -> Result<u32> {
        if let Some(id) = self.index.get(text) {
            return Ok(*id);
        }
        if self.entries.len() > u32::MAX as usize {
            bail!("the sealed string table is full");
        }
        let id = self.entries.len() as u32;
        self.entries.push(text.to_string());
        self.index.insert(text.to_string(), id);
        Ok(id)
    }

    pub fn get(&self, id: u32) -> Option<&str> {
        self.entries.get(id as usize).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.len() <= 1
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    /// `u32 count | (u32 len · bytes)…`, ascending by id (07 §4).
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.entries.len() * 8);
        out.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());
        for entry in &self.entries {
            out.extend_from_slice(&(entry.len() as u32).to_le_bytes());
            out.extend_from_slice(entry.as_bytes());
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 4 {
            bail!("the sealed string table is shorter than its count");
        }
        let count = u32::from_le_bytes(bytes[0..4].try_into().expect("4 bytes")) as usize;
        let mut table = Self::default();
        let mut off = 4usize;
        for id in 0..count {
            if off + 4 > bytes.len() {
                bail!("sealed string {id} is truncated at its length prefix");
            }
            let len = u32::from_le_bytes(bytes[off..off + 4].try_into().expect("4 bytes")) as usize;
            off += 4;
            let end = off.checked_add(len).ok_or_else(|| anyhow::anyhow!("sealed string length overflows"))?;
            if end > bytes.len() {
                bail!("sealed string {id} is truncated");
            }
            let text = std::str::from_utf8(&bytes[off..end])
                .map_err(|e| anyhow::anyhow!("sealed string {id} is not UTF-8: {e}"))?
                .to_string();
            off = end;
            if id == 0 {
                if !text.is_empty() {
                    bail!("sealed string 0 must be the empty string, like the live interner");
                }
            } else if table.index.insert(text.clone(), id as u32).is_some() {
                bail!("the sealed string table repeats {text:?}; interned ids are canonical");
            }
            table.entries.push(text);
        }
        if off != bytes.len() {
            bail!("the sealed string table has {} trailing bytes", bytes.len() - off);
        }
        Ok(table)
    }
}

/// The frozen genome container header (01 §9). `deny_unknown_fields` is the schema-version
/// lie detector: a header written by a future build is refused here, not partially
/// understood.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenomeHeader {
    /// `Format::Genome.name()`, so the magic's full format name travels with it.
    pub format: String,
    pub row_schema_version: u32,
    /// 07 §12 A10: `md-5` over the canonical file the artifact is made of.
    pub seal_md5: String,
    /// The same hash over only the three sorted shelf runs, printed so both
    /// readings of 07 §M.9 and 07 §4 are checkable (see `build::seal`).
    pub rows_md5: String,
    /// 01 §10 item 8 / 07 §13: the `u64` content tag, taken from the seal's first
    /// eight bytes, little-endian — `identity not security` (OD-49).
    pub version_tag: u64,
    /// A7's bound: the hub-damp reciprocal table's declared range, from the
    /// sealed header rather than from a host constant.
    pub degree_table_range: u32,
    pub kind_slots: u32,
    pub batch_slots: u32,
    pub dict_rows: u32,
    pub rel_rows: u32,
    pub rule_rows: u32,
    pub points: u32,
    pub edges: u32,
    pub kinds: u32,
    pub batches: u32,
    pub forms_bytes: u32,
    pub surface_keys: u32,
}

impl GenomeHeader {
    pub const ROW_SCHEMA_VERSION: u32 = 1;

    pub fn to_json(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }

    pub fn from_json(value: &serde_json::Value) -> Result<Self> {
        let header: Self = serde_json::from_value(value.clone())?;
        if header.format != crate::container::Format::Genome.name() {
            bail!("the container names format {:?}, not the frozen genome format", header.format);
        }
        if header.row_schema_version != Self::ROW_SCHEMA_VERSION {
            bail!(
                "row schema version {} is not this build's {}",
                header.row_schema_version,
                Self::ROW_SCHEMA_VERSION
            );
        }
        if header.kind_slots as usize != GENOME_KIND_SLOTS
            || header.batch_slots as usize != GENOME_BATCH_SLOTS
        {
            bail!(
                "the sealed slot spaces are {}/{}, want {GENOME_KIND_SLOTS}/{GENOME_BATCH_SLOTS}",
                header.kind_slots,
                header.batch_slots
            );
        }
        if header.kinds > header.kind_slots {
            bail!("{} sealed kind names exceed the {}-slot space", header.kinds, header.kind_slots);
        }
        if header.batches > header.batch_slots {
            bail!(
                "{} sealed batches exceed the {}-slot space",
                header.batches,
                header.batch_slots
            );
        }
        if header.rel_rows as u64 != header.edges as u64 {
            bail!(
                "the CSR index covers {} edges but shelf 2 holds {} rows",
                header.edges,
                header.rel_rows
            );
        }
        Ok(header)
    }

    /// 07 §5: `hyperpoint_id` is the md5 prefix of the canonical key string.
    pub fn tag_u64(seal_md5: &str) -> Result<u64> {
        let bytes = hex_bytes(seal_md5)?;
        if bytes.len() != 16 {
            bail!("a content seal is 16 bytes of md5, got {}", bytes.len());
        }
        Ok(u64::from_le_bytes(bytes[0..8].try_into().expect("8 bytes")))
    }
}

/// The eight bytes the `u64` content tag is read from — the seal's own digest,
/// little-endian (07 §13 / 01 §10 item 8).
pub fn hex_bytes(text: &str) -> Result<Vec<u8>> {
    let bytes = text.as_bytes();
    if bytes.len() % 2 != 0 {
        bail!("{text:?} is not a hex digest");
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let hi = hex_nibble(pair[0])?;
        let lo = hex_nibble(pair[1])?;
        out.push(hi << 4 | lo);
    }
    Ok(out)
}

fn hex_nibble(byte: u8) -> Result<u8> {
    Ok(match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => bail!("hex digest carries a non-hex byte {}", byte as char),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_packed_widths_are_the_declared_ones() {
        assert_eq!(DICT_BYTES, crate::GENOME_REC_BYTES[0]);
        assert_eq!(REL_BYTES, crate::GENOME_REC_BYTES[1]);
        assert_eq!(RULE_BYTES, crate::GENOME_REC_BYTES[2]);
        assert_eq!(POINT_BYTES % 4, 0);
        assert_eq!(SURFACE_ENTRY_BYTES % 4, 0);
    }

    #[test]
    fn every_record_round_trips_and_pads_zero() {
        let dict = DictRow {
            hyperpoint_id: 0x0123_4567_89ab_cdef,
            lemma: 7,
            forms_off: 128,
            forms_len: 5,
            pos: b'n',
            prov_batch: 1,
            synset_id: 9,
            version_tag: 31,
        };
        let bytes = dict.encode();
        assert_eq!(&bytes[28..32], &[0, 0, 0, 0]);
        assert_eq!(DictRow::decode(&bytes).unwrap(), dict);

        let rel = RelRow {
            a_point: 3,
            b_point: 9,
            edge_kind: GENOME_KIND_FIRST,
            weight: GENOME_WEIGHT_CEILING,
            prov_batch: 2,
            version_tag: 31,
        };
        let bytes = rel.encode();
        assert_eq!(bytes[11], 0);
        assert_eq!(RelRow::decode(&bytes).unwrap(), rel);
        assert_eq!(rel.weight_q12(), 2000);

        let rule =
            RuleRow { triggers_off: 0, triggers_len: 2, results_off: 4, results_len: 1, operator: 3, prov_batch: 2 };
        let bytes = rule.encode();
        assert_eq!(&bytes[14..16], &[0, 0]);
        assert_eq!(RuleRow::decode(&bytes).unwrap(), rule);

        let point = PointRow { hyperpoint_id: 5, dict_row: 1, surface_off: 0, surface_len: 4, batch: 0 };
        let bytes = point.encode();
        assert_eq!(&bytes[19..24], &[0, 0, 0, 0, 0]);
        assert_eq!(PointRow::decode(&bytes).unwrap(), point);

        let element = CsrElement { to_point: 11, edge_kind: 9, weight: 125, rel_row: 3 };
        let bytes = element.encode();
        assert_eq!(&bytes[6..8], &[0, 0]);
        assert_eq!(CsrElement::decode(&bytes).unwrap(), element);
        assert_eq!(element.weight_q12(), 2000);

        let entry = SurfaceEntry { point: 2, alias_off: 40 };
        let bytes = entry.encode();
        assert_eq!(SurfaceEntry::decode(&bytes).unwrap(), entry);
    }

    #[test]
    fn a_non_zero_reserved_byte_is_refused_not_trusted() {
        let mut bytes = RelRow {
            a_point: 1,
            b_point: 2,
            edge_kind: 7,
            weight: 10,
            prov_batch: 0,
            version_tag: 0,
        }
        .encode();
        bytes[11] = 0x7f;
        assert!(RelRow::decode(&bytes).is_err());

        let mut bytes = DictRow::default().encode();
        bytes[31] = 1;
        assert!(DictRow::decode(&bytes).is_err());

        let mut bytes = PointRow::default().encode();
        bytes[22] = 1;
        assert!(PointRow::decode(&bytes).is_err());

        let mut bytes = CsrElement::default().encode();
        bytes[7] = 1;
        assert!(CsrElement::decode(&bytes).is_err());
    }

    #[test]
    fn the_string_table_round_trips_in_id_order() {
        let mut table = StringTable::new();
        table.intern("IsA").unwrap();
        table.intern("DefinedAs").unwrap();
        assert_eq!(table.intern("IsA").unwrap(), 1);
        let bytes = table.canonical_bytes();
        let read = StringTable::decode(&bytes).unwrap();
        assert_eq!(read, table);
        assert_eq!(read.get(1), Some("IsA"));
        assert!(StringTable::decode(&[0, 0, 0, 1, 0, 0, 0, 1, b'x']).is_err());
    }

    #[test]
    fn the_header_refuses_a_lie_about_its_own_shape() {
        let header = GenomeHeader {
            format: crate::container::Format::Genome.name().to_string(),
            row_schema_version: GenomeHeader::ROW_SCHEMA_VERSION,
            seal_md5: "00".repeat(16),
            rows_md5: "11".repeat(16),
            version_tag: 0,
            degree_table_range: 64,
            kind_slots: GENOME_KIND_SLOTS as u32,
            batch_slots: GENOME_BATCH_SLOTS as u32,
            dict_rows: 2,
            rel_rows: 1,
            rule_rows: 0,
            points: 2,
            edges: 1,
            kinds: 1,
            batches: 1,
            forms_bytes: 8,
            surface_keys: 2,
        };
        let json = header.to_json().unwrap();
        assert_eq!(GenomeHeader::from_json(&json).unwrap(), header);
        assert!(GenomeHeader::tag_u64(&header.seal_md5).is_ok());

        let mut wrong = json.clone();
        wrong["row_schema_version"] = serde_json::json!(99);
        assert!(GenomeHeader::from_json(&wrong).is_err());

        let mut unknown = json.clone();
        unknown["extra"] = serde_json::json!(1);
        assert!(GenomeHeader::from_json(&unknown).is_err());

        let mut mismatched = json.clone();
        mismatched["edges"] = serde_json::json!(4);
        assert!(GenomeHeader::from_json(&mismatched).is_err());
    }
}
