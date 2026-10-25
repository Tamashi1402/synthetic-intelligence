//! 04 §15: arbitration. Tiers first, then the marker, then insistence; the
//! affective inputs do not exist, so no test can hand one in.

mod common;

use common::candidate;
use si_core::addr::SHELF_S_R_PAIRS;
use si_learn::gate::{self, Branch, Candidate, Conflict, Verdict};
use si_store::store::Lattice;

fn conflict(prov_kind: u8, row: u32, confidence: u16, written_at: i32, prov_step: i32) -> Conflict {
    Conflict {
        shelf: SHELF_S_R_PAIRS,
        row_id: row,
        val_point: 99,
        prov_kind,
        prov_speaker: gate::SPEAKER_OWNER,
        confidence,
        written_at,
        prov_step,
    }
}

fn owner_teach() -> Candidate {
    let mut store = Lattice::new();
    candidate(&mut store, gate::PROV_OWNER_TEACH, "apple", "red", 1)
}

#[test]
fn a_tier3_correction_supersedes_anything() {
    let c = candidate(&mut Lattice::new(), gate::PROV_CORRECTION, "apple", "green", 2);
    let losers = [conflict(gate::PROV_OWNER_TEACH, 1, 95, 1, 1)];
    assert!(matches!(
        gate::arbitrate(&c, 2, &losers),
        Verdict::Supersede { branch: Branch::Tier3, .. }
    ));
}

#[test]
fn a_marked_t2_supersedes_below_tier3() {
    let mut c = candidate(&mut Lattice::new(), gate::PROV_STATEMENT, "apple", "green", 2);
    c.marker = true;
    let losers = [conflict(gate::PROV_STATEMENT, 1, 90, 1, 1)];
    assert!(matches!(
        gate::arbitrate(&c, 2, &losers),
        Verdict::Supersede { branch: Branch::MarkerCorrection, .. }
    ));
}

#[test]
fn a_bare_first_t2_asks_and_the_second_insists() {
    let mut c = candidate(&mut Lattice::new(), gate::PROV_STATEMENT, "apple", "green", 2);
    let losers = [conflict(gate::PROV_STATEMENT, 1, 90, 1, 1)];
    assert!(matches!(gate::arbitrate(&c, 2, &losers), Verdict::Ask));
    c.attempts_at_slot = 1;
    assert!(matches!(
        gate::arbitrate(&c, 2, &losers),
        Verdict::Supersede { branch: Branch::Insistence, .. }
    ));
}

#[test]
fn a_teach_loses_on_rank_to_a_tier3_row_but_never_silently() {
    let c = owner_teach();
    let losers = [conflict(gate::PROV_CORRECTION, 1, 95, 1, 1)];
    assert!(matches!(
        gate::arbitrate(&c, 2, &losers),
        Verdict::Reject { reason: "teach_below_tier3" }
    ));
}

#[test]
fn t1_loses_to_owner_rows_and_wins_by_recency_against_t1() {
    let c = candidate(&mut Lattice::new(), gate::PROV_REPARSE, "apple", "green", 2);
    let owner = [conflict(gate::PROV_STATEMENT, 1, 90, 1, 1)];
    assert!(matches!(
        gate::arbitrate(&c, 2, &owner),
        Verdict::Reject { reason: "owner_wins" }
    ));
    let older = [conflict(gate::PROV_REPARSE, 1, 55, 1, 1)];
    assert!(matches!(
        gate::arbitrate(&c, 2, &older),
        Verdict::Supersede { branch: Branch::Recency, .. }
    ));
    let newer = [conflict(gate::PROV_REPARSE, 2, 55, 3, 3)];
    assert!(matches!(
        gate::arbitrate(&c, 2, &newer),
        Verdict::Reject { reason: "recency_tie" }
    ));
}

#[test]
fn no_conflict_is_a_commit() {
    let c = owner_teach();
    assert!(matches!(gate::arbitrate(&c, 1, &[]), Verdict::Commit));
}
