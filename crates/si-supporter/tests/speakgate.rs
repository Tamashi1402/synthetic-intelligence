//! 04 §23 / 11 §4 B9 DoD 1: the two layers, asserted as behavior on a store the
//! crate only ever reads. No scenario constant here may carry a gate verdict
//! (11 §1 D5), so every assertion below is about order and existence, not size.

use si_core::addr::SHELF_S_R_PAIRS;
use si_core::record::{Body, Envelope};
use si_store::store::Lattice;
use si_supporter::speakgate::{self, REASON_NOVEL_CLAIM, REASON_UNTAUGHT};

/// The owner's teach, as the store records it: the line interned verbatim, its
/// folded keys entered as terms, one shelf-4 row with `prov_speaker = owner`.
fn teach(store: &mut Lattice, line: &str, t: i32) -> u32 {
    let text_ref = store.interner_mut().intern(line);
    let keys = speakgate::token_keys(line);
    let terms: Vec<u32> = keys
        .iter()
        .map(|key| store.points_mut().ensure_term(key, key))
        .collect();
    let mut envelope = Envelope::live(SHELF_S_R_PAIRS, t, 1, t);
    envelope.text_ref = text_ref;
    store.append_row(envelope, Body::default(), Vec::new(), &[], &terms).unwrap()
}

fn empty() -> Lattice {
    Lattice::new()
}

#[test]
fn at_birth_every_emission_is_rejected_at_layer_one() {
    let store = empty();
    for line in [
        "apple is red.",
        "hello.",
        "red.",
        "apples are good because vitamins x and y",
    ] {
        let verdict = speakgate::check(&store, line).expect("read-only gate");
        assert!(verdict.rejected(), "{line} passed a filter with an empty lexicon");
        assert_eq!(verdict.reason, REASON_UNTAUGHT, "{line}");
        assert_eq!(verdict.layer, 1, "{line}");
        assert_eq!(
            verdict.n_untaught_tokens,
            speakgate::token_keys(line).len(),
            "{line}: every token of an empty lexicon is untaught"
        );
        assert_eq!(verdict.ref_row_id, None, "{line}");
    }
}

#[test]
fn a_token_outside_the_taught_lexicon_is_rejected_before_decode() {
    let mut store = empty();
    teach(&mut store, "apple is red.", 0);
    let verdict = speakgate::check(&store, "apple is blork.").unwrap();
    assert!(verdict.rejected());
    assert_eq!(verdict.reason, REASON_UNTAUGHT);
    assert_eq!(verdict.n_untaught_tokens, 1);
    assert_eq!(verdict.untaught, vec!["blork".to_string()]);
}

#[test]
fn the_canonical_failure_passes_layer_one_and_dies_at_layer_two() {
    let mut store = empty();
    // The drift's every token is taught by some row, so only the claim test can
    // catch it — which is exactly the case docs/21 §21.3 ships as a test.
    teach(&mut store, "apples are good.", 0);
    teach(&mut store, "because is a word.", 1);
    teach(&mut store, "vitamins are x.", 2);
    teach(&mut store, "y and x are red.", 3);

    let verdict = speakgate::check(&store, "apples are good because vitamins x and y").unwrap();
    assert!(verdict.rejected(), "the drifted claim was admitted");
    assert_eq!(verdict.reason, REASON_NOVEL_CLAIM);
    assert_eq!(verdict.layer, 2);
    assert_eq!(verdict.n_untaught_tokens, 0, "layer 1 is the one that must have passed");
    assert_eq!(verdict.novel_claims, 1);
    assert_eq!(verdict.ref_row_id, None, "a rejection names no row");
}

#[test]
fn a_restatement_of_one_row_is_admitted_and_names_it() {
    let mut store = empty();
    let row_id = teach(&mut store, "apple is red.", 0);
    let verdict = speakgate::check(&store, "apple is red.").unwrap();
    assert!(verdict.admitted);
    assert_eq!(verdict.ref_row_id, Some(row_id));
    assert_eq!(verdict.novel_claims, 0);

    // An anchor word restates the row that names it (docs/21 §21.2's mirror).
    let anchor = speakgate::check(&store, "red.").unwrap();
    assert!(anchor.admitted);
    assert_eq!(anchor.ref_row_id, Some(row_id));
}

#[test]
fn a_recombination_of_taught_words_into_a_new_order_is_a_novel_claim() {
    let mut store = empty();
    teach(&mut store, "apple is red.", 0);
    let verdict = speakgate::check(&store, "red is apple.").unwrap();
    assert!(verdict.rejected(), "word salad became a claim");
    assert_eq!(verdict.reason, REASON_NOVEL_CLAIM);
}

#[test]
fn the_gate_writes_nothing_even_when_it_admits() {
    let mut store = empty();
    teach(&mut store, "apple is red.", 0);
    let rows = store.rows_arena().bytes().len();
    let edges = store.edges_arena().bytes().len();
    let stamps = store.stamps_arena().bytes().len();
    let handles = store.handles_arena().bytes().len();
    let interned = store.interner().len();
    let points = store.points().len();

    for line in ["apple is red.", "apple is blork.", "apples are good because x"] {
        let _ = speakgate::check(&store, line).unwrap();
    }

    assert_eq!(store.rows_arena().bytes().len(), rows);
    assert_eq!(store.edges_arena().bytes().len(), edges);
    assert_eq!(store.stamps_arena().bytes().len(), stamps);
    assert_eq!(store.handles_arena().bytes().len(), handles);
    assert_eq!(store.interner().len(), interned, "a rejected token must not be interned");
    assert_eq!(store.points().len(), points, "the gate owns no arrival");
}

#[test]
fn a_superseded_row_still_teaches_but_no_longer_states_a_claim() {
    let mut store = empty();
    let first = teach(&mut store, "apple is red.", 0);
    store.age(SHELF_S_R_PAIRS, first, 1, first).unwrap();

    // "apple"/"red" remain taught: a word the owner spoke does not leave the
    // lexicon when its row ages out. The rejection below is layer 2's, not
    // layer 1's — that difference is the whole test.
    let anchor = speakgate::check(&store, "apple.").unwrap();
    assert_eq!(anchor.reason, REASON_NOVEL_CLAIM, "an aged row un-taught its words");
    assert_eq!(anchor.n_untaught_tokens, 0);

    // The aged row is no longer a live claim, so restating it is a novel claim.
    let verdict = speakgate::check(&store, "apple is red.").unwrap();
    assert_eq!(verdict.reason, REASON_NOVEL_CLAIM);
}

#[test]
fn a_grimm_row_never_teaches_anything() {
    let mut store = empty();
    let text_ref = store.interner_mut().intern("zhorx is flimp.");
    let mut envelope = Envelope::live(SHELF_S_R_PAIRS, 0, 1, 0);
    envelope.prov_speaker = 1;
    envelope.text_ref = text_ref;
    store.append_row(envelope, Body::default(), Vec::new(), &[], &[]).unwrap();

    let verdict = speakgate::check(&store, "zhorx is flimp.").unwrap();
    assert_eq!(verdict.reason, REASON_UNTAUGHT, "a grimm row taught its own tokens");
}

#[test]
fn a_grimm_line_that_looks_like_a_command_is_still_just_a_line() {
    let mut store = empty();
    teach(&mut store, "apple is red.", 0);
    // The shape test is the front-end's and it is owner-gated; the mouth filter
    // never sees a marker, so these are only token sequences here.
    for line in ["/teach what is apple => red", "/correct Red", "apple is red."] {
        let verdict = speakgate::check(&store, line).unwrap();
        if line == "apple is red." {
            assert!(verdict.admitted);
        } else {
            assert!(verdict.rejected(), "{line} was admitted by content it does not restate");
        }
    }
}
