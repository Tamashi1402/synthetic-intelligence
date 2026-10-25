//! engineering/09 §2–§3: the owner's teach act. The channel owns no vocabulary:
//! the two markers are `TEACH_LITERALS`, the left side goes through the same
//! decoder that answers questions, and the right side is content, never a shape.
//!
//! Malformations are data, not errors (09 §2.1). Every outcome carries its row
//! count so a caller can assert "zero rows" without re-deriving it, and the
//! rejections carry the owner's own named sub-slots back to the gap queue.

use si_core::patterns::PatternId;
use si_core::strings::Source;

use crate::decode::{decode, Decoded, Pattern, Slot};

/// The accept set for a left side: the four question frames (09 §2.1 reads
/// docs/01 §1.4's "decoded question shape"). A statement has no slot to key.
pub const LEFT_ACCEPT_FRAMES: [PatternId; 4] = [
    PatternId::PQuestA,
    PatternId::PQuestD,
    PatternId::PQuestW,
    PatternId::POfRel,
];

const MARKER: usize = 0;
const BIND: usize = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineKind {
    Teach { left: String, right: String },
    Sense,
}

/// 04 §3 B.1 / 09 §2: a prefix test on the raw line, split at the **first**
/// `=>`. Whitespace around both halves is trimmed and nothing else is
/// rewritten, so a right side may contain the marker's characters.
pub fn split_line(raw: &str) -> LineKind {
    let trimmed = raw.trim();
    let marker = si_core::TEACH_LITERALS[MARKER];
    let rest = match trimmed.strip_prefix(marker) {
        Some(rest) if rest.is_empty() || rest.starts_with(char::is_whitespace) => rest.trim(),
        _ => return LineKind::Sense,
    };
    match rest.split_once(si_core::TEACH_LITERALS[BIND]) {
        Some((left, right)) => LineKind::Teach {
            left: left.trim().to_string(),
            right: right.trim().to_string(),
        },
        // 09 §2.1: no `=>` is not a `P_TEACH` match at all; the line is sense input.
        None => LineKind::Sense,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeachOutcome {
    Row,
    /// A `source: grimm` line: the shape is recognized and inert (G12(d)).
    Inert,
    /// The left side did not decode, or was empty (09 §2.1 rows 2 and 4).
    Undecoded,
    /// The left decoded; the right has no content to bind (09 §2.1 row 3).
    EmptyResponse,
    /// The left decoded to a statement shape; zero rows (09 §2.1 row 5).
    LeftNotAQuestion,
}

impl TeachOutcome {
    /// The reason string the owner sees. Codes only: the channel owns no
    /// content word (09 §2, R-18).
    pub fn reason(self) -> &'static str {
        match self {
            TeachOutcome::Row => "",
            TeachOutcome::Inert => "grimm_source_inert",
            TeachOutcome::Undecoded => "left_undecoded",
            TeachOutcome::EmptyResponse => "empty_response",
            TeachOutcome::LeftNotAQuestion => "left_not_a_question",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeachDecision {
    pub outcome: TeachOutcome,
    pub rows: u8,
    /// The named dark spot this act leaves behind: the left side's own sub-slots
    /// when it did not decode, or the response slot when the right side is empty.
    pub dark: Vec<Slot>,
    /// The left side's decoded pattern, when the act may commit (09 §4's row key).
    pub pattern: Option<Pattern>,
    /// The right side verbatim.
    pub response: String,
}

/// `None` means "not a teach line": the caller decodes it as ordinary sense.
pub fn decide(raw: &str, source: Source) -> Option<TeachDecision> {
    let (left, right) = match split_line(raw) {
        LineKind::Teach { left, right } => (left, right),
        LineKind::Sense => return None,
    };
    if source != Source::Owner {
        return Some(TeachDecision {
            outcome: TeachOutcome::Inert,
            rows: 0,
            dark: Vec::new(),
            pattern: None,
            response: right,
        });
    }
    match decode(&left) {
        Decoded::Undecoded { named_dark, .. } => Some(TeachDecision {
            outcome: TeachOutcome::Undecoded,
            rows: 0,
            dark: named_dark,
            pattern: None,
            response: right,
        }),
        Decoded::Pattern(pattern) => {
            if !LEFT_ACCEPT_FRAMES.contains(&pattern.frame) {
                return Some(TeachDecision {
                    outcome: TeachOutcome::LeftNotAQuestion,
                    rows: 0,
                    dark: Vec::new(),
                    pattern: None,
                    response: right,
                });
            }
            if right.is_empty() {
                return Some(TeachDecision {
                    outcome: TeachOutcome::EmptyResponse,
                    rows: 0,
                    dark: pattern.slots.first().cloned().into_iter().collect(),
                    pattern: None,
                    response: right,
                });
            }
            Some(TeachDecision {
                outcome: TeachOutcome::Row,
                rows: 1,
                dark: Vec::new(),
                pattern: Some(pattern),
                response: right,
            })
        }
    }
}
