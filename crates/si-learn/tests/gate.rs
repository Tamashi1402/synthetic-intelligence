//! 04 §12: the ten-step gate's ordering laws, asserted on outcomes and rows.
//! L-A privilege, L-B teach-before-pending, L-D novelty-before-consistency,
//! L-F salience exactly once.

mod common;

use common::{candidate, ctx, live, params};
use si_core::addr::{SHELF_QUESTIONS, SHELF_S_R_PAIRS};
use si_core::strings::Source;
use si_learn::gate::{self, Outcome, QuestionSpec, WriteContext};
use si_store::store::Lattice;

#[test]
fn a_teach_short_circuits_the_pending_ask() {
    // L-B: an explicit teach is never read as a reply, and the ask it covers is
    // closed by row existence (09 §2.1).
    let mut store = Lattice::new();
    let mut c = candidate(&mut store, gate::PROV_OWNER_TEACH, "apple", "red", 3);
    let open = gate::append_question(
        &mut store,
        &QuestionSpec {
            pattern_id: c.pattern_id,
            gap_kind: 0,
            origin: gate::ASK_ORIGIN_SENSE,
            key_point: c.key_point,
            target_slot: 0,
            text_ref: c.stim_surface,
            prov_step: 2,
            arousal_at_ask: 0,
        },
        &ctx(2),
    )
    .unwrap();
    c.prov_step = 3;
    let mut stats = gate::GateStats::default();
    let report = gate::write(&mut store, &c, &ctx(3), &params(), &mut stats).unwrap();
    let Outcome::Committed { row_id, .. } = report.outcome else {
        panic!("a teach must commit, got {:?}", report.outcome);
    };
    let teach = store.row_at(SHELF_S_R_PAIRS, row_id).unwrap().unwrap();
    assert_eq!(teach.envelope.question_id, 0, "coverage closes the loop, not a match");
    assert_eq!(teach.envelope.bind_output_id, 0);
    let version = live(SHELF_QUESTIONS, &store)
        .into_iter()
        .max_by_key(|row| row.envelope.row_id)
        .unwrap();
    assert_eq!(version.envelope.row_id > open, true);
    assert_eq!(version.body.status, 1);
    assert_eq!(version.envelope.bind_output_id, row_id);
}

#[test]
fn a_grimm_line_never_reaches_the_gate() {
    // L-A / G12(d): the source is an input, and the gate refuses it before a row.
    let mut store = Lattice::new();
    let c = candidate(&mut store, gate::PROV_OWNER_TEACH, "apple", "red", 1);
    let mut scoped = ctx(1);
    scoped.source = Source::Grimm;
    let mut stats = gate::GateStats::default();
    let report = gate::write(&mut store, &c, &scoped, &params(), &mut stats).unwrap();
    assert!(matches!(report.outcome, Outcome::Inert { reason: "source_not_owner" }));
    assert!(live(SHELF_S_R_PAIRS, &store).is_empty());
    assert_eq!(stats.salience_calls, 0);
}

#[test]
fn a_commit_calls_salience_exactly_once() {
    let mut store = Lattice::new();
    let c = candidate(&mut store, gate::PROV_OWNER_TEACH, "apple", "red", 1);
    let mut stats = gate::GateStats::default();
    let report = gate::write(&mut store, &c, &ctx(1), &params(), &mut stats).unwrap();
    assert_eq!(report.salience_calls, 1);
    assert_eq!(stats.salience_calls, 1);
    assert_eq!(stats.commits, 1);
}

#[test]
fn a_teach_row_carries_the_owner_teach_shape() {
    let mut store = Lattice::new();
    let c = candidate(&mut store, gate::PROV_OWNER_TEACH, "apple", "red", 5);
    let mut stats = gate::GateStats::default();
    let report = gate::write(&mut store, &c, &ctx(5), &params(), &mut stats).unwrap();
    let Outcome::Committed { row_id, .. } = report.outcome else {
        panic!("expected a commit");
    };
    let row = store.row_at(SHELF_S_R_PAIRS, row_id).unwrap().unwrap();
    assert_eq!(row.envelope.prov_kind, gate::PROV_OWNER_TEACH);
    assert_eq!(row.envelope.prov_speaker, gate::SPEAKER_OWNER);
    assert_eq!(row.envelope.bind_output_id, 0);
    assert_eq!(row.envelope.question_id, 0);
    assert_eq!(row.envelope.confidence, 95);
    assert_eq!(row.envelope.gain_kind, gate::GAIN_ASSERTED);
    assert_eq!(row.envelope.gain_exact, 0);
    let edges = store.edges_of(&row).unwrap();
    assert_eq!(edges.iter().filter(|edge| edge.is_live()).count(), 1);
}

#[test]
fn surprise_precedes_consistency() {
    // L-D: the repeat of a conflicting candidate is not eaten as boring, because
    // the boring test only cancels on identical value *and* signature.
    let mut store = Lattice::new();
    let first = candidate(&mut store, gate::PROV_STATEMENT, "apple", "red", 1);
    let mut stats = gate::GateStats::default();
    gate::write(&mut store, &first, &ctx(1), &params(), &mut stats).unwrap();

    let red_again = candidate(&mut store, gate::PROV_STATEMENT, "apple", "red", 2);
    let report = gate::write(&mut store, &red_again, &ctx(2), &params(), &mut stats).unwrap();
    assert!(matches!(report.outcome, Outcome::Confirmed { .. }));
    assert_eq!(report.novelty.delta_q12, 0);

    let green = candidate(&mut store, gate::PROV_STATEMENT, "apple", "green", 3);
    let report = gate::write(&mut store, &green, &ctx(3), &params(), &mut stats).unwrap();
    assert!(report.novelty.delta_q12 > 0, "a new value is not a boring repeat");
}

#[test]
fn a_bare_first_statement_asks_and_writes_no_row() {
    let mut store = Lattice::new();
    let first = candidate(&mut store, gate::PROV_STATEMENT, "apple", "red", 1);
    let green = candidate(&mut store, gate::PROV_STATEMENT, "apple", "green", 2);
    let mut stats = gate::GateStats::default();
    gate::write(&mut store, &first, &ctx(1), &params(), &mut stats).unwrap();
    let report = gate::write(&mut store, &green, &ctx(2), &params(), &mut stats).unwrap();
    let Outcome::Asked { question_id } = report.outcome else {
        panic!("a bare first conflict asks, got {:?}", report.outcome);
    };
    assert_ne!(question_id, 0);
    assert_eq!(live(SHELF_S_R_PAIRS, &store).len(), 1, "the held candidate is not a row");
    assert_eq!(live(SHELF_QUESTIONS, &store).len(), 1);
}

#[test]
fn the_accessor_ctx_is_owner_and_zero_a() {
    let c = ctx(9);
    assert_eq!(c.source, Source::Owner);
    assert_eq!(c.a_formation, 0);
    let _ = WriteContext { ..c };
}
