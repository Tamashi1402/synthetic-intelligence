//! 04 §25 W.0, 01 §10, 10 §10: wake derives the temperament, the empty state
//! hashes to the pinned B0 value, and a save/resume pair keeps the hash.

use std::fs;
use std::path::PathBuf;

use si_core::hash::{HashInputs, StateHash, EMPTY_STATE_MD5_SEED_ZERO};
use si_core::seed;
use si_core::temperament::DERIV_ORDER;
use si_session::{OpenOutcome, Session};
use si_store::snapshot::ResumeRefusal;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("si-b1-session-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_empty_state_hash_is_the_pinned_value() {
    let session = Session::new(0).unwrap();
    assert_eq!(session.state_hash().unwrap().hex(), EMPTY_STATE_MD5_SEED_ZERO);

    for seed_value in [0u64, 1, 0x51, u64::MAX] {
        let session = Session::new(seed_value).unwrap();
        let expected = StateHash::compose(&HashInputs::empty_at_birth(seed_value));
        assert_eq!(
            session.state_hash().unwrap().digest(),
            expected.digest(),
            "seed {seed_value:#x} does not compose to the empty-state value"
        );
    }
}

#[test]
fn save_and_resume_keep_the_state_hash() {
    let dir = scratch("resume");
    let path = dir.join("SIst");
    let session = Session::new(0x51).unwrap();
    let before = session.state_hash().unwrap();
    session.save(&path).unwrap();
    assert!(path.exists());

    match Session::open(&path, 0x51).unwrap() {
        OpenOutcome::Opened(resumed) => {
            assert_eq!(resumed.seed(), 0x51);
            assert_eq!(resumed.state_hash().unwrap().digest(), before.digest());
            assert_eq!(resumed.fingerprint(), session.fingerprint());
            assert_eq!(resumed.core().temperament_digest, session.core().temperament_digest);
        }
        OpenOutcome::Refused(refusal) => panic!("a same-seed resume refused: {refusal}"),
    }

    match Session::open(&path, 0x52).unwrap() {
        OpenOutcome::Refused(ResumeRefusal::SeedMismatch { stored, requested }) => {
            assert_eq!(stored, 0x51);
            assert_eq!(requested, 0x52);
        }
        OpenOutcome::Opened(_) => panic!("another individual's session was resumed"),
        OpenOutcome::Refused(other) => panic!("expected a seed refusal, got {other}"),
    }

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_fingerprint_is_a_table_with_a_checkable_digest() {
    let session = Session::new(0x51).unwrap();
    let text = session.fingerprint();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3 + DERIV_ORDER.len() + 1, "header, 18 rows, digest");
    assert!(lines[0].starts_with("birth_seed 0x"));
    assert!(lines[1].starts_with("seed_format 0x"));
    assert!(lines[2].starts_with("deriv_order_len "));

    let digest = format!("temperament_digest 0x{:016x}", session.core().temperament_digest);
    assert!(text.contains(&digest));
    for id in DERIV_ORDER {
        let name = id.registry_name();
        assert!(text.contains(name), "{name} is missing from the fingerprint");
        let value = format!("{name} {} [", session.temperament().get(id));
        assert!(text.contains(&value), "{name} has no grid value or box");
    }
    for nickname in ["neophilic", "cautious", "restless"] {
        assert!(!text.contains(nickname), "the fingerprint carries a nickname: {nickname}");
    }
}

#[test]
fn the_wake_only_mindlog_has_no_invented_thought() {
    let session = Session::new(0x51).unwrap();
    let json = session.mindlog_json("b1-smoke").unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["run_id"], "b1-smoke");
    assert_eq!(value["birth_seed"], "0x0000000000000051");
    assert_eq!(value["seed_format"], "0x0001");
    // **08 §2.1 row 20** puts one frame at wake, before the first round of the
    // first thought, so "nothing at all" is no longer the wake-only mindlog. The
    // assertion that carries the test's name is the one below it: the lane holds
    // that readout and no thought-bearing frame — a walk, a settle, a commit.
    let thoughts = value["thoughts"].as_array().unwrap();
    assert_eq!(thoughts.len(), 1, "wake emits the fingerprint frame and nothing else");
    let first = &thoughts[0];
    assert_eq!(first["kind"].as_u64().unwrap(), 20);
    assert_eq!(first["t"].as_i64().unwrap(), 0);
    let digest = format!("0x{:016x}", session.core().temperament_digest);
    assert_eq!(value["temperament_digest"], digest);
    assert!(
        first["body"]
            .as_str()
            .unwrap()
            .contains(&format!("\"temperament_digest\":\"{digest}\"")),
        "the frame and the mindlog header disagree about the digest"
    );
}

#[test]
fn wake_derives_before_the_first_thought() {
    for seed_value in [0u64, 7, 0x51] {
        let session = Session::new(seed_value).unwrap();
        assert_eq!(
            session.core().temperament_digest,
            seed::temperament_digest(seed_value),
            "the core block's digest is not the derived-vector digest"
        );
        assert!(session.core().digest_matches_seed());
        assert_eq!(session.core().sleeps_completed, 0);
        assert_eq!(session.core().sleeps_interrupted, 0);
        assert_eq!(session.core().sleep_pass_mask, 0);
        assert_eq!(session.core().last_slept_at_t, 0);
    }
}
