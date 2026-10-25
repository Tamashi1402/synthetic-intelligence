//! si-tongue — 02 §18, 13 §5, 04 §34 AF.3–AF.4. The seventeenth crate: the
//! phrase walk and its gate.
//!
//! One surface, one caller, and a return type that is the whole removal surface.
//! The walker draws a different *order* over the same licensed chunks; it never
//! draws a new word, because with no handle on the point table a word has no
//! route in here except a [`License`] the caller handed it (**OD-181** as a hole,
//! not a rule — [00 §2](../../engineering/00-stack-workspace.md) win 8). **OD-189**
//! adds the variant bank: other *taught spellings* of the same licensed point,
//! carried as license data. Same claim, different surface, no new word and no
//! second chunk — the caller folds them, and the crate still holds no table.
//!
//! Error behaviour is a refusal, not a type: no `Result`, no panic, no writer. A
//! spent budget, an empty arc set, a missing license entry and a lost slot are all
//! the same answer, and the caller's fallback is the canonical surface unchanged
//! (**G20(j)**'s whole point, [13 §6.1](../../engineering/13-tongue.md)).

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use si_core::addr::edge_kind_is_claim;
use si_core::quant::mul_q12_u16;
use si_core::rng::Rng;
use si_core::tongue::{Style, StyleId};
use si_walk::Arc;
use si_walk::Graph;

/// 13 §0 fact 8: the only multi-character string this crate emits is the joiner,
/// and `tokenize` yields no token from it, so it can never be an unlicensed word.
pub const JOINER: &str = ", ";

/// **OD-189**: one taught spelling of a point the license already carries. A
/// variant is the same `point` and the same folded `keys` — a different surface
/// over the same claim, never a second chunk and never a new word. The caller
/// folds it, because the fold law is not this crate's to own (13 §1.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant {
    pub surface: String,
    pub keys: Vec<String>,
}

impl Variant {
    pub fn new(surface: String, keys: Vec<String>) -> Self {
        Self { surface, keys }
    }
}

/// The licensed vocabulary, as data. `point` is the handle the walk may step
/// onto, `surface` is what prints, `keys` are its folded token keys — folded by
/// the caller, because the fold law is not this crate's to own (13 §1.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct License {
    pub point: u32,
    pub surface: String,
    pub keys: Vec<String>,
    /// **OD-189**'s variant bank: other surfaces taught for this same point.
    /// Empty when the session taught one spelling, and then the walk draws
    /// nothing — the bare fixture's bytes are the pre-bank bytes.
    pub variants: Vec<Variant>,
}

impl License {
    pub fn new(point: u32, surface: String, keys: Vec<String>) -> Self {
        Self { point, surface, keys, variants: Vec::new() }
    }

    pub fn with_variants(mut self, variants: Vec<Variant>) -> Self {
        self.variants = variants;
        self
    }
}

/// 04 §34 AF.3 step 1: the species row owns the ceiling, the draw owns where
/// inside it an individual sits, and neither can exceed the other. A spread of
/// `0` prices the walk at zero extra chunks, which is what the row's own
/// "drawn `0` means" column says it means.
pub fn budget(phrase_max_steps: usize, style: &Style) -> usize {
    (phrase_max_steps as u64 * style.get(StyleId::Spread) as u64 >> 12) as usize
}

/// **OD-189**: draw one taught surface for a licensed point. With no alternative
/// the canonical bytes print and the draw is **not** consumed, so a session that
/// taught one spelling speaks exactly the bytes it spoke before the bank existed
/// (G20.b.2's bare fixture). With alternatives, one `next_q12` picks among them on
/// the walk's own per-render stream (OD-176/OD-183) — a function of
/// `(birth seed, style digest, thought id)`, never of `a`.
fn utter<'a>(entry: &'a License, rng: &mut Rng) -> Option<&'a str> {
    if entry.surface.is_empty() {
        return None;
    }
    let alternatives: Vec<&str> = entry
        .variants
        .iter()
        .map(|variant| variant.surface.as_str())
        .filter(|surface| !surface.is_empty() && *surface != entry.surface)
        .collect();
    if alternatives.is_empty() {
        return Some(entry.surface.as_str());
    }
    let pick = u32::from(rng.next_q12()) % (1 + alternatives.len() as u32);
    let chosen = if pick == 0 {
        entry.surface.as_str()
    } else {
        alternatives[(pick - 1) as usize]
    };
    (!chosen.is_empty()).then_some(chosen)
}

/// The phrase walk, in full. `graph` is the tick's own — this crate cannot build
/// one, and does not try.
pub fn realize(
    graph: &Graph,
    anchor: u32,
    license: &[License],
    slot_keys: &[String],
    style: &Style,
    phrase_max_steps: usize,
    rng: &mut Rng,
) -> Option<String> {
    // A missing license entry is a refusal, not a crash: nothing may print for a
    // point the citation does not name.
    let anchor_license = license.iter().find(|entry| entry.point == anchor)?;
    if anchor_license.surface.is_empty() {
        return None;
    }
    // The pre-flight half of check (c): a slot this line cannot utter is lost
    // before the walk starts, so the caller prints the canonical frame.
    let licensed = licensed_keys(license);
    if slot_keys.iter().any(|key| !licensed.contains(key.as_str())) {
        return None;
    }

    let extra = budget(phrase_max_steps, style);
    let mut parts: Vec<String> = vec![utter(anchor_license, rng)?.to_string()];
    let mut used: BTreeSet<u32> = BTreeSet::new();
    used.insert(anchor);
    let mut current = anchor;
    let mut depth: u8 = 0;

    while parts.len() < 1 + extra {
        let candidates: Vec<Arc> = graph
            .arcs(current)
            // **OD-179**: live arcs only — a newborn says nothing the species
            // text taught it, and **OD-181**: utter-able means licensed.
            .filter(|arc| !arc.is_genome())
            .filter(|arc| !used.contains(&arc.to_point))
            .filter(|arc| license.iter().any(|entry| entry.point == arc.to_point))
            .collect();
        if candidates.is_empty() {
            break;
        }
        // **OD-182**: habit is `bake` plus arena position, so the only ordering
        // this reads is the graph's own formation order.
        let median = median_edge_off(&candidates);
        let mut weights: Vec<u64> = Vec::with_capacity(candidates.len());
        let mut total: u64 = 0;
        for arc in &candidates {
            // Additive leans on a saturating accumulator, `idle.rs`'s shape: a
            // multiplier above 1.0 would be a second arithmetic law.
            let mut weight = u64::from(arc.weight);
            weight = weight.saturating_add(u64::from(mul_q12_u16(style.get(StyleId::Depth), owned_share(arc.kind))));
            weight = weight.saturating_add(u64::from(mul_q12_u16(
                style.get(StyleId::Lean),
                degree_share(graph.out_degree(arc.to_point)),
            )));
            weight = weight.saturating_add(u64::from(mul_q12_u16(
                style.get(StyleId::Recency),
                recency_share(arc.bake, arc.edge_off, median),
            )));
            weight = weight.saturating_add(u64::from(mul_q12_u16(
                style.get(StyleId::Proximity),
                hop_share(depth.saturating_add(1)),
            )));
            total = total.saturating_add(weight);
            weights.push(weight);
        }

        let chosen = if total == 0 {
            0
        } else {
            let fraction = u64::from(rng.next_q12());
            let target = (total * fraction) >> 12;
            let mut cumulative: u64 = 0;
            let mut index = candidates.len() - 1;
            for (candidate_index, weight) in weights.iter().enumerate() {
                cumulative = cumulative.saturating_add(*weight);
                if target < cumulative {
                    index = candidate_index;
                    break;
                }
            }
            index
        };

        let point = candidates[chosen].to_point;
        // The only push into the phrase buffer takes its bytes from a license.
        let entry = license.iter().find(|candidate| candidate.point == point)?;
        // A licensed point with no surface cannot be uttered, and a half-word is
        // not a sentence: G20(c)'s forced failure is this arm, and the caller
        // prints the canonical frame. `utter` owns that refusal now, including
        // the **OD-189** variant draw.
        parts.push(utter(entry, rng)?.to_string());
        used.insert(point);
        current = point;
        depth = depth.saturating_add(1);
    }

    Some(parts.join(JOINER))
}

/// The boundary half of checks (b) and (c) — a pure set comparison over keys the
/// caller already folded, so this crate holds no second fold law (13 §1.2).
/// (b) every emitted word is licensed; (c) every slot the line was generated from
/// is still in it (**OD-173**: slots-present, no decode walk invented).
pub fn realizes_the_slots(line_keys: &[String], slot_keys: &[String], license: &[License]) -> bool {
    let licensed = licensed_keys(license);
    line_keys.iter().all(|key| licensed.contains(key.as_str()))
        && slot_keys.iter().all(|key| line_keys.contains(key))
}

fn licensed_keys(license: &[License]) -> BTreeSet<&str> {
    license
        .iter()
        .flat_map(|entry| {
            entry.keys.iter().chain(entry.variants.iter().flat_map(|variant| variant.keys.iter()))
        })
        .map(String::as_str)
        .collect()
}

/// `owned_share` (04 §34 AF.3's table): formality is claim-side over associative.
/// 1024 rather than 0 because a co-occurrence is still an edge SI earned. The
/// predicate is `edge_kind_is_claim`, not `edge_kind_is_owned` — the owned table's
/// last slot is `6 cooccur-weak`, so the record-shape predicate would call an
/// associative edge a claim (**OD-186**).
fn owned_share(kind: u8) -> u16 {
    if edge_kind_is_claim(kind) {
        4096
    } else {
        1024
    }
}

/// `degree_share`: sixteen buckets, the last of them saturating, so a hub of 40
/// and a hub of 16 are one preference.
fn degree_share(degree: u32) -> u16 {
    degree.min(16) as u16 * 256
}

/// `recency_share`: `bake >> 4` plus the arena-position bonus above the current
/// arc set's median. No level-at-formation term (**OD-168**, **OD-182**).
fn recency_share(bake: u16, edge_off: u32, median: u32) -> u16 {
    let base = (bake >> 4).min(4096);
    if edge_off > median {
        base.saturating_add(1024).min(4096)
    } else {
        base
    }
}

/// `hop_share`: how tightly it sticks to the thing asked about. The counter is
/// the candidate's distance from the anchor, so hop 1 is preferred over hop 2 and
/// hop 2 over hop 3+ — which is what docs/27's law says, and a counter that
/// started at the current point instead would rank the anchor's own neighbours
/// last.
fn hop_share(hops: u8) -> u16 {
    match hops {
        1 => 4096,
        2 => 2048,
        _ => 1024,
    }
}

fn median_edge_off(candidates: &[Arc]) -> u32 {
    let mut offsets: Vec<u32> = candidates.iter().map(|arc| arc.edge_off).collect();
    offsets.sort_unstable();
    offsets[(offsets.len() - 1) / 2]
}
