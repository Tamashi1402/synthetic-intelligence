//! 11 §4's B20 DoD 2: the forced-failure table of 13 §6.6, run as unit tests in
//! the crate that owns the walk. Every arm here is a `None` or a canonical copy —
//! "assert the fallback, not the crash" (§27.8(c)) is the return type.
//!
//! The fold these tests hand to a `License` is the tests' own crude split, not a
//! second vocabulary law: production folds at the caller's boundary (13 §1.2), and
//! `si-tongue` itself contains no fold of any kind.

use si_core::rng::Rng;
use si_core::tongue::{render_seed, Style, STYLE_ORDER_LEN};
use si_tongue::{budget, realizes_the_slots, realize, License, Variant, JOINER};
use si_walk::Arc;
use si_walk::Graph;

const ANCHOR: u32 = 10;

fn arc(edge_off: u32, to_point: u32, kind: u8, weight: u16, bake: u16) -> (u32, Arc) {
    (
        ANCHOR,
        Arc { edge_off, to_point, kind, weight, bake, a_at: 0 },
    )
}

fn arc_from(source: u32, edge_off: u32, to_point: u32, kind: u8, weight: u16, bake: u16) -> (u32, Arc) {
    (
        source,
        Arc { edge_off, to_point, kind, weight, bake, a_at: 0 },
    )
}

fn fold(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|piece| !piece.is_empty())
        .map(|piece| piece.to_lowercase())
        .collect()
}

fn licensed(entries: &[(u32, &str)]) -> Vec<License> {
    entries
        .iter()
        .map(|(point, surface)| License::new(*point, (*surface).to_string(), fold(surface)))
        .collect()
}

fn licensed_with(entries: &[(u32, &str, &[&str])]) -> Vec<License> {
    entries
        .iter()
        .map(|(point, surface, variants)| {
            License::new(*point, (*surface).to_string(), fold(surface)).with_variants(
                variants
                    .iter()
                    .map(|variant| Variant::new((*variant).to_string(), fold(variant)))
                    .collect(),
            )
        })
        .collect()
}

fn style(spread: u16, rest: u16) -> Style {
    Style::new([spread, rest, rest, rest, rest])
}

fn draws() -> Rng {
    Rng::seeded(render_seed(7, style(4096, 1024).digest(), 3))
}

#[test]
fn a_web_with_no_arcs_prints_the_canonical_copy() {
    let graph = Graph::from_arcs([]);
    let license = licensed(&[(ANCHOR, "the cat sits")]);
    let line = realize(&graph, ANCHOR, &license, &fold("the cat sits"), &style(4096, 4096), 8, &mut draws());
    assert_eq!(line.as_deref(), Some("the cat sits"));
}

#[test]
fn an_unanchored_line_is_a_refusal_not_a_crash() {
    let graph = Graph::from_arcs([]);
    let license = licensed(&[(ANCHOR, "the cat sits")]);
    assert_eq!(realize(&graph, 999, &license, &[], &style(4096, 4096), 8, &mut draws()), None);
}

#[test]
fn an_empty_license_is_a_refusal() {
    let graph = Graph::from_arcs([arc(1, 11, 0, 2048, 0)]);
    let license: Vec<License> = Vec::new();
    assert_eq!(realize(&graph, ANCHOR, &license, &[], &style(4096, 4096), 8, &mut draws()), None);
}

#[test]
fn a_licensed_point_with_no_surface_refuses() {
    // 13 §6.6's G20(c) forced failure: the only arc leads to a point with no
    // surface, so the caller prints the canonical frame and nothing else moves.
    let graph = Graph::from_arcs([arc(1, 11, 0, 2048, 0)]);
    let license = licensed_with(&[(ANCHOR, "the cat sits", &[]), (11, "", &["on the mat"])]);
    assert_eq!(realize(&graph, ANCHOR, &license, &fold("the cat sits"), &style(4096, 4096), 8, &mut draws()), None);
}

#[test]
fn a_bank_with_no_other_spelling_spends_no_draw() {
    // **OD-189**'s bare half, mechanically: a license whose variants are the
    // canonical bytes (or nothing) must consume exactly the draws the pre-bank
    // walk consumed, or a fixture without taught variants would stop printing
    // byte-identical lines for a reason that has nothing to do with style.
    let graph = Graph::from_arcs([arc(1, 11, 0, 2048, 16), arc(2, 12, 0, 2048, 32)]);
    let slots = fold("the cat sits");
    let bare = licensed(&[(ANCHOR, "the cat sits"), (11, "the cat eats"), (12, "the cat sleeps")]);
    let duplicate: Vec<License> = bare
        .iter()
        .cloned()
        .map(|entry| {
            let copy = Variant::new(entry.surface.clone(), entry.keys.clone());
            entry.with_variants(vec![copy])
        })
        .collect();
    let seed = render_seed(7, style(4096, 1024).digest(), 3);
    let mut left = Rng::seeded(seed);
    let mut right = Rng::seeded(seed);
    let plain = realize(&graph, ANCHOR, &bare, &slots, &style(4096, 1024), 8, &mut left);
    let banked = realize(&graph, ANCHOR, &duplicate, &slots, &style(4096, 1024), 8, &mut right);
    assert_eq!(plain, banked);
    assert_eq!(left.state(), right.state(), "a canonical-only bank spent a draw");
}

#[test]
fn a_taught_variant_changes_the_bytes_and_not_the_claim() {
    // **OD-189**: a variant is another surface over the same point. The line's
    // chunk count, folded keys and slot set are the bare line's; only the bytes
    // may differ, and over enough thoughts both spellings appear.
    let graph = Graph::from_arcs([arc(1, 11, 0, 2048, 16)]);
    let slots = fold("the cat sits");
    let bare = licensed(&[(ANCHOR, "the cat sits"), (11, "on the mat")]);
    let banked = licensed_with(&[
        (ANCHOR, "the cat sits", &["The cat sits"]),
        (11, "on the mat", &["ON THE MAT"]),
    ]);
    let style = style(4096, 1024);
    let mut saw_canonical = false;
    let mut saw_variant = false;
    for thought_id in 0..64u32 {
        let seed = render_seed(7, style.digest(), thought_id);
        let plain =
            realize(&graph, ANCHOR, &bare, &slots, &style, 8, &mut Rng::seeded(seed)).unwrap();
        let variant =
            realize(&graph, ANCHOR, &banked, &slots, &style, 8, &mut Rng::seeded(seed)).unwrap();
        assert_eq!(variant.split(JOINER).count(), plain.split(JOINER).count());
        assert_eq!(fold(&variant), fold(&plain), "a variant changed the claim: {variant}");
        let keys = fold(&variant);
        assert!(realizes_the_slots(&keys, &slots, &banked), "unlicensed variant: {variant}");
        saw_canonical |= variant == plain;
        saw_variant |= variant != plain;
    }
    assert!(saw_canonical && saw_variant, "the bank did not draw both spellings");
}

#[test]
fn a_slot_that_cannot_be_uttered_is_lost_before_the_walk() {
    let graph = Graph::from_arcs([arc(1, 11, 0, 2048, 0)]);
    let license = licensed(&[(ANCHOR, "the cat sits"), (11, "on the mat")]);
    let slots = fold("the cat purrs");
    assert_eq!(realize(&graph, ANCHOR, &license, &slots, &style(4096, 4096), 8, &mut draws()), None);
}

#[test]
fn a_spread_drawn_zero_is_the_canonical_copy() {
    // 03 §5.4.1's "drawn `0` means" column, mechanically: no extra chunk.
    assert_eq!(budget(65535, &style(0, 4096)), 0);
    let graph = Graph::from_arcs([arc(1, 11, 0, 2048, 0)]);
    let license = licensed(&[(ANCHOR, "the cat sits"), (11, "on the mat")]);
    let line = realize(&graph, ANCHOR, &license, &fold("the cat sits"), &style(0, 4096), 65535, &mut draws());
    assert_eq!(line.as_deref(), Some("the cat sits"));
}

#[test]
fn the_walk_never_spends_past_the_budget() {
    // Termination side one of 04 §34 AF.3: `1 + budget` chunks, whatever the web
    // offers. A longer chain and a bigger ceiling must not lengthen the line.
    let mut chain = Vec::new();
    let mut source = ANCHOR;
    for step in 0..12u32 {
        chain.push(arc_from(source, step + 1, 100 + step, 0, 2048, 0));
        source = 100 + step;
    }
    let graph = Graph::from_arcs(chain);
    let mut entries: Vec<(u32, &str)> = vec![(ANCHOR, "alpha")];
    entries.extend((0..12u32).map(|step| (100 + step, "beta")));
    let license = licensed(&entries);
    let steps = 4usize;
    let line = realize(&graph, ANCHOR, &license, &fold("alpha"), &style(4096, 0), steps, &mut draws()).unwrap();
    assert_eq!(line.split(JOINER).count(), 1 + budget(steps, &style(4096, 0)));
    assert!(line.split(JOINER).count() <= 1 + steps, "the species bound is a ceiling, not a suggestion");
}

#[test]
fn the_same_thought_draws_the_same_line() {
    let graph = Graph::from_arcs([arc(1, 11, 0, 2048, 16), arc(2, 12, 0, 1024, 32)]);
    let license = licensed(&[(ANCHOR, "the cat sits"), (11, "the cat eats"), (12, "the cat sleeps")]);
    let slots = fold("the cat sits");
    let seed = render_seed(7, style(4096, 1024).digest(), 3);
    let first = realize(&graph, ANCHOR, &license, &slots, &style(4096, 1024), 8, &mut Rng::seeded(seed));
    let second = realize(&graph, ANCHOR, &license, &slots, &style(4096, 1024), 8, &mut Rng::seeded(seed));
    assert_eq!(first, second, "I5: same seed, same lattice, same sentence");
}

#[test]
fn a_different_draw_can_take_a_different_route() {
    // The point of the round: randomness inside the license. Over enough seeds
    // the two-arc web yields more than one line, and every one of them is
    // licensed.
    let graph = Graph::from_arcs([arc(1, 11, 0, 2048, 16), arc(2, 12, 0, 2048, 32)]);
    let license = licensed(&[(ANCHOR, "the cat sits"), (11, "the cat eats"), (12, "the cat sleeps")]);
    let slots = fold("the cat sits");
    let mut lines = std::collections::BTreeSet::new();
    for thought_id in 0..64u32 {
        let seed = render_seed(7, style(4096, 1024).digest(), thought_id);
        let line = realize(&graph, ANCHOR, &license, &slots, &style(4096, 1024), 8, &mut Rng::seeded(seed)).unwrap();
        let keys: Vec<String> = fold(&line);
        assert!(realizes_the_slots(&keys, &slots, &license), "unlicensed emission: {line}");
        lines.insert(line);
    }
    assert!(lines.len() > 1, "the voice drew one line over 64 thoughts: {lines:?}");
}

#[test]
fn a_genome_arc_is_never_walked() {
    // **OD-179** / G20(k): kind 7 is a frozen species edge, and a newborn whose
    // live arcs are empty says nothing the species text taught it.
    let graph = Graph::from_arcs([arc(1, 11, 7, 4095, 0)]);
    let license = licensed(&[(ANCHOR, "the cat sits"), (11, "water is wet")]);
    let line = realize(&graph, ANCHOR, &license, &fold("the cat sits"), &style(4096, 4096), 8, &mut draws());
    assert_eq!(line.as_deref(), Some("the cat sits"));
}

#[test]
fn a_neighbour_outside_the_license_is_not_utterable() {
    // **OD-181**: the walk may traverse further than it may speak, and here it may
    // not even traverse.
    let graph = Graph::from_arcs([arc(1, 11, 0, 4095, 0)]);
    let license = licensed(&[(ANCHOR, "the cat sits")]);
    let line = realize(&graph, ANCHOR, &license, &fold("the cat sits"), &style(4096, 4096), 8, &mut draws());
    assert_eq!(line.as_deref(), Some("the cat sits"));
}

#[test]
fn the_gate_reads_both_halves() {
    let license = licensed(&[(ANCHOR, "the cat sits"), (11, "on the mat")]);
    let slots = fold("the cat sits");
    assert!(realizes_the_slots(&fold("the cat sits, on the mat"), &slots, &license));
    // (b) a word nobody cited.
    assert!(!realizes_the_slots(&fold("the cat flies"), &slots, &license));
    // (c) a slot that fell out of the line.
    assert!(!realizes_the_slots(&fold("the cat"), &slots, &license));
}

#[test]
fn a_ceiling_depth_lean_prefers_the_claim_side() {
    // 04 §34 AF.3's `owned_share`: 4096 for a claim edge, 1024 for `6 cooccur-weak`
    // (**OD-186** — the record-shape predicate cannot be the formality predicate,
    // since its last slot *is* the co-occurrence). Arcs with no lattice weight of
    // their own leave the lean as the whole say, so the split is 2048 against 512
    // and 80% of the draws land on the argument. Deterministic: every seed is a
    // function of `thought_id`.
    let graph = Graph::from_arcs([arc(1, 11, 0, 0, 0), arc(2, 12, 6, 0, 0)]);
    let license = licensed(&[(ANCHOR, "the cat sits"), (11, "the cat eats"), (12, "the cat sleeps")]);
    let slots = fold("the cat sits");
    let style = Style::new([4096, 2048, 0, 0, 0]);
    let mut owned = 0usize;
    let total = 256usize;
    for thought_id in 0..total as u32 {
        let seed = render_seed(7, style.digest(), thought_id);
        let line = realize(&graph, ANCHOR, &license, &slots, &style, 8, &mut Rng::seeded(seed)).unwrap();
        if line.contains("eats") {
            owned += 1;
        }
    }
    assert!(
        owned * 10 > total * 7,
        "a ceiling formality lean drew the claim side {owned}/{total} times"
    );
}

#[test]
fn a_zero_axis_adds_exactly_zero() {
    // The additive shape of 13 §3.1: every lean at `0` leaves the lattice's own
    // weight as the only say, and the walk still terminates inside the budget.
    let graph = Graph::from_arcs([arc(1, 11, 0, 2048, 16), arc(2, 12, 6, 1024, 32)]);
    let license = licensed(&[(ANCHOR, "the cat sits"), (11, "the cat eats"), (12, "the cat sleeps")]);
    let slots = fold("the cat sits");
    let inert = Style::new([4096, 0, 0, 0, 0]);
    let mut lines = std::collections::BTreeSet::new();
    for thought_id in 0..32u32 {
        let seed = render_seed(7, inert.digest(), thought_id);
        let line = realize(&graph, ANCHOR, &license, &slots, &inert, 8, &mut Rng::seeded(seed)).unwrap();
        assert!(line.split(JOINER).count() <= 1 + budget(8, &inert));
        lines.insert(line);
    }
    assert_eq!(STYLE_ORDER_LEN, 5);
    assert!(!lines.is_empty());
}
