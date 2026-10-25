//! docs/25 / engineering/12 §5 §32, [04 §32 AD](../../../engineering/04-algorithms.md)
//! — skills: a shape detector that may nominate.
//!
//! The exact twin of `si-crystallize` applied to *shape* instead of *content*,
//! and the twin's independent deletion test (docs/25 §25.11 note 2). The edge
//! list is the contract: `si-core`, a read-only `si-store`, `anyhow`. There is
//! no `si-learn` edge, so "executing a skill" has no name in this build — a
//! skill is a row, not a program. There is no `si-affect` edge, so [docs/25
//! §25.4](../../../docs/25-skills.md)'s rule 3 ("the detector never reads `a`")
//! is a missing crate, not a sentence. There is no `si-walk` edge either: the
//! ordering preference this crate computes is an argument to the walk, never a
//! call into it (OD-118's only buildable half).
//!
//! The two bars, in [04 §32 AD.3](../../../engineering/04-algorithms.md)'s
//! order:
//!
//! 1. **generality** — the shape must have been instantiated over `K_FILLER`
//!    distinct filler sets. It runs first because it is the bar that decides
//!    *which detector owns the frame*: a shape whose recurrences all share one
//!    filler set is a content cluster, so this frame is dropped from the shape
//!    table's candidate count and **forwarded** (AD.4), and the session keeps
//!    feeding the same frame to §29's fold.
//! 2. **recurrence** — `R_MIN_SHAPE` distinct **cold** frames at
//!    `DELTA_MIN_SHAPE` apart. The OD-105 cold-frame rule is §29's, with the
//!    same meaning: a frame carrying any `primed` light counts nothing.
//!
//! Nothing here interns, weights, bakes or emits. A `Nomination` is a value; the
//! session turns it into an ordinary `questions` row on the ordinary queue
//! (law 29), and only an owner's answer to that ask becomes a row (AD.6).

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;

use si_core::addr::SHELF_SKILLS;
use si_core::encode::Writer;
use si_core::eventlog::Event;
use si_core::frame::{FrameLog, Role, RoleGraph};
use si_store::store::Lattice;

/// 12 §2.4: the `move-nomination` gap kind — `key_field`'s high byte.
pub const GAP_MOVE_NOMINATION: u8 = 6;
/// 12 §2.4: `origin = 2` — the nomination lane. Same byte as §29's; one value,
/// two crates, because neither may import the other (OD-119).
pub const ASK_ORIGIN_NOMINATION: u8 = 2;
/// AD.5's closed outcome set. There is no fifth value.
pub const TAG_QUESTION_RESOLVED: u8 = 0;
pub const TAG_REALIZED_GAIN: u8 = 1;
pub const TAG_OWNER_REPLY: u8 = 2;
pub const TAG_NONE: u8 = 3;
/// 12 §2.6: the candidate record's `outcome_tags: [u8; ≤4]` cap — the size of
/// the closed set, so a tag list can never grow into a second vocabulary.
pub const OUTCOME_TAG_MAX: usize = 4;

/// The two eventlog readings AD.5 allows, stated as the rows they are: the
/// detector reads **rows**, and the kinds are the loop's own (04 §6, 02 §11).
const ROW_SENSE_ECHO: u8 = 0;
const ROW_SIGNAL: u8 = 5;
const ROW_COMMIT: u8 = 13;
const ROW_SUPERSEDE: u8 = 14;
/// 02 §11's `source_byte`: `0` is the owner's line. A reply is a sense echo
/// that arrived on the owner's channel.
const SOURCE_OWNER: u8 = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub k_filler: u16,
    pub r_min_shape: u16,
    pub delta_min_shape: i32,
}

impl Default for Config {
    /// The registry's own floors (`K_FILLER` 2, `R_MIN_SHAPE` 1,
    /// `DELTA_MIN_SHAPE` 1): the point every leg that needs a number runs at
    /// while OD-116's sweep is a refusal (OD-95's precedent).
    fn default() -> Self {
        Self { k_filler: 2, r_min_shape: 1, delta_min_shape: 1 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    /// The folded template's id — 12 §2.6's `role_graph_template u32`, and the
    /// table's key. The template **text** travels in `Nomination` and in session
    /// RAM: the mint needs the string an id cannot yield.
    pub template: u32,
    pub template_text: String,
    /// AD.1's root role point: the seed role's most-recently-lit member, which
    /// is what makes a skill addressable in the walk's handle space (12 §2.3).
    pub root_point: u32,
    /// 12 §2.6's `filler_sets_seen`: the count of *distinct* filler sets this
    /// shape has been instantiated over. The set itself is kept because the
    /// count is only ever derived from it.
    pub filler_set_ids: BTreeSet<u64>,
    pub recurrence: u16,
    /// Distinct frame times AD.3's bar 2 counted, in arrival order.
    pub frames_seen: Vec<i32>,
    pub outcome_tags: Vec<u8>,
    pub first_seen_t: i32,
    pub last_seen_t: i32,
    /// Bar 1's verdict for this shape. A shape that has never cleared
    /// `K_FILLER` is a forwarded cluster, not a candidate (AD.4), so it cannot
    /// nominate and it is not in `Observation::candidates`.
    pub opened: bool,
    pub nominated_at: Option<i32>,
}

impl Candidate {
    pub fn filler_sets_seen(&self) -> u16 {
        self.filler_set_ids.len().min(u16::MAX as usize) as u16
    }
}

/// A shape that cleared both bars. It carries no word the owner has not taught:
/// `template_text` is the fold of the walk's own role labels and edge kinds,
/// and `root_point` is an existing handle — the session interns nothing until
/// an owner answers (AD.2, and 05 §23's byte-identical-interner leg).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Nomination {
    pub template: u32,
    pub template_text: String,
    pub root_point: u32,
    pub recurrence: u16,
    pub filler_sets_seen: u16,
    pub outcome_tags: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transition {
    /// OD-105: the frame carried a `primed` light, so it counts nothing.
    WarmFrame,
    /// `SHAPE_MAX = 0`, or a frame that built no scratch: over bound the frame
    /// builds none, never an eviction game (OD-99).
    NoScratch,
    /// AD.4: below `K_FILLER` — dropped from the candidate count and forwarded.
    Forwarded,
    /// Bar 1 passed for the first time on this frame.
    Opened,
    Recurred,
    /// Seen again, inside `DELTA_MIN_SHAPE`.
    InsideDelta,
    /// Both bars cleared and the ask not yet asked: exactly one per shape.
    Nominated,
    /// A skill row for this shape already exists (docs/25 §25.4 rule 4), so no
    /// candidate opens and its confidence leg is the session's to walk.
    AlreadyMinted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapeEvent {
    pub t: i32,
    pub transition: Transition,
    pub template: u32,
    pub recurrence: u16,
    pub filler_sets_seen: u16,
}

/// 02 §16's `ObserveReport`, plus the per-transition log the session writes to
/// `skills.jsonl` (OD-117: session-only, declared hash-exempt).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ObserveReport {
    pub nominations: Vec<Nomination>,
    pub events: Vec<ShapeEvent>,
    /// Shapes that cleared bar 1.
    pub candidates: u32,
    /// Distinct shapes the table has ever seen, forwarded ones included.
    pub shapes: u32,
    /// Frames dropped by bar 1, cumulative — the forwarding count AD.4 needs.
    pub forwarded: u32,
}

#[derive(Clone, Debug, Default)]
pub struct Skiller {
    config: Config,
    table: BTreeMap<u32, Candidate>,
    /// docs/25 §25.4 rule 4's "a skill row for the same role-graph does not
    /// already exist", as a value the session hands in. There is no store query
    /// here because the mint is not this crate's to make.
    minted: BTreeSet<u32>,
    forwarded: u32,
    frames: u32,
}

impl Skiller {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            table: BTreeMap::new(),
            minted: BTreeSet::new(),
            forwarded: 0,
            frames: 0,
        }
    }

    pub fn config(&self) -> Config {
        self.config
    }

    pub fn table(&self) -> &BTreeMap<u32, Candidate> {
        &self.table
    }

    pub fn opened_candidates(&self) -> u32 {
        self.table.values().filter(|candidate| candidate.opened).count() as u32
    }

    pub fn shapes(&self) -> u32 {
        self.table.len() as u32
    }

    pub fn forwarded_frames(&self) -> u32 {
        self.forwarded
    }

    /// Rule 4, from the session's own mint: a shape that already has a row never
    /// opens a candidate again. Replay re-derives it by replaying the mint.
    pub fn note_minted(&mut self, template: u32) {
        self.minted.insert(template);
    }

    /// 12 §4.1 step 7b. The frame's role graphs, the citable eventlog rows and
    /// the tick — and nothing else. `events` is filtered to `t <= now` inside so
    /// a live tick and a replay of the recorded trace see the same rows.
    pub fn observe(
        &mut self,
        frame: &FrameLog,
        events: &[Event],
        now: i32,
    ) -> Result<ObserveReport> {
        self.frames = self.frames.saturating_add(1);
        let mut out = ObserveReport {
            candidates: self.opened_candidates(),
            shapes: self.shapes(),
            forwarded: self.forwarded,
            ..ObserveReport::default()
        };
        let Some(graph) = frame.role_graphs.first().cloned() else {
            out.events.push(shape_event(now, Transition::NoScratch, 0, 0, 0));
            return Ok(out);
        };
        if !frame.is_cold() {
            out.events.push(shape_event(now, Transition::WarmFrame, 0, 0, 0));
            return Ok(out);
        }

        let template_text = fold(&graph);
        let template = template_id(&template_text);
        let Some(root_point) = root_point(frame) else {
            out.events.push(shape_event(now, Transition::NoScratch, template, 0, 0));
            return Ok(out);
        };

        if self.minted.contains(&template) {
            out.events.push(shape_event(now, Transition::AlreadyMinted, template, 0, 1));
            return Ok(out);
        }

        let filler_ids = filler_set_ids(frame);
        let outcome = outcome_tag(events, now);
        let k_filler = self.config.k_filler.max(1);
        let r_min_shape = self.config.r_min_shape.max(1);
        let delta_min_shape = self.config.delta_min_shape.max(1);

        // Everything that touches the table happens inside this scope. The
        // borrow it holds is why the counters below and the log entries after it
        // are read back as locals instead of through `&self`.
        let (transition, recurrence, filler_sets_seen, nomination, forwarded_once) = {
            let candidate = self.table.entry(template).or_insert_with(|| Candidate {
                template,
                template_text: template_text.clone(),
                root_point,
                filler_set_ids: BTreeSet::new(),
                recurrence: 0,
                frames_seen: Vec::new(),
                outcome_tags: Vec::new(),
                first_seen_t: now,
                last_seen_t: now,
                opened: false,
                nominated_at: None,
            });
            candidate.filler_set_ids.extend(filler_ids);
            if candidate.outcome_tags.len() < OUTCOME_TAG_MAX
                && !candidate.outcome_tags.contains(&outcome)
            {
                candidate.outcome_tags.push(outcome);
            }

            // Bar 1, first: it decides which detector owns this frame.
            if candidate.filler_sets_seen() < k_filler {
                candidate.opened = false;
                (
                    Transition::Forwarded,
                    candidate.recurrence,
                    candidate.filler_sets_seen(),
                    None,
                    true,
                )
            } else {
                // Bar 2, second, and only once bar 1 has passed.
                let bar2 = if !candidate.opened {
                    candidate.opened = true;
                    candidate.recurrence = 1;
                    candidate.frames_seen.push(now);
                    candidate.first_seen_t = now;
                    candidate.last_seen_t = now;
                    Transition::Opened
                } else if now.saturating_sub(candidate.last_seen_t) >= delta_min_shape {
                    candidate.recurrence = candidate.recurrence.saturating_add(1);
                    candidate.frames_seen.push(now);
                    candidate.last_seen_t = now;
                    Transition::Recurred
                } else {
                    Transition::InsideDelta
                };
                // One ask per shape, as in §29: the queue's own dedupe window
                // owns a re-ask, and a detector that re-emitted every tick would
                // be a second queue rule living in the wrong crate.
                let mut nomination = None;
                if bar2 != Transition::InsideDelta && candidate.nominated_at.is_none() {
                    if candidate.recurrence >= r_min_shape {
                        candidate.nominated_at = Some(now);
                        nomination = Some(Nomination {
                            template: candidate.template,
                            template_text: candidate.template_text.clone(),
                            root_point: candidate.root_point,
                            recurrence: candidate.recurrence,
                            filler_sets_seen: candidate.filler_sets_seen(),
                            outcome_tags: candidate.outcome_tags.clone(),
                        });
                    }
                }
                (bar2, candidate.recurrence, candidate.filler_sets_seen(), nomination, false)
            }
        };
        if forwarded_once {
            self.forwarded = self.forwarded.saturating_add(1);
        }

        out.events.push(shape_event(now, transition, template, recurrence, filler_sets_seen));
        if let Some(nomination) = nomination {
            out.events.push(shape_event(
                now,
                Transition::Nominated,
                template,
                nomination.recurrence,
                nomination.filler_sets_seen,
            ));
            out.nominations.push(nomination);
        }
        out.candidates = self.opened_candidates();
        out.shapes = self.shapes();
        out.forwarded = self.forwarded;
        Ok(out)
    }

    /// The candidate table as canonical bytes. G18(c) compares this under two
    /// `a` regimes; the signature above takes no arousal argument at all, which
    /// is the leg's whole claim.
    pub fn table_bytes(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u32(self.frames);
        w.u32(self.forwarded);
        w.u32(self.table.len() as u32);
        for candidate in self.table.values() {
            w.u32(candidate.template);
            w.str(&candidate.template_text);
            w.u32(candidate.root_point);
            w.u32(candidate.filler_set_ids.len() as u32);
            for id in &candidate.filler_set_ids {
                w.u64(*id);
            }
            w.u16(candidate.recurrence);
            w.u32(candidate.frames_seen.len() as u32);
            for t in &candidate.frames_seen {
                w.i32(*t);
            }
            w.u32(candidate.outcome_tags.len() as u32);
            for tag in &candidate.outcome_tags {
                w.u8(*tag);
            }
            w.i32(candidate.first_seen_t);
            w.i32(candidate.last_seen_t);
            w.u8(candidate.opened as u8);
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
        w.u32(self.minted.len() as u32);
        for template in &self.minted {
            w.u32(*template);
        }
        w.finish()
    }
}

/// AD.2's canonical fold: the roles, ascending, `|`-joined, then `:` and the
/// kinds actually traversed, ascending, `+`-joined — `seed|lit_a|shared_filler:2`.
/// Filler *identity* is deliberately absent: the same roles with different
/// fillers are the same shape (docs/25 §25.3), and the same fillers with
/// different roles are no shape at all.
pub fn fold(graph: &RoleGraph) -> String {
    let roles: Vec<&str> = graph.roles.iter().copied().filter_map(role_label).collect();
    let kinds: Vec<String> = graph.traversed_kinds.iter().map(|kind| kind.to_string()).collect();
    format!("{}:{}", roles.join("|"), kinds.join("+"))
}

/// The folded string's id — 12 §2.6's `role_graph_template u32`. FNV-1a over the
/// template text: integer-only, and a pure function of the string.
pub fn template_id(text: &str) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in text.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_01b3);
    }
    hash
}

fn role_label(byte: u8) -> Option<&'static str> {
    match byte {
        0 => Some(Role::Seed.label()),
        1 => Some(Role::LitA.label()),
        2 => Some(Role::LitB.label()),
        3 => Some(Role::SharedFiller.label()),
        4 => Some(Role::Ghost.label()),
        _ => None,
    }
}

/// AD.1's root role: "the seed role's most-recently-lit member". Every seed
/// lights in round 0, so the log's own order is the recency order and the last
/// `Seed` entry in it is the answer. `None` is a frame with no seed at all.
pub fn root_point(frame: &FrameLog) -> Option<u32> {
    frame
        .lights
        .iter()
        .rev()
        .find(|light| light.role == Role::Seed)
        .map(|light| light.point)
}

/// The frame's filler sets, one id per round that lit fillers: FNV-1a over the
/// round's ascending filler handles. Two rounds of the same points are one set;
/// the same shape over two different sets is the generality bar's evidence.
pub fn filler_set_ids(frame: &FrameLog) -> BTreeSet<u64> {
    let mut rounds: BTreeMap<u8, Vec<u32>> = BTreeMap::new();
    for light in &frame.lights {
        if light.role == Role::SharedFiller {
            rounds.entry(light.round).or_default().push(light.point);
        }
    }
    rounds
        .values()
        .map(|points| {
            let mut sorted = points.clone();
            sorted.sort_unstable();
            sorted.dedup();
            let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
            for point in sorted {
                for byte in point.to_le_bytes() {
                    hash ^= u64::from(byte);
                    hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
            hash
        })
        .collect()
}

/// AD.5: the outcome is read from the eventlog rows and nowhere else — never
/// `a`, never a label. One tag per recurrence frame, the closed set's own order
/// as the precedence.
pub fn outcome_tag(events: &[Event], now: i32) -> u8 {
    let mut owner_reply = false;
    for event in events.iter().filter(|event| event.t <= now) {
        let body = String::from_utf8_lossy(&event.body);
        match event.kind {
            ROW_SUPERSEDE | ROW_COMMIT if body.contains("\"resolve\"") => {
                return TAG_QUESTION_RESOLVED;
            }
            ROW_SIGNAL if body.contains("realized_gain") => return TAG_REALIZED_GAIN,
            ROW_SENSE_ECHO if event.source == SOURCE_OWNER => owner_reply = true,
            _ => {}
        }
    }
    if owner_reply {
        TAG_OWNER_REPLY
    } else {
        TAG_NONE
    }
}

/// AD.5's only use of a skill row: the walk's candidate-iteration preference.
/// Live shelf-9 rows, in row order, as their root handles. `SKILL_COMPOSITION =
/// off` never calls this, and the walk's empty hint is bit-identical to B13's
/// arithmetic — which is what G18(b)'s second half measures.
///
/// A retired row (`status = 1`) is not a preference: supersede keeps its bytes
/// citable (I4) and keeps it out of the walk.
pub fn composition_preference(store: &Lattice, now: i32) -> Result<Vec<u32>> {
    let mut hint = Vec::new();
    for (_, record) in store.active_at(SHELF_SKILLS, now)? {
        if !record.envelope.is_live() || record.body.status != 0 {
            continue;
        }
        if record.body.key_point != 0 && !hint.contains(&record.body.key_point) {
            hint.push(record.body.key_point);
        }
    }
    hint.sort_unstable();
    Ok(hint)
}

/// A `skills.jsonl` line, built from locals so the detector's table borrow never
/// outlives the row it came from.
fn shape_event(
    t: i32,
    transition: Transition,
    template: u32,
    recurrence: u16,
    filler_sets_seen: u16,
) -> ShapeEvent {
    ShapeEvent { t, transition, template, recurrence, filler_sets_seen }
}

#[cfg(test)]
mod tests {
    use super::*;
    use si_core::frame::{CoLight, Ghost};

    /// AD.5 reads the eventlog; a test of the bars needs no rows at all.
    const NO_EVENTS: &[Event] = &[];

    /// The smallest frame the walk can hand over: one seed, one shared filler
    /// reached by kind 0, and the role graph that says so (04 §32 AD.2).
    fn shape_frame(filler: u32, primed: bool) -> FrameLog {
        FrameLog {
            lights: vec![
                CoLight { point: 1, role: Role::Seed, edge_kind: 0, round: 0, primed: false },
                CoLight {
                    point: filler,
                    role: Role::SharedFiller,
                    edge_kind: 0,
                    round: 1,
                    primed,
                },
            ],
            ghosts: Vec::<Ghost>::new(),
            role_graphs: vec![RoleGraph {
                roles: vec![Role::Seed.byte(), Role::SharedFiller.byte()],
                traversed_kinds: vec![0],
                filler_sets: 1,
            }],
        }
    }

    fn no_graph_frame() -> FrameLog {
        FrameLog {
            lights: vec![CoLight {
                point: 1,
                role: Role::Seed,
                edge_kind: 0,
                round: 0,
                primed: false,
            }],
            ghosts: Vec::new(),
            role_graphs: Vec::new(),
        }
    }

    /// AD.3's order, as a test: bar 1 runs **first**, because it decides which
    /// detector owns the frame. A shape that has recurred forty times over one
    /// filler set is §29's content cluster, not a skill (docs/25 §25.3).
    #[test]
    fn generality_is_barred_before_recurrence() {
        let mut skiller =
            Skiller::new(Config { k_filler: 2, r_min_shape: 1, delta_min_shape: 1 });
        for t in 0..40 {
            let report = skiller.observe(&shape_frame(10, false), NO_EVENTS, t).unwrap();
            assert_eq!(report.events[0].transition, Transition::Forwarded);
            assert!(report.nominations.is_empty());
        }
        assert_eq!(skiller.opened_candidates(), 0);
        assert_eq!(skiller.forwarded_frames(), 40);
        assert_eq!(skiller.shapes(), 1);

        // The second filler set is what opens it — one frame, not forty.
        let report = skiller.observe(&shape_frame(11, false), NO_EVENTS, 40).unwrap();
        assert_eq!(report.events[0].transition, Transition::Opened);
        assert_eq!(report.nominations.len(), 1, "bar 2 was already met by bar 1's own frame");
        assert_eq!(report.nominations[0].filler_sets_seen, 2);
        assert_eq!(skiller.forwarded_frames(), 40, "the forwarded count is cumulative");
    }

    /// One ask per shape, and never a second one from the same table.
    #[test]
    fn a_shape_nominates_exactly_once() {
        let mut skiller =
            Skiller::new(Config { k_filler: 2, r_min_shape: 1, delta_min_shape: 1 });
        skiller.observe(&shape_frame(10, false), NO_EVENTS, 0).unwrap();
        let opened = skiller.observe(&shape_frame(11, false), NO_EVENTS, 1).unwrap();
        assert_eq!(opened.nominations.len(), 1);
        for t in 2..6 {
            let report = skiller.observe(&shape_frame(10 + t as u32, false), NO_EVENTS, t).unwrap();
            assert!(report.nominations.is_empty(), "a re-ask is the queue's rule, not this one");
            assert_eq!(report.events[0].transition, Transition::Recurred);
        }
        assert_eq!(skiller.table().values().next().unwrap().nominated_at, Some(1));
    }

    /// OD-105, in this detector's own words: a frame carrying a primed light
    /// counts nothing, and it does not even open a table entry.
    #[test]
    fn a_warm_frame_counts_nothing() {
        let mut skiller = Skiller::new(Config::default());
        let report = skiller.observe(&shape_frame(10, true), NO_EVENTS, 0).unwrap();
        assert_eq!(report.events[0].transition, Transition::WarmFrame);
        assert_eq!(report.shapes, 0);
        assert!(skiller.table().is_empty());
        assert_eq!(report.forwarded, 0, "a warm frame is not a forwarded one either");
    }

    /// A frame that built no scratch builds none — and the bound is never an
    /// eviction game (OD-99).
    #[test]
    fn a_frame_without_a_graph_builds_no_shape() {
        let mut skiller = Skiller::new(Config::default());
        let report = skiller.observe(&no_graph_frame(), NO_EVENTS, 0).unwrap();
        assert_eq!(report.events[0].transition, Transition::NoScratch);
        assert!(skiller.table().is_empty());
    }

    /// docs/25 §25.3's split, as bytes: the template is a fold of roles and
    /// kinds, and the fillers that differ are the *evidence* of generality, held
    /// in a set that never enters the key.
    #[test]
    fn the_fold_ignores_filler_identity_and_the_filler_sets_do_not() {
        let a = shape_frame(10, false);
        let b = shape_frame(11, false);
        let template_a = template_id(&fold(a.role_graphs.first().unwrap()));
        let template_b = template_id(&fold(b.role_graphs.first().unwrap()));
        assert_eq!(template_a, template_b, "same roles, same kinds: same shape");
        assert_eq!(fold(a.role_graphs.first().unwrap()), "seed|shared_filler:0");
        let sets_a = filler_set_ids(&a);
        let sets_b = filler_set_ids(&b);
        assert_eq!(sets_a.len(), 1);
        assert!(sets_a.is_disjoint(&sets_b), "different fillers are different sets");
    }

    /// G18(c)'s claim in the crate that owns it: the table is a function of the
    /// frame trace and the rows, so the same trace twice is the same bytes. The
    /// signature has no arousal argument to vary.
    #[test]
    fn the_table_is_a_function_of_the_trace_alone() {
        let trace = [(10u32, 0i32), (11, 1), (10, 2), (12, 3)];
        let mut first = Skiller::new(Config { k_filler: 2, r_min_shape: 2, delta_min_shape: 1 });
        let mut second = Skiller::new(Config { k_filler: 2, r_min_shape: 2, delta_min_shape: 1 });
        for (filler, t) in trace {
            first.observe(&shape_frame(filler, false), NO_EVENTS, t).unwrap();
            second.observe(&shape_frame(filler, false), NO_EVENTS, t).unwrap();
        }
        assert_eq!(first.table_bytes(), second.table_bytes());
        assert_ne!(first.table_bytes(), Vec::<u8>::new(), "an empty table would pass any equality");
    }

    /// docs/25 §25.4 rule 4: a shape that already has a row never opens a
    /// candidate again.
    #[test]
    fn an_already_minted_shape_still_forwarded_never_reopens() {
        let mut skiller =
            Skiller::new(Config { k_filler: 2, r_min_shape: 1, delta_min_shape: 1 });
        skiller.observe(&shape_frame(10, false), NO_EVENTS, 0).unwrap();
        let template = template_id(&fold(
            shape_frame(10, false).role_graphs.first().unwrap(),
        ));
        skiller.note_minted(template);
        let report = skiller.observe(&shape_frame(11, false), NO_EVENTS, 1).unwrap();
        assert_eq!(report.events[0].transition, Transition::AlreadyMinted);
        assert!(report.nominations.is_empty());
        assert_eq!(skiller.opened_candidates(), 0);
    }
}
