//! engineering/09 §2.1: every malformation row, row by row, and the accept set
//! for the left side (the four question frames and nothing else).

use si_core::patterns::PatternId;
use si_core::strings::Source;
use si_front::{decide, split_line, LineKind, TeachOutcome, LEFT_ACCEPT_FRAMES};

#[test]
fn row_one_a_teach_with_no_marker_is_ordinary_sense() {
    assert!(matches!(split_line("/teach what color is apple"), LineKind::Sense));
    assert_eq!(decide("/teach what color is apple", Source::Owner), None);
}

#[test]
fn row_two_an_empty_left_is_undecoded_with_zero_rows() {
    let decision = decide("/teach => red", Source::Owner).expect("still a teach line");
    assert_eq!(decision.outcome, TeachOutcome::Undecoded);
    assert_eq!(decision.rows, 0);
    assert_eq!(decision.outcome.reason(), "left_undecoded");
    assert!(decision.dark.is_empty());
    assert_eq!(decision.response, "red");
}

#[test]
fn row_three_an_empty_right_names_the_response_slot_and_writes_zero_rows() {
    let decision = decide("/teach what color is apple =>", Source::Owner).expect("a teach line");
    assert_eq!(decision.outcome, TeachOutcome::EmptyResponse);
    assert_eq!(decision.rows, 0);
    assert_eq!(decision.outcome.reason(), "empty_response");
    // The left side did decode: the named spot is the slot the response would
    // have bound.
    assert_eq!(decision.dark.len(), 1);
    assert_eq!(decision.dark[0].key, "color");
    assert_eq!(decision.response, "");
}

#[test]
fn row_four_an_undecoded_left_names_the_slots_it_did_decode() {
    let decision = decide("/teach what color is => blue", Source::Owner).expect("a teach line");
    assert_eq!(decision.outcome, TeachOutcome::Undecoded);
    assert_eq!(decision.rows, 0);
    assert_eq!(decision.dark.len(), 1);
    assert_eq!(decision.dark[0].key, "color");
}

#[test]
fn row_five_a_statement_left_is_rejected_by_name() {
    for left in ["apple is red", "znark means floop"] {
        let decision =
            decide(&format!("/teach {left} => content"), Source::Owner).expect("a teach line");
        assert_eq!(decision.outcome, TeachOutcome::LeftNotAQuestion, "left `{left}`");
        assert_eq!(decision.rows, 0);
        assert_eq!(decision.outcome.reason(), "left_not_a_question");
        assert!(decision.pattern.is_none());
    }
}

#[test]
fn row_six_a_well_formed_line_writes_one_row() {
    let decision =
        decide("/teach what color is apple => it is red", Source::Owner).expect("a teach line");
    assert_eq!(decision.outcome, TeachOutcome::Row);
    assert_eq!(decision.rows, 1);
    assert_eq!(decision.outcome.reason(), "");
    assert_eq!(decision.response, "it is red");
    let pattern = decision.pattern.expect("the row's key is the decoded left side");
    assert_eq!(pattern.frame, PatternId::PQuestA);
    assert_eq!(pattern.slots.len(), 2);
}

#[test]
fn the_accept_set_is_exactly_the_four_question_frames() {
    assert_eq!(
        LEFT_ACCEPT_FRAMES,
        [PatternId::PQuestA, PatternId::PQuestD, PatternId::PQuestW, PatternId::POfRel]
    );
    const ACCEPTED: [&str; 4] =
        ["what color is apple", "what is apple", "when was apple", "red of apple"];
    for left in ACCEPTED {
        let decision = decide(&format!("/teach {left} => content"), Source::Owner).unwrap();
        assert_eq!(decision.outcome, TeachOutcome::Row, "left `{left}`");
    }
    const REJECTED: [&str; 4] = [
        "apple is red",
        "apple means fruit",
        "/correct apple is green",
        "no wait apple is green",
    ];
    for left in REJECTED {
        let decision = decide(&format!("/teach {left} => content"), Source::Owner).unwrap();
        assert_eq!(decision.outcome, TeachOutcome::LeftNotAQuestion, "left `{left}`");
        assert_eq!(decision.rows, 0);
    }
}

#[test]
fn a_grimm_teach_line_is_inert_before_anything_else_is_read() {
    for line in ["/teach what color is apple => red", "/teach => red"] {
        let decision = decide(line, Source::Grimm).expect("still a teach-shaped line");
        assert_eq!(decision.outcome, TeachOutcome::Inert);
        assert_eq!(decision.rows, 0);
        assert_eq!(decision.outcome.reason(), "grimm_source_inert");
        assert!(decision.pattern.is_none());
    }
}

#[test]
fn the_marker_needs_a_word_boundary() {
    assert!(matches!(split_line("/teacher what is apple => x"), LineKind::Sense));
    assert_eq!(decide("/teacher what is apple => x", Source::Owner), None);
}

#[test]
fn the_right_side_is_verbatim_content_including_the_marker() {
    let decision = decide("/teach what is apple => red => blue", Source::Owner).unwrap();
    assert_eq!(decision.outcome, TeachOutcome::Row);
    assert_eq!(decision.response, "red => blue");
    match split_line("/teach what is apple => red => blue") {
        LineKind::Teach { left, right } => {
            assert_eq!(left, "what is apple");
            assert_eq!(right, "red => blue");
        }
        LineKind::Sense => panic!("a teach line was read as sense"),
    }
}
