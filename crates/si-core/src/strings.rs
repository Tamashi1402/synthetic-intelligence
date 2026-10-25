//! docs/03 §3.2: the complete innate vocabulary allowlist. These are the only
//! behavioural string constants in the build; `check_string_constants` diffs
//! this module against the spec byte-for-byte, and a word that is not here has
//! to arrive from the owner's transcript.

/// docs/03 §3.2 function words, in the table's order.
pub const FUNCTION_WORDS: [&str; 15] = [
    "the", "a", "an", "is", "are", "was", "were", "what", "who", "when", "where", "of", "means",
    "yes", "no",
];

/// docs/03 §3.2 `P_CORRECT` markers, longest first so the split is deterministic.
pub const CORRECTION_MARKERS: [&str; 6] = ["no wait", "wait", "actually", "instead", "rather", "no,"];

/// The nine named frames of the closed table (`Undecoded` is an outcome, not a
/// frame name shipped as vocabulary).
pub const PATTERN_FRAME_NAMES: [&str; 9] = [
    "P_ATTRIB",
    "P_DEF",
    "P_QUEST_A",
    "P_QUEST_D",
    "P_QUEST_W",
    "P_OF_REL",
    "P_CORRECT",
    "P_CMD_C",
    "P_TEACH",
];

pub fn function_word(word: &str) -> bool {
    FUNCTION_WORDS.contains(&word)
}

/// 04 §21.7: the voice tag is an input carried from the receiving module
/// (`panel` → owner, `mcp` → grimm), never inferred from text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Owner,
    Grimm,
}

impl Source {
    pub fn literal(self) -> &'static str {
        match self {
            Source::Owner => crate::SOURCE_LITERALS[0],
            Source::Grimm => crate::SOURCE_LITERALS[1],
        }
    }

    pub fn from_literal(text: &str) -> Option<Self> {
        if text == crate::SOURCE_LITERALS[0] {
            Some(Source::Owner)
        } else if text == crate::SOURCE_LITERALS[1] {
            Some(Source::Grimm)
        } else {
            None
        }
    }
}

/// Every declared vocabulary literal, for the auditor's byte-exact diff.
pub fn vocabulary() -> Vec<&'static str> {
    let mut out = Vec::new();
    out.extend_from_slice(&FUNCTION_WORDS);
    out.extend_from_slice(&CORRECTION_MARKERS);
    out.extend_from_slice(&PATTERN_FRAME_NAMES);
    out.extend_from_slice(&crate::SOURCE_LITERALS);
    out.extend_from_slice(&crate::TEACH_LITERALS);
    out
}
