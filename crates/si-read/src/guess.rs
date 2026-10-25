//! 04 §21 U.3 / docs/07 §7.4 / R11: the nonce.
//!
//! The guess is a **shape**, not a word: it takes a session token's
//! vowel/consonant pattern (a letter-class shape) and fills each position with
//! a letter that occurred at that position in the same class somewhere in this
//! session's tokens. `Rng` is seeded by the logical step, so a replay of the
//! same transcript draws the same nonce; no table shipped with the build
//! contains one (`blork` is in no file — that is the anti-cheat proof, G5c).
//! Nothing is written (G1): the function has no store argument.

use std::collections::BTreeSet;

use si_core::rng::Rng;

/// `None` only when the session has no tokens at all: a shape with no data is
/// not a guess, and inventing an alphabet here would be a shipped table.
pub fn guess_seed(step: u32, seen: &[String]) -> Option<String> {
    let mut tokens: Vec<&str> = seen.iter().map(String::as_str).filter(|t| !t.is_empty()).collect();
    tokens.sort_unstable();
    tokens.dedup();
    if tokens.is_empty() {
        return None;
    }
    let mut rng = Rng::seeded(step as u64);
    let mut last = String::new();
    for _ in 0..4 {
        let base = tokens[(rng.next_u32() as usize) % tokens.len()];
        let shape: Vec<bool> = base.chars().map(is_vowel).collect();
        let mut out = String::with_capacity(shape.len());
        let mut failed = false;
        for (index, vowel) in shape.iter().enumerate() {
            let mut choices = letters(&tokens, Some(index), *vowel);
            if choices.is_empty() {
                choices = letters(&tokens, None, *vowel);
            }
            if choices.is_empty() {
                failed = true;
                break;
            }
            let pick = (rng.next_u32() as usize) % choices.len();
            out.push(*choices.iter().nth(pick).expect("index is modulo the set length"));
        }
        if failed {
            continue;
        }
        last = out;
        if !tokens.contains(&last.as_str()) {
            return Some(last);
        }
    }
    if last.is_empty() {
        None
    } else {
        Some(last)
    }
}

/// Letters of the given class seen at `position` (or anywhere, when `None`).
fn letters(tokens: &[&str], position: Option<usize>, vowel: bool) -> BTreeSet<char> {
    let mut out = BTreeSet::new();
    for token in tokens {
        match position {
            Some(index) => {
                if let Some(ch) = token.chars().nth(index) {
                    if is_vowel(ch) == vowel {
                        out.insert(ch);
                    }
                }
            }
            None => {
                for ch in token.chars() {
                    if is_vowel(ch) == vowel {
                        out.insert(ch);
                    }
                }
            }
        }
    }
    out
}

fn is_vowel(ch: char) -> bool {
    matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u')
}
