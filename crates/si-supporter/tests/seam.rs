//! 02 §8's error behavior and 00 §2's missing edge, asserted from the crate's own
//! manifest and from the seam's three launch states.

use std::io::BufReader;

use si_supporter::speakgate::REASON_UNTAUGHT;
use si_supporter::{Admission, GateLog, Seam, SeamState};

/// A store with nothing in it is the birth state; the seam never writes, so one
/// store serves every leg here.
fn birth() -> si_store::store::Lattice {
    si_store::store::Lattice::new()
}

#[test]
fn the_manifest_lists_no_control_crate() {
    let manifest = include_str!("../Cargo.toml");
    let block = manifest
        .split("[dependencies]")
        .nth(1)
        .expect("the manifest has a dependency block");
    let deps: Vec<String> = block
        .lines()
        .map(|line| line.split('=').next().unwrap_or("").trim().to_string())
        .filter(|name| !name.is_empty())
        .map(|name| name.split('.').next().unwrap_or("").to_string())
        .collect();
    for forbidden in ["si-affect", "si-walk", "si-learn", "si-read", "si-front", "si-session", "grimm"] {
        assert!(
            !deps.iter().any(|name| name == forbidden),
            "si-supporter depends on {forbidden}; the membrane is rule (vi)"
        );
    }
    assert!(deps.iter().any(|name| name == "si-store"), "the filter has no lexicon to test");
}

#[test]
fn a_closed_seam_drops_the_attempt_before_the_gate_runs() {
    let store = birth();
    let mut seam = Seam::new(SeamState::Closed, GateLog::new());
    assert_eq!(seam.speak(&store, "apple is red.").unwrap(), Admission::Dropped);
    assert_eq!(seam.log.attempts.len(), 1);
    assert!(seam.log.attempts[0].dropped);
    assert_eq!(seam.log.attempts[0].verdict, None, "a dropped attempt reached the gate");
}

#[test]
fn a_silent_seam_reads_and_never_emits() {
    let store = birth();
    let mut seam = Seam::new(SeamState::Silent, GateLog::new());
    seam.read(&store, "read_vocab").expect("the lexicon is readable at birth");
    seam.read(&store, "read_rows").expect("the rows are readable at birth");
    assert_eq!(seam.reads.len(), 2, "OD-40(iii): every read is logged");
    assert_eq!(seam.speak(&store, "apple is red.").unwrap(), Admission::Dropped);
    assert!(seam.log.attempts[0].dropped);
}

#[test]
fn an_open_seam_rejects_with_the_counts_he_rephrases_from() {
    let store = birth();
    let mut seam = Seam::new(SeamState::Open, GateLog::new());
    match seam.speak(&store, "apple is red.").unwrap() {
        Admission::Rejected(verdict) => {
            assert_eq!(verdict.reason, REASON_UNTAUGHT);
            assert!(verdict.n_untaught_tokens > 0);
        }
        other => panic!("an open seam admitted at birth: {other:?}"),
    }
}

#[test]
fn the_gate_log_is_one_line_per_attempt_and_nothing_reads_it_back() {
    let dir = std::env::temp_dir().join(format!(
        "si-b9-seam-{}-log",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("speak_gate_log.jsonl");
    let store = birth();
    let mut seam = Seam::new(SeamState::Open, GateLog::at(&path));
    seam.speak(&store, "apple is red.").unwrap();
    seam.speak(&store, "red.").unwrap();
    seam.flush().unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    assert_eq!(text.lines().count(), 2, "one line per attempt");
    // The shape is `G12(b).1`'s machine-readable reason, asserted by field name so
    // the crate needs no serializer to prove it (docs/21 §21.3: stdlib and auditable).
    for line in text.lines() {
        for field in [
            "\"admitted\":",
            "\"reason\":",
            "\"n_untaught_tokens\":",
            "\"novel_claims\":",
            "\"ref_row_id\":",
        ] {
            assert!(line.contains(field), "{line} has no {field}");
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_frame_transport_answers_speak_and_read_frames() {
    let store = birth();
    let mut seam = Seam::new(SeamState::Open, GateLog::new());
    let mut reader = BufReader::new(
        "read_vocab\nspeak apple is red.\nread_rows\n".as_bytes(),
    );
    let mut out = Vec::new();
    let admissions = seam.serve(&store, &mut reader, &mut out).unwrap();
    assert_eq!(admissions, 0, "nothing is teachable at birth");
    let text = String::from_utf8(out).unwrap();
    let frames: Vec<&str> = text.lines().collect();
    assert_eq!(frames.len(), 3);
    assert!(frames[1].contains("\"admitted\":false"), "{}", frames[1]);
}
