//! 09 §6 / 04 §17: the gain ledger. Promotion is one-way, happens only when a
//! serve stands on the row, and a displacing correction takes one unit back.

mod common;

use common::{candidate, ctx, live, params};
use si_core::addr::SHELF_S_R_PAIRS;
use si_learn::gate::{self, Outcome};
use si_store::store::Lattice;

fn asserted_row(store: &mut Lattice) -> u32 {
    let c = candidate(store, gate::PROV_OWNER_TEACH, "apple", "red", 1);
    let mut stats = gate::GateStats::default();
    let report = gate::write(store, &c, &ctx(1), &params(), &mut stats).unwrap();
    let Outcome::Committed { row_id, .. } = report.outcome else {
        panic!("expected a commit");
    };
    row_id
}

#[test]
fn the_first_serve_promotes_an_asserted_row_once() {
    let mut store = Lattice::new();
    let row = asserted_row(&mut store);
    let mut stats = gate::GateStats::default();
    let served = gate::serve(&mut store, SHELF_S_R_PAIRS, row, 3, &mut stats).unwrap();
    assert!(served.promoted);
    assert_eq!(served.gain_exact, 1);
    assert_eq!(served.repeats, 2);
    assert_eq!(served.confidence, 99, "95 + 5, capped by CONF_MAX");
    let record = store.row_at(SHELF_S_R_PAIRS, row).unwrap().unwrap();
    assert_eq!(record.envelope.gain_kind, gate::GAIN_DEMONSTRATED);

    let again = gate::serve(&mut store, SHELF_S_R_PAIRS, row, 4, &mut stats).unwrap();
    assert!(!again.promoted, "promotion is one-way and once");
    assert_eq!(again.gain_exact, 2);
    assert_eq!(again.repeats, 3);
    assert_eq!(again.confidence, 99);
    assert_eq!(stats.serves, 2);
}

#[test]
fn a_superseded_row_cannot_be_served() {
    let mut store = Lattice::new();
    let row = asserted_row(&mut store);
    let correction = candidate(&mut store, gate::PROV_CORRECTION, "apple", "blue", 2);
    let mut stats = gate::GateStats::default();
    let report = gate::write(&mut store, &correction, &ctx(2), &params(), &mut stats).unwrap();
    assert!(matches!(report.outcome, Outcome::Committed { .. }));
    let served = gate::serve(&mut store, SHELF_S_R_PAIRS, row, 3, &mut stats);
    assert!(served.is_err(), "a serve cannot cite a dead row");
}

#[test]
fn a_tier3_supersession_takes_one_gain_unit_back() {
    let mut store = Lattice::new();
    // Answer-to-ask rows are demonstrated at birth (09 §6's table).
    let c = candidate(&mut store, gate::PROV_ANSWER_TO_ASK, "apple", "red", 1);
    let mut stats = gate::GateStats::default();
    let report = gate::write(&mut store, &c, &ctx(1), &params(), &mut stats).unwrap();
    let Outcome::Committed { row_id, .. } = report.outcome else {
        panic!("expected a commit");
    };
    gate::serve(&mut store, SHELF_S_R_PAIRS, row_id, 2, &mut stats).unwrap();
    let before = store.row_at(SHELF_S_R_PAIRS, row_id).unwrap().unwrap().envelope.gain_exact;
    let correction = candidate(&mut store, gate::PROV_CORRECTION, "apple", "blue", 3);
    let report = gate::write(&mut store, &correction, &ctx(3), &params(), &mut stats).unwrap();
    assert!(matches!(report.outcome, Outcome::Committed { .. }));
    let after = store.row_at(SHELF_S_R_PAIRS, row_id).unwrap().unwrap();
    assert!(!after.envelope.is_live());
    assert_eq!(after.envelope.gain_exact, before - 1);
    assert_eq!(live(SHELF_S_R_PAIRS, &store).len(), 1);
}
