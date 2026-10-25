//! 04 §21 U.1 / 02 §7: copy, never create.
//!
//! v1 is a template copier: the answer is the top cited row's response surface,
//! byte for byte. Containment is computed, not assumed — a token the copier
//! emits that no cited row carries is recorded in `leaked`, and the labeler
//! demotes on it. The renderer never errors on a leak; the demotion is the
//! contract.

use si_read::{Cited, Thought};

use crate::tokenize::tokenize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rendered {
    /// The surface text, copied from one cited row or empty.
    pub surface: String,
    /// The surface's folded tokens, in word order.
    pub content: Vec<String>,
    /// Cited rows the render stood on, in rank order.
    pub citations: Vec<u32>,
    /// Content keys that appear in no cited row's surface.
    pub leaked: Vec<String>,
    /// `leaked.is_empty()`: U.2's "no novel tokens" half of INFERENCE.
    pub containment: bool,
}

/// `cited` arrives in rank order from `si-read::retrieve`. The render picks the
/// cited row whose value point carries the most activation in the settled
/// thought, ties by lower row id, and copies its surface.
///
/// 04 §21 U.1: a row carrying `NAMELESS` is **not a surface**. It stays in
/// `citations` — an un-named mint is addressable and reason-over-able — and is
/// excluded from both the print source and the token pool the containment test
/// reads. That is the existing containment law doing G15(f)/G18's no-word-leak
/// leg, not a second rule beside it.
pub fn reverse_decode(thought: &Thought, cited: &[Cited]) -> Rendered {
    let activation = |point: u32| {
        thought
            .settle_set
            .iter()
            .find(|entry| entry.point == point)
            .map(|entry| entry.activation)
            .unwrap_or(0)
    };
    let pool: Vec<&Cited> = cited.iter().filter(|row| !row.nameless).collect();
    let top = pool.iter().copied().max_by(|left, right| {
        activation(left.val_point)
            .cmp(&activation(right.val_point))
            .then(right.row_id.cmp(&left.row_id))
    });
    let Some(top) = top else {
        return Rendered {
            surface: String::new(),
            content: Vec::new(),
            citations: Vec::new(),
            leaked: Vec::new(),
            containment: true,
        };
    };

    let content: Vec<String> = tokenize(&top.surface)
        .into_iter()
        .map(|token| token.key)
        .collect();
    let mut leaked = Vec::new();
    for key in &content {
        let present = pool
            .iter()
            .any(|row| tokenize(&row.surface).iter().any(|token| &token.key == key));
        if !present && !leaked.contains(key) {
            leaked.push(key.clone());
        }
    }
    Rendered {
        surface: top.surface.clone(),
        content,
        citations: cited.iter().map(|row| row.row_id).collect(),
        containment: leaked.is_empty(),
        leaked,
    }
}

/// 02 §7: the idle monologue uses **the same face**, so an internal cascade
/// needs no second generator. `depth` is the caller's `MONOLOGUE_RENDER_DEPTH`
/// reading; `words` is the session lexicon's `(point, surface)` view, and the
/// copy is in `Thought::display` order. Output goes to the separate lane only;
/// nothing here writes or labels.
pub fn render_monologue(thought: &Thought, words: &[(u32, String)], depth: usize) -> Rendered {
    let mut picked = Vec::new();
    for entry in thought.display(depth) {
        if let Some((_, word)) = words.iter().find(|(point, _)| *point == entry.point) {
            picked.push(word.clone());
        }
    }
    let surface = picked.join(" ");
    let content = tokenize(&surface).into_iter().map(|token| token.key).collect();
    Rendered {
        surface,
        content,
        citations: Vec::new(),
        leaked: Vec::new(),
        containment: true,
    }
}
