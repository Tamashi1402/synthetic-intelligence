//! 04 §12 step 5 / 06 §2: the boring repeat appends nothing and moves exactly
//! the FLUID columns. The assertion is byte-based: the rows arena's length is
//! the append count, and the record's identity bytes do not move.

mod common;

use common::{candidate, ctx, params};
use si_core::addr::SHELF_S_R_PAIRS;
use si_learn::gate::{self, Outcome};
use si_store::store::Lattice;

#[test]
fn a_boring_repeat_writes_no_new_bytes_and_moves_only_fluid_columns() {
    let mut store = Lattice::new();
    let first = candidate(&mut store, gate::PROV_OWNER_TEACH, "apple", "red", 1);
    let mut stats = gate::GateStats::default();
    let report = gate::write(&mut store, &first, &ctx(1), &params(), &mut stats).unwrap();
    let Outcome::Committed { row_id, .. } = report.outcome else {
        panic!("expected a commit");
    };
    let before = store.row_at(SHELF_S_R_PAIRS, row_id).unwrap().unwrap();
    let edge_before = store.edges_of(&before).unwrap()[0];
    let rows_bytes = store.rows_arena().len();
    let edges_bytes = store.edges_arena().len();
    let stamps_bytes = store.stamps_arena().len();

    let repeat = candidate(&mut store, gate::PROV_OWNER_TEACH, "apple", "red", 2);
    let report = gate::write(&mut store, &repeat, &ctx(2), &params(), &mut stats).unwrap();
    assert!(matches!(report.outcome, Outcome::Confirmed { .. }), "{:?}", report.outcome);
    assert_eq!(report.salience_calls, 0, "a boring repeat buys no bake");

    assert_eq!(store.rows_arena().len(), rows_bytes);
    assert_eq!(store.edges_arena().len(), edges_bytes);
    assert_eq!(store.stamps_arena().len(), stamps_bytes);

    let after = store.row_at(SHELF_S_R_PAIRS, row_id).unwrap().unwrap();
    assert_eq!(after.envelope.repeats, before.envelope.repeats + 1);
    assert_eq!(after.envelope.last_confirmed_at, 2);
    assert_eq!(after.envelope.confidence, (before.envelope.confidence + 5).min(99));
    assert_eq!(after.envelope.gain_exact, before.envelope.gain_exact);
    assert_eq!(after.envelope.gain_kind, before.envelope.gain_kind);
    assert_eq!(after.envelope.superseded_at, before.envelope.superseded_at);
    assert_eq!(after.envelope.written_at_t, before.envelope.written_at_t);
    assert_eq!(after.envelope.bind_output_id, before.envelope.bind_output_id);

    let edge_after = store.edges_of(&after).unwrap()[0];
    assert_eq!(edge_after.weight, edge_before.weight + 5);
    assert_eq!(edge_after.bake, edge_before.bake);
    let stamps = store.stamps_of(&edge_after).unwrap();
    assert_eq!(stamps.len(), 1, "confirmation adds no stamp");
}

#[test]
fn a_grimm_repeat_does_not_confirm() {
    // R-35: only the owner's repeat may move a row. The gate refuses the source
    // before the boring branch, so the row is untouched.
    let mut store = Lattice::new();
    let first = candidate(&mut store, gate::PROV_OWNER_TEACH, "apple", "red", 1);
    let mut stats = gate::GateStats::default();
    gate::write(&mut store, &first, &ctx(1), &params(), &mut stats).unwrap();
    let before = common::live(SHELF_S_R_PAIRS, &store)[0].clone();
    let repeat = candidate(&mut store, gate::PROV_OWNER_TEACH, "apple", "red", 2);
    let mut scoped = ctx(2);
    scoped.source = si_core::strings::Source::Grimm;
    let report = gate::write(&mut store, &repeat, &scoped, &params(), &mut stats).unwrap();
    assert!(matches!(report.outcome, Outcome::Inert { .. }));
    let after = common::live(SHELF_S_R_PAIRS, &store)[0].clone();
    assert_eq!(after.envelope.repeats, before.envelope.repeats);
    assert_eq!(after.envelope.confidence, before.envelope.confidence);
    assert_eq!(after.envelope.last_confirmed_at, before.envelope.last_confirmed_at);
}
