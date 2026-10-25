//! 04 §31 AC.1: the derived index, its one join key, and the grimm absence.
//!
//! A tug is built from two rows that already exist, so the fixture writes those
//! rows the way the write path does and asks the index to read them.

use si_core::addr::{SHELF_CONTEXT, SHELF_S_R_PAIRS};
use si_core::record::{Body, Envelope};
use si_core::quant::mul_q12_u16;
use si_store::store::{EdgeDraft, Lattice};

use si_read::{event_view_index, event_views};

const A: u32 = 11;
const B: u32 = 22;

fn owner_line(lattice: &mut Lattice, t: i32, speaker: u8) -> u32 {
    let mut envelope = Envelope::live(SHELF_CONTEXT, t, 1, t);
    envelope.prov_speaker = speaker;
    lattice
        .append_row(envelope, Body::default(), Vec::new(), &[], &[])
        .unwrap()
}

fn belief(lattice: &mut Lattice, t: i32, key: u32, val: u32, weight: u16) -> u32 {
    let mut body = Body::default();
    body.pattern_id = 1;
    body.key_point = key;
    body.val_point = val;
    let mut draft = EdgeDraft::owned(val, weight, 0);
    draft.stamps.push(si_core::record::Stamp {
        thought_id: 1,
        signal_ref: t as u32,
        magnitude: 4096,
        a_at: 128,
        signal: 1,
        src: 0,
    });
    lattice
        .append_row(
            Envelope::live(SHELF_S_R_PAIRS, t, 1, t),
            body,
            vec![draft],
            &[],
            &[],
        )
        .unwrap()
}

#[test]
fn the_index_is_one_tug_per_live_belief_and_the_event_id_is_the_context_y() {
    let mut lattice = Lattice::new();
    let event = owner_line(&mut lattice, 0, 0);
    let row = belief(&mut lattice, 0, A, B, 4096);
    let tugs = event_view_index(&lattice, 0, 1).unwrap();
    assert_eq!(tugs.len(), 1);
    assert_eq!(tugs[0].event_id, event);
    assert_eq!(tugs[0].points, vec![A, B]);
    // level · tug = 4096 · 1, the floor: a tiny nonzero light, on the grid.
    assert_eq!(tugs[0].weight_q12, mul_q12_u16(1, 4096));

    // Off is the empty vector, and nothing else about the lattice changes.
    assert!(event_view_index(&lattice, 0, 0).unwrap().is_empty());
    let _ = row;
}

#[test]
fn a_grimm_line_has_an_event_view_and_no_incoming_belief_link() {
    let mut lattice = Lattice::new();
    let owner = owner_line(&mut lattice, 0, 0);
    let grimm = owner_line(&mut lattice, 1, 1);
    let owner_row = belief(&mut lattice, 0, A, B, 4096);
    // A belief whose handle names the grimm line. His line forms none, so this
    // row is not reachable as an origin: his moment is addressable scenery.
    belief(&mut lattice, 1, B, A, 4096);

    let tugs = event_view_index(&lattice, 0, 1).unwrap();
    assert_eq!(tugs.len(), 1);
    assert_eq!(tugs[0].event_id, owner);

    let views = event_views(&lattice, 1).unwrap();
    assert_eq!(views.len(), 2);
    let grimm_view = views.iter().find(|view| view.event_id == grimm).unwrap();
    assert!(grimm_view.belief_rows.is_empty());
    assert_eq!(grimm_view.t, 1);
    let owner_view = views.iter().find(|view| view.event_id == owner).unwrap();
    assert_eq!(owner_view.belief_rows, vec![owner_row]);
}

#[test]
fn an_owner_belief_view_carries_its_rows_and_the_stamps_the_commit_laid() {
    let mut lattice = Lattice::new();
    let event = owner_line(&mut lattice, 0, 0);
    let row = belief(&mut lattice, 0, A, B, 4096);
    let views = event_views(&lattice, 0).unwrap();
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].event_id, event);
    assert_eq!(views[0].belief_rows, vec![row]);
    assert_eq!(views[0].stamps.len(), 1);
    assert_eq!(views[0].a_at, 128);
    assert_eq!(views[0].t, 0);
    assert_eq!(views[0].speaker_identity, 0);
}
