//! 04 §25 steps 2/5, 10 §3–§4, G14(c): a resume decides every refusal from the
//! header before the payload is read, and the derived vector is never in the
//! file while its digest is.

use si_core::container::Container;
use si_core::hash::{CoreBlock, HASH_FORMAT};
use si_core::record::{Body, Envelope};
use si_core::seed;
use si_store::snapshot::{Inspect, ResumeRefusal, Snapshot};
use si_store::store::{EdgeDraft, Lattice};

const SEED: u64 = 0x51;

fn populated_lattice() -> Lattice {
    let mut lattice = Lattice::new();
    lattice
        .append_row(Envelope::live(4, 0, 1, 0), Body::default(), vec![EdgeDraft::owned(42, 4096, 0)], &[], &[])
        .unwrap();
    lattice
}

fn encoded(core: CoreBlock) -> Vec<u8> {
    let lattice = populated_lattice();
    let rng = seed::runtime_stream(SEED).state();
    Snapshot::capture(core, &lattice, &[], &[], rng, &[], 0).encode().unwrap()
}

#[test]
fn a_resume_under_a_foreign_seed_refuses_from_the_header_alone() {
    let mut bytes = encoded(CoreBlock::at_birth(SEED));
    // Corrupt the payload: a refusal that still arrives proves the header was
    // decided before the payload was decoded.
    let header_len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    bytes.truncate(12 + header_len + 5);

    assert!(Snapshot::decode(&bytes).is_err(), "the truncated payload does not decode");
    match Snapshot::inspect(&bytes, SEED + 1).unwrap() {
        Inspect::Refused(ResumeRefusal::SeedMismatch { stored, requested }) => {
            assert_eq!(stored, SEED);
            assert_eq!(requested, SEED + 1);
        }
        other => panic!("expected a seed refusal, got {other:?}"),
    }
    assert!(matches!(Snapshot::inspect(&bytes, SEED).unwrap(), Inspect::Accepted(_)));
}

#[test]
fn a_geometry_or_version_mismatch_refuses_before_the_payload() {
    let bytes = encoded(CoreBlock::at_birth(SEED));

    let mut container = Container::decode(&bytes).unwrap();
    container.header["geometry"]["record_bytes"] = serde_json::json!(116);
    let tampered = container.encode().unwrap();
    assert!(matches!(
        Snapshot::inspect(&tampered, SEED).unwrap(),
        Inspect::Refused(ResumeRefusal::Geometry { field: "record_bytes" })
    ));

    let mut container = Container::decode(&bytes).unwrap();
    container.version = 2;
    let tampered = container.encode().unwrap();
    assert!(matches!(
        Snapshot::inspect(&tampered, SEED).unwrap(),
        Inspect::Refused(ResumeRefusal::ContainerVersion { stored: 2 })
    ));

    let mut bad_digest = CoreBlock::at_birth(SEED);
    bad_digest.temperament_digest = 0;
    let bytes = encoded(bad_digest);
    assert!(matches!(
        Snapshot::inspect(&bytes, SEED).unwrap(),
        Inspect::Refused(ResumeRefusal::DigestMismatch { .. })
    ));
}

#[test]
fn the_core_block_is_36_bytes_and_only_13_of_them_move_after_birth() {
    let birth = CoreBlock::at_birth(SEED);
    let birth_bytes = birth.encode();
    assert_eq!(birth_bytes.len(), 36);

    let awake = CoreBlock {
        sleeps_completed: 3,
        sleeps_interrupted: 1,
        last_slept_at_t: -5,
        sleep_pass_mask: 0b1010,
        ..birth
    };
    let bytes = awake.encode();
    assert_eq!(u32::from_le_bytes(bytes[20..24].try_into().unwrap()), 3);
    assert_eq!(u32::from_le_bytes(bytes[24..28].try_into().unwrap()), 1);
    assert_eq!(i32::from_le_bytes(bytes[28..32].try_into().unwrap()), -5);
    assert_eq!(bytes[32], 0b1010);

    for offset in 0..36 {
        if (20..33).contains(&offset) {
            continue;
        }
        assert_eq!(
            bytes[offset], birth_bytes[offset],
            "byte {offset} lies outside the 13 mutable bytes (20..=32) but moved"
        );
    }
    assert_eq!(CoreBlock::decode(&bytes).unwrap(), awake);
    assert!(CoreBlock::decode(&bytes).unwrap().digest_matches_seed());
}

#[test]
fn the_snapshot_holds_the_digest_and_never_the_vector() {
    let seed_value = SEED;
    let core = CoreBlock::at_birth(seed_value);
    let bytes = encoded(core);

    let vector = seed::derive_temperament(seed_value).canonical_bytes();
    assert_eq!(vector.len(), 36);
    assert!(
        !bytes.windows(vector.len()).any(|window| window == vector),
        "the derived vector is in SIst; only its digest may persist (10 §4 P.3)"
    );
    let digest = core.temperament_digest;
    let container = Container::decode(&bytes).unwrap();
    assert_eq!(
        container.header["core_block"]["temperament_digest"],
        serde_json::json!(digest),
        "the digest is not in SIst's core block; then the vector could not be checked at wake"
    );
    assert_eq!(container.header["geometry"]["hash_format"], HASH_FORMAT);
    let header = container.header.as_object().unwrap();
    assert_eq!(header.keys().map(String::as_str).collect::<Vec<_>>(), ["core_block", "geometry"]);
    let core_block = header["core_block"].as_object().unwrap();
    assert_eq!(
        core_block.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "birth_seed",
            "block_version",
            "deriv_order_len",
            "last_slept_at_t",
            "seed_format",
            "sleep_pass_mask",
            "sleeps_completed",
            "sleeps_interrupted",
            "temperament_digest",
        ]
    );
}
