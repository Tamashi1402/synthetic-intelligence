//! 01 §1: the hyperpoint table. A walk lights a `point_handle: u32` allocated on
//! first arrival; alongside it the table stores the content address
//! `hyperpoint_id: u64` over the canonical key string — the folded term, a frame
//! node, or `(frame, slot, entity)` for a named dark spot. Identity is dedup, not
//! similarity: the hash is a name (OD-49, I1).
//!
//! The table is session state, not a shelf: a point is allocated by arriving, and
//! a row cites one. It rides the `SIst` payload so a resume allocates the same
//! handles for the same arrival order; it is not one of the nine hashed classes.

use std::collections::BTreeMap;

use anyhow::{bail, Result};

use si_core::addr::hyperpoint_id;
use si_core::encode::{Reader, Writer};
use si_core::patterns::PatternId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointEntry {
    pub canonical: String,
    /// The first verbatim surface this point speaks, for the monologue face.
    /// `None` for a frame node.
    pub surface: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Points {
    entries: Vec<PointEntry>,
    by_canonical: BTreeMap<String, u32>,
    by_key: BTreeMap<String, Vec<u32>>,
    /// 07 §9 W.2: under CONFIG ENGINE the genome owns handles `0 … P_gen−1` of
    /// this same `u32` space, so a live arrival allocates from here upward. The
    /// base is *derived* — the sealed artifact's own point count, guarded by
    /// 01 §10 item 8's tag at every resume — so it is written nowhere, and CONFIG
    /// SI's `0` leaves the payload byte-for-byte as it was.
    base: u32,
}

impl Points {
    pub fn new() -> Self {
        Self::default()
    }

    /// W.2's allocation base for a CONFIG ENGINE session.
    pub fn with_base(base: u32) -> Self {
        Self { base, ..Default::default() }
    }

    pub fn base(&self) -> u32 {
        self.base
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[PointEntry] {
        &self.entries
    }

    pub fn get(&self, handle: u32) -> Option<&PointEntry> {
        let offset = handle.checked_sub(self.base)? as usize;
        self.entries.get(offset)
    }

    pub fn surface(&self, handle: u32) -> Option<&str> {
        self.get(handle).and_then(|entry| entry.surface.as_deref())
    }

    pub fn canonical(&self, handle: u32) -> Option<&str> {
        self.get(handle).map(|entry| entry.canonical.as_str())
    }

    pub fn hyperpoint(&self, handle: u32) -> Option<u64> {
        self.canonical(handle).map(hyperpoint_id)
    }

    /// The alias set for a folded key: one handle in the ordinary case. Every
    /// member is lit with the shared quantum (OD-55); an empty answer is 04
    /// §4.2's novelty event.
    pub fn aliases(&self, key: &str) -> Vec<u32> {
        self.by_key.get(key).cloned().unwrap_or_default()
    }

    pub fn by_key(&self) -> &BTreeMap<String, Vec<u32>> {
        &self.by_key
    }

    /// One point per folded term. The canonical key is the folded term itself
    /// (01 §1), so `Apples` and `apples` are one node and the fold is the only
    /// merge this table performs.
    pub fn ensure_term(&mut self, key: &str, surface: &str) -> u32 {
        if let Some(entry) = self.by_key.get(key).and_then(|set| set.first()) {
            return *entry;
        }
        let handle = self.alloc(key.to_string(), Some(surface.to_string()));
        self.by_key.entry(key.to_string()).or_default().push(handle);
        handle
    }

    pub fn ensure_frame(&mut self, frame: PatternId) -> u32 {
        self.alloc(format!("frame:{}", frame.name()), None)
    }

    /// The named dark spot: `(frame, slot, entity)` (01 §1), where `entity` is
    /// the entity's own point handle.
    pub fn ensure_named(&mut self, frame: PatternId, slot: u16, entity: u32) -> u32 {
        self.alloc(format!("frame:{}|slot:{slot:04x}|entity:{entity}", frame.name()), None)
    }

    fn alloc(&mut self, canonical: String, surface: Option<String>) -> u32 {
        if let Some(handle) = self.by_canonical.get(&canonical) {
            return *handle;
        }
        let handle = self.base + self.entries.len() as u32;
        self.entries.push(PointEntry { canonical: canonical.clone(), surface });
        self.by_canonical.insert(canonical, handle);
        handle
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u32(self.entries.len() as u32);
        for entry in &self.entries {
            w.str(&entry.canonical);
            match &entry.surface {
                Some(surface) => {
                    w.u8(1);
                    w.str(surface);
                }
                None => w.u8(0),
            }
        }
        w.finish()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        Self::decode_with(bytes, 0)
    }

    /// The payload carries no base: W.2's base is a *derived* reading of the
    /// pinned artifact, so a resume passes the same one a wake did — and if it
    /// does not, item 8's tag comparison has already refused the session.
    pub fn decode_with(bytes: &[u8], base: u32) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let count = r.u32()? as usize;
        if count > bytes.len() {
            bail!("point count {count} is not plausible");
        }
        let mut points = Self::with_base(base);
        for _ in 0..count {
            let canonical = r.str()?.to_string();
            let surface = match r.u8()? {
                0 => None,
                1 => Some(r.str()?.to_string()),
                other => bail!("point surface tag {other} is not 0 or 1"),
            };
            let handle = base + points.entries.len() as u32;
            if points.by_canonical.insert(canonical.clone(), handle).is_some() {
                bail!("duplicate point canonical {canonical:?}");
            }
            if !canonical.starts_with("frame:") {
                points.by_key.entry(canonical.clone()).or_default().push(handle);
            }
            points.entries.push(PointEntry { canonical, surface });
        }
        if !r.is_empty() {
            bail!("point table has {} trailing bytes", r.remaining());
        }
        Ok(points)
    }
}
