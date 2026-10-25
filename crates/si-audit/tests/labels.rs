//! engineering/11 §4 B6 DoD item 1. Three claims the crate owns:
//! (1) every emitted label cites rows that exist in the log — no uncitable label;
//! (2) classify is deterministic — a replay of the same rows gives a byte-equal
//!     stream (G9(b)'s acceptance condition);
//! (3) a log with no witnesses yields no label — nothing is fabricated.

use si_audit::{APoint, Act, ActKind, Signal, classify};

fn sig(seq: u64, t: i32, kind: u8, magnitude: u16) -> Signal {
    Signal { seq, t, thought: 1, kind, magnitude }
}

fn ap(seq: u64, t: i32, a: u8) -> APoint {
    APoint { seq, t, thought: 1, a }
}

fn act(seq: u64, t: i32, kind: ActKind) -> Act {
    Act { seq, t, thought: 1, kind }
}

fn rich() -> (Vec<Signal>, Vec<APoint>, Vec<Act>) {
    let signals = vec![
        sig(2, 1, si_audit::KIND_NOVELTY, 7),
        sig(5, 2, si_audit::KIND_REALIZED_GAIN, 4),
        sig(6, 2, si_audit::KIND_CONTRADICTION, 3),
        sig(7, 3, si_audit::KIND_CONTRADICTION, 2),
    ];
    let trajectory = vec![ap(1, 1, 5), ap(2, 1, 9), ap(3, 2, 2), ap(4, 3, 0)];
    let behavior = vec![
        act(3, 1, ActKind::Ask),
        act(5, 2, ActKind::ClosedAsk),
        act(6, 2, ActKind::ArbitrationAsk),
        act(8, 2, ActKind::SettleTauMiss),
        act(9, 3, ActKind::SettleMaxK),
    ];
    (signals, trajectory, behavior)
}

fn all_seqs(signals: &[Signal], trajectory: &[APoint], behavior: &[Act]) -> Vec<u64> {
    let mut seqs: Vec<u64> = signals
        .iter()
        .map(|s| s.seq)
        .chain(trajectory.iter().map(|p| p.seq))
        .chain(behavior.iter().map(|a| a.seq))
        .collect();
    seqs.sort_unstable();
    seqs
}

#[test]
fn every_label_cites_logged_rows() {
    let (signals, trajectory, behavior) = rich();
    let existing = all_seqs(&signals, &trajectory, &behavior);
    let labels = classify(&signals, &trajectory, &behavior);
    assert!(!labels.is_empty(), "a rich log must label something");
    for label in &labels {
        assert!(!label.citations.is_empty(), "{} emitted with no citation", label.name);
        for cite in &label.citations {
            assert!(
                existing.contains(cite),
                "label {} cites {cite}, which is not a logged row",
                label.name
            );
        }
    }
}

#[test]
fn the_six_labels_are_named_only_in_the_closed_vocabulary() {
    let (signals, trajectory, behavior) = rich();
    let names: Vec<&str> = classify(&signals, &trajectory, &behavior)
        .iter()
        .map(|label| label.name)
        .collect();
    assert!(names.contains(&"curiosity"));
    assert!(names.contains(&"surprise"));
    assert!(names.contains(&"satisfaction"));
    assert!(names.contains(&"caution"));
    assert!(names.contains(&"frustration"));
    assert!(!names.contains(&"boredom"), "a busy log is not bored");
    for name in names {
        assert!(
            si_audit::LABELS.contains(&name),
            "{name} is not in the installed vocabulary"
        );
    }
}

#[test]
fn classify_is_deterministic_on_replay() {
    let (signals, trajectory, behavior) = rich();
    let first = classify(&signals, &trajectory, &behavior);
    let second = classify(&signals, &trajectory, &behavior);
    assert_eq!(
        format!("{first:?}"),
        format!("{second:?}"),
        "the same logged rows must give a byte-equal stream"
    );
}

#[test]
fn an_empty_log_labels_nothing() {
    assert!(classify(&[], &[], &[]).is_empty());
    let quiet = vec![ap(1, 1, 0), ap(2, 2, 0)];
    // no novelty, no ask, no realized gain, no contradiction: even the boredom
    // predicate cites only rows that exist, and a flat zero-arousal log is a
    // finding, not an invented affect.
    let labels = classify(&[], &quiet, &[]);
    for label in &labels {
        for cite in &label.citations {
            assert!(quiet.iter().any(|p| &p.seq == cite));
        }
    }
}
