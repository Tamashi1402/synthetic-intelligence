//! 05 §1: `run` executes a scenario into a run directory. It never calls a wall
//! clock for loop state, never pre-seeds a row, term, frame or `a`, and writes
//! the six artifacts plus `gate_report.json`. The loop is driven only through
//! `Session::tick` and `Session::idle_tick`; the harness owns no branch of it.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use md5::{Digest, Md5};
use si_affect::params::Params;
use si_core::addr::SHELF_S_R_PAIRS;
use si_core::container::{Container, Format};
use si_core::eventlog::Event;
use si_core::strings::Source;
use si_core::world::{GrantRow, Identity};
use si_session::{
    IdleReport, InboxDrain, MonologueLine, OpenOutcome, Output, SenseFrame, Session, TickReport,
    WalkConstants,
};
use si_store::snapshot::write_atomic;
#[cfg(feature = "supporter")]
use si_supporter::seam::{Admission, GateLog, Seam, SeamState};

use super::scenario::{self, Identities, Scenario, Step, WorldOp};
use super::{build_identity, Workspace};

/// The owner-facing lane's kinds (08 §2.1); every kind is a record the loop
/// already wrote, and the lane is a forward, never a new fact. Kind 18 is
/// reserved and never emitted, so the list skips it.
///
/// B11 put 17 and 20 back in, against the list this stage first shipped with:
/// §3's CHAT row reads `…13, 17` and §6's O12 row reads `kind 20`, so a lane
/// that drops them leaves the two panels with nothing to forward — and §11's
/// fingerprint ruling is explicitly that the readout "reaches the panel as kind
/// 20 forwarded from the loop's own re-derivation", not as a viewer-side
/// recompute. The earlier comment claimed kind 17 travels on the monologue lane;
/// it does not — that lane carries the idle cascade's own lines
/// (04 §19 S.5) and nothing else, so a seam rejection had no lane at all.
///
/// Kind 21 (`ignition`, **OD-143**) is absent on purpose: it is the idle lane's
/// record, §6 O9 counts ignitions from `SIst` and kind 16, and O10 renders
/// blank-but-present until **OD-38** rules on monologue exposure.
const LANE_KINDS: [u8; 20] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20];
const SENSE_KIND: u8 = 0;

#[derive(Clone, Debug)]
pub struct RunRequest {
    pub gate: String,
    pub variant: String,
    pub config: String,
    pub seed: u64,
    pub scenario: Scenario,
    /// 05 §4: a sweep launches a run with a declared constant list. Empty for a
    /// gate run, whose values are the registry's own.
    pub constants: BTreeMap<String, i64>,
    /// 05 §18(g): the pass boundary an owner frame is appended at, for the four
    /// interrupt runs. `None` everywhere else — the frame is a launch state of the
    /// run, never a verb, and `identity` records it so a replay re-executes the
    /// same partial night rather than a whole one.
    pub sleep_interrupt: Option<u8>,
    /// 05 §4's V7 row: `--config engine` with a frozen artifact pinned by path and
    /// seal tag. Both are `None` for every CONFIG SI run, which is what keeps
    /// 00 §6's promise that an SI launch forms no path to a frozen layer — the
    /// loader is never called, so neither is its open register fed.
    pub genome: Option<PathBuf>,
    pub genome_tag: Option<u64>,
    pub run_dir: Option<PathBuf>,
}

/// The launch state a recorded run carries, as 05 §1's cross-process replay
/// re-executes it: the run's own constants, variant and interrupt, and from B10
/// its config and pinned artifact too. A V7 directory replays as the ENGINE run it
/// is, because the handle space the genome owns is inside the state hash
/// (07 §13, 00 §6) — replaying it as an SI wake would diff for a reason that has
/// nothing to do with determinism.
#[derive(Clone, Debug, Default)]
pub struct Launch {
    pub config: String,
    pub variant: String,
    pub constants: BTreeMap<String, i64>,
    pub sleep_interrupt: Option<u8>,
    pub genome: Option<PathBuf>,
    pub genome_tag: Option<u64>,
}

impl Launch {
    /// The launch a run holds in-process.
    pub fn of(request: &RunRequest) -> Self {
        Self {
            config: request.config.clone(),
            variant: request.variant.clone(),
            constants: request.constants.clone(),
            sleep_interrupt: request.sleep_interrupt,
            genome: request.genome.clone(),
            genome_tag: request.genome_tag,
        }
    }

    /// The launch a run's own `gate_report.json` names — the read-back half of
    /// 05 §1's cross-process replay. The variant, the swept constants, the
    /// interrupt an owner frame sat at, the config and the pinned artifact all come
    /// from the report the run wrote: a V7 directory replays as the ENGINE run it
    /// is, and a CONFIG SI directory has no layer and forms no path to one (00 §6).
    /// OD-93: one read-back, so `Run::read` and the CLI's `replay` verb cannot
    /// drift into re-executing the same directory as two different runs.
    pub fn recorded(report: &serde_json::Value, ws: &Workspace) -> Self {
        let identity = report.get("build_identity");
        let mut constants = BTreeMap::new();
        if let Some(list) =
            identity.and_then(|i| i.get("constants")).and_then(|values| values.as_array())
        {
            for entry in list {
                if let Some(map) = entry.as_object() {
                    for (name, value) in map {
                        if let Some(number) = value.as_i64() {
                            constants.insert(name.clone(), number);
                        }
                    }
                }
            }
        }
        let mut genome = None;
        let mut genome_tag = None;
        if let Some(entry) = identity.and_then(|i| i.get("frozen_layer")) {
            if let (Some(path), Some(tag)) = (
                entry.get("path").and_then(|value| value.as_str()),
                entry.get("version_tag").and_then(|value| value.as_str()),
            ) {
                genome = Some(ws.path(path));
                genome_tag = Some(u64::from_str_radix(tag.trim_start_matches("0x"), 16).ok());
            }
        }
        Self {
            config: report
                .get("config")
                .or_else(|| identity.and_then(|i| i.get("config")))
                .and_then(|config| config.as_str())
                .unwrap_or("si")
                .to_string(),
            variant: identity
                .and_then(|i| i.get("build_variant"))
                .and_then(|variant| variant.as_str())
                .unwrap_or("V0")
                .to_string(),
            constants,
            sleep_interrupt: identity
                .and_then(|i| i.get("sleep_interrupt"))
                .and_then(|pass| pass.as_u64())
                .map(|pass| pass as u8),
            genome,
            genome_tag: genome_tag.flatten(),
        }
    }

    /// 00 §6 / 04 §25: the config decides the wake, and only CONFIG ENGINE reaches
    /// the loader. A CONFIG SI launch takes the shipped path unchanged, so no
    /// pre-B10 gate's bytes move; an ENGINE launch refuses rather than waking
    /// without a pinned corpus, because 07 §1 says the manifest lock is the only
    /// source list. One refusal, one place — a replay that lost the artifact in
    /// its read-back fails here, not silently as an SI wake.
    fn layer(&self) -> Result<Option<std::rc::Rc<si_store::layers_frozen::Frozen>>> {
        if self.config != "engine" {
            return Ok(None);
        }
        let (path, tag) = match (&self.genome, self.genome_tag) {
            (Some(path), Some(tag)) => (path, tag),
            _ => bail!(
                "the launch is CONFIG ENGINE and names no frozen artifact: with nothing \
                 pinned there is no source list (07 §1)"
            ),
        };
        let manifest = si_store::layers_frozen::pin(path, tag);
        let frozen = si_store::layers_frozen::load(
            si_store::layers_frozen::Config::Engine,
            Some(&manifest),
        )?;
        Ok(Some(std::rc::Rc::new(frozen)))
    }

    pub fn wake(&self, seed: u64) -> Result<Session> {
        match self.layer()? {
            None => Session::new(seed),
            Some(layer) => Session::wake_engine(seed, layer),
        }
    }

    /// The resume half of the same law. W.2's base is derived from the artifact,
    /// never read from the file, so an ENGINE run re-opens its own snapshot against
    /// the genome it was born into — and 07 §13's tag comparison runs before a row
    /// decodes.
    pub fn reopen(&self, path: &Path, seed: u64) -> Result<OpenOutcome> {
        match self.layer()? {
            None => Session::open(path, seed),
            Some(layer) => Session::open_engine(path, seed, Some(layer)),
        }
    }
}

pub fn wake_session(request: &RunRequest, seed: u64) -> Result<Session> {
    Launch::of(request).wake(seed)
}

pub fn reopen_session(request: &RunRequest, path: &Path, seed: u64) -> Result<OpenOutcome> {
    Launch::of(request).reopen(path, seed)
}

/// 07 §15's `—SIgen` twin is a different binary built from the same tree: the
/// *activated* feature set is in no digest a report already carries, because
/// `binary_tree_md5` covers sources that *declare* the options while the command
/// line chooses them. Without this field a CONFIG SI report from the engine-off
/// twin and one from the engine-on build are indistinguishable, and G5c's handle
/// audit is exactly the leg that has to say which build made it. OD-E's rule is
/// that a report records what it ran at; the build's doors are half of that, and
/// they went unrecorded until B10 ran the twin for real.
fn build_features() -> Vec<&'static str> {
    let mut out = Vec::new();
    if cfg!(feature = "engine") {
        out.push("engine");
    }
    if cfg!(feature = "supporter") {
        out.push("supporter");
    }
    if cfg!(feature = "vulkan") {
        out.push("vulkan");
    }
    out
}

/// 05 §8 `G3.em.2` / §12 `G7.a.3`: the arena byte counts and the two digests a
/// supersession or a derivation must not move. Read from the live store at the
/// step boundary, so a deleted byte or an edited stamp is caught at the step that
/// did it and not only in a final export.
#[derive(Clone, Debug)]
pub struct Boundary {
    pub rows_bytes: usize,
    pub edges_bytes: usize,
    pub stamps_bytes: usize,
    pub stamps_digest: String,
    pub sr_envelope_digest: String,
}

/// 04 §23 W.3 as the harness recorded it: what one `source: grimm` emission
/// attempt did, before and independently of the loop. `reached_loop` is the whole
/// of "a rejected emission never becomes a sense event" — when it is false the
/// step has no tick behind it, and the boundary pair around the step is equal.
#[derive(Clone, Debug)]
pub struct SpeakNote {
    pub reached_loop: bool,
    pub admitted: bool,
    /// The seam was closed (V4) or silent (V5), so the gate never ran.
    pub dropped: bool,
    pub reason: &'static str,
    pub layer: u8,
    pub n_untaught_tokens: usize,
    pub novel_claims: usize,
    pub ref_row_id: Option<u32>,
    pub untaught: Vec<String>,
}

/// The run's one route for a `source: grimm` line: process C's side of the wire.
/// It is a **harness** object, not a loop object — `si-session` has no supporter
/// branch and no `check_no_stage_on_supporter`-visible predicate reads it — and
/// the `supporter` feature decides whether the seam exists in the binary at all.
/// With the feature compiled out nothing arrives, which is the closed state, so a
/// no-supporter build is byte-identical to V4 by construction (00 §4).
pub struct Mouth {
    #[cfg(feature = "supporter")]
    seam: Seam,
    pub state: &'static str,
    pub frames: Vec<String>,
    pub reads: Vec<String>,
}

impl Mouth {
    /// Launch one seam for one section at the run's variant, logging its gate log
    /// into the run directory when there is one.
    #[cfg(feature = "supporter")]
    pub fn launch(variant: &str, log: Option<&Path>, artifacts: Option<&Path>) -> Self {
        let state = SeamState::from_variant(variant);
        let seam = match log {
            Some(path) => Seam::new(state.clone(), GateLog::at(path)),
            None => Seam::new(state, GateLog::new()),
        };
        let seam = match artifacts {
            Some(dir) => seam.with_artifacts(dir),
            None => seam,
        };
        Self { seam, state: state.label(), frames: Vec::new(), reads: Vec::new() }
    }

    #[cfg(not(feature = "supporter"))]
    pub fn launch(_variant: &str, _log: Option<&Path>, _artifacts: Option<&Path>) -> Self {
        Self { state: "closed", frames: Vec::new(), reads: Vec::new() }
    }

    /// docs/21 §21.2: the five read-only tools, called in order. V5's row in
    /// 05 §2 is the run that calls them and emits nothing, so the harness calls
    /// them on his behalf rather than pretending a silent supporter read nothing.
    #[cfg(feature = "supporter")]
    pub fn read_all(&mut self, session: &Session) -> Result<()> {
        if self.state != SeamState::Silent.label() {
            return Ok(());
        }
        for tool in ["read_trace", "read_monologue", "read_state", "read_vocab", "read_rows"] {
            self.seam.read(session.store(), tool)?;
        }
        self.reads = self.seam.reads.clone();
        Ok(())
    }

    #[cfg(not(feature = "supporter"))]
    pub fn read_all(&mut self, _session: &Session) -> Result<()> {
        Ok(())
    }

    /// 04 §23 W.3: attempt one emission **before** the loop sees it. `Ok(None)`
    /// means the line never became a sense event — the gate refused it, or the
    /// seam was closed or silent — and the caller must not call `tick`.
    #[cfg(feature = "supporter")]
    pub fn attempt(
        &mut self,
        session: &mut Session,
        text: &str,
        identity: Option<si_core::world::Identity>,
    ) -> Result<(Option<TickReport>, SpeakNote)> {
        let admission = self.seam.speak(session.store(), text)?;
        self.frames = self.seam.frames();
        // 08 §2.1 row 17: a refusal is a frame as well as a gate-log line, so the
        // CHAT panel can show the machine-readable *why* on the same timeline as
        // the answer it did not get. An admitted line needs no frame here — it
        // becomes the sense event the loop already records. `frames()` is the
        // whole log in order, so the attempt just made is the last one.
        if !matches!(admission, Admission::Admitted(_)) {
            if let Some(frame) = self.frames.last() {
                let t = session.now();
                session.note_seam_reject(t, frame);
            }
        }
        let note = match &admission {
            Admission::Admitted(verdict) => SpeakNote {
                reached_loop: true,
                admitted: true,
                dropped: false,
                reason: verdict.reason,
                layer: verdict.layer,
                n_untaught_tokens: verdict.n_untaught_tokens,
                novel_claims: verdict.novel_claims,
                ref_row_id: verdict.ref_row_id,
                untaught: verdict.untaught.clone(),
            },
            Admission::Rejected(verdict) => SpeakNote {
                reached_loop: false,
                admitted: false,
                dropped: false,
                reason: verdict.reason,
                layer: verdict.layer,
                n_untaught_tokens: verdict.n_untaught_tokens,
                novel_claims: verdict.novel_claims,
                ref_row_id: verdict.ref_row_id,
                untaught: verdict.untaught.clone(),
            },
            Admission::Dropped => SpeakNote {
                reached_loop: false,
                admitted: false,
                dropped: true,
                reason: "seam_closed",
                layer: 0,
                n_untaught_tokens: 0,
                novel_claims: 0,
                ref_row_id: None,
                untaught: Vec::new(),
            },
        };
        let report = match admission {
            // 04 §33 AE.5: a Trusted line passes this same two-layer filter over
            // the same taught lexicon; only the identity it arrives with differs.
            Admission::Admitted(_) => Some(match identity {
                None => session.tick(text, Source::Grimm)?,
                Some(identity) => session.tick_frame(&SenseFrame {
                    text: text.to_string(),
                    identity,
                    conversation: session.conversation(),
                    source: Source::Grimm,
                })?,
            }),
            _ => None,
        };
        Ok((report, note))
    }

    #[cfg(not(feature = "supporter"))]
    pub fn attempt(
        &mut self,
        _session: &mut Session,
        _text: &str,
        _identity: Option<si_core::world::Identity>,
    ) -> Result<(Option<TickReport>, SpeakNote)> {
        Ok((
            None,
            SpeakNote {
                reached_loop: false,
                admitted: false,
                dropped: true,
                reason: "seam_absent",
                layer: 0,
                n_untaught_tokens: 0,
                novel_claims: 0,
                ref_row_id: None,
                untaught: Vec::new(),
            },
        ))
    }

    /// The section's admitted emissions, counted at the seam itself so the probe
    /// does not re-read the log it just wrote.
    #[cfg(feature = "supporter")]
    pub fn admissions(&self) -> usize {
        self.seam.admissions()
    }

    #[cfg(not(feature = "supporter"))]
    pub fn admissions(&self) -> usize {
        0
    }

    /// Write the section's gate log where the run directory can keep it. The log
    /// is an observation: nothing in the build reads it back (05 §17).
    #[cfg(feature = "supporter")]
    pub fn flush(&self) -> Result<()> {
        self.seam.flush()
    }

    #[cfg(not(feature = "supporter"))]
    pub fn flush(&self) -> Result<()> {
        Ok(())
    }
}

/// 04 §23: the one route a sense step takes through the loop, shared by `execute`
/// and every replay path. An owner line always reaches `tick` (L5's asymmetry law);
/// a `source: grimm` line reaches it only if the seam's gate admits it, so a
/// replayed transcript refuses the same lines in the same order as the run that
/// wrote it. A refused line does not advance the clock, so it reports the clock as
/// it stood.
fn sense_step(
    session: &mut Session,
    mouth: &mut Mouth,
    source: Source,
    text: &str,
    identity: Option<si_core::world::Identity>,
) -> Result<i32> {
    Ok(sense_step_noted(session, mouth, source, text, identity)?.0)
}

/// The same step as `sense_step`, plus the seam's verdict for a line that did not
/// come from the owner. **G20(a.2)** compares the launched process's voice rows
/// with the grading process's, and the run keeps a voice only when its step reached
/// `tick` (`runner.rs`'s `route.is_some()` guard) — so `drive` has to know the
/// verdict too, or it would read the previous tick's RAM as this step's voice.
fn sense_step_noted(
    session: &mut Session,
    mouth: &mut Mouth,
    source: Source,
    text: &str,
    identity: Option<si_core::world::Identity>,
) -> Result<(i32, Option<SpeakNote>)> {
    if source == Source::Owner {
        return Ok((session.tick(text, source)?.t, None));
    }
    let (report, note) = mouth.attempt(session, text, identity)?;
    Ok((report.map(|report| report.t).unwrap_or(session.now()), Some(note)))
}

#[derive(Clone, Debug)]
pub struct StepRecord {
    pub index: usize,
    pub t: i32,
    pub source: Option<Source>,
    pub text: String,
    pub idle: bool,
    pub output: Option<Output>,
    pub rows_written: u32,
    pub a_before: u8,
    pub a_after: u8,
    pub event_start: usize,
    pub event_end: usize,
    pub shelves_after: BTreeMap<u16, usize>,
    pub rows_written_total: u32,
    pub before: Boundary,
    pub after: Boundary,
    /// 05 §12/§13: an idle step's own two products, kept as data so `G7.a.1`,
    /// `G7.b.1` and `G8.e.1` read the pass rather than a rendered sentence.
    pub idle_revisited: Vec<(u32, u32)>,
    pub idle_derived: Vec<(u32, u32)>,
    /// The re-parse eligibility set read immediately before and immediately
    /// after the step (`G7.b.1`'s idempotence is this pair, not a promise).
    pub eligible_before: usize,
    pub eligible_after: usize,
    /// 04 §23: set on a `source: grimm` step — what the speak gate did with it,
    /// and whether the line ever reached `tick`. `None` on every other step.
    pub speak: Option<SpeakNote>,
    /// 11 §B18: set on a world declaration, with the same three words the scenario
    /// file speaks (`grant 7 teach`). It is not sense, so it is absent from the
    /// inbox and present in the transcript, which is what lets a cross-process
    /// replay re-declare what the run declared.
    pub world: Option<String>,
    /// 01 §9 / 04 §33: the id a sense line arrived with and the class derived from
    /// it at that tick. `None` on an idle step, which carries no speaker.
    pub identity: Option<Identity>,
    pub class: Option<&'static str>,
    /// 05 §11 `G6.b`: the route this step's read reported, as the walk counted
    /// it — OWNED claim edges and BASE (sealed) edges. `None` is a step that
    /// never reached a read. Session RAM at the tick, recorded here as data; it
    /// is hashed nowhere.
    pub route: Option<si_read::RouteCounts>,
    /// 13 §6.4: what step 9a did at this tick, or `None` for a tick that never
    /// entered the arm. `G20`'s four artifacts are written from these; at V0 the
    /// list is empty, and that emptiness is `G20(j)`'s evidence. Session RAM,
    /// hashed nowhere — the same standing `route` above has.
    pub tongue: Option<si_session::TongueTick>,
}

pub struct SectionRun {
    pub session: Session,
    pub hash: String,
    pub steps: Vec<StepRecord>,
    pub idle: Vec<IdleReport>,
    /// 02 §8's gate log for this section, one machine-readable frame per emission
    /// attempt, and OD-40(iii)'s read log. Both are observations the run exports;
    /// neither is ever an input (05 §17, R-33).
    pub gate_log: Vec<String>,
    pub gate_reads: Vec<String>,
    /// The seam state this section launched at, as 05 §2 names it.
    pub seam_state: &'static str,
}

/// 05 §15 `G10(b)`: the durable belief snapshot an idle ignition must not move.
/// The three digests cover every byte a belief write touches — the row envelope
/// (confidence, supersession), the edge arena (each edge's `bake`) and the stamp
/// arena. The two counts prove the comparison ran against a real ignition rather
/// than an empty run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DurableState {
    pub rows_digest: String,
    pub edges_digest: String,
    pub stamps_digest: String,
    pub row_count: usize,
    /// The stamp arena's shape: record count and byte length. 05 §15(b).2's
    /// may-differ list governs the stamp *values* (an idle cascade conducts drive
    /// and moves `a`, 04 §19 S.7), so this is what stays equal; the digest is
    /// reported beside it, and a difference there is a finding, not a failure.
    pub stamp_count: usize,
    pub stamps_len: usize,
    pub monologue_lines: usize,
    pub ignitions: usize,
    /// 01 §10: the nine-class state hash of each section, in scenario order.
    /// OD-91 makes the V0-vs-V3 comparison of this value `G10.e.5`'s pass
    /// condition, so the whole state is read, not only the belief arenas.
    pub section_hashes: Vec<String>,
    /// 04 §20.4a: every point's `bake` above zero, sorted, over all sections.
    /// The pair's identity is only worth asserting where a bake exists for the
    /// zeroed weight to be measured against.
    pub bake_values: Vec<u16>,
}

pub struct ContainerCheck {
    pub live_hash: String,
    pub reopened_hash: String,
    pub same_seed_ok: bool,
    pub other_seed_refused: Option<String>,
}

pub struct Run {
    pub request: RunRequest,
    pub dir: PathBuf,
    pub run_id: String,
    pub started: String,
    pub sections: Vec<SectionRun>,
    pub container: Option<ContainerCheck>,
}

/// 05 §15 `G10(b)`/`G10(e).5`: run one transcript through the loop at one launch
/// variant and read what it leaves behind. `Run::durable_state` is this over the
/// run's own scenario; a leg that names its own transcripts (OD-91) calls it
/// directly, so the same loop code — never a harness branch of it — produces both
/// sides of the comparison.
pub fn durable_state_of(
    scenario: &Scenario,
    seed: u64,
    constants: &BTreeMap<String, i64>,
    variant: &str,
) -> Result<DurableState> {
    let mut rows_digest = Md5::new();
    let mut edges_digest = Md5::new();
    let mut stamps_digest = Md5::new();
    let mut row_count = 0usize;
    let mut stamp_count = 0usize;
    let mut stamps_len = 0usize;
    let mut monologue_lines = 0usize;
    let mut ignitions = 0usize;
    let mut section_hashes: Vec<String> = Vec::new();
    let mut bake_values: Vec<u16> = Vec::new();
    for section in &scenario.sections {
        let mut session = Session::new(seed)?;
        install_launch_state(&mut session, constants)?;
        install_world(&mut session, scenario);
        session.set_launch_variant(variant);
        let mut mouth = Mouth::launch(variant, None, None);
        for step in section {
            match step {
                Step::Sense { source, text, identity, .. } => {
                    sense_step(&mut session, &mut mouth, *source, text, *identity)?;
                }
                Step::World(op) => {
                    world_step(&mut session, op)?;
                }
                Step::Idle => {
                    if session.idle_tick()?.ignited.is_some() {
                        ignitions += 1;
                    }
                }
            }
        }
        monologue_lines += session.monologue().len();
        // No freeze here, deliberately. `G10(b)` asks whether an **idle ignition**
        // moved a belief; the checkpoint sleep would dim both sides of the pair and
        // turn the comparison into a statement about P3 instead of about I9. The
        // run's own artifacts do freeze (`execute`), and `G13(a)` grades those.
        section_hashes.push(session.state_hash()?.hex());
        // 04 §20.4a's stored `bake`, read through the same home the sampler reads
        // it from. A pair of identical state hashes means nothing if no edge
        // carried a bake for the zeroed weight to ignore.
        let graph = si_walk::Graph::build(session.store(), session.now())?;
        bake_values.extend(
            graph
                .bake_touch_max()
                .values()
                .filter(|value| **value > 0)
                .copied(),
        );
        let store = session.store();
        let mut records: Vec<_> = store.rows()?.into_iter().map(|(_, record)| record).collect();
        records.sort_by_key(|record| record.envelope.row_id);
        for record in &records {
            rows_digest.update(record.encode());
            row_count += 1;
        }
        edges_digest.update(store.edges_arena().bytes());
        stamps_digest.update(store.stamps_arena().bytes());
        stamp_count += store.stamps_arena().count();
        stamps_len += store.stamps_arena().bytes().len();
    }
    bake_values.sort_unstable();
    Ok(DurableState {
        rows_digest: scenario::hex(rows_digest.finalize().as_slice()),
        edges_digest: scenario::hex(edges_digest.finalize().as_slice()),
        stamps_digest: scenario::hex(stamps_digest.finalize().as_slice()),
        row_count,
        stamp_count,
        stamps_len,
        monologue_lines,
        ignitions,
        section_hashes,
        bake_values,
    })
}

/// One section read at the end of a probe run: the **frozen** state hash (the same
/// freeze `execute` does, so a probe hash and the run's artifact hash are one
/// number), the seam's two observables, and the row envelopes + bakes a provenance
/// or a mirror leg needs to compare field by field.
#[derive(Clone, Debug)]
pub struct SectionProbe {
    pub hash: String,
    pub admissions: usize,
    pub reads: usize,
    pub rows: Vec<RowProbe>,
    pub bakes: Vec<u16>,
}

/// The four fields a mirror must not move, plus the two that identify the row and
/// its author. `05 §17 (e)` grades exactly these.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RowProbe {
    pub row_id: u32,
    pub shelf: u16,
    pub prov_speaker: u8,
    pub repeats: u32,
    pub last_confirmed_at: i32,
    pub confidence: u16,
    pub live: bool,
}

#[derive(Clone, Debug)]
pub struct VariantProbe {
    pub variant: String,
    pub sections: Vec<SectionProbe>,
    pub emissions: usize,
}

/// 05 §17 `G12(a)`'s pair, and `(e.3)`'s control: play one scenario at one launch
/// variant and read what each section leaves behind. Like `durable_state_of`, this
/// is a second use of the **same** loop code — `sense_step`, `Mouth::read_all`, the
/// same freeze — never a harness branch of it, which is what makes the comparison
/// mean something.
pub fn variant_probe(
    scenario: &Scenario,
    seed: u64,
    constants: &BTreeMap<String, i64>,
    variant: &str,
) -> Result<VariantProbe> {
    let mut sections = Vec::new();
    let mut emissions = 0usize;
    for section in &scenario.sections {
        let mut session = Session::new(seed)?;
        install_launch_state(&mut session, constants)?;
        install_world(&mut session, scenario);
        session.set_launch_variant(variant);
        let mut mouth = Mouth::launch(variant, None, None);
        for step in section {
            match step {
                Step::Sense { source, text, identity, .. } => {
                    if *source != Source::Owner {
                        emissions += 1;
                    }
                    mouth.read_all(&session)?;
                    sense_step(&mut session, &mut mouth, *source, text, *identity)?;
                }
                Step::World(op) => {
                    world_step(&mut session, op)?;
                }
                Step::Idle => {
                    session.idle_tick()?;
                }
            }
        }
        session.freeze()?;
        let graph = si_walk::Graph::build(session.store(), session.now())?;
        let mut bakes: Vec<u16> = graph.bake_touch_max().values().filter(|v| **v > 0).copied().collect();
        bakes.sort_unstable();
        sections.push(SectionProbe {
            hash: session.state_hash()?.hex(),
            admissions: mouth.admissions(),
            reads: mouth.reads.len(),
            rows: row_probe(&session)?,
            bakes,
        });
    }
    Ok(VariantProbe { variant: variant.to_string(), sections, emissions })
}

/// Every row of a session, in row-id order, with the fields a provenance leg reads.
pub fn row_probe(session: &Session) -> Result<Vec<RowProbe>> {
    let mut out: Vec<RowProbe> = session
        .store()
        .rows()?
        .into_iter()
        .map(|(_, record)| {
            let e = &record.envelope;
            RowProbe {
                row_id: e.row_id,
                shelf: e.shelf_x,
                prov_speaker: e.prov_speaker,
                repeats: e.repeats,
                last_confirmed_at: e.last_confirmed_at,
                confidence: e.confidence,
                live: e.is_live(),
            }
        })
        .collect();
    out.sort();
    Ok(out)
}

/// 05 §17 `G12(e.3)`'s "a control run where he stayed quiet": the same transcript
/// with every non-owner line removed, so the owner's own words arrive in the same
/// order and only the emissions are absent.
pub fn without_emissions(scenario: &Scenario) -> Scenario {
    Scenario {
        path: scenario.path.clone(),
        digest: scenario.digest.clone(),
        seed: scenario.seed,
        sections: scenario
            .sections
            .iter()
            .map(|section| {
                section
                    .iter()
                    .filter(|step| matches!(step, Step::Idle)
                        || matches!(step, Step::World(_))
                        || matches!(step, Step::Sense { source, .. } if *source == Source::Owner))
                    .cloned()
                    .collect()
            })
            .collect(),
        // The control run registers the same two ids the run did, and inherits its
        // declarations: an owner's grant is his own act, not an emission.
        identities: scenario.identities.clone(),
    }
}

impl Run {
    /// Execute the scenario. Sections are separate fresh sessions sharing the
    /// run's seed; a `--- cold` block is 05 §6's separate cold run inside the
    /// gate.
    pub fn execute(request: RunRequest, ws: &Workspace) -> Result<Self> {
        if let Some(stated) = request.scenario.seed {
            if stated != request.seed {
                bail!(
                    "scenario {} names seed 0x{stated:016x}; the launch names 0x{:016x} (G14.j.1: a scenario states its individual)",
                    request.scenario.path,
                    request.seed
                );
            }
        }
        let started = utc_stamp();
        let dir = match &request.run_dir {
            Some(path) => path.clone(),
            None => unique_dir(&ws.path(&format!(
                "gates/{}_{}_{}",
                request.gate, request.config, started
            ))),
        };
        fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        let run_id = dir
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "run".to_string());

        let mut sections = Vec::new();
        for section in &request.scenario.sections {
            let mut session = wake_session(&request, request.seed)?;
            install_launch_state(&mut session, &request.constants)?;
            install_world(&mut session, &request.scenario);
            session.set_launch_variant(&request.variant);
            // 05 §18(g): the harness may schedule an owner frame at one named pass
            // boundary. It is a launch state of this run, not a verb, and `None` in
            // every run that does not name it.
            session.set_sleep_interrupt(request.sleep_interrupt);
            // 04 §23: the seam is launched per section, at the run's variant. V4
            // gets no reader at all, V5 gets readers and no mouth, and every other
            // variant gets the shipped gate. `Session` itself has no branch for
            // any of this (`check_no_stage_on_supporter`).
            let mut mouth = Mouth::launch(&request.variant, None, Some(&dir));
            let mut steps = Vec::new();
            let mut idles = Vec::new();
            let mut total_rows = 0u32;
            for (offset, step) in section.iter().enumerate() {
                let event_start = session.events().len();
                let world_decl = match step {
                    Step::World(op) => Some(world_literal(op)),
                    _ => None,
                };
                // 01 §9: the id the line arrived with, as observed data. `None` is
                // the literal's own registered id, which is what keeps a pre-B18
                // transcript byte-for-byte what it was.
                let line_identity = match step {
                    Step::Sense { identity, .. } => *identity,
                    _ => None,
                };
                // The id the class is derived from: a frame's own when it names
                // one, otherwise the id registered for the literal it arrived on.
                let line_id = line_identity.or(match step {
                    Step::Sense { source, .. } => Some(match source {
                        Source::Owner => session.world().owner_identity,
                        Source::Grimm => session.world().grimm_identity,
                    }),
                    _ => None,
                });
                let before = boundary(&session)?;
                let eligible_before = si_session::eligible_derivations(&session)?.len();
                mouth.read_all(&session)?;
                let (t, source, text, idle, output, rows_written, a_before, a_after, revisited, derived, speak) = match step {
                    Step::Sense { source, text, identity, .. } if *source == Source::Grimm => {
                        let (report, note) = mouth.attempt(&mut session, text, *identity)?;
                        let t = report.as_ref().map(|report| report.t).unwrap_or(session.now());
                        let a = session.a();
                        let (output, rows_written, a_before, a_after) = match &report {
                            Some(report) => {
                                total_rows += report.rows_written;
                                (
                                    Some(report.output.clone()),
                                    report.rows_written,
                                    report.a_before,
                                    report.a_after,
                                )
                            }
                            // 04 §23.3: a refused line pays no sense event. The
                            // clock did not advance, `a` did not move, and the
                            // boundary pair around this step is equal by
                            // construction — that equality is `G12(b).2`.
                            None => (None, 0, a, a),
                        };
                        (
                            t,
                            Some(*source),
                            text.clone(),
                            false,
                            output,
                            rows_written,
                            a_before,
                            a_after,
                            Vec::new(),
                            Vec::new(),
                            Some(note),
                        )
                    }
                    Step::Sense { source, text, identity: _, .. } => {
                        // L5's asymmetry law: the owner's line is never filtered —
                        // a token that is not yet in the lexicon is the only way it
                        // ever gets there (docs/21 §21.3).
                        let report = session.tick(text, *source)?;
                        total_rows += report.rows_written;
                        (
                            report.t,
                            Some(*source),
                            text.clone(),
                            false,
                            Some(report.output),
                            report.rows_written,
                            report.a_before,
                            report.a_after,
                            Vec::new(),
                            Vec::new(),
                            None,
                        )
                    }
                    Step::World(op) => {
                        // A declaration is not a sense line: no output, no rows,
                        // and the same `a` before and after, which is what keeps
                        // the boundary pair around it equal (`G12(b).2`'s shape).
                        let (t, _) = world_step(&mut session, op)?;
                        let a = session.a();
                        (t, None, String::new(), false, None, 0, a, a, Vec::new(), Vec::new(), None)
                    }
                    Step::Idle => {
                        let report = session.idle_tick()?;
                        let revisited = report.revisited.clone();
                        let derived = report.derived.clone();
                        let output = Output::Inert {
                            reason: format!(
                                "idle: revisited {}, derived {}",
                                revisited.len(),
                                derived.len()
                            ),
                        };
                        idles.push(report.clone());
                        (report.t, None, String::new(), true, Some(output), 0, session.a(), session.a(), revisited, derived, None)
                    }
                };
                let after = boundary(&session)?;
                // 04 §33 AE.1 / law L-G: derived here, at the tick the step ran at,
                // from the id alone. Nothing on the session caches a class, so a
                // revoked grant has no standing on the very next line
                // (`check_speaker_class_derived`).
                let line_class = line_id.map(|id| session.class_of(id, session.now()).literal());
                let shelves_after = census(&session)?;
                // A declaration, and a line the seam refused, never reach a read,
                // so the session still carries the previous step's route. Recording
                // that as this step's observation would be a lie, so both are `None`.
                let route = if world_decl.is_some()
                    || matches!(&speak, Some(note) if !note.reached_loop)
                {
                    None
                } else {
                    si_session::last_route(&session)
                };
                // The voice is guarded by the same fact the route is: a line that
                // never reached `tick` cannot have entered step 9a, and reporting
                // the previous step's voice as this step's observation would be the
                // same lie `route` above refuses to tell.
                let tongue =
                    if route.is_some() { si_session::last_tongue(&session) } else { None };
                steps.push(StepRecord {
                    index: offset + 1,
                    t,
                    source,
                    text,
                    idle,
                    output,
                    rows_written,
                    a_before,
                    a_after,
                    event_start,
                    event_end: session.events().len(),
                    shelves_after,
                    rows_written_total: total_rows,
                    before,
                    after,
                    idle_revisited: revisited,
                    idle_derived: derived,
                    eligible_before: eligible_before as usize,
                    eligible_after: si_session::eligible_derivations(&session)?.len(),
                    speak,
                    world: world_decl,
                    identity: line_id,
                    class: line_class,
                    route,
                    tongue,
                });
            }
            // 04 §23 W.3's one log line per attempt, written where the run keeps it.
            // Nothing in the build reads it back (05 §17, R-33).
            let gate_log = mouth.frames.clone();
            let gate_reads = mouth.reads.clone();
            let seam_state = mouth.state;
            // 10 §7's trigger 2: a section ends where a checkpoint is written, so
            // the sleep runs **before** the hash that this section's artifacts
            // publish. Idempotent, and `Session::open` of the file written from it
            // therefore re-derives the same digest (**G13(h)**, **OD-81**).
            session.freeze()?;
            let hash = session.state_hash()?.hex();
            sections.push(SectionRun {
                session,
                hash,
                steps,
                idle: idles,
                gate_log,
                gate_reads,
                seam_state,
            });
        }

        let mut run = Self {
            request,
            dir,
            run_id,
            started,
            sections,
            container: None,
        };
        run.write_artifacts(ws)?;
        Ok(run)
    }

    pub fn read(ws: &Workspace, dir: &Path) -> Result<Self> {
        let report = super::gates::read_gate_report(dir)?;
        let scenario_path = report
            .get("build_identity")
            .and_then(|identity| identity.get("scenario"))
            .and_then(|scenario| scenario.get("path"))
            .and_then(|path| path.as_str())
            .context("gate_report.json has no build_identity.scenario.path")?;
        let seed = report
            .get("build_identity")
            .and_then(|identity| identity.get("birth_seed_hex"))
            .and_then(|seed| seed.as_str())
            .context("gate_report.json has no build_identity.birth_seed_hex")?;
        let scenario = Scenario::load(ws, scenario_path)?;
        // The launch state a re-execution needs, read from the report the run
        // itself wrote: the boundary a scheduled owner frame sat at, the swept
        // constant point, the variant clause and — from B10 — the config and the
        // pinned corpus. All properties of the launch, not of the stream, which
        // still comes from `transcript.txt` (R-33). A directory re-executed at the
        // registry placeholder would be a different run wearing the same name.
        let launch = Launch::recorded(&report, ws);
        let request = RunRequest {
            gate: report
                .get("gate")
                .and_then(|gate| gate.as_str())
                .unwrap_or("?")
                .to_string(),
            variant: launch.variant,
            config: launch.config,
            seed: scenario::parse_seed(seed)?,
            scenario,
            constants: launch.constants,
            sleep_interrupt: launch.sleep_interrupt,
            genome: launch.genome,
            genome_tag: launch.genome_tag,
            run_dir: Some(dir.to_path_buf()),
        };
        Self::execute(request, ws)
    }

    pub fn main(&self) -> &SectionRun {
        &self.sections[0]
    }

    /// 05 §1 `replay` across processes: the run's `transcript.txt` is the only
    /// input — the recorded sense stream, replayed line-for-line into fresh
    /// sessions at the launch seed. The run's JSON is never read back (R-33),
    /// and the loop assigns `t` itself, so the transcript's column is not an
    /// input either. The launch is the run's recorded state, so an interrupted
    /// night replays as the same partial night (**G13(g).3**) and a CONFIG ENGINE
    /// run replays into the genome-owned handle space it was born into (00 §6).
    pub fn replay_transcript(dir: &Path, seed: u64, launch: &Launch) -> Result<Vec<String>> {
        let path = dir.join("transcript.txt");
        let text = fs::read_to_string(&path)
            .with_context(|| format!("read {} (the replay's only input)", path.display()))?;
        let mut sections: Vec<Vec<Step>> = vec![Vec::new()];
        let mut registered = Identities::default();
        for (number, raw) in text.lines().enumerate() {
            let line = raw.trim_end_matches('\r');
            if line.trim() == "--- cold" {
                sections.push(Vec::new());
                continue;
            }
            if line.trim().is_empty() {
                continue;
            }
            // 11 §B18: the registration header, present only on a run that is not
            // at the birth pair. Same two words the scenario file speaks.
            if let Some(rest) = line.trim().strip_prefix("#identity ") {
                let (identity, source) = scenario::parse_identity(
                    &format!("{}:{}", path.display(), number + 1),
                    rest,
                )?;
                match source {
                    Source::Owner => registered.owner = identity,
                    Source::Grimm => registered.grimm = identity,
                }
                continue;
            }
            let mut parts = line.splitn(3, '\t');
            let _t = parts.next().unwrap_or_default();
            let source = parts.next().ok_or_else(|| {
                anyhow::anyhow!("{}:{}: a transcript line is `t<TAB>source<TAB>text`", path.display(), number + 1)
            })?;
            let body = parts.next().unwrap_or_default();
            if source == "idle" {
                sections.last_mut().expect("a section exists").push(Step::Idle);
                continue;
            }
            // A declaration re-declares, through the scenario's own parser: one
            // grammar, never a second dialect for the replay.
            if source == "world" {
                let parsed = Scenario::parse("<transcript line>", body, None)
                    .with_context(|| format!("{}:{}", path.display(), number + 1))?;
                let step = parsed
                    .sections
                    .first()
                    .and_then(|section| section.first())
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("{}:{}: no declaration", path.display(), number + 1))?;
                sections.last_mut().expect("a section exists").push(step);
                continue;
            }
            // `literal#identity` (01 §9): the id the line arrived with.
            let (literal, identity) = match source.split_once('#') {
                Some((literal, id)) => (
                    literal,
                    Some(id.parse::<Identity>().with_context(|| {
                        format!("{}:{}: `{id}` is not a u64 identity", path.display(), number + 1)
                    })?),
                ),
                None => (source, None),
            };
            let source = Source::from_literal(literal).ok_or_else(|| {
                anyhow::anyhow!(
                    "{}:{}: source `{source}` is not a literal of 01 §12",
                    path.display(),
                    number + 1
                )
            })?;
            sections.last_mut().expect("a section exists").push(Step::Sense {
                source,
                text: body.to_string(),
                identity,
                conversation: 0,
            });
        }
        let mut hashes = Vec::new();
        for section in sections.into_iter().filter(|section| !section.is_empty()) {
            let mut session = launch.wake(seed)?;
            install_launch_state(&mut session, &launch.constants)?;
            install_registered(&mut session, registered);
            // OD-93: the determinism clause is per-variant, so a replay re-executes
            // at the variant the run was made at, not at the registry's own values.
            session.set_launch_variant(&launch.variant);
            session.set_sleep_interrupt(launch.sleep_interrupt);
            let mut mouth = Mouth::launch(&launch.variant, None, None);
            for step in &section {
                match step {
                    Step::Sense { source, text, identity, .. } => {
                        sense_step(&mut session, &mut mouth, *source, text, *identity)?;
                    }
                    Step::World(op) => {
                        world_step(&mut session, op)?;
                    }
                    Step::Idle => {
                        session.idle_tick()?;
                    }
                }
            }
            // The replay freezes for the same reason `execute` does: the hash it
            // must reproduce was taken after step 12b's checkpoint sleep.
            session.freeze()?;
            hashes.push(session.state_hash()?.hex());
        }
        Ok(hashes)
    }

    /// 05 §1 `replay`: re-execute the recorded sense stream into fresh sessions
    /// at the same birth seed and return the hashes. The input is the steps, in
    /// the same order, through the same `tick`/`idle_tick` calls; the original
    /// run's JSON is never read back (R-33). `G7.b.2` is this pair inside one
    /// process; the CLI's V6 leg is the same function across processes.
    pub fn replay(&self) -> Result<Vec<String>> {
        let mut hashes = Vec::new();
        for section in &self.request.scenario.sections {
            let mut session = wake_session(&self.request, self.request.seed)?;
            install_launch_state(&mut session, &self.request.constants)?;
            install_world(&mut session, &self.request.scenario);
            session.set_launch_variant(&self.request.variant);
            session.set_sleep_interrupt(self.request.sleep_interrupt);
            let mut mouth = Mouth::launch(&self.request.variant, None, None);
            for step in section {
                match step {
                    Step::Sense { source, text, identity, .. } => {
                        sense_step(&mut session, &mut mouth, *source, text, *identity)?;
                    }
                    Step::World(op) => {
                        world_step(&mut session, op)?;
                    }
                    Step::Idle => {
                        session.idle_tick()?;
                    }
                }
            }
            // The replay freezes for the same reason `execute` does: the hash it
            // must reproduce was taken after step 12b's checkpoint sleep.
            session.freeze()?;
            hashes.push(session.state_hash()?.hex());
        }
        Ok(hashes)
    }

    /// The replay's per-step event frames, one slice per scenario step. 05 §10
    /// `e.1` compares the **activation trace** as shipped output, and comparing
    /// only the settle set is not enough: a step's trace is every frame it wrote,
    /// in order, so each slice is the exact counterpart of `Run::events`.
    pub fn replay_traces(&self) -> Result<Vec<Vec<Vec<Event>>>> {
        let mut sections = Vec::new();
        for section in &self.request.scenario.sections {
            let mut session = wake_session(&self.request, self.request.seed)?;
            install_launch_state(&mut session, &self.request.constants)?;
            install_world(&mut session, &self.request.scenario);
            session.set_launch_variant(&self.request.variant);
            session.set_sleep_interrupt(self.request.sleep_interrupt);
            let mut mouth = Mouth::launch(&self.request.variant, None, None);
            let mut steps = Vec::new();
            for step in section {
                let event_start = session.events().len();
                match step {
                    Step::Sense { source, text, identity, .. } => {
                        sense_step(&mut session, &mut mouth, *source, text, *identity)?;
                    }
                    Step::World(op) => {
                        world_step(&mut session, op)?;
                    }
                    Step::Idle => {
                        session.idle_tick()?;
                    }
                }
                steps.push(session.events().events()[event_start..].to_vec());
            }
            session.freeze()?;
            sections.push(steps);
        }
        Ok(sections)
    }

    /// The replay's idle records and monologue, one pair per section. 05 §10
    /// `f.1` compares the idle cascades and the lines they rendered — including
    /// `IdleReport::ignited`'s hyperpoint — against the run's own. The monologue
    /// is a separate lane from the event frames, so it is compared as its own
    /// typed sequence rather than through a rendered file.
    pub fn replay_idle(&self) -> Result<Vec<(Vec<IdleReport>, Vec<MonologueLine>)>> {
        let mut sections = Vec::new();
        for section in &self.request.scenario.sections {
            let mut session = wake_session(&self.request, self.request.seed)?;
            install_launch_state(&mut session, &self.request.constants)?;
            install_world(&mut session, &self.request.scenario);
            session.set_launch_variant(&self.request.variant);
            session.set_sleep_interrupt(self.request.sleep_interrupt);
            let mut mouth = Mouth::launch(&self.request.variant, None, None);
            let mut idles = Vec::new();
            for step in section {
                match step {
                    Step::Sense { source, text, identity, .. } => {
                        sense_step(&mut session, &mut mouth, *source, text, *identity)?;
                    }
                    Step::World(op) => {
                        world_step(&mut session, op)?;
                    }
                    Step::Idle => {
                        idles.push(session.idle_tick()?);
                    }
                }
            }
            session.freeze()?;
            sections.push((idles, session.monologue().to_vec()));
        }
        Ok(sections)
    }

    pub fn section(&self, index: usize) -> Option<&SectionRun> {
        self.sections.get(index)
    }

    /// The full written event log of one section, for a leg that classifies over
    /// the persisted rows rather than a single step (05 §14 G9).
    pub fn eventlog(&self, section: usize) -> Vec<Event> {
        self.sections
            .get(section)
            .map(|run| run.session.events().events().to_vec())
            .unwrap_or_default()
    }

    /// 05 §14 `G9(b)`: re-execute the recorded sense stream and return each
    /// section's full event log. `classify` over these rows must equal `classify`
    /// over the originals — the acceptance condition is byte-for-byte reproducibility.
    pub fn replay_eventlogs(&self) -> Result<Vec<Vec<Event>>> {
        let mut logs = Vec::new();
        for section in &self.request.scenario.sections {
            let mut session = wake_session(&self.request, self.request.seed)?;
            install_launch_state(&mut session, &self.request.constants)?;
            install_world(&mut session, &self.request.scenario);
            session.set_launch_variant(&self.request.variant);
            session.set_sleep_interrupt(self.request.sleep_interrupt);
            let mut mouth = Mouth::launch(&self.request.variant, None, None);
            for step in section {
                match step {
                    Step::Sense { source, text, identity, .. } => {
                        sense_step(&mut session, &mut mouth, *source, text, *identity)?;
                    }
                    Step::World(op) => {
                        world_step(&mut session, op)?;
                    }
                    Step::Idle => {
                        session.idle_tick()?;
                    }
                }
            }
            // The freeze belongs here too: the run's own log ends with the kind-19
            // lines step 12b wrote at the checkpoint, so a replay that skipped it
            // would differ from the original by records rather than by bytes.
            session.freeze()?;
            logs.push(session.events().events().to_vec());
        }
        Ok(logs)
    }

    /// 05 §15 `G10(b)`: the durable belief state, as the digests an idle ignition
    /// must not move. It reads the three arenas a belief row's confidence and
    /// supersession (the row envelope), an edge's `bake` (the edge arena) and the
    /// stamp list live in, plus the monologue and ignition counts that keep the
    /// V0-vs-V2 comparison from being vacuous. `durable_state("V0")` ignites;
    /// `durable_state("V2")` does not — the loop code is the same, only the launch
    /// state differs. An empty-agenda run leaves every digest equal.
    pub fn durable_state(&self, variant: &str) -> Result<DurableState> {
        durable_state_of(
            &self.request.scenario,
            self.request.seed,
            &self.request.constants,
            variant,
        )
    }

    /// 05 §15: the canonical ignition-on (`V0`) sessions, replayed from the same
    /// steps at the launch seed. The behavior legs — rate, precedence, monologue
    /// containment — are graded against this reference so a `V2`/`V3` launch still
    /// evaluates what ignition *does*, not only what disabling it removes. The
    /// launched variant's own run stays the artifact; this is in-process evidence.
    pub fn reference(&self) -> Result<Vec<Session>> {
        let mut sessions = Vec::new();
        for section in &self.request.scenario.sections {
            let mut session = wake_session(&self.request, self.request.seed)?;
            install_launch_state(&mut session, &self.request.constants)?;
            install_world(&mut session, &self.request.scenario);
            session.set_launch_variant("V0");
            session.set_sleep_interrupt(self.request.sleep_interrupt);
            let mut mouth = Mouth::launch("V0", None, None);
            for step in section {
                match step {
                    Step::Sense { source, text, identity, .. } => {
                        sense_step(&mut session, &mut mouth, *source, text, *identity)?;
                    }
                    Step::World(op) => {
                        world_step(&mut session, op)?;
                    }
                    Step::Idle => {
                        session.idle_tick()?;
                    }
                }
            }
            session.freeze()?;
            sessions.push(session);
        }
        Ok(sessions)
    }

    pub fn event(&self, section: usize, step: &StepRecord, index: usize) -> Option<&Event> {
        self.sections
            .get(section)?
            .session
            .events()
            .events()
            .get(step.event_start + index)
    }

    pub fn events(&self, section: usize, step: &StepRecord) -> &[Event] {
        let events = self.sections[section].session.events().events();
        &events[step.event_start.min(events.len())..step.event_end.min(events.len())]
    }

    pub fn step(&self, section: usize, index: usize) -> Option<&StepRecord> {
        self.sections.get(section)?.steps.get(index.checked_sub(1)?)
    }

    fn write_artifacts(&mut self, ws: &Workspace) -> Result<()> {
        let dir = self.dir.clone();
        let mut transcript = String::new();
        let mut inbox: Vec<Event> = Vec::new();
        let mut seq = 0u64;
        // 11 §B18: the two registered ids, recorded only when the run is not at
        // the birth pair. A pre-B18 scenario registers neither, so its transcript
        // starts at its first step exactly as it did before the world existed.
        if let Some(section) = self.sections.first() {
            let world = section.session.world();
            if (world.owner_identity, world.grimm_identity) != (si_core::world::IDENTITY_OWNER, 1) {
                transcript.push_str(&format!(
                    "#identity {} owner\n#identity {} grimm\n",
                    world.owner_identity, world.grimm_identity
                ));
            }
        }
        for (section_index, section) in self.sections.iter().enumerate() {
            if section_index > 0 {
                transcript.push_str("--- cold\n");
            }
            for step in &section.steps {
                // A world declaration is not sense: it has no body to transport and
                // no speaker, but a replaying process needs the words it spoke.
                if let Some(decl) = &step.world {
                    transcript.push_str(&format!("{}\tworld\t{}\n", step.t, decl));
                    continue;
                }
                // 04 §23 W.3: the transcript and the inbox are the *sense stream*,
                // so an emission the gate refused is absent from both — it never
                // became a sense event, and a replay of this file therefore
                // re-runs exactly the lines the loop actually saw.
                if step.speak.as_ref().is_some_and(|note| !note.reached_loop) {
                    continue;
                }
                let source = match (step.idle, step.source) {
                    (true, _) => "idle".to_string(),
                    (false, Some(source)) => {
                        // 01 §9: the id travels on the source column as
                        // `literal#identity`, and only when it is not the id that
                        // literal is born with. The text column never moves.
                        let born = match source {
                            Source::Owner => si_core::world::IDENTITY_OWNER,
                            Source::Grimm => 1,
                        };
                        match step.identity {
                            Some(id) if id != born => format!("{}#{}", source.literal(), id),
                            _ => source.literal().to_string(),
                        }
                    }
                    (false, None) => bail!("a sense step without a source"),
                };
                transcript.push_str(&format!("{}\t{}\t{}\n", step.t, source, step.text));
                if let Some(source) = step.source {
                    if !step.idle {
                        let identity = step.identity.unwrap_or(match source {
                            Source::Owner => si_core::world::IDENTITY_OWNER,
                            Source::Grimm => 1,
                        });
                        inbox.push(Event {
                            seq,
                            t: step.t,
                            source: match source {
                                Source::Owner => 0,
                                Source::Grimm => 1,
                            },
                            kind: SENSE_KIND,
                            // 01 §9: from B18 the body is the transport pair and
                            // then the words — `speaker_identity u64le |
                            // conversation u32le | text`.
                            body: si_core::eventlog::encode_sense_body(
                                step.text.as_bytes(),
                                identity,
                                section.session.conversation(),
                            )?,
                        });
                        seq += 1;
                    }
                }
            }
        }
        write_atomic(&dir.join("transcript.txt"), transcript.as_bytes())?;

        let identity = self.identity(ws)?;
        // OD-93: a variant is a build-graph state, so each exported report names
        // the variant id and the graph digest it executed on beside its hashes.
        let stamp = super::build_stamp(ws, &self.request.variant);
        write_atomic(&dir.join("mindlog.json"), self.mindlog(&stamp)?.as_bytes())?;
        // 04 §19 S.5: the monologue lane. One JSONL frame per rendered idle
        // cascade, and the same rows as a binary lane. These lines never touch
        // the owner lane (`MONOLOGUE_OWNER_FACING` is off in v1, OPEN-23's lean);
        // the copier asserts nothing, so a reader checks G10(d) against `cites`
        // and `lit_points` without re-walking.
        let mut monologue_jsonl = String::new();
        let mut monologue_events: Vec<Event> = Vec::new();
        for section in &self.sections {
            for line in section.session.monologue() {
                let frame = serde_json::json!({
                    "t": line.t,
                    "thought": line.thought,
                    "seed": line.seed_point,
                    "surface": line.surface,
                    "cites": line.cites.iter().map(|(node, round, edge)| serde_json::json!({"node": node, "round": round, "edge": edge})).collect::<Vec<_>>(),
                    "lit": line.lit_points,
                });
                monologue_jsonl.push_str(&frame.to_string());
                monologue_jsonl.push('\n');
                monologue_events.push(Event {
                    seq: monologue_events.len() as u64,
                    t: line.t,
                    source: 0,
                    kind: 17,
                    body: frame.to_string().into_bytes(),
                });
            }
        }
        write_atomic(&dir.join("monologue.jsonl"), monologue_jsonl.as_bytes())?;
        // 10 §8 / 05 §18: `sleep.jsonl` is the loop's own pass lines, verbatim —
        // one per pass that ran plus one per sleep — and `sleep_diff.json` is P3's
        // before/after table, the artifact the (b)/(c)/(d)/(j) legs grade from.
        // A run whose cadence never fired writes an empty file, which is 05 §18's
        // "reported as a finding about the scenario", not a missing artifact.
        let mut sleep_jsonl = String::new();
        let mut diff_rows: Vec<serde_json::Value> = Vec::new();
        for section in &self.sections {
            for line in section.session.sleep_lines() {
                sleep_jsonl.push_str(&line.to_string());
                sleep_jsonl.push('\n');
            }
            for entry in section.session.sleep_ledger() {
                diff_rows.push(serde_json::json!({
                    "sleep_t": entry.sleep_t,
                    "shelf": entry.shelf,
                    "edge_off": entry.edge_off,
                    "before": entry.before,
                    "after": entry.after,
                    "last_touched_at": entry.last_touched_at,
                    "d_q12": entry.d_q12,
                    "floor_q12": entry.floor_q12,
                    "below_floor": entry.below_floor,
                }));
            }
        }
        write_atomic(&dir.join("sleep.jsonl"), sleep_jsonl.as_bytes())?;
        // 12 §2.6's fourth log: one line per crystallize transition, per section.
        // An empty file is the honest record of a run in which the detector saw
        // no candidate — `CRYSTALLIZE` ships off, so most runs write nothing here.
        let mut crystallize_jsonl = String::new();
        for (section_index, section) in self.sections.iter().enumerate() {
            for line in section.session.crystallize_lines() {
                let mut entry = line.clone();
                if let Some(object) = entry.as_object_mut() {
                    object.insert("section".to_string(), serde_json::json!(section_index));
                }
                crystallize_jsonl.push_str(&entry.to_string());
                crystallize_jsonl.push('\n');
            }
        }
        write_atomic(&dir.join("crystallize.jsonl"), crystallize_jsonl.as_bytes())?;
        // 12 §2.6 / OD-103: the silt table's per-tick audit line, per section.
        // One line closes every tick the session ran, so a run without a table
        // still writes a line per tick with `active: 0` — the file is a log the
        // gate diffs, never an input, and its absence would mean "no ticks".
        let mut silt_jsonl = String::new();
        for (section_index, section) in self.sections.iter().enumerate() {
            for line in section.session.silt_lines() {
                let mut entry = line.clone();
                if let Some(object) = entry.as_object_mut() {
                    object.insert("section".to_string(), serde_json::json!(section_index));
                }
                silt_jsonl.push_str(&entry.to_string());
                silt_jsonl.push('\n');
            }
        }
        write_atomic(&dir.join("silt.jsonl"), silt_jsonl.as_bytes())?;
        // 12 §2.6 / OD-117: one line per shape transition, ask and mint. `SKILLS`
        // ships off, so most runs write nothing here; an empty file is the honest
        // record of a detector that never spoke, not a missing artifact.
        let mut skills_jsonl = String::new();
        for (section_index, section) in self.sections.iter().enumerate() {
            for line in section.session.skill_lines() {
                let mut entry = line.clone();
                if let Some(object) = entry.as_object_mut() {
                    object.insert("section".to_string(), serde_json::json!(section_index));
                }
                skills_jsonl.push_str(&entry.to_string());
                skills_jsonl.push('\n');
            }
        }
        write_atomic(&dir.join("skills.jsonl"), skills_jsonl.as_bytes())?;
        let sleeps: Vec<serde_json::Value> = self
            .sections
            .iter()
            .map(|section| {
                serde_json::json!({
                    "sleeps_completed": section.session.sleeps_completed(),
                    "sleeps_interrupted": section.session.core().sleeps_interrupted,
                    "last_slept_at_t": section.session.core().last_slept_at_t,
                    "sleep_pass_mask": section.session.core().sleep_pass_mask,
                    "quiet_run": section.session.quiet_run(),
                })
            })
            .collect();
        let below_floor = diff_rows
            .iter()
            .filter(|row| row["below_floor"].as_bool().unwrap_or(false))
            .count();
        let diff = serde_json::json!({
            "run_id": self.run_id,
            "gate": self.request.gate,
            "variant": self.request.variant,
            "config": self.request.config,
            "birth_seed_hex": format!("0x{:016x}", self.request.seed),
            "cadence_ticks": self.sections.first().map(|section| section.session.sleep_config().cadence_ticks),
            "max_walks": self.sections.first().map(|section| section.session.sleep_config().max_walks),
            "dim_base": self.sections.first().map(|section| section.session.sleep_config().dim_base),
            "floors_q12": self.sections.first().map(|section| serde_json::json!({
                "s_r": section.session.sleep_config().floor_s_r_q12,
                "facts": section.session.sleep_config().floor_facts_q12,
                "links": section.session.sleep_config().floor_links_q12,
            })),
            "edges_below_floor": below_floor,
            "sleeps": sleeps,
            "edges": diff_rows,
        });
        write_atomic(
            &dir.join("sleep_diff.json"),
            serde_json::to_string_pretty(&diff)?.as_bytes(),
        )?;
        let hashes = self
            .sections
            .iter()
            .map(|section| section.hash.clone())
            .collect::<Vec<_>>()
            .join("\n");
        write_atomic(&dir.join("state_hash.txt"), format!("{hashes}\n").as_bytes())?;

        let mut lane_header = serde_json::json!({
            "lane": "stream",
            "frames": self.sections.iter().map(|section| section.session.events().len()).sum::<usize>(),
            "run_id": self.run_id,
            "birth_seed": format!("0x{:016x}", self.request.seed),
        });
        stamp_into(&mut lane_header, &stamp);
        let stream: Vec<Event> = self
            .sections
            .iter()
            .flat_map(|section| section.session.events().events().iter().cloned())
            .filter(|event| LANE_KINDS.contains(&event.kind))
            .collect();
        write_atomic(&dir.join("stream.siro"), &lane(Format::StreamRecord, lane_header, &stream)?)?;
        let mut sense_header =
            serde_json::json!({"lane": "sense_inbox", "frames": inbox.len(), "run_id": self.run_id});
        stamp_into(&mut sense_header, &stamp);
        write_atomic(
            &dir.join("sense_inbox.siro"),
            &lane(Format::StreamRecord, sense_header, &inbox)?,
        )?;
        let mut monologue_header =
            serde_json::json!({"lane": "monologue", "frames": monologue_events.len(), "run_id": self.run_id});
        stamp_into(&mut monologue_header, &stamp);
        write_atomic(
            &dir.join("monologue.siro"),
            &lane(Format::StreamRecord, monologue_header, &monologue_events)?,
        )?;
        write_atomic(&dir.join("fingerprint.txt"), self.sections[0].session.fingerprint().as_bytes())?;

        // G14(c)(f): the container is saved, re-opened at the same seed and its
        // digest re-derived, then refused under a different seed. The seed is
        // never re-rollable and a resume never accepts another individual.
        let state_path = dir.join("SIst");
        self.sections[0].session.save(&state_path)?;
        let reopened = match reopen_session(&self.request, &state_path, self.request.seed)? {
            OpenOutcome::Opened(session) => session,
            OpenOutcome::Refused(refusal) => {
                bail!("the run's SIst refuses its own seed: {refusal}")
            }
        };
        let reopened_hash = reopened.state_hash()?.hex();
        let refusal = match reopen_session(&self.request, &state_path, self.request.seed ^ 1)? {
            OpenOutcome::Opened(_) => {
                bail!("SIst accepted a resume under another seed (G14.c.2)")
            }
            OpenOutcome::Refused(refusal) => Some(refusal.to_string()),
        };
        self.container = Some(ContainerCheck {
            live_hash: self.sections[0].hash.clone(),
            reopened_hash,
            same_seed_ok: true,
            other_seed_refused: refusal,
        });

        let report = super::gates::evaluate(self, ws)?;
        let path = dir.join("gate_report.json");
        write_atomic(&path, serde_json::to_string_pretty(&report)?.as_bytes())?;
        // 05 §4.1: `G14/seed_panel.json` is written per run. The run itself is one
        // individual's data point, so it is recorded after its own report exists —
        // the index carries the verdict the report reached, never a promise of one.
        super::panel::record(ws, self, report["pass"].as_bool().unwrap_or(false))?;
        let _ = identity;
        Ok(())
    }

    fn mindlog(&self, stamp: &serde_json::Value) -> Result<String> {
        let mut sections = Vec::new();
        for (section_index, section) in self.sections.iter().enumerate() {
            let mut steps = Vec::new();
            for step in &section.steps {
                let events: Vec<serde_json::Value> = self
                    .events(section_index, step)
                    .iter()
                    .map(event_json)
                    .collect();
                steps.push(serde_json::json!({
                    "index": step.index,
                    "t": step.t,
                    "source": step.source.map(|source| source.literal()).unwrap_or("idle"),
                    "input": step.text,
                    "idle": step.idle,
                    "a_before": step.a_before,
                    "a_after": step.a_after,
                    "rows_written": step.rows_written,
                    "output": step.output.as_ref().map(output_json),
                    "shelves_after": step.shelves_after,
                    "idle_revisited": step.idle_revisited.iter().map(|(ask, row)| serde_json::json!({"ask": ask, "row": row})).collect::<Vec<_>>(),
                    "idle_derived": step.idle_derived.iter().map(|(sr, row)| serde_json::json!({"s_r_row": sr, "fact_row": row})).collect::<Vec<_>>(),
                    "eligible": {"before": step.eligible_before, "after": step.eligible_after},
                    "route": step.route.map(|route| serde_json::json!({
                        "owned": route.owned,
                        "base": route.base,
                        "hops": route.hops,
                    })),
                    "boundary_before": boundary_json(&step.before),
                    "boundary_after": boundary_json(&step.after),
                    "speak": step.speak.as_ref().map(|note| serde_json::json!({
                        "reached_loop": note.reached_loop,
                        "admitted": note.admitted,
                        "dropped": note.dropped,
                        "reason": note.reason,
                        "layer": note.layer,
                        "n_untaught_tokens": note.n_untaught_tokens,
                        "novel_claims": note.novel_claims,
                        "ref_row_id": note.ref_row_id,
                        "untaught": note.untaught,
                    })),
                    "events": events,
                }));
            }
            steps.push(serde_json::json!({
                "final": true,
                "hash": section.hash,
                "rows": census(&section.session)?,
                "a": section.session.a(),
                "stage": si_session::stage(&section.session)?,
            }));
            sections.push(serde_json::json!({
                "index": section_index,
                "hash": section.hash,
                "steps": steps,
                // 02 §8 / OD-40(iii): the seam's own two logs, exported as data.
                // The run records what the mouth did; nothing feeds it back.
                "seam_state": section.seam_state,
                "gate_log": section.gate_log,
                "gate_reads": section.gate_reads,
                // 12 §2.6 / §4.3 (xv): the frame scratch as an observation. It is
                // session RAM, it is logged here, and it is hashed nowhere — so a
                // leg grades a ghost's life from this block, never from a byte.
                "frame_log": section.session.frame_logs(),
                // OD-98's audit trail and the candidate table as the log shows it.
                "crystallize": section.session.crystallize_lines(),
                "nomination_views": section.session.nomination_views(),
                // OD-103: the silt table as the log shows it. Derived query
                // RAM, never a byte; the same block `silt.jsonl` closes each
                // tick with, kept here so one file answers "what was warm".
                "silt_view": section.session.silt_view_json(),
                // 12 §2.6 / OD-117: the shape table as the log shows it. Usage
                // counters are derived from these rows plus the log (OD-120), so
                // the block is session RAM and is hashed nowhere.
                "skills": section.session.skill_lines(),
                "shape_views": section.session.shape_views(),
                // OD-120: 12 §2.6's usage counters, derived per live row from the
                // rows plus the log above. No row stores a number.
                "skill_usage": section.session.skill_usage_json()?,
                // 12 §2.6: the event views step 5b derives at frame load, keyed
                // by the `context` row's own id. Assembled from items 1–3, so
                // it is hash-exempt and stored nowhere (OD-110).
                "event_views": section.session.event_views_json(section.session.now())?,
            }));
        }
        let mut document = serde_json::json!({
            "run_id": self.run_id,
            "gate": self.request.gate,
            "birth_seed": format!("0x{:016x}", self.request.seed),
            "seed_format": format!("0x{:04x}", self.sections[0].session.core().seed_format),
            "temperament_digest": format!("0x{:016x}", self.sections[0].session.core().temperament_digest),
            "a": self.sections[0].session.a(),
            "sections": sections,
        });
        stamp_into(&mut document, stamp);
        Ok(serde_json::to_string_pretty(&document)?)
    }

    pub fn identity(&self, ws: &Workspace) -> Result<serde_json::Value> {
        let base = build_identity(ws);
        let session = &self.sections[0].session;
        let tags = session.hash_inputs()?.version_tags;
        let tags_hex = scenario::hex(&tags);
        Ok(serde_json::json!({
            "run_id": self.run_id,
            "gate": self.request.gate,
            "config": self.request.config,
            "binary_tree_md5": tree_digest(ws)?,
            "build_features": build_features(),
            "cargo_lock_md5": base.cargo_lock_md5,
            // OD-93 names this value, so the gate report carries the name too.
            "graph_digest": base.cargo_lock_md5,
            "deps_allowlist_md5": fs::read_to_string(ws.path("gates/deps_lock.md5"))
                .map(|text| text.trim().to_string())
                .unwrap_or_default(),
            "constant_version_tags": tags_hex,
            "consts_version": format!("0x{:04x}", base.consts_version),
            "hash_format": base.hash_format,
            "rng_seed": format!("0x{:016x}", self.request.seed),
            "rng": "derived from birth_seed by si_core::seed::runtime_stream (§4.1)",
            "birth_seed": self.request.seed,
            "birth_seed_hex": format!("0x{:016x}", self.request.seed),
            "seed_format": format!("0x{:04x}", session.core().seed_format),
            // 05 §2's identity list names this field, and §4.1 makes it
            // load-bearing: a run whose individual is not a panel member cannot be
            // cited by a box validation, so the absence is `null` rather than a guess.
            "seed_panel_member": super::panel::load(ws)?
                .and_then(|panel| panel.member_of(self.request.seed))
                .map(|member| member as u64),
            "build_variant": self.request.variant,
            "sleep_interrupt": self.request.sleep_interrupt,
            // 05 §4's V7 row is a launch state, so the report names the artifact and
            // the tag the lock pins — the same pair a resume compares (07 §13) — and
            // nothing else about the frozen layer. A CONFIG SI run carries `null`.
            "frozen_layer": match (&self.request.genome, self.request.genome_tag) {
                (Some(path), Some(tag)) => serde_json::json!({
                    "path": ws.relative(path),
                    "version_tag": format!("0x{tag:016x}"),
                    "points": self
                        .sections
                        .first()
                        .and_then(|section| section.session.store().frozen().map(|layer| layer.genome_points())),
                    "seal": self
                        .sections
                        .first()
                        .and_then(|section| section.session.store().frozen())
                        .map(|layer| layer.header.seal_md5.clone()),
                }),
                _ => serde_json::Value::Null,
            },
            // OD-E's rule is that a report records the launch it ran at. The
            // constant list belongs to that record: `read` and the cross-process
            // `replay` re-execute at these values, so a run at a swept point
            // replays to its own hash instead of the placeholder's.
            "constants": self.request.constants.iter().map(|(name, value)| {
                let mut entry = serde_json::Map::new();
                entry.insert(name.clone(), serde_json::json!(*value));
                serde_json::Value::Object(entry)
            }).collect::<Vec<_>>(),
            "scenario": {
                "path": self.request.scenario.path,
                "md5": self.request.scenario.digest,
                "steps": self.request.scenario.steps(),
            },
            "started_utc": self.started,
            "placeholders": si_core::consts::placeholder_count(),
        }))
    }
}

/// OD-93: copy the variant id + graph digest pair into an exported report's
/// header, so a hash is always read next to the build that produced it.
fn stamp_into(header: &mut serde_json::Value, stamp: &serde_json::Value) {
    if let (Some(map), Some(values)) = (header.as_object_mut(), stamp.as_object()) {
        for (key, value) in values {
            map.insert(key.clone(), value.clone());
        }
    }
}

fn output_json(output: &Output) -> serde_json::Value {
    match output {
        Output::Answer { surface, label, tag, citation } => serde_json::json!({
            "kind": "answer", "surface": surface, "label": label, "tag": tag, "citation": citation
        }),
        Output::Ask { surface, question_id, fresh } => serde_json::json!({
            "kind": "ask", "surface": surface, "question_id": question_id, "fresh": fresh
        }),
        Output::Committed { row_id, shelf, branch } => serde_json::json!({
            "kind": "committed", "row_id": row_id, "shelf": shelf, "branch": branch
        }),
        Output::Refused { reason } => serde_json::json!({"kind": "refused", "reason": reason}),
        Output::Inert { reason } => serde_json::json!({"kind": "inert", "reason": reason}),
    }
}

fn event_json(event: &Event) -> serde_json::Value {
    serde_json::json!({
        "seq": event.seq,
        "t": event.t,
        "source": event.source,
        "kind": event.kind,
        "body": String::from_utf8_lossy(&event.body),
    })
}

pub fn census(session: &Session) -> Result<BTreeMap<u16, usize>> {
    let mut out = BTreeMap::new();
    for (_, record) in session.store().rows()? {
        *out.entry(record.envelope.shelf_x).or_insert(0) += 1;
    }
    Ok(out)
}

/// The step-boundary observation: the three arena byte counts, the stamp arena's
/// digest, and the digest of the `s_r_pairs` envelopes in row-id order.
fn boundary(session: &Session) -> Result<Boundary> {
    let store = session.store();
    let mut sr = Md5::new();
    for (_, record) in store.rows()? {
        if record.envelope.shelf_x == SHELF_S_R_PAIRS {
            sr.update(record.envelope.encode());
        }
    }
    let mut stamps = Md5::new();
    stamps.update(store.stamps_arena().bytes());
    Ok(Boundary {
        rows_bytes: store.rows_arena().bytes().len(),
        edges_bytes: store.edges_arena().bytes().len(),
        stamps_bytes: store.stamps_arena().bytes().len(),
        stamps_digest: scenario::hex(stamps.finalize().as_slice()),
        sr_envelope_digest: scenario::hex(sr.finalize().as_slice()),
    })
}

pub fn boundary_json(boundary: &Boundary) -> serde_json::Value {
    serde_json::json!({
        "rows_bytes": boundary.rows_bytes,
        "edges_bytes": boundary.edges_bytes,
        "stamps_bytes": boundary.stamps_bytes,
        "stamps_digest": boundary.stamps_digest,
        "sr_envelope_digest": boundary.sr_envelope_digest,
    })
}

/// 05 §22: a sweep's declared constant list, installed at launch. Only the
/// scalar SPECIES rows a sweep owns — and the arc's RULING rows, whose shipped
/// value is the least-authority default — can be named here; an unknown name, a
/// draw, or an off-grid value refuses the run instead of clamping it.
fn launch_constants(
    session: &Session,
    values: &BTreeMap<String, i64>,
) -> Result<(Params, WalkConstants, si_sleep::SleepConfig, si_session::ArcConfig)> {
    let mut params = *session.params();
    let mut walk = WalkConstants::from_registry();
    let mut sleep = session.sleep_config();
    let mut arc = session.arc();
    for (name, value) in values {
        let row = si_core::consts::row(name)
            .ok_or_else(|| anyhow::anyhow!("`{name}` is not a registry row"))?;
        // OD-87: a TEMPERAMENT row is drawn, never installed; a LAW row is the
        // law itself and a DERIVED row is read off one. Neither may be a channel
        // value, so the class guard names the two that may.
        if !matches!(
            row.class,
            si_core::consts::Class::Species | si_core::consts::Class::Ruling
        ) {
            bail!("`{name}` is {:?}, not a swept SPECIES or RULING row", row.class);
        }
        if *value < row.lo || *value > row.hi || (value - row.lo) % row.grid != 0 {
            bail!("`{name} = {value}` is outside [{}, {}] on grid {}", row.lo, row.hi, row.grid);
        }
        match name.as_str() {
            "K_MAX_ROUNDS" => walk.k_max = *value as u16,
            "TAU_SETTLE" => walk.tau_settle = *value as u16,
            "DECAY_BASE" => walk.decay_base = (*value).clamp(0, 255) as u8,
            "HUB_DAMP_EXPONENT" => walk.hub_exponent_q12 = *value as u16,
            "FRONTIER_CAP" => walk.frontier_cap = *value as usize,
            "OSCILLATION_RULE" => {
                walk.oscillation = if *value == 1 {
                    si_walk::OscillationRule::PeriodCheck
                } else {
                    si_walk::OscillationRule::MaxKOnly
                }
            }
            // 03 §5.1's six sleep rows. A value here is a point of the registry's
            // own box, and `si_sleep::launch_ok` is the class check that keeps a
            // temperament draw out of it (**OD-A**).
            "SLEEP_EVERY_QUIET_TICKS" => sleep.cadence_ticks = *value as u32,
            "SLEEP_MAX_WALKS" => sleep.max_walks = *value as u32,
            "DIM_BASE" => sleep.dim_base = (*value).clamp(0, 255) as u8,
            "FLOOR_S_R" => sleep.floor_s_r_q12 = *value as u16,
            "FLOOR_FACTS" => sleep.floor_facts_q12 = *value as u16,
            "FLOOR_LINKS" => sleep.floor_links_q12 = *value as u16,
            "INFERENCE_STABLE_N" => sleep.stable_n = *value as u16,
            "GAP_SALIENCE_FLOOR" => params.gap_salience_floor = *value as u16,
            "DEDUPE_WINDOW" => params.dedupe_window = *value as u16,
            "BIAS_CAP_SHARE" => params.bias_cap_share = *value as u16,
            "B_MAX" => params.b_max = *value as u16,
            "K_TANH" => params.k_tanh = *value as u16,
            "MEDIAN_MIN_N" => params.median_min_n = *value as u16,
            "MEDIAN_SHAPE" => params.median_shape = (*value).clamp(0, 2) as u8,
            "MOD_LAG_ROUNDS" => params.lag_rounds = *value as u16,
            // engineering/12 §3.1: the arc's sixteen swept SPECIES rows and
            // §3.2's six RULING rows, installed on the one channel. A gate names
            // the point it needs (OD-95's precedent) because OD-83 still refuses
            // to enumerate them.
            "GHOST_MAX" => arc.ghost_max = *value as u16,
            "R_MIN_CRYSTALLIZE" => arc.r_min = *value as u16,
            "R_ASK_CRYSTALLIZE" => arc.r_ask = *value as u16,
            "DELTA_MIN_CRYSTALLIZE" => arc.delta_min = *value as i32,
            "LAMBDA_SILT" => arc.lambda_silt = (*value).clamp(0, 255) as u8,
            "SILT_LEVEL_MAX" => arc.silt_level_max = *value as u16,
            "SILT_BUMP" => arc.silt_bump = *value as u16,
            "THETA_SILT" => arc.theta_silt = *value as u16,
            "SILT_MAX" => arc.silt_max = *value as u16,
            "IGN_WEIGHT_SILT" => arc.ign_weight_silt = *value as u16,
            "PROV_LINK_TUG" => arc.prov_link_tug = *value as u16,
            "RELIGHT_MAX" => arc.relight_max = *value as u16,
            "R_MIN_SHAPE" => arc.r_min_shape = *value as u16,
            "DELTA_MIN_SHAPE" => arc.delta_min_shape = *value as i32,
            "K_FILLER" => arc.k_filler = *value as u16,
            "SHAPE_MAX" => arc.shape_max = *value as u16,
            "CRYSTALLIZE" => arc.crystallize = *value == 1,
            "SILT_SPREAD_PRIMING" => arc.silt_spread_priming = *value == 1,
            "EPISODIC_TRAVERSAL" => arc.episodic_traversal = *value == 1,
            "SKILLS" => arc.skills = *value == 1,
            "SKILL_COMPOSITION" => arc.skill_composition = *value == 1,
            "WORLD_TIERS" => arc.world_tiers = *value as u8,
            // 13 §3.2: the tongue's two qualifying rows. `TONGUE` is the RULING row
            // V14 installs; `PHRASE_MAX_STEPS` is the swept SPECIES ceiling a gate
            // names because no artifact has (**OD-83**, **OD-95**'s floor precedent).
            // The five `STYLE_*` rows are class `Tongue` and the guard above already
            // refuses them — which is why that text does not move.
            "TONGUE" => arc.tongue = *value == 1,
            "PHRASE_MAX_STEPS" => arc.phrase_max_steps = (*value).max(0) as usize,
            other => bail!("`{other}` is not a walk or affect parameter this sweep can install"),
        }
    }
    Ok((params, walk, sleep, arc))
}

/// 05 §4 / §22: install a run's declared constant list, if it has one. The sleep
/// config rides the same launch as the walk parameters because step 12b reads all
/// of them from the registry, and a run with no declared list runs at the
/// registry's own placeholder — the loop's code path is identical either way.
pub(crate) fn install_launch_state(session: &mut Session, values: &BTreeMap<String, i64>) -> Result<()> {
    if values.is_empty() {
        return Ok(());
    }
    let (params, walk, sleep, arc) = launch_constants(session, values)?;
    session.install_launch_constants(params, walk);
    session.install_sleep_config(sleep);
    session.install_arc_config(arc);
    Ok(())
}

/// 11 §B18 break-list row 10: the three in-run world declarations, carried out as
/// the scenario reaches them. They mutate session RAM — the grant view and the
/// declared window — and write no row of any shelf: OD-123 has not ruled where a
/// grant row lives, and 12 §11 refuses a new shelf, so the grant table is a launch
/// state installed by the owner's own declaration in the file (docs/26 §26.1), not
/// a `provenance` kind. A declaration is neither a sense line nor a quiet tick: it
/// moves no clock and answers with the tick it ran at.
pub fn world_step(session: &mut Session, op: &WorldOp) -> Result<(i32, bool)> {
    let t = session.now();
    match op {
        WorldOp::Grant { subject, right } => {
            let owner = session.world().owner_identity;
            session.grant(GrantRow {
                subject: *subject,
                right: *right,
                granted_by: owner,
                granted_at_t: t,
                revoked_at_t: None,
            });
        }
        WorldOp::Revoke { subject, right } => session.revoke(*subject, *right, t),
        WorldOp::Window(window) => session.set_window(*window),
    }
    Ok((session.now(), false))
}

/// The transcript's rendering of one declaration: the same three words the
/// scenario file speaks, so a replayed run re-declares exactly what it declared.
pub fn world_literal(op: &WorldOp) -> String {
    match op {
        WorldOp::Grant { subject, right } => format!("grant {subject} {}", right.literal()),
        WorldOp::Revoke { subject, right } => format!("revoke {subject} {}", right.literal()),
        WorldOp::Window(None) => "window close".to_string(),
        WorldOp::Window(Some(window)) => {
            format!("window {} {}", window.open_from_t, window.open_through_t)
        }
    }
}

/// 11 §B18: install the two ids a scenario registers. A file that declares
/// neither keeps the birth registration — owner `0`, grimm `1` — which is why a
/// pre-B18 scenario derives exactly the classes it derived before this existed.
pub fn install_world(session: &mut Session, scenario: &Scenario) {
    let registered = scenario.registered();
    session.set_registered_identities(registered.owner, registered.grimm);
}

/// The other half of `install_world` for a path that has the two numbers rather
/// than the file: the birth pair is what every pre-B18 launch state was.
pub fn install_registered(session: &mut Session, registered: Identities) {
    session.set_registered_identities(registered.owner, registered.grimm);
}

/// 05 §18's operating point for the sleep legs. `SLEEP_EVERY_QUIET_TICKS`'s
/// registry placeholder is the midpoint of its own box (**32769**), so no gate
/// scenario of `G5_LONG_SESSION_INPUTS` shape can reach it, and §18's
/// prerequisite — "the cadence fires **at least twice**" — is unsatisfiable at
/// that point. The declared alternative is the box's own floor: each of the six
/// sleep rows read at `lo`, from the registry, so the harness names a point the
/// spec already declares rather than inventing one (**OD-95**). Every value is
/// on-grid by construction, and `install_launch_state` is the same channel a
/// sweep uses.
pub fn sleep_lo_point() -> BTreeMap<String, i64> {
    let mut out = BTreeMap::new();
    for name in [
        "SLEEP_EVERY_QUIET_TICKS",
        "SLEEP_MAX_WALKS",
        "DIM_BASE",
        "FLOOR_S_R",
        "FLOOR_FACTS",
        "FLOOR_LINKS",
        "INFERENCE_STABLE_N",
    ] {
        let row = si_core::consts::row(name)
            .unwrap_or_else(|| panic!("{name} is not a registry row (03 §5.1)"));
        out.insert(name.to_string(), row.lo);
    }
    out
}

/// 05 §23.1's declared point for the crystallize family, and the point the run
/// itself must launch at (**12 §6.4**: "the gate launches at the rolled state it
/// grades"). The three recurrence rows are at their registry `lo` — the smallest
/// armed point of each, read out of the registry and inventing no value — and the
/// two shape caps are at the values §23.1 names, each asserted on its own grid.
/// `CRYSTALLIZE` is a `Ruling` bool, not a swept row, so the gate installs it and
/// no sweep may vary it.
pub fn g15_launch_point() -> BTreeMap<String, i64> {
    let mut out = BTreeMap::new();
    for name in [
        "R_MIN_CRYSTALLIZE",
        "R_ASK_CRYSTALLIZE",
        "DELTA_MIN_CRYSTALLIZE",
    ] {
        let row = si_core::consts::row(name)
            .unwrap_or_else(|| panic!("{name} is not a registry row (12 §3.1)"));
        out.insert(name.to_string(), row.lo);
    }
    for (name, value) in [("GHOST_MAX", 1i64), ("SHAPE_MAX", 4i64)] {
        let row = si_core::consts::row(name)
            .unwrap_or_else(|| panic!("{name} is not a registry row (12 §3.1)"));
        assert!(
            value >= row.lo && value <= row.hi && (value - row.lo) % row.grid == 0,
            "{name} = {value} is off its own grid"
        );
        out.insert(name.to_string(), value);
    }
    out.insert("CRYSTALLIZE".to_string(), 1);
    out
}

/// 05 §23.2's declared point for the five silt rows and the fifth ignition
/// term. The table's rows run at their registry `lo`; `IGN_WEIGHT_SILT` — a
/// `declared_row` whose `lo` is 0, i.e. the shipped unarmed default — runs at
/// `lo + grid`, the smallest *armed* point on its grid, so the term is present
/// and the wiring is what G16 grades. At that point `mul_q12(1, 1) == 0`, so
/// every equality leg (G16.a.2, G16(e)) still claims equality honestly.
pub fn g16_launch_point() -> BTreeMap<String, i64> {
    let mut out = BTreeMap::new();
    for name in ["LAMBDA_SILT", "SILT_LEVEL_MAX", "SILT_BUMP", "THETA_SILT", "SILT_MAX"] {
        let row = si_core::consts::row(name)
            .unwrap_or_else(|| panic!("{name} is not a registry row (12 §5 §30)"));
        out.insert(name.to_string(), row.lo);
    }
    let row = si_core::consts::row("IGN_WEIGHT_SILT")
        .unwrap_or_else(|| panic!("IGN_WEIGHT_SILT is not a registry row (12 §5 §30)"));
    let value = row.lo + row.grid;
    assert!(value <= row.hi, "IGN_WEIGHT_SILT has no armed point at lo + grid");
    out.insert("IGN_WEIGHT_SILT".to_string(), value);
    // (e) is a *detector* leg: it needs the detector running, so the arc's
    // ruling switch is on — the same launch G15 installs (12 §6.4). It is a
    // `Ruling` bool, not a swept row, so no sweep may vary it.
    out.insert("CRYSTALLIZE".to_string(), 1);
    out
}

/// 05 §23.3's declared point for the episodic family: the traversal ruling
/// installed `on`, `PROV_LINK_TUG` at its smallest *armed* point (`lo + grid`,
/// because at `lo` it is 0 and the index would tug nothing), and `RELIGHT_MAX`
/// at its own floor. `EPISODIC_TRAVERSAL` is a `Ruling` bool, not a swept row,
/// so no sweep may vary it; the gate installs it the way G15 installs
/// `CRYSTALLIZE` (12 §6.4).
pub fn g17_launch_point() -> BTreeMap<String, i64> {
    let mut out = BTreeMap::new();
    let row = si_core::consts::row("PROV_LINK_TUG")
        .unwrap_or_else(|| panic!("PROV_LINK_TUG is not a registry row (12 §3.1)"));
    let value = row.lo + row.grid;
    assert!(value <= row.hi, "PROV_LINK_TUG has no armed point at lo + grid");
    out.insert("PROV_LINK_TUG".to_string(), value);
    let row = si_core::consts::row("RELIGHT_MAX")
        .unwrap_or_else(|| panic!("RELIGHT_MAX is not a registry row (12 §3.1)"));
    out.insert("RELIGHT_MAX".to_string(), row.lo);
    out.insert("EPISODIC_TRAVERSAL".to_string(), 1);
    out
}

/// 05 §23.4's declared point for the skills family. The three swept bars are at
/// their registry `lo` — the only points in each box the law names, per OD-95's
/// precedent — and the two `Ruling` switches (`SKILLS`, `SKILL_COMPOSITION`) are
/// installed `on` because the gate grades the mechanism, not the switch.
/// `SHAPE_MAX` is the frame scratch's role-graph cap, and at its `lo` (**0**) the
/// fold builds no shape at all, so the detector would be blind and every leg
/// would pass over nothing. §23.4 names no value of its own; the only value any
/// section of the spec declares for this row is §23.1's **4**, so this point uses
/// it and says so. The crystallize rows ride along at the same point G15 ships:
/// leg (f) is one test, not two claims (**OD-121**), so the forwarded frames must
/// be observed by §29's fold **in the same run**, which needs that detector live.
pub fn g18_launch_point() -> BTreeMap<String, i64> {
    let mut out = g15_launch_point();
    for name in ["R_MIN_SHAPE", "DELTA_MIN_SHAPE", "K_FILLER"] {
        let row = si_core::consts::row(name)
            .unwrap_or_else(|| panic!("{name} is not a registry row (12 §3.1)"));
        out.insert(name.to_string(), row.lo);
    }
    out.insert("SKILLS".to_string(), 1);
    out.insert("SKILL_COMPOSITION".to_string(), 1);
    out
}

/// 05 §2's V13 row, as a launch state: `WORLD_TIERS = granted` is the registry
/// row's `hi` (0 = `owner-only`, the law value; 1 = `granted`), and the grant
/// table V13 names is **built by the scenario's own owner line** — the `grant …`
/// declaration in `g19_server.txt` — because no adapter exists (**OD-136**). This
/// function installs only the ruling switch; nothing in the build branches on
/// `arc.world_tiers` (`04 §33`'s AE reading, OD-125's "coded, not granted"), so
/// V0 and V13 run the same code and the gate's (c) leg is graded on the refusal
/// itself, printed with OD-135 as its reason.
pub fn g19_launch_point() -> BTreeMap<String, i64> {
    let mut out = g18_launch_point();
    let row = si_core::consts::row("WORLD_TIERS")
        .unwrap_or_else(|| panic!("WORLD_TIERS is not a registry row (03 §5)"));
    out.insert("WORLD_TIERS".to_string(), row.hi);
    out
}

/// 05 §2's V14 row and 13 §6.1, as a launch state: the voice is the thing the gate
/// grades, so `TONGUE` installs `on` (a Ruling row, 12 §3.2's widened filter) and
/// `PHRASE_MAX_STEPS` installs at **32** — the smallest value on its own row that
/// lets the *lowest-spread* individual speak at all, because the budget is
/// `steps × STYLE_SPREAD >> 12` and `STYLE_SPREAD`'s grid step is 128, so 32 is the
/// first `steps` at which a spread of 128 buys one extra chunk. A named point, not a
/// taste: at the registry `lo` of 1 every tick refuses and the gate grades its own
/// absence (**OD-95**'s floor discipline, measured in `failure_report.md`).
pub fn g20_launch_point() -> BTreeMap<String, i64> {
    let mut out = BTreeMap::new();
    out.insert("TONGUE".to_string(), 1);
    out.insert("PHRASE_MAX_STEPS".to_string(), 32);
    out
}

/// 05 §18's in-process driver. A sleep leg needs the loop at a named point of a
/// swept box, with an owner frame scheduled at one pass boundary, or cut open at
/// one step so the crash story can be re-run — none of which is a run directory
/// the gate should publish, and one of which (`gate: "G13"`) would recurse into
/// the evaluator that called it. So this is the same loop path `execute` drives
/// (`tick`, `idle_tick`, `freeze`), one step at a time, and it hands back the
/// session itself, which is what lets a leg diff the arenas around one night.
#[derive(Clone, Debug)]
pub struct DriveRequest {
    pub steps: Vec<Step>,
    /// 11 §B18: the two ids the driven file registers (`identity 3 owner`). `None`
    /// is the birth pair, which is what every pre-B18 driver asked for. A driver
    /// that replays a world fixture has to install it, or the class each step
    /// derives is not the class the run derived.
    pub identities: Option<Identities>,
    pub seed: u64,
    pub constants: BTreeMap<String, i64>,
    pub variant: String,
    pub sleep_interrupt: Option<u8>,
    /// 05 §18(h)'s crash: at this 1-based step index the session is saved, the
    /// file re-opened at the same seed, and the remaining steps run from the
    /// resumed session. `None` runs the steps straight through.
    pub cut_after: Option<usize>,
    pub cut_path: Option<PathBuf>,
    /// The checkpoint sleep, the way `execute` takes it before hashing.
    pub freeze_at_end: bool,
    /// 09 §8's bulk channel, as the loop hears it: the bytes of a `sense_inbox.siro`
    /// lane written by [`si_session::Session::encode_sense_inbox`] — the same writer
    /// `si-cli feed` uses. `drive` hands them to [`si_session::Session::hear_sense_inbox`]
    /// before the first step, because a lane that already arrived is earlier in logical
    /// time than the lines typed at this invocation. So a fed arm and a typed arm are
    /// one loop code and `G1.u.2`'s byte-identity has no harness branch in it (05 §1).
    /// Empty means no lane, which is what every other driver asked for.
    pub inbox_lane: Vec<u8>,
}

/// One step as the driver saw it: the logical tick it ran at, whether it was
/// quiet, the frame kinds its own event range contains, and the `sleep.jsonl`
/// lines it wrote. A leg locates the nights inside a run with this, and grades
/// **G13(l).2**'s Δt from the `t` column.
#[derive(Clone, Debug)]
pub struct StepTrace {
    pub index: usize,
    pub t: i32,
    pub idle: bool,
    pub kinds: Vec<u8>,
    pub sleep_lines: Vec<serde_json::Value>,
    pub conflicts: usize,
    pub rows_after: usize,
    pub stage_after: u8,
    /// 05 §18(f.1) grades "a sleep fires at exactly the tick the counter reaches
    /// the swept value", so the counter's value at each step is an observation the
    /// driver has to hand back. RAM only, never a byte (01 §13).
    pub quiet_run_after: u32,
    /// 04 §19 S.2 step 3 / §33 AE.4 (**G19(g)**): the two products a quiet tick
    /// hands back that no other step has — the hyperpoint it ignited on, and
    /// whether a declared window denied this tick. `None`/`false` on a sense step.
    /// The window leg is a *pair* leg: it compares the same transcript with and
    /// without a `window` line, so the reports have to travel with the trace.
    pub idle_ignited: Option<u32>,
    pub idle_suppressed_by_window: bool,
    /// How many open asks the tick's revisit pass found (04 §19 S.2 step 2). A
    /// suppressed tick must show zero: AE.4's denial sits on the quiet-and-empty-
    /// agenda predicate, so a tick the window stopped was a tick that had nothing
    /// to revisit.
    pub idle_revisited: usize,
    /// 13 §6.6 **G20**: the frame this tick printed — the label and citation the
    /// `label` event carries and the surface the `output` event carries, which is
    /// the string the owner actually read. `None` on a step that printed no frame.
    pub frame: Option<FrameTrace>,
    /// 13 §6.6 **G20**: step 9a's own record at this tick. Like `route` in
    /// `StepRecord`, it is session RAM and moves zero hashed bytes, and at
    /// `TONGUE = 0` it is `None` on every step — which is itself the evidence
    /// `G20(j)` asks for.
    pub tongue: Option<si_session::TongueTick>,
}

/// The printed frame, as the two events that carry it name it. Read off the event
/// log rather than off the `Output` value so a leg comparing the canonical and the
/// realized string is comparing what was emitted, not what was built.
#[derive(Clone, Debug)]
pub struct FrameTrace {
    pub label: String,
    pub tag: String,
    pub surface: String,
    pub citation: Vec<u32>,
}

/// One bulk lane as the loop heard it, kept as numbers rather than reports so a leg
/// can say what a fed arm applied without re-reading the lane (09 §8).
#[derive(Clone, Debug)]
pub struct InboxSummary {
    pub frames_heard: usize,
    pub torn_tail_bytes: usize,
    pub refused: Option<String>,
}

pub struct DriveOutcome {
    pub session: Session,
    pub hash: String,
    pub steps: Vec<StepTrace>,
    /// What `request.inbox_lane` did to this session: `None` when no lane was handed,
    /// else how many frames the loop applied, the size of any torn tail it refused to
    /// hear, and the named reason when the lane was refused instead. A bulk-equivalence
    /// leg reads this first, because a refused lane applies nothing and would otherwise
    /// compare as a fed arm that had never been fed (01 §9).
    pub inbox: Option<InboxSummary>,
    /// The state hash at the cut, before the file was written, and whether the
    /// run continued from a re-open (`G13(h).1`'s two halves).
    pub cut_hash: Option<String>,
    pub resumed: bool,
    /// The same bytes the cut wrote, re-read: `G13(h).1`'s "the resumed session
    /// is byte-identical to the pre-sleep checkpoint's" is this pair.
    pub resumed_hash: Option<String>,
}

pub fn drive(request: &DriveRequest) -> Result<DriveOutcome> {
    let mut session = Session::new(request.seed)?;
    install_launch_state(&mut session, &request.constants)?;
    if let Some(registered) = request.identities {
        install_registered(&mut session, registered);
    }
    session.set_launch_variant(&request.variant);
    session.set_sleep_interrupt(request.sleep_interrupt);
    let mut steps: Vec<StepTrace> = Vec::new();
    let mut cut_hash = None;
    let mut resumed_hash = None;
    let mut resumed = false;
    let mut logged = 0usize;
    let mut mouth = Mouth::launch(&request.variant, None, None);

    // 09 §8: a bulk lane is heard by the ordinary loop, before the lines typed at this
    // invocation — it arrived earlier in logical time. The frames it applies are the
    // lane's own events, not step one's, so the step trace's event slices stay honest.
    let mut inbox: Option<InboxSummary> = None;
    if !request.inbox_lane.is_empty() {
        inbox = Some(match session.hear_sense_inbox(&request.inbox_lane)? {
            InboxDrain::Heard { frames, torn_tail_bytes } => InboxSummary {
                frames_heard: frames.len(),
                torn_tail_bytes,
                refused: None,
            },
            InboxDrain::Refused(refusal) => InboxSummary {
                frames_heard: 0,
                torn_tail_bytes: 0,
                refused: Some(refusal.to_string()),
            },
        });
        logged = session.events().events().len();
    }

    for (offset, step) in request.steps.iter().enumerate() {
        let index = offset + 1;
        let mut idle_ignited = None;
        let mut idle_suppressed_by_window = false;
        let mut idle_revisited = 0usize;
        let mut reached_loop = true;
        let (t, idle) = match step {
            Step::Sense { source, text, identity, .. } => {
                let (t, note) =
                    sense_step_noted(&mut session, &mut mouth, *source, text, *identity)?;
                reached_loop = note.as_ref().map(|note| note.reached_loop).unwrap_or(true);
                (t, false)
            }
            Step::World(op) => {
                reached_loop = false;
                world_step(&mut session, op)?
            }
            Step::Idle => {
                let report = session.idle_tick()?;
                idle_ignited = report.ignited;
                idle_suppressed_by_window = report.suppressed_by_window;
                idle_revisited = report.revisited.len();
                (report.t, true)
            }
        };
        let events = session.events().events();
        let slice = &events[logged.min(events.len())..];
        let sleep_lines: Vec<serde_json::Value> = slice
            .iter()
            .filter(|event| event.kind == 19)
            .filter_map(|event| serde_json::from_str(&String::from_utf8_lossy(&event.body)).ok())
            .collect();
        let conflicts = slice
            .iter()
            .filter(|event| {
                event.kind == 8 && String::from_utf8_lossy(&event.body).contains("conflict")
            })
            .count();
        logged = events.len();
        // 13 §6.6 **G20**: the frame the tick printed and the voice's own record,
        // both read off the tick's events and RAM. `label` and `output` are the two
        // halves of one emission — the citation is in the first, the string the
        // owner read is in the second — so a leg that compares a canonical frame
        // with a realized line compares the emission, never a rebuilt value.
        use si_core::eventlog::kind;
        let frame = (|| {
            let label = slice.iter().rev().find(|event| event.kind == kind::LABEL)?;
            let output = slice.iter().rev().find(|event| event.kind == kind::OUTPUT)?;
            let label: serde_json::Value =
                serde_json::from_str(&String::from_utf8_lossy(&label.body)).ok()?;
            let output: serde_json::Value =
                serde_json::from_str(&String::from_utf8_lossy(&output.body)).ok()?;
            Some(FrameTrace {
                label: label["label"].as_str()?.to_string(),
                tag: label["tag"].as_str()?.to_string(),
                surface: output["surface"].as_str()?.to_string(),
                citation: label["citation"]
                    .as_array()?
                    .iter()
                    .filter_map(|row| row.as_u64().map(|id| id as u32))
                    .collect(),
            })
        })();
        let tongue = if idle || !reached_loop {
            None
        } else {
            si_session::last_tongue(&session)
        };
        steps.push(StepTrace {
            index,
            t,
            idle,
            kinds: slice.iter().map(|event| event.kind).collect(),
            sleep_lines,
            conflicts,
            rows_after: census(&session)?
                .values()
                .sum(),
            stage_after: si_session::stage(&session)?,
            quiet_run_after: session.quiet_run(),
            idle_ignited,
            idle_suppressed_by_window,
            idle_revisited,
            frame,
            tongue,
        });
        if Some(index) == request.cut_after {
            cut_hash = Some(session.state_hash()?.hex());
            if let Some(path) = &request.cut_path {
                session.save(path)?;
                session = match Session::open(path, request.seed)? {
                    OpenOutcome::Opened(opened) => *opened,
                    OpenOutcome::Refused(refusal) => {
                        bail!("the crash checkpoint refuses its own seed: {refusal}")
                    }
                };
                resumed_hash = Some(session.state_hash()?.hex());
                resumed = true;
                logged = 0;
                // 11 §B18: the grant view and the window are session RAM — no
                // snapshot carries them, because a grant row has no ruled home
                // (OD-123) — so a resumed session re-declares what the cut had
                // already declared, in file order and at the tick it resumes at.
                for step in request.steps.iter().take(index) {
                    if let Step::World(op) = step {
                        world_step(&mut session, op)?;
                    }
                }
            }
        }
    }
    if request.freeze_at_end {
        session.freeze()?;
    }
    let hash = session.state_hash()?.hex();
    Ok(DriveOutcome {
        session,
        hash,
        steps,
        inbox,
        cut_hash,
        resumed,
        resumed_hash,
    })
}

/// The three arena byte dumps a diff is taken over, as the store holds them.
pub fn arena_bytes(session: &Session) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let store = session.store();
    let mut rows = Vec::new();
    for (_, record) in store.rows()? {
        rows.extend(record.encode());
    }
    Ok((
        rows,
        store.edges_arena().bytes().to_vec(),
        store.stamps_arena().bytes().to_vec(),
    ))
}

fn lane(format: Format, header: serde_json::Value, events: &[Event]) -> Result<Vec<u8>> {
    let mut payload = Vec::new();
    for event in events {
        let frame = event.encode();
        payload.extend_from_slice(&(frame.len() as u32).to_le_bytes());
        payload.extend_from_slice(&frame);
    }
    Container::new(format, header, payload).encode()
}

/// 11 §?/05 §1: the source-tree digest a report carries so a run is attributable
/// to the build that produced it. The allowlist has no SHA-2 crate (02 §12), so
/// the digest is md5 and the field is named for what it is.
pub fn tree_digest(ws: &Workspace) -> Result<String> {
    use md5::{Digest, Md5};
    let mut files = Vec::new();
    for name in ws.crate_names()? {
        let src = ws.path(&format!("crates/{name}/src"));
        if src.is_dir() {
            collect(&src, &mut files)?;
        }
    }
    for extra in ["Cargo.toml", "Cargo.lock"] {
        files.push(ws.path(extra));
    }
    let harness = ws.path("harness");
    if harness.is_dir() {
        collect(&harness, &mut files)?;
    }
    files.sort();
    let mut hasher = Md5::new();
    for path in files {
        let relative = ws.relative(&path);
        hasher.update(relative.as_bytes());
        hasher.update([0u8]);
        hasher.update(fs::read(&path).with_context(|| format!("read {}", path.display()))?);
        hasher.update([0u8]);
    }
    Ok(scenario::hex(hasher.finalize().as_slice()))
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

fn unique_dir(base: &Path) -> PathBuf {
    if !base.exists() {
        return base.to_path_buf();
    }
    for n in 2..1000 {
        let candidate = PathBuf::from(format!("{}-{n}", base.display()));
        if !candidate.exists() {
            return candidate;
        }
    }
    base.to_path_buf()
}

/// The run-directory stamp (05 §1). This is a file name, never loop state.
fn utc_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0) as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}{month:02}{day:02}-{:02}{:02}",
        rem / 3600,
        (rem % 3600) / 60
    )
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    (y + i64::from(m <= 2), m, d)
}
