//! 04 §11 K.5 / docs/07 §7.4: the ask engine.
//!
//! An ask-back is a **copy** of the owner's own decoded words in word order:
//! the channel owns no template and no vocabulary of its own (§17.4's
//! anti-cheat). The supporter's line never produces an ask — the match is gated
//! on the owner (docs/21 §21.4).

use si_affect::drive::Ask;
use si_core::strings::Source;

use crate::retrieve::SlotKey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskBack {
    pub ask: Ask,
    /// The owner's words, joined in their original order.
    pub surface: String,
}

/// `None` for a `source: grimm` line or when the owner supplied no words: an
/// ask with nothing to address is not an utterance.
pub fn ask_back(slot: &SlotKey, owner_words: &[String], source: Source) -> Option<AskBack> {
    if source != Source::Owner || owner_words.is_empty() {
        return None;
    }
    Some(AskBack { ask: Ask { slot: slot.point }, surface: owner_words.join(" ") })
}
