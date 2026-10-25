//! si-sleep — engineering/10 §7–§8, 04 §28 Z, 02 §13, 06 §3.4/§5.1.
//!
//! The consolidation pass over an **existing** lattice, run inside the loop at
//! step 12b of a quiet tick. It reads the live web and rewrites one FLUID column
//! — an OWNED edge's `weight` — plus the RAM rehearsal pair, and it has no
//! `si-learn` edge, so it physically cannot make a row ([00 §2](engineering) win
//! 4, G13(i)). `Clock::recency` is its only curve (R-29), the walk budget is one
//! number spent in pass order (OD-85), and the calm view P4 publishes is consumed
//! by nothing here (law 26, Z.4).

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use anyhow::Result;
use serde::Serialize;

use si_core::addr::{SHELF_CONTEXT, SHELF_FACTS, SHELF_LINKS, SHELF_S_R_PAIRS};
use si_core::clock;
use si_core::consts::{self, Class};
use si_core::quant::{mul_q12_u16, u8_to_q12, Q12_ONE};
use si_store::store::Lattice;
use si_walk::{Curve, Graph, Integrator, Lit, Params, Thought, WalkConfig};

/// The sleep's swept box: the six SPECIES rows of 03 §5.1, each read through its
/// own registry row. Nothing here is a value the crate invents — `from_registry`
/// takes the OD-86 placeholder and a launch installs a swept point over it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SleepConfig {
    pub cadence_ticks: u32,
    pub max_walks: u32,
    pub dim_base: u8,
    pub floor_s_r_q12: u16,
    pub floor_facts_q12: u16,
    pub floor_links_q12: u16,
    pub stable_n: u16,
}

fn placeholder(name: &str) -> u16 {
    consts::row(name)
        .and_then(consts::registry_value)
        .unwrap_or_else(|| panic!("{name} is not a scalar SPECIES row with a value"))
        .clamp(0, u16::MAX as i64) as u16
}

impl SleepConfig {
    pub fn from_registry() -> Self {
        Self {
            cadence_ticks: u32::from(placeholder("SLEEP_EVERY_QUIET_TICKS").max(2)),
            max_walks: u32::from(placeholder("SLEEP_MAX_WALKS")),
            dim_base: placeholder("DIM_BASE").min(255) as u8,
            floor_s_r_q12: placeholder("FLOOR_S_R"),
            floor_facts_q12: placeholder("FLOOR_FACTS"),
            floor_links_q12: placeholder("FLOOR_LINKS"),
            stable_n: placeholder("INFERENCE_STABLE_N"),
        }
    }

    /// 10 §6 D.1's `floor_shelf`: the three live belief shelves, each with its own
    /// registry row. `questions` and `context` are absent because neither carries
    /// a `z` weight — there is no edge there to dim (D.1's table).
    fn floor(&self, shelf: u16) -> Option<u16> {
        match shelf {
            SHELF_S_R_PAIRS => Some(self.floor_s_r_q12),
            SHELF_FACTS => Some(self.floor_facts_q12),
            SHELF_LINKS => Some(self.floor_links_q12),
            _ => None,
        }
    }
}

/// 10 §7's row for `due`: a counter and a checkpoint, never a wall clock. `No`
/// while any sense frame is pending — a sleeping mind has to be a quiet one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Due {
    Yes,
    No,
}

/// 02 §13 / 04 §28 Z.1. Trigger 1 is the quiet-tick counter reaching the swept
/// cadence — the same quiet predicate idle uses (§19 S.1), one definition, and it
/// fires on the multiple, so exactly once per window (**G13(f).1**). Trigger 2 is
/// the freeze: the caller asks only when a checkpoint is about to be written, and
/// the guard that keeps a resume from repeating the sleep is the caller's
/// `last_slept_at_t` check, not a second counter (OD-81).
pub fn due(config: &SleepConfig, quiet_run: u32, freeze_requested: bool, sense_pending: bool) -> Due {
    if sense_pending || config.cadence_ticks == 0 {
        return Due::No;
    }
    let cadence_hit = quiet_run > 0 && quiet_run % config.cadence_ticks == 0;
    if cadence_hit || freeze_requested {
        Due::Yes
    } else {
        Due::No
    }
}

/// 04 §28 Z.2 P1: the sense events to replay. `si-sleep` may not import
/// `si-front` or `si-read` (02 §13), so it cannot turn a `context` row's surface
/// into lightings; the session that owned the tick supplies them, as the same
/// round-0 vector the live tick lit. A row with no entry here is a replay event
/// the sleep counts but cannot walk — `events_available` and `walks` differ, and
/// the report shows it (**G13(l).4**).
#[derive(Clone, Debug)]
pub struct ReplayEvent {
    pub row_id: u32,
    pub written_at_t: i32,
    pub lights: Vec<Lit>,
}

/// The walk arithmetic for this sleep, owned by the sleep rather than borrowed
/// from the session: a sleep walk mutates the store and the integrator while it
/// reads the parameters, so the bundle must not hold a second borrow of the
/// session. Both fields are the caller's own copies.
#[derive(Clone, Debug)]
pub struct SleepRig {
    pub params: Params,
    pub curve: Curve,
    pub k_max: u16,
    pub tau_settle: u16,
    pub decay_base: u8,
    pub hub_exponent_q12: u16,
    pub frontier_cap: usize,
    pub oscillation: si_walk::OscillationRule,
    /// 12 §2.6: a sleep walk's frame is observed by nobody, so the rig carries
    /// the two frame-scratch caps at zero and a sleep builds no ghost.
    pub ghost_max: u16,
    pub shape_max: u16,
}

impl SleepRig {
    fn config(&self) -> WalkConfig<'_> {
        WalkConfig {
            params: &self.params,
            curve: &self.curve,
            k_max: self.k_max,
            tau_settle: self.tau_settle,
            decay_base: self.decay_base,
            hub_exponent_q12: self.hub_exponent_q12,
            frontier_cap: self.frontier_cap,
            oscillation: self.oscillation,
            // A sleep walk is not a crystallize frame: nothing observes its
            // log, so it builds no ghost and no role graph (12 §2.6).
            ghost_max: 0,
            shape_max: 0,
            // 04 §31 AC.1: a caller with no index passes the shipped off state.
            tugs: &[],
            relight_max: 0,
            // 12 §4.1 step 12b: a sleep walk reads no silt, no candidate and no
            // skill, so it gets no composition hint.
            composition_hint: &[],
        }
    }
}

pub struct SleepRequest<'a> {
    pub store: &'a mut Lattice,
    pub integrator: &'a mut Integrator,
    pub rig: SleepRig,
    pub config: SleepConfig,
    /// The quiet tick's `t`. Each walk the sleep runs is itself a quiet tick, so
    /// the sleep's own `now` advances with its walks (10 §7's steps row, OD-78);
    /// `final_now` is where the session's clock lands after the tick's step 13.
    pub now: i32,
    pub last_slept_at_t: i32,
    pub thought_id_start: u32,
    pub replay: Vec<ReplayEvent>,
    /// The same non-blocking inbox read as tick step 1 (04 §2 note 9), taken at
    /// every pass boundary and before every walk. It reads; it never consumes —
    /// the pending frame runs as the next ordinary tick. The argument is the pass
    /// the read guards (5 = the boundary after P4's report line), which is what
    /// lets 05 §18(g) append a frame "at each of the four pass boundaries, one run
    /// per boundary" without a new wall clock or a new CLI verb.
    pub sense_pending: &'a dyn Fn(u8) -> bool,
}

/// 10 §8's P1 line. `rows_written: 0` is literal: the pass has no write path to a
/// row (the missing `si-learn` edge), and R-07's bump path stays shut because a
/// replay's coverage is 1.0, so §6.2's own arithmetic calls it BORING.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct P1Report {
    pub pass: u8,
    pub walks: u32,
    pub events_available: u32,
    pub edges_touched: u32,
    pub rows_written: u32,
}

/// 10 §8's P2 line. `candidates` is the composite tally read through
/// `Lattice::composite_count` — derived RAM folded in as each settle-set is
/// logged (§21 U.4), which is what keeps this pass O(composite points) rather
/// than O(session) (06 §5.1). **It writes nothing**: `INFERENCE_PROMOTION` is off
/// and OD-58 is unruled, so `rows_written` is the spec's zero, not a hope.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct P2Report {
    pub pass: u8,
    pub candidates: u32,
    #[allow(non_snake_case)]
    pub stable_ge_N: u32,
    pub rows_written: u32,
}

/// 10 §8's P3 line: the only durable write the sleep makes, and its own account
/// of what it cost. `max_relative_shift` is `(w − w′) / w` on the Q12 grid by
/// reciprocal multiply (OD-45: never a divide). The two inversion counts are
/// 10 §9's honest loss made visible — within a shelf because `d` is per-edge,
/// across shelves because the floors differ — and a suppressed count fails
/// `G13(n)`, so they are always present even when zero.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct P3Report {
    pub pass: u8,
    pub edges_scanned: u32,
    pub edges_changed: u32,
    pub bytes_changed: u32,
    pub max_relative_shift: u16,
    pub within_shelf_rank_inversions: u64,
    pub cross_shelf_rank_inversions: u64,
    pub floor_hits: u32,
    /// Not a spec key of the pass line — the count of live OWNED edges already
    /// **below** their swept floor, which is the OD-94 box conflict made visible
    /// rather than a policy branch hidden in the arithmetic. It is skipped by
    /// serde so that `sleep.jsonl`'s P3 line is 10 §8's field set exactly; the
    /// number reaches the artifacts through `sleep_diff.json`, whose rows carry
    /// the same fact per edge.
    #[serde(skip)]
    pub edges_below_floor: u32,
}

/// 10 §8's P4 line plus the calm view. The calm view's counts are published and
/// consumed by nothing (Z.4): no weight, floor, `d`, ignition, label or criterion
/// reads them, and `check_calm_view_is_terminal` is the static half of that claim.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct P4Report {
    pub pass: u8,
    pub walks: u32,
    pub points_swept: u32,
    pub rows_read_calm_not_settleable: u32,
    pub rows_relit: u32,
    /// OPEN-24 is unruled, so the re-stamp stays off and this is the spec's zero.
    pub stamps_written: u32,
    pub calm: CalmView,
}

#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq)]
pub struct CalmView {
    pub points_swept: u32,
    pub walks: u32,
    pub rows_relit: u32,
    pub rows_read_calm_not_settleable: u32,
    /// Z.4's "edges re-lit": the distinct arcs a calm walk crossed.
    pub edges_relit: u32,
    pub stamps_written: u32,
}

/// One walk the sleep ran, returned for the session to close. The sleep cannot
/// call back into the session — it would need `&mut` to the very store it holds —
/// so it hands the thought and its `t`/`thought_id` pair back and the session
/// performs §19's per-thought bookkeeping in order.
#[derive(Clone, Debug)]
pub struct SleepWalk {
    pub pass: u8,
    pub t: i32,
    pub thought_id: u32,
    /// The round-0 point: P1's `context` row's own lighting, P4's swept start.
    pub seed_point: u32,
    pub thought: Thought,
}

/// 10 §7's partial-sleep record. `rows_touched` is the count of **rows whose
/// bytes** the sleep rewrote, and in v1 that is zero by construction — the pass
/// writes an edge column and a RAM pair, never a row (G13(i), G13(j)).
#[derive(Clone, Debug, Serialize)]
pub struct SleepReport {
    pub t: i32,
    pub passes_completed: Vec<u8>,
    pub p1: Option<P1Report>,
    pub p2: Option<P2Report>,
    pub p3: Option<P3Report>,
    pub p4: Option<P4Report>,
    pub rows_touched: u32,
    pub bytes_changed: u32,
    pub walks: u32,
    pub aborted_reason: Option<String>,
    /// The sleep's own clock after its last walk — the tick's step 13 closes the
    /// final walk, so the session advances once more from here only when the
    /// sleep ran no walk at all (10 §7's steps row: `w` walks are `w` ticks).
    pub final_now: i32,
    #[serde(skip)]
    pub trail: Vec<SleepWalk>,
    /// P3's before/after table, for `sleep_diff.json` (05 §18's (b)/(c)/(d)/(j)
    /// evidence). It is not a `sleep.jsonl` key — 10 §8 fixes that line's set.
    #[serde(skip)]
    pub ledger: Vec<SleepLedgerEntry>,
}

impl SleepReport {
    /// The `sleep_pass_mask` bits the core block records (01 §13).
    pub fn pass_mask(&self) -> u8 {
        self.passes_completed
            .iter()
            .fold(0u8, |mask, pass| mask | (1 << (pass - 1)))
    }
}

/// One OWNED edge P3 looked at, as the diff artifact needs it: the byte before,
/// the byte after, the `last_touched_at` the one curve read, the factor it produced
/// and the floor that anchored the contraction. 05 §18's (b), (c), (d) and (j) legs
/// are this table — a headroom ratio, a rank pair, a floor comparison — so the
/// before/after is data the gate can grade, not a sentence the pass writes about
/// itself. `sleep.jsonl`'s pass line carries 10 §8's keys and nothing else; this
/// list is what `sleep_diff.json` is made of.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct SleepLedgerEntry {
    pub sleep_t: i32,
    pub shelf: u16,
    pub edge_off: u32,
    pub before: u16,
    pub after: u16,
    pub last_touched_at: i32,
    pub d_q12: u16,
    pub floor_q12: u16,
    /// OD-94's visible shape: the edge sat below its swept floor, so the literal
    /// contraction would have brightened it and the pass left it alone.
    pub below_floor: bool,
}

/// 02 §13's fixed order: `due` → P1 → (inbox) → P2 → (inbox) → P3 → (inbox) → P4
/// → (inbox) → report. The budget is one number spent in pass order (OD-85): P1
/// first, P4 the remainder, and P4's *selection sweep* still full even at
/// remainder 0, because a sweep is a read and a walk is a spend.
pub fn run(request: &mut SleepRequest<'_>) -> Result<SleepReport> {
    let mut now = request.now;
    let budget = request.config.max_walks;
    let mut spent = 0u32;
    let mut trail: Vec<SleepWalk> = Vec::new();
    let mut passes_completed = Vec::new();
    let mut aborted_reason: Option<String> = None;

    let mut p1 = None;
    let mut p2 = None;
    let mut p3 = None;
    let mut p4 = None;
    let mut sleep_ledger: Vec<SleepLedgerEntry> = Vec::new();

    // The inbox is read at every pass boundary and before every walk (10 §7's
    // interrupt row, 04 §2 note 9). A pass is the atomic unit, so an abort never
    // lands inside P3's sweep — `G13(g).2`'s "a rescale is never half-applied".
    if (request.sense_pending)(1) {
        aborted_reason = Some("inbox_pending_before_pass_1".to_string());
    } else {
        let remaining = budget.saturating_sub(spent);
        let (report, walked) = p1_replay(request, now, remaining, &mut trail)?;
        spent = spent.saturating_add(walked);
        now = now.saturating_add(walked as i32);
        passes_completed.push(1);
        p1 = Some(report);
        if (request.sense_pending)(2) {
            aborted_reason = Some("inbox_pending_before_pass_2".to_string());
        } else {
            let report = p2_integration_scan(request)?;
            passes_completed.push(2);
            p2 = Some(report);
            if (request.sense_pending)(3) {
                aborted_reason = Some("inbox_pending_before_pass_3".to_string());
            } else {
                // P3 always precedes the checkpoint, never the reverse (OD-81),
                // and P4 reads the weights it just wrote.
                let (report, mut ledger) = p3_rescale(request, now)?;
                passes_completed.push(3);
                p3 = Some(report);
                sleep_ledger.append(&mut ledger);
                if (request.sense_pending)(4) {
                    aborted_reason = Some("inbox_pending_before_pass_4".to_string());
                } else {
                    let remaining = budget.saturating_sub(spent);
                    let (report, walked) = p4_calm_reprocessing(request, now, remaining, &mut trail)?;
                    now = now.saturating_add(walked as i32);
                    passes_completed.push(4);
                    p4 = Some(report);
                    // Z.2/P4: its report line is the sleep's last write. A frame
                    // that arrives at this boundary is heard by the *next* tick —
                    // the sleep is complete, not partial, and `Aborted` would be
                    // a lie about a pass set that all ran (10 §8's pass-set law).
                    let _ = (request.sense_pending)(5);
                }
            }
        }
    }

    let bytes_changed = p3.as_ref().map(|report| report.bytes_changed).unwrap_or(0);
    Ok(SleepReport {
        t: request.now,
        passes_completed,
        p1,
        p2,
        p3,
        p4,
        // Every pass of v1 writes zero row bytes; the ledger has one column name
        // in it for the durable state (`weight`) and one for the RAM pair
        // (`last_touched_at`) — G13(j).4.
        rows_touched: 0,
        bytes_changed,
        walks: trail.len() as u32,
        aborted_reason,
        final_now: now,
        trail,
        ledger: sleep_ledger,
    })
}

/// 02 §13: the `context` rows with `t > last_slept_at_t`, oldest-first, capped at
/// the remaining budget with the **newest** kept when truncating.
pub fn p1_replay(
    request: &mut SleepRequest<'_>,
    now: i32,
    budget: u32,
    trail: &mut Vec<SleepWalk>,
) -> Result<(P1Report, u32)> {
    let mut events = Vec::new();
    for (offset, record) in request.store.rows()? {
        let _ = offset;
        if record.envelope.shelf_x != SHELF_CONTEXT {
            continue;
        }
        if record.envelope.written_at_t <= request.last_slept_at_t {
            continue;
        }
        if !record.envelope.is_live() {
            continue;
        }
        events.push((record.envelope.row_id, record.envelope.written_at_t));
    }
    // Row order is write order; the truncation drops the oldest and the walk set
    // still runs oldest-first, so a replay never violates row order (OD-85).
    // `events_available` is counted **before** that truncation, which is what makes
    // a night that consolidated less than it was given visible in the line rather
    // than inferable from it (**G13(l).4**): `walks` is short of it both when the
    // budget cut the set and when a row has no recorded lighting to walk.
    let events_available = events.len() as u32;
    let start = (events.len().saturating_sub(budget as usize)).min(events.len());
    let events = events.split_off(start);

    let mut edges: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    let mut walks = 0u32;
    let mut thought_id = request.thought_id_start;
    for (row_id, written_at_t) in &events {
        // 04 §28 Z.2: each walk opens with the inbox read. A frame scheduled at a
        // *later* boundary does not stop this one — the pass that owns the boundary
        // is the pass that hears it.
        if (request.sense_pending)(1) {
            break;
        }
        let lights = match request.replay.iter().find(|event| event.row_id == *row_id) {
            Some(event) => event.lights.clone(),
            // No lighting recorded for this row: the sleep counts the event it
            // was given and walks none, rather than minting a point it was not
            // shown. `G13(l).4` reads this pair of counts.
            None => continue,
        };
        if lights.is_empty() {
            continue;
        }
        let t = now.saturating_add(walks as i32);
        let graph = Graph::build(request.store, t)?;
        let thought = si_walk::walk(
            &graph,
            &lights,
            request.integrator,
            thought_id,
            &[],
            &[],
            &request.rig.config(),
        )?;
        record_walk(request.store, &mut edges, t, &thought);
        trail.push(SleepWalk {
            pass: 1,
            t,
            thought_id,
            seed_point: lights[0].point,
            thought,
        });
        thought_id = thought_id.saturating_add(1);
        walks += 1;
        let _ = written_at_t;
    }
    let walked = walks;
    Ok((
        P1Report {
            pass: 1,
            walks: walked,
            events_available,
            edges_touched: edges.len() as u32,
            rows_written: 0,
        },
        walked,
    ))
}

/// 02 §13: the composite tally and the `links` shelf, read; nothing written.
pub fn p2_integration_scan(request: &mut SleepRequest<'_>) -> Result<P2Report> {
    let mut candidates = 0u32;
    let mut stable = 0u32;
    let mut points: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for (_, record) in request.store.rows()? {
        if !record.envelope.is_live() {
            continue;
        }
        points.insert(record.body.key_point);
    }
    for point in points {
        let count = request.store.composite_count(point);
        if count == 0 {
            continue;
        }
        candidates += 1;
        if count >= u64::from(request.config.stable_n) {
            stable += 1;
        }
    }
    Ok(P2Report {
        pass: 2,
        candidates,
        stable_ge_N: stable,
        rows_written: 0,
    })
}

/// 10 §6 D.1 / §9 row (a): the shipped contraction. Shelf by shelf in `(x, y)`
/// order, per live OWNED edge:
///
/// ```text
/// d  = DIM_BASE + (1 − DIM_BASE) · Clock::recency(now − last_touched_at)
/// w' = floor_shelf + (w − floor_shelf) · d
/// ```
///
/// `d` is read through the one curve and from nothing else (R-29; a `d` that read
/// `bake`, a confidence or a stamp would be a new affect law wearing the
/// rescale's clothes), and `d ≤ 1` by construction, so the map is monotone
/// non-increasing: no edge brightens (**10 §9**, Z.5) and no live edge reaches 0
/// (D.3's biconditional).
///
/// The one branch the arithmetic cannot express lawfully is an edge already
/// **below** its swept floor: the literal formula would lift it, which is a
/// brightening. It stays where it is and is counted in `edges_below_floor` —
/// the OD-94 box conflict reported, not a workaround swallowed.
pub fn p3_rescale(request: &mut SleepRequest<'_>, now: i32) -> Result<(P3Report, Vec<SleepLedgerEntry>)> {
    let base = u8_to_q12(request.config.dim_base);
    let mut scanned = 0u32;
    let mut changed = 0u32;
    let mut floor_hits = 0u32;
    let mut below_floor = 0u32;
    let mut max_shift = 0u16;
    // Every OWNED edge this sweep read, in scan order, before and after.
    let mut ledger: Vec<SleepLedgerEntry> = Vec::new();

    let mut rows = request.store.rows()?;
    rows.sort_by_key(|(_, record)| (record.envelope.shelf_x, record.envelope.row_id));
    for (_, record) in rows {
        let shelf = record.envelope.shelf_x;
        let Some(floor) = request.config.floor(shelf) else {
            continue;
        };
        if !record.envelope.is_live() {
            continue;
        }
        let edges = request.store.edges_of(&record)?;
        for (index, edge) in edges.iter().enumerate() {
            if !edge.is_live() {
                continue;
            }
            let edge_off = record.envelope.edges_off + (index * si_core::record::EDGE_BYTES) as u32;
            scanned += 1;
            // 01 §8's rehearsal pair exists from the edge's formation, so a
            // resumed session whose RAM is empty reads the write that made it.
            let last = request
                .store
                .last_touched(edge_off)
                .unwrap_or(record.envelope.written_at_t);
            let recency = u8_to_q12(clock::recency(now, last));
            let d = base.saturating_add(mul_q12_u16(Q12_ONE - base, recency));
            let before = edge.weight;
            let under_floor = before < floor;
            let after = if under_floor {
                below_floor += 1;
                before
            } else {
                floor.saturating_add(mul_q12_u16(before - floor, d))
            };
            if after > before {
                // Unreachable by construction (`d ≤ 1` and the floor branch
                // above), and Z.5 says a brightening is a panic, not a policy
                // branch — so this stays an assert and never a clamp.
                panic!("P3 would brighten edge {edge_off} from {before} to {after} (10 §9, 04 §28 Z.5)");
            }
            if after == 0 && before != 0 {
                panic!("P3 would zero live edge {edge_off} (10 §6 D.3: weight == 0 means superseded)");
            }
            if after == floor {
                floor_hits += 1;
            }
            if after != before {
                // The store's own proof: only bytes 6..8 of the 16-byte record
                // move, and the write refuses a brightening or a zeroing.
                let moved = request.store.dim_edge_weight(edge_off, after).unwrap_or_else(|_| {
                    panic!("P3's weight write was refused for edge {edge_off} at weight {before} -> {after}")
                });
                if moved {
                    changed += 1;
                    max_shift = max_shift.max(relative_shift_q12(before, after));
                }
            }
            ledger.push(SleepLedgerEntry {
                sleep_t: now,
                shelf,
                edge_off,
                before,
                after,
                last_touched_at: last,
                d_q12: d,
                floor_q12: floor,
                below_floor: under_floor,
            });
        }
    }

    let (within, cross) = rank_inversions(&ledger);
    Ok((
        P3Report {
            pass: 3,
            edges_scanned: scanned,
            edges_changed: changed,
            // 06 §5.1: two bytes rewritten per dimmed edge, in place, no append.
            bytes_changed: changed * 2,
            max_relative_shift: max_shift,
            within_shelf_rank_inversions: within,
            cross_shelf_rank_inversions: cross,
            floor_hits,
            edges_below_floor: below_floor,
        },
        ledger,
    ))
}

/// `(w − w′) / w` on the Q12 grid, reciprocal multiply, round-half-up (OD-45).
fn relative_shift_q12(before: u16, after: u16) -> u16 {
    if before == 0 || after >= before {
        return 0;
    }
    let delta = u64::from(before - after);
    let reciprocal = (1u64 << 32) / u64::from(before);
    let scaled = delta.saturating_mul(reciprocal);
    (((scaled + (1 << 19)) >> 20) as u16).min(Q12_ONE)
}

/// 10 §9's kept-and-reported ordering. `within` counts pairs in one shelf whose
/// weight order flipped — inside a `last_touched_at` group the map is monotone, so
/// such a pair is the smoking gun for a per-edge factor that is not the one curve;
/// across groups an inversion is legal and is only counted. `cross` counts the
/// same over pairs in different shelves, where the floors themselves differ and
/// the frontier trim can therefore rank differently after a night (**G13(n)**).
fn rank_inversions(ledger: &[SleepLedgerEntry]) -> (u64, u64) {
    // `(before, after)` per edge, whole ledger and per shelf. Sorting by
    // `(before, after)` ascending means a pair that shares a `before` can never be
    // counted as a flip, which is the strict-inequality law's shape (an inversion
    // needs both comparisons to be strict).
    let mut all: Vec<(u16, u16)> = Vec::with_capacity(ledger.len());
    let mut shelves: BTreeMap<u16, Vec<(u16, u16)>> = BTreeMap::new();
    for entry in ledger {
        let pair = (entry.before, entry.after);
        all.push(pair);
        shelves.entry(entry.shelf).or_default().push(pair);
    }
    let sort = |pairs: &mut Vec<(u16, u16)>| {
        pairs.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)))
    };
    let mut within = 0u64;
    for (_shelf, pairs) in shelves.iter_mut() {
        sort(pairs);
        within += flip_count(pairs);
    }
    sort(&mut all);
    let total = flip_count(&all);
    // A cross-shelf pair is any flipped pair that is not within one shelf, and the
    // two counts therefore add up to what the sweep did — 10 §9's honest loss has
    // no room for a pair that is counted twice or dropped.
    (within, total - within)
}

/// The flipped pairs of a list already sorted by `before`: `count` of the pairs
/// whose order changed. Merge-counting keeps this **O(n log n)** — 06 §5.1's P3
/// term is linear in `E_live` and a quadratic counter would make the *report* the
/// most expensive part of the sweep, which is the shape §9's list of growth streams
/// refuses.
fn flip_count(pairs: &[(u16, u16)]) -> u64 {
    let mut values: Vec<u16> = pairs.iter().map(|pair| pair.1).collect();
    let mut scratch = vec![0u16; values.len()];
    let len = values.len();
    merge_count(&mut values, &mut scratch, 0, len)
}

fn merge_count(values: &mut [u16], scratch: &mut [u16], lo: usize, hi: usize) -> u64 {
    if hi - lo < 2 {
        return 0;
    }
    let mid = lo + (hi - lo) / 2;
    let mut count = merge_count(values, scratch, lo, mid) + merge_count(values, scratch, mid, hi);
    let (mut i, mut j, mut k) = (lo, mid, lo);
    while i < mid && j < hi {
        if values[i] <= values[j] {
            scratch[k] = values[i];
            i += 1;
        } else {
            scratch[k] = values[j];
            // The input is sorted by `before`, so every remaining left element has
            // a strictly smaller `before` and a strictly larger `after`.
            count += (mid - i) as u64;
            j += 1;
        }
        k += 1;
    }
    while i < mid {
        scratch[k] = values[i];
        i += 1;
        k += 1;
    }
    while j < hi {
        scratch[k] = values[j];
        j += 1;
        k += 1;
    }
    values[lo..hi].copy_from_slice(&scratch[lo..hi]);
    count
}

/// 02 §13 / Z.2 P4: the live web swept in **point-handle order** — no seeded
/// pick, no RNG handle, so stream R's trajectory is untouched by the presence of
/// a sleep — and the first `remaining` start points walked at rest. Each thought
/// starts at `a = 0` as a *start*, not a pin: §7's per-event integration runs
/// normally, so a signal that fires in a calm walk is a real event.
pub fn p4_calm_reprocessing(
    request: &mut SleepRequest<'_>,
    now: i32,
    budget: u32,
    trail: &mut Vec<SleepWalk>,
) -> Result<(P4Report, u32)> {
    // The full sweep is a read, so it happens even at remainder 0 (OD-85).
    let graph = Graph::build(request.store, now)?;
    let points: Vec<u32> = graph.points().collect();
    let points_swept = points.len() as u32;

    let mut reached: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    let mut edges: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    let mut walks = 0u32;
    let mut thought_id = request.thought_id_start.saturating_add(trail.len() as u32);
    for point in points.iter().take(budget as usize) {
        if (request.sense_pending)(4) {
            break;
        }
        let t = now.saturating_add(walks as i32);
        // At rest: the walk starts the integrator at 0 and then lets the round's
        // own signals move it (Z.2's "a start, not a pin").
        *request.integrator = Integrator::restore(0);
        let graph = Graph::build(request.store, t)?;
        let thought = si_walk::walk(
            &graph,
            &[Lit {
                point: *point,
                activation: si_walk::LIGHT_QUANTUM,
            }],
            request.integrator,
            thought_id,
            &[],
            &[],
            &request.rig.config(),
        )?;
        record_walk(request.store, &mut edges, t, &thought);
        for entry in &thought.settle_set {
            reached.insert(entry.point);
        }
        trail.push(SleepWalk {
            pass: 4,
            t,
            thought_id,
            seed_point: *point,
            thought,
        });
        thought_id = thought_id.saturating_add(1);
        walks += 1;
    }

    // Which live belief rows read differently at rest. A sweep that walked
    // nothing has *read* nothing at rest, so both row counts stay 0 rather than
    // reporting the budget as a property of the web.
    let mut relit = 0u32;
    let mut not_settleable = 0u32;
    if walks > 0 {
        let swept: std::collections::BTreeSet<u32> = points.into_iter().collect();
        for (_, record) in request.store.rows()? {
            let shelf = record.envelope.shelf_x;
            if request.config.floor(shelf).is_none() || !record.envelope.is_live() {
                continue;
            }
            let point = record.body.key_point;
            if !swept.contains(&point) {
                continue;
            }
            if reached.contains(&point) {
                relit += 1;
            } else {
                not_settleable += 1;
            }
        }
    }
    let calm = CalmView {
        points_swept,
        walks,
        rows_relit: relit,
        rows_read_calm_not_settleable: not_settleable,
        edges_relit: edges.len() as u32,
        stamps_written: 0,
    };
    Ok((
        P4Report {
            pass: 4,
            walks,
            points_swept,
            rows_read_calm_not_settleable: not_settleable,
            rows_relit: relit,
            stamps_written: 0,
            calm,
        },
        walks,
    ))
}

/// The rehearsal pair, reset for every arc this walk crossed — 04 §19 S.7's
/// allowance and nothing more. It moves RAM (`Lattice::touched`), no arena byte
/// and no hash input, which is what makes G13(b)'s digests survive a replay pass.
/// It is also the feedback loop: the pass that dims by `last_touched_at` is the
/// one whose own walks push that date forward.
fn record_walk(
    store: &mut Lattice,
    edges: &mut std::collections::BTreeSet<u32>,
    t: i32,
    thought: &Thought,
) {
    for round in &thought.rounds {
        for contribution in &round.contributions {
            // G13(e): a corpus edge has no `last_touched_at` column at all, and
            // its CSR index can alias a live arena offset, so the dimming pass's
            // one permitted write is keyed to the live shape only.
            if contribution.genome || !edges.insert(contribution.edge_off) {
                continue;
            }
            store.touch_edge_curve(contribution.edge_off, t);
        }
    }
    // 04 §21 U.4: the composite tally is derived RAM folded in as the settle-set
    // is logged, and P2 reads it. Folding it here is not a write to L3 — it moves
    // no byte and enters no digest — and it is what keeps P2 flat (06 §5.1).
    let points: Vec<u32> = thought.settle_set.iter().map(|entry| entry.point).collect();
    store.observe_settle(&points);
}

/// 03 §5.1's note on the three classes the sleep may read: every row it installs
/// is SPECIES, and a TEMPERAMENT row has no business here (OD-A: "no sweep may
/// ever touch a temperament row"). `launch_ok` is the runner's check, kept in this
/// crate so the registry's own class table is the only definition.
pub fn launch_ok(name: &str) -> bool {
    consts::row(name).is_some_and(|row| row.class == Class::Species && row.scalar)
}

/// The sleep's reads, for the diff artifact: the shelf floors as the run used
/// them. A map rather than a list so a reader can see which shelf a count names.
pub fn floors(config: &SleepConfig) -> BTreeMap<&'static str, u16> {
    let mut out = BTreeMap::new();
    out.insert("FLOOR_S_R", config.floor_s_r_q12);
    out.insert("FLOOR_FACTS", config.floor_facts_q12);
    out.insert("FLOOR_LINKS", config.floor_links_q12);
    out
}
