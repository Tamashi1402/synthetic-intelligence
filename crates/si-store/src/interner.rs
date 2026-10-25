//! 01 §7: one `u32` string table per session. The string is stored verbatim;
//! the lookup key is folded and lowercased, and a fold collision is an alias
//! candidate, never a merge (R2/R-04).

use anyhow::{bail, Result};

use si_core::encode::Writer;

/// 01 §7: `0` = `""`, `u32::MAX` = NULL.
pub const NULL: u32 = u32::MAX;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interned {
    pub verbatim: String,
    pub folded: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interner {
    entries: Vec<Interned>,
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}

impl Interner {
    pub fn new() -> Self {
        Self { entries: vec![Interned { verbatim: String::new(), folded: String::new() }] }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn get(&self, id: u32) -> Option<&Interned> {
        self.entries.get(id as usize)
    }

    pub fn folded(&self, id: u32) -> Option<&str> {
        self.get(id).map(|entry| entry.folded.as_str())
    }

    /// Verbatim identity: the same bytes reuse their id; a folded collision with
    /// different bytes gets its own id and is reported by `lookup`, never merged.
    pub fn intern(&mut self, surface: &str) -> u32 {
        if let Some(index) = self.entries.iter().position(|entry| entry.verbatim == surface) {
            return index as u32;
        }
        let id = self.entries.len() as u32;
        self.entries.push(Interned { verbatim: surface.to_string(), folded: fold_key(surface) });
        id
    }

    /// R2's alias set: every id whose folded key equals this surface's. The
    /// tokenizer lights all of them, split by count (OD-55); it never merges.
    pub fn lookup(&self, surface: &str) -> Vec<u32> {
        let key = fold_key(surface);
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.folded == key)
            .map(|(index, _)| index as u32)
            .collect()
    }

    pub fn surfaces(&self) -> impl Iterator<Item = &Interned> {
        self.entries.iter()
    }

    /// Snapshot form: id order, verbatim bytes. The folded key is recomputed on
    /// read, so it cannot drift from the one folding function.
    pub fn from_surfaces(surfaces: Vec<String>) -> Result<Self> {
        if surfaces.first().map(|s| s.as_str()) != Some("") {
            bail!("interner id 0 is not the empty string");
        }
        let mut seen = Vec::new();
        let mut entries = Vec::with_capacity(surfaces.len());
        for surface in surfaces {
            if seen.iter().any(|s: &String| s == &surface) {
                bail!("interner stores the same verbatim string twice");
            }
            entries.push(Interned { folded: fold_key(&surface), verbatim: surface.clone() });
            seen.push(surface);
        }
        Ok(Self { entries })
    }

    /// 01 §10 item 4: `id → folded bytes`, sorted by id (the table's own order).
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u32(self.entries.len() as u32);
        for entry in &self.entries {
            w.str(&entry.folded);
        }
        w.finish()
    }
}

/// R1/R2: lowercase, punctuation-split, whitespace-collapsed, plural-folded.
/// Surface bytes stay verbatim in the table; only this lookup key folds.
pub fn fold_key(surface: &str) -> String {
    surface
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| plural_fold(&token.to_lowercase()))
        .collect::<Vec<_>>()
        .join(" ")
}

fn plural_fold(token: &str) -> String {
    if token.len() > 3 && token.ends_with("ies") {
        let mut out = token[..token.len() - 3].to_string();
        out.push('y');
        return out;
    }
    for suffix in ["ches", "shes", "xes", "zes", "ses"] {
        if token.len() > suffix.len() && token.ends_with(suffix) {
            return token[..token.len() - 2].to_string();
        }
    }
    if token.len() > 2 && token.ends_with('s') {
        return token[..token.len() - 1].to_string();
    }
    token.to_string()
}
