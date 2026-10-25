//! 04 §21 U.2–U.3 / docs/09: the claim label, a pure function of the settled
//! thought and the cited rows. First match wins; the ladder is total (I6).
//!
//! Demote-only: there is no promote direction and no affective argument (OD-24).
//! `INFERENCE_MAX_HOPS_OWNED` is a **claim-side** cap: it never truncates the
//! walk, it decides whether the route may be called INFERENCE or falls through
//! to the uncertainty surface with the full route cited (04 §5.4).

use si_core::addr::{EDGE_KIND_COOCCUR_WEAK, EDGE_KIND_OWNED_LAST};
use si_core::consts;
use si_core::record::answerable_meta;
use si_read::{Cited, RouteCounts, StopReason, Thought};

use crate::render::Rendered;

/// The registry row the cap comes from. `si-session` may hand a different
/// `Caps` in (an ENGINE build, a sweep); the labeler never reads the walk.
pub const HOP_CAP_ROW: &str = "INFERENCE_MAX_HOPS_OWNED";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Caps {
    pub max_owned_hops: u32,
    /// ENGINE config only: genome (BASE) paths exist and may label INFERENCE.
    pub engine: bool,
}

impl Caps {
    /// The shipped default: the registry's value (a published sweep's, else the
    /// OD-86 placeholder) and CONFIG SI.
    pub fn registry() -> Self {
        let row = consts::row(HOP_CAP_ROW).expect("the hop cap is a registry row");
        let value = consts::registry_value(row).expect("the hop cap is a swept scalar SPECIES row");
        Self { max_owned_hops: value.clamp(1, u32::MAX as i64) as u32, engine: false }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClaimLabel {
    Fact,
    Inference,
    Speculation,
    DontKnow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flavor {
    Plain,
    /// Max-K, depth, or a demoted settle: the label is certain of its doubt.
    Uncertainty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceTag {
    Owned,
    Base,
    None,
}

impl SourceTag {
    pub fn surface(self) -> &'static str {
        match self {
            SourceTag::Owned => "OWNED",
            SourceTag::Base => "BASE",
            SourceTag::None => "NONE",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    SettledFact,
    Composed,
    BaseOnly,
    Depth,
    Novel,
    MaxK,
    NamedDark,
    NoSlot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labeled {
    pub label: ClaimLabel,
    pub flavor: Flavor,
    pub tag: SourceTag,
    pub citation: Vec<u32>,
    pub route_counts: RouteCounts,
    pub reason: Reason,
}

impl Labeled {
    /// The chat-panel surface. `UNCERTAINTY` is the uncertainty-flavored
    /// SPECULATION surface docs/09 names; the four labels are FACT, INFERENCE,
    /// SPECULATION and I-DON'T-KNOW.
    pub fn surface(&self) -> &'static str {
        match (self.label, self.flavor) {
            (ClaimLabel::Fact, _) => "FACT",
            (ClaimLabel::Inference, _) => "INFERENCE",
            (ClaimLabel::Speculation, Flavor::Uncertainty) => "UNCERTAINTY",
            (ClaimLabel::Speculation, Flavor::Plain) => "SPECULATION",
            (ClaimLabel::DontKnow, _) => "I-DON'T-KNOW",
        }
    }

    pub fn is_uncertain(&self) -> bool {
        self.flavor == Flavor::Uncertainty
    }
}

pub struct LabelInput<'a> {
    pub thought: &'a Thought,
    pub caps: &'a Caps,
    pub cited: &'a [Cited],
    pub rendered: &'a Rendered,
    /// The input decoded to one of the four question frames.
    pub question_shaped: bool,
    /// `si-read` named a dark slot from the owner's own tokens.
    pub named_dark: bool,
    pub route: RouteCounts,
}

pub fn label(input: &LabelInput<'_>) -> Labeled {
    let tag = tag_for(input.route);
    let settled = input.thought.stop_reason == StopReason::Settled;
    let claim_rows: Vec<&Cited> = input
        .cited
        .iter()
        .filter(|row| is_claim(row) && answerable_meta(row.confidence, row.kind, row.active_at_anchor))
        .collect();

    if settled {
        // FACT: every content token ∈ one cited OWNED row, and that row is a
        // live belief at the anchor. `answerable_meta` is the one cited-row gate.
        if input.route.owned >= 1 && input.rendered.containment {
            if let Some(row) = claim_rows.iter().find(|row| row.contains_all(&input.rendered.content)) {
                return Labeled {
                    label: ClaimLabel::Fact,
                    flavor: Flavor::Plain,
                    tag,
                    citation: vec![row.row_id],
                    route_counts: input.route,
                    reason: Reason::SettledFact,
                };
            }
        }
        // INFERENCE (a): OWNED, composed across ≥ 2 rows, all cited, no novel
        // tokens, and the route is inside the claim-side hop cap.
        if input.route.owned >= 1
            && claim_rows.len() >= 2
            && claim_rows.len() == input.cited.len()
            && input.rendered.containment
            && input.route.hops <= input.caps.max_owned_hops
        {
            return Labeled {
                label: ClaimLabel::Inference,
                flavor: Flavor::Plain,
                tag,
                citation: claim_ids(input.cited),
                route_counts: input.route,
                reason: Reason::Composed,
            };
        }
        // INFERENCE (b): genome (BASE) paths only, ENGINE config only.
        if input.caps.engine && input.route.owned == 0 && input.route.base > 0 {
            return Labeled {
                label: ClaimLabel::Inference,
                flavor: Flavor::Plain,
                tag,
                citation: claim_ids(input.cited),
                route_counts: input.route,
                reason: Reason::BaseOnly,
            };
        }
        // Settled but unclaimable: over the cap is `depth`, leaked content is a
        // novel token. Both are the uncertainty surface with the route cited.
        let reason = if input.route.hops > input.caps.max_owned_hops {
            Reason::Depth
        } else {
            Reason::Novel
        };
        return Labeled {
            label: ClaimLabel::Speculation,
            flavor: Flavor::Uncertainty,
            tag,
            citation: claim_ids(input.cited),
            route_counts: input.route,
            reason,
        };
    }

    // I-DON'T-KNOW with a name: the render is the ask-back (docs/09).
    if input.named_dark {
        return Labeled {
            label: ClaimLabel::DontKnow,
            flavor: Flavor::Plain,
            tag: SourceTag::None,
            citation: claim_ids(input.cited),
            route_counts: input.route,
            reason: Reason::NamedDark,
        };
    }
    // Max-K without settle: the uncertainty-flavored SPECULATION (docs/09).
    if input.thought.stop_reason == StopReason::MaxK {
        return Labeled {
            label: ClaimLabel::Speculation,
            flavor: Flavor::Uncertainty,
            tag,
            citation: claim_ids(input.cited),
            route_counts: input.route,
            reason: Reason::MaxK,
        };
    }
    // Unnamed dark + question shape + no OWNED route: the nonce guess.
    if input.question_shaped && input.route.owned == 0 {
        return Labeled {
            label: ClaimLabel::Speculation,
            flavor: Flavor::Plain,
            tag: SourceTag::None,
            citation: Vec::new(),
            route_counts: input.route,
            reason: Reason::Novel,
        };
    }
    Labeled {
        label: ClaimLabel::DontKnow,
        flavor: Flavor::Plain,
        tag: SourceTag::None,
        citation: claim_ids(input.cited),
        route_counts: input.route,
        reason: Reason::NoSlot,
    }
}

/// Docs/03 §3.2's claim kinds: 0..=5 are OWNED claim edges; `6 cooccur-weak`
/// and the frozen genome kinds are not claims.
fn is_claim(row: &Cited) -> bool {
    row.kind <= EDGE_KIND_OWNED_LAST && row.kind != EDGE_KIND_COOCCUR_WEAK
}

fn tag_for(route: RouteCounts) -> SourceTag {
    if route.owned > 0 {
        SourceTag::Owned
    } else if route.base > 0 {
        SourceTag::Base
    } else {
        SourceTag::None
    }
}

fn claim_ids(cited: &[Cited]) -> Vec<u32> {
    cited.iter().map(|row| row.row_id).collect()
}
