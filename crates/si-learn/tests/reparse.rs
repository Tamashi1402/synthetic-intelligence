//! 04 §18: RE-PARSE. Eligibility is a parse of the row's own surfaces, the
//! derived row carries the parent's slot identity and no fresh charge, and a
//! second pass adds nothing.

mod common;

use common::{candidate, ctx, live, params};
use si_core::addr::{SHELF_FACTS, SHELF_S_R_PAIRS};
use si_learn::gate::{self, Outcome};
use si_learn::reparse::{self, DerivedSlot, Parser, PassOutcome};
use si_store::store::Lattice;

/// A parser over the fixture's own convention: the stimulus is
/// `what color is <key>`, the response starts with the value.
struct FixtureParser;

impl Parser for FixtureParser {
    fn slot_of(&self, stimulus: &str, response: &str) -> Option<DerivedSlot> {
        let key = stimulus.split_whitespace().last()?.to_string();
        let value = response.split_whitespace().next()?.to_string();
        Some(DerivedSlot {
            pattern_id: si_core::patterns::PatternId::PQuestA.index() as u16,
            key_field: gate::Candidate::key_field_for(
                si_core::patterns::PatternId::PQuestA,
                0,
            ),
            key,
            value,
            target_slot: 0,
        })
    }
}

fn write_sr(store: &mut Lattice, key: &str, value: &str, t: i32) -> u32 {
    let c = candidate(store, gate::PROV_OWNER_TEACH, key, value, t);
    let mut stats = gate::GateStats::default();
    let report = gate::write(store, &c, &ctx(t), &params(), &mut stats).unwrap();
    match report.outcome {
        Outcome::Committed { row_id, .. } => row_id,
        other => panic!("expected a commit, got {other:?}"),
    }
}

#[test]
fn a_pass_derives_one_fact_per_eligible_parent() {
    let mut store = Lattice::new();
    let apple = write_sr(&mut store, "apple", "red", 1);
    let pear = write_sr(&mut store, "pear", "green", 2);
    let mut stats = gate::GateStats::default();
    let outcomes = reparse::pass(&mut store, &FixtureParser, &ctx(3), &params(), &mut stats).unwrap();
    assert_eq!(outcomes.len(), 2);
    for outcome in &outcomes {
        assert!(matches!(outcome, PassOutcome::Derived { .. }), "{outcome:?}");
    }
    let facts = live(SHELF_FACTS, &store);
    assert_eq!(facts.len(), 2);
    for fact in &facts {
        assert_eq!(fact.envelope.prov_kind, gate::PROV_REPARSE);
        assert_eq!(fact.envelope.prov_speaker, gate::SPEAKER_SYSTEM_INFERRED);
        assert_eq!(fact.envelope.confidence, 55);
        assert_eq!(fact.envelope.gain_kind, gate::GAIN_ASSERTED);
        assert_eq!(fact.envelope.gain_exact, 0);
        assert_eq!(fact.envelope.derived_len, 1);
        let parent = store
            .handles_at(fact.envelope.derived_off, fact.envelope.derived_len)
            .unwrap()[0];
        assert!(parent == apple || parent == pear);
        let edges = store.edges_of(fact).unwrap();
        assert!(edges.iter().all(|edge| edge.bake == 0));
        assert!(edges.iter().flat_map(|edge| store.stamps_of(edge).unwrap()).all(|stamp| stamp.src == 3));
        assert!(edges.iter().flat_map(|edge| store.stamps_of(edge).unwrap()).count() > 0);
    }
    assert_eq!(stats.salience_calls, 0, "a derivation never buys a boost");
    let parents: Vec<_> = live(SHELF_S_R_PAIRS, &store).into_iter().map(|row| row.envelope.row_id).collect();
    assert_eq!(parents.len(), 2, "the original rows are untouched");
}

#[test]
fn a_second_pass_is_idle() {
    let mut store = Lattice::new();
    write_sr(&mut store, "apple", "red", 1);
    let mut stats = gate::GateStats::default();
    reparse::pass(&mut store, &FixtureParser, &ctx(2), &params(), &mut stats).unwrap();
    let again = reparse::pass(&mut store, &FixtureParser, &ctx(3), &params(), &mut stats).unwrap();
    assert!(again.is_empty());
    assert_eq!(live(SHELF_FACTS, &store).len(), 1);
}

#[test]
fn an_unparseable_row_is_skipped_without_a_row() {
    let mut store = Lattice::new();
    // A context row on the S-R shelf with an unrelated stimulus: the fixture
    // parser names "q" as the key, which is not the row's key point.
    write_sr(&mut store, "apple", "red", 1);
    struct NoneParser;
    impl Parser for NoneParser {
        fn slot_of(&self, _: &str, _: &str) -> Option<DerivedSlot> {
            None
        }
    }
    let mut stats = gate::GateStats::default();
    let outcomes = reparse::pass(&mut store, &NoneParser, &ctx(2), &params(), &mut stats).unwrap();
    assert!(outcomes.is_empty());
    assert!(live(SHELF_FACTS, &store).is_empty());
}
