//! si-session — engineering/02 §11, 04 §18, §24–§26, 10 §3–§4. The loop.
//!
//! One tick is 02 §11's order: frame, decode, short-circuits, pending-ask match,
//! light, walk, settle, gaps, label, render, commit, idle, flush, hash, advance.
//! The session is the only module allowed to call both `si-learn` and
//! `si-walk`, and the only one that knows a stage. It never names a drive policy
//! and never sets `a`: the integrator is fed by the six signal kinds and by
//! nothing else (I8).
//!
//! Stage B5: tick steps 1–11 and 13, the run directory, the source-tagged inbox.
//! Step 12's ignition is B7's and step 12b's sleep is B8's; the idle tick at this
//! stage runs only what §19 S.2 makes safe without ignition — the queue revisit
//! and RE-PARSE.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{bail, Context, Result};
use si_affect::drive::{self, Budget};
use si_affect::params::Params;
use si_affect::{Integrator, Lagged};
use si_core::addr::{SHELF_CONTEXT, SHELF_FACTS, SHELF_QUESTIONS, SHELF_S_R_PAIRS, SHELF_SKILLS};
use si_core::clock::{self, Clock};
use si_core::consts;
use si_core::eventlog::EventLog;
use si_core::hash::{
    encode_rng, encode_settle_set, encode_trajectory, CoreBlock, HashInputs, StateHash, VersionTags,
};
use si_core::patterns::PatternId;
use si_core::quant::{mul_q12_u16, u8_to_q12, Q12_ONE};
use si_core::record::{Body, Envelope};
use si_core::rng::Rng;
use si_core::seed::{self, SEED_FORMAT};
use si_core::strings::Source;
use si_core::temperament::{DerivId, Temperament, DERIV_ORDER};
use si_learn::gate::{
    self, Candidate, GateStats, Outcome, QuestionSpec, WriteContext, WriteReport,
};
use si_learn::reparse::{DerivedSlot, Parser};
use si_read::ask::ask_back;
use si_read::retrieve::{retrieve, ReadRequest, Retrieval, SlotKey};
use si_read::light::Resolve;
use si_store::layers_frozen::Frozen;
use si_store::points::Points;
use si_store::snapshot::{Inspect, ResumeRefusal, Snapshot};
use si_store::store::Lattice;
use si_walk::{Curve, CurvePoint, OscillationRule, Thought, WalkConfig};

/// The arc's three leaf crates, re-exported for the harness. `si-cli` already
/// depends on the session; reaching `si_silt`/`si_crystallize`/`si_skills`
/// through it keeps the manifest graph unchanged while the gate grades each
/// crate's own API (12 §6.4's frame-trace-fixed and clears-backwards legs).
pub use si_crystallize;
pub use si_silt;
pub use si_skills;

// The event-log kinds (08 §2.1), aliased from the one table in `si-core` that
// process B reads too. Only the kinds the loop produces are named here.
use si_core::eventlog::kind as frame;
const EV_SENSE_ECHO: u8 = frame::SENSE_ECHO;
const EV_DECODED: u8 = frame::DECODED;
const EV_LIT_ROUND0: u8 = frame::LIT_ROUND0;
const EV_ROUND: u8 = frame::ROUND;
const EV_SETTLE: u8 = frame::SETTLE;
const EV_SIGNAL: u8 = frame::SIGNAL_EVENT;
const EV_A_UPDATE: u8 = frame::A_UPDATE;
const EV_MEDIAN: u8 = frame::MEDIAN_USED;
const EV_DOOR: u8 = frame::DOOR_CALL;
const EV_GAP: u8 = frame::GAP;
const EV_ASK: u8 = frame::ASK;
const EV_LABEL: u8 = frame::LABEL;
const EV_OUTPUT: u8 = frame::OUTPUT;
const EV_COMMIT: u8 = frame::COMMIT;
const EV_SUPERSEDE: u8 = frame::SUPERSEDE;
const EV_STAGE: u8 = frame::STAGE;
const EV_TICK_END: u8 = frame::TICK_END;
const EV_SEAM_REJECT: u8 = frame::SEAM_REJECT;
/// An idle ignition marker — the one logged fact that an idle tick chose
/// to think, before the cascade's own lit/settle rows. It carries no belief.
/// **OD-143**: it sits above 08 §2.1's sealed table because that table has no
/// slot for it and §2.1's own note refuses a flag on the old kinds.
const EV_IGNITION: u8 = frame::IGNITION;
/// `sleep_pass` — the `sleep.jsonl` line **verbatim**, emitted at step
/// 12b after each pass's writes complete, and the interrupt line in place of the
/// passes a cut-short night did not run. Kind 18 is reserved and never emitted;
/// 19 and 20 are the TEMPERAMENT/SLEEP batch's two frames. It is a record, not a
/// lever: 08 §6 O11 reads it and nothing in the loop does (law 26, Z.4).
const EV_SLEEP_PASS: u8 = frame::SLEEP_PASS;
/// 08 §2.1 row 20: the same table `fingerprint.txt` holds, emitted at wake before
/// the first round of the first thought, so a panel and a text file cannot
/// disagree — and so the viewer never re-derives it (08 §11's rule, not a missing
/// dependency: `temperament::derive` is one crate edge away from it).
const EV_TEMPERAMENT: u8 = frame::TEMPERAMENT_READOUT;

fn species(name: &str) -> u16 {
    consts::row(name)
        .and_then(consts::registry_value)
        .unwrap_or_else(|| panic!("{name} is not a scalar SPECIES row with a value"))
        .clamp(0, u16::MAX as i64) as u16
}

/// 03 §5.1's declared ceiling of the `SLEEP_EVERY_QUIET_TICKS` box — the number
/// 05 §2's V9 row means by "beyond the scenario's tick count". Read from the row
/// rather than written here, so the variant names no new constant and bakes no
/// scenario length into the build.
fn sleep_cadence_ceiling() -> u32 {
    consts::row("SLEEP_EVERY_QUIET_TICKS")
        .unwrap_or_else(|| panic!("SLEEP_EVERY_QUIET_TICKS is not a registry row"))
        .hi
        .clamp(0, u32::MAX as i64) as u32
}

/// The walk's numeric box for this build. Every value is the registry's
/// OD-86 placeholder until its sweep lands (11 §1 D5); the session invents none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WalkConstants {
    pub k_max: u16,
    pub tau_settle: u16,
    pub decay_base: u8,
    pub hub_exponent_q12: u16,
    pub frontier_cap: usize,
    pub oscillation: OscillationRule,
}

impl WalkConstants {
    pub fn from_registry() -> Self {
        let rule = species("OSCILLATION_RULE");
        Self {
            k_max: species("K_MAX_ROUNDS"),
            tau_settle: species("TAU_SETTLE"),
            decay_base: species("DECAY_BASE").min(255) as u8,
            hub_exponent_q12: species("HUB_DAMP_EXPONENT"),
            frontier_cap: species("FRONTIER_CAP") as usize,
            oscillation: if rule == 1 { OscillationRule::PeriodCheck } else { OscillationRule::MaxKOnly },
        }
    }
}

/// engineering/12 §3 / 13 §3.2: the arc's and the tongue's 24 registry rows, read
/// once at birth and overwritten only by the launch-constant channel (05 §22). The
/// values are declared by the registry — seventeen swept SPECIES rows at OD-86's
/// placeholder (or a declared `0`), seven Ruling rows at their least-authority
/// default — so a pre-arc run reads this struct and changes no byte: every consumer
/// below is gated by a Ruling row that ships `off`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArcConfig {
    pub ghost_max: u16,
    pub shape_max: u16,
    pub relight_max: u16,
    pub r_min: u16,
    pub r_ask: u16,
    pub delta_min: i32,
    pub crystallize: bool,
    pub silt_spread_priming: bool,
    pub episodic_traversal: bool,
    pub skills: bool,
    pub skill_composition: bool,
    pub lambda_silt: u8,
    pub silt_level_max: u16,
    pub silt_bump: u16,
    pub theta_silt: u16,
    pub silt_max: u16,
    pub ign_weight_silt: u16,
    pub prov_link_tug: u16,
    pub r_min_shape: u16,
    pub delta_min_shape: i32,
    pub k_filler: u16,
    pub world_tiers: u8,
    pub tongue: bool,
    pub phrase_max_steps: usize,
}

/// The one reading of a registry row's shipped value: declared, else OD-86's
/// placeholder, else the law's own number.
fn arc_value(name: &str) -> i64 {
    let row = consts::row(name).unwrap_or_else(|| panic!("{name} is not a registry row"));
    consts::registry_value(row).unwrap_or_else(|| panic!("{name} carries no value"))
}

fn arc_bool(name: &str) -> bool {
    arc_value(name) == 1
}

impl ArcConfig {
    pub fn from_registry() -> Self {
        Self {
            ghost_max: arc_value("GHOST_MAX").clamp(0, u16::MAX as i64) as u16,
            shape_max: arc_value("SHAPE_MAX").clamp(0, u16::MAX as i64) as u16,
            relight_max: arc_value("RELIGHT_MAX").clamp(0, u16::MAX as i64) as u16,
            r_min: arc_value("R_MIN_CRYSTALLIZE").clamp(0, u16::MAX as i64) as u16,
            r_ask: arc_value("R_ASK_CRYSTALLIZE").clamp(0, u16::MAX as i64) as u16,
            delta_min: arc_value("DELTA_MIN_CRYSTALLIZE").clamp(0, i32::MAX as i64) as i32,
            crystallize: arc_bool("CRYSTALLIZE"),
            silt_spread_priming: arc_bool("SILT_SPREAD_PRIMING"),
            episodic_traversal: arc_bool("EPISODIC_TRAVERSAL"),
            skills: arc_bool("SKILLS"),
            skill_composition: arc_bool("SKILL_COMPOSITION"),
            lambda_silt: arc_value("LAMBDA_SILT").clamp(0, 255) as u8,
            silt_level_max: arc_value("SILT_LEVEL_MAX").clamp(0, u16::MAX as i64) as u16,
            silt_bump: arc_value("SILT_BUMP").clamp(0, u16::MAX as i64) as u16,
            theta_silt: arc_value("THETA_SILT").clamp(0, u16::MAX as i64) as u16,
            silt_max: arc_value("SILT_MAX").clamp(0, u16::MAX as i64) as u16,
            ign_weight_silt: arc_value("IGN_WEIGHT_SILT").clamp(0, u16::MAX as i64) as u16,
            prov_link_tug: arc_value("PROV_LINK_TUG").clamp(0, u16::MAX as i64) as u16,
            r_min_shape: arc_value("R_MIN_SHAPE").clamp(0, u16::MAX as i64) as u16,
            delta_min_shape: arc_value("DELTA_MIN_SHAPE").clamp(0, i32::MAX as i64) as i32,
            k_filler: arc_value("K_FILLER").clamp(0, u16::MAX as i64) as u16,
            world_tiers: arc_value("WORLD_TIERS").clamp(0, u8::MAX as i64) as u8,
            tongue: arc_bool("TONGUE"),
            phrase_max_steps: arc_value("PHRASE_MAX_STEPS").max(0) as usize,
        }
    }

    fn crystallize_config(self) -> si_crystallize::Config {
        si_crystallize::Config { r_min: self.r_min, r_ask: self.r_ask, delta_min: self.delta_min }
    }

    /// engineering/12 §5 §32: the shape detector's three registry rows. The
    /// candidate table is session RAM and hash-exempt (OD-117).
    fn skills_config(self) -> si_skills::Config {
        si_skills::Config {
            k_filler: self.k_filler,
            r_min_shape: self.r_min_shape,
            delta_min_shape: self.delta_min_shape,
        }
    }

    /// engineering/12 §5 §30: the five silt rows as the crate reads them. The
    /// table itself is session RAM and hash-exempt (OD-103).
    fn silt_config(self) -> si_silt::Config {
        si_silt::Config {
            lambda_silt: self.lambda_silt,
            level_max: self.silt_level_max,
            bump: self.silt_bump,
            theta: self.theta_silt,
            max: self.silt_max,
        }
    }
}

/// 11 §1 D5's fixture: the modulation artifact is G9's, and until it exists the
/// session conducts with a curve that modulates nothing. `reach` is the
/// registry's own placeholder, so even the fixture names no new number.
pub fn fixture_curve() -> Curve {
    Curve::fixture(CurvePoint {
        reach: species("FRONTIER_CAP"),
        decay_q12: si_core::quant::Q12_ONE,
        settle_shift_q12: 0,
    })
}

/// The identity the slot table answers queries with; the point table already
/// owns alias sets, so this is a borrow and not a second table.
///
/// Under CONFIG ENGINE the sealed surface index is a second member list for the
/// *same* folded key (07 §5: the fold may alias, it may never merge), and 04 §4's
/// lighting law already lights every member of an alias set with the shared
/// quantum. So the corpus enters the walk through that law and not beside it: the
/// union is sorted by handle, which is W.2's order — genome points first, because
/// they own `0 … P_gen−1`.
struct PointsResolver<'a> {
    points: &'a Points,
    frozen: Option<&'a Frozen>,
}

impl Resolve for PointsResolver<'_> {
    fn aliases(&self, key: &str) -> Vec<u32> {
        let mut members = self.points.aliases(key);
        if let Some(layer) = self.frozen {
            for point in layer.points_for_folded(key) {
                if !members.contains(&point) {
                    members.push(point);
                }
            }
        }
        members.sort_unstable();
        members
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Output {
    /// The renderer's surface, its label and the rows it stood on.
    Answer { surface: String, label: String, tag: String, citation: Vec<u32> },
    /// The ask-back: the owner's own words, and the queue row it wrote or reused.
    Ask { surface: String, question_id: u32, fresh: bool },
    /// A row was written; the panel echoes the act, not an answer.
    Committed { row_id: u32, shelf: u16, branch: &'static str },
    /// A designed refusal. `reason` is the code the mind log carries.
    Refused { reason: String },
    /// Nothing to do (a boring repeat, a grimm line, an ordinary input).
    Inert { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TickReport {
    pub t: i32,
    pub thought_id: u32,
    pub stage: u8,
    pub a_before: u8,
    pub a_after: u8,
    pub output: Output,
    pub rows_written: u32,
    pub salience_calls: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdleReport {
    pub t: i32,
    pub revisited: Vec<(u32, u32)>,
    pub derived: Vec<(u32, u32)>,
    /// 04 §19 S.2 step 3: the hyperpoint this quiet tick ignited on, if it fired.
    /// One quiet tick does at most one ignition, so this is at most `Some`.
    pub ignited: Option<u32>,
    /// 05 §15 `G10.e.2`: the rehearsal recovery this ignition caused, or `None`
    /// when the tick did not ignite. A readout of the RAM rehearsal pair, so it
    /// moves no durable byte and G10(b)'s digests stay untouched.
    pub rehearsal: Option<RehearsalShift>,
    /// 04 §33 AE.4 / OD-126 (B18): this quiet tick had the agenda empty and the
    /// rate bound open and was **suppressed by a declared window** — the tick did
    /// not think, and it is named rather than silent. `None` means the window
    /// denied nothing: either there is no window, or it was open, or the quiet law
    /// did not hold anyway. An open window never produces a tick by itself.
    pub suppressed_by_window: bool,
}

/// 05 §15 `G10.e.2`: how far the rehearsal reset moved the decay-deficit pull,
/// in Q12 drift units on the one clock curve (R-29). Two levels are measured
/// because they answer different questions: the rehearsed edges' own drift is the
/// recovery itself, while the point's pull is what the next draw actually reads —
/// a point's deficit is the furthest-drifted of its live arcs, so reheating one
/// arc cannot lower a pull another arc is holding up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RehearsalShift {
    /// The edges this cascade crossed. Zero means the idle walk spent its round on
    /// sub-quantum light and had no arc to rehearse, so the two readings below are
    /// both the floor and the recovery is not sized on this web.
    pub traversed: usize,
    /// The edges this cascade traversed, at their worst, as the draw read them at `t`.
    pub arc_before_q12: u16,
    /// The same edges at `t` after §S.7's reset — the recovery. The floor is 16,
    /// the Q12 step of the u8 curve at age 0, not 0.
    pub arc_after_q12: u16,
    /// The seed point's max-drift deficit as the draw read it at `t`.
    pub point_before_q12: u16,
    /// The seed point's deficit at `t + 1`, which is what the next quiet tick pulls on.
    pub point_next_q12: u16,
}

/// 04 §19 S.5: one rendered monologue line. It is a truthful rendering of what
/// lit in an idle cascade — the reverse face over the settle set, not generated
/// prose — so it carries the lit hyperpoints it stands on. A reader checks G10(d):
/// 1:1 onto lit nodes with their first-lit round, and no surface token whose
/// point did not light. Nothing here is a belief row and none reaches the owner
/// lane (`MONOLOGUE_OWNER_FACING` is off in v1, OPEN-23's lean).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonologueLine {
    pub t: i32,
    pub thought: u32,
    /// The ignited round-0 point (§S.4).
    pub seed_point: u32,
    /// The rendered surface — the lexicon words for the lit points, in display order.
    pub surface: String,
    /// `{node, round, edge}` citations (05 §15 G10.d.1): each displayed point, the
    /// round it first lit, and the arena offset of the edge that carried it there
    /// (`0` for the round-0 ignition seed, which lit without an edge).
    pub cites: Vec<(u32, u32, u32)>,
    /// The display points, so containment is checkable without re-walking.
    pub lit_points: Vec<u32>,
}

/// 04 §19 S.6–S.7: the idle-tick runtime state. The rate-bound counter and the
/// recency map live here so an idle run replays identically; they are derived from
/// the deterministic step trace and are **never** hash inputs — the only state an
/// idle tick may durably change is a `questions` row (S.2), which the hash already
/// covers. `per_window_override` is the V2 launch state: 05 §2 disables ignition by
/// setting the rate bound to zero in the harness launch state, so the engine's own
/// §S.6 check is what returns Nothing and the loop code is unchanged.
/// `weights_override` is the V3 state (`IGN_WEIGHT_BAKE` at its range floor).
/// Neither is a value this build invents.
#[derive(Clone, Debug)]
struct IdleState {
    last_lit: BTreeMap<u32, i32>,
    idles_this_window: u32,
    per_window_override: Option<u32>,
    weights_override: Option<si_walk::IdleWeights>,
}

impl IdleState {
    /// A fresh session ignites freely until the rate bound fills; no override is
    /// installed unless a variant asks. The counter and map are empty at birth.
    fn at_birth() -> Self {
        Self {
            last_lit: BTreeMap::new(),
            idles_this_window: 0,
            per_window_override: None,
            weights_override: None,
        }
    }
}

/// 10 §7 / 04 §28 Z: the sleep's runtime state. `quiet_run` is §19 S.1's predicate
/// counted — senses quiet, one tick at a time, reset by any sense line. It is
/// **derived RAM**: 01 §13 names the 13 mutable bytes of item 9 and the quiet-run
/// count is not among them, while the four the sleep does move (`sleeps_completed`,
/// `sleeps_interrupted`, `last_slept_at_t`, `sleep_pass_mask`) are core-block fields.
/// `cadence_override` and `freeze_armed` are 05 §2's **V9** launch state; the loop
/// code and the binary are unchanged by them, which is the whole point of the row.
/// `interrupt_before_pass` is 05 §18(g)'s owner frame appended at a named pass
/// boundary — a harness launch state, never a verb (`check_no_seed_control_surface`
/// refuses a "sleep now"), and `None` in every ordinary run because this build's
/// sense channel is synchronous: a line arrives as a tick, so the only frame that
/// can be pending *while* SI dreams is one the harness placed there on purpose.
#[derive(Clone, Debug)]
struct SleepState {
    config: si_sleep::SleepConfig,
    cadence_override: Option<u32>,
    freeze_armed: bool,
    quiet_run: u32,
    /// The logical tick step 12b last *attempted* a sleep at — derived RAM, and
    /// the reason `freeze()` is idempotent even for a night that was cut short,
    /// when `last_slept_at_t` (the durable half of the guard, OD-81) stayed put.
    ran_at: Option<i32>,
    interrupt_before_pass: Option<u8>,
    /// The `context` row of the sense tick in flight, so the lighting it produced
    /// can be attributed to the row that paid for it (04 §28 Z.2 P1).
    context_row: Option<(u32, i32)>,
    /// P1's input table: the round-0 vector each sense tick lit, kept per
    /// `context` row and pruned at every sleep. `si-sleep` may not import
    /// `si-front` or `si-read` ([02 §13](engineering)), so the session that owns
    /// the decode owns the replay lights.
    replay: Vec<si_sleep::ReplayEvent>,
    /// `sleep.jsonl`'s lines, and `sleep_diff.json`'s rows, as the loop wrote them.
    lines: Vec<serde_json::Value>,
    ledger: Vec<si_sleep::SleepLedgerEntry>,
}

impl SleepState {
    fn at_birth() -> Self {
        Self {
            config: si_sleep::SleepConfig::from_registry(),
            cadence_override: None,
            // Trigger 2 is a property of the checkpoint path, not a switch: armed
            // until a launch state says the freeze trigger is not taken (V9).
            freeze_armed: true,
            quiet_run: 0,
            ran_at: None,
            interrupt_before_pass: None,
            context_row: None,
            replay: Vec::new(),
            lines: Vec::new(),
            ledger: Vec::new(),
        }
    }
}

pub struct Session {
    seed: u64,
    core: CoreBlock,
    store: Lattice,
    temperament: Temperament,
    tags: VersionTags,
    rng: Rng,
    advances: Vec<(i32, u32)>,
    settle: Vec<(u32, u16)>,
    /// The route counts of *this* tick's read, as the walk reported them, and
    /// `None` once a tick has begun and not yet read. Session RAM only: never
    /// captured, never hashed, never an input to a later step. It exists
    /// because 05 §11 `G6.b.1` asks for both webs surfaced in one output frame,
    /// and the number that says so is computed at the read and gone after it.
    route: Option<si_read::RouteCounts>,
    /// 13 §6.4: what step 9a did on *this* tick, as RAM the harness reads and the
    /// state hash never sees — the same standing [`last_route`] has. `None` is a
    /// tick that never entered the arm: `TONGUE` uninstalled, no cited answer row,
    /// or a build without the feature. It is cleared at the top of every tick for
    /// the same reason `route` is: a step that spoke with no voice must not report
    /// the previous line's draw.
    tongue_tick: Option<TongueTick>,
    trajectory: Vec<(u16, u8)>,
    clock: Clock,
    a: Integrator,
    params: Params,
    curve: Curve,
    walk: WalkConstants,
    events: EventLog,
    stats: GateStats,
    thought_id: u32,
    prov_session: u32,
    /// The row the last output stood on: a cited belief or the `questions` row an
    /// ask-back wrote. `P_CMD_C` binds to it (G1.b.1's `bind_output_id ≠ 0`).
    last_output_row: Option<(u16, u32)>,
    /// The admitted sense stream, for `transcript.txt`.
    transcript: Vec<(i32, Source, String)>,
    /// The owner-facing lines, for the same artifact.
    rendered: Vec<(i32, String)>,
    /// 04 §19: the idle-tick runtime state and its monologue lines (separate lane).
    idle: IdleState,
    monologue: Vec<MonologueLine>,
    /// 04 §28: step 12b's runtime state — the quiet-run counter, P1's replay
    /// lights, and the sleep's own report lines.
    sleep: SleepState,
    /// engineering/12 §3: the arc's registry rows as they were launched.
    arc: ArcConfig,
    /// docs/22 §22.3 stage 2 — session RAM, hash-exempt by OD-98.
    crystallizer: si_crystallize::Crystallizer,
    /// `crystallize.jsonl`: one line per candidate transition (OD-98's audit trail).
    crystallize_lines: Vec<serde_json::Value>,
    /// `mindlog.json`'s `frame_log[]`: the frame scratch each thought exported.
    frame_logs: Vec<serde_json::Value>,
    /// docs/23 §23.3 / 12 §30: the one session RAM table, keyed by conversation
    /// and hash-exempt (OD-103). It writes nothing and proposes nothing.
    silt: si_silt::Silt,
    /// `silt.jsonl`: one audit line per tick (OD-103).
    silt_lines: Vec<serde_json::Value>,
    /// 12 §6.1 V12: the traversal row is installed and the derived index is
    /// forced empty. A launch state like V2/V3/V9, never a stored byte.
    episodic_index: bool,
    /// `gates/G17/relight_log.jsonl`: one line per fired tug, nil for none.
    relight_lines: Vec<serde_json::Value>,
    /// docs/25 §25.3 stage 2 — session RAM, hash-exempt by OD-117.
    skiller: si_skills::Skiller,
    /// `skills.jsonl`: one line per shape transition and per mint (OD-117).
    skill_lines: Vec<serde_json::Value>,
    /// 12 §2.1 / 04 §33 AE.1 (B18): the world's session RAM — the two registered
    /// ids, the (empty at birth — OD-125) grant view, the declared idle window
    /// (OD-126) and the conversation the transport names. None of it is a row, a
    /// hash item, an export field or a checkpoint byte (OD-123).
    world: World,
}

/// 12 §2.6: the world's session RAM, with the two launch-state ids. A scenario
/// declares these; no container carries them. **There is no `class` field**: the
/// class is re-derived at every use (`Session::class_of`), because AE.1 stores it
/// nowhere — not in a row, not in a cache.
#[derive(Clone, Debug)]
pub struct World {
    pub owner_identity: si_core::world::Identity,
    pub grimm_identity: si_core::world::Identity,
    pub grants: si_core::world::Grants,
    pub window: Option<si_core::world::Window>,
    pub conversation: u32,
    /// The identity carried by the frame being read, or the id the voice tag
    /// stands for when a caller hands `tick` a bare line (12 §2.1: the
    /// `speaker_identity` byte-pair is the frame's, and `owner`/`grimm` are the
    /// two identities that pre-date it).
    pub line_identity: si_core::world::Identity,
}

impl World {
    /// The birth state: the two literal indices as the registered ids (`owner` 0,
    /// `grimm` 1, 01 §3.2), no grant, no window, conversation `0`, the owner's own
    /// line. Under `WORLD_TIERS = owner-only` this is the only world a session can
    /// have, and `class_of` never answers `Trusted` (OD-125).
    pub fn at_birth() -> Self {
        Self {
            owner_identity: si_core::world::IDENTITY_OWNER,
            grimm_identity: si_core::world::IDENTITY_GRIMM,
            grants: si_core::world::Grants::birth(si_core::world::IDENTITY_GRIMM),
            window: None,
            conversation: 0,
            line_identity: si_core::world::IDENTITY_OWNER,
        }
    }
}

/// 01 §9: one decoded sense frame — the line, the identity that spoke it, the
/// thread it arrived on and the voice tag the receiving module carried. The two
/// transport fields are what B18 adds to the body; `source` stays the input 04
/// §12 step 0 demands, never a computation from the text, so a granted or
/// stranger voice arrives as `Grimm` with its own identity and the class function
/// tells the two apart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SenseFrame {
    pub text: String,
    pub identity: si_core::world::Identity,
    pub conversation: u32,
    pub source: Source,
}

/// 00 §5 / 01 §9: the inbox reader's named refusal. A resume into an
/// older-versioned inbox is a version mismatch, never a silent re-parse of bytes
/// that mean something else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InboxRefusal {
    VersionMismatch { stored: u32 },
    BodyTooShort { seq: u64 },
}

impl std::fmt::Display for InboxRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VersionMismatch { stored } => write!(
                f,
                "sense inbox container version {stored} is not {}",
                si_core::container::Format::StreamRecord.container_version()
            ),
            Self::BodyTooShort { seq } => write!(
                f,
                "sense frame {seq} carries no word beside its {}-byte transport prefix",
                si_core::eventlog::SENSE_TRANSPORT_BYTES
            ),
        }
    }
}

impl std::error::Error for InboxRefusal {}

pub enum OpenOutcome {
    Opened(Box<Session>),
    Refused(ResumeRefusal),
}

/// 01 §9: the sense inbox's two outcomes. A version refusal is a branch, not an
/// error string, because DoD 2 names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InboxOutcome {
    /// The frames the lane carries, plus the byte count of a tail cut mid-append — the
    /// part of the lane that was never written, named rather than swallowed
    /// ([08 §2.2]'s dead-window row).
    Accepted { frames: Vec<SenseFrame>, torn_tail_bytes: usize },
    Refused(InboxRefusal),
}

/// 01 §9's frame `source u8`, the same two values the CLI's feeder writes.
pub const SOURCE_OWNER_BYTE: u8 = 0;

/// One frame the lane carried, heard: the words the loop was handed and the tick
/// report they produced.
#[derive(Clone, Debug)]
pub struct HeardFrame {
    pub text: String,
    pub report: TickReport,
}

/// 09 §8's reader half, as the loop runs it. `Heard` counts its own torn tail the
/// way [`InboxOutcome::Accepted`] does, so a caller can say what it applied without
/// re-reading the lane.
#[derive(Clone, Debug)]
pub enum InboxDrain {
    Heard { frames: Vec<HeardFrame>, torn_tail_bytes: usize },
    Refused(InboxRefusal),
}

impl Session {
    /// 04 §25 W.0, CONFIG SI: derive the temperament from the seed, compare it
    /// against the core block's digest, and wake on an empty lattice. No
    /// container is read and no frozen path is formed.
    pub fn new(seed: u64) -> Result<Self> {
        Self::at_birth(seed, None)
    }

    /// 04 §25 step 2's W.0→W.4 for CONFIG ENGINE: the same derive-and-compare
    /// birth, then the pinned layer is the genome this individual is born into —
    /// its points own handles `0 … P_gen−1` before the first live arrival, and its
    /// tag is sealed into the version tags so the first snapshot carries the base
    /// the next resume will refuse against. The layer is held by reference: the
    /// merged web is read from the mapped artifact, never copied (§9's load
    /// budget).
    pub fn wake_engine(seed: u64, pinned: std::rc::Rc<Frozen>) -> Result<Self> {
        Self::at_birth(seed, Some(pinned))
    }

    fn at_birth(seed: u64, pinned: Option<std::rc::Rc<Frozen>>) -> Result<Self> {
        let arc = ArcConfig::from_registry();
        let temperament = seed::derive_temperament(seed);
        let core = CoreBlock::at_birth(seed);
        if core.seed_format != SEED_FORMAT {
            bail!("core block seed_format {:#06x} is not {SEED_FORMAT:#06x}", core.seed_format);
        }
        if core.temperament_digest != temperament.digest() {
            bail!(
                "W.0 derive-and-compare failed: core digest {:#018x} is not the re-derived {:#018x}",
                core.temperament_digest,
                temperament.digest()
            );
        }
        let mut store = Lattice::new();
        let mut tags = VersionTags::at_birth(seed);
        if let Some(layer) = pinned {
            tags.genome_t0_tag = layer.version_tag();
            store.attach_frozen(layer)?;
        }
        let mut session = Self {
            seed,
            core,
            store,
            temperament,
            tags,
            rng: seed::runtime_stream(seed),
            advances: Vec::new(),
            settle: Vec::new(),
            route: None,
            tongue_tick: None,
            trajectory: Vec::new(),
            clock: Clock::new(0),
            a: Integrator::at_birth(),
            params: Params::from_temperament(&temperament),
            curve: fixture_curve(),
            walk: WalkConstants::from_registry(),
            events: EventLog::new(),
            stats: GateStats::default(),
            thought_id: 0,
            prov_session: 1,
            last_output_row: None,
            transcript: Vec::new(),
            rendered: Vec::new(),
            idle: IdleState::at_birth(),
            monologue: Vec::new(),
            sleep: SleepState::at_birth(),
            arc,
            crystallizer: si_crystallize::Crystallizer::new(arc.crystallize_config()),
            crystallize_lines: Vec::new(),
            frame_logs: Vec::new(),
            silt: si_silt::Silt::new(arc.silt_config()),
            silt_lines: Vec::new(),
            episodic_index: true,
            relight_lines: Vec::new(),
            skiller: si_skills::Skiller::new(arc.skills_config()),
            skill_lines: Vec::new(),
            world: World::at_birth(),
        };
        // 08 §2.1 row 20's production site: "wake, before the first round of the
        // first thought". It is `fingerprint()`'s own table as a frame, so the
        // panel and the file are one read of one derivation — and the viewer never
        // calls `derive` itself, which 08 §11 names as a rule rather than a missing
        // dependency.
        let rows: Vec<String> = DERIV_ORDER
            .iter()
            .map(|id| {
                let name = id.registry_name();
                let row = consts::row(name).expect("sealed draw has a registry row");
                format!(
                    "[{:?},{},[{},{}]]",
                    name,
                    session.temperament.get(*id),
                    row.lo,
                    row.hi
                )
            })
            .collect();
        session.log(
            0,
            SOURCE_OWNER_BYTE,
            EV_TEMPERAMENT,
            format!(
                "{{\"birth_seed\":\"0x{:016x}\",\"seed_format\":\"0x{:04x}\",\
                 \"deriv_order_len\":{},\"rows\":[{}],\"temperament_digest\":\"0x{:016x}\"}}",
                session.seed,
                session.core.seed_format,
                session.core.deriv_order_len,
                rows.join(","),
                session.core.temperament_digest
            ),
        );
        Ok(session)
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn core(&self) -> &CoreBlock {
        &self.core
    }

    pub fn temperament(&self) -> &Temperament {
        &self.temperament
    }

    pub fn store(&self) -> &Lattice {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut Lattice {
        &mut self.store
    }

    pub fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    pub fn walk(&self) -> &WalkConstants {
        &self.walk
    }

    /// 06 §5.1's `ρ`: the walk's own modulation curve, read-only. `G13(l).3`
    /// bounds a night's edge touches from the run's curve rather than assuming
    /// the cap, and the curve is derived state, so reading it writes nothing.
    pub fn curve(&self) -> &Curve {
        &self.curve
    }

    /// 05 §4: the sweep runner installs the run's declared constant list at
    /// launch. It reaches the arithmetic the registry owns and nothing else —
    /// not `a`, not a row, not a term, not a stage (05 §22) — and the loop's
    /// code path is identical to a run at the registry's own values.
    pub fn install_launch_constants(&mut self, params: Params, walk: WalkConstants) {
        self.params = params;
        self.walk = walk;
    }

    /// engineering/12 §3.2: the launch channel's arc half. A Ruling row installs
    /// through the same door a SPECIES row does; the class check lives in the
    /// harness, and `Temperament`/`Law`/`Derived` stay refused there.
    pub fn install_arc_config(&mut self, arc: ArcConfig) {
        self.arc = arc;
        self.crystallizer = si_crystallize::Crystallizer::new(arc.crystallize_config());
        self.skiller = si_skills::Skiller::new(arc.skills_config());
        // The launch channel installs before the first tick, so an empty table is
        // the right state at that point; the rows it carries are the only thing
        // that changes.
        self.silt = si_silt::Silt::new(arc.silt_config());
    }

    pub fn arc(&self) -> ArcConfig {
        self.arc
    }

    /// OD-98's audit trail: one line per candidate transition.
    pub fn crystallize_lines(&self) -> &[serde_json::Value] {
        &self.crystallize_lines
    }

    /// 12 §2.6: the frame scratch each thought exported. Hashed nowhere.
    pub fn frame_logs(&self) -> &[serde_json::Value] {
        &self.frame_logs
    }

    /// `mindlog.json`'s `nomination_views{}`: the candidate table as the log
    /// shows it. Session RAM, never a durable byte.
    pub fn nomination_views(&self) -> serde_json::Value {
        let mut views = serde_json::Map::new();
        for (hash, candidate) in self.crystallizer.table() {
            views.insert(
                format!("{hash:016x}"),
                serde_json::json!({
                    "members": candidate.members,
                    "shared_fillers": candidate.shared_fillers,
                    "recurrence": candidate.recurrence,
                    "frames_seen": candidate.frames_seen,
                    "first_seen_t": candidate.first_seen_t,
                    "last_seen_t": candidate.last_seen_t,
                    "nominated_at": candidate.nominated_at,
                }),
            );
        }
        serde_json::Value::Object(views)
    }

    /// 12 §6.4 G15(c): the candidate table's canonical bytes, the same encoding
    /// `crystallize_trace_table` produces, so a replay is compared byte for byte.
    pub fn crystallize_table_bytes(&self) -> Vec<u8> {
        self.crystallizer.table_bytes()
    }

    /// `skills.jsonl` (OD-117): one line per shape transition, ask and mint.
    pub fn skill_lines(&self) -> &[serde_json::Value] {
        &self.skill_lines
    }

    /// 12 §6.4 G18(c): the shape table's canonical bytes — the same encoding
    /// `skills_trace_table` produces, so the frame-trace-fixed pair is compared
    /// byte for byte and `a` is nowhere in either call.
    pub fn skills_table_bytes(&self) -> Vec<u8> {
        self.skiller.table_bytes()
    }

    /// 12 §2.6's `shape candidate` record as session RAM shows it. Usage counters
    /// are derived here, from the rows plus the log — never stored (OD-120).
    pub fn shape_views(&self) -> serde_json::Value {
        let mut views = serde_json::Map::new();
        for (template, candidate) in self.skiller.table() {
            views.insert(
                format!("{template:08x}"),
                serde_json::json!({
                    "role_graph_template": candidate.template_text,
                    "root_point": candidate.root_point,
                    "filler_sets_seen": candidate.filler_sets_seen(),
                    "recurrence": candidate.recurrence,
                    "outcome_tags": candidate.outcome_tags,
                    "first_seen_t": candidate.first_seen_t,
                    "last_seen_t": candidate.last_seen_t,
                    "opened": candidate.opened,
                    "nominated_at": candidate.nominated_at,
                }),
            );
        }
        serde_json::Value::Object(views)
    }

    /// G18(c)'s fixed-trace half: a recorded `frame_log[]` trace through a
    /// **fresh** `Skiller`, with the run's own eventlog rows as the outcome
    /// input. `a` is not an argument, so two `a` regimes are the same call.
    pub fn skills_trace_table(
        &self,
        frames: &[serde_json::Value],
        events: &[si_core::eventlog::Event],
    ) -> Result<Vec<u8>> {
        let mut detector = si_skills::Skiller::new(self.arc.skills_config());
        for frame in frames {
            let log = si_core::frame::FrameLog::from_json(frame)
                .map_err(|error| anyhow::anyhow!("frame log: {error}"))?;
            let t = frame["t"].as_i64().unwrap_or(0) as i32;
            // The cursor the live detector read (observe_frame's step-7 note):
            // a replay that saw rows this tick appended after step 7 would read
            // an outcome the live table never could.
            let seen = frame["events_seen"].as_u64().unwrap_or(events.len() as u64) as usize;
            detector.observe(&log, &events[..seen.min(events.len())], t)?;
        }
        Ok(detector.table_bytes())
    }

    /// The forwarding rule's witness for G18(f): the shape table's own counts.
    pub fn skill_counts(&self) -> (u32, u32, u32) {
        (self.skiller.shapes(), self.skiller.opened_candidates(), self.skiller.forwarded_frames())
    }

    /// 12 §2.6's `skill usage counters`: `{times_consulted, outcome_tags}` per
    /// live `skills` row, **computed** from the rows plus this session's own log
    /// (OD-120). Nothing stores a usage number, so nothing can drift from it: the
    /// only place the count exists is the derived block `mindlog.json` publishes.
    pub fn skill_usage_json(&self) -> Result<serde_json::Value> {
        let mut consults: std::collections::BTreeMap<u32, u32> = std::collections::BTreeMap::new();
        for line in &self.skill_lines {
            if line["transition"].as_str() != Some("consult") {
                continue;
            }
            if let Some(roots) = line["roots"].as_array() {
                for root in roots.iter().filter_map(|value| value.as_u64()) {
                    *consults.entry(root as u32).or_insert(0) += 1;
                }
            }
        }
        let now = self.clock.now();
        let mut out = Vec::new();
        for (row_id, record) in self.store.active_at(SHELF_SKILLS, now)? {
            if !record.envelope.is_live() || record.body.status != 0 {
                continue;
            }
            let root = record.body.key_point;
            let template_text = self
                .store
                .interner()
                .get(record.body.stim_signature)
                .map(|entry| entry.verbatim.clone())
                .unwrap_or_default();
            let outcome_tags = self
                .skiller
                .table()
                .values()
                .find(|candidate| candidate.root_point == root)
                .map(|candidate| candidate.outcome_tags.clone())
                .unwrap_or_default();
            out.push(serde_json::json!({
                "row": row_id,
                "root_point": root,
                "template": format!("{:08x}", si_skills::template_id(&template_text)),
                "template_text": template_text,
                "times_consulted": consults.get(&root).copied().unwrap_or(0),
                "outcome_tags": outcome_tags,
            }));
        }
        Ok(serde_json::Value::Array(out))
    }

    /// 04 §32 AD.5's read of the installed rows, for the report: `off` never
    /// consults it, and `on` uses it only as the walk's iteration order.
    pub fn composition_hint(&self) -> Result<Vec<u32>> {
        si_skills::composition_preference(&self.store, self.clock.now())
    }

    /// 12 §4.1: silt is keyed by conversation from its first build (OD-109). From
    /// B18 the key is the `conversation u32le` the transport carries; a scenario
    /// built before it — and every line handed to `tick` without a frame — is the
    /// one conversation `0`, which is what keeps silt's pre-B18 table untouched.
    fn conversation_key(&self) -> u32 {
        self.world.conversation
    }

    /// The thread key this session is currently on — the value a written frame
    /// body must carry (01 §9). Reading it is not caching a class: the transport
    /// number and the derived class are different objects.
    pub fn conversation(&self) -> u32 {
        self.world.conversation
    }

    /// 04 §33 AE.1: the class of one identity, **derived here and now**. There is
    /// no cached class on the session: the grant view is read with this tick's
    /// clock so a revoked grant has no standing, and the answer is a return value
    /// (docs/12 rule (ix), OD-123).
    pub fn class_of(&self, identity: si_core::world::Identity, t: i32) -> si_core::world::SpeakerClass {
        si_core::world::class(
            identity,
            self.world.owner_identity,
            &self.world.grants.clone().with_now(t),
        )
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    /// The launch state a scenario declares (`identity <u64> owner|grimm`). It is
    /// an installation, never a read: nothing in the build derives an id from text.
    pub fn set_registered_identities(
        &mut self,
        owner_identity: si_core::world::Identity,
        grimm_identity: si_core::world::Identity,
    ) {
        self.world.owner_identity = owner_identity;
        self.world.grimm_identity = grimm_identity;
        self.world.line_identity = owner_identity;
        // The birth view's grimm id travels with the registration: the class
        // function's second arm compares against the *registered* id.
        self.world.grants = si_core::world::Grants::birth(grimm_identity);
    }

    /// 04 §33 AE.1's owner-lane-only writer. The only caller in this build is the
    /// scenario's `grant` declaration, which arrives through the owner's own act
    /// (docs/26 §26.1); no transport, adapter or `/trust` parser reaches it
    /// (12 §11's refusals, OD-125).
    pub fn grant(&mut self, row: si_core::world::GrantRow) -> si_core::world::Identity {
        self.world.grants.grant(row);
        row.subject
    }

    /// The revocation appends its own mark; the grant row stays citable (I4).
    pub fn revoke(
        &mut self,
        subject: si_core::world::Identity,
        right: si_core::world::Right,
        t: i32,
    ) {
        self.world.grants.revoke(subject, right, t);
    }

    /// docs/26 §26.4: the declared idle window. OD-126 gives it exactly one use,
    /// `window_denies`, below.
    pub fn set_window(&mut self, window: Option<si_core::world::Window>) {
        self.world.window = window;
    }

    /// OD-126 / 04 §33 AE.4: the window's **only** consultation in the build — a
    /// deny conjunct on step 12's quiet-and-empty-agenda predicate. It has no path
    /// to `ignite`, `sleep::due`, `clock.advance` or a frame read, and an open
    /// window makes nothing happen (`check_windows_suppress_only`).
    fn window_denies(&self, t: i32) -> bool {
        self.world.window.map(|window| window.denies(t)).unwrap_or(false)
    }

    /// 01 §9: read a `sense_inbox.siro` lane as the frames it carries. The refusal
    /// is the version law (00 §5): a lane written before the transport byte is a
    /// named mismatch, never a re-parse of bytes that mean something else.
    ///
    /// A lane whose **last frame is cut** reads as the frames before it, because
    /// [08 §2.2]'s dead-window row rules the effect of a killed viewer: *no effect*. A
    /// frame that never finished being written was never written, and a loop that
    /// refused to wake because a window died would be a loop that notices its observer —
    /// 08 §1's HEADLESS law, not a robustness nicety. The cut is reported, never hidden.
    pub fn read_sense_inbox(bytes: &[u8]) -> Result<InboxOutcome> {
        let container = si_core::container::Container::decode(bytes)?;
        if container.format != si_core::container::Format::StreamRecord {
            bail!(
                "the sense inbox is a {} lane, not a stream",
                container.format.name()
            );
        }
        let want = si_core::container::Format::StreamRecord.container_version();
        if container.version != want {
            return Ok(InboxOutcome::Refused(InboxRefusal::VersionMismatch {
                stored: container.version,
            }));
        }
        let mut pos = 0usize;
        let mut out = Vec::new();
        while pos + 4 <= container.payload.len() {
            let len =
                u32::from_le_bytes(container.payload[pos..pos + 4].try_into().expect("4 bytes"))
                    as usize;
            let end = (pos + 4)
                .checked_add(len)
                .ok_or_else(|| anyhow::anyhow!("an inbox frame length overflows its payload"))?;
            if end > container.payload.len() {
                break;
            }
            let event = si_core::eventlog::Event::decode(&container.payload[pos + 4..end])?;
            pos = end;
            let (identity, conversation, text) = match si_core::eventlog::decode_sense_body(
                &event.body,
            ) {
                Ok(decoded) => decoded,
                Err(_) => {
                    return Ok(InboxOutcome::Refused(InboxRefusal::BodyTooShort { seq: event.seq }))
                }
            };
            let text = String::from_utf8_lossy(&text).to_string();
            let source = if event.source == SOURCE_OWNER_BYTE {
                Source::Owner
            } else {
                Source::Grimm
            };
            out.push(SenseFrame { text, identity, conversation, source });
        }
        Ok(InboxOutcome::Accepted { frames: out, torn_tail_bytes: container.payload.len() - pos })
    }

    /// 01 §9: one admitted frame, through the same steps 1–13 a keystroke takes.
    /// The identity and the thread are carried in, never read out of the text.
    pub fn tick_frame(&mut self, frame: &SenseFrame) -> Result<TickReport> {
        self.world.conversation = frame.conversation;
        self.world.line_identity = frame.identity;
        self.tick_line(&frame.text, frame.source)
    }

    /// 09 §8's second half, in the loop's own crate: **the ordinary loop is the
    /// bulk channel's reader**. The lane a `/teach` feeder wrote is heard here —
    /// [`Session::read_sense_inbox`] for the frames, then one [`Session::tick_frame`]
    /// each, in file order — so the feeder's lines take the same steps 1–13 a
    /// keystroke takes and `G1.u.2`'s byte-identity is a property of one code path
    /// rather than of two writers. A refusal is returned, not applied: the session
    /// is left exactly as an absent lane leaves it (01 §9).
    pub fn hear_sense_inbox(&mut self, bytes: &[u8]) -> Result<InboxDrain> {
        match Self::read_sense_inbox(bytes)? {
            InboxOutcome::Refused(refusal) => Ok(InboxDrain::Refused(refusal)),
            InboxOutcome::Accepted { frames, torn_tail_bytes } => {
                let mut heard = Vec::with_capacity(frames.len());
                for frame in &frames {
                    heard.push(HeardFrame { text: frame.text.clone(), report: self.tick_frame(frame)? });
                }
                Ok(InboxDrain::Heard { frames: heard, torn_tail_bytes })
            }
        }
    }

    /// 09 §8's first half, in the same crate as its second: the bytes a `/teach`
    /// bulk feeder puts in the lane. It opens no file and touches no store — it
    /// turns lines into a container, and the caller (09 §10 says exactly one:
    /// `si-cli feed`) writes them — but both halves of `G1.u.2`'s equivalence are
    /// built here, so the lane a gate feeds and the lane the owner feeds are the
    /// same bytes from the same writer, not two writers of one idea (05 §1).
    ///
    /// A lane already on disk is walked **strictly**: every frame must be whole.
    /// [`Session::read_sense_inbox`] stops at the last whole frame on purpose,
    /// because [08 §2.2]'s dead-window row forbids a killed viewer from breaking a
    /// wake; a feeder has no such license, because a line appended behind a hole
    /// would be buried by it — so a torn lane is a named error here and nothing is
    /// produced.
    ///
    /// The frames carry the registered owner's id on the transport prefix and the
    /// session's one thread (01 §9, OD-136): a feeder speaks only for the owner.
    pub fn encode_sense_inbox(prior: &[u8], lines: &[&str], run_id: &str) -> Result<(Vec<u8>, usize)> {
        let mut events: Vec<si_core::eventlog::Event> = Vec::new();
        if !prior.is_empty() {
            let container = si_core::container::Container::decode(prior)?;
            let mut pos = 0usize;
            while pos + 4 <= container.payload.len() {
                let len = u32::from_le_bytes(
                    container.payload[pos..pos + 4].try_into().expect("4 bytes"),
                ) as usize;
                let end = (pos + 4)
                    .checked_add(len)
                    .ok_or_else(|| anyhow::anyhow!("an inbox frame length overflows its payload"))?;
                if end > container.payload.len() {
                    bail!(
                        "the lane on disk is torn: its last frame is {} byte(s) short of the \
                         length it declares — a feeder that wrote past the hole would bury \
                         these lines behind it",
                        end - container.payload.len()
                    );
                }
                events.push(si_core::eventlog::Event::decode(
                    &container.payload[pos + 4..end],
                )?);
                pos = end;
            }
        }
        let mut seq = events.len() as u64;
        let mut t = events.last().map(|event| event.t + 1).unwrap_or(0);
        for line in lines {
            events.push(si_core::eventlog::Event {
                seq,
                t,
                source: SOURCE_OWNER_BYTE,
                kind: frame::SENSE_ECHO,
                body: si_core::eventlog::encode_sense_body(
                    line.as_bytes(),
                    si_core::world::IDENTITY_OWNER,
                    0,
                )?,
            });
            seq += 1;
            t += 1;
        }
        let mut payload = Vec::new();
        for event in &events {
            let encoded = event.encode();
            payload.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
            payload.extend_from_slice(&encoded);
        }
        let header = serde_json::json!({
            "lane": "sense_inbox",
            "frames": events.len(),
            "run_id": run_id,
        });
        let container = si_core::container::Container::new(
            si_core::container::Format::StreamRecord,
            header,
            payload,
        );
        Ok((container.encode()?, events.len()))
    }

    /// `silt.jsonl`: one audit line per tick (OD-103). It is a log, never an
    /// input: nothing in the loop reads these lines back.
    pub fn silt_lines(&self) -> &[serde_json::Value] {
        &self.silt_lines
    }

    /// 04 §31 AC.2 / 05 §23.3(d): `gates/G17/relight_log.jsonl` — one line per
    /// fired origin tug, with the tick and thought that fired it. A log, never an
    /// input; an empty log is a run where no tug fired.
    pub fn relight_lines(&self) -> &[serde_json::Value] {
        &self.relight_lines
    }

    /// 12 §2.6: the frame's `event_views{}` block for `mindlog.json`. Assembled
    /// from the lattice at `anchor`; every field is already a hashed byte, so the
    /// block is an observation and nothing more.
    pub fn event_views_json(&self, anchor: i32) -> Result<serde_json::Value> {
        let mut views = serde_json::Map::new();
        for view in si_read::event_views(&self.store, anchor)? {
            views.insert(
                format!("{}", view.event_id),
                serde_json::json!({
                    "belief_rows": view.belief_rows,
                    "stamps": view.stamps,
                    "a_at": view.a_at,
                    "t": view.t,
                    "speaker_identity": view.speaker_identity,
                }),
            );
        }
        Ok(serde_json::Value::Object(views))
    }

    /// `mindlog.json`'s `silt_view{}`: the table as the log shows it, per
    /// conversation. Session RAM, never a durable byte.
    pub fn silt_view_json(&self) -> serde_json::Value {
        let mut conversations = serde_json::Map::new();
        for conversation in self.silt.conversations() {
            let readout = self.silt.readout(conversation);
            let levels: Vec<serde_json::Value> = readout
                .levels
                .iter()
                .map(|(point, level)| serde_json::json!([point, level]))
                .collect();
            conversations.insert(
                conversation.to_string(),
                serde_json::json!({
                    "levels": levels,
                    "active": readout.active(),
                    "over_bound": readout.over_bound(),
                    "max": readout.max,
                }),
            );
        }
        serde_json::Value::Object(conversations)
    }

    pub fn silt_level(&self, conversation: u32, point: u32) -> u16 {
        self.silt.level(conversation, point)
    }

    pub fn silt_is_empty(&self) -> bool {
        self.silt.is_empty()
    }

    /// 12 §4.1 / §30 AB.1 step 7's tail: deposit every distinct point this frame
    /// lit that is live-layer knowledge. `si-silt` never sees the store; the
    /// caller decides *which* handles deposit, and the table keeps only handles.
    fn deposit_frame(&mut self, t: i32, log: &si_core::frame::FrameLog) -> Result<()> {
        let conversation = self.conversation_key();
        for point in self.depositable_points(log)? {
            self.silt.deposit(conversation, point, t);
        }
        Ok(())
    }

    /// docs/23 §23.3: "when a hyperpoint lights in a frame … frozen rows never
    /// deposit". A point qualifies when a live row on an L3 shelf owns it.
    fn depositable_points(&self, log: &si_core::frame::FrameLog) -> Result<Vec<u32>> {
        let mut live: BTreeSet<u32> = BTreeSet::new();
        for (_, record) in self.store.rows()? {
            if !record.envelope.is_live() {
                continue;
            }
            let l3 = si_core::shelves::entry(record.envelope.shelf_x)
                .map(|entry| entry.layer == si_core::shelves::Layer::L3Live)
                .unwrap_or(false);
            if l3 {
                live.insert(record.body.key_point);
            }
        }
        let mut points: Vec<u32> = log
            .lights
            .iter()
            .map(|light| light.point)
            .filter(|point| live.contains(point))
            .collect();
        points.sort_unstable();
        points.dedup();
        Ok(points)
    }

    /// One `silt.jsonl` line for the tick that is closing. `active` is the entry
    /// count and `over_bound` is `active > SILT_MAX` — the diagnostic bound,
    /// reported and never enforced (docs/23 §23.3, 12 §30 AB.3).
    fn log_silt(&mut self, t: i32) {
        let conversation = self.conversation_key();
        let readout = self.silt.readout(conversation);
        self.silt_lines.push(serde_json::json!({
            "t": t,
            "conversation": conversation,
            "active": readout.active(),
            "over_bound": readout.over_bound(),
            "max": readout.max,
            "levels": readout.levels.iter().map(|(point, level)| serde_json::json!([point, level])).collect::<Vec<_>>(),
        }));
    }

    /// 05 §4: the sweep's declared point for the six sleep rows of 03 §5.1. It
    /// installs the same way a walk parameter does — the registry owns the box and
    /// `si_sleep::launch_ok` is the class check — and the loop's code path is
    /// identical at any point of it.
    pub fn install_sleep_config(&mut self, config: si_sleep::SleepConfig) {
        self.sleep.config = config;
    }

    pub fn sleep_config(&self) -> si_sleep::SleepConfig {
        self.sleep.config
    }

    /// 05 §18(g): "an owner frame appended at each of the four pass boundaries, one
    /// run per boundary". A launch state, set by the harness — there is no sleep
    /// verb and no sleep button (`check_no_seed_control_surface`), because a frame
    /// *arriving* is the sense channel doing its job while a *summoned* night would
    /// be an external setter of SI's inner state (I8).
    pub fn set_sleep_interrupt(&mut self, pass: Option<u8>) {
        self.sleep.interrupt_before_pass = pass;
    }

    /// `sleep.jsonl`: one line per pass and one per sleep, each verbatim 10 §8's
    /// field set. Also the payload of event-log kind 19 (08 §2.1).
    pub fn sleep_lines(&self) -> &[serde_json::Value] {
        &self.sleep.lines
    }

    /// `sleep_diff.json`'s rows: every OWNED edge P3 read, before and after.
    pub fn sleep_ledger(&self) -> &[si_sleep::SleepLedgerEntry] {
        &self.sleep.ledger
    }

    /// §19 S.1's predicate, counted. Derived RAM, never a durable byte (01 §13).
    pub fn quiet_run(&self) -> u32 {
        self.sleep.quiet_run
    }

    /// 06 §5.1's cadence readout for O11: the count pair, not a countdown.
    pub fn sleeps_completed(&self) -> u32 {
        self.core.sleeps_completed
    }

    pub fn a(&self) -> u8 {
        self.a.a()
    }

    pub fn now(&self) -> i32 {
        self.clock.now()
    }

    pub fn events(&self) -> &EventLog {
        &self.events
    }

    pub fn stats(&self) -> GateStats {
        self.stats
    }

    pub fn transcript(&self) -> &[(i32, Source, String)] {
        &self.transcript
    }

    pub fn rendered(&self) -> &[(i32, String)] {
        &self.rendered
    }

    /// 04 §19 S.5 / 02 §7: the monologue lane — the rendered idle cascades. The
    /// runner writes these to `monologue.jsonl`; none of them enters the owner
    /// lane (`MONOLOGUE_OWNER_FACING` is off in v1, OPEN-23's lean).
    pub fn monologue(&self) -> &[MonologueLine] {
        &self.monologue
    }

    /// 05 §2: the variant launch state for the idle tick. `V2` disables ignition
    /// (the rate bound zeroed at launch; the loop code path is unchanged, so an
    /// idle tick still revisits and re-parses, it simply never decides to think).
    /// `V3` zeroes `IGN_WEIGHT_BAKE` — a TEMPERAMENT draw, so it is a launch
    /// override, not a species sweep — leaving the sampler's other three weights
    /// at the individual's own draws. `V9` disables **sleep** the same way
    /// ([10 §7](engineering)'s two triggers, both refused at launch). Any other
    /// string (including `V0`) runs the registry's own values. No state invents a
    /// value; each is a constant pinned to a bound the registry already declares.
    pub fn set_launch_variant(&mut self, variant: &str) {
        match variant {
            "V2" => self.idle.per_window_override = Some(0),
            "V9" => {
                // 05 §2 line 83, both halves: the cadence pinned to the ceiling of
                // its own box, so no scenario's quiet run can reach it, **and** the
                // freeze trigger not taken. `due()` is the same function V0 calls;
                // it just never returns Yes here.
                self.sleep.cadence_override = Some(sleep_cadence_ceiling());
                self.sleep.freeze_armed = false;
            }
            "V3" => {
                self.idle.weights_override = Some(si_walk::IdleWeights {
                    recency_q12: self.temperament.get(DerivId::IgnRecency),
                    bake_q12: 0,
                    decay_deficit_q12: self.temperament.get(DerivId::IgnDecayDeficit),
                    contradiction_q12: self.temperament.get(DerivId::IgnContradiction),
                    // The fifth weight is a SPECIES row, not one of the four
                    // TEMPERAMENT draws, so V3 (which zeroes the bake draw) does
                    // not touch it.
                    silt_q12: self.arc.ign_weight_silt,
                });
            }
            // 12 §6.1: the traversal exists and the tugs do not — the derived
            // index is forced empty, so step 5b reads no row and round 0 is the
            // token lighting alone. This is the deletion control, and the state
            // hash equality it must produce is the whole of G17(b).
            "V12" => self.episodic_index = false,
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // The tick.
    // -----------------------------------------------------------------------

    /// 02 §11 steps 1–11 and 13, one sense line. `source` is an input carried
    /// from the panel or the MCP mouth, never inferred from the text.
    pub fn tick(&mut self, raw: &str, source: Source) -> Result<TickReport> {
        // 12 §2.1: a bare line carries no transport identity, so it is read as the
        // literal that names it — the owner's own id or grimm's registered one.
        self.world.line_identity = match source {
            Source::Owner => self.world.owner_identity,
            Source::Grimm => self.world.grimm_identity,
        };
        self.tick_line(raw, source)
    }

    fn tick_line(&mut self, raw: &str, source: Source) -> Result<TickReport> {
        let t = self.clock.now();
        // 02 §11 step 5a / 12 §30 AB.1: the active conversation's table erodes
        // before this tick's walk and before anything can deposit — law 27's
        // one-round lag. A tick that never reaches a walk (a teach, a refusal)
        // still decayed: silt erodes per logical tick (docs/23 §23.3).
        self.silt.tick(self.conversation_key(), t);
        // §19 S.1 / 04 §28 Z.1: a sense line is the only thing in this build that
        // can make the senses noisy, so it ends the quiet run trigger 1 counts. The
        // counter is RAM (01 §13's 13 mutable bytes do not include it) and a replay
        // re-derives it from the same recorded stream — which is what makes a
        // cadence-fired sleep a counter event and not a wall-clock one (**G13(a).1**).
        self.sleep.quiet_run = 0;
        // The route is this tick's observation or nothing: a step that never
        // reaches a read must not report the previous line's walk.
        self.route = None;
        self.tongue_tick = None;
        let stage = gate::stage_of(&self.store)?;
        let a_before = self.a.a();
        let thought_id = self.thought_id;
        self.transcript.push((t, source, raw.to_string()));
        let text_ref = self.store.interner_mut().intern(raw);
        self.log(t, 0, EV_SENSE_ECHO, format!("{{\"line\":{}}}", json_string(raw)));

        // 01 §6/§12, 09 §2.1: every line pays the sense-input floor — its own
        // `context` row — before anything is decided. A rejected teach is not
        // free, and a replayed session rebuilds its transcript from these rows.
        self.shelve_context(t, stage, source, text_ref)?;

        // Step 1: privilege. A `source: grimm` line reaches the gate as an inert
        // input: no teach, no correction, no statement (G12(d)).
        let teach = si_front::decide(raw, source);
        if let Some(decision) = &teach {
            self.log(
                t,
                source_byte(source),
                EV_DECODED,
                format!(
                    "{{\"kind\":\"teach\",\"outcome\":\"{}\",\"rows\":{},\"response\":{}}}",
                    decision.outcome.reason(),
                    decision.rows,
                    json_string(&decision.response)
                ),
            );
            return match decision.outcome {
                si_front::TeachOutcome::Row => {
                    self.teach_tick(t, stage, thought_id, text_ref, decision, source)
                }
                si_front::TeachOutcome::Inert => {
                    self.finish(t, thought_id, stage, a_before, Output::Inert {
                        reason: "grimm_source_inert".to_string(),
                    }, 0, 0)
                }
                other => {
                    let reason = other.reason();
                    let dark: Vec<String> = decision
                        .dark
                        .iter()
                        .map(|slot| json_string(&slot.key))
                        .collect();
                    self.log(
                        t,
                        source_byte(source),
                        EV_GAP,
                        format!("{{\"dark\":[{}],\"reason\":\"{reason}\"}}", dark.join(",")),
                    );
                    self.finish(t, thought_id, stage, a_before, Output::Refused {
                        reason: reason.to_string(),
                    }, 0, 0)
                }
            };
        }

        let decoded = si_front::decode(raw);
        let frame = decoded.frame();
        self.log(
            t,
            source_byte(source),
            EV_DECODED,
            format!(
                "{{\"frame\":\"{}\",\"pattern\":{},\"slots\":[{}]}}",
                frame.name(),
                frame.index(),
                decoded
                    .slots()
                    .iter()
                    .map(|slot| json_string(&slot.key))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        );

        // Step 2: the pending-ask match, owner-only (R-32). A teach never
        // reaches here (09 §2.1), and a correction never answers an ask.
        let pending = if source == Source::Owner
            && !matches!(frame, PatternId::PCorrect | PatternId::PCmdC)
        {
            gate::latest_open_ask(&self.store, t)?
        } else {
            None
        };
        // A bare reply is an undecoded line carrying at least one content word;
        // `Undecoded`'s `named_dark` is empty exactly when no shape anchored, so
        // the count must read the words, not the partial slots.
        let bare = decoded.words().iter().filter(|token| !is_function(&token.key)).count();
        if let Some(ask) = &pending {
            if matches!(decoded, si_front::Decoded::Undecoded { .. }) && bare > 0 {
                return self.answer_tick(t, stage, thought_id, text_ref, ask.clone(), &decoded, source);
            }
        }
        // Step 2's second arm — the nomination lane (OD-128). `latest_open_ask`
        // never returns an `origin = 2` row, so this is a separate read of the same
        // shelf, owner-only like the arm above, and it does not preempt: an
        // ordinary open ask still answers first. A `grimm:` line cannot reach it,
        // which is what `check_nomination_owner_lane` greps for.
        if source == Source::Owner
            && pending.is_none()
            && matches!(decoded, si_front::Decoded::Undecoded { .. })
            && bare > 0
        {
            if let Some(ask) =
                gate::latest_open_nomination(&self.store, t, gate::GAP_MOVE_NOMINATION)?
            {
                return self.skill_mint_tick(t, stage, thought_id, text_ref, ask, &decoded, source);
            }
        }

        match frame {
            PatternId::PCorrect | PatternId::PCmdC => {
                self.correction_tick(t, stage, thought_id, text_ref, &decoded, frame, source)
            }
            PatternId::PAttrib | PatternId::PDef | PatternId::POfRel => {
                self.statement_tick(t, stage, thought_id, text_ref, &decoded, source)
            }
            PatternId::PQuestA | PatternId::PQuestD | PatternId::PQuestW => {
                self.question_tick(t, stage, thought_id, &decoded, source)
            }
            _ => self.finish(t, thought_id, stage, a_before, Output::Inert {
                reason: "input_not_a_teach".to_string(),
            }, 0, 0),
        }
    }

    /// 09 §4's teach path: one row, keyed by the left side's decoded pattern.
    fn teach_tick(
        &mut self,
        t: i32,
        stage: u8,
        thought_id: u32,
        text_ref: u32,
        decision: &si_front::TeachDecision,
        source: Source,
    ) -> Result<TickReport> {
        let pattern = decision.pattern.clone().expect("a Row decision carries its pattern");
        let left = pattern.surface.clone();
        let right = decision.response.clone();
        let left_tokens = si_front::tokenize(&left);
        let right_tokens = si_front::tokenize(&right);
        let stimulus = left_tokens
            .iter()
            .map(|token| token.key.clone())
            .collect::<Vec<_>>()
            .join(" ");
        let response_fold = right_tokens
            .iter()
            .map(|token| token.key.clone())
            .collect::<Vec<_>>()
            .join(" ");

        // The entity is the left side's last slot; the value is the response's
        // last content token. Both are arrivals: the points are allocated here.
        let key_key = pattern.slots.last().map(|slot| slot.key.clone()).unwrap_or_default();
        let value_key = right_tokens
            .iter()
            .map(|token| token.key.clone())
            .last()
            .unwrap_or_default();
        let key_point = self.arrive(&key_key, pattern.slots.last().map(|slot| slot.surface.as_str()).unwrap_or(""));
        let val_point = self.arrive(&value_key, right_tokens.last().map(|token| token.surface.as_str()).unwrap_or(""));

        let stim_surface = self.store.interner_mut().intern(&left);
        let resp_surface = self.store.interner_mut().intern(&right);
        let stim_signature = gate::intern_signature(
            &mut self.store,
            &format!("{left}"),
            &format!("{right}"),
        );
        let shelf = if stage == 0 { SHELF_S_R_PAIRS } else { SHELF_FACTS };
        let key_field = Candidate::key_field_for(pattern.frame, 0);
        let terms: Vec<u32> = left_tokens
            .iter()
            .map(|token| self.arrive(&token.key, &token.surface))
            .collect();
        let candidate = Candidate {
            shelf,
            pattern_id: pattern.frame.index() as u16,
            key_field,
            key_point,
            val_point,
            stim_signature,
            stim_surface,
            resp_surface,
            target_slot: 0,
            prov_kind: gate::PROV_OWNER_TEACH,
            prov_speaker: gate::SPEAKER_OWNER,
            bind_output_id: 0,
            question_id: 0,
            text_ref,
            prov_step: t,
            marker: false,
            attempts_at_slot: 0,
            terms,
            derived_from: Vec::new(),
            stimulus: stimulus.clone(),
            response: response_fold.clone(),
        };
        self.commit_tick(t, stage, thought_id, candidate, source, &left, &right)
    }

    /// A bare owner reply to the latest open ask (§12 step 2). The row copies the
    /// ask's slot identity, so a later recall can cite the ask event.
    fn answer_tick(
        &mut self,
        t: i32,
        stage: u8,
        thought_id: u32,
        text_ref: u32,
        ask: gate::AskRow,
        decoded: &si_front::Decoded,
        source: Source,
    ) -> Result<TickReport> {
        let reply = decoded.surface().trim().to_string();
        let value_key = si_front::tokenize(&reply)
            .iter()
            .map(|token| token.key.clone())
            .last()
            .unwrap_or_default();
        let val_point = self.arrive(&value_key, &reply);
        let stim_surface = ask.record.body.stim_surface;
        let resp_surface = self.store.interner_mut().intern(&reply);
        let stim_signature = gate::intern_signature(&mut self.store, &reply, &reply);
        let pattern = PatternId::from_index(ask.pattern_id as u8).unwrap_or(PatternId::Undecoded);
        let shelf = if stage == 0 { SHELF_S_R_PAIRS } else { SHELF_FACTS };
        let candidate = Candidate {
            shelf,
            pattern_id: ask.pattern_id,
            key_field: Candidate::key_field_for(pattern, ask.target_slot),
            key_point: ask.key_point,
            val_point,
            stim_signature,
            stim_surface,
            resp_surface,
            target_slot: ask.target_slot,
            prov_kind: gate::PROV_ANSWER_TO_ASK,
            prov_speaker: gate::SPEAKER_OWNER,
            bind_output_id: ask.row_id,
            question_id: ask.row_id,
            text_ref,
            prov_step: t,
            marker: false,
            attempts_at_slot: 0,
            terms: Vec::new(),
            derived_from: Vec::new(),
            stimulus: reply.clone(),
            response: reply.clone(),
        };
        self.commit_tick(t, stage, thought_id, candidate, source, &reply, &reply)
    }

    /// `P_CMD_C` (`/correct …`) is T3; `P_CORRECT` (a bare statement carrying a
    /// marker) is T2 **with** the marker flag, so it arbitrates to the
    /// `marker-correction` branch and not to T3's
    /// outranks-everything row (04 §15.2, docs/06 §6.4). Both bind to the last
    /// output row: the corrected slot is that row's, the value is what the
    /// content names (`docs/04` Act 3's surface).
    fn correction_tick(
        &mut self,
        t: i32,
        stage: u8,
        thought_id: u32,
        text_ref: u32,
        decoded: &si_front::Decoded,
        frame: PatternId,
        source: Source,
    ) -> Result<TickReport> {
        let Some((shelf, row_id)) = self.last_output_row else {
            return self.finish(t, thought_id, stage, self.a.a(), Output::Refused {
                reason: "no_binding_target".to_string(),
            }, 0, 0);
        };
        let Some(bound) = self.store.row_at(shelf, row_id)? else {
            return self.finish(t, thought_id, stage, self.a.a(), Output::Refused {
                reason: "no_binding_target".to_string(),
            }, 0, 0);
        };
        let content = decoded.surface().trim().to_string();
        let content = match frame {
            PatternId::PCmdC => content
                .strip_prefix(si_front::decode::CMD_PREFIX)
                .map(str::trim)
                .unwrap_or(&content)
                .to_string(),
            _ => content.clone(),
        };
        if content.is_empty() {
            return self.finish(t, thought_id, stage, self.a.a(), Output::Refused {
                reason: "empty_correction".to_string(),
            }, 0, 0);
        }
        // The corrected slot is the row the command bound; the value is what the
        // content names, decoded as a statement when it has a shape.
        let value_slot = match si_front::decode(&content) {
            si_front::Decoded::Pattern(pattern) if pattern.slots.len() >= 2 => {
                pattern.slots.last().cloned()
            }
            _ => si_front::tokenize(&content).last().map(|token| si_front::Slot {
                word: 0,
                surface: token.surface.clone(),
                key: token.key.clone(),
            }),
        };
        let Some(value_slot) = value_slot else {
            return self.finish(t, thought_id, stage, self.a.a(), Output::Refused {
                reason: "empty_correction".to_string(),
            }, 0, 0);
        };
        let val_point = self.arrive(&value_slot.key, &value_slot.surface);
        // `/correct` keeps the owner's whole utterance as the response surface
        // (Act 3's sentence form); a marker statement's response is the value it
        // names (Act 4's `green`), because the marker is not part of the belief.
        let response_surface = match frame {
            PatternId::PCorrect => value_slot.surface.clone(),
            _ => content.clone(),
        };
        let resp_surface = self.store.interner_mut().intern(&response_surface);
        let stim_signature = gate::intern_signature(&mut self.store, &content, &content);
        // A correction bound to a question row answers it: the slot identity
        // comes from the ask and the belief lands on the knowledge shelf. A
        // correction bound to a belief keeps that row's shelf and key field.
        let bound_to_question = bound.envelope.shelf_x == SHELF_QUESTIONS;
        let shelf = if bound_to_question {
            if stage == 0 { SHELF_S_R_PAIRS } else { SHELF_FACTS }
        } else {
            bound.envelope.shelf_x
        };
        let pattern = PatternId::from_index(bound.body.pattern_id as u8)
            .unwrap_or(PatternId::Undecoded);
        let key_field = if bound_to_question {
            Candidate::key_field_for(pattern, bound.body.target_slot)
        } else {
            bound.body.key_field
        };
        let candidate = Candidate {
            shelf,
            pattern_id: bound.body.pattern_id,
            key_field,
            key_point: bound.body.key_point,
            val_point,
            stim_signature,
            stim_surface: bound.body.stim_surface,
            resp_surface,
            target_slot: bound.body.target_slot,
            prov_kind: if frame == PatternId::PCorrect { gate::PROV_STATEMENT } else { gate::PROV_CORRECTION },
            prov_speaker: gate::SPEAKER_OWNER,
            bind_output_id: row_id,
            question_id: if bound_to_question { row_id } else { 0 },
            text_ref,
            prov_step: t,
            marker: frame == PatternId::PCorrect,
            attempts_at_slot: 0,
            terms: Vec::new(),
            derived_from: Vec::new(),
            stimulus: content.clone(),
            response: content.clone(),
        };
        self.commit_tick(t, stage, thought_id, candidate, source, &content, &content)
    }

    /// A decoded statement is T2: it may conflict, and it may be the second
    /// telling that arbitrates as insistence.
    fn statement_tick(
        &mut self,
        t: i32,
        stage: u8,
        thought_id: u32,
        text_ref: u32,
        decoded: &si_front::Decoded,
        source: Source,
    ) -> Result<TickReport> {
        let si_front::Decoded::Pattern(pattern) = decoded else {
            return self.finish(t, thought_id, stage, self.a.a(), Output::Inert {
                reason: "input_not_a_teach".to_string(),
            }, 0, 0);
        };
        let Some(first) = pattern.slots.first() else {
            return self.finish(t, thought_id, stage, self.a.a(), Output::Inert {
                reason: "input_not_a_teach".to_string(),
            }, 0, 0);
        };
        let Some(last) = pattern.slots.last() else {
            return self.finish(t, thought_id, stage, self.a.a(), Output::Inert {
                reason: "input_not_a_teach".to_string(),
            }, 0, 0);
        };
        if first.key == last.key {
            return self.finish(t, thought_id, stage, self.a.a(), Output::Inert {
                reason: "input_not_a_teach".to_string(),
            }, 0, 0);
        }
        let key_point = self.arrive(&first.key, &first.surface);
        let val_point = self.arrive(&last.key, &last.surface);
        let text = pattern.surface.clone();
        let stim_surface = self.store.interner_mut().intern(&text);
        let resp_surface = self.store.interner_mut().intern(&last.surface);
        let stim_signature = gate::intern_signature(&mut self.store, &text, &text);
        let key_field = Candidate::key_field_for(pattern.frame, 0);
        let attempts = self.statement_attempts(pattern.frame.index() as u16, key_point, t)?;
        let shelf = if stage == 0 { SHELF_S_R_PAIRS } else { SHELF_FACTS };
        let candidate = Candidate {
            shelf,
            pattern_id: pattern.frame.index() as u16,
            key_field,
            key_point,
            val_point,
            stim_signature,
            stim_surface,
            resp_surface,
            target_slot: 0,
            prov_kind: gate::PROV_STATEMENT,
            prov_speaker: gate::SPEAKER_OWNER,
            bind_output_id: 0,
            question_id: 0,
            text_ref,
            prov_step: t,
            marker: false,
            attempts_at_slot: attempts,
            terms: Vec::new(),
            derived_from: Vec::new(),
            stimulus: text.clone(),
            response: last.key.clone(),
        };
        self.commit_tick(t, stage, thought_id, candidate, source, &text, &last.surface)
    }

    /// 04 §15.3: the repeat counter branch 2 reads is the pending event for
    /// this slot, not the row count. One open ask at the slot means this bare
    /// statement is the insistence that arbitrates.
    fn statement_attempts(&self, pattern_id: u16, key_point: u32, now: i32) -> Result<u32> {
        match gate::ask_at(&self.store, now, pattern_id, key_point, 0)? {
            Some(row) if row.status == gate::Q_OPEN => Ok(1),
            _ => Ok(0),
        }
    }

    /// The read path: light, walk, settle, label, render, and either an answer or
    /// the ask-back. No belief row is written (I9's read-only half).
    fn question_tick(
        &mut self,
        t: i32,
        stage: u8,
        thought_id: u32,
        decoded: &si_front::Decoded,
        source: Source,
    ) -> Result<TickReport> {
        let some_pattern = PatternId::from_index(decoded.frame().index()).unwrap_or(PatternId::Undecoded);
        let frame_point = self.store.points_mut().ensure_frame(some_pattern);
        let entity = decoded.slots().last().cloned();
        let keys = decoded.keys();
        let slot = match &entity {
            Some(slot) => {
                let point = self.arrive(&slot.key, &slot.surface);
                Some(SlotKey { frame: some_pattern, point })
            }
            None => None,
        };
        let retrieval = self.walk_read(t, thought_id, keys, Some(frame_point), slot.clone())?;
        let a_after = self.a.a();

        if let (Some(named), true) = (&retrieval.named, source == Source::Owner) {
            let words: Vec<String> = si_front::tokenize(decoded.surface())
                .into_iter()
                .map(|token| token.surface)
                .collect();
            if let Some(back) = ask_back(named, &words, source) {
                let (question_id, fresh) = self.ask(t, stage, some_pattern, named, decoded, source)?;
                self.log(t, source_byte(source), EV_ASK, format!(
                    "{{\"question\":{question_id},\"fresh\":{fresh},\"slot\":{}}}",
                    named.point
                ));
                self.log(t, source_byte(source), EV_OUTPUT, format!(
                    "{{\"label\":\"I-DON'T-KNOW\",\"tag\":\"NONE\",\"surface\":{}}}",
                    json_string(&back.surface)
                ));
                self.rendered.push((t, back.surface.clone()));
                let _ = stage;
                return self.finish(t, thought_id, stage, a_after, Output::Ask {
                    surface: back.surface,
                    question_id,
                    fresh,
                }, 0, 0);
            }
        }

        // A cited row is an answer: render it, label it, and serve the row the
        // render stands on (09 §6's one promotion).
        let rendered = si_front::reverse_decode(&retrieval.thought, &retrieval.cited);
        let caps = self.label_caps();
        let labeled = si_front::label(&si_front::LabelInput {
            thought: &retrieval.thought,
            caps: &caps,
            cited: &retrieval.cited,
            rendered: &rendered,
            question_shaped: true,
            named_dark: retrieval.named.is_some(),
            route: retrieval.route,
        });
        let label = labeled.surface().to_string();
        let tag = labeled.tag.surface().to_string();
        // 04 §31 AC.4 / 05 §23.3(f): recall cites the belief row **and** the
        // moment it came from, when this frame actually re-lit that origin. The
        // event id is a `context` row's `y` — its own per-shelf namespace — so it
        // is appended as its own id and never merged with a belief row id. The
        // de-duplication is therefore among the appended event ids only: a
        // `citation.contains` test here would drop the moment whenever its value
        // coincides with the belief row's, which is exactly the merge AC.4 bans.
        let mut citation = labeled.citation.clone();
        let mut from_events: Vec<u32> = Vec::new();
        for cited in &retrieval.cited {
            for relight in &retrieval.thought.relit {
                if (relight.point == cited.key_point || relight.point == cited.val_point)
                    && !from_events.contains(&relight.event_id)
                {
                    from_events.push(relight.event_id);
                }
            }
        }
        citation.extend(from_events.iter().copied());
        self.log(t, source_byte(source), EV_LABEL, format!(
            "{{\"label\":{},\"tag\":\"{tag}\",\"citation\":[{}]}}",
            json_string(&label),
            citation
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ));
        if !rendered.surface.is_empty() && label != "I-DON'T-KNOW" {
            // 13 §4.1's step 9a. `label`, `tag` and `citation` are bound above and
            // are not inputs to the call: check (a) is a type fact, not a promise
            // (02 §18). A `None` from the walker is the same string this file printed
            // before the tongue existed — the fallback, not a crash (**G20(g)**).
            let surface = self.realized_surface(t, thought_id, &retrieval, &rendered)?;
            self.log(t, source_byte(source), EV_OUTPUT, format!(
                "{{\"label\":{},\"tag\":\"{tag}\",\"surface\":{}}}",
                json_string(&label),
                json_string(&surface)
            ));
            self.rendered.push((t, surface.clone()));
            if let Some(&row_id) = rendered.citations.first() {
                self.serve_cited(t, row_id, thought_id)?;
            }
            return self.finish(t, thought_id, stage, a_after, Output::Answer {
                surface,
                label,
                tag,
                citation,
            }, 0, 0);
        }
        self.finish(t, thought_id, stage, a_after, Output::Inert {
            reason: "no_answer".to_string(),
        }, 0, 0)
    }

    // -----------------------------------------------------------------------
    // Step 9a — the tongue (13 §4.1). One call site, one insertion, and the
    // `else` value is the expression this file shipped before B20.
    // -----------------------------------------------------------------------

    /// The ruling row decides whether the arm is entered at all; off, the printed
    /// surface is `rendered.surface.clone()`, `tongue_tick` stays `None`, and no
    /// byte of the loop moves (**G20(j)**). On, a `None` from the walker is the
    /// same clone.
    #[cfg(feature = "tongue")]
    fn realized_surface(
        &mut self,
        t: i32,
        thought_id: u32,
        retrieval: &Retrieval,
        rendered: &si_front::Rendered,
    ) -> Result<String> {
        if !self.arc.tongue {
            return Ok(rendered.surface.clone());
        }
        let tick = self.tongue_line(t, thought_id, retrieval, rendered)?;
        let spoken = tick.realized.clone();
        self.tongue_tick = Some(tick);
        Ok(spoken.unwrap_or_else(|| rendered.surface.clone()))
    }

    #[cfg(not(feature = "tongue"))]
    fn realized_surface(
        &self,
        _t: i32,
        _thought_id: u32,
        _retrieval: &Retrieval,
        rendered: &si_front::Rendered,
    ) -> Result<String> {
        Ok(rendered.surface.clone())
    }

    /// The phrase walk, with its license built at this boundary (**OD-181**,
    /// 13 §1.2): the citation's own composed chunks and their endpoints' own terms,
    /// every one folded by the one tokenizer, because a second fold law in the leaf
    /// crate is what `G20(c)` exists to catch. The seed is the three snapshot values
    /// of **OD-183** — never the state hash, whose item 3 carries arousal.
    ///
    /// Returns the tick's record — anchor, license width, chunk cap, the citation
    /// the frame stands on, and the line, if any. A `None` line is the caller's
    /// fallback, not a crash.
    #[cfg(feature = "tongue")]
    fn tongue_line(
        &self,
        t: i32,
        thought_id: u32,
        retrieval: &Retrieval,
        rendered: &si_front::Rendered,
    ) -> Result<TongueTick> {
        // The answer row is the one `reverse_decode` printed: the same pool and the
        // same comparator, so the two can never disagree about which row speaks.
        let activation = |point: u32| {
            retrieval
                .thought
                .settle_set
                .iter()
                .find(|entry| entry.point == point)
                .map(|entry| entry.activation)
                .unwrap_or(0)
        };
        let record = |anchor: Option<u32>, licensed: usize, chunk_cap: usize, realized: Option<String>| TongueTick {
            thought: thought_id,
            anchor,
            licensed,
            chunk_cap,
            citation: rendered.citations.clone(),
            canonical: rendered.surface.clone(),
            realized,
        };
        let Some(answer) = retrieval
            .cited
            .iter()
            .filter(|row| !row.nameless)
            .max_by(|left, right| {
                activation(left.val_point)
                    .cmp(&activation(right.val_point))
                    .then(right.row_id.cmp(&left.row_id))
            })
        else {
            return Ok(record(None, 0, 0, None));
        };
        // **OD-188**: the anchor is the answer row's `key_point` — the point the line
        // is *about*, which is also docs/27 §27.4's second candidate, "the ask's
        // named target". Its first candidate, the `val_point`, is a lattice leaf: a
        // taught statement is one live edge from subject to predicate, the write gate
        // keeps one live predicate per subject, and no edge points back, so a walk
        // starting at the answer has no arc to take and every tick falls back. The
        // measurement is in `failure_report.md`; the walk's own check (c) is what
        // makes the re-seat honest — a line that never reaches the answer token is
        // refused, so an individual that cannot finish the thought says the canonical
        // frame instead of half of it.
        let anchor = answer.key_point;
        let license = self.license_for(&retrieval.cited, answer);
        // The tick's own web, read-only: the crate cannot build one, so the session
        // hands it over (02 §18).
        let graph = si_walk::Graph::build(&self.store, t)?;
        let style = si_core::tongue::derive_style(self.seed);
        let seed = si_core::tongue::render_seed(self.seed, style.digest(), thought_id);
        let chunk_cap = 1 + si_tongue::budget(self.arc.phrase_max_steps, &style);
        let mut draws = Rng::seeded(seed);
        let Some(line) = si_tongue::realize(
            &graph,
            anchor,
            &license,
            &rendered.content,
            &style,
            self.arc.phrase_max_steps,
            &mut draws,
        ) else {
            return Ok(record(Some(anchor), license.len(), chunk_cap, None));
        };
        // Checks (b) and (c), at the boundary, over the same fold.
        let keys: Vec<String> = si_front::tokenize(&line)
            .into_iter()
            .map(|token| token.key)
            .collect();
        Ok(record(
            Some(anchor),
            license.len(),
            chunk_cap,
            si_tongue::realizes_the_slots(&keys, &rendered.content, &license).then_some(line),
        ))
    }

    /// The mouth, as data: every surface the walk may utter, and nothing else. The
    /// answer row's composed chunk is pushed first so a point that arrives twice
    /// keeps the canonical copy, which is citation order; the two terms of each row
    /// follow, because a line is allowed to name the thing it is about (**OD-188**)
    /// only if the boundary handed that name over.
    ///
    /// **OD-189**'s variant bank rides each entry: every *other* verbatim surface
    /// the session has interned whose folded token sequence is this entry's. Same
    /// words by the one fold law, different bytes — a different spelling of the
    /// same claim, not a paraphrase, so no new row, no new citation and no second
    /// chunk. The bank is grouped once per call by the tokenizer's own fold; the
    /// selection among a group happens inside `si-tongue`, and this boundary only
    /// says what was taught.
    #[cfg(feature = "tongue")]
    fn license_for(&self, cited: &[si_read::retrieve::Cited], answer: &si_read::retrieve::Cited) -> Vec<si_tongue::License> {
        let fold = |surface: &str| -> Vec<String> {
            si_front::tokenize(surface).into_iter().map(|token| token.key).collect()
        };
        let mut bank: std::collections::BTreeMap<Vec<String>, Vec<si_tongue::Variant>> =
            std::collections::BTreeMap::new();
        let mut seen = std::collections::BTreeSet::new();
        for entry in self.store.interner().surfaces() {
            if entry.verbatim.is_empty() || !seen.insert(entry.verbatim.clone()) {
                continue;
            }
            let keys = fold(&entry.verbatim);
            bank.entry(keys.clone())
                .or_default()
                .push(si_tongue::Variant::new(entry.verbatim.clone(), keys));
        }
        let variants = |keys: &[String]| bank.get(keys).cloned().unwrap_or_default();
        let entry = |point: u32, surface: &str| -> si_tongue::License {
            let keys = fold(surface);
            let banked = variants(&keys);
            si_tongue::License::new(point, surface.to_string(), keys).with_variants(banked)
        };
        let mut license = vec![entry(answer.val_point, &answer.surface)];
        for row in cited.iter().filter(|row| !row.nameless) {
            license.push(entry(row.val_point, &row.surface));
            for point in [row.key_point, row.val_point] {
                let term = self
                    .store
                    .points()
                    .surface(point)
                    .or_else(|| self.store.points().canonical(point));
                if let Some(term) = term {
                    license.push(entry(point, term));
                }
            }
        }
        license
    }

    // -----------------------------------------------------------------------
    // Shared halves.
    // -----------------------------------------------------------------------

    fn commit_tick(
        &mut self,
        t: i32,
        stage: u8,
        thought_id: u32,
        candidate: Candidate,
        source: Source,
        stimulus: &str,
        response: &str,
    ) -> Result<TickReport> {
        let key_point = candidate.key_point;
        let val_point = candidate.val_point;
        let retrieval = self.walk_read(
            t,
            thought_id,
            si_front::keys(stimulus),
            None,
            None,
        )?;
        let _ = (&retrieval, key_point, val_point);
        let a_formation = self.a.a();

        let ctx = WriteContext {
            t,
            prov_session: self.prov_session,
            source,
            class: self.class_of(self.world.line_identity, t),
            stage,
            a_formation,
            thought_id,
            signal_ref: t as u32,
        };
        let report = gate::write(&mut self.store, &candidate, &ctx, &self.params, &mut self.stats)?;
        self.absorb(t, thought_id, &report, candidate.key_point, response);

        // The act's own signals are integrated after the commit, so the salience
        // read above saw the pre-spike `a` (OD-56's cold-first-row litmus).
        if !report.signals.is_empty() {
            let round = self
                .settle_round()
                .unwrap_or(1);
            let lagged = drive::update(
                &mut self.a,
                &report.signals,
                4096u16.saturating_add(round),
                thought_id,
                &self.params,
                &mut self.trajectory,
            );
            self.door_call(
                t,
                thought_id,
                "update",
                &format!("{{\"signals\":{},\"slot\":{}}}", report.signals.len(), 4096u16.saturating_add(round)),
            );
            // 08 §2.1 row 5's `rows`: the six signals **as numbers**, the whole of
            // what `Signal` is allowed to carry (01 §5.3: kind and its own value,
            // never a source and never a referent). So `citation` is `null` here
            // by law, not by omission — **OD-146** — and the frame's own
            // `(t, thought)` pair is what the panel prints in that column, since
            // the commit frame at the same thought is the act these signals came
            // from. No name appears: §18.1's labels live in `si-audit`.
            let rows: Vec<String> = report
                .signals
                .iter()
                .map(|signal| {
                    format!(
                        "{{\"signal\":{},\"magnitude_q12\":{},\"citation\":null}}",
                        signal.kind.index(),
                        signal.magnitude
                    )
                })
                .collect();
            self.log(t, source_byte(source), EV_SIGNAL, format!(
                "{{\"signals\":{},\"a_after\":{},\"applies_at\":{},\"rows\":[{}]}}",
                report.signals.len(),
                lagged.a,
                lagged.applies_at,
                rows.join(",")
            ));
        }
        let a_after = self.a.a();
        let a_before = a_formation;
        let (output, rows) = match &report.outcome {
            Outcome::Committed { row_id, shelf, branch, .. } => {
                self.last_output_row = Some((*shelf, *row_id));
                self.log(t, source_byte(source), EV_COMMIT, format!(
                    "{{\"row\":{row_id},\"shelf\":{shelf},\"branch\":\"{}\",\"t\":{t}}}",
                    branch.as_str()
                ));
                (Output::Committed { row_id: *row_id, shelf: *shelf, branch: branch.as_str() }, 1)
            }
            Outcome::Confirmed { row_id, shelf } => {
                self.last_output_row = Some((*shelf, *row_id));
                (Output::Inert { reason: "boring".to_string() }, 0)
            }
            Outcome::Asked { question_id } => {
                (Output::Ask { surface: String::new(), question_id: *question_id, fresh: true }, 0)
            }
            Outcome::Rejected { reason } => (Output::Refused { reason: reason.to_string() }, 0),
            Outcome::Inert { reason } => (Output::Inert { reason: reason.to_string() }, 0),
            // 04 §33 AE.2: the class arm's answer. It is a distinct output from
            // `Inert`'s tag refusal — the session prints the derived class with it
            // so a report can tell the two arms apart.
            Outcome::Refused { class, reason } => (Output::Refused {
                reason: format!("{reason}:{class}"),
            }, 0),
        };
        let _ = (response, key_point);
        self.finish(t, thought_id, stage, a_after, output, rows, report.salience_calls)
            .map(|mut report_tick| {
                // `a_before` is the pre-commit value the salience call read.
                report_tick.a_before = a_before;
                report_tick
            })
    }

    /// 04 §31 AC.1 / 12 §4.1 step 5b: this frame's derived origin links. The
    /// index is rebuilt at frame load and never stored, so
    /// `EPISODIC_TRAVERSAL = off` — the shipped default — returns the empty
    /// vector and step 6's arithmetic is bit-identical to B13's.
    fn episodic_tugs(&self, t: i32) -> Result<Vec<si_core::episodic::OriginTug>> {
        if !self.arc.episodic_traversal || !self.episodic_index {
            return Ok(Vec::new());
        }
        si_read::event_view_index(&self.store, t, self.arc.prov_link_tug)
    }

    /// The walk half shared by every tick: the one `si-read::retrieve` call, the
    /// settle bookkeeping, and the mind-log frames the read path owns.
    fn walk_read(
        &mut self,
        t: i32,
        thought_id: u32,
        keys: Vec<String>,
        frame_point: Option<u32>,
        slot: Option<SlotKey>,
    ) -> Result<Retrieval> {
        // Local copies so the walk's bonds do not overlap `&mut self.a`.
        let params = self.params;
        let curve = self.curve.clone();
        let tugs = self.episodic_tugs(t)?;
        // 04 §32 AD.5 / OD-118: the installed skills' root handles, as nothing
        // more than an iteration order. `SKILL_COMPOSITION = off` — the shipped
        // default — never reads shelf 9 here, and an empty hint leaves the
        // walk's arithmetic bit-identical (G18(b)'s second half).
        let hint = if self.arc.skill_composition {
            si_skills::composition_preference(&self.store, t)?
        } else {
            Vec::new()
        };
        let config = WalkConfig {
            params: &params,
            curve: &curve,
            k_max: self.walk.k_max,
            tau_settle: self.walk.tau_settle,
            decay_base: self.walk.decay_base,
            hub_exponent_q12: self.walk.hub_exponent_q12,
            frontier_cap: self.walk.frontier_cap,
            oscillation: self.walk.oscillation,
            ghost_max: self.arc.ghost_max,
            shape_max: self.arc.shape_max,
            tugs: &tugs,
            relight_max: self.arc.relight_max,
            composition_hint: &hint,
        };
        let store = &self.store;
        let resolver =
            PointsResolver { points: store.points(), frozen: store.frozen().map(|l| &**l) };
        let retrieval = retrieve(
            store,
            store.interner(),
            &resolver,
            &mut self.a,
            &config,
            &ReadRequest { keys, frame_point, slot, anchor: t, thought_id },
        )?;
        self.route = Some(retrieval.route);
        self.record_thought(t, thought_id, &retrieval.thought);
        // OD-120: `times_consulted` is a *derived* counter, so the log has to
        // carry the event it counts. One line per walk whose hint was non-empty —
        // session RAM, hashed nowhere, and the only writer of the number §2.6's
        // usage block prints.
        if !hint.is_empty() {
            self.skill_lines.push(serde_json::json!({
                "t": t,
                "thought": thought_id,
                "transition": "consult",
                "roots": hint,
            }));
        }
        // 02 §11 step 7a (engineering/12 §4.1): observation is after settle and
        // before commit — a detector that ran earlier would nominate on a
        // half-formed thought, one that ran later could read its own minted row.
        self.observe_frame(t, thought_id, &retrieval.thought)?;
        Ok(retrieval)
    }

    /// Step 7a: export the frame log (12 §2.6) and, when `CRYSTALLIZE` is on,
    /// ask the detector. Its only output is a `Nomination` — the ask below is
    /// this session's ordinary `questions` write through the ordinary queue,
    /// never a row the detector could reach (12 §1.2 win 5).
    fn observe_frame(&mut self, t: i32, thought_id: u32, thought: &Thought) -> Result<()> {
        // 02 §11 step 7's tail / 12 §30 AB.1: every point this frame lit that is
        // live-layer knowledge deposits `SILT_BUMP` before the detector runs.
        // Frozen rows never deposit (docs/23 §23.3); under `WORLD_TIERS =
        // owner-only` the frozen layer is empty, so the filter selects live L3.
        self.deposit_frame(t, &thought.frame_log)?;
        let mut logged = frame_log_json(thought_id, &thought.frame_log);
        // `t` is the frame's tick: G15(c) feeds the trace back at the ticks it
        // was recorded at, so the recurrence window is the run's, not a replay's.
        logged["t"] = serde_json::json!(t);
        // 12 §4.1's two observations are **independent arms**, not one guarded
        // block: `SKILLS` is gated at `CRYSTALLIZE = off` (the shipped default),
        // and a skills run whose detector never spoke would grade nothing.
        if self.arc.crystallize {
            let stage = gate::stage_of(&self.store)?;
            let observation = self.crystallizer.observe(&thought.frame_log, &self.store, t)?;
            for event in &observation.events {
                self.crystallize_lines.push(serde_json::json!({
                    "t": event.t,
                    "thought": thought_id,
                    "transition": transition_label(event.transition),
                    "members": event.members,
                    "recurrence": event.recurrence,
                    "candidates": observation.candidates,
                }));
            }
            for nomination in &observation.nominations {
                self.nomination_ask(t, thought_id, stage, nomination)?;
            }
        }
        // G18(c)'s trace is **eventlog-fixed as well as frame-fixed**: the shape
        // detector reads the log as it stands at step 7, and rows this tick
        // appends after step 7 (an answer's resolve commit, say) are outside what
        // it could see. A replay that handed the detector the finished log would
        // read a different outcome tag and differ by a byte — of `a`, of nothing
        // else. So the trace carries the cursor the detector actually read, and
        // `skills_trace_table` slices to it.
        let events_seen = if self.arc.skills {
            let stage = gate::stage_of(&self.store)?;
            self.observe_shapes(t, thought_id, stage, thought)?
        } else {
            self.events.events().len()
        };
        logged["events_seen"] = serde_json::json!(events_seen);
        self.frame_logs.push(logged);
        Ok(())
    }

    /// Step 7b: the shape detector, over the same frame log §29 read. Its output
    /// is a `Nomination` value and the forwarding **count** — the exclusion is
    /// this session's decision about its own two tables, never a message between
    /// the crates (AD.4, and the two have no edge to each other).
    fn observe_shapes(
        &mut self,
        t: i32,
        thought_id: u32,
        stage: u8,
        thought: &Thought,
    ) -> Result<usize> {
        let events = self.events.events().to_vec();
        let events_seen = events.len();
        let observation = self.skiller.observe(&thought.frame_log, &events, t)?;
        for event in &observation.events {
            self.skill_lines.push(serde_json::json!({
                "t": event.t,
                "thought": thought_id,
                "transition": shape_transition_label(event.transition),
                "template": format!("{:08x}", event.template),
                "recurrence": event.recurrence,
                "filler_sets_seen": event.filler_sets_seen,
                "candidates": observation.candidates,
                "shapes": observation.shapes,
                "forwarded": observation.forwarded,
            }));
        }
        for nomination in &observation.nominations {
            self.skill_nomination_ask(t, thought_id, stage, nomination)?;
        }
        Ok(events_seen)
    }

    /// §22.4 / 12 §2.4: an ordinary ask on the ordinary queue. `origin = 2`
    /// decides the lane, `arousal_at_ask = 255` is the row's own I8 assertion,
    /// and `text_ref` is an already-interned member surface so the hashed
    /// interner cannot grow from a detector's output (05 §23).
    fn nomination_ask(
        &mut self,
        t: i32,
        thought_id: u32,
        stage: u8,
        nomination: &si_crystallize::Nomination,
    ) -> Result<()> {
        let anchor = nomination
            .members
            .first()
            .copied()
            .ok_or_else(|| anyhow::anyhow!("a nomination names no member"))?;
        let budget = drive::budget(&self.params);
        let emitted = drive::emit(
            &[drive::Ranked {
                gap: drive::Gap {
                    slot: anchor,
                    salience_q12: budget.salience_floor,
                    last_pulsed_at: t,
                    usage_q12: Q12_ONE,
                },
                weight_q12: budget.salience_floor,
            }],
            &self.params,
        );
        self.ask_doors(t, thought_id, anchor, emitted.len());
        if emitted.is_empty() {
            return Ok(());
        }
        if let Some(existing) =
            gate::ask_at(&self.store, t, si_core::record::PATTERN_NAMELESS, anchor, 0)?
        {
            if existing.status == gate::Q_OPEN {
                return Ok(());
            }
        }
        let ctx = WriteContext {
            t,
            prov_session: self.prov_session,
            source: Source::Owner,
            class: si_core::world::SpeakerClass::Owner,
            stage,
            a_formation: self.a.a(),
            thought_id,
            signal_ref: t as u32,
        };
        let question_id = gate::append_question(
            &mut self.store,
            &QuestionSpec {
                pattern_id: si_core::record::PATTERN_NAMELESS,
                gap_kind: si_crystallize::GAP_CLUSTER_NOMINATION,
                origin: si_crystallize::ASK_ORIGIN_NOMINATION,
                key_point: anchor,
                target_slot: 0,
                text_ref: nomination.text_ref,
                prov_step: t,
                arousal_at_ask: si_core::record::AROUSAL_NONE,
            },
            &ctx,
        )?;
        self.stats.asks += 1;
        self.last_output_row = Some((SHELF_QUESTIONS, question_id));
        let members: Vec<String> = nomination
            .members
            .iter()
            .map(|point| point.to_string())
            .collect();
        self.crystallize_lines.push(serde_json::json!({
            "t": t,
            "thought": thought_id,
            "transition": "ask",
            "members": nomination.members,
            "recurrence": nomination.recurrence,
            "question": question_id,
            "kind": si_crystallize::GAP_CLUSTER_NOMINATION,
            "origin": si_crystallize::ASK_ORIGIN_NOMINATION,
        }));
        let detail = format!(
            "{{\"kind\":\"cluster-nomination\",\"question\":{question_id},\"origin\":2,\"members\":[{}]}}",
            members.join(",")
        );
        self.log(t, 0, EV_ASK, detail);
        Ok(())
    }

    /// 04 §32 AD.4's ask: a `move-nomination` gap on the **same** queue, through
    /// the same `drive::budget`/`drive::emit` doors, so a shape cannot buy a second
    /// ask (`ASK_CAP_PER_INPUT` and `DEDUPE_WINDOW` are the whole of OD-128's
    /// "a busier world may not buy more asks"). It carries **no** `text_ref`:
    /// 01 §11's absent id, because a shape has no word to name — the ask enumerates
    /// its root handle, and the hashed interner cannot grow from a detector
    /// (05 §23's byte-identical-interner half of G18(a)).
    fn skill_nomination_ask(
        &mut self,
        t: i32,
        thought_id: u32,
        stage: u8,
        nomination: &si_skills::Nomination,
    ) -> Result<()> {
        let anchor = nomination.root_point;
        let budget = drive::budget(&self.params);
        let emitted = drive::emit(
            &[drive::Ranked {
                gap: drive::Gap {
                    slot: anchor,
                    salience_q12: budget.salience_floor,
                    last_pulsed_at: t,
                    usage_q12: Q12_ONE,
                },
                weight_q12: budget.salience_floor,
            }],
            &self.params,
        );
        self.ask_doors(t, thought_id, anchor, emitted.len());
        if emitted.is_empty() {
            return Ok(());
        }
        if let Some(existing) =
            gate::ask_at(&self.store, t, si_core::record::PATTERN_NAMELESS, anchor, 0)?
        {
            if existing.status == gate::Q_OPEN {
                return Ok(());
            }
        }
        let ctx = WriteContext {
            t,
            prov_session: self.prov_session,
            source: Source::Owner,
            class: si_core::world::SpeakerClass::Owner,
            stage,
            a_formation: self.a.a(),
            thought_id,
            signal_ref: t as u32,
        };
        let question_id = gate::append_question(
            &mut self.store,
            &QuestionSpec {
                pattern_id: si_core::record::PATTERN_NAMELESS,
                gap_kind: si_skills::GAP_MOVE_NOMINATION,
                origin: si_skills::ASK_ORIGIN_NOMINATION,
                key_point: anchor,
                target_slot: 0,
                text_ref: 0,
                prov_step: t,
                arousal_at_ask: si_core::record::AROUSAL_NONE,
            },
            &ctx,
        )?;
        self.stats.asks += 1;
        self.last_output_row = Some((SHELF_QUESTIONS, question_id));
        self.skill_lines.push(serde_json::json!({
            "t": t,
            "thought": thought_id,
            "transition": "ask",
            "template": format!("{:08x}", nomination.template),
            "root_point": nomination.root_point,
            "recurrence": nomination.recurrence,
            "filler_sets_seen": nomination.filler_sets_seen,
            "outcome_tags": nomination.outcome_tags,
            "question": question_id,
            "kind": si_skills::GAP_MOVE_NOMINATION,
            "origin": si_skills::ASK_ORIGIN_NOMINATION,
        }));
        let detail = format!(
            "{{\"kind\":\"move-nomination\",\"question\":{question_id},\"origin\":2,\"root\":{anchor}}}"
        );
        self.log(t, 0, EV_ASK, detail);
        Ok(())
    }

    /// The shape an open `move-nomination` ask names, as the detector's own table
    /// holds it: `key_point` is the root handle, so the lookup is by handle and
    /// needs no word. Ascending template order makes it a total function.
    fn shape_of(&self, root_point: u32) -> Option<(u32, String)> {
        self.skiller
            .table()
            .values()
            .find(|candidate| candidate.root_point == root_point)
            .map(|candidate| (candidate.template, candidate.template_text.clone()))
    }

    /// 04 §32 AD.6: the owner's answer to a `move-nomination` ask mints a `skills`
    /// row through **step 11's ordinary commit** — the same ten-step pipeline, the
    /// same `prov_kind = 3 answer-to-ask`, `gain_kind` from §8.2's own mapping, and
    /// `arousal_at_ask = 255` because the gate's commit already writes it.
    ///
    /// The naming rule adds no vocabulary (AD.2's "zero new vocabulary"): a reply
    /// carrying **exactly one** content word *is* the name, and `resp_surface` is
    /// that owner's word. Any other reply leaves the row placeholder-named —
    /// `pattern_id = NAMELESS`, `resp_surface = skill:{hash}` — which is the state
    /// [04 §21 U.1](../../../engineering/04-algorithms.md) makes addressable and
    /// unspeakable, not a failed naming.
    fn skill_mint_tick(
        &mut self,
        t: i32,
        stage: u8,
        thought_id: u32,
        text_ref: u32,
        ask: gate::AskRow,
        decoded: &si_front::Decoded,
        source: Source,
    ) -> Result<TickReport> {
        let reply = decoded.surface().trim().to_string();
        let Some((template, template_text)) = self.shape_of(ask.key_point) else {
            // The ask is open but its shape is gone from RAM — a resumed session
            // rebuilds empty tables (12 §2.6). A name with no shape behind it is
            // not a skill row, so this mints nothing and says so.
            self.skill_lines.push(serde_json::json!({
                "t": t,
                "thought": thought_id,
                "transition": "mint-refused",
                "reason": "shape_unknown",
                "root_point": ask.key_point,
            }));
            return self.finish(t, thought_id, stage, self.a.a(), Output::Inert {
                reason: "nomination_shape_unknown".to_string(),
            }, 0, 0);
        };
        let names: Vec<String> = si_front::tokenize(&reply)
            .into_iter()
            .map(|token| token.key)
            .filter(|key| !is_function(key))
            .collect();
        let (pattern_id, name) = if names.len() == 1 {
            (0, names[0].clone())
        } else {
            (si_core::record::PATTERN_NAMELESS, format!("skill:{template:08x}"))
        };
        let stim_signature = self.store.interner_mut().intern(&template_text);
        let resp_surface = self.store.interner_mut().intern(&name);
        let candidate = Candidate {
            shelf: SHELF_SKILLS,
            pattern_id,
            key_field: si_core::record::SKILL_KIND_SHAPE,
            key_point: ask.key_point,
            val_point: 0,
            stim_signature,
            stim_surface: stim_signature,
            resp_surface,
            target_slot: 0,
            prov_kind: gate::PROV_ANSWER_TO_ASK,
            prov_speaker: gate::SPEAKER_OWNER,
            bind_output_id: ask.row_id,
            question_id: ask.row_id,
            text_ref,
            prov_step: t,
            marker: false,
            attempts_at_slot: 0,
            terms: Vec::new(),
            derived_from: Vec::new(),
            stimulus: template_text.clone(),
            response: name.clone(),
        };
        let report =
            self.commit_tick(t, stage, thought_id, candidate, source, &template_text, &name)?;
        if let Output::Committed { row_id, shelf, branch } = report.output {
            // docs/25 §25.4 rule 4: a shape that has a row never opens a
            // candidate again. The mint is the session's, so the note is too.
            self.skiller.note_minted(template);
            self.skill_lines.push(serde_json::json!({
                "t": t,
                "thought": thought_id,
                "transition": "mint",
                "template": format!("{template:08x}"),
                "root_point": ask.key_point,
                "row": row_id,
                "shelf": shelf,
                "branch": branch,
                "named": pattern_id != si_core::record::PATTERN_NAMELESS,
                "ask": ask.row_id,
            }));
            // A **named** mint carries `pattern_id = 0`, so step 11's own
            // ask-matching resolve (keyed by the answering row's pattern) cannot
            // find the NAMELESS ask it answered. The lane resolves it here, by the
            // same version-row supersede the revisit path uses; an unnamed mint
            // was already resolved inside the commit and this finds nothing open.
            self.resolve_nomination_ask(t, thought_id, row_id, ask.key_point)?;
        }
        Ok(report)
    }

    /// The lane's own resolution, for the mint the commit could not match.
    fn resolve_nomination_ask(
        &mut self,
        t: i32,
        thought_id: u32,
        resolved_by: u32,
        key_point: u32,
    ) -> Result<()> {
        let Some(ask) =
            gate::latest_open_nomination(&self.store, t, gate::GAP_MOVE_NOMINATION)?
        else {
            return Ok(());
        };
        if ask.key_point != key_point {
            return Ok(());
        }
        let ctx = WriteContext {
            t,
            prov_session: self.prov_session,
            source: Source::Owner,
            class: si_core::world::SpeakerClass::Owner,
            stage: gate::stage_of(&self.store)?,
            a_formation: self.a.a(),
            thought_id,
            signal_ref: t as u32,
        };
        let version = gate::resolve_ask(&mut self.store, &ask, resolved_by, &ctx, t)?;
        self.log(
            t,
            0,
            EV_SUPERSEDE,
            format!(
                "{{\"event\":\"resolve\",\"question\":{},\"version_row\":{version},\"resolved_by\":{resolved_by},\"by\":\"owner-lane\"}}",
                ask.row_id
            ),
        );
        Ok(())
    }

    fn record_thought(&mut self, t: i32, thought_id: u32, thought: &Thought) {
        let lit: Vec<String> = thought
            .lit_round0
            .iter()
            .map(|entry| format!("{{\"point\":{},\"q12\":{}}}", entry.point, entry.activation))
            .collect();
        self.log(t, 0, EV_LIT_ROUND0, format!("{{\"thought\":{thought_id},\"lit\":[{}]}}", lit.join(",")));
        for round in &thought.rounds {
            self.log(t, 0, EV_A_UPDATE, format!(
                "{{\"thought\":{thought_id},\"round\":{},\"a_used\":{},\"a_after\":{},\"applies_at\":{}}}",
                round.round,
                round.a_used,
                round.a_after,
                round.round.saturating_add(self.params.lag_rounds)
            ));
            // 08 §2.1 row 3: the round as the PREVIEW walks it. Every field is a
            // `RoundTrace` field, so the panel draws the walk's own numbers
            // (§1's third law) — `new_edges` carries the traversed route with the
            // weight it actually paid and the `genome` bit that says OWNED or BASE
            // (§5.2's two-colour rule reads that bit, never a name).
            let lit: Vec<String> = round
                .activations
                .iter()
                .map(|entry| format!("{{\"point\":{},\"h_q12\":{}}}", entry.point, entry.activation))
                .collect();
            let edges: Vec<String> = round
                .contributions
                .iter()
                .map(|c| {
                    format!(
                        "{{\"from\":{},\"to\":{},\"weight_eff_q12\":{},\"amount_q12\":{},\"base\":{}}}",
                        c.source, c.to_point, c.weight_eff_q12, c.amount, c.genome
                    )
                })
                .collect();
            self.log(
                t,
                0,
                EV_ROUND,
                format!(
                    "{{\"thought\":{thought_id},\"k\":{},\"lit_set\":[{}],\"new_edges\":[{}],\
                     \"a_used_u8\":{},\"modulation\":{{\"reach\":{},\"decay_q12\":{},\"tau_shift_q12\":{}}}}}",
                    round.round,
                    lit.join(","),
                    edges.join(","),
                    round.a_used,
                    round.reach,
                    round.decay_q12,
                    round.tau_q12
                ),
            );
            // 08 §2.1 row 7 / §4 E5: the medians this round conducted with. The
            // only witness the walk keeps is `EdgeBias::median_used`, so the frame
            // reports it per touched point — and `n_stamps` is `null`, not `0`,
            // because **OD-144**: 04 §9's H has no producer anywhere in `src/`, so
            // no point has a stamp count and the γ term is structurally absent.
            // §9.4's own rule is that an undefined median contributes *nothing*,
            // so an absent `n_stamps` is the record and a printed `0` would be a
            // measurement the build never made.
            let mut to_point: BTreeMap<u32, u32> = BTreeMap::new();
            for c in &round.contributions {
                to_point.entry(c.edge_off).or_insert(c.to_point);
            }
            let rows: Vec<String> = round
                .biases
                .iter()
                .map(|bias| {
                    let point = to_point.get(&bias.edge_off).copied().unwrap_or(u32::MAX);
                    format!(
                        "{{\"point\":{},\"median_q12\":{},\"n_stamps\":null}}",
                        point, bias.median_used
                    )
                })
                .collect();
            self.log(
                t,
                0,
                EV_MEDIAN,
                format!(
                    "{{\"thought\":{thought_id},\"round\":{},\"rows\":[{}],\"producer\":\
                     \"absent (OD-144: 04 §9's H is unbuilt, so GAMMA is inert)\"}}",
                    round.round,
                    rows.join(",")
                ),
            );
            // 08 §2.1 row 9 / §6 O4's cheap tripwire: the walk's two doors, one
            // frame each. **OD-145**: §2.1 says "exactly two calls per thought"
            // while [02 §11] says the walk makes the two *calls* — `update` and
            // `conduction` — and it makes one of each per **round**. This build
            // records every call and lets the panel count both ways, because a
            // frame that pre-agreed with one of the two sentences would hide the
            // disagreement on the one screen the owner reads it from.
            for (door, args) in [
                (
                    "update",
                    format!("{{\"round\":{},\"a_used\":{}}}", round.round, round.a_used),
                ),
                (
                    "conduction",
                    format!("{{\"round\":{},\"biases\":{}}}", round.round, round.biases.len()),
                ),
            ] {
                self.door_call(t, thought_id, door, &args);
            }
        }
        let settle: Vec<String> = thought
            .settle_set
            .iter()
            .map(|entry| format!("{{\"point\":{},\"q12\":{}}}", entry.point, entry.activation))
            .collect();
        self.log(t, 0, EV_SETTLE, format!(
            "{{\"thought\":{thought_id},\"stop\":\"{:?}\",\"settle\":[{}]}}",
            thought.stop_reason,
            settle.join(",")
        ));
        self.settle = thought
            .settle_set
            .iter()
            .map(|entry| (entry.point, entry.activation.clamp(0, u16::MAX as i32) as u16))
            .collect();
        let points: Vec<u32> = thought.settle_set.iter().map(|entry| entry.point).collect();
        self.store.observe_settle(&points);
        // 04 §19 S.3: ignition's recency term reads `last_lit(point)`. Every
        // point this thought reached — sensed or idle — is stamped with the tick,
        // so the sampler sees which hyperpoints the individual has been on. This
        // map is derived session state, never a hash input and never a shelf row.
        for point in thought
            .lit_round0
            .iter()
            .map(|entry| entry.point)
            .chain(thought.settle_set.iter().map(|entry| entry.point))
        {
            self.idle.last_lit.insert(point, t);
        }
        // 04 §28 Z.2 P1: the sleep replays a sense tick as the round-0 vector that
        // tick actually lit. `si-sleep` may not import `si-front` or `si-read`
        // (02 §13), so the lighting is kept here, in RAM, against the `context` row
        // the line paid for. A row with no entry is a replay event the pass counts
        // and does not walk — `events_available` beside `walks`, **G13(l).4** —
        // which is the honest shape of "less than it was given", not a bug.
        if let Some((row_id, written_at_t)) = self.sleep.context_row.take() {
            self.sleep.replay.push(si_sleep::ReplayEvent {
                row_id,
                written_at_t,
                lights: thought.lit_round0.clone(),
            });
        }
        // 04 §31 AC.2–AC.3 / 05 §23.3(d): the frame's origin re-lights, one line
        // each. The log is the gate's evidence that N happened; no stamp, bake,
        // confidence, edge or question is written here or anywhere on this path.
        for relight in &thought.relit {
            self.relight_lines.push(serde_json::json!({
                "t": t,
                "thought": thought_id,
                "event_id": relight.event_id,
                "point": relight.point,
                "q12": relight.activation,
                "open_24_restamp": "off (03 §6's row is unrulled)",
            }));
        }
        self.thought_id = thought_id.saturating_add(1);
    }

    fn settle_round(&self) -> Option<u16> {
        self.settle.len().checked_sub(0).map(|_| 1)
    }

    /// The serve path: one cited row, through the ledger's only promotion.
    fn serve_cited(&mut self, t: i32, row_id: u32, thought_id: u32) -> Result<()> {
        let mut shelf = None;
        for candidate_shelf in [SHELF_S_R_PAIRS, SHELF_FACTS] {
            if self.store.row_at(candidate_shelf, row_id)?.is_some() {
                shelf = Some(candidate_shelf);
                break;
            }
        }
        let Some(shelf) = shelf else { return Ok(()) };
        let served = gate::serve(&mut self.store, shelf, row_id, t, &mut self.stats)?;
        self.last_output_row = Some((shelf, row_id));
        self.log(t, 0, EV_SIGNAL, format!(
            "{{\"signals\":1,\"realized_gain\":{},\"row\":{row_id},\"promoted\":{},\"thought\":{thought_id},\
             \"rows\":[{{\"signal\":1,\"magnitude_q12\":{},\"citation\":{row_id}}}]}}",
            served.magnitude_q12,
            served.promoted,
            served.magnitude_q12
        ));
        drive::update(
            &mut self.a,
            &[si_affect::Signal::new(si_affect::SignalKind::RealizedGain, served.magnitude_q12)],
            4097,
            thought_id,
            &self.params,
            &mut self.trajectory,
        );
        self.door_call(
            t,
            thought_id,
            "update",
            &format!("{{\"signals\":1,\"slot\":4097,\"magnitude_q12\":{}}}", served.magnitude_q12),
        );
        Ok(())
    }

    /// Write or reuse the queue row for a named dark slot (04 §11 K.5's cap and
    /// dedupe live in the drive; the row's identity is the ask's).
    fn ask(
        &mut self,
        t: i32,
        stage: u8,
        pattern: PatternId,
        slot: &SlotKey,
        decoded: &si_front::Decoded,
        source: Source,
    ) -> Result<(u32, bool)> {
        let budget = drive::budget(&self.params);
        let emitted = drive::emit(
            &[drive::Ranked {
                gap: drive::Gap {
                    slot: slot.point,
                    salience_q12: budget.salience_floor,
                    last_pulsed_at: t,
                    usage_q12: si_core::quant::Q12_ONE,
                },
                weight_q12: budget.salience_floor,
            }],
            &self.params,
        );
        self.ask_doors(t, self.thought_id, slot.point, emitted.len());
        if emitted.is_empty() {
            return Ok((0, false));
        }
        if let Some(existing) = gate::ask_at(&self.store, t, pattern.index() as u16, slot.point, 0)? {
            if existing.status == gate::Q_OPEN {
                self.last_output_row = Some((SHELF_QUESTIONS, existing.row_id));
                return Ok((existing.row_id, false));
            }
        }
        let text_ref = self.store.interner_mut().intern(decoded.surface());
        let ctx = WriteContext {
            t,
            prov_session: self.prov_session,
            source,
            class: self.class_of(self.world.line_identity, t),
            stage,
            a_formation: self.a.a(),
            thought_id: self.thought_id,
            signal_ref: t as u32,
        };
        let question_id = gate::append_question(
            &mut self.store,
            &QuestionSpec {
                pattern_id: pattern.index() as u16,
                gap_kind: 0,
                origin: gate::ASK_ORIGIN_SENSE,
                key_point: slot.point,
                target_slot: 0,
                text_ref,
                prov_step: t,
                arousal_at_ask: self.a.a(),
            },
            &ctx,
        )?;
        self.stats.asks += 1;
        self.last_output_row = Some((SHELF_QUESTIONS, question_id));
        Ok((question_id, true))
    }

    /// 01 §6: the `context` row is the substrate's control group — raw surface
    /// only, `charge_off = 0`, `edges_off = 0`, no terms, no charge.
    ///
    /// It also returns the row's id, because 04 §28 Z.2/P1 replays **this** row:
    /// the sleep's first pass walks the `context` rows newer than the last night,
    /// and the row id is the only handle that ties a replayed lighting back to the
    /// line that paid for it.
    fn shelve_context(&mut self, t: i32, stage: u8, source: Source, text_ref: u32) -> Result<u32> {
        let mut envelope = Envelope::live(SHELF_CONTEXT, t, self.prov_session, t);
        envelope.prov_speaker = source_byte(source);
        envelope.stage = stage;
        envelope.text_ref = text_ref;
        let body = Body { stim_surface: text_ref, ..Body::default() };
        let row_id = self.store.append_row(envelope, body, Vec::new(), &[], &[])?;
        self.sleep.context_row = Some((row_id, t));
        Ok(row_id)
    }

    /// The stage frame and the tick's closing hash.
    fn finish(
        &mut self,
        t: i32,
        thought_id: u32,
        stage: u8,
        a_before: u8,
        output: Output,
        rows_written: u32,
        salience_calls: u32,
    ) -> Result<TickReport> {
        // A sense tick that never reached a walk — a refusal, an inert supporter
        // line — has no round-0 vector to hand P1. Its `context` row stays in the
        // replay table *unlit* rather than borrowing a later tick's lighting: the
        // sleep will count it and walk none, which is the true story (Z.2/P1).
        self.sleep.context_row = None;
        self.log(t, 0, EV_STAGE, format!("{{\"stage\":{stage}}}"));
        // OD-103: the audit line closes the tick after every deposit and before
        // the advance, so `t` names the tick the table was in when it closed.
        self.log_silt(t);
        self.clock.advance();
        let hash = self.state_hash()?;
        let mut digest = [0u8; 16];
        digest.copy_from_slice(hash.digest().as_slice());
        self.log(t, 0, EV_TICK_END, format!(
            "{{\"t\":{t},\"hash\":\"{}\",\"eventlog_len\":{},\"a\":{}}}",
            hex16(&digest),
            self.events.len(),
            self.a.a()
        ));
        Ok(TickReport {
            t,
            thought_id,
            stage,
            a_before,
            a_after: self.a.a(),
            output,
            rows_written,
            salience_calls,
        })
    }

    /// 04 §19 S.2 steps 1–3: the queue revisit, then RE-PARSE, then — only with
    /// the agenda empty and the rate bound open — the ignition. The first two
    /// write belief rows only through the derived-row path; the third writes
    /// nothing (I9's boundary), it only thinks and renders into the monologue
    /// lane. `IDLE ⇔ senses quiet AND agenda empty` is the caller's gate; a tick
    /// that still has an open ask after the revisit never reaches ignition.
    pub fn idle_tick(&mut self) -> Result<IdleReport> {
        let t = self.clock.now();
        // 02 §11 step 5a, the same order a sense tick keeps: silt decays once per
        // tick — an idle tick too, or a quiet hour would freeze the table — and
        // the decay reads the clock this tick was opened at, before any deposit.
        self.silt.tick(self.conversation_key(), t);
        self.route = None;
        self.tongue_tick = None;
        let stage = gate::stage_of(&self.store)?;
        let mut revisited = Vec::new();
        let mut derived = Vec::new();

        let open = gate::open_asks(&self.store, t)?;
        for ask in open {
            // OD-128 / 12 §2.4: an `origin = 2` nomination ask answers only in
            // the owner lane that mints its point (B17). The revisit is not that
            // lane: letting it resolve the ask would have SI answer its own
            // nomination from the very rows it noticed. The ask stays open, so
            // it keeps the agenda non-empty and suppresses ignition exactly as
            // any other unanswered ask does.
            if ask.origin() == gate::ASK_ORIGIN_NOMINATION {
                continue;
            }
            let pattern = PatternId::from_index(ask.pattern_id as u8).unwrap_or(PatternId::Undecoded);
            let frame_point = self.store.points_mut().ensure_frame(pattern);
            let surface = self.store.points().surface(ask.key_point).unwrap_or("").to_string();
            let slot = SlotKey { frame: pattern, point: ask.key_point };
            let retrieval = self.walk_read(
                t,
                self.thought_id,
                si_front::keys(&surface),
                Some(frame_point),
                Some(slot),
            )?;
            if let Some(cited) = retrieval.cited.first() {
                let ctx = WriteContext {
                    t,
                    prov_session: self.prov_session,
                    source: Source::Owner,
                    class: si_core::world::SpeakerClass::Owner,
                    stage,
                    a_formation: self.a.a(),
                    thought_id: self.thought_id,
                    signal_ref: t as u32,
                };
                let version = gate::resolve_ask(&mut self.store, &ask, cited.row_id, &ctx, t)?;
                self.log(t, 0, EV_SUPERSEDE, format!(
                    "{{\"event\":\"resolve\",\"question\":{},\"version_row\":{version},\"resolved_by\":{},\"by\":\"revisit\"}}",
                    ask.row_id, cited.row_id
                ));
                revisited.push((ask.row_id, cited.row_id));
            }
        }

        if stage >= 1 {
            let parser = SessionParser;
            let ctx = WriteContext {
                t,
                prov_session: self.prov_session,
                source: Source::Owner,
                class: si_core::world::SpeakerClass::Owner,
                stage,
                a_formation: self.a.a(),
                thought_id: self.thought_id,
                signal_ref: t as u32,
            };
            let outcomes = si_learn::reparse::pass(
                &mut self.store,
                &parser,
                &ctx,
                &self.params,
                &mut self.stats,
            )?;
            for outcome in outcomes {
                if let si_learn::reparse::PassOutcome::Derived { sr_row, row_id } = outcome {
                    self.log(t, 0, EV_COMMIT, format!(
                        "{{\"row\":{row_id},\"shelf\":{SHELF_FACTS},\"branch\":\"reparse\",\"from\":{sr_row}}}"
                    ));
                    derived.push((sr_row, row_id));
                }
            }
        }

        // 04 §19 S.2 step 3 — ignition. Only now: the agenda has been revisited,
        // so this reads `open_asks` again and fires only on the emptied queue
        // (§S.1). The rate bound (§S.6) is the only gate the V2 launch state
        // touches, and it is counted in logical ticks, never wall-clock (I5).
        let mut ignited = None;
        let mut rehearsal = None;
        let agenda_clear = gate::open_asks(&self.store, t)?.is_empty();
        // 04 §19 S.1's predicate, counted for step 12b (`IDLE ⇔ senses quiet AND
        // agenda empty`; an idle tick is by construction a tick step 1 found no
        // frame in). Z.1 says the sleep trigger counts *the same* predicate "one
        // definition, not two", so this reuses the very boolean §S.2 step 3
        // tested above rather than deriving a second one — and a tick that spent
        // itself revisiting an open question is not a quiet tick. A sense line
        // zeroes the run (`tick`); the count is derived RAM, never a byte (01 §13).
        if agenda_clear {
            self.sleep.quiet_run += 1;
        }
        let per_window = self
            .idle
            .per_window_override
            .unwrap_or(species("IDLE_PER_HOUR") as u32);
        let within_rate = self.idle.idles_this_window < per_window;
        // 04 §33 AE.4 / OD-126: the window's one consultation. It is a **deny**
        // conjunct on the predicate above and nothing else — it cannot start a
        // thought, cannot reach `sleep::due` (the counter above is the quiet law
        // alone), cannot advance the clock and reads no frame. A suppressed tick
        // still revisited its agenda and still decayed silt, because those are not
        // an idle thought.
        let denied = self.window_denies(t);
        let suppressed_by_window = denied && agenda_clear && within_rate;
        if agenda_clear && within_rate && !denied {
            let thought_id = self.thought_id;
            let graph = si_walk::Graph::build(&self.store, t)?;
            let weights = self.idle.weights_override.unwrap_or_else(|| si_walk::IdleWeights {
                recency_q12: self.temperament.get(DerivId::IgnRecency),
                bake_q12: self.temperament.get(DerivId::IgnBake),
                decay_deficit_q12: self.temperament.get(DerivId::IgnDecayDeficit),
                contradiction_q12: self.temperament.get(DerivId::IgnContradiction),
                silt_q12: self.arc.ign_weight_silt,
            });
            // §S.3: decay-deficit and contradiction-warmth are read-side
            // statistics the caller supplies. The deficit has a producer: the
            // rehearsal pair (01 §8) exists from the edge's formation and the
            // curve value is derived on read from the one clock (R-29), so a
            // point's deficit is how far its furthest-drifted live edge has
            // fallen toward its floor — no constant is read and no value is
            // invented. Warmth still has no producer: which region an unsettled
            // contradiction occupies is OPEN-28's undecided adjacency rule, so
            // the set is empty and its term contributes zero by construction.
            let deficit = self.ignition_deficit(&graph, t);
            let warmth: BTreeSet<u32> = BTreeSet::new();
            // 12 §30 AB.2: the fifth term reads the table through an argument, so
            // si-walk stays ignorant of si-silt. The table is the same one the
            // cascade will leave behind at this tick's tail; the deposit comes
            // after the draw, so the draw sees the state the prior tick closed.
            let no_silt: BTreeMap<u32, u16> = BTreeMap::new();
            let silt = self.silt.view(self.conversation_key()).unwrap_or(&no_silt);
            let seed = si_walk::ignite(
                &graph,
                t,
                &self.idle.last_lit,
                &deficit,
                &warmth,
                silt,
                &weights,
                &self.params,
                &mut self.rng,
            );
            if let Some(point) = seed {
                let deficit_before = deficit.get(&point).copied().unwrap_or(0);
                // Copied before the deposit below: the fifth term's value at the
                // moment of the draw, which is what the log must report.
                let silt_level = silt.get(&point).copied().unwrap_or(0);
                let silt_term = mul_q12_u16(weights.silt_q12, silt_level);
                let params = self.params;
                let curve = self.curve.clone();
                // 12 §4.1 step 5b: an idle frame's index is built the same way
                // the read path builds it, so a quiet tick can drag an origin in.
                let tugs = self.episodic_tugs(t)?;
                let config = WalkConfig {
                    params: &params,
                    curve: &curve,
                    k_max: self.walk.k_max,
                    tau_settle: self.walk.tau_settle,
                    decay_base: self.walk.decay_base,
                    hub_exponent_q12: self.walk.hub_exponent_q12,
                    frontier_cap: self.walk.frontier_cap,
                    oscillation: self.walk.oscillation,
                    ghost_max: self.arc.ghost_max,
                    shape_max: self.arc.shape_max,
                    tugs: &tugs,
                    relight_max: self.arc.relight_max,
                    // 12 §4.1 step 12: the idle walk is not a tick's read path, so
                    // it takes no skill hint.
                    composition_hint: &[],
                };
                let mut thought = si_walk::cascade(&graph, point, &mut self.a, thought_id, &[], &config)?;
                // docs/23 §23.4(b) / OD-105: the `primed` tag's producer. The
                // only selection that reads silt is this seed choice (OD-104(a)),
                // so the round-0 seed light carries the tag exactly when the
                // fifth term was numerically non-zero — the tag means "this
                // frame's existence was silt's doing", and at the floor
                // `mul_q12(1, 1) == 0` keeps it false, which is why the unarmed
                // pair's equality legs still hold. At B15 the idle frame is
                // post-commit, so the tag reaches `mindlog.json` here and B16's
                // tug re-light is the producer on frames 7a observes.
                if silt_term > 0 {
                    for light in thought.frame_log.lights.iter_mut() {
                        if light.round == 0 && light.point == point {
                            light.primed = true;
                        }
                    }
                }
                // 12 §30 AB.1: an ignition cascade is a frame like any other, so
                // the live-layer points it lit deposit at step 7's tail — after
                // the draw (the sampler saw the prior tick's table) and before
                // this tick's `silt.jsonl` line reports the result.
                self.deposit_frame(t, &thought.frame_log)?;
                // 04 §S.7's may-column, exercised: the edges this cascade
                // traversed have their rehearsal curve reset to this tick. The
                // pair is RAM and not a hash item, so no durable byte moves
                // (G10(b)), and the reset is the feedback loop — the point that
                // was about to be forgotten is the point just thought about.
                let rehearsed: Vec<u32> = thought
                    .rounds
                    .iter()
                    .flat_map(|round| round.contributions.iter())
                    // G13(e): the curve is a live column. A corpus edge has no
                    // `last_touched_at` to move, so its address never enters.
                    .filter(|contribution| !contribution.genome)
                    .map(|contribution| contribution.edge_off)
                    .collect();
                let arc_before = rehearsed
                    .iter()
                    .map(|edge| self.arc_drift(*edge, t))
                    .max()
                    .unwrap_or(0);
                for edge in &rehearsed {
                    self.store.touch_edge_curve(*edge, t);
                }
                // 05 §15 `G10.e.2` needs the recovery measured, not asserted. Both
                // readings come from the one clock curve; the four raw §S.3 inputs
                // are logged with the weights this tick used, each read through its
                // own home, so no sampler arithmetic is re-derived here and the log
                // cannot disagree with the draw that fired.
                let arc_after = rehearsed
                    .iter()
                    .map(|edge| self.arc_drift(*edge, t))
                    .max()
                    .unwrap_or(0);
                let point_next = self.point_deficit(&graph, t + 1, point);
                rehearsal = Some(RehearsalShift {
                    traversed: rehearsed.len(),
                    arc_before_q12: arc_before,
                    arc_after_q12: arc_after,
                    point_before_q12: deficit_before,
                    point_next_q12: point_next,
                });
                let recency_u8 = self
                    .idle
                    .last_lit
                    .get(&point)
                    .map(|last| clock::recency(t, *last))
                    .unwrap_or(0);
                let bake_q12 = graph
                    .bake_touch_max()
                    .get(&point)
                    .copied()
                    .unwrap_or(0);
                self.log(
                    t,
                    0,
                    EV_IGNITION,
                    format!(
                        "{{\"thought\":{thought_id},\"seed\":{point},\"lit\":{},\"traversed\":{},\
                         \"arc_deficit_q12\":[{arc_before},{arc_after}],\
                         \"point_deficit_q12\":[{deficit_before},{point_next}],\
                         \"inputs\":{{\"recency_u8\":{recency_u8},\"bake_q12\":{bake_q12},\"silt_level\":{silt_level},\"warmth\":false}},\
                         \"weights_q12\":{{\"recency\":{},\"bake\":{},\"deficit\":{},\"warmth\":{},\"silt\":{}}}}}",
                        thought.settle_set.len(),
                        rehearsed.len(),
                        weights.recency_q12,
                        weights.bake_q12,
                        weights.decay_deficit_q12,
                        weights.contradiction_q12,
                        weights.silt_q12,
                    ),
                );
                self.record_thought(t, thought_id, &thought);
                // §S.5: render the cascade through the same reverse face. The
                // monologue is a truthful rendering of what lit — no invented
                // top-k, since `MONOLOGUE_RENDER_DEPTH` is still Derived with no
                // ruled value (OPEN-27); it renders the whole lit set so it can
                // neither drop a real citation nor add a token that did not light.
                let depth = thought.settle_set.len();
                let words: Vec<(u32, String)> = thought
                    .settle_set
                    .iter()
                    .map(|entry| {
                        (
                            entry.point,
                            self.store.points().surface(entry.point).unwrap_or("").to_string(),
                        )
                    })
                    .collect();
                let rendered = si_front::render_monologue(&thought, &words, depth);
                let shown = thought.display(depth);
                // §G10.d.1: each displayed point maps onto the edge that carried
                // it and the round it first lit, read straight off the cascade's
                // own trace — a citation, not a re-walk.
                let cites: Vec<(u32, u32, u32)> = shown
                    .iter()
                    .map(|entry| {
                        let reached = thought.rounds.iter().find_map(|round| {
                            round
                                .contributions
                                .iter()
                                .find(|contribution| contribution.to_point == entry.point)
                                .map(|contribution| (u32::from(round.round), contribution.edge_off))
                        });
                        match reached {
                            Some((round, edge)) => (entry.point, round, edge),
                            None => (entry.point, u32::from(entry.first_lit), 0),
                        }
                    })
                    .collect();
                let lit_points: Vec<u32> = shown.iter().map(|entry| entry.point).collect();
                self.monologue.push(MonologueLine {
                    t,
                    thought: thought_id,
                    seed_point: point,
                    surface: rendered.surface,
                    cites,
                    lit_points,
                });
                self.idle.idles_this_window += 1;
                ignited = Some(point);
            }
        }

        // Step 12b: strictly after the idle work, before step 13's flush → compose
        // → advance (10 §7's precedence row). A sleep that ran walks has already
        // spent its own ticks; the advance below closes the last one.
        self.run_sleep(false)?;
        self.log_silt(t);
        self.clock.advance();
        Ok(IdleReport { t, revisited, derived, ignited, rehearsal, suppressed_by_window })
    }

    /// 04 §28 Z: step 12b. `si-sleep` owns the four passes and cannot call back
    /// into the session — it would need `&mut` to the store it already holds — so
    /// it returns the walk trail and the session closes each walk the same way an
    /// ordinary thought is closed: `record_thought`, then the clock. **`w` walks
    /// are `w` ticks** (04 §2 note 7, OD-78): the hosting tick's step-13 advance
    /// closes the last, so a night of `w` walks leaves the clock at `t₀ + w` and
    /// never further (**G13(l).2**).
    fn run_sleep(&mut self, freeze_requested: bool) -> Result<()> {
        let now = self.clock.now();
        // One attempt per logical tick, and one *completed* sleep per checkpoint
        // content: `last_slept_at_t` is the durable half of the guard, the field a
        // resume reads so it cannot consolidate the same night twice (OD-81,
        // 01 §13 item 28).
        if self.sleep.ran_at == Some(now) || self.core.last_slept_at_t == now {
            return Ok(());
        }
        let mut config = self.sleep.config;
        if let Some(cadence) = self.sleep.cadence_override {
            config.cadence_ticks = cadence;
        }
        // `sense_pending` is false at this entrance by construction: step 1 of this
        // very tick read the inbox and found nothing, which is what made the tick
        // quiet (04 §2 note 9). The frame **05 §18(g)** appends is scheduled at a
        // named pass boundary and is heard by `run()`'s own reads, not here.
        let freeze_due = freeze_requested && self.sleep.freeze_armed;
        if si_sleep::due(&config, self.sleep.quiet_run, freeze_due, false) != si_sleep::Due::Yes {
            return Ok(());
        }
        self.sleep.ran_at = Some(now);
        let rig = si_sleep::SleepRig {
            params: self.params,
            curve: self.curve.clone(),
            k_max: self.walk.k_max,
            tau_settle: self.walk.tau_settle,
            decay_base: self.walk.decay_base,
            hub_exponent_q12: self.walk.hub_exponent_q12,
            frontier_cap: self.walk.frontier_cap,
            oscillation: self.walk.oscillation,
            ghost_max: self.arc.ghost_max,
            shape_max: self.arc.shape_max,
        };
        let scheduled = self.sleep.interrupt_before_pass;
        let sense_pending = move |pass: u8| scheduled == Some(pass);
        let replay = std::mem::take(&mut self.sleep.replay);
        let last_slept_at_t = self.core.last_slept_at_t;
        let thought_id_start = self.thought_id;
        let (report, replay_back) = {
            let mut request = si_sleep::SleepRequest {
                store: &mut self.store,
                integrator: &mut self.a,
                rig,
                config,
                now,
                last_slept_at_t,
                thought_id_start,
                replay,
                sense_pending: &sense_pending,
            };
            let report = si_sleep::run(&mut request)?;
            // A night that was cut short consolidates nothing, so its replay table
            // is handed back untouched and the next quiet window sees the same
            // sense events (**G13(g).3**'s "replays to the same partial state").
            let replay = std::mem::take(&mut request.replay);
            (report, replay)
        };
        let aborted = report.aborted_reason.is_some();
        if aborted {
            self.sleep.replay = replay_back;
        }
        // Each walk of the trail is a quiet tick with the note-1 shape: its events
        // carry its own `t`, and §19's per-thought bookkeeping runs in order.
        for (index, walk) in report.trail.iter().enumerate() {
            if index > 0 {
                self.clock.advance();
            }
            debug_assert_eq!(walk.t, self.clock.now());
            self.record_thought(walk.t, walk.thought_id, &walk.thought);
        }
        // 10 §8: one line per pass that ran, each with that pass's own field set —
        // the four structs serialize as the table's keys and nothing else. A cut
        // short night therefore emits the lines of the passes it completed, in
        // order, and its sleep line *is* the interrupt record (`{t,
        // passes_completed, aborted_reason}` of 08 §2.1 is this line's subset).
        let mut lines: Vec<serde_json::Value> = Vec::new();
        if let Some(p1) = &report.p1 {
            lines.push(serde_json::to_value(p1)?);
        }
        if let Some(p2) = &report.p2 {
            lines.push(serde_json::to_value(p2)?);
        }
        if let Some(p3) = &report.p3 {
            lines.push(serde_json::to_value(p3)?);
        }
        if let Some(p4) = &report.p4 {
            lines.push(serde_json::to_value(p4)?);
        }
        let sleep_line = serde_json::json!({
            "t": report.t,
            "passes_completed": report.passes_completed,
            "rows_touched": report.rows_touched,
            "bytes_changed": report.bytes_changed,
            "aborted_reason": report.aborted_reason,
        });
        lines.push(sleep_line);
        for line in &lines {
            self.log(now, 0, EV_SLEEP_PASS, line.to_string());
            self.sleep.lines.push(line.clone());
        }
        // 06 §3.4's 9-or-5 bytes: a completed sleep moves `sleeps_completed`,
        // `last_slept_at_t` and clears the mask; an aborted one moves
        // `sleeps_interrupted` and writes the passes it did complete. The mask is
        // 0 for a completed night by 01 §13 item 32's own wording.
        if aborted {
            self.core.sleeps_interrupted = self.core.sleeps_interrupted.saturating_add(1);
            self.core.sleep_pass_mask = report.pass_mask();
        } else {
            self.core.sleeps_completed = self.core.sleeps_completed.saturating_add(1);
            self.core.last_slept_at_t = now;
            self.core.sleep_pass_mask = 0;
        }
        // P3's before/after table: `sleep_diff.json`'s rows, and the evidence the
        // (b)/(c)/(d)/(j) legs are graded from rather than a sentence about it.
        self.sleep.ledger.extend(report.ledger);
        Ok(())
    }

    /// 10 §7's trigger 2 (**OD-81**): the freeze before a checkpoint. It is not a
    /// verb and no panel can press it — the only caller is the path that is about
    /// to write `SIck`, and `due()` is the same function the cadence calls.
    pub fn freeze(&mut self) -> Result<()> {
        self.run_sleep(true)
    }

    /// 04 §S.3's third term, read the way §10's table names it: "how far an
    /// edge's `last_touched_at` has drifted toward its floor; feeds
    /// `IGN_WEIGHT_DECAY_DEFICIT`". Per point the pull is its **furthest**-
    /// drifted live edge, because the term's meaning is "what it is about to
    /// forget" (docs/19 §19.2) — a point is only as remembered as its
    /// most-forgotten arc, and reading the freshest arc instead would make a
    /// reheated edge hide the decay of every other one. `Clock::recency` is the
    /// one curve (R-29); an edge with no rehearsal pair has no curve to read,
    /// so it pulls with zero deficit rather than an invented one. Reads nothing
    /// that is stored, writes nothing.
    fn ignition_deficit(&self, graph: &si_walk::Graph, now: i32) -> BTreeMap<u32, u16> {
        let mut deficit: BTreeMap<u32, u16> = graph.points().map(|point| (point, 0)).collect();
        for source in graph.points() {
            for arc in graph.arcs(source) {
                // The rehearsal curve is keyed by arena offset, so a genome arc's
                // CSR index must never be read as one — it can alias a live edge
                // that does have a curve (07 §9's two record shapes).
                if arc.is_genome() {
                    continue;
                }
                let drift = self.arc_drift(arc.edge_off, now);
                for endpoint in [source, arc.to_point] {
                    let slot = deficit.entry(endpoint).or_insert(0);
                    *slot = (*slot).max(drift);
                }
            }
        }
        deficit
    }

    /// How far one edge's rehearsal curve has fallen toward its floor by `now`.
    fn arc_drift(&self, edge_off: u32, now: i32) -> u16 {
        match self.store.last_touched(edge_off) {
            Some(last) => Q12_ONE - u8_to_q12(clock::recency(now, last)),
            None => 0,
        }
    }

    /// The same reading for one point: the furthest-drifted live arc it touches.
    fn point_deficit(&self, graph: &si_walk::Graph, now: i32, point: u32) -> u16 {
        let mut worst = 0u16;
        for arc in graph.arcs(point) {
            if arc.is_genome() {
                continue;
            }
            worst = worst.max(self.arc_drift(arc.edge_off, now));
        }
        for source in graph.points() {
            for arc in graph.arcs(source) {
                if arc.is_genome() || arc.to_point != point {
                    continue;
                }
                worst = worst.max(self.arc_drift(arc.edge_off, now));
            }
        }
        worst
    }

    /// 04 §21 row 2(b)'s config half: the BASE cap can fire only where a genome
    /// exists, and the attached layer is that fact. CONFIG SI keeps the shipped
    /// `Caps::registry()` reading, engine off, byte for byte.
    fn label_caps(&self) -> si_front::Caps {
        si_front::Caps { engine: self.store.frozen().is_some(), ..si_front::Caps::registry() }
    }

    fn arrive(&mut self, key: &str, surface: &str) -> u32 {
        if key.is_empty() {
            return self.store.points_mut().ensure_term("", "");
        }
        self.store.points_mut().ensure_term(key, surface)
    }

    fn log(&mut self, t: i32, source: u8, kind: u8, body: String) {
        self.events.append(t, source, kind, body.into_bytes());
    }

    /// 08 §2.1 row 9: `{door, thought_id, arg_digest}`. The digest is sha256 of
    /// the door's own argument text, so the panel can prove two calls took
    /// different arguments without the viewer holding a second copy of the
    /// drive's inputs (§1's first law: process B reads frames, never state).
    fn door_call(&mut self, t: i32, thought_id: u32, door: &str, args: &str) {
        self.log(
            t,
            0,
            EV_DOOR,
            format!(
                "{{\"door\":\"{door}\",\"thought\":{thought_id},\"arg_digest\":\"{}\"}}",
                si_core::sha256::sha256_hex(args.as_bytes())
            ),
        );
    }

    /// Step 8's two queue doors, framed once per ask-shaped call. `budget`'s
    /// argument set is genuinely empty — [02 §11] gives it "no `a`, no level, no
    /// signal" (OD-22), so its `arg_digest` is the hash of `{}` and the panel
    /// shows that rather than a number the door never took.
    ///
    /// **OD-145**: four of the six doors have a frame. `prioritize` has no
    /// production call site anywhere in `src/` (the caller's single `Ranked`
    /// arrives pre-weighted), and `salience` fires inside `si-learn`'s commit,
    /// where the session cannot see it — its output rides the commit frame
    /// instead. A `door_call` for either would be a picture of a call this build
    /// does not make.
    fn ask_doors(&mut self, t: i32, thought_id: u32, anchor: u32, emitted: usize) {
        self.door_call(t, thought_id, "budget", "{}");
        self.door_call(
            t,
            thought_id,
            "emit",
            &format!("{{\"gaps\":1,\"anchor\":{anchor},\"asks\":{emitted}}}"),
        );
    }

    /// 08 §2.1 row 17: the supporter seam's refusal, machine-readable. The seam
    /// lives in the tick driver, not the session, and its frame builder already
    /// emits the row's three fields verbatim to `gate.jsonl`
    /// ([si-supporter::seam::Attempt::frame]); this forwards that same string so
    /// a panel line and the gate log can never disagree, and so no `si-session` →
    /// `si-supporter` edge appears in [02 §10]'s import table.
    ///
    /// The source byte is the supporter's (`1`) for every frame: the author of a
    /// refusal is the seam, whatever identity the rejected line arrived with.
    pub fn note_seam_reject(&mut self, t: i32, frame: &str) {
        self.log(t, 1, EV_SEAM_REJECT, frame.to_string());
    }

    fn absorb(&mut self, t: i32, thought_id: u32, report: &WriteReport, key_point: u32, response: &str) {
        for event in &report.events {
            let kind = match event.kind {
                gate::EV_NOVELTY => EV_SIGNAL,
                gate::EV_CONTRADICTION => EV_GAP,
                _ => EV_COMMIT,
            };
            self.log(t, 0, kind, format!("{{\"thought\":{thought_id},\"gate\":{}}}", event.body));
        }
        let _ = (key_point, response);
    }

    // -----------------------------------------------------------------------
    // Artifacts.
    // -----------------------------------------------------------------------

    pub fn hash_inputs(&self) -> Result<HashInputs> {
        let mut inputs = self.store.hash_inputs()?;
        inputs.settle_set = encode_settle_set(&self.settle);
        inputs.trajectory = encode_trajectory(&self.trajectory);
        inputs.rng = encode_rng(&self.rng.state(), &self.advances);
        inputs.version_tags = self.tags.encode();
        inputs.core_block = self.core.encode().to_vec();
        Ok(inputs)
    }

    /// 01 §10 / 04 X.1: the nine item classes over the live state.
    pub fn state_hash(&self) -> Result<StateHash> {
        Ok(StateHash::compose(&self.hash_inputs()?))
    }

    /// 05 §23 / 12 §6.4 G15(a): the same nine-item composition with the
    /// `questions` shelf filtered out of item 1. A reader that opens one
    /// `origin = 2` ask really does change `live_rows`, so a whole-state claim
    /// across an on/off pair would be false; this digest is the honest one.
    /// Item 4 (the interner) is untouched, which is what makes "the interner is
    /// byte-identical" a real comparison rather than a restatement.
    pub fn questions_filtered_digest(&self) -> Result<String> {
        let mut inputs = self.hash_inputs()?;
        let mut live: Vec<si_core::record::Record> = self
            .store
            .rows()?
            .into_iter()
            .filter(|(_, record)| {
                record.envelope.is_live() && record.envelope.shelf_x != SHELF_QUESTIONS
            })
            .map(|(_, record)| record)
            .collect();
        live.sort_by_key(|record| (record.envelope.shelf_x, record.envelope.row_id));
        let mut bytes = Vec::with_capacity(live.len() * si_core::record::RECORD_BYTES);
        for record in live {
            bytes.extend_from_slice(&record.encode());
        }
        inputs.live_rows = bytes;
        Ok(StateHash::compose(&inputs).hex())
    }

    /// The `questions` shelf on its own terms (05 §23): count and, for each live
    /// row, `(row_id, key_field, val_point)` in row-id order. A leg compares this
    /// pair directly, so "the only extra rows are the family's own asks" is a
    /// statement about rows, never about a digest that hides them.
    pub fn questions_shelf(&self) -> Result<Vec<(u32, u16, u32)>> {
        let mut out: Vec<(u32, u16, u32)> = self
            .store
            .rows()?
            .into_iter()
            .filter(|(_, record)| {
                record.envelope.is_live() && record.envelope.shelf_x == SHELF_QUESTIONS
            })
            .map(|(_, record)| {
                let body = record.body;
                (record.envelope.row_id, body.key_field, body.val_point)
            })
            .collect();
        out.sort_unstable();
        Ok(out)
    }

    /// 12 §6.4 G15(c)'s frame-trace-fixed reading: feed a recorded
    /// `frame_log[]` trace through a **fresh** detector over this session's store
    /// and return the candidate table's canonical bytes. `a` is not an argument —
    /// the detector cannot see it — so two `a` regimes are the same call, and the
    /// bytes are the proof rather than a restatement.
    pub fn crystallize_trace_table(&self, frames: &[serde_json::Value]) -> Result<Vec<u8>> {
        let mut detector = si_crystallize::Crystallizer::new(self.arc.crystallize_config());
        for frame in frames {
            let log = si_core::frame::FrameLog::from_json(frame)
                .map_err(|error| anyhow::anyhow!("frame log: {error}"))?;
            let t = frame["t"].as_i64().unwrap_or(0) as i32;
            detector.observe(&log, &self.store, t)?;
        }
        Ok(detector.table_bytes())
    }

    /// 10 §10: the derived vector as a table, never as a stored field. No
    /// nickname and no explanation — the digest is the checkable claim.
    pub fn fingerprint(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("birth_seed 0x{:016x}\n", self.seed));
        out.push_str(&format!("seed_format 0x{:04x}\n", self.core.seed_format));
        out.push_str(&format!("deriv_order_len {}\n", self.core.deriv_order_len));
        for id in DERIV_ORDER {
            let name = id.registry_name();
            let row = consts::row(name).expect("sealed draw has a registry row");
            out.push_str(&format!(
                "{} {} [{}, {}]\n",
                name,
                self.temperament.get(id),
                row.lo,
                row.hi
            ));
        }
        out.push_str(&format!("temperament_digest 0x{:016x}\n", self.core.temperament_digest));
        out
    }

    /// 08 §6 O12 / 01 §8: the per-thought export, from the mind log the loop
    /// already wrote.
    pub fn mindlog_json(&self, run_id: &str) -> Result<String> {
        let events: Vec<serde_json::Value> = self
            .events
            .events()
            .iter()
            .map(|event| {
                serde_json::json!({
                    "seq": event.seq,
                    "t": event.t,
                    "source": event.source,
                    "kind": event.kind,
                    "body": String::from_utf8_lossy(&event.body),
                })
            })
            .collect();
        let document = serde_json::json!({
            "run_id": run_id,
            "birth_seed": format!("0x{:016x}", self.seed),
            "seed_format": format!("0x{:04x}", self.core.seed_format),
            "temperament_digest": format!("0x{:016x}", self.core.temperament_digest),
            "stage": gate::stage_of(&self.store)?,
            "a_now": self.a.a(),
            "thoughts": events,
        });
        Ok(serde_json::to_string_pretty(&document)?)
    }

    /// The run directory of 05 §1: transcript, mind log, the two lanes, the hash
    /// and the fingerprint. `monologue.jsonl` is empty until B7's ignition.
    pub fn write_run_dir(&self, dir: &Path, run_id: &str) -> Result<()> {
        std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
        let mut transcript = String::new();
        for (t, source, line) in &self.transcript {
            transcript.push_str(&format!("{t}\t{}\t{line}\n", source.literal()));
        }
        std::fs::write(dir.join("transcript.txt"), transcript)?;
        std::fs::write(dir.join("mindlog.json"), self.mindlog_json(run_id)?)?;
        let mut events = String::new();
        for event in self.events.events() {
            events.push_str(&serde_json::json!({
                "seq": event.seq,
                "t": event.t,
                "source": event.source,
                "kind": event.kind,
                "body": String::from_utf8_lossy(&event.body),
            }).to_string());
            events.push('\n');
        }
        std::fs::write(dir.join("events.jsonl"), events)?;
        std::fs::write(dir.join("monologue.jsonl"), "")?;
        std::fs::write(dir.join("fingerprint.txt"), self.fingerprint())?;
        let mut digest = [0u8; 16];
        digest.copy_from_slice(self.state_hash()?.digest().as_slice());
        std::fs::write(dir.join("state_hash.txt"), format!("{}\n", hex16(&digest)))?;
        let mut owner = String::new();
        for (t, line) in &self.rendered {
            owner.push_str(&format!("{t}\t{line}\n"));
        }
        std::fs::write(dir.join("output.txt"), owner)?;
        Ok(())
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot::capture(
            self.core,
            &self.store,
            &self.settle,
            &self.trajectory,
            self.rng.state(),
            &self.advances,
            self.a.a(),
        )
    }

    /// 00 §5: `name.tmp` → `sync_all` → `rename`.
    pub fn save(&self, path: &Path) -> Result<()> {
        self.snapshot()
            .save(path)
            .with_context(|| format!("save session to {}", path.display()))
    }

    /// Resume. Every refusal is decided from the header, before the payload is
    /// decoded (04 §25 step 5 / G14(c)).
    pub fn open(path: &Path, seed: u64) -> Result<OpenOutcome> {
        Self::open_engine(path, seed, None)
    }

    /// 04 §25 step 2's W.1, second half: a resume compares the genome the session
    /// was born on against the layer CONFIG ENGINE now has open, and the payload
    /// is decoded with that layer's allocation base. `None` is CONFIG SI, where
    /// the load has already refused and both are the shipped behavior.
    pub fn open_engine(
        path: &Path,
        seed: u64,
        pinned: Option<std::rc::Rc<Frozen>>,
    ) -> Result<OpenOutcome> {
        let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
        let inspect = Snapshot::inspect_pinned(
            &bytes,
            seed,
            pinned.as_ref().map(|layer| layer.version_tag()),
        )?;
        let header = match inspect {
            Inspect::Refused(refusal) => return Ok(OpenOutcome::Refused(refusal)),
            Inspect::Accepted(header) => header,
        };
        let base = pinned.as_ref().map_or(0, |layer| layer.genome_points());
        let snapshot = Snapshot::decode_with(&bytes, base)?;
        let session = Self::from_snapshot(snapshot, pinned).context("resume")?;
        debug_assert_eq!(session.core.birth_seed, header.core.birth_seed);
        Ok(OpenOutcome::Opened(Box::new(session)))
    }

    fn from_snapshot(snapshot: Snapshot, pinned: Option<std::rc::Rc<Frozen>>) -> Result<Self> {
        let core = snapshot.header.core;
        let temperament = seed::derive_temperament(core.birth_seed);
        if core.temperament_digest != temperament.digest() {
            bail!(
                "core block digest {:#018x} does not re-derive from seed {:#018x} (G14(c))",
                core.temperament_digest,
                core.birth_seed
            );
        }
        let mut tags = VersionTags::at_birth(core.birth_seed);
        tags.genome_t0_tag = snapshot.header.geometry.genome_t0_tag;
        let mut store = snapshot.store()?;
        // W.2's replay is not finished until the layer is back on the lattice: the
        // point base above and this attachment are the same fact read twice, which
        // is why `attach_frozen` refuses a base the payload was not decoded with.
        if let Some(layer) = pinned {
            store.attach_frozen(layer)?;
        }
        let a = Integrator::restore(snapshot.a);
        let arc = ArcConfig::from_registry();
        let now = store.now();
        Ok(Self {
            seed: core.birth_seed,
            core,
            store,
            temperament,
            tags,
            rng: Rng::from_state(snapshot.rng),
            advances: snapshot.advances,
            settle: snapshot.settle,
            route: None,
            tongue_tick: None,
            trajectory: snapshot.trajectory,
            clock: Clock::new(now),
            a,
            params: Params::from_temperament(&temperament),
            curve: fixture_curve(),
            walk: WalkConstants::from_registry(),
            events: EventLog::new(),
            stats: GateStats::default(),
            thought_id: 0,
            prov_session: 1,
            last_output_row: None,
            transcript: Vec::new(),
            rendered: Vec::new(),
            idle: IdleState::at_birth(),
            monologue: Vec::new(),
            sleep: SleepState::at_birth(),
            arc,
            crystallizer: si_crystallize::Crystallizer::new(arc.crystallize_config()),
            crystallize_lines: Vec::new(),
            frame_logs: Vec::new(),
            silt: si_silt::Silt::new(arc.silt_config()),
            silt_lines: Vec::new(),
            episodic_index: true,
            relight_lines: Vec::new(),
            skiller: si_skills::Skiller::new(arc.skills_config()),
            skill_lines: Vec::new(),
            world: World::at_birth(),
        })
    }
}

/// 12 §2.6: the frame scratch as the mind log shows it. `role` and `round` are
/// the log's whole content; a ghost carries no name (docs/22 §22.2).
fn frame_log_json(thought_id: u32, log: &si_core::frame::FrameLog) -> serde_json::Value {
    let lights: Vec<serde_json::Value> = log
        .lights
        .iter()
        .map(|light| {
            serde_json::json!({
                "point": light.point,
                "role": light.role.label(),
                "edge_kind": light.edge_kind,
                "round": light.round,
                "primed": light.primed,
            })
        })
        .collect();
    let ghosts: Vec<serde_json::Value> = log
        .ghosts
        .iter()
        .map(|ghost| {
            serde_json::json!({
                "members": ghost.members,
                "formed_in_frame": ghost.formed_in_frame,
                "frame_seed": ghost.frame_seed,
            })
        })
        .collect();
    let role_graphs: Vec<serde_json::Value> = log
        .role_graphs
        .iter()
        .map(|graph| {
            serde_json::json!({
                "roles": graph.roles,
                "traversed_kinds": graph.traversed_kinds,
                "filler_sets": graph.filler_sets,
            })
        })
        .collect();
    serde_json::json!({
        "thought": thought_id,
        "cold": log.is_cold(),
        "lights": lights,
        "ghosts": ghosts,
        "role_graphs": role_graphs,
    })
}

fn transition_label(transition: si_crystallize::Transition) -> &'static str {
    use si_crystallize::Transition as T;
    match transition {
        T::WarmFrame => "warm-frame",
        T::TooFewMembers => "too-few-members",
        T::NoSharedFiller => "no-shared-filler",
        T::AlreadyBound => "already-bound",
        T::Opened => "open",
        T::Recurred => "recurrence",
        T::InsideDelta => "inside-delta",
        T::Nominated => "nominate",
    }
}

/// `skills.jsonl`'s transition column — AD.3's two bars and AD.4's forwarding,
/// as the log names them. The label is an observation, never a lever.
fn shape_transition_label(transition: si_skills::Transition) -> &'static str {
    use si_skills::Transition as T;
    match transition {
        T::WarmFrame => "warm-frame",
        T::NoScratch => "no-scratch",
        T::Forwarded => "forwarded",
        T::Opened => "open",
        T::Recurred => "recurrence",
        T::InsideDelta => "inside-delta",
        T::Nominated => "nominate",
        T::AlreadyMinted => "already-minted",
    }
}

impl SessionParser {
    fn parse(&self, stimulus: &str, response: &str) -> Option<DerivedSlot> {
        let decoded = si_front::decode(stimulus);
        let si_front::Decoded::Pattern(pattern) = decoded else {
            return None;
        };
        if !si_front::QUESTION_FRAMES.contains(&pattern.frame) {
            return None;
        }
        let key = pattern.slots.last()?.key.clone();
        let value = si_front::tokenize(response).last()?.key.clone();
        Some(DerivedSlot {
            pattern_id: pattern.frame.index() as u16,
            key_field: Candidate::key_field_for(pattern.frame, 0),
            key,
            value,
            target_slot: 0,
        })
    }
}

struct SessionParser;

impl Parser for SessionParser {
    fn slot_of(&self, stimulus: &str, response: &str) -> Option<DerivedSlot> {
        self.parse(stimulus, response)
    }
}

fn is_function(word: &str) -> bool {
    si_core::strings::function_word(word)
}

fn source_byte(source: Source) -> u8 {
    match source {
        Source::Owner => 0,
        Source::Grimm => 1,
    }
}

fn json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn hex16(bytes: &[u8; 16]) -> String {
    let mut out = String::with_capacity(32);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// 09 §6's ledger door: the promotion is the only thing a serve may change about
/// a row's `gain_kind`, and it is one-way. Re-exported so a caller cannot reach
/// `si-store::promote_gain` directly.
pub fn serve_row(
    session: &mut Session,
    shelf: u16,
    row_id: u32,
    at_t: i32,
) -> Result<si_learn::gate::ServeReport> {
    gate::serve(&mut session.store, shelf, row_id, at_t, &mut session.stats)
}

/// 04 §18's eligibility test, for the harness's idempotence leg: one call, no
/// pass, so a second eligible check can be compared before and after.
pub fn eligible_derivations(session: &Session) -> Result<Vec<si_learn::reparse::Derivation>> {
    let parser = SessionParser;
    si_learn::reparse::eligible(&session.store, &parser)
}

/// The stage the lattice currently reads as (docs/11 §11.1).
pub fn stage(session: &Session) -> Result<u8> {
    gate::stage_of(&session.store)
}

/// The budget the drive would speak under (04 §11 K.4); read-only, and the one
/// place a harness can see that no affective field rides it.
pub fn budget(session: &Session) -> Budget {
    drive::budget(&session.params)
}

/// The lagged pointer's last value, exposed for the trace assertions.
pub fn lagged(session: &Session) -> Lagged {
    Lagged { a: session.a.a(), applies_at: 0 }
}

/// A set of every point the last settle reached, for the harness's route checks.
pub fn last_settle(session: &Session) -> BTreeSet<u32> {
    session.settle.iter().map(|(point, _)| *point).collect()
}

/// The route the current step's read reported, for 05 §11 `G6.b`'s two-traces
/// claim. `None` is a step that never reached a read, or a session that has not
/// yet ticked.
pub fn last_route(session: &Session) -> Option<si_read::RouteCounts> {
    session.route
}

/// The voice, as the boundary saw it.
///
/// There is no `reason` field, and **OD-187** says why: the walker's answer is
/// `Option<String>`, so the name of a refusal lives inside `si-tongue` and cannot
/// reach here without either an out-parameter the crate's own purity check refuses
/// or a second copy of the walk in the caller, which `G20(c)` refuses. The forcing
/// legs therefore name the refusal by the *fixture* that made it, and the four
/// arms keep their per-arm grading where each arm is constructed —
/// `crates/si-tongue/tests/realize.rs`.
#[derive(Clone, Debug)]
pub struct TongueTick {
    pub thought: u32,
    /// The anchor the walk was handed: the answer row's `key_point` (**OD-188**),
    /// or `None` when the citation named no speakable row and the arm refused
    /// before a point could be chosen.
    pub anchor: Option<u32>,
    /// How many surfaces the boundary handed the walk (**OD-181**'s mouth). A
    /// citation of one row licenses that row's composed chunk and its two terms,
    /// so the widest honest line it can speak is two chunks — **OD-189**.
    pub licensed: usize,
    /// The draw `STYLE_SPREAD` priced this tick's walk into: `1 + budget` chunks.
    /// **G20(i)**'s bound is this number, not a reviewer's arithmetic. **OD-189**:
    /// it is a ceiling, not the line's length — the citation's license binds
    /// first (a one-row citation licenses two chunks), so G20 publishes the
    /// number beside a note saying exactly that.
    pub chunk_cap: usize,
    /// The frame's own citation, copied from the render the voice replaces. The
    /// voice may not touch it (**G20(a)** check (a)), and it is what **G20(h)**
    /// greps the realized words against.
    pub citation: Vec<u32>,
    pub canonical: String,
    pub realized: Option<String>,
}

/// The current step's voice, for **G20**'s legs. `None` is a tick that never
/// entered step 9a — which at V0 is *every* tick, and is itself the evidence
/// `G20(j)` asks for.
pub fn last_tongue(session: &Session) -> Option<TongueTick> {
    session.tongue_tick.clone()
}
