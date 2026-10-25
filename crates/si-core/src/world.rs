//! 12 §2 / 04 §33 AE / docs/26 §26.2–§26.4 — the world family.
//!
//! This is the leaf the arc adds **no crate** for ([02 §17](../../../engineering/02-module-contracts.md)):
//! `si-front` and `si-learn` may not import each other, so the class function lives
//! where both can reach it. Everything here is derived and RAM-only: a speaker class
//! is a return value, never a row, a cache, an export field or a checkpoint
//! (OD-123's shipped half, docs/12 rule (ix)).

/// docs/26 §26.2's four words, printed in reports only. `Display` is the literal,
/// so a log line that names a class names the tier's word and nothing else.
impl std::fmt::Display for SpeakerClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.literal())
    }
}

/// The identity carried by a sense frame (`speaker_identity u64le`, 01 §9). It is
/// transport metadata, never content.
pub type Identity = u64;

/// 12 §2.1: the identity a line without a transport byte stands for is its
/// literal's own index — `owner` 0, `grimm` 1 (01 §3.2's two voices). This is the
/// birth registration, and the reason a pre-B18 scenario's class is unchanged.
pub const IDENTITY_OWNER: Identity = 0;
pub const IDENTITY_GRIMM: Identity = 1;

/// docs/26 §26.2: the four classes. The order is the arbitration tier of
/// 04 §33 AE.3 (`Owner > Trusted > Stranger-sense`); `Grimm` is a *registered*
/// voice, not a tier — it is grimm-class in the birth state and never teaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpeakerClass {
    Stranger,
    Grimm,
    Trusted,
    Owner,
}

impl SpeakerClass {
    /// The literal docs/26 §26.2 puts in the report. Not vocabulary: this module
    /// ships no word that reaches a row, an output or the interner.
    pub fn literal(self) -> &'static str {
        match self {
            SpeakerClass::Stranger => "stranger",
            SpeakerClass::Grimm => "grimm",
            SpeakerClass::Trusted => "trusted",
            SpeakerClass::Owner => "owner",
        }
    }

    /// 04 §33 AE.3: only the owner tier may write a belief row. `Trusted` is the
    /// class the refusal arm exists for — OD-135 gives it no honest
    /// `prov_speaker` value, so the class is derived and the write refused.
    pub fn may_write_belief(self) -> bool {
        self == SpeakerClass::Owner
    }
}

/// docs/26 §26.3(b): the smallest-right-first menu of a `/trust` line. Declared
/// here and granted by no path in this build (OD-125: *coded, not granted*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Right {
    Teach,
    Correct,
    Answer,
}

impl Right {
    pub fn from_literal(text: &str) -> Option<Self> {
        match text {
            "teach" => Some(Right::Teach),
            "correct" => Some(Right::Correct),
            "answer" => Some(Right::Answer),
            _ => None,
        }
    }

    pub fn literal(self) -> &'static str {
        match self {
            Right::Teach => "teach",
            Right::Correct => "correct",
            Right::Answer => "answer",
        }
    }
}

/// 04 §33 AE.1's `grant` row type: declared, owner-lane-only, append-only, and
/// **never hashed** (12 §2.6's session-RAM carriers). A revocation is a second
/// row, not an edit — I4 keeps grants and revocations rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GrantRow {
    pub subject: Identity,
    pub right: Right,
    /// The owner act the grant traces to (docs/26 §26.1: every act of authority
    /// traces to an owner act or to a grant that traces to one).
    pub granted_by: Identity,
    pub granted_at_t: i32,
    /// `None` while live. A revocation appends a row that sets this; it never
    /// removes the grant row.
    pub revoked_at_t: Option<i32>,
}

impl GrantRow {
    /// AE.1's `live_at(now)`: granted at or before `now`, not revoked at or before
    /// `now`. A revoked grant has no arbitration standing against a later stranger
    /// (docs/26 OD-125's recommended reading).
    pub fn live_at(&self, now: i32) -> bool {
        let granted = self.granted_at_t <= now;
        let unrevoked = match self.revoked_at_t {
            Some(revoked) => now < revoked,
            None => true,
        };
        granted && unrevoked
    }
}

/// The grant view AE.1's third argument. `now` travels with it so `class` stays a
/// pure function of three arguments.
#[derive(Clone, Debug, Default)]
pub struct Grants {
    rows: Vec<GrantRow>,
    grimm_identity: Identity,
    now: i32,
}

impl Grants {
    /// OD-125's birth state: the empty view, whose only known identity is grimm's
    /// registered id. `class` never returns `Trusted` over it.
    pub fn birth(grimm_identity: Identity) -> Self {
        Self { rows: Vec::new(), grimm_identity, now: 0 }
    }

    pub fn with_now(mut self, now: i32) -> Self {
        self.now = now;
        self
    }

    /// The clock the `live_at(now)` reads are taken against. One assignment per
    /// tick, because a grant's liveness is decided at the tick that consults it.
    pub fn set_now(&mut self, now: i32) {
        self.now = now;
    }

    /// The owner-lane-only writer. There is no other path to this function in the
    /// build: no `/trust` parser, no adapter, no transport mints a grant
    /// (12 §11's refusals), so a stranger-class line cannot reach it.
    pub fn grant(&mut self, row: GrantRow) {
        self.rows.push(row);
    }

    /// The revocation is an appended row, so the grant stays citable.
    pub fn revoke(&mut self, subject: Identity, right: Right, now: i32) {
        for row in self.rows.iter_mut()
            .filter(|row| row.subject == subject && row.right == right)
        {
            if row.live_at(now) {
                row.revoked_at_t = Some(now);
            }
        }
    }

    pub fn rows(&self) -> &[GrantRow] {
        &self.rows
    }

    pub fn grimm_identity(&self) -> Identity {
        self.grimm_identity
    }

    pub fn now(&self) -> i32 {
        self.now
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    fn find(&self, subject: Identity) -> Option<&GrantRow> {
        self.rows
            .iter()
            .find(|row| row.subject == subject && row.live_at(self.now))
    }
}

/// 04 §33 AE.1 / 02 §17: the class function. Four arms, first match wins, total
/// over every `u64` and every grant view.
pub fn class(identity: Identity, owner_identity: Identity, grants: &Grants) -> SpeakerClass {
    if identity == owner_identity {
        return SpeakerClass::Owner;
    }
    if identity == grants.grimm_identity() {
        return SpeakerClass::Grimm;
    }
    if grants.find(identity).is_some() {
        return SpeakerClass::Trusted;
    }
    SpeakerClass::Stranger
}

/// docs/26 §26.4, OD-126: a declared idle window. It appears **only** as a deny
/// conjunct on step 12's quiet-and-empty-agenda predicate — there is no path from
/// a window to `ignite`, `sleep::due`, `clock.advance` or a frame read
/// (`check_windows_suppress_only`). An open window does not make a tick happen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub open_from_t: i32,
    pub open_through_t: i32,
}

impl Window {
    /// `true` means the window denies the idle tick. Closed ⇒ deny, whatever else
    /// the quiet law says; open ⇒ this conjunct adds nothing.
    pub fn denies(&self, now: i32) -> bool {
        !(self.open_from_t <= now && now <= self.open_through_t)
    }
}

/// 04 §33 AE.3's one new arbitration axis. Returns `Some(winner)` only when the
/// two sides are strictly comparable; `None` means the existing §15 tiers decide.
pub fn tier_outranks(a: SpeakerClass, b: SpeakerClass) -> Option<bool> {
    (a != b).then_some(a > b)
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: Identity = 7;
    const GRIMM: Identity = 1;

    fn row(subject: Identity, right: Right, at: i32) -> GrantRow {
        GrantRow { subject, right, granted_by: OWNER, granted_at_t: at, revoked_at_t: None }
    }

    #[test]
    fn class_is_total_and_pure() {
        let birth = Grants::birth(GRIMM);
        assert_eq!(class(OWNER, OWNER, &birth), SpeakerClass::Owner);
        assert_eq!(class(GRIMM, OWNER, &birth), SpeakerClass::Grimm);
        for identity in [0u64, 2, 3, 99, u64::MAX] {
            assert_eq!(class(identity, OWNER, &birth), SpeakerClass::Stranger);
        }
        // The same arguments give the same answer; calling it changes nothing.
        let again = class(42, OWNER, &birth);
        assert_eq!(again, class(42, OWNER, &birth));
    }

    #[test]
    fn empty_grants_never_returns_trusted() {
        let birth = Grants::birth(GRIMM);
        assert!(birth.is_empty());
        for identity in 0..64u64 {
            assert_ne!(class(identity, OWNER, &birth), SpeakerClass::Trusted);
        }
    }

    #[test]
    fn a_grant_is_the_only_path_to_trusted_and_revocation_ends_it() {
        let mut grants = Grants::birth(GRIMM);
        grants.grant(row(5, Right::Answer, 10));
        let at = grants.clone().with_now(12);
        assert_eq!(class(5, OWNER, &at), SpeakerClass::Trusted);
        // before it was granted, the same identity was a stranger
        assert_eq!(class(5, OWNER, &grants.clone().with_now(9)), SpeakerClass::Stranger);
        grants.revoke(5, Right::Answer, 12);
        let after = grants.with_now(20);
        assert_eq!(class(5, OWNER, &after), SpeakerClass::Stranger);
        // the revoked grant is still a citable row: nothing was erased
        assert_eq!(after.rows().len(), 1);
    }

    #[test]
    fn owner_beats_its_own_registered_names() {
        // If the owner's identity *is* grimm's id, the first arm wins: the class
        // function never demotes the owner.
        let grants = Grants::birth(OWNER);
        assert_eq!(class(OWNER, OWNER, &grants), SpeakerClass::Owner);
    }

    #[test]
    fn tiers_are_strict_and_grimm_is_not_a_tier_above_trusted() {
        assert_eq!(tier_outranks(SpeakerClass::Owner, SpeakerClass::Trusted), Some(true));
        assert_eq!(tier_outranks(SpeakerClass::Trusted, SpeakerClass::Owner), Some(false));
        assert_eq!(tier_outranks(SpeakerClass::Stranger, SpeakerClass::Stranger), None);
        assert!(SpeakerClass::Trusted > SpeakerClass::Grimm);
        assert!(SpeakerClass::Owner.may_write_belief());
        assert!(!SpeakerClass::Trusted.may_write_belief());
        assert!(!SpeakerClass::Stranger.may_write_belief());
    }

    #[test]
    fn a_window_only_denies() {
        let w = Window { open_from_t: 100, open_through_t: 200 };
        assert!(w.denies(99));
        assert!(!w.denies(100));
        assert!(!w.denies(200));
        assert!(w.denies(201));
    }
}
