//! 04 §21 U.3 / R11: the nonce is a letter-class shape drawn from this
//! session's tokens, seeded by the step so a replay draws the same guess.

use si_read::guess_seed;

fn shape(word: &str) -> Vec<bool> {
    word.chars().map(|ch| matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u')).collect()
}

#[test]
fn a_replay_draws_the_same_nonce() {
    let seen = vec![
        "apple".to_string(),
        "red".to_string(),
        "green".to_string(),
        "plum".to_string(),
        "round".to_string(),
    ];
    let first = guess_seed(7, &seen).expect("the session has tokens");
    let second = guess_seed(7, &seen).expect("the session has tokens");
    assert_eq!(first, second);
    assert!(!first.is_empty());
}

#[test]
fn the_nonce_is_a_shape_of_a_session_token() {
    let seen = vec!["apple".to_string(), "red".to_string(), "green".to_string()];
    let nonce = guess_seed(3, &seen).expect("the session has tokens");
    let nonce_shape = shape(&nonce);
    assert!(
        seen.iter().any(|token| shape(token) == nonce_shape),
        "`{nonce}` is not any session token's vowel/consonant shape"
    );
    for ch in nonce.chars() {
        assert!(ch.is_ascii_lowercase());
        assert!(
            seen.iter().any(|token| token.contains(ch)),
            "letter `{ch}` appears in no session token"
        );
    }
}

#[test]
fn no_tokens_is_no_guess() {
    assert!(guess_seed(1, &[]).is_none());
    assert!(guess_seed(1, &[String::new()]).is_none());
}

#[test]
fn a_replay_after_a_transcript_change_is_still_deterministic_on_its_own_input() {
    let left = vec!["apple".to_string(), "red".to_string(), "green".to_string()];
    let right = vec!["apple".to_string(), "red".to_string(), "green".to_string(), "plum".to_string()];
    assert_eq!(guess_seed(11, &left), guess_seed(11, &left));
    assert_eq!(guess_seed(11, &right), guess_seed(11, &right));
}
