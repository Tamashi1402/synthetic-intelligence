//! 04 §3 B.3 / docs/03 §3.2: the shape-only frame test, and the checked-in
//! 20-line fixture whose `Pattern|Undecoded` sequence is byte-exact.

use std::fs;
use std::path::PathBuf;

use md5::{Digest, Md5};
use si_core::patterns::{PatternId, PATTERN_TABLE, PATTERN_TABLE_SIZE};
use si_front::{decode, split_line, Decoded, LineKind, QUESTION_FRAMES};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

#[test]
fn the_pattern_table_is_ten_entries_nine_named_plus_the_outcome() {
    assert_eq!(PATTERN_TABLE_SIZE, 10);
    assert_eq!(PATTERN_TABLE.len(), 10);
    let named = PATTERN_TABLE.iter().filter(|frame| **frame != PatternId::Undecoded).count();
    assert_eq!(named, 9);
    assert_eq!(PATTERN_TABLE[PATTERN_TABLE_SIZE - 1], PatternId::Undecoded);
}

#[test]
fn every_named_frame_is_reachable() {
    const LINES: [(&str, PatternId); 9] = [
        ("what color is apple", PatternId::PQuestA),
        ("what is apple", PatternId::PQuestD),
        ("where is apple", PatternId::PQuestW),
        ("red of apple", PatternId::POfRel),
        ("apple means fruit", PatternId::PDef),
        ("apple is red", PatternId::PAttrib),
        ("no wait it is green", PatternId::PCorrect),
        ("/correct apple is green", PatternId::PCmdC),
        ("/teach what color is apple => red", PatternId::PTeach),
    ];
    for (line, frame) in LINES {
        let decoded = decode(line);
        assert_eq!(decoded.frame(), frame, "line `{line}`");
        assert!(matches!(decoded, Decoded::Pattern(_)), "line `{line}` did not match");
    }
    // The tenth entry is the outcome, never a matched frame.
    assert!(matches!(decode("the sky was blue"), Decoded::Undecoded { .. }));
}

#[test]
fn the_four_question_frames_are_the_teach_accept_set() {
    assert_eq!(
        QUESTION_FRAMES,
        [PatternId::PQuestA, PatternId::PQuestD, PatternId::PQuestW, PatternId::POfRel]
    );
    for line in ["what color is apple", "what is apple", "who was apple", "red of apple"] {
        assert!(decode(line).is_question(), "line `{line}`");
    }
    for line in ["apple is red", "apple means fruit", "/correct apple is green"] {
        assert!(!decode(line).is_question(), "line `{line}`");
    }
}

#[test]
fn match_order_puts_questions_before_statements() {
    // `what is apple` is a definition-shaped statement AND a P_QUEST_D shape.
    assert_eq!(decode("what is apple").frame(), PatternId::PQuestD);
    assert_eq!(decode("apple is red").frame(), PatternId::PAttrib);
    // A correction marker wins over any frame it happens to resemble.
    assert_eq!(decode("no wait it is green").frame(), PatternId::PCorrect);
}

#[test]
fn question_slots_follow_docs_03_index_positions() {
    let decoded = decode("what color is apple");
    let words: Vec<u8> = decoded.slots().iter().map(|slot| slot.word).collect();
    assert_eq!(words, vec![1, 3]);
    assert_eq!(decoded.slots()[0].key, "color");
    assert_eq!(decoded.slots()[1].key, "apple");

    let relation = decode("red of apple");
    let words: Vec<u8> = relation.slots().iter().map(|slot| slot.word).collect();
    assert_eq!(words, vec![0, 2]);
}

#[test]
fn a_command_carries_every_token_as_a_slot() {
    let decoded = decode("/correct apple is green");
    assert_eq!(decoded.frame(), PatternId::PCmdC);
    assert_eq!(decoded.slots().len(), 3);
    assert_eq!(decoded.surface(), "/correct apple is green");
}

#[test]
fn a_longer_word_is_not_the_command_marker() {
    assert_eq!(decode("/correction apple is green").frame(), PatternId::Undecoded);
    assert_eq!(decode("/correct").frame(), PatternId::Undecoded);
}

#[test]
fn gibberish_parity_is_the_same_frame_with_different_slots() {
    let real = decode("what color is apple");
    let gibberish = decode("what znark is floop");
    assert_eq!(real.frame(), gibberish.frame());
    assert_eq!(real.slots().len(), 2);
    assert_eq!(gibberish.slots().len(), 2);
    assert_ne!(real.slots()[1].key, gibberish.slots()[1].key);
}

#[test]
fn teach_splits_at_the_first_marker_and_decodes_its_left_side() {
    match split_line("/teach what color is apple => red => blue") {
        LineKind::Teach { left, right } => {
            assert_eq!(left, "what color is apple");
            assert_eq!(right, "red => blue");
        }
        LineKind::Sense => panic!("a teach line was read as sense"),
    }
    let decoded = decode("/teach what color is apple => red => blue");
    assert_eq!(decoded.frame(), PatternId::PTeach);
    assert_eq!(decoded.surface(), "what color is apple");
    // Shape-only: a statement-shaped left side still decodes; `teach::decide`
    // is what refuses it.
    assert_eq!(decode("/teach apple is red => it is red").frame(), PatternId::PTeach);
}

#[test]
fn a_named_sub_slot_set_survives_an_undecoded_line() {
    match decode("what color is") {
        Decoded::Undecoded { named_dark, .. } => {
            assert_eq!(named_dark.len(), 1);
            assert_eq!(named_dark[0].key, "color");
        }
        Decoded::Pattern(pattern) => panic!("decoded to {:?}", pattern.frame),
    }
    assert!(matches!(decode(""), Decoded::Undecoded { .. }));
    assert!(matches!(decode("what is"), Decoded::Undecoded { .. }));
}

#[test]
fn the_checked_in_fixture_decodes_byte_for_byte() {
    let fixture_path = fixture("decode_20.txt");
    let bytes = fs::read(&fixture_path).expect("read the decode fixture");
    let pinned = fs::read_to_string(fixture("decode_20.md5")).expect("read the fixture hash");
    let digest = si_core::hash::hex(Md5::digest(&bytes).as_slice());
    assert_eq!(digest, pinned.trim(), "the decode fixture changed; its sequence is re-derived");

    let text = String::from_utf8(bytes).expect("the fixture is UTF-8");
    let lines: Vec<&str> = text.lines().filter(|line| !line.trim().is_empty()).collect();
    assert_eq!(lines.len(), 20);
    let produced: Vec<String> =
        lines.iter().map(|line| decode(line).frame().name().to_string()).collect();
    let expected = fs::read_to_string(fixture("decode_20.expected")).expect("read the sequence");
    assert_eq!(produced.join("\n"), expected.trim_end_matches('\n'));
}
