//! si-cli — engineering/00 §7. `run` wakes a CONFIG SI session, drives a
//! transcript through the sense loop and writes the wake artifacts; `feed` is the
//! inbox-only bulk path (09 §10) and `export` re-derives the derived tables
//! (export-only). `view` is process B's launcher ([08 §5]): it decodes the run
//! directory's artifacts and hands the values to `si-vulkan`, which writes nothing
//! but the owner's inbox.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use si_core::container::{Container, Format};
use si_core::consts;
use si_core::eventlog::Event;
use si_core::strings::Source;
use si_session::{InboxDrain, OpenOutcome, Output, Session};
use si_store::layers_frozen::{self, Config};
use si_store::snapshot::write_atomic;

#[derive(Parser)]
#[command(name = "si-cli", about = "SI — zero-gradient synthetic life (00 §7)")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// One session: wake, feed a transcript, settle, sleep when due.
    Run {
        /// The individual's birth seed (`0x…` or decimal). Required, and never
        /// drawn again (G14.j.1).
        #[arg(long)]
        seed: String,
        #[arg(long)]
        transcript: Option<String>,
        #[arg(long, default_value = "si")]
        config: String,
        /// 07 §9's W.1: the artifact this ENGINE session loads, and 07 §13's tag
        /// the manifest lock pins it by. Neither belongs to a CONFIG SI run, which
        /// forms no path to a frozen layer (00 §6.1).
        #[arg(long)]
        genome: Option<String>,
        #[arg(long)]
        genome_tag: Option<String>,
        #[arg(long)]
        run_dir: String,
        #[arg(long)]
        strict_consts: bool,
    },
    /// Feed lines into a live session's inbox (the bulk, inbox-only path).
    Feed {
        /// The run directory whose `sense_inbox.siro` receives the lines.
        #[arg(long)]
        session: String,
        #[arg(long)]
        file: String,
    },
    /// Re-derive the derived tables into an export (export-only, never read back).
    Export {
        /// The run directory holding `SIst`.
        #[arg(long)]
        session: String,
        #[arg(long)]
        format: Option<String>,
        /// The birth seed the container re-opens under; required, and only ever
        /// compared (never drawn again — G14.j).
        #[arg(long)]
        seed: String,
    },
    /// Watch a session through the viewer (observer only).
    View {
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        exit_after_frames: Option<u32>,
    },
    /// 07 §3's offline converter: a pinned manifest in, one sealed artifact out.
    /// It is the build's own pipeline, so it arrives with the loader it feeds
    /// (00 §6.2: a binary without the feature carries no converter at all).
    BuildGenome {
        /// The manifest whose sources are pinned by content hash (07 §3 M.0).
        #[arg(long)]
        manifest: String,
        /// Where the artifact and its sidecars are published. Absent means the
        /// manifest's own `out_dir`.
        #[arg(long)]
        out: Option<String>,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode> {
    match Args::parse().command {
        Command::Run { seed, transcript, config, genome, genome_tag, run_dir, strict_consts } => {
            run_session(
                &seed,
                transcript.as_deref(),
                &config,
                genome.as_deref(),
                genome_tag.as_deref(),
                &run_dir,
                strict_consts,
            )
        }
        Command::Feed { session, file } => feed_inbox(&session, &file),
        Command::BuildGenome { manifest, out } => build_genome(&manifest, out.as_deref()),
        Command::Export { session, format, seed } => {
            export_session(&session, &seed, format.as_deref())
        }
        Command::View { session, exit_after_frames } => {
            view_session(session.as_deref(), exit_after_frames)
        }
    }
}

fn run_session(
    seed_text: &str,
    transcript: Option<&str>,
    config_text: &str,
    genome: Option<&str>,
    genome_tag: Option<&str>,
    run_dir: &str,
    strict_consts: bool,
) -> Result<ExitCode> {
    let seed = parse_seed(seed_text)?;
    let config = Config::parse(config_text)
        .ok_or_else(|| anyhow::anyhow!("`--config` is `si` or `engine`, not `{config_text}`"))?;

    if strict_consts {
        let count = consts::placeholder_count();
        if count > 0 {
            bail!(
                "--strict-consts: {count} species rows are still {} (OD-86)",
                consts::PLACEHOLDER_TAG
            );
        }
    }

    // 00 §6: CONFIG SI ships no frozen layer and forms no path to one (I2), so a
    // SI launch that names an artifact is refused at this entry rather than
    // reaching the loader. CONFIG ENGINE asks, and refuses again when nothing is
    // pinned (07 §1: the manifest lock is the only source list).
    if config == Config::Si && (genome.is_some() || genome_tag.is_some()) {
        bail!(
            "--genome/--genome-tag belong to --config engine; a CONFIG SI run names no \
             frozen path (00 §6.1)"
        );
    }
    let frozen = if config == Config::Engine {
        let (path, tag) = match (genome, genome_tag) {
            (Some(path), Some(tag)) => (path, parse_tag(tag)?),
            _ => {
                println!("frozen layer: CONFIG ENGINE names no pinned artifact (07 §1)");
                println!("no session was created");
                return Ok(ExitCode::SUCCESS);
            }
        };
        let manifest = layers_frozen::pin(PathBuf::from(path), tag);
        match layers_frozen::load(config, Some(&manifest)) {
            Err(error) => {
                println!("frozen layer: {error}");
                println!("no session was created");
                return Ok(ExitCode::SUCCESS);
            }
            Ok(layer) => Some(std::rc::Rc::new(layer)),
        }
    } else {
        None
    };

    let dir = PathBuf::from(run_dir);
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let run_id = dir
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| run_dir.to_string());

    let mut session = match &frozen {
        None => Session::new(seed)?,
        Some(layer) => Session::wake_engine(seed, layer.clone())?,
    };

    // 09 §8's bulk channel is a pure writer and **the ordinary loop is its reader**:
    // `G1.u.2` licenses the feeder only if a fed inbox reaches the byte-identical state
    // of the same lines typed one by one. The hearing is the loop's own method, so this
    // arm and a gate leg are the same code path and not two writers of one idea. A lane
    // that already arrived is earlier in logical time than the lines typed at this
    // invocation, so it goes first. The refusal is a branch and not an error string
    // (01 §9): the lane says what it is and no frame is applied.
    let inbox_path = dir.join("sense_inbox.siro");
    if inbox_path.exists() {
        let bytes =
            std::fs::read(&inbox_path).with_context(|| format!("read {}", inbox_path.display()))?;
        match session.hear_sense_inbox(&bytes)? {
            InboxDrain::Heard { frames, torn_tail_bytes } => {
                if !frames.is_empty() || torn_tail_bytes > 0 {
                    match torn_tail_bytes {
                        0 => println!("inbox: {} frame(s) heard from {}", frames.len(), run_id),
                        torn => println!(
                            "inbox: {} frame(s) heard from {run_id}; {torn} torn tail byte(s) \
                             were never written and are not heard (08 §2.2's dead-window row)",
                            frames.len()
                        ),
                    }
                }
                for heard in &frames {
                    println!(
                        "si@{}\t{}\t-> {}",
                        heard.report.t, heard.text, render_output(&heard.report.output)
                    );
                }
            }
            InboxDrain::Refused(refusal) => {
                println!("inbox refused: {refusal} — no frame was applied (01 §9)");
            }
        }
    }

    // 01 §6, 02 §11: every owner line of the transcript arrives through the one
    // sense step and settles there; the loop is driven only through `tick`.
    if let Some(path) = transcript {
        let text = std::fs::read_to_string(path).with_context(|| format!("read {path}"))?;
        for line in transcript_lines(&text) {
            let report = session.tick(line, Source::Owner)?;
            println!("si@{}\t{}\t-> {}", report.t, line, render_output(&report.output));
        }
    }
    let state_hash = session.state_hash()?;
    let stamp = build_stamp("V0");

    println!("run {run_id}");
    println!("birth_seed 0x{seed:016x}");
    println!("seed_format 0x{:04x}", session.core().seed_format);
    println!("config {}", config.as_str());
    if let Some(layer) = &frozen {
        // W.1's readout, and W.2's whole difference stated as one number: the
        // handle the mind's own arrivals start at.
        println!("genome_tag 0x{:016x}", layer.version_tag());
        println!("genome_points {}", layer.genome_points());
    }
    println!("temperament_digest 0x{:016x}", session.core().temperament_digest);
    println!(
        "build_variant {} graph_digest {}",
        stamp["build_variant"].as_str().unwrap_or("V0"),
        stamp["graph_digest"].as_str().unwrap_or("UNMEASURED (no Cargo.lock in the workspace)")
    );
    println!("state_hash {}", state_hash.hex());

    let snapshot_path = dir.join("SIst");
    session.save(&snapshot_path)?;

    // The file must carry the state that was hashed; a silent partial write
    // would be a false pass for every later replay.
    let reopened = match match &frozen {
        None => Session::open(&snapshot_path, seed)?,
        Some(layer) => Session::open_engine(&snapshot_path, seed, Some(layer.clone()))?,
    } {
        OpenOutcome::Opened(opened) => opened,
        OpenOutcome::Refused(refusal) => {
            bail!("the just-written SIst refuses its own session: {refusal}")
        }
    };
    if reopened.state_hash()?.digest() != state_hash.digest() {
        bail!("the just-written SIst does not re-hash to the run's state hash");
    }

    write_text(&dir.join("state_hash.txt"), &format!("{}\n", state_hash.hex()))?;
    write_text(&dir.join("fingerprint.txt"), &session.fingerprint())?;
    write_text(&dir.join("mindlog.json"), &session.mindlog_json(&run_id)?)?;
    Ok(ExitCode::SUCCESS)
}

/// OD-93 (owner ruling 2026-10-23, B7 batch OD-E): a variant is a build-graph state, pinned by
/// the deps_lock digest and not by a constants row, and the determinism clause
/// reads per-variant. So each report names the variant id it executed on beside
/// that digest; `si-cli` launches no build-graph variant, so its id is `V0`.
fn build_stamp(variant: &str) -> serde_json::Value {
    match si_cli::harness::Workspace::locate() {
        Ok(ws) => si_cli::harness::build_stamp(&ws, variant),
        Err(_) => serde_json::json!({ "build_variant": variant, "graph_digest": null }),
    }
}

fn parse_seed(text: &str) -> Result<u64> {
    let trimmed = text.trim();
    let (digits, radix) = match trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
        Some(hex) => (hex, 16),
        None => (trimmed, 10),
    };
    u64::from_str_radix(digits, radix)
        .map_err(|error| anyhow::anyhow!("`--seed {text}` is not a u64: {error}"))
}

/// The seal tag the manifest lock pins (07 §13). Same two forms as the seed: a
/// `0x…` hex word or a decimal one.
fn parse_tag(text: &str) -> Result<u64> {
    let trimmed = text.trim();
    let digits = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")).unwrap_or(trimmed);
    u64::from_str_radix(digits, 16)
        .map_err(|error| anyhow::anyhow!("`--genome-tag {text}` is not a seal tag (`0x…`): {error}"))
}

/// 07 §3's offline converter, M.0–M.11. It is the loader's own pipeline, so a
/// binary built without the feature has none: 00 §6.2's build guard says so at the
/// entry rather than half-building an artifact it cannot read back.
fn build_genome(manifest: &str, out: Option<&str>) -> Result<ExitCode> {
    #[cfg(feature = "engine")]
    {
        let manifest = PathBuf::from(manifest);
        let destination = out.map(PathBuf::from);
        let built = si_store::build::Builder::new(
            &manifest,
            destination.as_deref(),
        )
        .run()?;
        println!("manifest {}", manifest.display());
        println!("artifact {}", built.artifact.display());
        println!("version_tag 0x{:016x}", built.tag);
        println!("seal {}", built.seal.seal_md5);
        println!(
            "points {} edges {} rows {} batches {}",
            built.points, built.edges, built.rows, built.batches
        );
        println!("unmatched_points {}", built.unmatched_points);
        println!("asserts {}", built.asserts.join(" "));
        for path in &built.files {
            println!("published {}", path.display());
        }
        Ok(ExitCode::SUCCESS)
    }
    #[cfg(not(feature = "engine"))]
    {
        let _ = (manifest, out);
        bail!(
            "this binary carries no converter: build with `--features engine` (00 §6.2's \
             build guard — the converter and the loader are the same half of the build)"
        );
    }
}

fn write_text(path: &Path, text: &str) -> Result<()> {
    si_store::snapshot::write_atomic(path, text.as_bytes())
        .with_context(|| format!("write {}", path.display()))
}

/// Owner-facing transcript lines: a blank line, a `#` comment, the `seed …`
/// header and a `--- cold` marker are not utterances.
fn transcript_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim_end)
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with('#')
                && !line.starts_with("seed ")
                && !line.starts_with("---")
        })
        .collect()
}

fn render_output(output: &Output) -> String {
    match output {
        Output::Answer { surface, .. } => format!("answer: {surface}"),
        Output::Ask { surface, .. } => format!("ask: {surface}"),
        Output::Committed { row_id, shelf, branch } => {
            format!("commit row {row_id} shelf {shelf} via {branch}")
        }
        Output::Refused { reason } => format!("refused: {reason}"),
        Output::Inert { reason } => format!("inert: {reason}"),
    }
}

/// 09 §10: the `/teach` bulk feeder is an inbox writer and nothing else. It opens
/// exactly two files — the curriculum file and `sense_inbox.siro` — and reaches no
/// store, learn, walk or commit symbol, so its lines arrive through the same
/// step-1 read as a keystroke.
///
/// The lane's bytes are built by the loop's own writer ([`Session::encode_sense_inbox`]),
/// which is also what a `G1.u.2` leg feeds, so the equivalence is between one writer
/// and the reader rather than between two writers (09 §8, 05 §1).
fn feed_inbox(session_dir: &str, file: &str) -> Result<ExitCode> {
    let raw = std::fs::read_to_string(file).with_context(|| format!("read {file}"))?;
    let lines: Vec<&str> = raw
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();

    let inbox_path = PathBuf::from(session_dir).join("sense_inbox.siro");
    let prior = if inbox_path.exists() {
        std::fs::read(&inbox_path).with_context(|| format!("read {}", inbox_path.display()))?
    } else {
        Vec::new()
    };

    let run_id = PathBuf::from(session_dir)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| session_dir.to_string());
    // 01 §9: the transport pair is the registered owner id and the session's one
    // thread; nothing here invents an identity (OD-136 names who mints one, and no
    // adapter exists in this build to ask).
    let (bytes, total) = Session::encode_sense_inbox(&prior, &lines, &run_id)?;
    write_atomic(&inbox_path, &bytes)
        .with_context(|| format!("write {}", inbox_path.display()))?;
    println!(
        "fed {} line(s) into {} ({} frame(s) total)",
        lines.len(),
        inbox_path.display(),
        total
    );
    Ok(ExitCode::SUCCESS)
}

/// 00 §7: re-derive the derived tables into an export artifact. It opens `SIst`
/// under the recorded seed, re-hashes it, and writes `export.json` — a leaf the
/// build never reads back (`check_no_json_readback`).
fn export_session(session_dir: &str, seed_text: &str, format: Option<&str>) -> Result<ExitCode> {
    let seed = parse_seed(seed_text)?;
    let format = format.unwrap_or("json");
    if format != "json" {
        bail!("export format `{format}` is not built; only `json` re-derives the derived tables");
    }
    let state_path = PathBuf::from(session_dir).join("SIst");
    let session = match Session::open(&state_path, seed)? {
        OpenOutcome::Opened(session) => session,
        OpenOutcome::Refused(refusal) => bail!("SIst refuses its seed: {refusal}"),
    };
    let run_id = PathBuf::from(session_dir)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| session_dir.to_string());
    let state_hash = session.state_hash()?;
    let mindlog: serde_json::Value = serde_json::from_str(&session.mindlog_json(&run_id)?)?;
    let stamp = build_stamp("V0");
    let export = serde_json::json!({
        "birth_seed": format!("0x{seed:016x}"),
        "build_variant": stamp["build_variant"],
        "graph_digest": stamp["graph_digest"],
        "state_hash": state_hash.hex(),
        "fingerprint": session.fingerprint(),
        "mindlog": mindlog,
    });
    let export_path = PathBuf::from(session_dir).join("export.json");
    write_atomic(&export_path, format!("{export:#}\n").as_bytes())
        .with_context(|| format!("write {}", export_path.display()))?;
    println!("exported {} (re-derived to {})", export_path.display(), state_hash.hex());
    Ok(ExitCode::SUCCESS)
}

/// One lane's payload: each frame prefixed with its own length, the same layout
/// [`Session::encode_sense_inbox`] writes for the owner's inbox.
fn encode_frames(events: &[Event]) -> Result<Vec<u8>> {
    let mut payload = Vec::new();
    for event in events {
        let frame = event.encode();
        payload.extend_from_slice(&(frame.len() as u32).to_le_bytes());
        payload.extend_from_slice(&frame);
    }
    Ok(payload)
}

/// 08 §5 + 11 §B11: process B's launcher.
///
/// This function is the **only** place that opens the run directory's artifacts.
/// [00 §8] gives `si-vulkan` one writable file and [08 §2] gives its panels a read
/// list, and both hold only because the viewer is handed *values* — so the decode
/// lives here, in the binary that is allowed the schema crates, and reaches the
/// panels through [`si_vulkan::surface::Handover`].
///
/// It exits `0` with or without a window: [02 §10]'s specified behaviour on a
/// machine that refuses one is `Outcome::Degraded`, and the DoD's "the loop never
/// notices" is honoured by writing a log rather than by pretending a frame was seen.
fn view_session(session: Option<&str>, exit_after_frames: Option<u32>) -> Result<ExitCode> {
    #[cfg(not(feature = "vulkan"))]
    {
        let _ = (session, exit_after_frames);
        bail!(
            "this binary carries no viewer: build with `--features vulkan` \
             (00 §6.2's build-guard shape — a viewer half that is compiled out says so \
             instead of drawing nothing)"
        )
    }
    #[cfg(feature = "vulkan")]
    {
        use si_vulkan::surface::{Live, Options, Session as Observed, Viewer};
        let text = session.ok_or_else(|| {
            anyhow::anyhow!(
                "view needs `--session runs/<id>`: process B owns no state and reads one \
                 run directory (08 §1's first law)"
            )
        })?;
        let dir = PathBuf::from(text);
        if !dir.is_dir() {
            bail!("--session {} is not a run directory", dir.display());
        }
        let run_id = dir
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| text.to_string());

        // 08 §1's OBSERVER-ONLY law as a byte comparison rather than a promise: the
        // run's own hash is read before the window opens and again after it closes.
        let hash_path = dir.join("state_hash.txt");
        let before = read_state_hash(&hash_path)?;

        let options = Options {
            session: Observed { run_dir: dir.clone(), run_id: run_id.clone() },
            extent: (1280, 800),
            exit_after_frames: exit_after_frames.unwrap_or(120) as u64,
        };
        // The log names the lane the panels were built from, because a live
        // `stream.siro` tail and a `mindlog.json` re-frame are different witnesses and
        // a reader must not be able to confuse them (**OD-150**). This is one pre-flight
        // read for the note only; the per-frame tail is the closure below, and the kind
        // of source cannot change inside one view because a run publishes its lane once,
        // when it settles.
        let (lane, re_frame) = lane_bytes(&dir)?;
        let note = if lane.is_empty() {
            "lane none (the session has published no frames yet)".to_string()
        } else if re_frame {
            format!("lane mindlog.json re-framed {} bytes", lane.len())
        } else {
            format!("lane stream.siro live tail {} bytes", lane.len())
        };
        let source = {
            let dir = dir.clone();
            move || handover(&dir)
        };
        let outcome = Viewer::new(Live::new(source), options).run()?;

        let after = read_state_hash(&hash_path)?;
        let unchanged = before == after;
        let log = format!(
            "run {run_id}\n{}\n{note}\nstate_hash before  {before}\nstate_hash after   {after}\n\
             state_hash unchanged {unchanged}\n",
            outcome.summary()
        );
        write_atomic(&dir.join("viewer.log"), log.as_bytes())
            .with_context(|| format!("write {}", dir.join("viewer.log").display()))?;
        print!("{log}");
        if !unchanged {
            bail!(
                "the viewer changed `state_hash.txt` — 08 §1's OBSERVER-ONLY law is broken, \
                 and a run that survived it is not a pass"
            );
        }
        Ok(ExitCode::SUCCESS)
    }
}

/// The state hash as ONE log value, or a name for its absence. A run with N settled
/// sections writes an N-line `state_hash.txt` (measured: a G13 run's hash is three
/// lines), and a key-per-line log cannot hold a value that spans lines —
/// `check_headless_run` reads one line per key, so the digests are folded to spaces
/// here and the file is folded the same way there. The digests stay in file order and
/// byte-identical; only the separator between them changes.
///
/// A run directory that never wrote a hash is a run that never settled, and the viewer
/// says *absent* rather than drawing a hash it did not read (§7's rule for an
/// unmeasured cell, applied to the log).
#[cfg(feature = "vulkan")]
fn read_state_hash(path: &Path) -> Result<String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok("<absent>".to_string()),
        Err(error) => Err(error).with_context(|| format!("read {}", path.display())),
    }
}

/// One poll of the handover: the lane's bytes and the decoded artifacts.
#[cfg(feature = "vulkan")]
fn handover(dir: &Path) -> Result<(Vec<u8>, si_vulkan::panels::Artifacts, bool)> {
    let profile =
        std::env::var_os("SI_PROFILE").is_some_and(|value| value != std::ffi::OsString::from("0"));
    let (lane, reconstructed) = lane_bytes(dir)?;
    let artifacts = artifacts(dir, reconstructed)?;
    Ok((lane, artifacts, profile))
}

/// The frames the panels draw.
///
/// `stream.siro` is [08 §2]'s output lane and the first thing read. A CONFIG SI run
/// written by `si-cli run` has not published one yet — the loop's lane writer is
/// not in this stage's creates-list — so the launcher falls back to `mindlog.json`,
/// which §2's table already names as the source of kinds 0–16, and re-frames **the
/// event lines exactly as the loop wrote them**. The fallback is marked
/// [`si_vulkan::panels::RECONSTRUCTED`] rather than passed off as a live tail
/// (**OD-150**).
#[cfg(feature = "vulkan")]
fn lane_bytes(dir: &Path) -> Result<(Vec<u8>, bool)> {
    let lane_path = dir.join("stream.siro");
    if lane_path.exists() {
        let bytes =
            std::fs::read(&lane_path).with_context(|| format!("read {}", lane_path.display()))?;
        return Ok((bytes, false));
    }
    let mindlog = dir.join("mindlog.json");
    if !mindlog.exists() {
        // No lane and no mindlog is a session that has committed nothing yet, which is
        // §2.2's late-start row read at its limit: the viewer renders from the last `SIst`
        // plus the frames after it, and here there are none.
        return Ok((Vec::new(), false));
    }
    let raw =
        std::fs::read_to_string(&mindlog).with_context(|| format!("read {}", mindlog.display()))?;
    let document: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("decode {}", mindlog.display()))?;
    let thoughts = document["thoughts"].as_array().cloned().unwrap_or_default();
    let mut events = Vec::with_capacity(thoughts.len());
    for thought in &thoughts {
        let field = |name: &str| thought[name].as_i64().unwrap_or_default();
        events.push(Event {
            seq: field("seq") as u64,
            t: field("t") as i32,
            source: field("source") as u8,
            kind: field("kind") as u8,
            body: thought["body"].as_str().unwrap_or_default().as_bytes().to_vec(),
        });
    }
    let header = serde_json::json!({
        "lane": "mindlog",
        "frames": events.len(),
        "run_id": document["run_id"],
    });
    let container = Container::new(Format::StreamRecord, header, encode_frames(&events)?);
    Ok((container.encode()?, true))
}

/// §8's non-lane read list, decoded by the launcher and forwarded as values.
#[cfg(feature = "vulkan")]
fn artifacts(dir: &Path, reconstructed: bool) -> Result<si_vulkan::panels::Artifacts> {
    use si_core::record::{Edge, Record, EDGE_BYTES, RECORD_BYTES};
    use si_core::shelves::SHELVES;
    use si_store::snapshot::Snapshot;
    use si_vulkan::panels::{Artifacts, Row};

    let mut out = Artifacts { reconstructed, ..Default::default() };

    let state_path = dir.join("SIst");
    if state_path.exists() {
        let bytes =
            std::fs::read(&state_path).with_context(|| format!("read {}", state_path.display()))?;
        let snapshot = Snapshot::decode(&bytes)
            .with_context(|| format!("decode {}", state_path.display()))?;
        // O2 KNOW: one count per **registry seat**, reserved ones included, because
        // §6 O2's rule is that a reserved shelf renders rather than being skipped.
        let mut counts: std::collections::BTreeMap<u16, u64> = std::collections::BTreeMap::new();
        for record in snapshot.rows.records() {
            if record.len() != RECORD_BYTES {
                continue;
            }
            let row = Record::decode(record)?;
            *counts.entry(row.envelope.shelf_x).or_default() += 1;
        }
        out.shelf_rows = SHELVES
            .iter()
            .map(|entry| (entry.shelf_id, counts.get(&entry.shelf_id).copied().unwrap_or(0)))
            .collect();
        // E6 BAKE: the durable, write-once half of an edge, from the edge arena.
        // `a_at` and `thought_id` live in the stamp list and reach the panel only
        // when a lane frame carries them, so the row names what it actually holds.
        for (index, record) in snapshot.edges.records().enumerate() {
            if record.len() != EDGE_BYTES {
                continue;
            }
            let edge = Edge::decode(record)?;
            if edge.bake != 0 {
                out.bakes.push(Row::new(
                    format!("edge {}→{}", index * EDGE_BYTES, edge.to_point),
                    format!(
                        "bake={} kind={} weight={} a_at=absent thought_id=absent",
                        edge.bake, edge.kind, edge.weight
                    ),
                ));
            }
        }
        // O1 and the sleep counters: [01 §13] item 9's core block, forwarded.
        let core = snapshot.header.core;
        out.core = vec![
            Row::new("birth_seed", format!("0x{:016x}", core.birth_seed)),
            Row::new("seed_format", format!("0x{:04x}", core.seed_format)),
            Row::new("temperament_digest", format!("0x{:016x}", core.temperament_digest)),
            Row::new("deriv_order_len", core.deriv_order_len.to_string()),
            Row::new("sleeps_completed", core.sleeps_completed.to_string()),
            Row::new("sleeps_interrupted", core.sleeps_interrupted.to_string()),
            Row::new("last_slept_at_t", core.last_slept_at_t.to_string()),
            Row::new("sleep_pass_mask", core.sleep_pass_mask.to_string()),
        ];
    }

    let sleep_path = dir.join("sleep.jsonl");
    if sleep_path.exists() {
        let raw = std::fs::read_to_string(&sleep_path)
            .with_context(|| format!("read {}", sleep_path.display()))?;
        out.sleep_lines = raw.lines().filter(|line| !line.trim().is_empty()).map(str::to_string).collect();
    }

    let fingerprint_path = dir.join("fingerprint.txt");
    if let Ok(text) = std::fs::read_to_string(&fingerprint_path) {
        out.fingerprint = text.lines().next().map(str::to_string);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A directory under this process, named for the test, emptied before use.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("si-feed-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    /// The texts the lane on disk carries, in order, plus its cut tail. Reading it
    /// back goes through the loop's own reader — the same one a wake uses — so the
    /// assertion is about what the ordinary loop would hear, not about our bytes.
    fn heard(dir: &Path) -> Result<(Vec<String>, usize)> {
        let bytes = std::fs::read(dir.join("sense_inbox.siro"))?;
        match Session::read_sense_inbox(&bytes)? {
            si_session::InboxOutcome::Accepted { frames, torn_tail_bytes } => Ok((
                frames.into_iter().map(|frame| frame.text).collect(),
                torn_tail_bytes,
            )),
            si_session::InboxOutcome::Refused(refusal) => {
                bail!("the lane refuses its reader: {refusal:?}")
            }
        }
    }

    /// 09 §10's first half, run as the producer actually runs: a curriculum file in,
    /// one `sense_inbox.siro` out and no other file, with the header comment and the
    /// blank lines dropped and each remaining line one whole frame in order.
    #[test]
    fn the_feeder_writes_the_lane_and_nothing_else() -> Result<()> {
        let dir = scratch("write");
        let file = dir.join("curriculum.txt");
        std::fs::write(
            &file,
            "# a header the owner left\n\n/teach what is apple => a fruit   \n\ncarriage return at the end\n",
        )?;
        feed_inbox(dir.to_str().unwrap(), file.to_str().unwrap())?;

        let mut names: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            names.push(entry?.file_name().to_string_lossy().to_string());
        }
        names.sort();
        assert_eq!(names, ["curriculum.txt", "sense_inbox.siro"], "the importer opened a third file");

        let (texts, torn) = heard(&dir)?;
        assert_eq!(
            texts,
            ["/teach what is apple => a fruit", "carriage return at the end"],
            "a dropped line, a kept comment or a re-ordered frame means the feeder is not a loader"
        );
        assert_eq!(torn, 0, "a lane the writer finished still reads as cut");
        Ok(())
    }

    /// The lane is append-only from the owner's side: a second feed walks the frames
    /// already on disk and puts the new lines behind them, never in front and never
    /// over them.
    #[test]
    fn a_second_feed_appends_behind_the_lane_already_written() -> Result<()> {
        let dir = scratch("append");
        let first = dir.join("first.txt");
        let second = dir.join("second.txt");
        std::fs::write(&first, "/teach what is apple => a fruit\n/teach what is znark => a floop\n")?;
        std::fs::write(&second, "/teach what is floop => a znark\n")?;
        feed_inbox(dir.to_str().unwrap(), first.to_str().unwrap())?;
        feed_inbox(dir.to_str().unwrap(), second.to_str().unwrap())?;

        let (texts, torn) = heard(&dir)?;
        assert_eq!(
            texts,
            [
                "/teach what is apple => a fruit",
                "/teach what is znark => a floop",
                "/teach what is floop => a znark",
            ],
            "the second feed reordered or replaced the lane"
        );
        assert_eq!(torn, 0);
        Ok(())
    }

    /// 09 §8's strict-walk half on the feeder's own handle: a lane whose last frame
    /// never finished is a named error and **nothing is written**. A feeder that
    /// wrote past the hole would bury the lines behind it, and the reader's tolerance
    /// for a cut tail is a viewer's license, not a writer's.
    #[test]
    fn a_torn_lane_refuses_the_feeder_and_stays_exactly_as_it_was() -> Result<()> {
        let dir = scratch("torn");
        let one = dir.join("one.txt");
        std::fs::write(&one, "/teach what is apple => a fruit\n")?;
        feed_inbox(dir.to_str().unwrap(), one.to_str().unwrap())?;

        let lane = dir.join("sense_inbox.siro");
        let whole = std::fs::read(&lane)?;
        // Cut the last frame short by three bytes: the length it declares now runs
        // past the payload, which is exactly what a killed mid-append write leaves.
        std::fs::write(&lane, &whole[..whole.len() - 3])?;

        let two = dir.join("two.txt");
        std::fs::write(&two, "/teach what is znark => a floop\n")?;
        let outcome = feed_inbox(dir.to_str().unwrap(), two.to_str().unwrap())
            .err()
            .context("a torn lane was accepted by a feeder, so lines hid behind the hole")?;
        // The name is part of the law, not a nicety: 09 §8 says a torn lane is a
        // **named** error, so a silent repair that re-wrote the lane behind the hole
        // would read as a fail here rather than as a pass with an unnamed cause.
        let text = format!("{outcome:#}");
        assert!(
            text.contains("the lane on disk is torn"),
            "the feeder refused for a reason other than the torn lane: {text}"
        );

        assert_eq!(
            std::fs::read(&lane)?,
            whole[..whole.len() - 3].to_vec(),
            "the refused feed still wrote to the lane"
        );
        Ok(())
    }
}
