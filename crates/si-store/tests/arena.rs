//! 06 §1 / 01 §4–§5: the arena is fixed-width, append-only, and every write
//! carries zero pads. These are the B1 DoD's storage asserts.

use si_core::record::{Body, Edge, Envelope, Record, Stamp, EDGE_BYTES, RECORD_BYTES, STAMP_BYTES};
use si_store::arena::Arena;
use si_store::interner::{fold_key, Interner};
use si_store::store::{EdgeDraft, Lattice};

#[test]
fn arena_rejects_unaligned_widths_and_misaligned_offsets() {
    assert!(std::panic::catch_unwind(|| Arena::new(0)).is_err());
    assert!(std::panic::catch_unwind(|| Arena::new(15)).is_err());
    assert!(Arena::from_bytes(16, vec![0u8; 15]).is_err());
    assert!(Arena::from_bytes(0, Vec::new()).is_err());

    let mut arena = Arena::new(16);
    assert!(arena.append(&[0u8; 15]).is_err());
    assert_eq!(arena.append(&[1u8; 16]).unwrap(), 0);
    assert_eq!(arena.append(&[2u8; 16]).unwrap(), 16);
    assert_eq!(arena.count(), 2);
    assert_eq!(arena.read(16).unwrap(), &[2u8; 16]);
    assert!(arena.read(3).is_err(), "an unaligned offset is not an address");
    assert!(arena.read(32).is_err());
    assert!(arena.write(3, &[0u8; 16]).is_err());
    assert!(arena.read_list(0, 0).is_ok());
    assert!(arena.read_list(16, 0).is_err());
    assert!(arena.read_list(16, 2).is_err(), "two records from offset 16 runs past the arena");
}

#[test]
fn every_arena_write_carries_zero_pads() {
    let mut lattice = Lattice::new();
    let stamp = Stamp { thought_id: 7, signal_ref: 3, magnitude: 2048, a_at: 51, signal: 2, src: 0 };
    let mut draft = EdgeDraft::owned(9, 4096, 0);
    draft.stamps.push(stamp);
    let row_id = lattice
        .append_row(Envelope::live(4, 0, 1, 0), Body::default(), vec![draft], &[], &[])
        .unwrap();
    assert_eq!(row_id, 1);

    let record = lattice.rows_arena().read(0).unwrap();
    assert_eq!(record.len(), RECORD_BYTES);
    assert_eq!(record[79], 0, "the envelope's trailing reserved u16 is zero");
    assert_eq!(Record::decode(record).unwrap().envelope.row_id, row_id);

    let edge_bytes = lattice.edges_arena().read(0).unwrap();
    assert_eq!(edge_bytes.len(), EDGE_BYTES);
    assert_eq!(&edge_bytes[10..12], &[0, 0], "the edge pad after bake is zero");
    let edge = Edge::decode(edge_bytes).unwrap();
    assert_eq!(edge.bake, 0);
    assert_eq!(
        edge.charge_off, STAMP_BYTES as u32,
        "slot 0 is reserved for `charge_off = 0`'s \"no stamps\", so the first real list starts at 16"
    );
    assert_eq!(lattice.stamps_arena().read(0).unwrap(), &[0u8; STAMP_BYTES], "the reserved slot is zero");

    let stamp_bytes = lattice.stamps_arena().read(edge.charge_off).unwrap();
    assert_eq!(&stamp_bytes[13..16], &[0, 0, 0], "the stamp pads are zero");
    assert_eq!(Stamp::decode(stamp_bytes).unwrap(), stamp);
}

#[test]
fn in_place_write_keeps_the_width_and_the_address() {
    let mut arena = Arena::new(8);
    let at = arena.append(&[1u8; 8]).unwrap();
    arena.write(at, &[9u8; 8]).unwrap();
    assert_eq!(arena.read(at).unwrap(), &[9u8; 8]);
    assert_eq!(arena.count(), 1);
}

#[test]
fn interner_is_per_session_and_never_merges_surfaces() {
    let mut interner = Interner::new();
    assert_eq!(interner.get(0).unwrap().verbatim, "");
    let a = interner.intern("Cat");
    let b = interner.intern("cat");
    assert_ne!(a, b, "verbatim identity: two surfaces are two ids");
    assert_eq!(interner.folded(a), interner.folded(b), "the alias set folds together");
    assert_eq!(interner.lookup("CAT").len(), 2);

    let surfaces: Vec<String> = interner.surfaces().map(|entry| entry.verbatim.clone()).collect();
    let round = Interner::from_surfaces(surfaces).unwrap();
    assert_eq!(round.canonical_bytes(), interner.canonical_bytes());
    assert!(Interner::from_surfaces(vec!["not-empty-first".into()]).is_err());
}

#[test]
fn fold_key_collapses_case_and_plurals() {
    assert_eq!(fold_key("Cats"), "cat");
    assert_eq!(fold_key("boxes"), "box");
    assert_eq!(fold_key("stories"), "story");
    assert_eq!(fold_key("  two   Words "), "two word");
    // docs/03 R2 strips one `s` suffix per token in a single pass. 04 §3 B.2's
    // example says `clas` and `class` collide on one folded key; one pass cannot
    // produce that (`class` -> `clas`, `clas` -> `cla`). The code follows the
    // rule; the clash is an open item in the B4 report.
    assert_eq!(fold_key("class"), "clas");
    assert_eq!(fold_key("clas"), "cla");

    let mut interner = Interner::new();
    let long = interner.intern("class");
    let short = interner.intern("clas");
    assert_ne!(long, short, "verbatim identity is untouched by the fold");
    assert_eq!(interner.lookup("class"), vec![long], "only `class` folds to `clas`");
    assert_eq!(interner.lookup("clas"), vec![short]);
}
