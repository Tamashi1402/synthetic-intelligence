//! si-harness — engineering/05 §1. The only binary that opens `gates/`.
//! `check` is live from B0; `run`, `replay`, `verify` and `sweep` arrive with
//! the stages that give them a loop to drive.
//!
//! The loop is driven only through `Session::tick` and `Session::idle_tick`:
//! this binary owns no branch of it, pre-seeds nothing, and never reads a run's
//! JSON back as an input (R-33).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use si_cli::harness::runner::{Launch, Run};
use si_cli::harness::scenario::Scenario;
use si_cli::harness::sweep;
use si_cli::harness::{self, Stage, Workspace};

#[derive(Parser)]
#[command(name = "si-harness", about = "SI gate harness (05 §1)")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the static checks live at a stage and write gates/checks/stage-Bn.json.
    Check {
        /// Stage label, B0..B13.
        #[arg(long, conflicts_with = "all")]
        stage: Option<String>,
        /// Run every static check (the B13 set).
        #[arg(long)]
        all: bool,
        /// Override the report path, relative to the workspace root.
        #[arg(long)]
        report: Option<String>,
    },
    /// Execute a scenario: the transcript is fed line-by-line as sense events.
    Run {
        #[arg(long)]
        gate: String,
        #[arg(long, default_value = "V0")]
        variant: String,
        /// The birth seed. Required: a run with an unnamed individual cannot be
        /// cited by a box validation (05 §1, G14.f.1).
        #[arg(long, required = true)]
        seed: String,
        #[arg(long)]
        scenario: String,
        #[arg(long, default_value = "si")]
        config: String,
        /// 05 §4's V7 row: the frozen artifact this run loads, relative to the
        /// workspace. Only `--config engine` reads it; a CONFIG SI run that names
        /// one is refused rather than quietly ignored (00 §6).
        #[arg(long)]
        genome: Option<String>,
        /// The seal tag the manifest lock pins for that artifact (`0x…`).
        #[arg(long)]
        genome_tag: Option<String>,
        /// Refuse a run that still carries an UNMEASURED_PLACEHOLDER (11 §1 D5).
        #[arg(long)]
        strict_consts: bool,
        /// A sweep's declared constant, `NAME=VALUE`, repeatable.
        #[arg(long = "constant")]
        constants: Vec<String>,
    },
    /// Re-execute a recorded transcript into a fresh state and diff the hash.
    Replay {
        /// The run directory, relative to the workspace root.
        #[arg(long)]
        session: String,
        #[arg(long, required = true)]
        seed: String,
    },
    /// Evaluate an assert spec against an existing run directory. A pure reader.
    Verify {
        #[arg(long)]
        session: String,
        #[arg(long)]
        gate: Option<String>,
    },
    /// Run a gate's sweep runner (05 §4).
    Sweep {
        #[arg(long)]
        gate: String,
        #[arg(long, required = true)]
        seed: String,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("si-harness: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode> {
    let ws = Workspace::locate()?;
    match Args::parse().command {
        Command::Check { stage, all, report } => check(&ws, stage, all, report),
        Command::Run { gate, variant, seed, scenario, config, genome, genome_tag, strict_consts, constants } => {
            let seed = si_cli::harness::scenario::parse_seed(&seed)
                .with_context(|| format!("`{seed}` is not a birth seed"))?;
            // 00 §6.1: CONFIG SI forms no path to a frozen layer. A launch that
            // names one is a mistake, not a config, so it ends here rather than
            // being dropped on the floor.
            if config != "engine" && (genome.is_some() || genome_tag.is_some()) {
                bail!("--genome/--genome-tag belong to --config engine; a CONFIG SI run names no frozen path (00 §6)");
            }
            let genome = match &genome {
                Some(relative) => Some(ws.path(relative)),
                None => None,
            };
            let genome_tag = match &genome_tag {
                Some(text) => Some(
                    u64::from_str_radix(text.trim_start_matches("0x").trim_start_matches("0X"), 16)
                        .with_context(|| format!("`{text}` is not a seal tag (`0x…` or hex)"))?,
                ),
                None => None,
            };
            if strict_consts {
                let open = si_core::consts::placeholder_count();
                if open > 0 {
                    bail!(
                        "--strict-consts refuses this run: {open} swept SPECIES row(s) still carry \
                         UNMEASURED_PLACEHOLDER (11 §1 D5)"
                    );
                }
            }
            let scenario = Scenario::load(&ws, &scenario)?;
            let mut constants = parse_constants(&constants)?;
            // 05 §18's prerequisite is that the cadence fires **at least twice**,
            // and `SLEEP_EVERY_QUIET_TICKS`'s registry placeholder is the midpoint
            // of its own box, which no gate scenario of §3's long-session shape can
            // reach. A `G13` run with no declared list therefore launches at the six
            // sleep rows' own `lo` — the point `runner::sleep_lo_point` reads out of
            // the registry, inventing no value — and records it in its own identity
            // (**OD-95**). A launch that names its constants still wins.
            if gate.as_str() == "G13" && constants.is_empty() {
                constants = si_cli::harness::runner::sleep_lo_point();
                println!("  G13 launch point: the six sleep rows at their registry `lo` (05 §18's prerequisite, OD-95)");
            }
            // 05 §23.1 / 12 §6.4: the fixture is run "across enough distinct cold
            // frames to clear `R_MIN` at the channel floor", so the three
            // crystallize rows launch at their registry `lo` and the two frame caps
            // at the smallest values that let one ghost and one role-graph exist.
            // `CRYSTALLIZE = on` is the state the gate grades, so the gate launches
            // at it (OD-95's precedent, OD-134's reading). A launch that names its
            // constants still wins.
            if gate.as_str() == "G15" && constants.is_empty() {
                constants = si_cli::harness::runner::g15_launch_point();
                println!("  G15 launch point: R_MIN/R_ASK/DELTA_MIN_CRYSTALLIZE at `lo`, GHOST_MAX = 1, SHAPE_MAX = 4, CRYSTALLIZE = on (05 §23.1, 12 §6.4, OD-95)");
            }
            // 05 §23.2's two settings: the five table rows at `lo` and the fifth
            // ignition term at its smallest armed point (`lo + grid`), because at
            // `lo` it is 0 and the term would be absent. A launch that names its
            // constants still wins (12 §6.4's "the gate launches at the point it
            // grades").
            if gate.as_str() == "G16" && constants.is_empty() {
                constants = si_cli::harness::runner::g16_launch_point();
                println!("  G16 launch point: LAMBDA_SILT/SILT_LEVEL_MAX/SILT_BUMP/THETA_SILT/SILT_MAX at `lo`, IGN_WEIGHT_SILT at `lo + grid` (12 §6.4, OD-102)");
            }
            // 05 §23.3 / 12 §6.5: the traversal ruling is on and the tug is at
            // the smallest armed point on its grid, because at `lo` it is 0 and
            // the derived index would light nothing; RELIGHT_MAX is at its own
            // floor. A launch that names its constants still wins.
            if gate.as_str() == "G17" && constants.is_empty() {
                constants = si_cli::harness::runner::g17_launch_point();
                println!("  G17 launch point: EPISODIC_TRAVERSAL = on, PROV_LINK_TUG at `lo + grid`, RELIGHT_MAX at `lo` (12 §6.5, OD-110)");
            }
            // 05 §23.4 / 12 §6.4: the three shape bars launch at their registry
            // `lo`, both skills rulings `on` (the gate grades the mechanism, not
            // the switch), and `SHAPE_MAX` at §23.1's declared 4 because at its
            // own `lo` the fold builds no role-graph and the detector is blind.
            // The crystallize rows ride along so G18(f)'s forwarded frames are
            // seen by §29's fold in the same run — one test, not two claims
            // (OD-121). A launch that names its constants still wins.
            if gate.as_str() == "G18" && constants.is_empty() {
                constants = si_cli::harness::runner::g18_launch_point();
                println!("  G18 launch point: R_MIN_SHAPE/DELTA_MIN_SHAPE/K_FILLER at `lo`, SKILLS = on, SKILL_COMPOSITION = on, SHAPE_MAX = 4, plus G15's crystallize point (05 §23.4, 12 §6.4, OD-121)");
            }
            // 05 §2's V13 row / §23.5: `WORLD_TIERS` at its registry `hi`
            // (`granted`), because the gate grades the class arm and the arm only
            // ever sees a grant when the ruling is installed on. The skills and
            // crystallize rows ride along from G18's point: legs (d) and (f) need
            // the `origin = 2` asks the shape detector opens, and a leg that needs
            // a mechanism launches at it (OD-121's reading, one test not two
            // claims). The grant table itself is the scenario's owner line, never a
            // constant (OD-136: no adapter exists). A launch that names its
            // constants still wins.
            if gate.as_str() == "G19" && constants.is_empty() {
                constants = si_cli::harness::runner::g19_launch_point();
                println!("  G19 launch point: WORLD_TIERS = `granted` (registry `hi`), plus G18's skills point so legs (d)/(f) have an origin-2 ask to route (05 §2's V13, §23.5, 12 §6.4, OD-121)");
            }
            // 05 §2's V14 row and 13 §6.1: the voice is the mechanism the gate
            // grades, so `TONGUE` installs `on` and `PHRASE_MAX_STEPS` installs at
            // 32 — the first point on its own grid at which the *lowest-spread*
            // individual can buy one extra chunk (the budget is
            // `steps × STYLE_SPREAD >> 12`, `STYLE_SPREAD`'s grid step is 128). At
            // the registry `lo` of 1 every tick refuses, and a gate that launches
            // there grades its own absence (OD-95's floor discipline; the refusal
            // itself is G20(c)/(g)'s measured evidence, not the launch point). A
            // launch that names its constants still wins.
            if gate.as_str() == "G20" && constants.is_empty() {
                constants = si_cli::harness::runner::g20_launch_point();
                println!("  G20 launch point: TONGUE = on, PHRASE_MAX_STEPS = 32 (05 §2's V14, 13 §6.1, OD-95)");
            }
            let request = si_cli::harness::runner::RunRequest {
                gate,
                variant,
                config,
                seed,
                scenario,
                constants,
                // 05 §18(g)'s four interrupt runs are built in-process by the gate
                // legs; the CLI has no sleep verb and so no interrupt flag.
                sleep_interrupt: None,
                genome,
                genome_tag,
                run_dir: None,
            };
            let run = Run::execute(request, &ws)?;
            report_run(&ws, &run)
        }
        Command::Replay { session, seed } => {
            let seed = si_cli::harness::scenario::parse_seed(&seed)?;
            replay(&ws, &session, seed)
        }
        Command::Verify { session, gate } => verify(&ws, &session, gate),
        Command::Sweep { gate, seed } => {
            let seed = si_cli::harness::scenario::parse_seed(&seed)?;
            sweep::run(&ws, &gate, seed)
        }
    }
}

fn parse_constants(values: &[String]) -> Result<BTreeMap<String, i64>> {
    let mut out = BTreeMap::new();
    for item in values {
        let (name, value) = item
            .split_once('=')
            .with_context(|| format!("`{item}` is not NAME=VALUE"))?;
        let value: i64 = value
            .trim()
            .parse()
            .with_context(|| format!("`{item}` has a non-integer value"))?;
        out.insert(name.trim().to_string(), value);
    }
    Ok(out)
}

fn check(ws: &Workspace, stage: Option<String>, all: bool, report_override: Option<String>) -> Result<ExitCode> {
    let (report, stage) = if all {
        (harness::run_all(ws)?, Stage::union())
    } else {
        let text = stage.ok_or_else(|| anyhow::anyhow!("give --stage Bn or --all"))?;
        let stage = Stage::parse(&text)
            .ok_or_else(|| anyhow::anyhow!("unknown stage {text:?}; expected B0..{}", Stage::union().label()))?;
        (harness::run_stage(stage, ws)?, stage)
    };
    let path: PathBuf = match report_override {
        Some(relative) => ws.path(&relative),
        None => ws.path(&harness::report_path(stage).to_string_lossy()),
    };
    harness::write_report(&report, &path)?;

    println!(
        "stage {} — {} checks, {}",
        report.stage,
        report.enabled.len(),
        if report.pass { "PASS" } else { "FAIL" }
    );
    for record in &report.enabled {
        println!(
            "  {:<34} {}",
            record.id,
            if record.verdict == "pass" { "pass" } else { "FAIL" }
        );
    }
    println!("  placeholders: {}", report.placeholders.len());
    println!("  disabled: {}", report.disabled.len());
    for record in &report.disabled {
        println!("    {:<34} turns on at {}", record.id, record.turns_on_at);
    }
    println!("  report: {}", ws.relative(&path));
    Ok(if report.pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

/// The run's own console line: the verdict, the failed legs and the count of
/// unresolved ones. No magnitude leaves here that the report does not carry.
fn report_run(ws: &Workspace, run: &Run) -> Result<ExitCode> {
    let report = harness::gates::read_gate_report(&run.dir)?;
    let pass = report["pass"].as_bool().unwrap_or(false);
    println!(
        "gate {} variant {} seed 0x{:016x} — {}",
        run.request.gate,
        run.request.variant,
        run.request.seed,
        if pass { "PASS" } else { "FAIL" }
    );
    for criterion in report["criteria"].as_array().into_iter().flatten() {
        let verdict = criterion["verdict"].as_str().unwrap_or("?");
        if verdict != "pass" {
            println!(
                "  {:<12} {:<12} {}",
                criterion["id"].as_str().unwrap_or("?"),
                verdict,
                criterion["observed"]["why"].as_str().unwrap_or("")
            );
        }
    }
    let unresolved = report["unresolved_legs"]
        .as_array()
        .map(Vec::len)
        .unwrap_or_default();
    println!("  legs: {}  unresolved: {}", report["criteria"].as_array().map(Vec::len).unwrap_or_default(), unresolved);
    println!("  sweeps cited: {}", report["sweeps_cited"].as_array().map(Vec::len).unwrap_or_default());
    println!("  run dir: {}", ws.relative(&run.dir));
    Ok(if pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

/// 05 §1 `replay`: the recorded `transcript.txt` and nothing else, re-executed
/// at the launch seed, diffed against `state_hash.txt` at tolerance 0.
fn replay(ws: &Workspace, relative: &str, seed: u64) -> Result<ExitCode> {
    let dir = ws.path(relative);
    let recorded = read_hash_lines(&dir.join("state_hash.txt"))?;
    // OD-93 (owner ruling 2026-10-23, B7 batch OD-E): determinism is per-variant, so the replay
    // re-executes at the variant the run recorded and reports both the variant id
    // and the build-graph digest it itself ran on. A run directory with no recorded
    // variant is replayed as the registry's own values (`V0`) and says so.
    let recorded_report = harness::gates::read_gate_report(&dir).ok();
    let recorded_variant = recorded_report
        .as_ref()
        .and_then(|report| {
            report["build_identity"]["build_variant"]
                .as_str()
                .map(str::to_string)
        })
        .unwrap_or_else(|| "V0 (unrecorded: no gate_report.json)".to_string());
    let stamp = harness::build_stamp(ws, &recorded_variant);
    let graph_digest = stamp["graph_digest"]
        .as_str()
        .unwrap_or("UNMEASURED (no Cargo.lock in the workspace)")
        .to_string();
    // The whole launch state the run recorded, read back by the harness's own
    // decoder: an interrupted night replays as the same partial night
    // (05 §18(g).3), a run at a swept point replays at that point rather than at
    // the registry placeholder, and a CONFIG ENGINE directory replays against the
    // corpus it was born into instead of waking an SI mind with a different handle
    // space (00 §6).
    let launch = match recorded_report.as_ref() {
        Some(report) => Launch::recorded(report, ws),
        None => Launch { variant: "V0".to_string(), ..Launch::default() },
    };
    let replayed = Run::replay_transcript(&dir, seed, &launch)?;
    if recorded.len() != replayed.len() {
        bail!(
            "the replay produced {} sections; the run recorded {}",
            replayed.len(),
            recorded.len()
        );
    }
    let mut differed = 0usize;
    for (index, (a, b)) in recorded.iter().zip(&replayed).enumerate() {
        if a == b {
            println!("  section {index}: identical");
        } else {
            differed += 1;
            println!("  section {index}: DIFFERS\n    recorded  {a}\n    replayed  {b}");
        }
    }
    println!(
        "replay of {} at seed 0x{:016x} — {} (tolerance 0)",
        relative,
        seed,
        if differed == 0 { "PASS" } else { "FAIL" }
    );
    println!("  variant {recorded_variant} · graph digest {graph_digest}");
    Ok(if differed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn read_hash_lines(path: &Path) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read {} (the replay's counterpart)", path.display()))?;
    let lines = text.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_string).collect();
    Ok(lines)
}

/// 05 §1 `verify`: a pure reader of `gate_report.json`. It never re-runs the
/// loop, so a verify of an old directory grades the arithmetic that made it.
fn verify(ws: &Workspace, relative: &str, gate: Option<String>) -> Result<ExitCode> {
    let dir = ws.path(relative);
    let report = harness::gates::read_gate_report(&dir)?;
    if let Some(gate) = &gate {
        let stated = report["gate"].as_str().unwrap_or_default();
        if stated != gate.as_str() {
            bail!("{} reports gate {stated}, not {gate}", relative);
        }
    }
    let pass = report["pass"].as_bool().unwrap_or(false);
    println!(
        "verify {} — gate {} pass={} variant_pair={}",
        relative,
        report["gate"].as_str().unwrap_or("?"),
        pass,
        report["variant_pair"].as_str().unwrap_or("?")
    );
    for criterion in report["criteria"].as_array().into_iter().flatten() {
        println!(
            "  {:<12} {:<12} {}",
            criterion["id"].as_str().unwrap_or("?"),
            criterion["verdict"].as_str().unwrap_or("?"),
            criterion["leg"].as_str().unwrap_or("")
        );
    }
    Ok(if pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}
