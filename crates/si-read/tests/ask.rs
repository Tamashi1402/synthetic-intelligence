//! 04 §11 K.5 / docs/07 §7.4: an ask-back is a copy of the owner's own words.

use si_core::patterns::PatternId;
use si_core::strings::{Source, FUNCTION_WORDS};
use si_read::{ask_back, SlotKey};

fn slot() -> SlotKey {
    SlotKey { frame: PatternId::PQuestA, point: 30 }
}

#[test]
fn an_owner_ask_copies_the_words_in_order() {
    let words = vec![
        FUNCTION_WORDS[7].to_string(),
        "color".to_string(),
        "apple".to_string(),
    ];
    let ask = ask_back(&slot(), &words, Source::Owner).expect("the owner asked");
    assert_eq!(ask.ask.slot, 30);
    assert_eq!(ask.surface, words.join(" "));
    assert_eq!(ask.surface, "what color apple");
}

#[test]
fn the_supporter_never_produces_an_ask() {
    let words = vec!["color".to_string()];
    assert!(ask_back(&slot(), &words, Source::Grimm).is_none());
}

#[test]
fn no_words_is_not_an_utterance() {
    assert!(ask_back(&slot(), &[], Source::Owner).is_none());
}
