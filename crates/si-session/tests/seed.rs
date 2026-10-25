//! 10 §3–§4, G14(b)(c)(j): the seed is the whole individual. Two seeds differ
//! in the core block and the derived readout, nowhere else — and seed 0 is a
//! legal individual, not a null.

use si_core::hash::EMPTY_STATE_MD5_SEED_ZERO;
use si_session::Session;

#[test]
fn seed_zero_is_an_individual() {
    let session = Session::new(0).unwrap();
    assert_eq!(session.core().birth_seed, 0);
    assert_eq!(session.state_hash().unwrap().hex(), EMPTY_STATE_MD5_SEED_ZERO);
    assert!(session.core().digest_matches_seed());
}

#[test]
fn two_seeds_differ_only_in_the_core_blocks_identity_bytes() {
    let a = Session::new(0).unwrap();
    let b = Session::new(1).unwrap();
    assert_ne!(a.core().temperament_digest, b.core().temperament_digest);
    assert_ne!(a.state_hash().unwrap().hex(), b.state_hash().unwrap().hex());
    assert_ne!(a.fingerprint(), b.fingerprint());

    let first = a.core().encode();
    let second = b.core().encode();
    let differing: Vec<usize> = (0..36).filter(|offset| first[*offset] != second[*offset]).collect();
    assert!(!differing.is_empty());
    assert!(
        differing.iter().all(|offset| *offset < 20),
        "at birth only the seed (0..8) and digest (12..20) bytes may differ, got {differing:?}"
    );
}

#[test]
fn the_same_seed_is_the_same_individual_twice() {
    let a = Session::new(0x51).unwrap();
    let b = Session::new(0x51).unwrap();
    assert_eq!(a.core().encode(), b.core().encode());
    assert_eq!(a.state_hash().unwrap().hex(), b.state_hash().unwrap().hex());
    assert_eq!(a.fingerprint(), b.fingerprint());
    assert_eq!(a.mindlog_json("x").unwrap(), b.mindlog_json("x").unwrap());
}
