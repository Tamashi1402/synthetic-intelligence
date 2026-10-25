//! 05 §4: the sweep-runner protocol. A sweep enumerates the grid over the
//! declared legal ranges for the constants its gate owns, runs the gate's
//! scenario at each point, evaluates the design's objective, and publishes the
//! whole curve beside the winner. A point chosen off the curve is a named fail,
//! so this module never writes a winner it could not reach by enumeration, and it
//! never substitutes a sub-grid for the one the registry declares.
//!
//! The runner is also the only place that may *refuse*: a refusal is written as
//! the artifact's own record, with the axes, the product and the question the
//! spec has to answer — not as a curve with one row in it.
//!
//! **OD-83's ruling is validation, not search.** A winner is re-run with both box
//! corners at every `seed_panel` member (§4.1), and a failing panel point narrows
//! the box in engineering/03 — it never re-samples the panel. The search step is
//! §4 step 1 as written, and several declared legal ranges are open bounds
//! (`> 0`, `≥ 0`), so the registry's `hi` is the type maximum rather than a design
//! ceiling. That residual question is **OD-192**; until it is ruled, every
//! curve-bearing runner writes its refusal and names it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Result, bail};
use serde_json::{Value, json};
use si_core::consts::{Class, ConstRow};

use super::Workspace;

/// The winner-searching and panel runners. §4 lists **twelve artifacts / ten
/// runners**; the v2 arc's four (G15–G18) and the tongue's one (G20) join them,
/// and §23.6 calls the resulting set fifteen. Each entry is one artifact the law
/// names: G13 owns two (cadence and dimming), and G11 owns two (the median and
/// the composition cap) because they answer different questions on one build.
///
/// G12's trio is **observation-only** (§17) and is written by the G12 evaluator,
/// not by this module: `run` dispatches it to `observe_g12`, which runs the gate
/// and verifies the three files rather than inventing a fourth.
const RUNNERS: [(&str, &str, &str); 14] = [
    ("G2", "gates/G2/constants_sweep.json", "constants"),
    ("G8", "gates/G8/budget_sweep.json", "budget"),
    ("G9", "gates/G9/arousal_sweep.json", "arousal"),
    ("G10", "gates/G10/ignition_sweep.json", "ignition"),
    ("G11", "gates/G11/median_sweep.json", "median"),
    ("G11", "gates/G11/composition_sweep.json", "composition"),
    ("G13", "gates/G13/sleep_sweep.json", "sleep"),
    ("G13", "gates/G13/dimming_sweep.json", "dimming"),
    ("G14", "gates/G14/seed_panel.json", "panel"),
    ("G15", "gates/G15/recurrence_sweep.json", "recurrence"),
    ("G16", "gates/G16/silt_sweep.json", "silt"),
    ("G17", "gates/G17/prov_sweep.json", "prov"),
    ("G18", "gates/G18/shape_sweep.json", "shape"),
    ("G20", "gates/G20/phrase_sweep.json", "phrase"),
];

/// The G12 trio 05 §17 declares observation-only. Named here so `sweep --gate
/// G12` can verify them without pretending they are curves.
const G12_ARTIFACTS: [&str; 3] = [
    "gates/G12/absence_diff.json",
    "gates/G12/rejection_reasons.json",
    "gates/G12/speak_gate_log.jsonl",
];

/// A swept axis: the registry's own row, its declared box and the point count
/// that box enumerates to on its own grid.
#[derive(Clone, Debug)]
pub struct Axis {
    pub name: &'static str,
    pub home: &'static str,
    pub class: Class,
    pub lo: i64,
    pub hi: i64,
    pub grid: i64,
    pub points: u128,
    /// 03 declares this row's range as an open bound (`≥ 0`, `> 0`); the `hi`
    /// the registry carries is the type's maximum, not a design ceiling.
    pub open_ceiling: bool,
}

impl Axis {
    fn of(row: &'static ConstRow) -> Self {
        let points = 1 + ((row.hi - row.lo) / row.grid.max(1)) as u128;
        Self {
            name: row.name,
            home: row.home,
            class: row.class,
            lo: row.lo,
            hi: row.hi,
            grid: row.grid,
            points,
            open_ceiling: row.hi == i64::from(u16::MAX),
        }
    }

    fn json(&self) -> Value {
        json!({
            "name": self.name,
            "home": self.home,
            "class": format!("{:?}", self.class),
            "lo": self.lo,
            "hi": self.hi,
            "grid": self.grid,
            "points": self.points.to_string(),
            "declared_range_is_open": self.open_ceiling,
        })
    }
}

/// The artifact a gate's runner owns. G13 owns **two** — 05 §4 gives the cadence
/// a winner search and the dimming a distribution report — so a lookup that
/// returns a single path cannot describe the protocol.
pub fn artifact_paths(gate: &str) -> Vec<&'static str> {
    RUNNERS
        .iter()
        .filter(|(owner, _, _)| *owner == gate)
        .map(|(_, path, _)| *path)
        .collect()
}

/// The axes of **one** artifact. The registry's `sweep` tag names the file a row
/// is swept into, so two files under one gate sweep disjoint axes; a runner that
/// swept the other gate's axes would publish a curve for constants it does not
/// own.
pub fn axes_of(artifact: &'static str) -> Vec<Axis> {
    let tag = artifact.strip_prefix("gates/").unwrap_or(artifact);
    si_core::consts::all()
        .iter()
        .filter(|row| row.class == Class::Species && row.scalar && row.sweep == Some(tag))
        .map(Axis::of)
        .collect()
}

/// The gate's axes: the scalar SPECIES rows the registry sends to any of this
/// gate's artifacts. TEMPERAMENT rows are drawn, never chosen, so a sweep may not
/// own one (03 §1.2), and a non-scalar row has no grid to enumerate.
pub fn axes(gate: &str) -> Vec<Axis> {
    let mut out: Vec<Axis> = Vec::new();
    for artifact in artifact_paths(gate) {
        for axis in axes_of(artifact) {
            if !out.iter().any(|seen| seen.name == axis.name) {
                out.push(axis);
            }
        }
    }
    out
}

/// The rows the registry tags with this gate but no sweep may install: the draws
/// (OD-87) and the tongue's style rows (OD-175). A sweep reports them as untouched
/// rather than silently dropping them from its axis list — the difference between
/// "the axis is not mine" and "I never saw the row" is the audit's whole point.
fn untouched_rows(gate: &str) -> Vec<Value> {
    si_core::consts::all()
        .iter()
        .filter(|row| {
            row.gate == gate
                && matches!(row.class, Class::Temperament | Class::Tongue)
                && row.sweep.is_some()
        })
        .map(|row| {
            json!({
                "name": row.name,
                "class": format!("{:?}", row.class),
                "why": if row.class == Class::Temperament {
                    "drawn at wake from (birth_seed, position in DERIV_ORDER); no sweep may give it a value (OD-87)"
                } else {
                    "a Tongue row is drawn and refused by class; `launch_constants` cannot install it (OD-175)"
                },
                "gate_tag": row.sweep,
            })
        })
        .collect()
}

/// The scenario a gate's runner plays at each grid point (05 §4 step 2). The
/// runner name separates two artifacts that share one gate: G11's median and
/// composition scenes are different fixtures, and G13's cadence and dimming
/// reports read the same sleep scene.
fn scenario_path(gate: &str, runner: &str) -> &'static str {
    match (gate, runner) {
        ("G11", "median") => "harness/scenarios/g11_median.txt",
        ("G11", "composition") => "harness/scenarios/g11_composition.txt",
        ("G14", "panel") => super::panel::TWINS_SCENARIO,
        ("G12", _) => "harness/scenarios/g12_grimm.txt",
        _ => match gate {
            "G2" => "harness/scenarios/g2_paraphrase.txt",
            "G8" => "harness/scenarios/g8_curiosity.txt",
            "G9" => "harness/scenarios/g9_arousal.txt",
            "G10" => "harness/scenarios/g10_idle.txt",
            "G13" => "harness/scenarios/g13_sleep.txt",
            "G15" => "harness/scenarios/g15_fruit.txt",
            "G16" => "harness/scenarios/g16_warm.txt",
            "G17" => "harness/scenarios/g17_origin.txt",
            "G18" => "harness/scenarios/g18_compare.txt",
            "G19" => "harness/scenarios/g19_server.txt",
            "G20" => "harness/scenarios/g20_tongue.txt",
            _ => "harness/scenarios/g8_curiosity.txt",
        },
    }
}

/// Run the gate's sweep — **every** artifact its runner owns. A grid the protocol
/// can execute is enumerated and each point runs the gate's scenario through the
/// ordinary `run` path; a grid it cannot execute is refused in writing. G12 is the
/// observation-only exception: its trio is verified, never written here.
pub fn run(ws: &Workspace, gate: &str, seed: u64) -> Result<ExitCode> {
    if gate == "G12" {
        return observe_g12(ws, seed);
    }
    let owned: Vec<(&'static str, &'static str)> = RUNNERS
        .iter()
        .filter(|(owner, _, _)| *owner == gate)
        .map(|(_, path, runner)| (*path, *runner))
        .collect();
    if owned.is_empty() {
        bail!(
            "gate {gate} owns no sweep artifact in 05 §4 (the remaining artifacts are \
             observation-only and their gates write them); nothing to sweep"
        );
    }
    let mut code = ExitCode::SUCCESS;
    for (artifact, runner) in owned {
        let next = if runner == "panel" {
            panel_report(ws, gate, artifact, seed)?
        } else {
            sweep_one(ws, gate, runner, artifact, seed)?
        };
        if next != ExitCode::SUCCESS {
            code = next;
        }
    }
    Ok(code)
}

/// One runner, one artifact. `runner` is the name 05 §4's table gives the file
/// (`sleep`, `dimming`, …), which is what separates two artifacts sharing a gate.
fn sweep_one(
    ws: &Workspace,
    gate: &str,
    runner: &str,
    artifact: &'static str,
    seed: u64,
) -> Result<ExitCode> {
    let scenario = scenario_path(gate, runner);
    let axes = axes_of(artifact);
    if axes.is_empty() {
        bail!("runner {runner} owns no swept SPECIES row, so its sweep has no axis");
    }
    let product = product_of(&axes);
    let open: Vec<&Axis> = axes.iter().filter(|axis| axis.open_ceiling).collect();
    let panel = super::panel::load(ws)?;
    let panel_seeds = panel.as_ref().map(|panel| panel.seeds.len()).unwrap_or(0);

    // §4 step 1 has no last point while a declared bound is open, and step 4
    // cannot publish a curve that was never walked. Refuse, in writing — and name
    // the *residual* question (OD-192), because OD-83 ruled validation and
    // explicitly left the search enumeration to §4's own text.
    if !open.is_empty() || product > walkable_ceiling() {
        let mut reason = format!(
            "05 §4 step 1 says \"enumerate the grid over the declared legal ranges in 03 for the \
             constants the gate owns\". {} of {} axes declare an open bound in 03 (`{}`), so the \
             registry's `hi` is the type maximum rather than a design ceiling and the declared grid \
             has no last point; the cross product over the registry's closed representation is {} \
             points. §4 names no per-axis search rule — the full cross product, one axis at a time \
             against a named baseline, or a declared sub-grid — and choosing one is a design \
             decision, not a runner's. OD-83 ruled the *validation* protocol (the winner and both \
             box corners at every panel member) and says §4 step 1 is executed as written; this \
             runner therefore refuses rather than publishing a winner whose curve it cannot name. \
             The residual question is OD-192.",
            open.len(),
            axes.len(),
            open.iter().map(|axis| axis.name).collect::<Vec<_>>().join(", "),
            product,
        );
        if open.is_empty() {
            reason = format!(
                "05 §4 step 1 says \"enumerate the grid over the declared legal ranges\". Every \
                 axis of this artifact is closed, but their product is {} points, which is not a \
                 curve a stage can walk. §4 names no per-axis search rule for a box this wide, and \
                 choosing one is a design decision, not a runner's. The residual question is \
                 OD-192.",
                product,
            );
        }
        // G13's two runners each have a second blocker of their own, and a
        // refusal that stopped at the open bound would hide it.
        match runner {
            "sleep" => reason.push_str(
                " A bounded grid would not make this curve walkable either: 05 §4's objective for this \
                 artifact is G13's own leg set — \"the smallest cadence at which a sleep actually fires \
                 inside the scenario's tick budget and every leg (b)(c)(d)(e)(m) still passes\" — and \
                 leg (e) is the CONFIG ENGINE run with a `SIgen` checksum before and after N sleeps.",
            ),
            "dimming" => reason.push_str(
                " This artifact has a second blocker, and it is not a missing rule: 05 §4.2 defines it \
                 as \"a distribution report, not a winner search\", so the field the other runners exist \
                 to fill — `winner` — is never its content. What it must publish per grid point is \
                 §4.2's list, and that is still a curve over a box the open bound leaves unwalkable.",
            ),
            _ => {}
        }
        let mut od_question = format!(
            "OD-192 — which enumeration does a sweep runner own: the full cross product, one axis at \
             a time against a named baseline, or a declared sub-grid? If one axis at a time, what is \
             the baseline point, given that every axis's current value is an UNMEASURED_PLACEHOLDER? \
             And is a legal range written `> 0` or `≥ 0` in 03 an open bound (so the registry's \
             `hi` is only a type ceiling) or a closed design box? The artifact is a named refusal \
             until the owner rules; no placeholder is replaced and no leg quotes a magnitude from \
             it.",
        );
        match runner {
            "sleep" => od_question.push_str(
                " **OD-95**, which this stage re-raises: with this sweep refused, at what operating \
                 point is a G13 leg graded at all? `si-harness run --gate G13` installs the six sleep \
                 rows at their registry `lo` (`runner::sleep_lo_point`), because that is the only point \
                 in the box the law names rather than a reviewer's hand — but a `lo` is not a swept \
                 winner.",
            ),
            "dimming" => od_question.push_str(
                " **OD-94**, the dimming box's own question: 03 gives `FLOOR_S_R`, `FLOOR_FACTS` and \
                 `FLOOR_LINKS` a legal range from 1 to 65535 Q12, while a live OWNED edge forms at a \
                 weight of the order of 4096 Q12 plus its bake, so most of that box puts the floor above \
                 every live weight and 10 §6 D.2's clamp makes P3 a no-op there.",
            ),
            "phrase" => od_question.push_str(
                " **OD-172**, ruled: `PHRASE_MAX_STEPS` is a swept SPECIES row and the floor leg runs at \
                 the box's `lo`. The artifact stays a refusal because OD-192 is open, not because the \
                 row's class is in doubt.",
            ),
            _ => {}
        }
        let mut document = json!({
            "gate": gate,
            "runner": runner,
            "status": "refused",
            "artifact": artifact,
            "seed_requested": format!("0x{:016x}", seed),
            "seed_is_panel_reference": panel
                .as_ref()
                .and_then(|panel| panel.member_of(seed))
                == Some(1),
            "scenario": scenario,
            "axes": axes.iter().map(Axis::json).collect::<Vec<_>>(),
            "cross_product_points": product.to_string(),
            "objective": objective(runner),
            "reason": reason,
            "od_question": od_question,
            "temperament_rows_untouched": untouched_rows(gate),
            "identity": identity(ws)?,
            "whole_box_protocol": whole_box_protocol(&axes, panel_seeds),
            "placeholders_replaced": [],
            "winner": null,
            "curve": [],
            "consequence": "no winner exists, so no placeholder is replaced and `--strict-consts` still \
                            refuses a run of this gate's swept rows; every leg that cites this artifact \
                            stays UNMEASURED and quotes no magnitude from it",
        });
        // 05 §4.2 is this artifact's own law, so its record carries what a
        // distribution reports and the violation its absence would be.
        if runner == "dimming" {
            if let Some(object) = document.as_object_mut() {
                object.insert(
                    "selection".into(),
                    json!("none — 05 §4.2: \"There is no 'chosen DIM_BASE' in v1 unless a guard leg fails \
                            inside the box, in which case the box narrows and is re-published\""),
                );
                object.insert(
                    "fields_per_grid_point".into(),
                    json!([
                        "post-sleep weight histogram",
                        "within_shelf_rank_inversions",
                        "cross_shelf_rank_inversions",
                        "floor_hits",
                        "max_relative_shift",
                        "recall behavior after the sleep"
                    ]),
                );
                object.insert(
                    "publishing_a_winner_would_be".into(),
                    json!("10 §2 test-1's violation on the harness's side of the fence: a report that \
                          prints a single dimming magnitude as *the* value has turned a species range \
                          into a personality"),
                );
            }
        }
        return write(ws, artifact, &document, false);
    }

    // The executable branch: enumerate the product in registry order, at the
    // reference seed, on one build and one variant (OD-93: no hash is compared
    // across variants).
    let scenario = super::scenario::Scenario::load(ws, scenario)?;
    let mut curve: Vec<Value> = Vec::new();
    let mut index = vec![0usize; axes.len()];
    loop {
        let constants = point_of(&axes, &index);
        let run = execute_point(ws, gate, scenario.clone(), seed, constants)?;
        let report = super::gates::read_gate_report(&run.dir)?;
        curve.push(json!({
            "point": run.request.constants.iter().map(|(name, value)| json!({name: value.to_string()})).collect::<Vec<_>>(),
            "run_dir": ws.relative(&run.dir).replace('\\', "/"),
            "state_hash": run.sections.last().map(|section| section.hash.clone()),
            "gate_pass": report["pass"],
            "objective": objective_value(gate, &report),
        }));
        if !advance(&mut index, &axes) {
            break;
        }
    }

    let best = curve.iter().position(|point| {
        point["objective"]["holds"].as_bool().unwrap_or(false)
    });
    let mut document = json!({
        "gate": gate,
        "runner": runner,
        "status": if best.is_some() { "published" } else { "no_winner" },
        "artifact": artifact,
        "seed_requested": format!("0x{:016x}", seed),
        "seed_is_panel_reference": panel
            .as_ref()
            .and_then(|panel| panel.member_of(seed))
            == Some(1),
        "scenario": scenario.path,
        "axes": axes.iter().map(Axis::json).collect::<Vec<_>>(),
        "cross_product_points": product.to_string(),
        "objective": objective(runner),
        "identity": identity(ws)?,
        "whole_box_protocol": whole_box_protocol(&axes, panel_seeds),
        "curve": curve,
        "winner": best.map(|position| curve[position].clone()),
        "placeholders_replaced": [],
    });

    // §4.1: a winner is not published until the winner and both box corners pass
    // at every panel member. A failing point narrows the box in 03; it never
    // re-samples the panel.
    if let Some(position) = best {
        if let Some(panel) = &panel {
            let winner = curve[position].clone();
            let (matrix, box_holds) = whole_box(ws, gate, &scenario, &axes, &winner, panel)?;
            document["whole_box"] = matrix;
            if !box_holds {
                document["status"] = json!("box_failed");
                document["box_failure"] = json!(
                    "§4.1 step 4: a panel point failed at the winner or a box corner. The fix is to \
                     narrow the declared legal range in engineering/03 and re-run (a `seed_format` \
                     bump); the panel is never re-sampled and the point is never re-picked."
                );
            }
        } else {
            document["status"] = json!("no_panel");
            document["panel_missing"] = json!(
                "05 §4.1 requires the winner at every `seed_panel` member, and \
                 `harness/scenarios/g14_seed_panel.json` is absent. A winner validated only at the \
                 reference seed is not a published box (OD-83)."
            );
        }
    }
    write(ws, artifact, &document, document["status"] == json!("published"))
}

/// The mixed-radix point at `index`, in registry order.
fn point_of(axes: &[Axis], index: &[usize]) -> BTreeMap<String, i64> {
    axes.iter()
        .zip(index)
        .map(|(axis, slot)| (axis.name.to_string(), axis.lo + (*slot as i64) * axis.grid))
        .collect()
}

/// Advance the mixed-radix odometer. False when the walk is complete (a carry off
/// the last axis).
fn advance(index: &mut [usize], axes: &[Axis]) -> bool {
    let mut position = index.len();
    while position > 0 {
        position -= 1;
        index[position] += 1;
        if (index[position] as u128) < axes[position].points {
            return true;
        }
        index[position] = 0;
    }
    false
}

fn product_of(axes: &[Axis]) -> u128 {
    axes.iter().fold(1u128, |acc, axis| acc.saturating_mul(axis.points))
}

/// §4.1's cost is `|panel|` runs per re-sweep, so a curve this module walks must
/// also fit a stage. The ceiling is a refusal threshold, not a search rule: it
/// names no point and drops no axis. It is stated in the artifact so a box that
/// crosses it is a named refusal rather than a hung stage.
fn walkable_ceiling() -> u128 {
    1_000_000
}

/// The per-point run path. One variant, one build, one seed — the curve is a
/// function of the constants alone (OD-93's "no hash across variants").
fn execute_point(
    ws: &Workspace,
    gate: &str,
    scenario: super::scenario::Scenario,
    seed: u64,
    constants: BTreeMap<String, i64>,
) -> Result<super::runner::Run> {
    let request = super::runner::RunRequest {
        gate: gate.to_string(),
        variant: "V0".to_string(),
        config: "si".to_string(),
        seed,
        scenario,
        constants,
        sleep_interrupt: None,
        genome: None,
        genome_tag: None,
        run_dir: None,
    };
    super::runner::Run::execute(request, ws)
}

/// §4.1 steps 2–3 as data: the winner and both box corners, every panel member.
fn whole_box(
    ws: &Workspace,
    gate: &str,
    scenario: &super::scenario::Scenario,
    axes: &[Axis],
    winner: &Value,
    panel: &super::panel::Panel,
) -> Result<(Value, bool)> {
    let mut winner_constants = BTreeMap::new();
    for (name, value) in winner["point"].as_array().into_iter().flatten().filter_map(|point| {
        let object = point.as_object()?;
        let (name, value) = object.iter().next()?;
        Some((name.clone(), value.as_str()?.parse::<i64>().ok()?))
    }) {
        winner_constants.insert(name, value);
    }
    let lo = axes.iter().map(|axis| (axis.name.to_string(), axis.lo)).collect::<BTreeMap<_, _>>();
    let hi = axes.iter().map(|axis| (axis.name.to_string(), axis.hi)).collect::<BTreeMap<_, _>>();
    let mut rows = Vec::new();
    let mut holds = true;
    for seed in &panel.seeds {
        for (corner, constants) in
            [("winner", winner_constants.clone()), ("all_lo", lo.clone()), ("all_hi", hi.clone())]
        {
            let run = execute_point(ws, gate, scenario.clone(), *seed, constants)?;
            let report = super::gates::read_gate_report(&run.dir)?;
            let objective = objective_value(gate, &report);
            let point_holds = objective["holds"].as_bool().unwrap_or(false);
            if !point_holds {
                holds = false;
            }
            rows.push(json!({
                "seed": format!("0x{seed:016x}"),
                "member": panel.member_of(*seed),
                "corner": corner,
                "run_dir": ws.relative(&run.dir).replace('\\', "/"),
                "state_hash": run.sections.last().map(|section| section.hash.clone()),
                "gate_pass": report["pass"],
                "objective": objective,
            }));
        }
    }
    Ok((
        json!({
            "status": if holds { "holds" } else { "failed" },
            "protocol": "05 §4.1: winner + all-lo + all-hi at every `seed_panel` member",
            "panel_size": panel.seeds.len(),
            "runs": rows.len(),
            "matrix": rows,
        }),
        holds,
    ))
}

fn whole_box_protocol(axes: &[Axis], panel_seeds: usize) -> Value {
    json!({
        "reference_seed": "the panel's first member (0x51)",
        "validation": "05 §4.1: re-run the winner AND both box corners (every axis at `lo`, every axis at `hi`) at every `seed_panel` member, grading the gate's behavioral legs at each point",
        "runs_when_published": 3 * panel_seeds.max(1),
        "panel_size": panel_seeds,
        "box_points": axes.len(),
        "failure_rule": "a failing panel point narrows the declared legal range in engineering/03 (a `seed_format` bump); the panel is never re-sampled and the point is never re-picked (OD-83)",
    })
}

/// OD-93: what a sweep record may and may not compare. One build, one variant, one
/// seed per point; a hash from another variant is never a control.
fn identity(ws: &Workspace) -> Result<Value> {
    Ok(json!({
        "variant": "V0",
        "config": "si",
        "tree_md5": super::runner::tree_digest(ws)?,
        "cross_variant_hash_comparison": false,
        "note": "OD-93: a control's identical hash is the pass only within one variant; this runner \
                 records the build it ran on and never compares a hash across variants",
    }))
}

/// The G14 panel: §4's one artifact with no constant to search. Its objective is
/// §4.1's two assertions — the whole box holds (unmeasurable without a published
/// winner) and every seed is a distinct individual (measurable now).
fn panel_report(ws: &Workspace, gate: &str, artifact: &'static str, seed: u64) -> Result<ExitCode> {
    let Some(panel) = super::panel::load(ws)? else {
        println!(
            "sweep G14 — unmeasured ({} is absent; 03 §7's panel is the prerequisite)",
            super::panel::PANEL_ASSET
        );
        return Ok(ExitCode::from(1));
    };
    let digests: Vec<String> = panel
        .seeds
        .iter()
        .map(|seed| format!("0x{:016x}", si_core::seed::temperament_digest(*seed)))
        .collect();
    let distinct_digests = digests.iter().collect::<BTreeSet<_>>().len() == digests.len();
    let structure_ok = panel.size_ok && panel.distinct_ok && panel.rule_matches && distinct_digests;

    let mut document = super::panel::read(ws)?.unwrap_or_else(|| json!({}));
    let members = document["members"].as_array().cloned().unwrap_or_default();
    let tree = super::runner::tree_digest(ws)?;
    let records_this_build: usize = members
        .iter()
        .map(|member| {
            member["runs"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|run| run["tree_md5"].as_str() == Some(tree.as_str()))
                .count()
        })
        .sum();
    let members_with_runs = members
        .iter()
        .filter(|member| member["runs"].as_array().is_some_and(|runs| !runs.is_empty()))
        .count();

    let validation = json!({
        "status": if structure_ok { "structure_ok" } else { "structure_failed" },
        "gate": gate,
        "asset": super::panel::PANEL_ASSET,
        "artifact": artifact,
        "seed_requested": format!("0x{seed:016x}"),
        "panel_size": panel.seeds.len(),
        "declared_floor": super::panel::MIN_PANEL,
        "size_ok": panel.size_ok,
        "members_are_distinct_seeds": panel.distinct_ok,
        "rule_matches_file": panel.rule_matches,
        "distinct_vector_digests": distinct_digests,
        "digest_count": digests.iter().collect::<BTreeSet<_>>().len(),
        "members_with_runs": members_with_runs,
        "records_from_this_build": records_this_build,
        "identity": identity(ws)?,
        "whole_box": {
            "status": "unmeasured",
            "why": "§4.1's first assertion validates a *published* winner and its two corners; no \
                    winner-searching runner has published (OD-192), so this panel can prove \
                    distinctness but not that a box holds",
        },
        "objective": objective("panel"),
    });
    document["sweep_validation"] = validation;
    super::panel::write_document(ws, &document)?;

    println!(
        "sweep G14 — {} ({} members, {} distinct digests, {} records from this build)",
        if structure_ok { "panel_structure_ok" } else { "panel_structure_failed" },
        panel.seeds.len(),
        digests.iter().collect::<BTreeSet<_>>().len(),
        records_this_build
    );
    println!("  whole box: unmeasured — no winner-searching runner has published (OD-192)");
    Ok(if structure_ok { ExitCode::SUCCESS } else { ExitCode::from(1) })
}

/// G12's three observation-only files (§17). The evaluator writes them during a
/// run; this verifier never invents a fourth, never edits them, and reports what
/// they contain. A run in which the filter rejected nothing is a finding, not a
/// green.
fn observe_g12(ws: &Workspace, seed: u64) -> Result<ExitCode> {
    let missing = G12_ARTIFACTS.iter().any(|path| !ws.exists(path));
    if missing {
        let scenario = super::scenario::Scenario::load(ws, scenario_path("G12", "grimm"))?;
        let run = execute_point(ws, "G12", scenario, seed, BTreeMap::new())?;
        println!(
            "sweep G12 — ran the gate to produce the observation trio ({})",
            ws.relative(&run.dir).replace('\\', "/")
        );
    }
    let mut ok = true;
    for path in G12_ARTIFACTS {
        if !ws.exists(path) {
            println!("sweep G12 — missing {path}");
            ok = false;
            continue;
        }
        let bytes = std::fs::read(ws.path(path))?;
        let digest = si_core::sha256::sha256_hex(&bytes);
        if path.ends_with(".json") {
            let document: Value = serde_json::from_slice(&bytes)
                .map_err(|error| anyhow::anyhow!("parse {path}: {error}"))?;
            if document["output_only"].as_bool() != Some(true) {
                println!("sweep G12 — {path} is not marked `output_only: true`");
                ok = false;
            }
            println!("sweep G12 — {path} ({digest})");
        } else {
            let lines = bytes.iter().filter(|byte| **byte == b'\n').count();
            println!("sweep G12 — {path} ({digest}, {lines} lines)");
        }
    }
    println!(
        "sweep G12 — observation-only: no curve, no winner, no placeholder (05 §17); a run in which \
         the filter rejected nothing is a finding about the harness"
    );
    Ok(if ok { ExitCode::SUCCESS } else { ExitCode::from(1) })
}

/// The objective is the design's, never the harness's (05 §4 step 3), so it is
/// quoted from §4's table row for this runner rather than paraphrased.
fn objective(runner: &str) -> &'static str {
    match runner {
        "constants" => "T2 recall stable AND anti-mush: mean pairwise settle-overlap across distinct queries \
                 below PAIR_OVERLAP_MAX (OPEN-12)",
        "arousal" => "bake distribution: at each arousal-constant point the audit labels the run earns match the \
                 emission legs G9(e).1–(e).4, i.e. the six words fire on their logged triggers with the \
                 bake lag the point sets — the curve over B_MAX, K_TANH, MOD_LAG_ROUNDS, LAG_UNIT, \
                 BIAS_CAP_SHARE, MOD_CURVE_VARIANTS",
        "ignition" => "ignition liveliness against the sparsity ceiling: at each IDLE_PER_HOUR point the quiet \
                 run still thinks (G10(e).1) while the rate bounds hold (G10(a).1–(a).3) and the \
                 bake-weight control leaves the state hash unchanged (G10(e).5 — the identity, per OD-91) \
                 — the curve over IDLE_PER_HOUR, the only swept SPECIES row 03 gives this gate, since \
                 OD-87 makes the four IGN_WEIGHT_* draws that no sweep may touch",
        "median" => "MEDIAN_SHAPE, GAMMA, MEDIAN_MIN_N (OPEN-30/31) — N, the repetition count and every bias \
                 magnitude are outputs: one hot stamp on a calm history does not move the word's bias \
                 (b), N repeated stamps do (c), and the two curves are the same function with the same \
                 constant and version tag (c.2)",
        "composition" => "INFERENCE_MAX_HOPS_OWNED — the smallest cap that still passes (f) and (g), because a \
                 bound that never fires in a scenario is decorative; every row records \
                 owned_hops_observed, uncertainty_count and rows_written_during_composition (0 in every \
                 row, including the winner's)",
        "sleep" => "SLEEP_EVERY_QUIET_TICKS and SLEEP_MAX_WALKS — the objective is G13's own leg set, not a \
                 magnitude: the smallest cadence at which a sleep actually fires inside the scenario's \
                 tick budget and every leg (b)(c)(d)(e)(m) still passes. A cadence that never fires in a \
                 scenario is a decorative constant, exactly the failure mode INFERENCE_MAX_HOPS_OWNED \
                 guards against one section up",
        "dimming" => "DIM_BASE, FLOOR_S_R, FLOOR_FACTS, FLOOR_LINKS — a distribution report, not a winner \
                 search (05 §4.2): for each grid point the post-sleep weight histogram, the \
                 within_shelf_rank_inversions and cross_shelf_rank_inversions counts, floor_hits, and the \
                 recall behavior that follows. No point is \"chosen\" unless a guard leg fails somewhere \
                 in the box, in which case the box is narrowed and re-published",
        "panel" => "no tuning at all: 03 §7's fixed `seed_panel`, run across every other sweep. Its \
                 objective is 05 §4.1's two assertions — the whole box holds, and each seed is a \
                 distinct individual (vector digest, not just a distinct integer)",
        "recurrence" => "GHOST_MAX, R_MIN_CRYSTALLIZE, R_ASK_CRYSTALLIZE, DELTA_MIN_CRYSTALLIZE — the smallest \
                 recurrence depth at which a real session's own repetition mints a candidate, with G15's \
                 spam bound (e) still zero on the junk fixture and G15(f)'s no-word-leak intact: a bar \
                 above the measured recurrence depth of any real session is untested, not generous",
        "silt" => "LAMBDA_SILT, SILT_LEVEL_MAX, SILT_BUMP, THETA_SILT, SILT_MAX, IGN_WEIGHT_SILT — not a \
                 magnitude search (05 §23.2): the question a sweep would answer is how warm a level may \
                 get and how much it should steer the sampler, and both are the arc's open design \
                 question, so every point runs the gate's own leg set (G16 a/a′/d/e/f)",
        "prov" => "PROV_LINK_TUG, RELIGHT_MAX — not a magnitude search (05 §23.6, OD-83): the axis is \
                 how strongly a moment tugs the read it began, and that is the arc's open design \
                 question, so every point runs G17's own leg set (a/b/c/d/e/f)",
        "shape" => "R_MIN_SHAPE, DELTA_MIN_SHAPE, K_FILLER, SHAPE_MAX — not a magnitude search \
                 (05 §23.6, docs/25 §25.4, OD-116): the axes are how deep a shape must recur, how \
                 spread its filler sets must be and how far apart its frames must lie before a move \
                 is a skill rather than an anecdote, so every point runs G18's own leg set (a/b/c/d/e/f)",
        "phrase" => "PHRASE_MAX_STEPS — 05 §24: the step bound of the tongue's walk. The objective is the \
                 smallest cap at which a licensed two-arc fixture still realizes a line longer than its \
                 canonical chunk with G20(c)/G20(g) opening zero refusals; a bound below that point is \
                 decorative. OD-172 ruled the row swept and its floor leg runs at `lo`; the artifact is \
                 still a refusal because OD-192 (the enumeration) is open, so no leg quotes a budget",
        _ => "gap-emission behavior at each cap: budget respected, dedupe holds, no fabricated answers",
    }
}

/// The objective as data: the gate's own behavioral legs, read off the report it
/// just wrote. A leg that is UNMEASURED cannot make a point hold.
fn objective_value(gate: &str, report: &Value) -> Value {
    let legs: &[&str] = match gate {
        "G2" => &["G2.d.1", "G2.d.2", "G2.w.1", "G2.t3.1", "G2.t3.2"],
        "G9" => &["G9.e.1", "G9.e.2", "G9.e.3", "G9.e.4"],
        "G10" => &["G10.a.1", "G10.a.2", "G10.a.3", "G10.e.1", "G10.e.5"],
        "G11" => &[
            "G11.a.1", "G11.a.2", "G11.a.3", "G11.b.1", "G11.c.1", "G11.c.2", "G11.d.1",
            "G11.d.2", "G11.e.1", "G11.e.2", "G11.f.1", "G11.f.2", "G11.f.3", "G11.f.4",
            "G11.f.5", "G11.g.1", "G11.g.2", "G11.g.3", "G11.h.1", "G11.h.2", "G11.h.3",
            "G11.h.4", "G11.i.1", "G11.i.2", "G11.i.3", "G11.p.1", "G11.p.2", "G11.p.3",
        ],
        "G13" => &[
            "G13.b.1", "G13.b.2", "G13.c.1", "G13.c.3", "G13.d.1", "G13.d.2", "G13.d.3", "G13.e.1",
            "G13.m.1", "G13.m.2",
        ],
        "G15" => &[
            "G15.a.1", "G15.a.2", "G15.a.3", "G15.b.1", "G15.d.1", "G15.e.1", "G15.f.1",
        ],
        "G16" => &["G16.a.1", "G16.a.2", "G16.d.1", "G16.d.2", "G16.e.1", "G16.f.1"],
        "G17" => &[
            "G17.a.1", "G17.b.1", "G17.c.1", "G17.c.2", "G17.c.3", "G17.d.1", "G17.d.2",
            "G17.e.1", "G17.e.2", "G17.f.1", "G17.f.2",
        ],
        "G18" => &[
            "G18.a.1", "G18.a.2", "G18.a.3", "G18.b.2", "G18.c.1", "G18.c.2", "G18.d.1",
            "G18.e.1", "G18.f.1", "G18.f.2", "G18.f.3",
        ],
        "G20" => &["G20.i.1", "G20.i.2", "G20.c.1", "G20.g.1"],
        _ => &["G8.a.1", "G8.a.2", "G8.a.3", "G8.b.1", "G8.b.2", "G8.d.2", "G8.f.1", "G8.f.2", "G8.f.3"],
    };
    let mut seen = Vec::new();
    let mut holds = true;
    for criterion in report["criteria"].as_array().into_iter().flatten() {
        let id = criterion["id"].as_str().unwrap_or_default();
        if !legs.contains(&id) {
            continue;
        }
        let verdict = criterion["verdict"].as_str().unwrap_or("unmeasured");
        seen.push(json!({ "leg": id, "verdict": verdict }));
        if verdict != "pass" {
            holds = false;
        }
    }
    json!({ "legs": seen, "holds": holds, "legs_missing": legs.len() - seen.len() })
}

fn write(ws: &Workspace, relative: &str, document: &Value, ok: bool) -> Result<ExitCode> {
    let path: PathBuf = ws.path(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(document)?.as_bytes())?;
    println!(
        "sweep {} — {} ({})",
        document["gate"].as_str().unwrap_or("?"),
        document["status"].as_str().unwrap_or("?"),
        ws.relative(&path)
    );
    if let Some(reason) = document["reason"].as_str() {
        println!("  reason: {reason}");
    }
    if let Some(question) = document["od_question"].as_str() {
        println!("  od: {question}");
    }
    Ok(if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn axis(name: &'static str, lo: i64, hi: i64, grid: i64) -> Axis {
        Axis {
            name,
            home: "test",
            class: Class::Species,
            lo,
            hi,
            grid,
            points: 1 + ((hi - lo) / grid) as u128,
            open_ceiling: hi == i64::from(u16::MAX),
        }
    }

    #[test]
    fn the_odometer_walks_every_point_once_in_registry_order() {
        let axes = [axis("a", 0, 2, 1), axis("b", 10, 14, 2)];
        let mut seen = Vec::new();
        let mut index = vec![0usize; axes.len()];
        loop {
            seen.push(point_of(&axes, &index));
            if !advance(&mut index, &axes) {
                break;
            }
        }
        assert_eq!(seen.len(), 9);
        assert_eq!(seen.first().unwrap()["a"], 0);
        assert_eq!(seen.first().unwrap()["b"], 10);
        assert_eq!(seen.last().unwrap()["a"], 2);
        assert_eq!(seen.last().unwrap()["b"], 14);
        let distinct: BTreeSet<Vec<i64>> = seen
            .iter()
            .map(|point| point.values().copied().collect())
            .collect();
        assert_eq!(distinct.len(), 9, "a point was visited twice");
        assert_eq!(product_of(&axes).to_string(), "9");
    }

    #[test]
    fn a_winner_and_both_corners_are_three_distinct_points() {
        let axes = [axis("a", 1, 5, 1), axis("b", 2, 10, 2)];
        let winner = BTreeMap::from([("a".to_string(), 3), ("b".to_string(), 6)]);
        let lo = axes.iter().map(|a| (a.name.to_string(), a.lo)).collect::<BTreeMap<_, _>>();
        let hi = axes.iter().map(|a| (a.name.to_string(), a.hi)).collect::<BTreeMap<_, _>>();
        assert_eq!(lo["a"], 1);
        assert_eq!(lo["b"], 2);
        assert_eq!(hi["a"], 5);
        assert_eq!(hi["b"], 10);
        assert_ne!(winner, lo);
        assert_ne!(winner, hi);
        assert_ne!(lo, hi);
    }

    #[test]
    fn the_runner_table_is_the_law_s_fifteen_and_each_artifact_has_a_scenario() {
        // §4's twelve + the arc's four + the tongue's one = fifteen runners; G13
        // and G11 each own two artifacts, G12's trio is observation-only.
        assert_eq!(RUNNERS.len(), 14);
        let mut artifacts: Vec<&str> = RUNNERS.iter().map(|(_, path, _)| *path).collect();
        artifacts.sort_unstable();
        artifacts.dedup();
        assert_eq!(artifacts.len(), 14, "an artifact is listed twice");
        for (gate, _, runner) in RUNNERS {
            let path = scenario_path(gate, runner);
            assert!(path.starts_with("harness/scenarios/"), "{gate}/{runner} has no scenario");
        }
    }

    #[test]
    fn open_bounds_are_refusals_and_closed_boxes_are_not() {
        let open = [axis("a", 1, i64::from(u16::MAX), 1)];
        let closed = [axis("a", 1, 64, 1)];
        assert!(open.iter().any(|axis| axis.open_ceiling));
        assert!(closed.iter().all(|axis| !axis.open_ceiling));
        assert!(product_of(&closed) <= walkable_ceiling());
    }
}
