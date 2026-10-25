//! 04 §3 B.3 / docs/03 §3.2: the shape-only frame test.
//!
//! Nine named frames, one trailing `Undecoded`; the test reads anchor positions
//! and the §3.2 function-word table and nothing else. No semantics, no
//! similarity, no spelling correction (R-14): `what znark is floop` and
//! `what color is apple` produce the same frame with different slot keys.
//!
//! The raw-line prefixes (`/teach`, `/correct`) are tested before tokenizing
//! (04 §3 B.1). `/teach` is the session's channel shape; the left side is then
//! decoded by the same frame test (`teach::decide`), never by a second parser.

use si_core::patterns::PatternId;
use si_core::strings::FUNCTION_WORDS;

use crate::teach::{split_line, LineKind};
use crate::tokenize::{tokenize, Token};

/// docs/03 §3.2 `P_CMD_C`'s prefix. The §3.2 allowlist declares no separate
/// constant for it, so the frame's own shape carries it (B4 report).
pub const CMD_PREFIX: &str = "/correct";

// `FUNCTION_WORDS`' table order, pinned byte-exact by `check_string_constants`.
const IS: usize = 3;
const ARE: usize = 4;
const WAS: usize = 5;
const WERE: usize = 6;
const WHAT: usize = 7;
const WHO: usize = 8;
const WHEN: usize = 9;
const WHERE: usize = 10;
const OF: usize = 11;
const MEANS: usize = 12;

const VERBS: [usize; 4] = [IS, ARE, WAS, WERE];
const IS_OR_WAS: [usize; 2] = [IS, WAS];
const WH_WORDS: [usize; 3] = [WHO, WHEN, WHERE];

/// The four frames a `/teach` left side may carry (09 §2.1, `check_teach_left_side`).
pub const QUESTION_FRAMES: [PatternId; 4] = [
    PatternId::PQuestA,
    PatternId::PQuestD,
    PatternId::PQuestW,
    PatternId::POfRel,
];

/// One decoded term handle in word order. `word` is the token's index in the
/// original sequence, so a render can put the owner's words back in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub word: u8,
    pub surface: String,
    pub key: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pattern {
    pub frame: PatternId,
    pub slots: Vec<Slot>,
    pub words: Vec<Token>,
    pub surface: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decoded {
    Pattern(Pattern),
    /// 04 §3 B.3: the longest decoded sub-slot set, or empty when no slot
    /// decoded. These are the owner's own tokens; naming a slot asserts nothing.
    Undecoded { named_dark: Vec<Slot>, words: Vec<Token>, surface: String },
}

impl Decoded {
    pub fn frame(&self) -> PatternId {
        match self {
            Decoded::Pattern(pattern) => pattern.frame,
            Decoded::Undecoded { .. } => PatternId::Undecoded,
        }
    }

    pub fn words(&self) -> &[Token] {
        match self {
            Decoded::Pattern(pattern) => &pattern.words,
            Decoded::Undecoded { words, .. } => words,
        }
    }

    pub fn slots(&self) -> &[Slot] {
        match self {
            Decoded::Pattern(pattern) => &pattern.slots,
            Decoded::Undecoded { named_dark, .. } => named_dark,
        }
    }

    pub fn keys(&self) -> Vec<String> {
        self.words().iter().map(|token| token.key.clone()).collect()
    }

    pub fn surface(&self) -> &str {
        match self {
            Decoded::Pattern(pattern) => &pattern.surface,
            Decoded::Undecoded { surface, .. } => surface,
        }
    }

    /// Docs/03 §3.2's four question shapes: the only teachable left sides.
    pub fn is_question(&self) -> bool {
        QUESTION_FRAMES.contains(&self.frame())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Anchor {
    /// Any of the given `FUNCTION_WORDS` positions, compared on `lower`.
    Any(&'static [usize]),
}

#[derive(Clone, Copy, Debug)]
struct Shape {
    frame: PatternId,
    len: usize,
    anchors: &'static [(usize, Anchor)],
    slots: &'static [usize],
}

/// Match order: questions first (`what is apple` is both a definition question
/// and an attribute shape), corrections and commands before all of them.
const SHAPES: [Shape; 6] = [
    Shape {
        frame: PatternId::PQuestA,
        len: 4,
        anchors: &[(0, Anchor::Any(&[WHAT])), (2, Anchor::Any(&VERBS))],
        slots: &[1, 3],
    },
    Shape {
        frame: PatternId::PQuestD,
        len: 3,
        anchors: &[(0, Anchor::Any(&[WHAT])), (1, Anchor::Any(&IS_OR_WAS))],
        slots: &[2],
    },
    Shape {
        frame: PatternId::PQuestW,
        len: 3,
        anchors: &[(0, Anchor::Any(&WH_WORDS)), (1, Anchor::Any(&VERBS))],
        slots: &[2],
    },
    Shape {
        frame: PatternId::POfRel,
        len: 3,
        anchors: &[(1, Anchor::Any(&[OF]))],
        slots: &[0, 2],
    },
    Shape {
        frame: PatternId::PDef,
        len: 3,
        anchors: &[(1, Anchor::Any(&[MEANS]))],
        slots: &[0, 2],
    },
    Shape {
        frame: PatternId::PAttrib,
        len: 3,
        anchors: &[(1, Anchor::Any(&VERBS))],
        slots: &[0, 2],
    },
];

pub fn decode(raw: &str) -> Decoded {
    let trimmed = raw.trim();
    match split_line(trimmed) {
        LineKind::Teach { left, .. } => {
            return match decode(&left) {
                Decoded::Pattern(pattern) => Decoded::Pattern(Pattern {
                    frame: PatternId::PTeach,
                    slots: pattern.slots,
                    words: pattern.words,
                    surface: left,
                }),
                Decoded::Undecoded { named_dark, words, .. } => {
                    Decoded::Undecoded { named_dark, words, surface: left }
                }
            };
        }
        LineKind::Sense => {}
    }
    if let Some(rest) = trimmed.strip_prefix(CMD_PREFIX) {
        if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
            // `/correction` is not the command marker; it is ordinary input.
            return undecoded(trimmed, tokenize(trimmed));
        }
        let words = tokenize(rest);
        if words.is_empty() {
            return undecoded(trimmed, words);
        }
        let slots = all_slots(&words);
        return Decoded::Pattern(Pattern {
            frame: PatternId::PCmdC,
            slots,
            words,
            surface: trimmed.to_string(),
        });
    }
    let words = tokenize(trimmed);
    if words.is_empty() {
        return undecoded("", words);
    }
    if let Some(slots) = correction(&words) {
        return Decoded::Pattern(Pattern {
            frame: PatternId::PCorrect,
            slots,
            words,
            surface: trimmed.to_string(),
        });
    }
    for shape in SHAPES {
        if let Some(slots) = match_shape(&shape, &words) {
            return Decoded::Pattern(Pattern {
                frame: shape.frame,
                slots,
                words,
                surface: trimmed.to_string(),
            });
        }
    }
    Decoded::Undecoded {
        named_dark: partial_slots(&words),
        words,
        surface: trimmed.to_string(),
    }
}

fn undecoded(surface: &str, words: Vec<Token>) -> Decoded {
    Decoded::Undecoded { named_dark: Vec::new(), words, surface: surface.to_string() }
}

fn correction(words: &[Token]) -> Option<Vec<Slot>> {
    for marker in si_core::strings::CORRECTION_MARKERS {
        let marker_words = tokenize(marker);
        if marker_words.is_empty() || marker_words.len() >= words.len() {
            continue;
        }
        let matched = marker_words
            .iter()
            .zip(words.iter())
            .all(|(marker, token)| marker.lower == token.lower);
        if !matched {
            continue;
        }
        return Some(
            words
                .iter()
                .enumerate()
                .skip(marker_words.len())
                .map(|(index, token)| slot(index, token))
                .collect(),
        );
    }
    None
}

fn match_shape(shape: &Shape, words: &[Token]) -> Option<Vec<Slot>> {
    if words.len() != shape.len {
        return None;
    }
    for (index, anchor) in shape.anchors {
        if !anchor_matches(*anchor, &words[*index]) {
            return None;
        }
    }
    Some(shape.slots.iter().map(|index| slot(*index, &words[*index])).collect())
}

fn anchor_matches(anchor: Anchor, token: &Token) -> bool {
    match anchor {
        Anchor::Any(indices) => {
            indices.iter().any(|index| token.lower == FUNCTION_WORDS[*index])
        }
    }
}

fn partial_slots(words: &[Token]) -> Vec<Slot> {
    let mut best: Vec<Slot> = Vec::new();
    for shape in SHAPES {
        let anchors_ok = shape.anchors.iter().all(|(index, anchor)| {
            words.get(*index).is_some_and(|token| anchor_matches(*anchor, token))
        });
        if !anchors_ok {
            continue;
        }
        let slots: Vec<Slot> = shape
            .slots
            .iter()
            .filter(|index| **index < words.len())
            .map(|index| slot(*index, &words[*index]))
            .collect();
        if slots.len() > best.len() {
            best = slots;
        }
    }
    best
}

/// `P_CMD_C` and `P_CORRECT` carry an utterance: every token is a slot.
fn all_slots(words: &[Token]) -> Vec<Slot> {
    words.iter().enumerate().map(|(index, token)| slot(index, token)).collect()
}

fn slot(index: usize, token: &Token) -> Slot {
    Slot {
        word: index.min(u8::MAX as usize) as u8,
        surface: token.surface.clone(),
        key: token.key.clone(),
    }
}
