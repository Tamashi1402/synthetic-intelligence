//! 04 §19 / 05 §15: the session-side law of the idle tick. These are B7's DoD
//! assertions written against the store and the monologue lane themselves, so a
//! thought that is rendered but also believed still fails.

use si_core::record::Record;
use si_core::strings::Source;
use si_session::Session;

/// Every byte a belief write can touch, as one comparable blob: the row
/// envelopes in `row_id` order, the edge arena (each edge's formation-time
/// `bake`) and the stamp arena.
fn belief_bytes(session: &Session) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let store = session.store();
    let mut rows: Vec<Record> = store.rows().unwrap().into_iter().map(|(_, r)| r).collect();
    rows.sort_by_key(|record| record.envelope.row_id);
    let mut rows_blob = Vec::new();
    for record in &rows {
        rows_blob.extend(record.encode());
    }
    (
        rows_blob,
        store.edges_arena().bytes().to_vec(),
        store.stamps_arena().bytes().to_vec(),
    )
}

/// The taught-then-asked transcript: every question is answerable, so the
/// agenda is empty when the quiet ticks arrive.
fn taught(session: &mut Session) {
    for line in ["apple is red.", "what color is apple?", "banana is yellow."] {
        session.tick(line, Source::Owner).unwrap();
    }
}

#[test]
fn a_quiet_tick_with_a_clear_agenda_thinks_and_writes_no_belief() {
    let mut with_thoughts = Session::new(0x51).unwrap();
    taught(&mut with_thoughts);
    let before = belief_bytes(&with_thoughts);
    let rows_before = with_thoughts.store().rows().unwrap().len();
    let mut reports = Vec::new();
    for _ in 0..3 {
        reports.push(with_thoughts.idle_tick().unwrap());
    }
    assert!(
        !with_thoughts.monologue().is_empty(),
        "an agenda-empty quiet run thought nothing at all"
    );
    assert_eq!(
        with_thoughts.store().rows().unwrap().len(),
        rows_before,
        "an idle tick wrote a belief row"
    );
    assert_eq!(belief_bytes(&with_thoughts), before, "an idle tick moved a durable byte");
    assert!(
        with_thoughts.monologue().len() <= reports.len(),
        "the monologue outran the idle clock"
    );
    let ticks: Vec<i32> = with_thoughts.monologue().iter().map(|line| line.t).collect();
    let distinct: std::collections::BTreeSet<i32> = ticks.iter().copied().collect();
    assert_eq!(distinct.len(), ticks.len(), "two monologue entries share one tick");
}

#[test]
fn a_pending_agenda_is_revisited_before_any_thought() {
    let mut session = Session::new(0x51).unwrap();
    session.tick("what color is dragon?", Source::Owner).unwrap();
    let report = session.idle_tick().unwrap();
    assert!(report.ignited.is_none(), "ignition fired while the agenda still held a question");
    assert!(session.monologue().is_empty(), "the monologue lane filled during a pending ask");
}

#[test]
fn the_owner_line_after_a_quiet_tick_keeps_the_open_question() {
    let mut session = Session::new(0x51).unwrap();
    session.tick("what color is dragon?", Source::Owner).unwrap();
    session.idle_tick().unwrap();
    let report = session.tick("what color is dragon?", Source::Owner).unwrap();
    assert!(matches!(report.output, si_session::Output::Ask { .. }), "{:?}", report.output);
    let open = si_learn::gate::open_asks(session.store(), session.now()).unwrap();
    assert!(!open.is_empty(), "the daydream dropped a real question");
}

#[test]
fn a_v2_launch_thinks_nothing_and_changes_no_durable_byte() {
    let mut on = Session::new(0x51).unwrap();
    taught(&mut on);
    for _ in 0..3 {
        on.idle_tick().unwrap();
    }
    let mut off = Session::new(0x51).unwrap();
    off.set_launch_variant("V2");
    taught(&mut off);
    for _ in 0..3 {
        let report = off.idle_tick().unwrap();
        assert!(report.ignited.is_none(), "V2 ignited");
    }
    assert!(!on.monologue().is_empty(), "the comparison had nothing to compare");
    assert!(off.monologue().is_empty(), "V2 rendered a monologue line");
    assert_eq!(belief_bytes(&on), belief_bytes(&off), "ignition moved a durable field");
}

#[test]
fn every_monologue_token_is_a_token_of_a_point_that_lit() {
    let mut session = Session::new(0x51).unwrap();
    taught(&mut session);
    for _ in 0..3 {
        session.idle_tick().unwrap();
    }
    for line in session.monologue() {
        assert!(!line.cites.is_empty(), "thought {} cites nothing", line.thought);
        assert_eq!(
            line.cites.len(),
            line.lit_points.len(),
            "thought {}'s citations are not 1:1 with its lit set",
            line.thought
        );
        let mut lit_words: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for (node, _, _) in &line.cites {
            assert!(line.lit_points.contains(node), "citation {} is not displayed", node);
            assert!(
                session.store().points().canonical(*node).is_some(),
                "citation {} names a point the store does not hold",
                node
            );
            let surface = session.store().points().surface(*node).unwrap_or("");
            for word in surface.split_whitespace() {
                lit_words.insert(word);
            }
        }
        for word in line.surface.split_whitespace() {
            assert!(
                lit_words.contains(word),
                "thought {} said {word:?}, which never lit",
                line.thought
            );
        }
    }
}
