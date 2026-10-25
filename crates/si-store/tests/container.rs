//! 00 §5 / 01 §9: the SIst container, its header, and the atomic write. A
//! staged write that never commits leaves the old file exactly as it was.

use std::fs;
use std::path::PathBuf;

use si_core::container::{Format, MAGIC_STATE};
use si_core::hash::{CoreBlock, HASH_FORMAT};
use si_core::record::{Body, Envelope, Stamp};
use si_core::rng::RngState;
use si_store::snapshot::{Snapshot, StagedWrite};
use si_store::store::{EdgeDraft, Lattice};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("si-b1-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn populated_lattice() -> Lattice {
    let mut lattice = Lattice::new();
    let stamp = Stamp { thought_id: 1, signal_ref: 0, magnitude: 4096, a_at: 128, signal: 1, src: 0 };
    let mut draft = EdgeDraft::owned(42, 4096, 7);
    draft.stamps.push(stamp);
    let mut body = Body::default();
    body.pattern_id = 3;
    body.key_point = 42;
    body.arousal_at_ask = 128;
    lattice.append_row(Envelope::live(4, 0, 1, 0), body, vec![draft], &[11, 22], &[33]).unwrap();
    lattice
}

#[test]
fn snapshot_header_carries_geometry_and_the_core_block() {
    let lattice = populated_lattice();
    let core = CoreBlock::at_birth(0x51);
    let rng = RngState { state: 7, inc: 11, spare: si_core::rng::NO_SPARE };
    let snapshot = Snapshot::capture(core, &lattice, &[(42, 4096)], &[(0, 128)], rng, &[(0, 3)], 0);
    let bytes = snapshot.encode().unwrap();

    assert_eq!(&bytes[0..4], &MAGIC_STATE, "SIst is read from the magic, not the extension");
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    assert_eq!(version, 1);
    let header_len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    assert!(12 + header_len < bytes.len(), "the header is not the whole file");

    let header = Snapshot::read_header(&bytes).unwrap();
    assert_eq!(header.core, core);
    assert_eq!(header.geometry.hash_format, HASH_FORMAT);
    assert_eq!(header.geometry.record_bytes, 112);
    assert_eq!(header.geometry.edge_bytes, 16);
    assert_eq!(header.geometry.stamp_bytes, 16);
    assert_eq!(header.geometry.core_block_bytes, 36);
    assert_eq!(header.geometry.shelf_row_schema.len(), 11);
}

#[test]
fn snapshot_round_trips_every_item_it_stores() {
    let lattice = populated_lattice();
    let core = CoreBlock::at_birth(0x51);
    let rng = RngState { state: 7, inc: 11, spare: si_core::rng::NO_SPARE };
    let snapshot = Snapshot::capture(core, &lattice, &[(42, 4096)], &[(0, 128)], rng, &[(0, 3)], 0);
    let bytes = snapshot.encode_as(Format::Checkpoint).unwrap();
    assert_eq!(&bytes[0..4], b"SIck");

    let decoded = Snapshot::decode(&bytes).unwrap();
    assert_eq!(decoded.rows.bytes(), lattice.rows_arena().bytes());
    assert_eq!(decoded.edges.bytes(), lattice.edges_arena().bytes());
    assert_eq!(decoded.stamps.bytes(), lattice.stamps_arena().bytes());
    assert_eq!(decoded.handles.bytes(), lattice.handles_arena().bytes());
    assert_eq!(decoded.interner.canonical_bytes(), lattice.interner().canonical_bytes());
    assert_eq!(decoded.settle, vec![(42, 4096)]);
    assert_eq!(decoded.trajectory, vec![(0, 128)]);
    assert_eq!(decoded.rng, rng);
    assert_eq!(decoded.advances, vec![(0, 3)]);

    let rebuilt = decoded.store().unwrap();
    assert_eq!(rebuilt.rows_arena().bytes(), lattice.rows_arena().bytes());
    assert_eq!(rebuilt.hash_inputs().unwrap().live_rows, lattice.hash_inputs().unwrap().live_rows);
}

#[test]
fn a_staged_write_that_never_commits_leaves_the_old_file() {
    let dir = scratch("atomic");
    let target = dir.join("SIst");
    fs::write(&target, b"old bytes").unwrap();

    let staged = StagedWrite::stage(&target, b"new bytes").unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"old bytes", "staging must not touch the target");
    drop(staged);
    assert_eq!(fs::read(&target).unwrap(), b"old bytes", "a failure before rename keeps the old file");
    assert!(!dir.join("SIst.tmp").exists(), "the temp file is removed on drop");

    StagedWrite::stage(&target, b"new bytes").unwrap().commit().unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"new bytes");
    assert!(!dir.join("SIst.tmp").exists());

    let _ = fs::remove_dir_all(&dir);
}
