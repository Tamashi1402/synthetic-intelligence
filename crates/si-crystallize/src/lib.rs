//! docs/22 / engineering/12 §29 — crystallize: experience may NOMINATE.
//!
//! A reader-only leaf (12 §1.1): `si-core` and `si-store` only, and *read-only*
//! against the store. There is no `si-learn` edge, so this crate has no symbol
//! through which a row, an edge, a bake or a confidence byte could move; its
//! output is a `Nomination` value the session turns into an ordinary drive gap.
//! There is no `si-affect` edge, so §22.3's "`a` is NEVER read by the detector"
//! is a missing crate edge rather than a sentence.
//!
//! The four rules of §22.3, in order:
//!
//! 1. `|M| ≥ 3` — a pair is an edge, not a concept.
//! 2. the shared-filler test: every member carries a live edge of a common
//!    relation kind, and this frame's own spread carried that kind as a filler.
//! 3. recurrence over **distinct cold frames** at least `DELTA_MIN_CRYSTALLIZE`
//!    apart (OD-105: a frame with any `primed` light contributes nothing).
//! 4. no live row already binds exactly this member set — else recurrence is
//!    that point's confidence leg (a write the session owns) and no candidate
//!    opens.
//!
//! Merging is by member-set hash, never by stacking (§22.7). Members are live
//! L3 points with a live row; a point bound by nothing is not a member, which is
//! §22.3's all-BASE discard read mechanically.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;

use si_core::addr::{SHELF_FACTS, SHELF_LINKS, SHELF_S_R_PAIRS};
use si_core::encode::Writer;
use si_core::frame::{FrameLog, Role};
use si_store::store::Lattice;

/// §22.3 rule 1: depth is earned.
pub const MEMBERS_MIN: usize = 3;
/// 12 §2.4: the `cluster-nomination` gap kind, a value in `key_field`'s high byte.
pub const GAP_CLUSTER_NOMINATION: u8 = 5;
/// 12 §2.4: `origin = 2` — the nomination lane.
pub const ASK_ORIGIN_NOMINATION: u8 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub r_min: u16,
    pub r_ask: u16,
    pub delta_min: i32,
}

impl Default for Config {
    /// The registry's own floors — the point every leg that needs a number runs
    /// at when its sweep is a refusal (OD-95's precedent).
    fn default() -> Self {
        Self { r_min: 1, r_ask: 1, delta_min: 1 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub members: Vec<u32>,
    pub shared_fillers: Vec<u16>,
    pub recurrence: u16,
    /// Distinct frame times, in arrival order. The rule counts *frames*, not
    /// calls: two thoughts at the same `t` are one frame time.
    pub frames_seen: Vec<i32>,
    pub first_seen_t: i32,
    pub last_seen_t: i32,
    pub nominated_at: Option<i32>,
}

/// A candidate that cleared `R_ASK_CRYSTALLIZE`. It carries no word: the session
/// renders it by enumerating the members' own surfaces (12 §9 row 9), and
/// `text_ref` is an **already interned** id so the detector can never leak a
/// string into the hashed interner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Nomination {
    pub member_hash: u64,
    pub members: Vec<u32>,
    pub shared_fillers: Vec<u16>,
    pub recurrence: u16,
    pub text_ref: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transition {
    /// A frame that carried a `primed` light: no recurrence (OD-105).
    WarmFrame,
    TooFewMembers,
    NoSharedFiller,
    /// A live row already binds this member set under one label (§22.3 rule 4).
    AlreadyBound,
    Opened,
    Recurred,
    /// Seen again, but inside `DELTA_MIN_CRYSTALLIZE`.
    InsideDelta,
    Nominated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub t: i32,
    pub transition: Transition,
    pub members: Vec<u32>,
    pub recurrence: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Observation {
    pub nominations: Vec<Nomination>,
    pub events: Vec<Event>,
    pub candidates: u32,
    pub frames_counted: u32,
}

#[derive(Clone, Debug, Default)]
pub struct Crystallizer {
    config: Config,
    table: BTreeMap<u64, Candidate>,
    frames: u32,
}

impl Crystallizer {
    pub fn new(config: Config) -> Self {
        Self { config, table: BTreeMap::new(), frames: 0 }
    }

    pub fn config(&self) -> Config {
        self.config
    }

    pub fn candidates(&self) -> u32 {
        self.table.len() as u32
    }

    pub fn table(&self) -> &BTreeMap<u64, Candidate> {
        &self.table
    }

    /// 12 §4.2: the frame's `CoLight` slice and a read-only store view. The
    /// arguments are the whole input — there is no `a`, and no other path in.
    pub fn observe(
        &mut self,
        frame: &FrameLog,
        store: &Lattice,
        now: i32,
    ) -> Result<Observation> {
        self.frames = self.frames.saturating_add(1);
        let mut out = Observation {
            candidates: self.candidates(),
            frames_counted: self.frames,
            ..Observation::default()
        };

        let mut members = frame.seed_points();
        members.sort_unstable();
        members.dedup();
        if !frame.is_cold() {
            out.events.push(Event {
                t: now,
                transition: Transition::WarmFrame,
                members,
                recurrence: 0,
            });
            return Ok(out);
        }

        let index = StoreIndex::build(store)?;
        let mut members: Vec<u32> = members
            .into_iter()
            .filter(|point| index.live.contains_key(point))
            .collect();
        // §22.3: contradiction pairs are excluded from member sets until resolved.
        members.retain(|point| !index.conflicted.contains(point));
        if members.len() < MEMBERS_MIN {
            out.events.push(Event {
                t: now,
                transition: Transition::TooFewMembers,
                members,
                recurrence: 0,
            });
            return Ok(out);
        }
        out.events.push(Event {
            t: now,
            transition: Transition::Opened,
            members: members.clone(),
            recurrence: 0,
        });
        let first = out.events.len() - 1;

        // Rule 2: a common relation kind on live rows, carried by this frame's
        // own spread. The frame log names the fillers a pulse reached; the rows
        // name the relations each member actually carries.
        let frame_kinds: BTreeSet<u16> = frame
            .lights
            .iter()
            .filter(|light| light.role == Role::SharedFiller)
            .map(|light| light.edge_kind)
            .collect();
        let shared: Vec<u16> = index
            .common_kinds(&members)
            .into_iter()
            .filter(|kind| frame_kinds.contains(kind))
            .collect();
        if shared.is_empty() {
            out.events[first].transition = Transition::NoSharedFiller;
            out.events[first].members = members;
            out.candidates = self.candidates();
            return Ok(out);
        }

        // Rule 4: an existing live row already binds exactly this member set.
        if index.bound_sets.contains(&members) {
            out.events[first].transition = Transition::AlreadyBound;
            out.events[first].members = members;
            out.candidates = self.candidates();
            return Ok(out);
        }

        let hash = member_set_hash(&members);
        let transition = match self.table.get_mut(&hash) {
            None => {
                self.table.insert(
                    hash,
                    Candidate {
                        members: members.clone(),
                        shared_fillers: shared,
                        recurrence: 1,
                        frames_seen: vec![now],
                        first_seen_t: now,
                        last_seen_t: now,
                        nominated_at: None,
                    },
                );
                Transition::Opened
            }
            Some(candidate) => {
                if now.saturating_sub(candidate.last_seen_t) >= self.config.delta_min.max(1) {
                    candidate.recurrence = candidate.recurrence.saturating_add(1);
                    candidate.frames_seen.push(now);
                    candidate.last_seen_t = now;
                    Transition::Recurred
                } else {
                    Transition::InsideDelta
                }
            }
        };

        // §22.3 rule 3's second bar, and §22.4's ask. One nomination per
        // candidate: the queue's dedupe window owns re-asks, and a detector that
        // re-emitted every tick would be a second queue rule.
        let mut nominated = false;
        if let Some(candidate) = self.table.get_mut(&hash) {
            let recurrence = candidate.recurrence;
            if transition != Transition::InsideDelta
                && candidate.nominated_at.is_none()
                && recurrence >= self.config.r_ask.max(1)
            {
                candidate.nominated_at = Some(now);
                out.nominations.push(Nomination {
                    member_hash: hash,
                    members: candidate.members.clone(),
                    shared_fillers: candidate.shared_fillers.clone(),
                    recurrence,
                    text_ref: index.surface_of(candidate.members[0]),
                });
                nominated = true;
            }
        }
        out.events[first].transition = transition;
        out.events[first].recurrence = self.table.get(&hash).map(|c| c.recurrence).unwrap_or(0);
        if nominated {
            out.events.push(Event {
                t: now,
                transition: Transition::Nominated,
                members: self.table.get(&hash).map(|c| c.members.clone()).unwrap_or_default(),
                recurrence: out.events[first].recurrence,
            });
        }
        out.candidates = self.candidates();
        Ok(out)
    }

    /// The candidate table as canonical bytes — the leg that compares a table
    /// under two `a` regimes compares this, never a serialization of chance.
    pub fn table_bytes(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u32(self.table.len() as u32);
        for (hash, candidate) in &self.table {
            w.u64(*hash);
            w.u32(candidate.members.len() as u32);
            for member in &candidate.members {
                w.u32(*member);
            }
            w.u32(candidate.shared_fillers.len() as u32);
            for kind in &candidate.shared_fillers {
                w.u16(*kind);
            }
            w.u16(candidate.recurrence);
            w.u32(candidate.frames_seen.len() as u32);
            for t in &candidate.frames_seen {
                w.i32(*t);
            }
            w.i32(candidate.first_seen_t);
            w.i32(candidate.last_seen_t);
            match candidate.nominated_at {
                Some(t) => {
                    w.u8(1);
                    w.i32(t);
                }
                None => {
                    w.u8(0);
                    w.i32(0);
                }
            }
        }
        w.finish()
    }
}

/// §22.3: recurrence counts *distinct* frames. FNV-1a over the ascending member
/// handles — integer-only, single-threaded, and a pure function of the set.
pub fn member_set_hash(members: &[u32]) -> u64 {
    let mut sorted = members.to_vec();
    sorted.sort_unstable();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for member in sorted {
        for byte in member.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

/// The read-only projection of the store one observation needs. Built once per
/// frame from `rows()`/`edges_of()`, never mutated.
struct StoreIndex {
    live: BTreeMap<u32, u32>,
    kinds: BTreeMap<u32, BTreeSet<u16>>,
    conflicted: BTreeSet<u32>,
    bound_sets: BTreeSet<Vec<u32>>,
}

impl StoreIndex {
    fn build(store: &Lattice) -> Result<Self> {
        let mut index = Self {
            live: BTreeMap::new(),
            kinds: BTreeMap::new(),
            conflicted: BTreeSet::new(),
            bound_sets: BTreeSet::new(),
        };
        let mut slots: BTreeMap<(u16, u32, u16), BTreeSet<u32>> = BTreeMap::new();
        for (_, record) in store.rows()? {
            if !record.envelope.is_live() {
                continue;
            }
            let shelf = record.envelope.shelf_x;
            if !matches!(shelf, SHELF_S_R_PAIRS | SHELF_FACTS | SHELF_LINKS) {
                continue;
            }
            let point = record.body.key_point;
            index.live.entry(point).or_insert(record.body.resp_surface);
            let kinds = index.kinds.entry(point).or_default();
            for edge in store.edges_of(&record)? {
                if edge.is_live() {
                    kinds.insert(u16::from(edge.kind));
                }
            }
            slots
                .entry((shelf, record.body.key_point, record.body.key_field))
                .or_default()
                .insert(record.body.val_point);
            if record.envelope.derived_len > 0 {
                let mut handles = store
                    .handles_at(record.envelope.derived_off, record.envelope.derived_len)?;
                handles.sort_unstable();
                handles.dedup();
                if handles.len() >= MEMBERS_MIN {
                    index.bound_sets.insert(handles);
                }
            }
        }
        // A slot carrying two live values is a contradiction still open (§8).
        for ((_, key_point, _), values) in slots {
            if values.len() > 1 {
                index.conflicted.insert(key_point);
            }
        }
        Ok(index)
    }

    fn common_kinds(&self, members: &[u32]) -> Vec<u16> {
        let mut common: Option<BTreeSet<u16>> = None;
        for member in members {
            let kinds = self.kinds.get(member).cloned().unwrap_or_default();
            common = Some(match common {
                None => kinds,
                Some(previous) => previous.intersection(&kinds).copied().collect(),
            });
        }
        common.map(|set| set.into_iter().collect()).unwrap_or_default()
    }

    /// An **already interned** surface id for the member (12 §9 row 9): the first
    /// live row's `resp_surface`, or `0` (01 §11's absent id) — never a new string.
    fn surface_of(&self, point: u32) -> u32 {
        self.live.get(&point).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
//! 11 §B14 DoD 1 / 12 §5 §29, docs/22 §22.3: the four rules, in order.
//!
//! Each leg is built so that the rule *before* the one under test is the only
//! one that could have denied it. That is what "in order" means as a check: a
//! fixture that fails two rules at once proves nothing about which ran first, so
//! the `NoSharedFiller` frame names a member set rule 4 would also deny, and the
//! `TooFewMembers` frame names a pair rule 2 would clear.
//!
//! The last test is OD-105's cold-frame rule read from the detector's side: a
//! frame carrying a `primed` light is a replay, so it moves no recurrence and no
//! `last_seen_t`, which is why the next cold frame still clears the delta.
//!
//! The frames are built here rather than in `crates/si-crystallize/tests/`
//! because `check_ghost_is_frame_scoped` allows a `CoLight` value to be
//! constructed only inside `si-walk`, and an integration test is not exempt the
//! way a `#[cfg(test)]` module is (12 §4.3 (xv)).

    use super::*;
    use si_core::addr::{EDGE_KIND_LINK_MEANS, EDGE_KIND_LINK_SLOT_UNIQUE};
    use si_core::frame::CoLight;
    use si_core::record::{Body, Envelope};
    use si_store::store::EdgeDraft;
    /// The three members rule 1 accepts as a concept, and the filler they share.
    const A: u32 = 10;
    const B: u32 = 11;
    const C: u32 = 12;
    const D: u32 = 13;
    const E: u32 = 14;
    const F: u32 = 15;
    const FILLER: u32 = 20;

    /// One member: a live shelf-4 row whose key point carries one live edge of
    /// `kind` to the filler. Rule 2 reads this row's edge; rule 1 reads its liveness.
    fn member_row(lattice: &mut Lattice, point: u32, kind: u8) {
        let mut draft = EdgeDraft::owned(FILLER, 4096, 0);
        draft.kind = kind;
        let mut body = Body::default();
        body.pattern_id = 1;
        body.key_point = point;
        body.val_point = FILLER;
        lattice
            .append_row(Envelope::live(SHELF_S_R_PAIRS, 0, 1, 0), body, vec![draft], &[], &[])
            .unwrap();
    }

    /// §22.3 rule 4's denial: a live row already binds exactly this member set.
    fn binding_row(lattice: &mut Lattice, members: &[u32]) {
        let mut body = Body::default();
        body.pattern_id = 1;
        body.key_point = 30;
        body.val_point = 31;
        lattice
            .append_row(Envelope::live(SHELF_FACTS, 0, 1, 0), body, Vec::new(), members, &[])
            .unwrap();
    }

    /// The frame the walk would hand the detector: the seeds round 0, then the
    /// filler this frame's spread actually reached, carrying `kind` in.
    fn frame(members: &[u32], kind_carried: u8, primed: bool) -> FrameLog {
        let mut log = FrameLog::default();
        for point in members {
            log.lights.push(CoLight {
                point: *point,
                role: Role::Seed,
                edge_kind: 0,
                round: 0,
                primed: false,
            });
        }
        log.lights.push(CoLight {
            point: FILLER,
            role: Role::SharedFiller,
            edge_kind: u16::from(kind_carried),
            round: 1,
            primed,
        });
        log
    }

    fn detector() -> Crystallizer {
        Crystallizer::new(Config { r_min: 1, r_ask: 2, delta_min: 4 })
    }

    fn transitions(observation: &Observation) -> Vec<Transition> {
        observation.events.iter().map(|event| event.transition).collect()
    }

    /// Rules 1, 2 and 4 each deny before the next is consulted, and the one frame
    /// that clears all three is the only one that opens a candidate.
    #[test]
    fn the_four_rules_fire_in_order_and_only_the_clearing_frame_opens() {
        let mut lattice = Lattice::new();
        for point in [A, B, C] {
            member_row(&mut lattice, point, EDGE_KIND_LINK_MEANS);
        }
        for point in [D, E, F] {
            member_row(&mut lattice, point, EDGE_KIND_LINK_MEANS);
        }
        // The set rule 4 guards is bound; {D, E, F} is not, so the last frame opens.
        binding_row(&mut lattice, &[A, B, C]);
        let mut detector = detector();

        // Rule 1 against a pair that rule 2 would clear and rule 4 would not deny.
        let first = detector.observe(&frame(&[A, B], EDGE_KIND_LINK_MEANS, false), &lattice, 1).unwrap();
        assert_eq!(transitions(&first), vec![Transition::TooFewMembers]);
        assert_eq!(first.candidates, 0, "a pair is an edge, not a concept");

        // Rule 2 against a set rule 4 also binds: the earlier rule answers.
        let second = detector.observe(&frame(&[A, B, C], EDGE_KIND_LINK_SLOT_UNIQUE, false), &lattice, 2).unwrap();
        assert_eq!(transitions(&second), vec![Transition::NoSharedFiller]);
        assert_eq!(second.candidates, 0);

        // Rule 4, once rule 2 is satisfied by this frame's own spread.
        let third = detector.observe(&frame(&[A, B, C], EDGE_KIND_LINK_MEANS, false), &lattice, 3).unwrap();
        assert_eq!(transitions(&third), vec![Transition::AlreadyBound]);
        assert_eq!(third.candidates, 0, "recurrence is that set's confidence leg, not a candidate");

        // All three cleared: the candidate opens at rule 3's first frame.
        let fourth = detector.observe(&frame(&[D, E, F], EDGE_KIND_LINK_MEANS, false), &lattice, 10).unwrap();
        assert_eq!(transitions(&fourth), vec![Transition::Opened]);
        assert_eq!(fourth.candidates, 1);
        let hash = member_set_hash(&[D, E, F]);
        let candidate = detector.table()[&hash].clone();
        assert_eq!(candidate.members, vec![D, E, F], "members are stored ascending");
        assert_eq!(candidate.shared_fillers, vec![u16::from(EDGE_KIND_LINK_MEANS)]);
        assert_eq!(candidate.recurrence, 1);

        // Rule 3's delta: inside it the frame counts as nothing.
        let inside = detector.observe(&frame(&[D, E, F], EDGE_KIND_LINK_MEANS, false), &lattice, 13).unwrap();
        assert_eq!(transitions(&inside), vec![Transition::InsideDelta]);
        assert_eq!(detector.table()[&member_set_hash(&[D, E, F])].recurrence, 1);

        // Across it, the recurrence — and `r_ask` is reached, so the ask is emitted.
        let across = detector.observe(&frame(&[D, E, F], EDGE_KIND_LINK_MEANS, false), &lattice, 14).unwrap();
        assert_eq!(
            transitions(&across),
            vec![Transition::Recurred, Transition::Nominated],
            "rule 3 clears and §22.4's bar is the same frame's news"
        );
        assert_eq!(across.nominations.len(), 1);
        assert_eq!(across.nominations[0].members, vec![D, E, F]);
        assert_eq!(across.nominations[0].recurrence, 2);
    }

    /// OD-105: a `primed` light is the re-light path's tag, so the frame is a replay
    /// of an experience already counted. It moves no recurrence and no `last_seen_t`,
    /// and the table's bytes do not change at all.
    #[test]
    fn a_primed_frame_increments_no_recurrence() {
        let mut lattice = Lattice::new();
        for point in [D, E, F] {
            member_row(&mut lattice, point, EDGE_KIND_LINK_MEANS);
        }
        let mut detector = detector();
        detector.observe(&frame(&[D, E, F], EDGE_KIND_LINK_MEANS, false), &lattice, 10).unwrap();
        let before = detector.table_bytes();

        let warm = detector.observe(&frame(&[D, E, F], EDGE_KIND_LINK_MEANS, true), &lattice, 20).unwrap();
        assert_eq!(transitions(&warm), vec![Transition::WarmFrame]);
        assert_eq!(warm.candidates, 1, "the warm frame neither opens nor closes anything");
        assert_eq!(detector.table_bytes(), before, "a primed frame leaves the table byte-identical");
        let candidate = &detector.table()[&member_set_hash(&[D, E, F])];
        assert_eq!(candidate.recurrence, 1);
        assert_eq!(candidate.last_seen_t, 10, "the replay is not the last cold frame");
        assert_eq!(candidate.frames_seen, vec![10]);

        // Because the replay moved nothing, the next cold frame still clears the
        // delta measured from 10 — and `r_ask` is reached on it, not one frame later.
        let cold = detector.observe(&frame(&[D, E, F], EDGE_KIND_LINK_MEANS, false), &lattice, 14).unwrap();
        assert_eq!(transitions(&cold), vec![Transition::Recurred, Transition::Nominated]);
        assert_eq!(cold.nominations.len(), 1);
    }

    /// §22.3 rule 3 counts distinct frames: two thoughts at one `t` are one sighting.
    #[test]
    fn recurrence_counts_frames_not_calls() {
        let mut lattice = Lattice::new();
        for point in [D, E, F] {
            member_row(&mut lattice, point, EDGE_KIND_LINK_MEANS);
        }
        let mut detector = detector();
        detector.observe(&frame(&[D, E, F], EDGE_KIND_LINK_MEANS, false), &lattice, 10).unwrap();
        for _ in 0..3 {
            let again = detector.observe(&frame(&[D, E, F], EDGE_KIND_LINK_MEANS, false), &lattice, 10).unwrap();
            assert_eq!(transitions(&again), vec![Transition::InsideDelta]);
        }
        let candidate = &detector.table()[&member_set_hash(&[D, E, F])];
        assert_eq!(candidate.recurrence, 1, "four calls, one frame time");
        assert_eq!(candidate.frames_seen, vec![10]);
        assert!(candidate.nominated_at.is_none());
    }

    /// 12 §2.4's one nomination per candidate: the queue's dedupe window owns
    /// re-asks, so a detector that re-emitted every cleared frame would be a second
    /// queue rule written in the wrong crate.
    #[test]
    fn a_candidate_nominates_exactly_once() {
        let mut lattice = Lattice::new();
        for point in [D, E, F] {
            member_row(&mut lattice, point, EDGE_KIND_LINK_MEANS);
        }
        let mut detector = detector();
        let mut emitted = 0usize;
        for step in 0..6 {
            let observation = detector
                .observe(&frame(&[D, E, F], EDGE_KIND_LINK_MEANS, false), &lattice, 10 + 4 * step)
                .unwrap();
            emitted += observation.nominations.len();
        }
        assert_eq!(emitted, 1, "six cleared frames, one nomination");
        let candidate = &detector.table()[&member_set_hash(&[D, E, F])];
        assert_eq!(candidate.recurrence, 6);
        assert_eq!(candidate.nominated_at, Some(14));
    }
}
