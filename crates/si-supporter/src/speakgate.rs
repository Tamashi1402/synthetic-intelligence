//! 04 §23 W: the speak gate. Two mechanical layers, both checkable from shelves
//! that already exist, and both run **upstream of sense** — a rejected emission
//! never becomes a sense event, so it costs zero state bytes.
//!
//! The crate sees the store read-only and nothing else: no `a`, no charge, no
//! `bake`, no median, no door, no walk (02 §8's may-never-import list, rule (vi)
//! of docs/12).

use anyhow::Result;
use si_core::addr::{SHELF_FACTS, SHELF_QUESTIONS, SHELF_S_R_PAIRS};
use si_store::interner::fold_key;
use si_store::store::Lattice;

/// The shelves whose owner rows make a token **taught** (01 §7: every arriving
/// token is interned; only an owner teaching event makes it taught, and "taught"
/// is exactly the set layer 1 tests against). `links` is deliberately absent: a
/// bridge is SI's own inference, not a lesson the owner spoke.
const TAUGHT_SHELVES: [u16; 3] = [SHELF_S_R_PAIRS, SHELF_FACTS, SHELF_QUESTIONS];

/// 01 §4.1's provenance speaker byte. `si-learn` owns the writer's copy of this
/// table and the supporter may not import it, so the reader states it here.
const SPEAKER_OWNER: u8 = 0;

/// Layer 1's reason codes and layer 2's, as they appear in the gate log and in
/// `rejection_reasons.json`. Codes, never content words (R-18's rule).
pub const REASON_EMPTY: &str = "speak_empty";
pub const REASON_UNTAUGHT: &str = "speak_layer1_untaught";
pub const REASON_NOVEL_CLAIM: &str = "speak_layer2_novel_claim";

/// R1's split and R2's fold, without importing the surface: the supporter may not
/// depend on `si-front`, and this is the same two steps on the same bytes.
pub fn token_keys(line: &str) -> Vec<String> {
    line.split(|c: char| !c.is_alphanumeric())
        .filter(|piece| !piece.is_empty())
        .map(fold_key)
        .collect()
}

/// One owner row's normalized claim: the folded key sequence of the line the
/// owner actually spoke.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claim {
    pub row_id: u32,
    pub keys: Vec<String>,
}

/// Layer 1's reference set and layer 2's candidate set, derived read-only from
/// the store at the moment of the attempt.
#[derive(Clone, Debug, Default)]
pub struct Lexicon {
    pub taught: Vec<String>,
    pub claims: Vec<Claim>,
}

impl Lexicon {
    pub fn teaches(&self, key: &str) -> bool {
        self.taught.iter().any(|taught| taught == key)
    }

    /// A row whose key sequence **equals** the emission's, or whose sequence
    /// contains it as an ordered run — the restatement test, in order (OD-40(i)).
    fn restate(&self, keys: &[String]) -> Option<u32> {
        let mut best: Option<u32> = None;
        for claim in &self.claims {
            let hit = if claim.keys == keys {
                true
            } else if keys.len() == 1 {
                // An anchor word restates the row that names it.
                claim.keys.iter().any(|key| key == &keys[0])
            } else {
                false
            };
            if hit {
                best = Some(match best {
                    Some(current) => current.min(claim.row_id),
                    None => claim.row_id,
                });
            }
        }
        best
    }
}

/// The two layers' shared read of the store. Aged rows still teach — a word the
/// owner once taught does not un-enter the lexicon when its row is superseded —
/// but only a live row is a claim that still exists to be restated.
pub fn lexicon(store: &Lattice) -> Result<Lexicon> {
    let mut taught: Vec<String> = Vec::new();
    let mut claims: Vec<Claim> = Vec::new();
    for (_, record) in store.rows()? {
        let envelope = &record.envelope;
        if envelope.prov_speaker != SPEAKER_OWNER {
            continue;
        }
        if !TAUGHT_SHELVES.contains(&envelope.shelf_x) {
            continue;
        }
        let mut keys = match store.interner().get(envelope.text_ref) {
            Some(interned) => token_keys(&interned.verbatim),
            None => Vec::new(),
        };
        for handle in store
            .handles_at(envelope.terms_off, envelope.terms_len)
            .unwrap_or_default()
        {
            if let Some(canonical) = store.points().canonical(handle) {
                if !canonical.is_empty() && !keys.iter().any(|key| key == canonical) {
                    keys.push(canonical.to_string());
                }
            }
        }
        for key in &keys {
            if !taught.iter().any(|taught| taught == key) {
                taught.push(key.clone());
            }
        }
        if envelope.is_live() && !keys.is_empty() {
            claims.push(Claim { row_id: envelope.row_id, keys });
        }
    }
    taught.sort();
    claims.sort_by_key(|claim| claim.row_id);
    Ok(Lexicon { taught, claims })
}

/// What one attempt cost, in the machine-readable shape 05 §17 `G12(b).1` names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    pub admitted: bool,
    pub reason: &'static str,
    pub n_untaught_tokens: usize,
    pub novel_claims: usize,
    pub ref_row_id: Option<u32>,
    /// The untaught keys, in order, for the rephrasing loop (§21.2: the gate
    /// returns WHY). Counts and keys, never a sentence.
    pub untaught: Vec<String>,
    pub layer: u8,
}

impl Verdict {
    pub fn rejected(&self) -> bool {
        !self.admitted
    }
}

/// 04 §23.1 — layer 1. Returns the untaught keys of the emission, in order.
pub fn layer1(lexicon: &Lexicon, keys: &[String]) -> Vec<String> {
    keys.iter()
        .filter(|key| !lexicon.teaches(key))
        .cloned()
        .collect()
}

/// 04 §23.2 — layer 2. A strict single-row restatement, row-id-referenced.
pub fn layer2(lexicon: &Lexicon, keys: &[String]) -> (usize, Option<u32>) {
    match lexicon.restate(keys) {
        Some(row_id) => (0, Some(row_id)),
        None => (1, None),
    }
}

/// 04 §23: both layers, in order, on one already-tokenized emission.
pub fn gate(lexicon: &Lexicon, keys: &[String]) -> Verdict {
    if keys.is_empty() {
        return Verdict {
            admitted: false,
            reason: REASON_EMPTY,
            n_untaught_tokens: 0,
            novel_claims: 0,
            ref_row_id: None,
            untaught: Vec::new(),
            layer: 1,
        };
    }
    let untaught = layer1(lexicon, keys);
    if !untaught.is_empty() {
        return Verdict {
            admitted: false,
            reason: REASON_UNTAUGHT,
            n_untaught_tokens: untaught.len(),
            novel_claims: 0,
            ref_row_id: None,
            untaught,
            layer: 1,
        };
    }
    let (novel_claims, ref_row_id) = layer2(lexicon, keys);
    Verdict {
        admitted: novel_claims == 0,
        reason: if novel_claims == 0 { "" } else { REASON_NOVEL_CLAIM },
        n_untaught_tokens: 0,
        novel_claims,
        ref_row_id,
        untaught: Vec::new(),
        layer: 2,
    }
}

/// The whole filter, from the raw line to the verdict. Read-only: this function
/// is why a rejected emission leaves no trace in SI's state.
pub fn check(store: &Lattice, line: &str) -> Result<Verdict> {
    let lexicon = lexicon(store)?;
    Ok(gate(&lexicon, &token_keys(line)))
}
