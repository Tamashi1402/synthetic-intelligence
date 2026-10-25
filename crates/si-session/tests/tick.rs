//! 04 §12 / 09 §2.1: the tick end to end. These are the B5 DoD's session-side
//! assertions, written against the store rather than the mind log so a row that
//! is logged but not shelved still fails.

use si_core::addr::{SHELF_CONTEXT, SHELF_FACTS, SHELF_QUESTIONS, SHELF_S_R_PAIRS};
use si_core::record::{Record, EDGE_BYTES, RECORD_BYTES, STAMP_BYTES};
use si_core::strings::Source;
use si_session::{Output, Session};

/// 02 §9's `gap` event code; the session's constants are crate-private.
const EV_GAP: u8 = 8;

fn gap_lines(session: &Session) -> Vec<String> {
    session
        .events()
        .events()
        .iter()
        .filter(|event| event.kind == EV_GAP)
        .map(|event| String::from_utf8_lossy(&event.body).into_owned())
        .collect()
}

fn shelf_rows(session: &Session, shelf: u16) -> Vec<Record> {
    session
        .store()
        .rows()
        .unwrap()
        .into_iter()
        .map(|(_, record)| record)
        .filter(|record| record.envelope.shelf_x == shelf)
        .collect()
}

fn live_rows(session: &Session, shelf: u16) -> Vec<Record> {
    shelf_rows(session, shelf)
        .into_iter()
        .filter(|record| record.envelope.is_live())
        .collect()
}

#[test]
fn a_question_asks_back_and_writes_no_belief() {
    let mut session = Session::new(0x51).unwrap();
    let report = session.tick("what color is apple?", Source::Owner).unwrap();
    assert!(matches!(report.output, Output::Ask { fresh: true, .. }), "{:?}", report.output);
    assert_eq!(report.rows_written, 0);
    assert_eq!(shelf_rows(&session, SHELF_QUESTIONS).len(), 1);
    assert_eq!(shelf_rows(&session, SHELF_S_R_PAIRS).len(), 0);
    assert_eq!(shelf_rows(&session, SHELF_FACTS).len(), 0);
    let question = &shelf_rows(&session, SHELF_QUESTIONS)[0];
    assert_eq!(question.body.status, 0, "the ask is open");
    assert_eq!(question.envelope.prov_speaker, 0);
    assert_eq!(question.body.arousal_at_ask, session.a());
}

#[test]
fn a_correction_after_an_ask_writes_one_bound_row_and_closes_it() {
    let mut session = Session::new(0x51).unwrap();
    let ask = session.tick("what color is apple?", Source::Owner).unwrap();
    let Output::Ask { question_id, .. } = ask.output else {
        panic!("the first line must ask, got {:?}", ask.output);
    };
    let corrected = session.tick("/correct Red", Source::Owner).unwrap();
    assert!(matches!(corrected.output, Output::Committed { .. }), "{:?}", corrected.output);
    assert_eq!(corrected.rows_written, 1);
    assert_eq!(corrected.salience_calls, 1);
    let beliefs = shelf_rows(&session, SHELF_S_R_PAIRS);
    assert_eq!(beliefs.len(), 1, "one S-R row, never on the questions shelf");
    // Step 9 is an event, not a rewrite: the open row stays, and a version row
    // carries the resolution (01 §4.2's status column has one writer).
    let questions = shelf_rows(&session, SHELF_QUESTIONS);
    assert_eq!(questions.len(), 2);
    let belief = &beliefs[0];
    assert_ne!(belief.envelope.bind_output_id, 0);
    assert_eq!(belief.envelope.bind_output_id, question_id);
    assert_eq!(belief.envelope.confidence, 95);
    assert_eq!(belief.envelope.gain_kind, 1, "a correction is demonstrated");
    let version = questions.iter().max_by_key(|row| row.envelope.row_id).unwrap();
    assert_eq!(version.body.status, 1, "step 9 resolves by row existence");
    assert_eq!(version.envelope.bind_output_id, belief.envelope.row_id);
    assert_eq!(version.envelope.superseded_at, i32::MAX, "an event, not a rewrite");
}

#[test]
fn a_repeated_question_serves_the_row_without_writing() {
    let mut session = Session::new(0x51).unwrap();
    session.tick("what color is apple?", Source::Owner).unwrap();
    session.tick("/correct Red", Source::Owner).unwrap();
    let recall = session.tick("what color is apple?", Source::Owner).unwrap();
    assert!(matches!(recall.output, Output::Answer { .. }), "{:?}", recall.output);
    assert_eq!(recall.rows_written, 0);
    let beliefs = live_rows(&session, SHELF_S_R_PAIRS);
    assert_eq!(beliefs.len(), 1);
    assert_eq!(beliefs[0].envelope.repeats, 2);
    assert_eq!(beliefs[0].envelope.confidence, 99);
}

#[test]
fn a_bare_statement_then_repeat_is_boring() {
    let mut session = Session::new(0x51).unwrap();
    let first = session.tick("apple is red.", Source::Owner).unwrap();
    assert!(matches!(first.output, Output::Committed { branch: "commit", .. }), "{:?}", first.output);
    assert_eq!(first.salience_calls, 1);
    let second = session.tick("apple is red.", Source::Owner).unwrap();
    assert_eq!(second.rows_written, 0, "{:?}", second.output);
    assert_eq!(second.salience_calls, 0);
    let rows = live_rows(&session, SHELF_S_R_PAIRS);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].envelope.repeats, 2);
    assert_eq!(rows[0].envelope.confidence, 95, "90 init + 5, capped only by 99");
}

#[test]
fn a_teach_writes_one_asserted_row_with_no_question() {
    let mut session = Session::new(0x51).unwrap();
    let report = session.tick("/teach what color is apple => red", Source::Owner).unwrap();
    assert!(matches!(report.output, Output::Committed { .. }), "{:?}", report.output);
    assert_eq!(report.rows_written, 1);
    let rows = shelf_rows(&session, SHELF_S_R_PAIRS);
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.envelope.prov_kind, 4);
    assert_eq!(row.envelope.prov_speaker, 0);
    assert_eq!(row.envelope.bind_output_id, 0);
    assert_eq!(row.envelope.question_id, 0);
    assert_eq!(row.envelope.confidence, 95);
    assert_eq!(row.envelope.gain_kind, 0);
    assert_eq!(row.envelope.gain_exact, 0);
}

#[test]
fn an_undecoded_line_writes_only_its_context_row() {
    let mut session = Session::new(0x51).unwrap();
    let report = session.tick("hello.", Source::Owner).unwrap();
    assert_eq!(report.rows_written, 0);
    assert_eq!(shelf_rows(&session, SHELF_S_R_PAIRS).len(), 0);
    assert_eq!(shelf_rows(&session, SHELF_QUESTIONS).len(), 0);
    assert_eq!(shelf_rows(&session, SHELF_CONTEXT).len(), 1, "01 §6: the input floor is paid");
}

#[test]
fn every_teach_malformation_pays_the_context_floor_and_nothing_else() {
    // 09 §2.1, row by row: the reason string, one `context` row (112 B), zero
    // knowledge rows, zero question rows, no edge and no stamp.
    let cases = [
        (si_core::TEACH_LITERALS[0], "input_not_a_teach"),
        ("/teach => red", "left_undecoded"),
        ("/teach what color is apple =>", "empty_response"),
        ("/teach what color is => blork", "left_undecoded"),
        ("/teach apple is red => blue", "left_not_a_question"),
    ];
    for (line, reason) in cases {
        let mut session = Session::new(0x51).unwrap();
        let rows0 = session.store().rows_arena().len();
        let edges0 = session.store().edges_arena().len();
        let stamps0 = session.store().stamps_arena().len();
        let report = session.tick(line, Source::Owner).unwrap();
        match &report.output {
            Output::Refused { reason: got } | Output::Inert { reason: got } => {
                assert_eq!(got, reason, "{line}")
            }
            other => panic!("{line}: expected a refusal, got {other:?}"),
        }
        assert_eq!(report.rows_written, 0, "{line}");
        assert_eq!(
            session.store().rows_arena().len(),
            rows0 + RECORD_BYTES,
            "{line}: exactly one context record"
        );
        assert_eq!(session.store().edges_arena().len(), edges0, "{line}: no edge");
        assert_eq!(session.store().stamps_arena().len(), stamps0, "{line}: no stamp");
        assert_eq!(shelf_rows(&session, SHELF_CONTEXT).len(), 1, "{line}");
        assert_eq!(shelf_rows(&session, SHELF_S_R_PAIRS).len(), 0, "{line}");
        assert_eq!(shelf_rows(&session, SHELF_FACTS).len(), 0, "{line}");
        assert_eq!(shelf_rows(&session, SHELF_QUESTIONS).len(), 0, "{line}");
        let context = &shelf_rows(&session, SHELF_CONTEXT)[0];
        assert_eq!(context.envelope.charge_off, 0, "01 §6: context carries no charge");
        assert_eq!(context.envelope.edges_len, 0, "01 §6: context carries no edge");
        assert_eq!(context.envelope.text_ref, context.body.stim_surface);
        assert_eq!(context.envelope.prov_speaker, 0, "a line's speaker is its source");
    }
}

#[test]
fn a_teach_malformation_names_the_dark_slot_it_could_not_use() {
    // 09 §2.1 rows 2 and 4: an empty left side decodes nothing (no dark slot);
    // a partly decoded left names its own sub-slot without writing a row.
    let mut session = Session::new(0x51).unwrap();
    session.tick("/teach => red", Source::Owner).unwrap();
    assert_eq!(gap_lines(&session), vec!["{\"dark\":[],\"reason\":\"left_undecoded\"}"]);

    let mut session = Session::new(0x51).unwrap();
    session.tick("/teach what color is => blork", Source::Owner).unwrap();
    let gaps = gap_lines(&session);
    assert_eq!(gaps.len(), 1);
    assert!(gaps[0].contains("\"dark\":[\"color\"]"), "{}", gaps[0]);
    assert_eq!(shelf_rows(&session, SHELF_QUESTIONS).len(), 0, "G1.t.3: zero rows");

    // 09 §2.1 row 3: the left decoded, so the slot SI would have answered (the
    // first decoded slot, `color`) is the named dark spot of the empty response.
    let mut session = Session::new(0x51).unwrap();
    session.tick("/teach what color is apple =>", Source::Owner).unwrap();
    let gaps = gap_lines(&session);
    assert_eq!(gaps.len(), 1);
    assert!(gaps[0].contains("\"dark\":[\"color\"]"), "{}", gaps[0]);
    assert!(gaps[0].contains("\"reason\":\"empty_response\""), "{}", gaps[0]);
}

#[test]
fn a_good_teach_pays_the_floor_plus_one_112_16_16_row() {
    let mut session = Session::new(0x51).unwrap();
    let rows0 = session.store().rows_arena().len();
    let edges0 = session.store().edges_arena().len();
    let stamps0 = session.store().stamps_arena().len();
    session.tick("/teach what color is apple => red", Source::Owner).unwrap();
    assert_eq!(session.store().rows_arena().len(), rows0 + 2 * RECORD_BYTES, "context + belief");
    assert_eq!(session.store().edges_arena().len(), edges0 + EDGE_BYTES);
    assert_eq!(session.store().stamps_arena().len(), stamps0 + STAMP_BYTES);
    assert_eq!(shelf_rows(&session, SHELF_CONTEXT).len(), 1);
    assert_eq!(shelf_rows(&session, SHELF_S_R_PAIRS).len(), 1);
}

#[test]
fn a_bare_reply_to_an_open_ask_commits_answer_to_ask() {
    let mut session = Session::new(0x51).unwrap();
    let ask = session.tick("what color is apple?", Source::Owner).unwrap();
    let Output::Ask { question_id, .. } = ask.output else {
        panic!("expected an ask, got {:?}", ask.output);
    };
    let reply = session.tick("red.", Source::Owner).unwrap();
    assert!(matches!(reply.output, Output::Committed { .. }), "{:?}", reply.output);
    let rows = shelf_rows(&session, SHELF_S_R_PAIRS);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].envelope.prov_kind, 3);
    assert_eq!(rows[0].envelope.question_id, question_id);
    assert_eq!(rows[0].envelope.bind_output_id, question_id);
}

#[test]
fn a_bare_reply_with_no_ask_writes_nothing() {
    let mut session = Session::new(0x51).unwrap();
    let report = session.tick("red.", Source::Owner).unwrap();
    assert_eq!(report.rows_written, 0);
    assert_eq!(shelf_rows(&session, SHELF_S_R_PAIRS).len(), 0);
}

#[test]
fn a_second_statement_at_the_slot_asks_before_committing() {
    let mut session = Session::new(0x51).unwrap();
    session.tick("apple is red.", Source::Owner).unwrap();
    let conflict = session.tick("apple is green.", Source::Owner).unwrap();
    assert!(matches!(conflict.output, Output::Ask { .. }), "{:?}", conflict.output);
    assert_eq!(conflict.rows_written, 0);
    let questions = shelf_rows(&session, SHELF_QUESTIONS);
    assert_eq!(questions.len(), 1);
    let insistence = session.tick("apple is green.", Source::Owner).unwrap();
    assert!(matches!(insistence.output, Output::Committed { branch: "insistence", .. }), "{:?}", insistence.output);
    let beliefs = shelf_rows(&session, SHELF_S_R_PAIRS);
    assert_eq!(beliefs.len(), 2, "the loser is aged, never deleted");
    let aged: Vec<_> = beliefs.iter().filter(|row| !row.envelope.is_live()).collect();
    assert_eq!(aged.len(), 1);
    assert_eq!(aged[0].envelope.superseded_by, insistence_row(&session));
}

fn insistence_row(session: &Session) -> u32 {
    shelf_rows(session, SHELF_S_R_PAIRS)
        .into_iter()
        .filter(|row| row.envelope.is_live())
        .map(|row| row.envelope.row_id)
        .max()
        .unwrap()
}

#[test]
fn six_corrections_derive_facts_and_a_second_pass_is_idle() {
    let mut session = Session::new(0x51).unwrap();
    let entities = ["apple", "pear", "plum", "fig", "kiwi"];
    for entity in entities {
        session
            .tick(&format!("what color is {entity}?"), Source::Owner)
            .unwrap();
        session.tick("/correct red", Source::Owner).unwrap();
    }
    // Stage S0 exits only with a superseded S-R row; this correction is still
    // an S0 write because the stage is read before the tick's commit.
    session.tick("/correct blue", Source::Owner).unwrap();
    assert_eq!(live_rows(&session, SHELF_S_R_PAIRS).len(), 5);
    assert_eq!(
        shelf_rows(&session, SHELF_S_R_PAIRS).iter().filter(|row| !row.envelope.is_live()).count(),
        1
    );

    let idle = session.idle_tick().unwrap();
    // The superseded parent still derives, but its typed view is an old value at
    // a slot the owner has since superseded: the ordinary gate rejects it.
    assert_eq!(idle.derived.len(), 5);
    assert_eq!(live_rows(&session, SHELF_FACTS).len(), 5);
    let again = session.idle_tick().unwrap();
    assert!(again.derived.is_empty(), "re-parse is idempotent");

    let fact = live_rows(&session, SHELF_FACTS)[0].clone();
    assert_eq!(fact.envelope.prov_kind, 2);
    assert_eq!(fact.envelope.prov_speaker, 2);
    assert_eq!(fact.envelope.derived_len, 1);
    let parents = session.store().handles_at(fact.envelope.derived_off, fact.envelope.derived_len).unwrap();
    assert_eq!(parents.len(), 1);
    let parent = shelf_rows(&session, SHELF_S_R_PAIRS)
        .into_iter()
        .find(|row| row.envelope.row_id == parents[0])
        .expect("the chain resolves to an original S-R row");
    assert_eq!(fact.body.key_point, parent.body.key_point);
    assert_eq!(fact.body.val_point, parent.body.val_point);
    assert_eq!(fact.envelope.confidence, 55);
    assert_eq!(fact.envelope.gain_kind, 0);
    assert_eq!(fact.envelope.gain_exact, 0);
    let edges = session.store().edges_of(&fact).unwrap();
    assert!(edges.iter().all(|edge| edge.bake == 0), "a derivation manufactures no boost");
}

#[test]
fn a_taught_spelling_is_a_zero_row_confirm_and_joins_the_bank() {
    // **OD-189**: a case spelling taught in the predicate position is the same
    // claim by the one fold law, so it writes no row; the verbatim bytes still
    // enter the interner, which is the surface the license's variant bank groups.
    let mut session = Session::new(0x51).unwrap();
    session.tick("apple is red.", Source::Owner).unwrap();
    session.tick("red is warm.", Source::Owner).unwrap();
    let rows_before = shelf_rows(&session, SHELF_S_R_PAIRS).len();
    for line in ["apple is Red.", "apple is RED.", "red is Warm.", "red is WARM."] {
        let report = session.tick(line, Source::Owner).unwrap();
        assert_eq!(report.rows_written, 0, "{line} asserted a row");
        assert!(matches!(&report.output, Output::Inert { reason } if reason == "boring"), "{line}: {:?}", report.output);
    }
    assert_eq!(shelf_rows(&session, SHELF_S_R_PAIRS).len(), rows_before, "the bank is not a shelf");
    assert!(live_rows(&session, SHELF_S_R_PAIRS).iter().all(|row| row.envelope.is_live()));

    let spellings = |surface: &str| -> Vec<String> {
        session
            .store()
            .interner()
            .lookup(surface)
            .into_iter()
            .filter_map(|id| session.store().interner().get(id).map(|entry| entry.verbatim.clone()))
            .collect()
    };
    for (folded, wanted) in [("red", ["Red", "RED"]), ("warm", ["Warm", "WARM"])] {
        let bank = spellings(folded);
        for surface in wanted {
            assert!(bank.iter().any(|entry| entry == surface), "{surface} is not beside {folded}: {bank:?}");
        }
    }

    // The claim still answers, and the answer cites the taught row.
    let answer = session.tick("what color is apple?", Source::Owner).unwrap();
    assert!(matches!(answer.output, Output::Answer { .. }), "{:?}", answer.output);
}
