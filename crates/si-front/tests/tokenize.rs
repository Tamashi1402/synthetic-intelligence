//! 04 §3 B.2 / docs/03 §3.2: R1's split and R2's fold. The surface stays
//! verbatim, `lower` is the frame test's input, `key` is term identity.

use si_core::strings::FUNCTION_WORDS;
use si_front::{keys, tokenize};

fn word(index: usize) -> &'static str {
    FUNCTION_WORDS[index]
}

#[test]
fn surface_lower_and_key_are_three_different_views() {
    let tokens = tokenize("Was CLASS");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].surface, "Was");
    assert_eq!(tokens[0].lower, word(5));
    // R2: the plural fold touches the lookup key, so `was` and `wa` collide.
    assert_eq!(tokens[0].key, "wa");
    assert_eq!(tokens[1].surface, "CLASS");
    assert_eq!(tokens[1].key, "clas");
}

#[test]
fn punctuation_splits_and_case_is_preserved_in_the_surface() {
    let tokens = tokenize("What COLOR is apple?");
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[1].surface, "COLOR");
    assert_eq!(tokens[1].lower, "color");
    assert_eq!(tokens[3].surface, "apple");
}

#[test]
fn a_word_broken_by_a_quote_is_two_tokens() {
    let tokens = tokenize("don't");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].surface, "don");
    assert_eq!(tokens[1].surface, "t");
}

#[test]
fn blank_input_tokenizes_to_nothing() {
    assert!(tokenize("").is_empty());
    assert!(tokenize("  \t ").is_empty());
    assert!(tokenize("?!.,").is_empty());
}

#[test]
fn keys_are_the_token_keys_in_word_order() {
    let folded = keys("What color is apples?");
    let tokens = tokenize("What color is apples?");
    let expected: Vec<String> = tokens.into_iter().map(|token| token.key).collect();
    assert_eq!(folded, expected);
    assert_eq!(keys("apples"), keys("apple"));
}

#[test]
fn the_plural_fold_strips_one_suffix_meaning_blind() {
    // docs/03 R2 is an `s`-suffix alias. 04 §3 B.2's example sentence says
    // `clas` and `class` collide on one folded key; a single-pass suffix strip
    // cannot produce that (`class` -> `clas`, `clas` -> `cla`). The build
    // follows the rule and the clash is an open item in the B4 report.
    assert_eq!(keys("class"), vec!["clas".to_string()]);
    assert_eq!(keys("clas"), vec!["cla".to_string()]);
    assert_eq!(keys("apples"), keys("apple"));
    // The surface itself is never rewritten.
    let long = tokenize("class");
    let short = tokenize("clas");
    assert_eq!(long[0].surface, "class");
    assert_eq!(short[0].surface, "clas");
}
