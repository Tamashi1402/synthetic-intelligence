//! 04 §3 B.2 / docs/03 §3.2: R1's split and R2's fold.
//!
//! Whitespace and punctuation split; lowercase and the plural fold touch the
//! **lookup key only**, so the owner's casing survives in `surface`. The folded
//! key is an alias candidate, never a merge (R-04). Frame matching reads
//! `lower` (unfolded: `was` and `wa` are different frame words); term identity
//! reads `key`.

use si_store::interner::fold_key;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Token {
    /// The bytes the owner typed, verbatim.
    pub surface: String,
    /// Lowercase only; what the frame table's function-word test compares.
    pub lower: String,
    /// Folded: lowercase, plural-folded; what the interner and points key on.
    pub key: String,
}

pub fn tokenize(line: &str) -> Vec<Token> {
    line.split(|c: char| !c.is_alphanumeric())
        .filter(|piece| !piece.is_empty())
        .map(|piece| Token {
            surface: piece.to_string(),
            lower: piece.to_lowercase(),
            key: fold_key(piece),
        })
        .collect()
}

/// The folded keys in word order: the lighting request's input.
pub fn keys(line: &str) -> Vec<String> {
    tokenize(line).into_iter().map(|token| token.key).collect()
}
