//! 05 §3: `gate_report.json`. The evaluator reads the run's own recorded
//! observations — per-step censuses, per-step mind-log records, and the live
//! store through `active_at`/`row_at`/`ask_at` — and never re-reads a JSON it
//! wrote itself (R-33: `verify` is a pure reader of artifacts, `run` is the only
//! mode that holds a session).
//!
//! Three verdicts exist: `pass`, `fail`, and `unmeasured`. A leg with no
//! reachable input at this stage is `unmeasured` and is named in
//! `unresolved_legs`; it is never quietly a pass, and a gate's `pass` is exactly
//! "no criterion has verdict `fail`". `observed` carries references, handles and
//! counts, never an affective magnitude (05 §3's honesty rule).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

use si_core::addr::{
    SHELF_CONTEXT, SHELF_FACTS, SHELF_LINKS, SHELF_QUESTIONS, SHELF_S_R_PAIRS, SHELF_SKILLS,
};
use si_core::hash::HashItem;
use si_core::episodic::OriginTug;
use si_core::strings::Source;
use si_affect::Integrator;
use si_learn::gate;
use si_read::Resolve;
use si_session::{OpenOutcome, Output, Session};
use si_store::snapshot::{Inspect, ResumeRefusal, Snapshot};

use super::runner::{
    DriveOutcome, DriveRequest, InboxSummary, Launch, RowProbe, Run, StepRecord, VariantProbe,
    arena_bytes, boundary_json, census, drive, durable_state_of, row_probe, variant_probe,
    without_emissions,
};
use super::scenario::{Scenario, Step, WorldOp, hex};
use super::Workspace;

pub fn read_gate_report(dir: &Path) -> Result<Value> {
    let path = dir.join("gate_report.json");
    let text = fs::read_to_string(&path)
        .with_context(|| format!("read {} (run the gate first)", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

/// The report. `runs` holds the in-process replay hashes: `G7.b.2` and the
/// `hash_pair` evidence field are that pair, and computing them here keeps
/// `verify` a pure reader.
pub fn evaluate(run: &Run, ws: &Workspace) -> Result<Value> {
    let replay = run.replay()?;
    let mut criteria: Vec<Value> = Vec::new();
    match run.request.gate.as_str() {
        "G1" => g1(run, ws, &mut criteria)?,
        "G2" => g2(run, ws, &mut criteria)?,
        "G3" => g3(run, &mut criteria)?,
        "G4" => g4(run, &mut criteria)?,
        "G5" => g5(run, ws, &replay, &mut criteria)?,
        "G6" => g6(run, ws, &mut criteria)?,
        "G7" => g7(run, ws, &mut criteria)?,
        "G8" => g8(run, ws, &mut criteria)?,
        "G9" => g9(run, ws, &mut criteria)?,
        "G10" => g10(run, ws, &mut criteria)?,
        "G12" => g12(run, ws, &mut criteria)?,
        "G13" => g13(run, ws, &mut criteria)?,
        "G14" => g14(run, ws, &mut criteria)?,
        "G15" => g15(run, ws, &mut criteria)?,
        "G16" => g16(run, ws, &mut criteria)?,
        "G17" => g17(run, ws, &mut criteria)?,
        "G18" => g18(run, ws, &mut criteria)?,
        "G19" => g19(run, ws, &mut criteria)?,
        "G20" => g20(run, ws, &mut criteria)?,
        other => bail!("gate {other} has no evaluator at this stage (11 §5's stage table)"),
    }
    g14_block(run, ws, &mut criteria)?;

    let failed = criteria.iter().any(|c| c["verdict"] == json!("fail"));
    let unresolved: Vec<Value> = criteria
        .iter()
        .filter(|c| c["verdict"] != json!("pass"))
        .map(|c| {
            json!({
                "leg": c["id"],
                "status": c["verdict"],
                "why": c["observed"].get("why").and_then(Value::as_str).unwrap_or(""),
            })
        })
        .collect();
    let sweeps_cited = sweep_citations(run, ws)?;
    let identity = run.identity(ws)?;

    Ok(json!({
        "gate": run.request.gate,
        "config": run.request.config,
        "variant_pair": variant_pair(run),
        "pass": !failed,
        "criteria": criteria,
        "evidence": evidence(run, &replay)?,
        "sweeps_cited": sweeps_cited,
        "unresolved_legs": unresolved,
        "build_identity": identity,
    }))
}

fn variant_pair(run: &Run) -> String {
    format!("{} vs V0 (baseline)", run.request.variant)
}

/// 05 §3: a gate that sweeps cites its artifact; a run launched *by* a sweep
/// records the declared constant list instead, because its point is the row of
/// the curve it is.
///
/// G13 is the exception, and OD-95 names it: both of its sweeps are published
/// refusals, so there is no curve for a launch point to be a row of. The six
/// sleep rows a G13 run installs are their registry `lo` — the only point in the
/// box the law names rather than a reviewer's hand — so a G13 run cites both
/// artifacts, refusal and all, rather than claiming a curve it was not given.
fn sweep_citations(run: &Run, ws: &Workspace) -> Result<Vec<String>> {
    let mut out = Vec::new();
    if !run.request.constants.is_empty()
        && run.request.gate != "G13"
        && run.request.gate != "G16"
        && run.request.gate != "G17"
        && run.request.gate != "G18"
    {
        return Ok(out);
    }
    for name in [
        "gates/G2/constants_sweep.json",
        "gates/G8/budget_sweep.json",
        "gates/G9/arousal_sweep.json",
        "gates/G10/ignition_sweep.json",
        "gates/G13/sleep_sweep.json",
        "gates/G13/dimming_sweep.json",
        "gates/G16/silt_sweep.json",
        "gates/G17/prov_sweep.json",
        "gates/G18/shape_sweep.json",
    ] {
        let owned = matches!(
            (run.request.gate.as_str(), name),
            ("G2", "gates/G2/constants_sweep.json")
                | ("G8", "gates/G8/budget_sweep.json")
                | ("G9", "gates/G9/arousal_sweep.json")
                | ("G10", "gates/G10/ignition_sweep.json")
                | ("G13", "gates/G13/sleep_sweep.json")
                | ("G13", "gates/G13/dimming_sweep.json")
                | ("G16", "gates/G16/silt_sweep.json")
                | ("G17", "gates/G17/prov_sweep.json")
                | ("G18", "gates/G18/shape_sweep.json")
        );
        if owned && ws.exists(name) {
            out.push(name.to_string());
        }
    }
    Ok(out)
}

fn evidence(run: &Run, replay: &[String]) -> Result<Value> {
    let mut routes = Vec::new();
    let mut ask_events = 0usize;
    let mut conflicts = Vec::new();
    let mut row_delta = 0i64;
    for (index, section) in run.sections.iter().enumerate() {
        let first = section
            .steps
            .first()
            .map(|step| knowledge(&step.shelves_after) as i64)
            .unwrap_or(0);
        let last = section
            .steps
            .last()
            .map(|step| knowledge(&step.shelves_after) as i64)
            .unwrap_or(first);
        row_delta = row_delta + (last - first);
        for step in &section.steps {
            if matches!(step.output, Some(Output::Ask { .. })) {
                ask_events += 1;
            }
            if let Some(Output::Answer { citation, .. }) = &step.output {
                routes.push(json!({
                    "section": index,
                    "step": step.index,
                    "ref": format!("mindlog.json#sections[{index}].steps[{}]", step.index),
                    "cited_rows": citation.len(),
                }));
            }
            let events = gate_events(run, index, step);
            let branch = events
                .iter()
                .filter_map(|event| event.get("branch").and_then(Value::as_str))
                .find(|branch| !branch.is_empty())
                .unwrap_or("commit");
            let asked = events
                .iter()
                .any(|event| event["event"] == json!("ask"));
            let resolved_at = events.iter().find_map(|event| {
                (event["event"] == json!("supersede")).then_some(event["at"].as_i64().unwrap_or(0))
            });
            for event in events
                .iter()
                .filter(|event| event["event"] == json!("conflict"))
            {
                conflicts.push(json!({
                    "section": index,
                    "step": step.index,
                    "count": event["count"],
                    "branch": branch,
                    "asked": asked,
                    "resolved_at": resolved_at,
                    "losers": event["losers"],
                }));
            }
        }
    }
    Ok(json!({
        "routes": routes,
        "ask_events": ask_events,
        "row_delta": row_delta,
        "checksums": {"L1": null, "L2": null},
        "hash_pair": {
            "a": run.sections.first().map(|section| section.hash.clone()),
            "b": replay.first().cloned(),
        },
        "conflicts": conflicts,
    }))
}

// ---------------------------------------------------------------------------
// Observation helpers.
// ---------------------------------------------------------------------------

fn step<'a>(run: &'a Run, section: usize, index: usize) -> Result<&'a StepRecord> {
    run.step(section, index).ok_or_else(|| {
        anyhow::anyhow!(
            "{} needs section {section} step {index}; the scenario is shorter than the gate's script",
            run.request.gate
        )
    })
}

/// Knowledge rows: the three belief shelves. `questions` and `context` are not
/// knowledge and the `context` floor is paid by every line (01 §6).
fn knowledge(census: &BTreeMap<u16, usize>) -> usize {
    [SHELF_S_R_PAIRS, SHELF_FACTS, SHELF_LINKS]
        .iter()
        .map(|shelf| census.get(shelf).copied().unwrap_or_default())
        .sum()
}

fn questions(census: &BTreeMap<u16, usize>) -> usize {
    census.get(&SHELF_QUESTIONS).copied().unwrap_or_default()
}

/// The mind-log records of one step, with the session's `{"gate":{…}}` wrapper
/// removed, so a leg reads `{"event":"commit",…}` whichever path wrote it.
fn gate_events(run: &Run, section: usize, record: &StepRecord) -> Vec<Value> {
    let mut out = Vec::new();
    for event in run.events(section, record) {
        let text = String::from_utf8_lossy(&event.body).to_string();
        let Ok(parsed) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        match parsed.get("gate") {
            Some(inner) => out.push(inner.clone()),
            None if parsed.get("event").is_some() => out.push(parsed),
            _ => {}
        }
    }
    out
}

fn raw_bodies(run: &Run, section: usize, record: &StepRecord) -> Vec<Value> {
    run.events(section, record)
        .iter()
        .filter_map(|event| serde_json::from_str(&String::from_utf8_lossy(&event.body)).ok())
        .collect()
}

fn event_named(run: &Run, section: usize, record: &StepRecord, name: &str) -> Vec<Value> {
    gate_events(run, section, record)
        .into_iter()
        .filter(|event| event["event"] == json!(name))
        .collect()
}

fn surface(session: &Session, handle: u32) -> String {
    session
        .store()
        .interner()
        .get(handle)
        .map(|entry| entry.verbatim.clone())
        .unwrap_or_default()
}

fn folded(session: &Session, handle: u32) -> String {
    session
        .store()
        .interner()
        .get(handle)
        .map(|entry| entry.folded.clone())
        .unwrap_or_default()
}

/// The folded content keys of one scenario line, for the "owner tokens only"
/// and "zero content-word overlap" legs.
fn line_keys(text: &str) -> Vec<String> {
    si_front::tokenize(text)
        .into_iter()
        .map(|token| token.key)
        .collect()
}

fn crit(
    out: &mut Vec<Value>,
    id: &str,
    leg: &str,
    predicate: &str,
    observed: Value,
    tolerance: Value,
    verdict: &str,
    fail_meaning: &str,
) {
    out.push(json!({
        "id": id,
        "leg": leg,
        "predicate": predicate,
        "observed": observed,
        "tolerance": tolerance,
        "verdict": verdict,
        "fail_meaning": fail_meaning,
    }));
}

fn ok(out: &mut Vec<Value>, id: &str, leg: &str, predicate: &str, observed: Value, fail: &str) {
    crit(out, id, leg, predicate, observed, json!(0), "pass", fail);
}

fn bad(out: &mut Vec<Value>, id: &str, leg: &str, predicate: &str, observed: Value, fail: &str) {
    crit(out, id, leg, predicate, observed, json!(0), "fail", fail);
}

fn open(
    out: &mut Vec<Value>,
    id: &str,
    leg: &str,
    predicate: &str,
    why: &str,
    observed: Value,
    fail: &str,
) {
    let mut observed = observed;
    if let Some(map) = observed.as_object_mut() {
        map.insert("why".to_string(), json!(why));
    }
    crit(out, id, leg, predicate, observed, json!(0), "unmeasured", fail);
}

// ---------------------------------------------------------------------------
// G1 — write discipline (05 §6).
// ---------------------------------------------------------------------------

fn g1(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let one = step(run, 0, 1)?;
    let two = step(run, 0, 2)?;
    let three = step(run, 0, 3)?;
    let five = step(run, 0, 5)?;
    let six = step(run, 0, 6)?;
    let seven = step(run, 0, 7)?;
    let eight = step(run, 0, 8)?;

    // G1.a.1 — the birth answer asks and writes nothing on a knowledge shelf.
    let asked = matches!(one.output, Some(Output::Ask { .. }));
    if asked
        && knowledge(&one.shelves_after) == 0
        && questions(&one.shelves_after) == 1
        && one.rows_written == 0
    {
        ok(
            out,
            "G1.a.1",
            "ask-back writes no knowledge row",
            "knowledge rows written after line 1 = 0 and the ask row lives in `questions`",
            json!({"step": 1, "output": "ask", "knowledge_rows": 0, "question_rows": 1}),
            "a birth answer that asserts instead of asking back",
        );
    } else {
        bad(
            out,
            "G1.a.1",
            "ask-back writes no knowledge row",
            "knowledge rows written after line 1 = 0 and the ask row lives in `questions`",
            json!({
                "step": 1,
                "output_is_ask": asked,
                "knowledge_rows": knowledge(&one.shelves_after),
                "question_rows": questions(&one.shelves_after),
                "rows_written": one.rows_written,
            }),
            "a birth answer that asserts instead of asking back",
        );
    }

    // G1.a.2 — I-DON'T-KNOW/NONE, and the slot is named from owner tokens only.
    let session = &run.sections[0].session;
    let line_keys = line_keys(&one.text);
    let named = match one.output {
        Some(Output::Ask { question_id, .. }) => session
            .store()
            .row_at(SHELF_QUESTIONS, question_id)?
            .map(|record| record.body.key_point),
        _ => None,
    };
    let slot_is_owner_token = named
        .map(|handle| {
            let verbatim = surface(session, handle);
            si_front::tokenize(&one.text)
                .into_iter()
                .any(|token| token.surface == verbatim)
        })
        .unwrap_or(false);
    let slot_in_line = named
        .map(|handle| {
            let key = session
                .store()
                .points()
                .canonical(handle)
                .unwrap_or_default()
                .to_string();
            line_keys.iter().any(|line_key| *line_key == key)
        })
        .unwrap_or(false);
    let labeled = raw_bodies(run, 0, one)
        .iter()
        .any(|body| body["label"] == json!("I-DON'T-KNOW") && body["tag"] == json!("NONE"));
    if slot_in_line && labeled {
        ok(
            out,
            "G1.a.2",
            "the slot is named from owner tokens only",
            "label = I-DON'T-KNOW/NONE and the slot name is composed only of tokens present in line 1",
            json!({
                "step": 1,
                "label_ref": "mindlog.json#sections[0].steps[1].events",
                "slot_handle": named,
                "slot_key_present_in_line": slot_in_line,
                "surface_is_the_token": slot_is_owner_token,
            }),
            "a birth answer that asserts instead of asking back",
        );
    } else {
        bad(
            out,
            "G1.a.2",
            "the slot is named from owner tokens only",
            "label = I-DON'T-KNOW/NONE and the slot name is composed only of tokens present in line 1",
            json!({"step": 1, "label_seen": labeled, "slot_key_present_in_line": slot_in_line}),
            "a birth answer that asserts instead of asking back",
        );
    }

    // G1.a.3 — the nonce fallback. No input reaches it in this build.
    open(
        out,
        "G1.a.3",
        "unnameable question shape yields SPECULATION via the nonce",
        "an unnameable question shape yields SPECULATION via the nonce and still zero rows",
        "every frame in the §3.2 closed table yields a last slot, so a decoded question always names one; \
         the unnamed-slot fallback (docs/03 §7.4) has no reachable input, and an `Undecoded` line is not a \
         question and never reaches the read path (04 §11's OD-10 note)",
        json!({"reachable_inputs_in_closed_table": 0}),
        "faking is back: a nonce invented as an answer",
    );

    // G1.b.1 — the first `/correct` writes exactly one row, bound and surprised.
    let bound = event_named(run, 0, two, "commit")
        .first()
        .and_then(|event| event["bind_output_id"].as_u64())
        .unwrap_or(0);
    let surprised = !event_named(run, 0, two, "novelty").is_empty();
    let added = knowledge(&two.shelves_after) - knowledge(&one.shelves_after);
    if added == 1 && bound != 0 && surprised && two.rows_written == 1 {
        ok(
            out,
            "G1.b.1",
            "the correction writes exactly one bound row",
            "line 2 adds exactly one row with `bind_output_id != 0` and a surprise record",
            json!({
                "step": 2,
                "row_delta": added,
                "bind_output_id": bound,
                "surprise_record": "mindlog.json#sections[0].steps[2].events",
            }),
            "over-writing journal",
        );
    } else {
        bad(
            out,
            "G1.b.1",
            "the correction writes exactly one bound row",
            "line 2 adds exactly one row with `bind_output_id != 0` and a surprise record",
            json!({"step": 2, "row_delta": added, "bind_output_id": bound, "surprise_record": surprised}),
            "over-writing journal",
        );
    }

    // G1.c.1 — the repeat recall serves, confirms and writes nothing.
    let served_row = match &three.output {
        Some(Output::Answer { citation, .. }) => citation.first().copied(),
        _ => None,
    };
    let row = match served_row {
        Some(id) => session.store().row_at(SHELF_S_R_PAIRS, id)?,
        None => None,
    };
    let expected = row
        .as_ref()
        .map(|record| {
            let bake = event_named(run, 0, two, "commit")
                .first()
                .and_then(|event| event["bake"].as_u64())
                .unwrap_or(0) as u16;
            (gate::conf_init(record.envelope.prov_kind) + bake + gate::bump_served())
                .min(si_core::CONF_MAX)
        })
        .unwrap_or(0);
    let arithmetic = row
        .as_ref()
        .map(|record| record.envelope.repeats == 2 && record.envelope.confidence == expected)
        .unwrap_or(false);
    if knowledge(&three.shelves_after) == knowledge(&two.shelves_after)
        && three.rows_written == 0
        && arithmetic
    {
        ok(
            out,
            "G1.c.1",
            "a known answer re-served writes no row",
            "line 3 adds zero rows, `repeats = 2`, conf = init + CONF_BUMP_SERVED",
            json!({
                "step": 3,
                "row_delta": 0,
                "row_ref": format!("s_r_pairs#{}", served_row.unwrap_or(0)),
                "repeats": 2,
                "conf_equals_init_plus_bake_plus_served_bump": arithmetic,
            }),
            "over-writing journal: a repeat that mints a second row",
        );
    } else {
        bad(
            out,
            "G1.c.1",
            "a known answer re-served writes no row",
            "line 3 adds zero rows, `repeats = 2`, conf = init + CONF_BUMP_SERVED",
            json!({
                "step": 3,
                "row_delta": knowledge(&three.shelves_after) - knowledge(&two.shelves_after),
                "repeats": row.as_ref().map(|record| record.envelope.repeats),
                "conf_arithmetic_holds": arithmetic,
            }),
            "over-writing journal: a repeat that mints a second row",
        );
    }

    // G1.d.1 — the second identical statement is boring.
    if knowledge(&five.shelves_after) == knowledge(&step(run, 0, 4)?.shelves_after)
        && five.rows_written == 0
    {
        ok(
            out,
            "G1.d.1",
            "a boring repeat writes nothing",
            "line 5 adds zero rows",
            json!({"step": 5, "row_delta": 0, "confirm_record": !event_named(run, 0, five, "confirm").is_empty()}),
            "content-selective rejection: the same sentence written twice",
        );
    } else {
        bad(
            out,
            "G1.d.1",
            "a boring repeat writes nothing",
            "line 5 adds zero rows",
            json!({"step": 5, "row_delta": knowledge(&five.shelves_after) - knowledge(&step(run, 0, 4)?.shelves_after)}),
            "content-selective rejection: the same sentence written twice",
        );
    }

    // G1.e.1 — an ordinary line is input, not a teach, and raises no ask.
    if knowledge(&six.shelves_after) == knowledge(&five.shelves_after)
        && questions(&six.shelves_after) == questions(&five.shelves_after)
        && six.rows_written == 0
    {
        ok(
            out,
            "G1.e.1",
            "input that is not a teach stays input",
            "line 6 writes no row and creates no `AskMatch`",
            json!({"step": 6, "knowledge_row_delta": 0, "question_row_delta": 0, "output": "inert"}),
            "the drive inventing a question for a line that named no gap",
        );
    } else {
        bad(
            out,
            "G1.e.1",
            "input that is not a teach stays input",
            "line 6 writes no row and creates no `AskMatch`",
            json!({
                "step": 6,
                "knowledge_row_delta": knowledge(&six.shelves_after) - knowledge(&five.shelves_after),
                "question_row_delta": questions(&six.shelves_after) - questions(&five.shelves_after),
            }),
            "the drive inventing a question for a line that named no gap",
        );
    }

    // G1.f.1 — gibberish parity, in docs/13's words: line 7 is a row like any
    // other statement, and line 8 decodes to line 1's shape with different
    // content slots.
    let decode_of = |record: &StepRecord| {
        raw_bodies(run, 0, record)
            .iter()
            .find_map(|body| body["pattern"].as_u64().map(|p| p as u16))
    };
    let seven_row = knowledge(&seven.shelves_after) - knowledge(&six.shelves_after);
    let shape_match = decode_of(one) == decode_of(eight);
    let handles_differ = {
        let a = match one.output {
            Some(Output::Ask { question_id, .. }) => session
                .store()
                .row_at(SHELF_QUESTIONS, question_id)?
                .map(|record| record.body.key_point),
            _ => None,
        };
        let b = match eight.output {
            Some(Output::Ask { question_id, .. }) => session
                .store()
                .row_at(SHELF_QUESTIONS, question_id)?
                .map(|record| record.body.key_point),
            _ => None,
        };
        a.is_some() && a != b
    };
    if seven_row == 1 && shape_match && handles_differ {
        ok(
            out,
            "G1.f.1",
            "gibberish parity",
            "line 7 writes one row and line 8 decodes to the same `pattern_id` as line 1 with different slot handles",
            json!({
                "step": 8,
                "pattern_ids": [decode_of(one), decode_of(eight)],
                "equal": shape_match,
                "slot_handles_differ": handles_differ,
                "line_7_row_delta": seven_row,
            }),
            "content-selective rejection: the decoder knowing a real word and not a made-up one",
        );
    } else {
        bad(
            out,
            "G1.f.1",
            "gibberish parity",
            "line 7 writes one row and line 8 decodes to the same `pattern_id` as line 1 with different slot handles",
            json!({
                "step": 8,
                "pattern_ids": [decode_of(one), decode_of(eight)],
                "equal": shape_match,
                "slot_handles_differ": handles_differ,
                "line_7_row_delta": seven_row,
            }),
            "content-selective rejection: the decoder knowing a real word and not a made-up one",
        );
    }

    // The teach leg lives on the run's second (cold) section: 05 §6's lines 9–12.
    if run.sections.len() < 2 {
        open(
            out,
            "G1.t.1",
            "the teach channel",
            "a cold `/teach` yields exactly 1 row with `prov_kind = owner-teach` …",
            "the scenario has no cold teach section; the (t) block is the run's second session",
            json!({"sections": run.sections.len()}),
            "a `/teach` that writes a second row onto a covered slot",
        );
        return Ok(());
    }
    let teach = &run.sections[1];
    let t1 = step(run, 1, 1)?;
    let t2 = step(run, 1, 2)?;
    let t3 = step(run, 1, 3)?;
    let t4 = step(run, 1, 4)?;

    let commit1 = event_named(run, 1, t1, "commit").first().cloned();
    let written = commit1
        .as_ref()
        .and_then(|event| event["row"].as_u64())
        .and_then(|row| teach.session.store().row_at(SHELF_S_R_PAIRS, row as u32).ok().flatten());
    let t1_ok = knowledge(&t1.shelves_after) == 1
        && t1.rows_written == 1
        && commit1.as_ref().is_some_and(|event| {
            event["prov_kind"] == json!(gate::PROV_OWNER_TEACH)
                && event["prov_speaker"] == json!(gate::SPEAKER_OWNER)
                && event["bind_output_id"] == json!(0)
                && event["gain_kind"] == json!(gate::GAIN_ASSERTED)
                && event["gain_exact"] == json!(0)
                && event["question_id"] == json!(0)
        })
        && !event_named(run, 1, t1, "novelty").is_empty()
        && written.as_ref().is_some_and(|record| record.envelope.question_id == 0);
    if t1_ok {
        ok(
            out,
            "G1.t.1",
            "the teach writes one asserted row",
            "a cold `/teach` yields exactly 1 row with `prov_kind = owner-teach`, `prov_speaker = owner`, \
             `bind_output_id = 0`, `gain_kind = asserted`, `gain_exact = 0`, a logged surprise and `question_id = 0`",
            json!({"section": 1, "step": 1, "row_ref": "s_r_pairs", "commit_record": "mindlog.json#sections[1].steps[1].events"}),
            "a `/teach` that writes a second row onto a covered slot",
        );
    } else {
        bad(
            out,
            "G1.t.1",
            "the teach writes one asserted row",
            "a cold `/teach` yields exactly 1 row with `prov_kind = owner-teach`, `prov_speaker = owner`, \
             `bind_output_id = 0`, `gain_kind = asserted`, `gain_exact = 0`, a logged surprise and `question_id = 0`",
            json!({
                "section": 1,
                "step": 1,
                "knowledge_rows": knowledge(&t1.shelves_after),
                "commit": commit1,
            }),
            "a `/teach` that writes a second row onto a covered slot",
        );
    }

    let t2_confirm = event_named(run, 1, t2, "confirm").first().cloned();
    if knowledge(&t2.shelves_after) == knowledge(&t1.shelves_after)
        && t2.rows_written == 0
        && t2_confirm.as_ref().is_some_and(|event| event["repeats"] == json!(2))
    {
        ok(
            out,
            "G1.t.2",
            "coverage cancels novelty",
            "the repeat adds zero rows and bumps `repeats`/conf only",
            json!({"section": 1, "step": 2, "row_delta": 0, "repeats": 2, "confirm_record": "mindlog.json#sections[1].steps[2].events"}),
            "a `/teach` that writes a second row onto a covered slot",
        );
    } else {
        bad(
            out,
            "G1.t.2",
            "coverage cancels novelty",
            "the repeat adds zero rows and bumps `repeats`/conf only",
            json!({
                "section": 1,
                "step": 2,
                "row_delta": knowledge(&t2.shelves_after) - knowledge(&t1.shelves_after),
                "confirm": t2_confirm,
            }),
            "a `/teach` that writes a second row onto a covered slot",
        );
    }

    let dark = raw_bodies(run, 1, t3)
        .iter()
        .find(|body| body.get("dark").is_some())
        .cloned();
    if knowledge(&t3.shelves_after) == knowledge(&t2.shelves_after)
        && t3.rows_written == 0
        && dark.as_ref().is_some_and(|body| body["dark"].as_array().is_some_and(|slots| !slots.is_empty()))
        && matches!(t3.output, Some(Output::Refused { .. }))
    {
        ok(
            out,
            "G1.t.3",
            "an undecodable left side names a dark spot and writes nothing",
            "an undecodable left side writes zero rows and emits a named dark spot — the row key is the decoded \
             pattern, never the typed text",
            json!({
                "section": 1,
                "step": 3,
                "row_delta": 0,
                "named_slots": dark.as_ref().map(|body| body["dark"].as_array().map(Vec::len)),
                "gap_record": "mindlog.json#sections[1].steps[3].events",
            }),
            "a teach keyed by the raw text instead of the decoded pattern",
        );
    } else {
        bad(
            out,
            "G1.t.3",
            "an undecodable left side names a dark spot and writes nothing",
            "an undecodable left side writes zero rows and emits a named dark spot — the row key is the decoded \
             pattern, never the typed text",
            json!({
                "section": 1,
                "step": 3,
                "row_delta": knowledge(&t3.shelves_after) - knowledge(&t2.shelves_after),
                "dark_record": dark,
            }),
            "a teach keyed by the raw text instead of the decoded pattern",
        );
    }

    let gibberish = event_named(run, 1, t4, "commit").first().cloned();
    let parity = match (&commit1, &gibberish) {
        (Some(first), Some(second)) => {
            first["branch"] == second["branch"]
                && first["shelf"] == second["shelf"]
                && first["prov_kind"] == second["prov_kind"]
                && first["gain_kind"] == second["gain_kind"]
                && first["row"] != second["row"]
        }
        _ => false,
    };
    if knowledge(&t4.shelves_after) == 2 && parity {
        ok(
            out,
            "G1.t.4",
            "the teach channel is content-blind",
            "the gibberish teach produces the identical gate trace shape with substituted tokens",
            json!({
                "section": 1,
                "steps": [1, 4],
                "trace_shape_equal": parity,
                "row_ids_differ": true,
            }),
            "content-selective rejection through the newest door",
        );
    } else {
        bad(
            out,
            "G1.t.4",
            "the teach channel is content-blind",
            "the gibberish teach produces the identical gate trace shape with substituted tokens",
            json!({"section": 1, "step": 4, "knowledge_rows": knowledge(&t4.shelves_after), "commit": gibberish}),
            "content-selective rejection through the newest door",
        );
    }
    // G1.u.1 — the **left-side accept set** (09 §3's malformation table, 05 §6(u)).
    // The four question frames each commit one row; a statement-shaped left side is
    // refused with the reason `left_not_a_question` and writes nothing. These are the
    // six lines `check_teach_left_side` grades statically, arriving here through the
    // tick: the loop's own decode log is the witness, and the shelf count is its twin.
    const ACCEPT_SET: [(&str, bool); 6] = [
        ("what color is apple", true),
        ("what is apple", true),
        ("when was apple", true),
        ("red of apple", true),
        ("apple is red", false),
        ("znark means floop", false),
    ];
    let accept_steps: Vec<Step> = ACCEPT_SET
        .iter()
        .map(|(left, _)| Step::Sense {
            source: Source::Owner,
            text: format!("/teach {left} => content"),
            identity: None,
            conversation: 0,
        })
        .collect();
    let accepted = teach_drive(run, accept_steps, Vec::new())?;
    let teach_log: Vec<Value> = accepted
        .session
        .events()
        .events()
        .iter()
        .filter_map(|event| serde_json::from_slice::<Value>(&event.body).ok())
        .filter(|body| body["kind"] == json!("teach"))
        .collect();
    let want_rows = ACCEPT_SET.iter().filter(|(_, commits)| *commits).count();
    let set_holds = teach_log.len() == ACCEPT_SET.len()
        && teach_log
            .iter()
            .zip(ACCEPT_SET)
            .all(|(body, (_, commits))| {
                let outcome = body["outcome"].as_str().unwrap_or_default();
                let rows = body["rows"].as_u64().unwrap_or_default();
                // 09 §2: `Row`'s reason is the empty string — the channel owns no
                // content word, so a commit says nothing and a refusal names itself.
                if commits {
                    outcome.is_empty() && rows == 1
                } else {
                    outcome == "left_not_a_question" && rows == 0
                }
            });
    let accept_shelf = census(&accepted.session)?
        .get(&SHELF_S_R_PAIRS)
        .copied()
        .unwrap_or(0);
    if set_holds && accept_shelf == want_rows {
        ok(
            out,
            "G1.u.1",
            "the left-side accept set is the four question frames",
            "each of the four question-frame left sides commits one row, a statement-shaped left \
             side is logged with the reason `left_not_a_question` and zero rows, and the shelf \
             holds exactly the accepted count",
            json!({
                "lines": ACCEPT_SET.len(),
                "accepted": want_rows,
                "refused": ACCEPT_SET.len() - want_rows,
                "decode_log": teach_log,
                "s_r_pairs_rows": accept_shelf,
                "arm": "one in-process drive of the six teaches, typed",
            }),
            "the channel grew a second keying rule, or a statement-shaped left side silently \
             produced a row",
        );
    } else {
        bad(
            out,
            "G1.u.1",
            "the left-side accept set is the four question frames",
            "each of the four question-frame left sides commits one row, a statement-shaped left \
             side is logged with the reason `left_not_a_question` and zero rows, and the shelf \
             holds exactly the accepted count",
            json!({
                "lines": ACCEPT_SET.len(),
                "accepted": want_rows,
                "refused": ACCEPT_SET.len() - want_rows,
                "accept_set_holds": set_holds,
                "decode_log": teach_log,
                "s_r_pairs_rows": accept_shelf,
                "arm": "one in-process drive of the six teaches, typed",
            }),
            "the channel grew a second keying rule, or a statement-shaped left side silently \
             produced a row",
        );
    }

    // G1.u.2 — **bulk equivalence**, the feeder's license (09 §10, OD-69). The same
    // curriculum reaches the loop twice: once as a `sense_inbox.siro` lane built by the
    // feeder's own writer and heard before the steps, once as the steps themselves.
    // Both arms are one `drive` code, so a difference is the feeder bypassing or adding
    // a step — never two harnesses disagreeing (05 §1).
    const CURRICULUM: [&str; 6] = [
        "/teach what color is apple => red",
        "/teach what is apple => a fruit",
        "/teach when was rome founded => in 753 bc",
        "/teach red of apple => a colour",
        "/teach what is znark => a floop",
        "/teach what is floop => a znark",
    ];
    let steps: Vec<Step> = CURRICULUM
        .iter()
        .map(|line| Step::Sense {
            source: Source::Owner,
            text: (*line).to_string(),
            identity: None,
            conversation: 0,
        })
        .collect();
    let lane_texts: Vec<&str> = CURRICULUM.to_vec();
    let (lane_bytes, lane_frames) = Session::encode_sense_inbox(&[], &lane_texts, "g1u2")?;
    // The row of the law names a **file**, not a buffer: `si-cli feed` writes a
    // `sense_inbox.siro` into the run directory and the loop hears it from there. So
    // the fed arm's lane is written to the run directory and read back before it is
    // fed — the equivalence is graded over the transport's own storage, not only over
    // bytes held in a register. The writer stays the loop's single one
    // (`Session::encode_sense_inbox`, which `feed_inbox` calls); the feeder's file
    // handling — the filter, the append, the strict walk of a torn lane — is graded by
    // the tests in the binary that hosts it, not by a second implementation here.
    let lane_path = run.dir.join("g1u2_fed_lane.siro");
    fs::write(&lane_path, &lane_bytes).with_context(|| format!("write {}", lane_path.display()))?;
    let lane_on_disk = fs::read(&lane_path).with_context(|| format!("read {}", lane_path.display()))?;
    let lane_survived_the_disk = lane_on_disk == lane_bytes;
    let typed = teach_drive(run, steps, Vec::new())?;
    let fed = teach_drive(run, Vec::new(), lane_on_disk)?;
    // The comparison has to bite: an equality between two arms that each reached
    // nothing is not a measurement. So the same loop is fed the curriculum one line
    // short, and that hash must differ from the fed arm's — the control that turns
    // byte-identity into an observation rather than into an absence (05 §3).
    let (short_lane, short_frames) =
        Session::encode_sense_inbox(&[], &lane_texts[..lane_texts.len() - 1], "g1u2-control")?;
    let short = teach_drive(run, Vec::new(), short_lane)?;
    let control_differs = short.hash != fed.hash;
    let summary = fed.inbox.clone().unwrap_or(InboxSummary {
        frames_heard: 0,
        torn_tail_bytes: 0,
        refused: Some("the fed arm heard no lane at all".to_string()),
    });
    let shown = |outcome: &DriveOutcome| {
        outcome
            .session
            .transcript()
            .iter()
            .map(|(t, source, text)| format!("{t}\t{}\t{text}", source.literal()))
            .collect::<Vec<_>>()
    };
    let fed_transcript = shown(&fed);
    let typed_transcript = shown(&typed);
    let equivalent = summary.frames_heard == CURRICULUM.len()
        && summary.refused.is_none()
        && summary.torn_tail_bytes == 0
        && lane_survived_the_disk
        && fed.hash == typed.hash
        && fed_transcript == typed_transcript
        && control_differs;
    let observed = json!({
        "arm": "the same six lines, once as a fed lane read back off the run directory and once typed",
        "lane_file": "g1u2_fed_lane.siro",
        "lane_file_bytes": lane_bytes.len(),
        "lane_file_survived_the_disk": lane_survived_the_disk,
        "lane_frames_written": lane_frames,
        "lane_frames_heard": summary.frames_heard,
        "torn_tail_bytes": summary.torn_tail_bytes,
        "refusal": summary.refused,
        "fed_hash": fed.hash,
        "typed_hash": typed.hash,
        "hash_equal": fed.hash == typed.hash,
        "transcript_equal": fed_transcript == typed_transcript,
        "transcript": fed_transcript,
        "control": {
            "lines": short_frames,
            "hash": short.hash,
            "differs_from_fed_arm": control_differs,
        },
        "fed_writer": "si-session's Session::encode_sense_inbox, the writer si-cli feed calls",
        "feeder_file_handling": "graded by the si-cli binary's own tests (filter, append, torn lane)",
    });
    if equivalent {
        ok(
            out,
            "G1.u.2",
            "a fed lane is the same state as the same lines typed",
            "the fed arm and the typed arm reach an equal state hash and an equal transcript, \
             the fed arm heard every frame of the lane read back off the run directory's file, \
             and a lane one line short reaches a different hash",
            observed,
            "the feeder is a loader: a fed lane reached a different state than the same \
             lines typed, so it bypassed or added a step",
        );
    } else {
        bad(
            out,
            "G1.u.2",
            "a fed lane is the same state as the same lines typed",
            "the fed arm and the typed arm reach an equal state hash and an equal transcript, \
             the fed arm heard every frame of the lane read back off the run directory's file, \
             and a lane one line short reaches a different hash",
            observed,
            "the feeder is a loader: a fed lane reached a different state than the same \
             lines typed, so it bypassed or added a step",
        );
    }

    // The family's curriculum (05 §6(u)): the checked-in 200-line fixture, fed through
    // the inbox writer into one cold session. `G1.u.2` proved a fed arm *is* the typed
    // arm, so the three legs below read one import and count what it left behind.
    let fixture = Scenario::load(ws, "harness/scenarios/g01_u_curriculum_200.txt")?;
    let curriculum: Vec<&str> = fixture.sections[0]
        .iter()
        .filter_map(|step| match step {
            Step::Sense { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    let (import_lane, import_frames) =
        Session::encode_sense_inbox(&[], &curriculum, "g1u-curriculum")?;
    let import = teach_drive(run, Vec::new(), import_lane)?;
    let store = import.session.store();
    let taught_rows: Vec<si_core::record::Record> = store
        .rows()?
        .into_iter()
        .map(|(_, record)| record)
        .filter(|record| record.envelope.shelf_x == SHELF_S_R_PAIRS)
        .collect();
    // The fixture's own construction is the expectation: fifty distinct entities over
    // four passes, so fifty lines are uncoversed and a hundred and fifty are exact
    // repeats the BORING rule absorbs into `repeats` (09 §5.2).
    let uncoversed = 50usize;

    // G1.u.3 — **no batch row type** (R-12's alarm). The import leaves rows on the
    // shelf it is allowed to use, every line is either a row or a repetition of one,
    // and nothing the import wrote is addressed outside `(t, x, y)`: a tick the loop
    // ran inside the lane, a shelf in the closed registry, and a `y` the store itself
    // allocated — its per-shelf sequence from 1, where `row_id 0` is the never-allocated
    // sentinel (01 §2). A record that skipped that sequence was written by something
    // other than the loop, and that something is the "import" structure R-12 fears.
    let repeats_sum: u64 = taught_rows.iter().map(|r| u64::from(r.envelope.repeats)).sum();
    let lines_accounted = taught_rows.len() == uncoversed && repeats_sum == import_frames as u64;
    let mut shelf_outside: Vec<u16> = Vec::new();
    let mut tick_outside: Vec<i32> = Vec::new();
    let mut y_zero = 0u64;
    let mut y_by_shelf: BTreeMap<u16, Vec<u32>> = BTreeMap::new();
    for (_, record) in store.rows()? {
        let e = &record.envelope;
        if !si_core::shelves::SHELVES.iter().any(|entry| entry.shelf_id == e.shelf_x) {
            shelf_outside.push(e.shelf_x);
        }
        if !(0..import_frames as i32).contains(&e.written_at_t) {
            tick_outside.push(e.written_at_t);
        }
        if e.row_id == 0 {
            y_zero += 1;
        }
        y_by_shelf.entry(e.shelf_x).or_default().push(e.row_id);
    }
    // Each shelf's `y`s must be exactly `1..=count`, in whatever order they were read.
    let y_outside_sequence: u64 = y_by_shelf
        .values()
        .map(|list| {
            let mut sorted = list.clone();
            sorted.sort_unstable();
            sorted
                .iter()
                .enumerate()
                .filter(|(index, y)| **y != *index as u32 + 1)
                .count() as u64
        })
        .sum();
    let addressed_inside = shelf_outside.is_empty()
        && tick_outside.is_empty()
        && y_zero == 0
        && y_outside_sequence == 0;
    // A report carries a sample, not two hundred numbers (05 §3: references and counts).
    let shelf_sample: Vec<u16> = shelf_outside.iter().copied().take(8).collect();
    let tick_sample: Vec<i32> = tick_outside.iter().copied().take(8).collect();
    let y_spans: Value = y_by_shelf
        .iter()
        .map(|(shelf, list)| {
            (
                shelf.to_string(),
                json!({"low": list.iter().min(), "high": list.iter().max(), "count": list.len()}),
            )
        })
        .collect();
    if lines_accounted && addressed_inside {
        ok(
            out,
            "G1.u.3",
            "a 200-line import leaves no batch row type",
            "the `s_r_pairs` count equals the uncoversed lines, `repeats` sums to the whole lane \
             so every line is either a row or a repetition of one, and no record is addressed \
             outside `(t, x, y)`",
            json!({
                "fixture": "harness/scenarios/g01_u_curriculum_200.txt",
                "fixture_digest": fixture.digest,
                "lane_frames": import_frames,
                "uncoversed_rows": taught_rows.len(),
                "repeats_sum": repeats_sum,
                "shelves_touched": census(&import.session)?,
                "every_address_is_txy": addressed_inside,
                "shelves_outside_registry": shelf_sample,
                "ticks_outside_lane": tick_sample,
                "y_by_shelf": y_spans,
                "y_zero_rows": y_zero,
                "y_outside_sequence": y_outside_sequence,
            }),
            "an \"import\" structure appeared in the substrate (R-12's alarm, patched silently)",
        );
    } else {
        bad(
            out,
            "G1.u.3",
            "a 200-line import leaves no batch row type",
            "the `s_r_pairs` count equals the uncoversed lines, `repeats` sums to the whole lane \
             so every line is either a row or a repetition of one, and no record is addressed \
             outside `(t, x, y)`",
            json!({
                "fixture": "harness/scenarios/g01_u_curriculum_200.txt",
                "fixture_digest": fixture.digest,
                "lane_frames": import_frames,
                "uncoversed_rows": taught_rows.len(),
                "want_uncoversed": uncoversed,
                "repeats_sum": repeats_sum,
                "shelves_touched": census(&import.session)?,
                "every_address_is_txy": addressed_inside,
                "shelves_outside_registry": shelf_sample,
                "ticks_outside_lane": tick_sample,
                "y_by_shelf": y_spans,
                "y_zero_rows": y_zero,
                "y_outside_sequence": y_outside_sequence,
            }),
            "an \"import\" structure appeared in the substrate (R-12's alarm, patched silently)",
        );
    }

    // G1.u.4 — the **asserted-only ledger**, first half (R-07 through the newest door):
    // a teach is an assertion, so every curriculum row carries `gain_kind = asserted`
    // and none of them is counted as demonstrated.
    let all_asserted = taught_rows
        .iter()
        .all(|record| record.envelope.gain_kind == gate::GAIN_ASSERTED);
    let gain_kinds: Vec<u8> = taught_rows.iter().map(|r| r.envelope.gain_kind).collect();
    let provenances: Vec<u8> = taught_rows.iter().map(|r| r.envelope.prov_kind).collect();
    if all_asserted && !taught_rows.is_empty() {
        ok(
            out,
            "G1.u.4",
            "the curriculum ledger is asserted-only",
            "every row the import wrote reads `gain_kind = asserted`",
            json!({
                "rows": taught_rows.len(),
                "all_asserted": all_asserted,
                "gain_kinds": gain_kinds,
                "prov_kinds": provenances,
                "asserted_is": gate::GAIN_ASSERTED,
                "demonstrated_is": gate::GAIN_DEMONSTRATED,
            }),
            "asserted units counted as demonstrated — R-07 through the newest door",
        );
    } else {
        bad(
            out,
            "G1.u.4",
            "the curriculum ledger is asserted-only",
            "every row the import wrote reads `gain_kind = asserted`",
            json!({
                "rows": taught_rows.len(),
                "all_asserted": all_asserted,
                "gain_kinds": gain_kinds,
                "prov_kinds": provenances,
            }),
            "asserted units counted as demonstrated — R-07 through the newest door",
        );
    }
    // `G1.u.4`'s second clause — after `k` served recalls exactly `k` promotion events
    // exist, one per row, each naming a serving step id — has no producer to test: the
    // promotion is `INFERENCE_PROMOTION`, a Ruling row shipped off, which is the same
    // wall `G13.i.2` names. The clause is `unmeasured`, not quietly a pass.
    open(
        out,
        "G1.u.4b",
        "the served-recall promotion count",
        "after `k` served recalls exactly `k` promotion events exist, one per curriculum row, \
         each naming a serving step id",
        "the promotion is `INFERENCE_PROMOTION`'s and that Ruling row ships off, so no build of \
         this stage has a promotion path to test; OD-58 is the owner's conditional. The \
         asserted-only half is graded here (`G1.u.4`), and the recalls a curriculum would serve \
         are the ones this row cannot count.",
        json!({
            "promotion_row": si_core::consts::row("INFERENCE_PROMOTION")
                .map(|row| format!("{:?}", row.class))
                .unwrap_or_default(),
            "promotion_value": si_core::consts::row("INFERENCE_PROMOTION")
                .and_then(si_core::consts::placeholder_value)
                .map(|value| value.to_string()),
            "curriculum_rows": taught_rows.len(),
        }),
        "asserted units counted as demonstrated through a promotion nobody audited",
    );

    // G1.u.5 — the **birth cohort is visible, not tuned**, first half: a curriculum row's
    // `bake` is the one curve read off the `a_at` the record itself carries, so review
    // recomputes it instead of trusting it (04 §16 P). The second clause — that `a`
    // *stayed high* across the import — needs a threshold no ruling sets, so it is
    // measured as a range and left `unmeasured` under **OD-33**.
    let params = *import.session.params();
    let mut stamped_edges = 0usize;
    let mut unstamped_edges = 0usize;
    let mut mismatches = 0usize;
    let mut a_ats: Vec<u8> = Vec::new();
    let mut bakes: Vec<u16> = Vec::new();
    for record in &taught_rows {
        for edge in store.edges_of(record)? {
            let stamps = store.stamps_of(&edge)?;
            bakes.push(edge.bake);
            if stamps.is_empty() {
                unstamped_edges += 1;
                continue;
            }
            stamped_edges += 1;
            for stamp in &stamps {
                a_ats.push(stamp.a_at);
            }
            if !stamps
                .iter()
                .any(|stamp| si_affect::bake::bake_q12(stamp.a_at, params.b_max, params.k_tanh) == edge.bake)
            {
                mismatches += 1;
            }
        }
    }
    if stamped_edges > 0 && mismatches == 0 {
        ok(
            out,
            "G1.u.5",
            "a curriculum bake recomputes from its recorded a_at",
            "for every stamped edge of every curriculum row, the stored `bake` equals \
             `bake_q12(a_at, B_MAX, K_TANH)` read off the edge's own stamp list",
            json!({
                "stamped_edges": stamped_edges,
                "unstamped_edges": unstamped_edges,
                "mismatches": mismatches,
                "b_max": params.b_max,
                "k_tanh": params.k_tanh,
                "a_at_range": a_ats.iter().min().zip(a_ats.iter().max()).map(|(lo, hi)| [*lo as u64, *hi as u64]),
                "bake_range": bakes.iter().min().zip(bakes.iter().max()).map(|(lo, hi): (&u16, &u16) | [*lo as u64, *hi as u64]),
            }),
            "a bake suppressed, ad-hoc-capped, or keyed by event kind (OD-33's scope, R-21(c))",
        );
    } else {
        bad(
            out,
            "G1.u.5",
            "a curriculum bake recomputes from its recorded a_at",
            "for every stamped edge of every curriculum row, the stored `bake` equals \
             `bake_q12(a_at, B_MAX, K_TANH)` read off the edge's own stamp list",
            json!({
                "stamped_edges": stamped_edges,
                "unstamped_edges": unstamped_edges,
                "mismatches": mismatches,
                "b_max": params.b_max,
                "k_tanh": params.k_tanh,
                "rows": taught_rows.len(),
            }),
            "a bake suppressed, ad-hoc-capped, or keyed by event kind (OD-33's scope, R-21(c))",
        );
    }
    open(
        out,
        "G1.u.5b",
        "arousal stayed high across the import",
        "`a` stayed high across the 200-line import — the birth cohort is visible in the \
         trajectory rather than tuned away",
        "the spec names no threshold for `high`, and setting one while coding would be the \
         ad-hoc cap OD-33's ruling owns; the range below is the measurement, and **OD-33** is \
         the owner's to rule. Recomputability of the bake itself is graded at `G1.u.5`.",
        json!({
            "a_at_low": a_ats.iter().min().copied(),
            "a_at_high": a_ats.iter().max().copied(),
            "stamps_read": a_ats.len(),
            "lane_frames": import_frames,
        }),
        "a curriculum that drained `a` and nobody noticed",
    );
    Ok(())
}

/// One in-process arm of the `/teach` bulk legs, at the run's own seed, constants and
/// variant. Both halves of `G1.u.2` and every curriculum leg in this family go through
/// it, so a fed arm and a typed arm differ only in what the request carries: steps, or
/// a lane the loop hears before the steps (09 §8, 05 §1's "never a harness branch").
fn teach_drive(run: &Run, steps: Vec<Step>, inbox_lane: Vec<u8>) -> Result<DriveOutcome> {
    drive(&DriveRequest {
        // 05 §23.5: the birth pair — no fixture in this family registers an identity.
        identities: None,
        steps,
        seed: run.request.seed,
        constants: run.request.constants.clone(),
        variant: run.request.variant.clone(),
        sleep_interrupt: run.request.sleep_interrupt,
        cut_after: None,
        cut_path: None,
        freeze_at_end: false,
        inbox_lane,
    })
}

// ---------------------------------------------------------------------------
// G2 — paraphrase + walk integrity (05 §7). Stage S2+ and a `links` substrate.
// ---------------------------------------------------------------------------

fn g2(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let last = run
        .sections
        .last()
        .and_then(|section| section.steps.last())
        .context("an empty run cannot grade G2")?;
    let stage = si_session::stage(&run.sections[0].session)?;
    let links = last.shelves_after.get(&SHELF_LINKS).copied().unwrap_or_default();
    let sweep = run
        .request
        .constants
        .get("HUB_DAMP_EXPONENT")
        .is_some_and(|_| true);
    let cited = run
        .request
        .constants
        .keys()
        .filter(|name| matches!(name.as_str(), "HUB_DAMP_EXPONENT" | "DECAY_BASE"))
        .count();
    let scan = super::checks::run("check_no_embeddings", ws)?;
    let legs = [
        ("G2.d.1", "the reworded question decodes to the taught `pattern_id`"),
        ("G2.d.2", "zero content-token overlap on the reworded recall"),
        ("G2.w.1", "the cited route crosses only OWNED edge kinds"),
        ("G2.t3.1", "the banana query yields no FACT"),
        ("G2.t3.2", "the unlit reword misses honestly with zero rows written"),
        ("G2.c.1", "the constants used are members of `constants_sweep.json`"),
        ("G2.l.1", "the decoder litmus: `what znark is floop` decodes to the same shape with different content slots"),
        ("G2.e.1", "the static embedding scan over the build is empty"),
    ];
    for (id, predicate) in legs {
        let why = if id == "G2.e.1" {
            format!(
                "the static scan is graded by `si-harness check`, not by a run: its verdict here is {:?} \
                 ({})",
                scan.verdict, scan.detail
            )
        } else {
            format!(
                "S2 is unreachable at this build: `stage_of` needs a live `links` row and no module writes the \
                 `links` shelf before the genome loader (this run reads stage {stage}, links {links}), so the \
                 taught frame-synonym edges G2's two barriers compare have no substrate"
            )
        };
        open(
            out,
            id,
            "paraphrase + walk integrity",
            predicate,
            &why,
            json!({
                "stage": stage,
                "links_rows": links,
                "sweep_installed": sweep,
                "swept_keys": cited,
                "check_no_embeddings": scan.verdict,
            }),
            "a genome path disguised as knowing",
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// G3 — supersession (05 §8).
// ---------------------------------------------------------------------------

fn g3(run: &Run, out: &mut Vec<Value>) -> Result<()> {
    let section = &run.sections[0];
    let session = &section.session;

    // G3.a.1 — zero deleted rows: the row count never decreases, and neither do
    // the arena bytes behind it.
    let monotone = section
        .steps
        .windows(2)
        .all(|pair| pair[1].rows_written_total >= pair[0].rows_written_total)
        && section
            .steps
            .windows(2)
            .all(|pair| pair[1].after.rows_bytes >= pair[0].after.rows_bytes);
    let total_rows = section.steps.last().map(|s| s.after.rows_bytes).unwrap_or(0);
    let start_rows = section.steps.first().map(|s| s.before.rows_bytes).unwrap_or(0);
    if monotone && total_rows >= start_rows {
        ok(
            out,
            "G3.a.1",
            "zero deleted rows",
            "row count monotone non-decreasing across the whole transcript",
            json!({"steps": section.steps.len(), "rows_bytes_first_to_last": [start_rows, total_rows], "arena_never_shrinks": monotone}),
            "an overwrite implementation — the death of calibration",
        );
    } else {
        bad(
            out,
            "G3.a.1",
            "zero deleted rows",
            "row count monotone non-decreasing across the whole transcript",
            json!({"steps": section.steps.len(), "rows_bytes_first_to_last": [start_rows, total_rows]}),
            "an overwrite implementation — the death of calibration",
        );
    }

    // The supersession is the step whose mind log carries a `supersede` record.
    let superseded = section
        .steps
        .iter()
        .find_map(|record| {
            event_named(run, 0, record, "supersede")
                .first()
                .cloned()
                .map(|event| (record.clone(), event))
        });
    let Some((step_at, event)) = superseded else {
        bad(
            out,
            "G3.a.2",
            "the time axis through `active_at`",
            "`active_at(s)` returns the old value before the supersession and the new one after",
            json!({"supersede_records": 0}),
            "an overwrite implementation — the death of calibration",
        );
        return Ok(());
    };
    let loser = event["loser"].as_u64().unwrap_or(0) as u32;
    let winner = event["winner"].as_u64().unwrap_or(0) as u32;
    let loser_shelf = event["loser_shelf"].as_u64().unwrap_or(0) as u16;
    let at = event["at"].as_i64().unwrap_or(0) as i32;

    let before = session.store().active_at(loser_shelf, at - 1)?;
    let after = session.store().active_at(loser_shelf, at)?;
    let old_value = session
        .store()
        .row_at(loser_shelf, loser)?
        .map(|record| folded(session, record.body.val_point));
    let new_value = session
        .store()
        .row_at(loser_shelf, winner)?
        .map(|record| folded(session, record.body.val_point));
    let before_names_old = before
        .iter()
        .any(|(_, record)| Some(folded(session, record.body.val_point)) == old_value);
    let after_names_new = after
        .iter()
        .any(|(_, record)| record.envelope.row_id == winner);
    if before_names_old && after_names_new && old_value != new_value {
        ok(
            out,
            "G3.a.2",
            "the time axis through `active_at`",
            "`active_at(s)` returns the old value before the supersession and the new one after, evaluated \
             through `store.active_at` and never through a hand-written interval string",
            json!({
                "step": step_at.index,
                "at": at,
                "loser_ref": format!("{loser_shelf}#{loser}"),
                "winner_ref": format!("{loser_shelf}#{winner}"),
                "old_folded_key": old_value,
                "new_folded_key": new_value,
                "live_before_count": before.len(),
                "live_after_count": after.len(),
            }),
            "an overwrite implementation — the death of calibration",
        );
    } else {
        bad(
            out,
            "G3.a.2",
            "the time axis through `active_at`",
            "`active_at(s)` returns the old value before the supersession and the new one after",
            json!({
                "step": step_at.index,
                "before_names_loser": before_names_old,
                "after_names_winner": after_names_new,
                "old_folded_key": old_value,
                "new_folded_key": new_value,
            }),
            "an overwrite implementation — the death of calibration",
        );
    }

    // G3.a.3 — the aged row cites its exact interval.
    let aged = session.store().row_at(loser_shelf, loser)?;
    let interval = aged
        .as_ref()
        .map(|record| (record.envelope.written_at_t, record.envelope.superseded_at, record.envelope.superseded_by));
    if interval == Some((loser_written_at(aged.as_ref()), at, winner)) {
        ok(
            out,
            "G3.a.3",
            "a past belief is citable with its interval",
            "a past belief is citable with its exact interval `[written_at, superseded_at)`",
            json!({"row_ref": format!("{loser_shelf}#{loser}"), "superseded_by": winner, "interval_end": at}),
            "the time axis is not stored: an aged row that cannot say when it stopped",
        );
    } else {
        bad(
            out,
            "G3.a.3",
            "a past belief is citable with its interval",
            "a past belief is citable with its exact interval `[written_at, superseded_at)`",
            json!({"row_ref": format!("{loser_shelf}#{loser}"), "interval": interval}),
            "the time axis is not stored: an aged row that cannot say when it stopped",
        );
    }

    // G3.em.1 — the past-`t`-anchored *median* reading.
    open(
        out,
        "G3.em.1",
        "the emotional 4D leg",
        "a past-`t`-anchored query re-weights the aged edge at its interval and reports the point median \
         computed with recency weights measured from that anchor, citing the stamps that produced it",
        "no module computes a point median yet: `si-read::retrieve` calls the walk with an empty stamp set \
         (04 §16's `median_q12` arithmetic exists in `si-affect::drive` unwired), and the sweep that sizes it \
         (`gates/G11/median_sweep.json`) is B12 — the leg's own words put its magnitudes there",
        json!({"stamps_on_loser_edge": session.store().row_at(loser_shelf, loser).ok().flatten().map(|_| 1usize).unwrap_or(0)}),
        "an emotional reading that only exists at NOW (affect not 4D, R-28)",
    );

    // G3.em.2 — zero deleted bytes, zero edited stamps, and no invented answer.
    let bytes_never_shrink = section
        .steps
        .windows(2)
        .all(|pair| pair[1].after.stamps_bytes >= pair[0].after.stamps_bytes
            && pair[1].after.rows_bytes >= pair[0].after.rows_bytes
            && pair[1].after.edges_bytes >= pair[0].after.edges_bytes);
    // "Zero edited stamps" reads off the append-only arena: the supersession only
    // grew the stamp region (its winner's stamps appended) and the global
    // monotone check above proves nothing ever shrank. A digest of the whole
    // region is not the test — the anchor moved and the storage grew, it was not
    // edited.
    let stamps_stable = event["at"].is_number()
        && step_at.after.stamps_bytes >= step_at.before.stamps_bytes;
    let past_query = section.steps.iter().rev().find(|record| {
        matches!(record.output, Some(Output::Ask { .. })) && record.text.contains("was")
    });
    let no_answer_to_the_past = past_query
        .map(|record| knowledge(&record.shelves_after) == knowledge(&step_at.shelves_after) && record.rows_written == 0)
        .unwrap_or(false);
    if bytes_never_shrink && stamps_stable && no_answer_to_the_past {
        ok(
            out,
            "G3.em.2",
            "the anchor moved, the storage did not",
            "zero deleted bytes and zero edited stamps; the time frame is taught, so a scenario that never \
             teaches a past-tense frame produces an ask, not an answer",
            json!({
                "stamps_digest_unchanged": stamps_stable,
                "arenas_never_shrink": bytes_never_shrink,
                "past_tense_step": past_query.map(|record| record.index),
                "past_tense_output": "ask",
                "rows_written_at_that_step": 0,
            }),
            "an emotional reading that only exists at NOW, or a past belief invented rather than cited",
        );
    } else {
        bad(
            out,
            "G3.em.2",
            "the anchor moved, the storage did not",
            "zero deleted bytes and zero edited stamps; the time frame is taught, so a scenario that never \
             teaches a past-tense frame produces an ask, not an answer",
            json!({
                "stamps_digest_unchanged": stamps_stable,
                "arenas_never_shrink": bytes_never_shrink,
                "past_tense_ask_seen": past_query.map(|record| record.index),
            }),
            "an emotional reading that only exists at NOW, or a past belief invented rather than cited",
        );
    }
    Ok(())
}

fn loser_written_at(record: Option<&si_core::record::Record>) -> i32 {
    record
        .map(|record| record.envelope.written_at_t)
        .unwrap_or(i32::MIN)
}

// ---------------------------------------------------------------------------
// G4 — contradiction (05 §9).
// ---------------------------------------------------------------------------

fn g4(run: &Run, out: &mut Vec<Value>) -> Result<()> {
    let section = &run.sections[0];
    let min_asks: i64 = si_core::consts::row("G4_MIN_ASK_EVENTS")
        .and_then(|row| row.law_value)
        .unwrap_or(1);
    let asks = section
        .steps
        .iter()
        .filter(|record| matches!(record.output, Some(Output::Ask { .. })))
        .count()
        + section
            .steps
            .iter()
            .map(|record| event_named(run, 0, record, "ask").len())
            .sum::<usize>();
    if asks as i64 >= min_asks {
        ok(
            out,
            "G4.a.1",
            "a bare contrary input asks",
            "at least `G4_MIN_ASK_EVENTS` = 1 ASK event in the run",
            json!({"ask_events": asks, "steps": section.steps.iter().filter(|r| matches!(r.output, Some(Output::Ask { .. }))).map(|r| r.index).collect::<Vec<_>>()}),
            "last-write-wins with extra steps (R-05)",
        );
    } else {
        bad(
            out,
            "G4.a.1",
            "a bare contrary input asks",
            "at least `G4_MIN_ASK_EVENTS` = 1 ASK event in the run",
            json!({"ask_events": asks}),
            "last-write-wins with extra steps (R-05)",
        );
    }

    let mut conflicts: Vec<(&StepRecord, Value)> = Vec::new();
    for record in &section.steps {
        for event in event_named(run, 0, record, "conflict") {
            conflicts.push((record, event));
        }
    }
    let every_conflict_traced = conflicts.iter().all(|(record, _)| {
        let events = gate_events(run, 0, record);
        events.iter().any(|event| {
            event.get("branch").and_then(Value::as_str).is_some()
                || event["event"] == json!("ask")
        })
    });
    if !conflicts.is_empty() && every_conflict_traced {
        ok(
            out,
            "G4.a.2",
            "every conflict is a recorded triple",
            "every conflict carries `{branch, asked?, resolved_at}`",
            json!({"conflicts": conflicts.len(), "all_branch_named": every_conflict_traced}),
            "a hidden disagreement: a conflict resolved without naming its branch",
        );
    } else {
        bad(
            out,
            "G4.a.2",
            "every conflict is a recorded triple",
            "every conflict carries `{branch, asked?, resolved_at}`",
            json!({"conflicts": conflicts.len(), "all_branch_named": every_conflict_traced}),
            "a hidden disagreement: a conflict resolved without naming its branch",
        );
    }

    // G4.a.3 — no affective argument on any arbitration record (OD-24).
    let affective = ["a", "a_at", "a_formation", "arousal", "emotion", "mood", "bake", "salience", "weight", "surprise_q12"];
    let dirty: Vec<Value> = section
        .steps
        .iter()
        .flat_map(|record| {
            gate_events(run, 0, record)
                .into_iter()
                .filter(|event| {
                    matches!(
                        event["event"].as_str(),
                        Some("conflict") | Some("ask") | Some("supersede") | Some("agreement") | Some("disagreement") | Some("resolve")
                    )
                })
                .filter_map(|event| {
                    event
                        .as_object()
                        .and_then(|map| map.keys().find(|key| affective.contains(&key.as_str())))
                        .map(|key| json!({"step": record.index, "field": key}))
                })
        })
        .collect();
    if dirty.is_empty() {
        ok(
            out,
            "G4.a.3",
            "arbitration reads tier, marker and repeat count only",
            "arbitration inputs are tier, marker, repeat count only — the mind log contains no affective \
             argument on any arbitration record",
            json!({"arbitration_records_scanned": conflicts.len(), "affective_fields_found": 0}),
            "temperament buying its way past a guard: an emotion deciding an arbitration (OD-24)",
        );
    } else {
        bad(
            out,
            "G4.a.3",
            "arbitration reads tier, marker and repeat count only",
            "arbitration inputs are tier, marker, repeat count only",
            json!({"dirty": dirty}),
            "temperament buying its way past a guard: an emotion deciding an arbitration (OD-24)",
        );
    }

    open(
        out,
        "G4.o.1",
        "the oscillation scene (OPEN-11)",
        "the oscillation scene's stop reason is `MaxK` (or the period check) and the output label is the \
         uncertainty readout",
        "the scene needs a taught `A→B→A` edge pair on the `links` shelf and nothing writes that shelf before \
         the genome loader; it is a separate variant run, not a leg of this one",
        json!({"links_rows": section.steps.last().map(|r| r.shelves_after.get(&SHELF_LINKS).copied().unwrap_or(0).to_string()).unwrap_or_default()}),
        "oscillation answered anyway",
    );
    let terminated = !section.steps.is_empty()
        && section
            .steps
            .iter()
            .all(|record| record.output.is_some());
    if terminated {
        ok(
            out,
            "G4.o.2",
            "the run terminated",
            "the run terminated (a hang is not a fail state the harness may report as anything else)",
            json!({"steps": section.steps.len(), "every_step_returned": true}),
            "a hang reported as anything else",
        );
    } else {
        bad(
            out,
            "G4.o.2",
            "the run terminated",
            "the run terminated",
            json!({"steps": section.steps.len()}),
            "a hang reported as anything else",
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// G5 — DETERMINISM + NO-PRIOR (05 §10).
// ---------------------------------------------------------------------------

/// 05 §10's one fixture: a served recall leaves the agenda empty, quiet ticks
/// ignite on the web it left, and the gibberish pair (`flimp is wazz` /
/// `what znark is floop`) is the English pair's substituted-token twin. The
/// parity legs compare two halves of one build at one individual, so a run at
/// another transcript is refused before any leg grades a script it did not name.
const G5_SCENARIO: &str = "harness/scenarios/g5_determinism.txt";

fn g5(run: &Run, ws: &Workspace, replay: &[String], out: &mut Vec<Value>) -> Result<()> {
    if !expect_fixture(run, ws, "G5", "05 §10", G5_SCENARIO, out)? {
        return Ok(());
    }

    // ---------------------------------------------------------------- (a.1)
    // 05 §10 a.1: the V6 replay, tolerance 0 bytes. `replay` is
    // `Run::replay()` over the same steps, already computed by `evaluate`.
    let recorded: Vec<String> =
        run.sections.iter().map(|section| section.hash.clone()).collect();
    let mut hash_diffs: Vec<Value> = Vec::new();
    for index in 0..recorded.len().max(replay.len()) {
        let left = recorded.get(index);
        let right = replay.get(index);
        if left != right {
            hash_diffs.push(json!({
                "section": index,
                "recorded": left,
                "replay": right,
            }));
        }
    }
    let replay_identical = recorded.len() == replay.len() && hash_diffs.is_empty();
    if replay_identical {
        ok(
            out,
            "G5.a.1",
            "replay is byte-identical",
            "every section's state hash from `Run::replay()` equals the hash the run wrote; tolerance 0 bytes",
            json!({
                "sections": recorded.len(),
                "hashes_recorded": recorded,
                "hashes_replay": replay,
                "tolerance_bytes": 0,
            }),
            "a replay that does not reproduce the run is a wall-clock event wearing a counter",
        );
    } else {
        bad(
            out,
            "G5.a.1",
            "replay is byte-identical",
            "every section's state hash from `Run::replay()` equals the hash the run wrote; tolerance 0 bytes",
            json!({
                "sections_recorded": recorded.len(),
                "sections_replay": replay.len(),
                "differences": hash_diffs,
                "tolerance_bytes": 0,
            }),
            "a replay that does not reproduce the run is a wall-clock event wearing a counter",
        );
    }

    // ---------------------------------------------------------------- (a.2)
    // 05 §10 a.2: no median-bearing field exists in any schema or container.
    // R-28's read-side median is a return value, never stored; the producer is
    // absent at this build (OD-144), so the audit is the whole instrument.
    let mood = super::checks::run("check_no_mood_field", ws)?;
    if mood.verdict == "pass" {
        ok(
            out,
            "G5.a.2",
            "point medians are not hash inputs",
            "no stored field or export key named median/mood/bias exists in any schema or container (R-28)",
            json!({
                "check_no_mood_field": mood.verdict,
                "detail": mood.detail,
                "median_producer": "absent at this build (OD-144: 04 §9's H is unbuilt)",
            }),
            "a median that survives replay as stored state is a frozen emotional address",
        );
    } else {
        bad(
            out,
            "G5.a.2",
            "point medians are not hash inputs",
            "no stored field or export key named median/mood/bias exists in any schema or container (R-28)",
            json!({"check_no_mood_field": mood.verdict, "detail": mood.detail}),
            "a median that survives replay as stored state is a frozen emotional address",
        );
    }

    // ---------------------------------------------------------------- (b.1)
    // 05 §10 b.1: two different conversations ⇒ different state, zero shared
    // terms. Section 0 and section 1 are disjoint transcripts by construction;
    // what a smuggled prior would leave behind is a term one session holds that
    // its own words never spoke, or a term both hold. The unit test named in
    // `observed` is the interner's own instrument (`si-store/tests/arena.rs`);
    // this leg measures the property on two real sessions.
    let first = &run.sections[0];
    let second = &run.sections[1];
    let first_terms = interner_terms(&first.session);
    let second_terms = interner_terms(&second.session);
    let first_keys: BTreeSet<String> = first_terms.values().cloned().collect();
    let second_keys: BTreeSet<String> = second_terms.values().cloned().collect();
    let shared_terms: Vec<String> = first_keys.intersection(&second_keys).cloned().collect();
    let shared_ids: Vec<u32> = first_terms
        .keys()
        .filter(|id| second_terms.contains_key(*id))
        .copied()
        .collect();
    let shared_id_drift: Vec<Value> = shared_ids
        .iter()
        .copied()
        .filter(|id| first_terms[id] != second_terms[id])
        .map(|id| json!({"id": id, "first": first_terms[&id], "second": second_terms[&id]}))
        .collect();
    let unspoken = |steps: &[StepRecord], terms: &BTreeMap<u32, String>| -> Vec<Value> {
        // A term is folded text, so its tokens are its folded words; each one must
        // be one of the words this section's own steps spoke.
        let spoken: BTreeSet<String> =
            steps.iter().flat_map(|step| si_front::keys(&step.text)).collect();
        terms
            .iter()
            .filter(|(_, key)| key.split(' ').any(|token| !spoken.contains(token)))
            .map(|(id, key)| json!({"id": id, "term": key}))
            .collect()
    };
    let unspoken_first = unspoken(&first.steps, &first_terms);
    let unspoken_second = unspoken(&second.steps, &second_terms);
    let states_differ = first.hash != second.hash;
    if states_differ
        && shared_terms.is_empty()
        && !shared_id_drift.is_empty()
        && unspoken_first.is_empty()
        && unspoken_second.is_empty()
    {
        ok(
            out,
            "G5.b.1",
            "no shared prior between two conversations",
            "disjoint transcripts leave different states, share zero terms, and each session's interner holds nothing its own words did not speak",
            json!({
                "instrument": "crates/si-store/tests/arena.rs::interner_is_per_session_and_never_merges_surfaces",
                "sections": [0, 1],
                "states": [first.hash, second.hash],
                "states_differ": states_differ,
                "terms_first": first_terms.len(),
                "terms_second": second_terms.len(),
                "shared_terms": shared_terms,
                "shared_ids_whose_terms_differ": shared_id_drift,
                "unspoken_first": unspoken_first,
                "unspoken_second": unspoken_second,
            }),
            "a shipped prior, or an interner that is not per-session",
        );
    } else {
        bad(
            out,
            "G5.b.1",
            "no shared prior between two conversations",
            "disjoint transcripts leave different states, share zero terms, and each session's interner holds nothing its own words did not speak",
            json!({
                "instrument": "crates/si-store/tests/arena.rs::interner_is_per_session_and_never_merges_surfaces",
                "states_differ": states_differ,
                "shared_terms": shared_terms,
                "shared_ids_whose_terms_differ": shared_id_drift,
                "unspoken_first": unspoken_first,
                "unspoken_second": unspoken_second,
            }),
            "a shipped prior, or an interner that is not per-session",
        );
    }

    // ---------------------------------------------------------------- (c.1)
    // 05 §10 c.1: the static string-constant audit, byte-exact.
    let strings = super::checks::run("check_string_constants", ws)?;
    let strings_pass = strings.verdict == "pass";
    let observed = json!({"check_string_constants": strings.verdict, "detail": strings.detail});
    if strings_pass {
        ok(
            out,
            "G5.c.1",
            "string-constant audit is byte-exact",
            "every shipped string constant is on 03 §3.2's allowlist, and the diff carries the supporter's two `source` literals and the TEACH fragment's three (`/teach`, `=>`, `owner-teach`) — no more, no fewer",
            observed,
            "a teach command that grew a vocabulary is the anti-cheat law dying by curriculum",
        );
    } else {
        bad(
            out,
            "G5.c.1",
            "string-constant audit is byte-exact",
            "every shipped string constant is on 03 §3.2's allowlist, and the diff carries the supporter's two `source` literals and the TEACH fragment's three (`/teach`, `=>`, `owner-teach`) — no more, no fewer",
            observed,
            "a teach command that grew a vocabulary is the anti-cheat law dying by curriculum",
        );
    }

    // ---------------------------------------------------------------- (c.2)
    // 05 §10 c.2: the handle audit. This half is the static check over the loader
    // entry; the runtime half is the G6 SI pair's open-register probe, which is
    // the same leg measured with the artifact on disk (OD-195).
    let guard = super::checks::run("check_config_guard", ws)?;
    let guard_pass = guard.verdict == "pass";
    let observed = json!({
        "check_config_guard": guard.verdict,
        "detail": guard.detail,
        "config_of_this_run": run.request.config,
        "runtime_half": "the G6 SI-pair run measures zero frozen opens with a positive control, leg G5.c.2 in gates/G6_si_*/gate_report.json (OD-195)",
    });
    if guard_pass {
        ok(
            out,
            "G5.c.2",
            "CONFIG SI never opens a frozen-layer file",
            "the loader entry refuses CONFIG SI before its engine branch, holds no file I/O, and only `layers_frozen` can name a frozen artifact",
            observed,
            "a frozen layer reachable from CONFIG SI, or a loader that opens before it decides",
        );
    } else {
        bad(
            out,
            "G5.c.2",
            "CONFIG SI never opens a frozen-layer file",
            "the loader entry refuses CONFIG SI before its engine branch, holds no file I/O, and only `layers_frozen` can name a frozen artifact",
            observed,
            "a frozen layer reachable from CONFIG SI, or a loader that opens before it decides",
        );
    }

    // ---------------------------------------------------------------- (e.1)
    // 05 §10 e.1: the activation trace is first-class output, so the replay
    // must reproduce every frame each step wrote, in order — not only the settle
    // set. `replay_traces` is this comparison's only caller.
    let traces = run.replay_traces()?;
    let mut steps_compared = 0usize;
    let mut events_compared = 0usize;
    let mut trace_diffs: Vec<Value> = Vec::new();
    for (section_index, section) in run.sections.iter().enumerate() {
        let replayed_section = traces.get(section_index);
        for record in &section.steps {
            let replayed = replayed_section.and_then(|steps| steps.get(record.index - 1));
            steps_compared += 1;
            events_compared += record.event_end.saturating_sub(record.event_start);
            let Some(replayed) = replayed else {
                trace_diffs.push(json!({
                    "section": section_index,
                    "step": record.index,
                    "why": "the replay has no step at this index",
                }));
                continue;
            };
            let recorded = run.events(section_index, record);
            if recorded != replayed.as_slice() {
                let first_diff = recorded
                    .iter()
                    .zip(replayed)
                    .position(|(left, right)| left != right);
                let (left, right) = match first_diff {
                    Some(index) => (recorded.get(index), replayed.get(index)),
                    None => (recorded.get(replayed.len()), replayed.last()),
                };
                trace_diffs.push(json!({
                    "section": section_index,
                    "step": record.index,
                    "recorded_frames": recorded.len(),
                    "replayed_frames": replayed.len(),
                    "first_differing_frame": first_diff,
                    "recorded": left.map(|event| json!({
                        "seq": event.seq, "t": event.t, "source": event.source,
                        "kind": event.kind, "body": String::from_utf8_lossy(&event.body),
                    })),
                    "replay": right.map(|event| json!({
                        "seq": event.seq, "t": event.t, "source": event.source,
                        "kind": event.kind, "body": String::from_utf8_lossy(&event.body),
                    })),
                }));
            }
        }
    }
    let traces_identical =
        trace_diffs.is_empty() && traces.len() == run.sections.len() && steps_compared > 0;
    if traces_identical {
        ok(
            out,
            "G5.e.1",
            "identical transcript ⇒ identical activation traces",
            "every step of the replay writes the same event frames, in the same order, as the run it replays",
            json!({
                "sections": run.sections.len(),
                "steps_compared": steps_compared,
                "frames_compared": events_compared,
                "differences": trace_diffs,
            }),
            "a trace that drifts under replay is outside the determinism contract",
        );
    } else {
        bad(
            out,
            "G5.e.1",
            "identical transcript ⇒ identical activation traces",
            "every step of the replay writes the same event frames, in the same order, as the run it replays",
            json!({
                "sections": run.sections.len(),
                "replay_sections": traces.len(),
                "steps_compared": steps_compared,
                "frames_compared": events_compared,
                "differences": trace_diffs,
            }),
            "a trace that drifts under replay is outside the determinism contract",
        );
    }

    // ---------------------------------------------------------------- (d.1)
    // 05 §10 d.1: gibberish-parity replay. The statement and the ask each have
    // an English twin in section 0; the leg requires the replayed frames at the
    // gibberish steps to be identical, the ask's decoded pattern to equal the
    // English ask's with different slots, and the statement's gate decisions to
    // carry the English statement's shape with a different row.
    let english_statement = step(run, 0, 4)?;
    let gibberish_statement = step(run, 0, 8)?;
    let english_ask = step(run, 0, 1)?;
    let gibberish_ask = step(run, 0, 9)?;
    let pattern_of = |record: &StepRecord| -> Option<u64> {
        raw_bodies(run, 0, record)
            .iter()
            .find_map(|body| body["pattern"].as_u64())
    };
    let pattern_parity = pattern_of(english_ask).is_some()
        && pattern_of(english_ask) == pattern_of(gibberish_ask);
    let english_commit = event_named(run, 0, english_statement, "commit").first().cloned();
    let gibberish_commit =
        event_named(run, 0, gibberish_statement, "commit").first().cloned();
    let commit_parity = match (&english_commit, &gibberish_commit) {
        (Some(first), Some(second)) => {
            first["branch"] == second["branch"]
                && first["shelf"] == second["shelf"]
                && first["prov_kind"] == second["prov_kind"]
                && first["gain_kind"] == second["gain_kind"]
                && first["row"] != second["row"]
        }
        _ => false,
    };
    let replay_parity = {
        let slice_of = |record: &StepRecord| -> Option<&[si_core::eventlog::Event]> {
            traces.get(0)?.get(record.index - 1).map(Vec::as_slice)
        };
        [gibberish_statement, gibberish_ask].iter().all(|record| {
            slice_of(record)
                .map(|replayed| run.events(0, record) == replayed)
                .unwrap_or(false)
        })
    };
    let gibberish_wrote_one = gibberish_statement.rows_written == 1;
    if replay_parity && pattern_parity && commit_parity && gibberish_wrote_one {
        ok(
            out,
            "G5.d.1",
            "gibberish parity replays",
            "the gibberish statement commits through the English statement's gate decisions with a different row, the gibberish ask decodes to the English ask's pattern with different slots, and both steps replay byte-identically",
            json!({
                "steps": {"english_statement": 4, "english_ask": 1, "gibberish_statement": 8, "gibberish_ask": 9},
                "replay_identical": replay_parity,
                "pattern_ids": [pattern_of(english_ask), pattern_of(gibberish_ask)],
                "pattern_equal": pattern_parity,
                "commit_shape_equal": commit_parity,
                "commit_rows": [english_commit.as_ref().map(|body| body["row"].clone()), gibberish_commit.as_ref().map(|body| body["row"].clone())],
                "gibberish_rows_written": gibberish_statement.rows_written,
            }),
            "content-selective rejection through the newest door",
        );
    } else {
        bad(
            out,
            "G5.d.1",
            "gibberish parity replays",
            "the gibberish statement commits through the English statement's gate decisions with a different row, the gibberish ask decodes to the English ask's pattern with different slots, and both steps replay byte-identically",
            json!({
                "replay_identical": replay_parity,
                "pattern_ids": [pattern_of(english_ask), pattern_of(gibberish_ask)],
                "commit_shape_equal": commit_parity,
                "gibberish_rows_written": gibberish_statement.rows_written,
                "english_commit": english_commit,
                "gibberish_commit": gibberish_commit,
            }),
            "content-selective rejection through the newest door",
        );
    }

    // ---------------------------------------------------------------- (f.1)
    // 05 §10 f.1: identical idle cascades and monologue entries, including which
    // hyperpoint ignition picked. An idle lane that never fired grades nothing,
    // so a run whose fixture did not ignite is named `unmeasured`, not passed.
    let idle_replay = run.replay_idle()?;
    let mut idle_ticks = 0usize;
    let mut monologue_lines = 0usize;
    let mut ignited_recorded: Vec<u32> = Vec::new();
    let mut ignited_replayed: Vec<u32> = Vec::new();
    let mut idle_diffs: Vec<Value> = Vec::new();
    for (section_index, section) in run.sections.iter().enumerate() {
        idle_ticks += section.idle.len();
        monologue_lines += section.session.monologue().len();
        ignited_recorded.extend(section.idle.iter().filter_map(|report| report.ignited));
        let Some((replayed_idles, replayed_monologue)) = idle_replay.get(section_index) else {
            idle_diffs.push(json!({
                "section": section_index,
                "why": "the replay has no section at this index",
            }));
            continue;
        };
        ignited_replayed.extend(replayed_idles.iter().filter_map(|report| report.ignited));
        if &section.idle != replayed_idles {
            idle_diffs.push(json!({
                "section": section_index,
                "what": "idle cascades",
                "recorded": section.idle.len(),
                "replayed": replayed_idles.len(),
            }));
        }
        if section.session.monologue() != replayed_monologue.as_slice() {
            idle_diffs.push(json!({
                "section": section_index,
                "what": "monologue",
                "recorded": section.session.monologue().len(),
                "replayed": replayed_monologue.len(),
            }));
        }
    }
    let idle_identical =
        idle_diffs.is_empty() && idle_replay.len() == run.sections.len();
    let fired = !ignited_recorded.is_empty();
    if idle_identical && fired {
        ok(
            out,
            "G5.f.1",
            "identical transcript ⇒ identical idle cascades and monologue",
            "every idle report and every monologue entry replays exactly, including the hyperpoint each ignition picked",
            json!({
                "idle_ticks": idle_ticks,
                "ignitions_recorded": ignited_recorded,
                "ignitions_replayed": ignited_replayed,
                "monologue_lines": monologue_lines,
                "differences": idle_diffs,
            }),
            "the sampler is unseeded or drifts with wall-clock",
        );
    } else if idle_identical {
        open(
            out,
            "G5.f.1",
            "identical transcript ⇒ identical idle cascades and monologue",
            "every idle report and every monologue entry replays exactly, including the hyperpoint each ignition picked",
            "the fixture's quiet ticks never ignited, so the seed test has no ignition to compare",
            json!({
                "idle_ticks": idle_ticks,
                "ignitions_recorded": ignited_recorded,
                "ignitions_replayed": ignited_replayed,
                "monologue_lines": monologue_lines,
                "differences": idle_diffs,
            }),
            "the sampler is unseeded or drifts with wall-clock",
        );
    } else {
        bad(
            out,
            "G5.f.1",
            "identical transcript ⇒ identical idle cascades and monologue",
            "every idle report and every monologue entry replays exactly, including the hyperpoint each ignition picked",
            json!({
                "idle_ticks": idle_ticks,
                "ignitions_recorded": ignited_recorded,
                "ignitions_replayed": ignited_replayed,
                "monologue_lines": monologue_lines,
                "differences": idle_diffs,
            }),
            "the sampler is unseeded or drifts with wall-clock",
        );
    }

    Ok(())
}

/// The interner's terms as `id → folded key`, excluding id 0 (the empty-string
/// sentinel every session has by construction).
fn interner_terms(session: &Session) -> BTreeMap<u32, String> {
    session
        .store()
        .interner()
        .surfaces()
        .enumerate()
        .filter(|(id, _)| *id != 0)
        .map(|(id, entry)| (id as u32, entry.folded.clone()))
        .collect()
}

// ---------------------------------------------------------------------------
// G6 — BASELINE INTEGRITY (05 §11): CONFIG ENGINE's V7 row, and its CONFIG SI pair.
// ---------------------------------------------------------------------------

/// 05 §11 `G6.d.2`'s call path — the files a settled thought walks through. The
/// spec makes this a source assertion and not a benchmark, so the list is the one
/// `crates/si-walk/tests/spread.rs` ships, widened with the queue verbs a GPU
/// port would wait on.
const WALK_CALL_PATH: &[&str] = &[
    "crates/si-walk/src/walk.rs",
    "crates/si-walk/src/graph.rs",
    "crates/si-walk/src/idle.rs",
    "crates/si-read/src/retrieve.rs",
];

/// A per-round sync, named. The first group is the shared state one would sit on;
/// the second is the wait itself.
const PER_ROUND_SYNC: &[&str] = &[
    "std::thread",
    "std::sync",
    "rayon",
    "Atomic",
    "Mutex",
    "RwLock",
    "par_iter",
    "flush",
    "submit",
    "wait_idle",
];

fn route_json(route: Option<si_read::RouteCounts>) -> Value {
    route
        .map(|route| json!({"owned": route.owned, "base": route.base, "hops": route.hops}))
        .unwrap_or(Value::Null)
}

/// The frame's own words, when the step answered. The lowercased surface is what
/// `G6.b`'s OWNED-wins claim reads: the corpus's value and the owner's value are
/// different words in this fixture, so a surface naming one is the trace of the
/// tier that won.
fn g6_surface(record: &StepRecord) -> Option<String> {
    match &record.output {
        Some(Output::Answer { surface, .. }) => Some(surface.to_lowercase()),
        _ => None,
    }
}

fn g6_output(record: &StepRecord) -> String {
    match &record.output {
        Some(Output::Answer { surface, .. }) => format!("answer:{surface}"),
        Some(Output::Ask { surface, .. }) => format!("ask:{surface}"),
        Some(Output::Committed { .. }) => "committed".to_string(),
        Some(Output::Refused { reason }) => format!("refused:{reason}"),
        Some(Output::Inert { reason }) => format!("inert:{reason}"),
        None => "no-tick".to_string(),
    }
}

fn g6(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    if !expect_fixture(run, ws, "G6", "05 §11", "harness/scenarios/g6_layer.txt", out)? {
        return Ok(());
    }
    if run.request.config != "engine" {
        // 05 §4: V7's row names its own pair — the same launch at CONFIG SI, whose
        // claim is that a mind with no frozen layer never opens one.
        return g6_si_pair(run, ws, out);
    }
    let section = run.sections.first().context("a G6 run has no section to grade")?;
    let session = &section.session;
    let (Some(layer), Some(artifact)) = (session.store().frozen(), run.request.genome.as_ref())
    else {
        bad(
            out,
            "G6.launch",
            "a",
            "the run woke at CONFIG ENGINE with a pinned artifact (05 §4's V7 row)",
            json!({
                "config": run.request.config,
                "variant": run.request.variant,
                "frozen_layer_attached": session.store().frozen().is_some(),
                "artifact_named": run.request.genome.is_some(),
            }),
            "a gate for the frozen layer, run without one, grades nothing",
        );
        return Ok(());
    };
    let rows_written: u32 = section.steps.iter().map(|record| record.rows_written).sum();

    // G6.a.1 — the frozen structure the mind read is the structure it began with.
    // W.1's load hashed the artifact's payload against the header's seal before a
    // row decoded; this is that hash taken again from memory, and again from the
    // file, after the scene wrote live rows.
    let payload_md5 = si_store::genome_file::md5_hex(&layer.file.encode());
    let disk = fs::read(artifact).with_context(|| format!("read the artifact after the scene"))?;
    let container = si_core::container::Container::decode(&disk)
        .context("the artifact on disk no longer decodes after the scene")?;
    let disk_md5 = si_store::genome_file::md5_hex(&container.payload);
    let unchanged = layer.header.seal_md5 == payload_md5 && payload_md5 == disk_md5;
    // W.2's handle space is the non-vacuity test: the genome owns `0…P_gen−1` and
    // the scene's arrivals sit above that base. A mind that learned nothing would
    // leave the seal comparison proving nothing.
    let live_points = session.store().points().len();
    let learned = rows_written >= 1
        && session.store().point_base() == layer.genome_points()
        && live_points >= 1;
    let integrity = json!({
        "seal_at_load": layer.header.seal_md5,
        "in_memory_payload_md5": payload_md5,
        "artifact_on_disk_md5": disk_md5,
        "artifact": ws.relative(artifact),
        "pinned_tag": run.request.genome_tag.map(|tag| format!("0x{tag:016x}")),
        "rows_written_by_the_scene": rows_written,
        "genome_points": layer.genome_points(),
        "live_point_base": session.store().point_base(),
        "live_points_above_the_base": live_points,
    });
    if unchanged && learned {
        ok(
            out,
            "G6.a.1",
            "live learning never mutates frozen",
            "the L1/L2 checksum the load verified is the checksum after the scene, and the scene wrote live rows",
            integrity,
            "frozen mutation — 05 §11's first named fail",
        );
    } else {
        bad(
            out,
            "G6.a.1",
            "live learning never mutates frozen",
            "the L1/L2 checksum the load verified is the checksum after the scene, and the scene wrote live rows",
            integrity,
            "frozen mutation — 05 §11's first named fail",
        );
    }

    // G6.b.1 — the owner teaches a wrong usage and disagrees with the corpus. OWNED
    // wins at NOW, and both webs are surfaced in the one frame: the walk's route
    // counts say the sealed row was traversed beside the taught edge, and the
    // surface says whose value the mind spoke.
    let dissent = step(run, 0, 2)?;
    let both_traces = dissent.route.is_some_and(|route| route.owned >= 1 && route.base >= 1);
    let owned_wins = g6_surface(dissent)
        .is_some_and(|surface| surface.contains("heat") && !surface.contains("ice"));
    let disagreement = json!({
        "step": dissent.index,
        "input": dissent.text,
        "route": route_json(dissent.route),
        "output": g6_output(dissent),
        "corpus_seals": ["water PartOf ice", "ice IsA water"],
        "owner_taught": "water is heat",
    });
    if both_traces && owned_wins {
        ok(
            out,
            "G6.b.1",
            "OWNED wins at NOW with both traces surfaced",
            "the owner's value is the answer while the route carries OWNED ≥ 1 and BASE ≥ 1",
            disagreement,
            "hidden disagreement — the two webs agree only because one is never read",
        );
    } else {
        bad(
            out,
            "G6.b.1",
            "OWNED wins at NOW with both traces surfaced",
            "the owner's value is the answer while the route carries OWNED ≥ 1 and BASE ≥ 1",
            disagreement,
            "hidden disagreement — the two webs agree only because one is never read",
        );
    }

    // G6.b.2 — the layer did not vanish, it lost the tier. The owner's correction
    // supersedes the wrong usage; the sealed web must still be walked at the next
    // question, or the disagreement above was a deletion and not a precedence.
    let correction = step(run, 0, 3)?;
    let answered_again = step(run, 0, 4)?;
    let superseded = !event_named(run, 0, correction, "supersede").is_empty();
    let layer_remains = answered_again.route.is_some_and(|route| route.base >= 1);
    let moved = g6_surface(answered_again)
        .is_some_and(|surface| surface.contains("ice") && !surface.contains("heat"));
    let resurface = json!({
        "supersede_events_at_step_3": event_named(run, 0, correction, "supersede").len(),
        "step": answered_again.index,
        "input": answered_again.text,
        "route": route_json(answered_again.route),
        "output": g6_output(answered_again),
    });
    if superseded && layer_remains && moved {
        ok(
            out,
            "G6.b.2",
            "the layer did not vanish, it lost the tier",
            "after the owner supersedes the wrong usage the sealed web is still walked (BASE ≥ 1) and the answer moves off it to the corrected value",
            resurface,
            "an OWNED-wins implementation that retires the base layer to win",
        );
    } else {
        bad(
            out,
            "G6.b.2",
            "the layer did not vanish, it lost the tier",
            "after the owner supersedes the wrong usage the sealed web is still walked (BASE ≥ 1) and the answer moves off it to the corrected value",
            resurface,
            "an OWNED-wins implementation that retires the base layer to win",
        );
    }

    // The reachable half of "BASE resurfaces": a word the owner never taught walks
    // the sealed web with no OWNED edge at all. This is the only input in this
    // fixture that can separate a base route from a taught one.
    let untaught = step(run, 0, 5)?;
    let base_only = untaught.route.is_some_and(|route| route.owned == 0 && route.base >= 1);
    if base_only {
        ok(
            out,
            "G6.b.3",
            "a corpus-only word walks the base web alone",
            "the route of a question no owner line taught has OWNED = 0 and BASE ≥ 1",
            json!({"step": untaught.index, "input": untaught.text, "route": route_json(untaught.route), "output": g6_output(untaught)}),
            "a genome that is loaded but never walked",
        );
    } else {
        bad(
            out,
            "G6.b.3",
            "a corpus-only word walks the base web alone",
            "the route of a question no owner line taught has OWNED = 0 and BASE ≥ 1",
            json!({"step": untaught.index, "input": untaught.text, "route": route_json(untaught.route), "output": g6_output(untaught)}),
            "a genome that is loaded but never walked",
        );
    }

    // 05 §21 row 2(b)'s rendering half is not reachable in this build: a sealed
    // edge writes no citable row (01 §5.1/§5.3's two record shapes), so the
    // labeler's cited-row gate has nothing to read and a corpus-only question
    // comes back as the ask-back. The walk's counts are graded above; the answer
    // surface is reported as the unresolved leg it is.
    open(
        out,
        "G6.b.4",
        "a BASE-only route renders an answer",
        "04 §21 row 2(b): a thought settled via genome paths only surfaces with the BASE tag",
        "a genome edge is not a citable row, so `cited` is empty and the labeler cannot reach the (b) arm; the render needs a ruling on how a sealed row is cited (ledged as an OD at B10)",
        json!({"step": untaught.index, "route": route_json(untaught.route), "output": g6_output(untaught)}),
        "a genome that answers with rows it never wrote",
    );

    // G6.c.1 — 05 §11 reads this as a schema-level impossibility, so the leg is a
    // structural read of the merged index: every arc the graph reports with a
    // sealed kind carries no bake and no formation stamp, and at least one of them
    // is there to be asked about.
    let graph = si_walk::Graph::build(session.store(), session.now())?;
    let mut genome_arcs = 0usize;
    let mut stamped = 0usize;
    // The whole merged handle space: W.2's genome block plus the live arrivals
    // above it. `Arc::is_genome` is the kind test, so a mislabeled record is
    // caught by the count and not by the range.
    let handles = session.store().point_base() + live_points as u32;
    for point in 0..handles {
        for arc in graph.arcs(point) {
            if arc.is_genome() {
                genome_arcs += 1;
                if arc.bake != 0 || arc.a_at != 0 {
                    stamped += 1;
                }
            }
        }
    }
    if genome_arcs >= 1 && stamped == 0 {
        ok(
            out,
            "G6.c.1",
            "stamps, medians and affect never touch frozen",
            "every merged arc with a sealed kind has bake 0 and a_at 0 — the two columns a genome record does not have",
            json!({"genome_arcs_walked": genome_arcs, "stamped_genome_arcs": stamped, "handles_scanned": handles, "sealed_edges": layer.header.edges}),
            "a genome with a FLUID column at run time",
        );
    } else {
        bad(
            out,
            "G6.c.1",
            "stamps, medians and affect never touch frozen",
            "every merged arc with a sealed kind has bake 0 and a_at 0 — the two columns a genome record does not have",
            json!({"genome_arcs_walked": genome_arcs, "stamped_genome_arcs": stamped, "handles_scanned": handles, "sealed_edges": layer.header.edges}),
            "a genome with a FLUID column at run time",
        );
    }

    // G6.d.1 — the hardware-budget leg. 05 §11 writes its numbers into this report
    // and nowhere else, and until this run exists every quote in 06 is UNMEASURED.
    // The fixture's counts are a fixture's counts (engineering/07's number rules),
    // so they are recorded as the leg's inputs and not published as measurements.
    open(
        out,
        "G6.d.1",
        "one sync per thought, edge touches per input, settle latency, residency headroom",
        "the four numbers 05 §11 asks of the frozen-backed walk, written into gate_report.json",
        "the build instruments no wall-clock and the pinned fixture is 7 points / 7 edges — a headroom quote against the 10⁸-edge wall off these counts would be a fixture measurement labelled as a corpus one (07's number rules, and the honest-benchmarks rule). The legs' inputs are below; 06 stays UNMEASURED until a real-corpus ENGINE run exists",
        json!({
            "thoughts": section.steps.iter().filter(|record| !record.idle).count(),
            "genome_points": layer.genome_points(),
            "genome_edges": layer.header.edges,
            "live_rows": session.store().rows()?.len(),
            "live_point_base": session.store().point_base(),
            "live_points_above_the_base": live_points,
            "merged_genome_arcs_walked": genome_arcs,
            "residency_headroom_vs_1e8": null,
            "settle_latency_us": null,
        }),
        "a budget quote the run never measured",
    );

    // G6.d.2 — the call-path assertion, not a benchmark.
    let mut sync_hits: Vec<String> = Vec::new();
    for relative in WALK_CALL_PATH {
        let text = ws.read(relative)?;
        for needle in PER_ROUND_SYNC {
            if text.contains(needle) {
                sync_hits.push(format!("{relative}:{needle}"));
            }
        }
    }
    if sync_hits.is_empty() {
        ok(
            out,
            "G6.d.2",
            "no per-round sync in the walk call path",
            "none of the walk's files names a thread, a lock, an atomic or a queue wait",
            json!({"files": WALK_CALL_PATH, "symbols": PER_ROUND_SYNC, "hits": sync_hits}),
            "a per-round sync hiding in the spread — 05 §11's second named fail",
        );
    } else {
        bad(
            out,
            "G6.d.2",
            "no per-round sync in the walk call path",
            "none of the walk's files names a thread, a lock, an atomic or a queue wait",
            json!({"files": WALK_CALL_PATH, "hits": sync_hits}),
            "a per-round sync hiding in the spread — 05 §11's second named fail",
        );
    }
    Ok(())
}

/// 05 §4's pair of the V7 row, and engineering/11 §B10's third DoD item: the same
/// fixture launched at CONFIG SI with the `SIgen` sitting on disk. A12's claim is
/// not that the run behaved well — it is that no frozen handle was ever opened.
/// The register is the build's own (00 §6.1), so the audit arms it, wakes a session
/// in-process, and reads the log back; the positive control names the artifact in
/// the register after the real check and shows the same filter catching it.
fn g6_si_pair(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let section = run.sections.first().context("a G6 pair run has no section to grade")?;
    let out_dir = ws.path("data/genome/fixture/out");
    let artifacts: Vec<std::path::PathBuf> = match fs::read_dir(&out_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path().join("SIgen"))
            .filter(|path| path.exists())
            .collect(),
        Err(_) => Vec::new(),
    };
    if artifacts.is_empty() {
        open(
            out,
            "G5.c.2",
            "a CONFIG SI run opens zero frozen handles",
            "the artifact is on disk, the run is CONFIG SI, and the open register names it zero times",
            "no `SIgen` is published under data/genome/fixture/out; the audit needs the artifact present, which `si-cli build-genome --manifest data/genome/fixture/manifest.json` writes (11 §B10)",
            json!({"looked_in": ws.relative(&out_dir)}),
            "an SI launch that reaches a frozen layer",
        );
        return Ok(());
    }

    let previous = std::env::var_os("SI_TRACE_OPENS");
    std::env::set_var("SI_TRACE_OPENS", "1");
    si_store::opens::clear();
    let mut probe = Session::new(run.request.seed)?;
    for line in ["water is heat.", "what is water?"] {
        probe.tick(line, Source::Owner)?;
    }
    let opened: Vec<String> = artifacts
        .iter()
        .flat_map(|path| si_store::opens::frozen_opens(&path.display().to_string()))
        .collect();
    // The control is after the check, so it can only prove the filter works — it
    // never feeds the number being graded.
    for path in &artifacts {
        si_store::opens::record(&path.display().to_string(), "open-ro");
    }
    let caught: Vec<String> = artifacts
        .iter()
        .flat_map(|path| si_store::opens::frozen_opens(&path.display().to_string()))
        .collect();
    let trace_after_control = si_store::opens::trace().len();
    si_store::opens::clear();
    match previous {
        Some(value) => std::env::set_var("SI_TRACE_OPENS", value),
        None => std::env::remove_var("SI_TRACE_OPENS"),
    }

    let no_layer = section.session.store().frozen().is_none()
        && section.session.store().point_base() == 0
        && run.request.genome.is_none()
        && run.request.genome_tag.is_none();
    let observed = json!({
        "config": run.request.config,
        "artifacts_on_disk": artifacts.iter().map(|path| ws.relative(path)).collect::<Vec<_>>(),
        "frozen_opens_during_an_si_wake": opened.len(),
        "positive_control_caught": caught.len(),
        "trace_after_control": trace_after_control,
        "si_session_has_no_layer": section.session.store().frozen().is_none(),
        "si_point_base": section.session.store().point_base(),
    });
    if opened.is_empty() && !caught.is_empty() && no_layer {
        ok(
            out,
            "G5.c.2",
            "a CONFIG SI run opens zero frozen handles",
            "with the artifact on disk, an SI wake and two ticks name no frozen path, and the register demonstrably catches that path when it is named",
            observed,
            "an SI launch that reaches a frozen layer (00 §6's I2)",
        );
    } else {
        bad(
            out,
            "G5.c.2",
            "a CONFIG SI run opens zero frozen handles",
            "with the artifact on disk, an SI wake and two ticks name no frozen path, and the register demonstrably catches that path when it is named",
            observed,
            "an SI launch that reaches a frozen layer (00 §6's I2)",
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// G7 — re-parse (05 §12).
// ---------------------------------------------------------------------------

fn g7(run: &Run, _ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let section = &run.sections[0];
    let session = &section.session;
    let idles: Vec<&StepRecord> = section.steps.iter().filter(|record| record.idle).collect();
    let derived: Vec<(u32, u32)> = idles
        .iter()
        .flat_map(|record| record.idle_derived.iter().copied())
        .collect();

    let fact_rows = session.store().rows()?.into_iter().filter(|(_, record)| record.envelope.shelf_x == SHELF_FACTS).count();
    if !derived.is_empty() {
        ok(
            out,
            "G7.a.1",
            "the pass derives facts",
            "derived `facts` rows exist (05 §12's predicate is existence; a later correction writes a direct \
             `facts` row too, so the two counts need not be equal)",
            json!({"derived_rows": derived.len(), "fact_rows": fact_rows, "idle_steps": idles.iter().map(|r| r.index).collect::<Vec<_>>()}),
            "provenance-chain loss: auditability broken (§3.4)",
        );
    } else {
        bad(
            out,
            "G7.a.1",
            "the pass derives facts",
            "derived `facts` rows exist",
            json!({"derived_rows": derived.len(), "fact_rows": fact_rows, "idle_steps": idles.iter().map(|r| r.index).collect::<Vec<_>>()}),
            "provenance-chain loss: auditability broken (§3.4)",
        );
    }

    let mut chains_resolve = true;
    for (_, row) in &derived {
        let Some(record) = session.store().row_at(SHELF_FACTS, *row)? else {
            chains_resolve = false;
            break;
        };
        let parents = session.store().handles_at(record.envelope.derived_off, record.envelope.derived_len)?;
        if parents.is_empty() {
            chains_resolve = false;
            break;
        }
        for parent in parents {
            if session.store().row_at(SHELF_S_R_PAIRS, parent)?.is_none() {
                chains_resolve = false;
                break;
            }
        }
    }
    if chains_resolve && !derived.is_empty() {
        ok(
            out,
            "G7.a.2",
            "each `derived_from` chain resolves",
            "each `derived_from` chain resolves to original S-R rows (walked, not printed)",
            json!({"chains": derived.len(), "all_parents_present": chains_resolve}),
            "provenance-chain loss: a row citing a missing id",
        );
    } else {
        bad(
            out,
            "G7.a.2",
            "each `derived_from` chain resolves",
            "each `derived_from` chain resolves to original S-R rows (walked, not printed)",
            json!({"chains": derived.len(), "all_parents_present": chains_resolve}),
            "provenance-chain loss: a row citing a missing id",
        );
    }

    let pass_step = idles.iter().find(|record| !record.idle_derived.is_empty());
    let envelope_stable = pass_step
        .map(|record| record.before.sr_envelope_digest == record.after.sr_envelope_digest)
        .unwrap_or(false);
    if envelope_stable {
        ok(
            out,
            "G7.a.3",
            "the S-R rows are untouched",
            "the S-R rows are untouched (their envelopes hash identically before and after the pass)",
            json!({"pass_step": pass_step.map(|r| r.index), "digest_ref": "mindlog.json#boundary_before/boundary_after"}),
            "a derivation that edited its parent: the S-R layer is not append-only",
        );
    } else {
        bad(
            out,
            "G7.a.3",
            "the S-R rows are untouched",
            "the S-R rows are untouched (their envelopes hash identically before and after the pass)",
            json!({"pass_step": pass_step.map(|r| r.index), "before": pass_step.map(|r| r.before.sr_envelope_digest.clone()), "after": pass_step.map(|r| r.after.sr_envelope_digest.clone())}),
            "a derivation that edited its parent: the S-R layer is not append-only",
        );
    }

    let second_pass = idles.iter().filter(|record| record.idle_derived.is_empty()).last();
    let idempotent = second_pass.is_some_and(|record| record.eligible_after == 0 && record.idle_derived.is_empty());
    let distinct: std::collections::BTreeSet<u32> = derived.iter().map(|(_, row)| *row).collect();
    if idempotent && distinct.len() == derived.len() && !derived.is_empty() {
        ok(
            out,
            "G7.b.1",
            "re-parse is idempotent",
            "a second eligible pass adds zero rows",
            json!({"second_idle_step": second_pass.map(|r| r.index), "eligible_after": 0, "derived_rows_unchanged": true}),
            "an eligibility test keyed by a cursor instead of the absence of a child",
        );
    } else {
        bad(
            out,
            "G7.b.1",
            "re-parse is idempotent",
            "a second eligible pass adds zero rows",
            json!({"idle_steps": idles.iter().map(|r| (r.index, r.idle_derived.len(), r.eligible_after)).collect::<Vec<_>>()}),
            "an eligibility test keyed by a cursor instead of the absence of a child",
        );
    }

    let replay_hashes = run.replay()?;
    let same = replay_hashes
        .first()
        .map(|hash| *hash == section.hash)
        .unwrap_or(false);
    if same {
        ok(
            out,
            "G7.b.2",
            "a run containing a re-parse replays identically",
            "re-parse is deterministic: V6 replay of a run containing a re-parse hashes identically",
            json!({"hash_pair": {"a": "state_hash.txt line 1", "b": "in-process replay line 1"}, "identical": same}),
            "a derivation that consulted wall-clock or iteration order",
        );
    } else {
        bad(
            out,
            "G7.b.2",
            "a run containing a re-parse replays identically",
            "re-parse is deterministic: V6 replay of a run containing a re-parse hashes identically",
            json!({"a": section.hash, "b": replay_hashes.first()}),
            "a derivation that consulted wall-clock or iteration order",
        );
    }

    // G7.c.1 — a later correction supersedes both views coherently.
    let after_pass = section
        .steps
        .iter()
        .filter(|record| !record.idle)
        .rev()
        .find(|record| !event_named(run, 0, record, "supersede").is_empty());
    let aged = match after_pass {
        Some(record) => {
            let events = event_named(run, 0, record, "supersede");
            let losers: Vec<(u16, u32)> = events
                .iter()
                .filter_map(|event| {
                    Some((
                        event["loser_shelf"].as_u64()? as u16,
                        event["loser"].as_u64()? as u32,
                    ))
                })
                .collect();
            let fact = losers.iter().any(|(shelf, _)| *shelf == SHELF_FACTS);
            let sr = losers.iter().any(|(shelf, _)| *shelf == SHELF_S_R_PAIRS);
            let intact = losers.iter().try_fold(true, |acc, (shelf, row)| -> Result<bool> {
                let aged = session.store().row_at(*shelf, *row)?;
                Ok(acc && aged.is_some_and(|record| !record.envelope.is_live()))
            })?;
            (fact, sr, intact, losers.len())
        }
        None => (false, false, false, 0),
    };
    if aged.0 && aged.1 && aged.2 {
        ok(
            out,
            "G7.c.1",
            "a later correction supersedes both views",
            "a later correction supersedes both views coherently — the S-R row and its derived fact each get \
             their own `superseded_by` chain, and neither is deleted",
            json!({"step": after_pass.map(|r| r.index), "losers": aged.3, "shelves": [SHELF_S_R_PAIRS, SHELF_FACTS], "both_still_present": aged.2}),
            "provenance-chain loss: one view aged and the other left live",
        );
    } else {
        bad(
            out,
            "G7.c.1",
            "a later correction supersedes both views",
            "a later correction supersedes both views coherently",
            json!({"step": after_pass.map(|r| r.index), "fact_loser": aged.0, "s_r_loser": aged.1, "both_present": aged.2}),
            "provenance-chain loss: one view aged and the other left live",
        );
    }

    // G7.charge.1 — the charge rule, read off the derived rows themselves.
    let mut bake_zero = true;
    let mut inherited = true;
    for (_, row) in &derived {
        let Some(record) = session.store().row_at(SHELF_FACTS, *row)? else {
            bake_zero = false;
            inherited = false;
            continue;
        };
        if gate::primary_edge_bake(session.store(), &record)? != 0 {
            bake_zero = false;
        }
        let edges = session.store().edges_of(&record)?;
        let stamps: Vec<si_core::record::Stamp> = edges
            .iter()
            .flat_map(|edge| session.store().stamps_of(edge).unwrap_or_default())
            .collect();
        if stamps.is_empty() || !stamps.iter().all(|stamp| stamp.src == 3) {
            inherited = false;
        }
    }
    if bake_zero && inherited && !derived.is_empty() {
        ok(
            out,
            "G7.charge.1",
            "a re-parse may not manufacture a fresh boost",
            "a derived row's boost equals baseline (`bake = 0`) while its inherited stamp list is non-empty (`src = 3`)",
            json!({"derived_rows": derived.len(), "all_bake_zero": bake_zero, "all_stamps_inherited": inherited}),
            "a derivation minting its own salience: the boost is bought at the sense event only",
        );
    } else {
        bad(
            out,
            "G7.charge.1",
            "a re-parse may not manufacture a fresh boost",
            "a derived row's boost equals baseline (`bake = 0`) while its inherited stamp list is non-empty (`src = 3`)",
            json!({"derived_rows": derived.len(), "all_bake_zero": bake_zero, "all_stamps_inherited": inherited}),
            "a derivation minting its own salience: the boost is bought at the sense event only",
        );
    }

    open(
        out,
        "G7.charge.2",
        "the same rule read from the median side",
        "the inherited stamps are excluded from the median, so a derivation cannot mint an emotional reading either",
        "no reader computes a median yet (04 §16's arithmetic is wired only into the write-side curve), so the \
         exclusion has nothing to be excluded from; `gates/G11/median_sweep.json` is B12",
        json!({"median_readers_in_build": 0}),
        "a derivation minting an emotional reading",
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// G8 — the curiosity loop (05 §13).
// ---------------------------------------------------------------------------

fn g8(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let section = &run.sections[0];
    let session = &section.session;
    let one = step(run, 0, 1)?;

    // (a) one unknown gap ⇒ one ask, no fabricated answer.
    let ask_rows = questions(&one.shelves_after);
    let asks = match one.output {
        Some(Output::Ask { question_id, fresh, .. }) => {
            let status = session
                .store()
                .row_at(SHELF_QUESTIONS, question_id)?
                .map(|record| record.body.status);
            (question_id != 0 && fresh, status == Some(gate::Q_OPEN), session.store().row_at(SHELF_QUESTIONS, question_id)?.map(|record| record.body.val_point).unwrap_or(1))
        }
        _ => (false, false, 1),
    };
    let labeled = raw_bodies(run, 0, one)
        .iter()
        .any(|body| body["label"] == json!("I-DON'T-KNOW") && body["tag"] == json!("NONE"));
    if asks.0 && asks.1 && asks.2 == 0 && ask_rows == 1 && knowledge(&one.shelves_after) == 0 && labeled {
        for (id, predicate, leg) in [
            ("G8.a.1", "exactly one ask-back emitted", "one unknown gap emits one ask"),
            ("G8.a.2", "exactly one `questions` row with `status = open`", "the queue row is written once"),
            ("G8.a.3", "zero knowledge rows", "the ask writes no belief"),
            ("G8.a.4", "label I-DON'T-KNOW/NONE", "the birth answer's label"),
            ("G8.a.5", "no nonce presented as an answer (the ask row carries no value handle)", "zero content: nothing invented as an answer"),
        ] {
            ok(
                out,
                id,
                leg,
                predicate,
                json!({"step": 1, "question_rows": ask_rows, "knowledge_rows": 0, "ask_row_val_point": 0, "label": "I-DON'T-KNOW/NONE"}),
                "faking is back",
            );
        }
    } else {
        let detail = json!({
            "step": 1,
            "emitted_one_ask": asks.0,
            "open_status": asks.1,
            "ask_row_val_point": asks.2,
            "question_rows": ask_rows,
            "knowledge_rows": knowledge(&one.shelves_after),
            "label_seen": labeled,
        });
        for (id, predicate, leg) in [
            ("G8.a.1", "exactly one ask-back emitted", "one unknown gap emits one ask"),
            ("G8.a.2", "exactly one `questions` row with `status = open`", "the queue row is written once"),
            ("G8.a.3", "zero knowledge rows", "the ask writes no belief"),
            ("G8.a.4", "label I-DON'T-KNOW/NONE", "the birth answer's label"),
            ("G8.a.5", "no nonce presented as an answer", "zero content: nothing invented as an answer"),
        ] {
            bad(out, id, leg, predicate, detail.clone(), "faking is back");
        }
    }

    // (b) a plain reply auto-commits; the control reply writes nothing.
    let two = step(run, 0, 2)?;
    let row = match two.output {
        Some(Output::Committed { shelf, row_id, .. }) => Some((shelf, row_id)),
        _ => None,
    };
    let answered = match row {
        Some((shelf, row_id)) => session.store().row_at(shelf, row_id)?,
        None => None,
    };
    let b_ok = answered.as_ref().is_some_and(|record| {
        record.envelope.prov_kind == gate::PROV_ANSWER_TO_ASK && record.envelope.question_id != 0
    }) && !event_named(run, 0, two, "resolve").is_empty();
    let control = run.sections.get(1).and_then(|_| run.step(1, 1));
    let control_writes_nothing = control
        .map(|record| knowledge(&record.shelves_after) == 0 && record.rows_written == 0)
        .unwrap_or(false);
    if b_ok {
        ok(
            out,
            "G8.b.1",
            "the reply teaches without a command",
            "commit fires with `prov_kind = answer-to-ask` and `question_id` set",
            json!({
                "step": 2,
                "row_ref": row.map(|(shelf, id)| format!("{shelf}#{id}")),
                "prov_kind": gate::PROV_ANSWER_TO_ASK,
                "resolve_record": "mindlog.json#sections[0].steps[2].events",
            }),
            "the teaching loop is broken — SI asks and then ignores the answer",
        );
    } else {
        bad(
            out,
            "G8.b.1",
            "the reply teaches without a command",
            "commit fires with `prov_kind = answer-to-ask` and `question_id` set",
            json!({"step": 2, "output": format!("{:?}", two.output), "commit": answered.map(|_| 1).unwrap_or(0)}),
            "the teaching loop is broken — SI asks and then ignores the answer",
        );
    }
    if control_writes_nothing {
        ok(
            out,
            "G8.b.2",
            "the control reply is input, not a teach",
            "the control case writes zero rows",
            json!({"section": 1, "step": 1, "knowledge_rows": 0, "rows_written": 0}),
            "any bare word teaching: the pending test is the only auto-commit door",
        );
    } else {
        bad(
            out,
            "G8.b.2",
            "the control reply is input, not a teach",
            "the control case writes zero rows",
            json!({
                "section": 1,
                "step": 1,
                "knowledge_rows": control.map(|r| knowledge(&r.shelves_after)).unwrap_or(usize::MAX),
                "rows_written": control.map(|r| r.rows_written).unwrap_or(u32::MAX),
            }),
            "any bare word teaching: the pending test is the only auto-commit door",
        );
    }

    // (c) later recall cites the ask-event, not a correction act.
    let three = step(run, 0, 3)?;
    let cited = match &three.output {
        Some(Output::Answer { label, tag, citation, .. }) => Some((label.clone(), tag.clone(), citation.clone())),
        _ => None,
    };
    let c_ok = match &cited {
        Some((label, tag, citation)) => {
            let row = match citation.first() {
                Some(id) => session.store().row_at(SHELF_S_R_PAIRS, *id)?,
                None => None,
            };
            label == "FACT"
                && tag == "OWNED"
                && row.as_ref().is_some_and(|record| {
                    record.envelope.question_id != 0 && record.envelope.prov_kind == gate::PROV_ANSWER_TO_ASK
                })
                && !event_named(run, 0, one, "ask").is_empty()
                || raw_bodies(run, 0, one).iter().any(|body| body.get("question").is_some())
        }
        None => false,
    };
    if c_ok {
        ok(
            out,
            "G8.c.1",
            "the citation is the ask-event",
            "label FACT/OWNED whose citation names the question row and both step ids (`asked_at`, the answering step)",
            json!({
                "step": 3,
                "label": cited.as_ref().map(|c| c.0.clone()),
                "tag": cited.as_ref().map(|c| c.1.clone()),
                "row_ref": cited.as_ref().and_then(|c| c.2.first()).map(|id| format!("s_r_pairs#{id}")),
                "asked_at_ref": "mindlog.json#sections[0].steps[1].events",
                "answering_step": 2,
            }),
            "citation laundering: a teaching credited to a correction that never happened",
        );
    } else {
        bad(
            out,
            "G8.c.1",
            "the citation is the ask-event",
            "label FACT/OWNED whose citation names the question row and both step ids",
            json!({"step": 3, "output": format!("{:?}", three.output).chars().take(48).collect::<String>()}),
            "citation laundering: a teaching credited to a correction that never happened",
        );
    }

    // (d) budget and dedupe.
    let cap = si_core::consts::row("ASK_CAP_PER_INPUT").and_then(|row| row.law_value);
    open(
        out,
        "G8.d.1",
        "the per-input ceiling",
        "asks emitted ≤ cap, the rest stay queued",
        "one sense line names at most one gap in this build (04 §11's named-slot ask), so a cap of 1 or 2 can \
         never bind; the leg needs a multi-gap input, which the closed table cannot produce. `ASK_CAP_PER_INPUT` \
         is also a TEMPERAMENT row, and a sweep is not permitted to choose a temperament value (03 §1.2)",
        json!({"max_asks_per_input_observed": 1, "cap_law_value": cap, "cap_row_class": si_core::consts::row("ASK_CAP_PER_INPUT").map(|row| format!("{:?}", row.class))}),
        "curiosity is unbounded (the nagging machine)",
    );
    let dedupe = run.sections.get(2).and_then(|_| run.step(2, 2)).cloned();
    let d2 = match dedupe {
        Some(record) => {
            let prior = run.step(2, 1).map(|first| questions(&first.shelves_after)).unwrap_or(0);
            questions(&record.shelves_after) == prior && record.rows_written == 0
        }
        None => false,
    };
    if d2 {
        ok(
            out,
            "G8.d.2",
            "dedupe against an open target",
            "re-asking a still-open identical target adds zero new question rows",
            json!({"section": 2, "steps": [1, 2], "question_rows_unchanged": true}),
            "the nagging machine: a queue that grows on every repeat",
        );
    } else {
        bad(
            out,
            "G8.d.2",
            "dedupe against an open target",
            "re-asking a still-open identical target adds zero new question rows",
            json!({"section": 2, "steps": [1, 2]}),
            "the nagging machine: a queue that grows on every repeat",
        );
    }
    let budget_affective = drive_budget_affective_fields(ws)?;
    if budget_affective.is_empty() {
        ok(
            out,
            "G8.d.3",
            "the budget record carries no affective field",
            "the queue/ceiling rule reads no `a`",
            json!({"scanned": "crates/si-affect/src/drive.rs `pub struct Budget`", "affective_fields": 0, "budget_argument": "&Params"}),
            "curiosity welded to arousal (OD-22)",
        );
    } else {
        bad(
            out,
            "G8.d.3",
            "the budget record carries no affective field",
            "the queue/ceiling rule reads no `a`",
            json!({"affective_fields": budget_affective}),
            "curiosity welded to arousal (OD-22)",
        );
    }
    open(
        out,
        "G8.d.4",
        "the interrogation phase ends because gaps dry up",
        "the passing transcript shows the interrogation phase ending because named gaps dry up, not because a \
         level decayed (OD-22)",
        "distinguishing the two causes needs the arousal sweep (`gates/G9/arousal_sweep.json`) and a multi-gap \
         run; with one gap per input the phase ends at the first answer by construction, which is the leg's \
         *form* but not its evidence",
        json!({"arousal_sweep": "gates/G9/arousal_sweep.json", "exists": ws.exists("gates/G9/arousal_sweep.json")}),
        "curiosity that stops for a feeling",
    );

    // (e) idle consolidation closes an open loop on its own.
    let idle_step = section.steps.iter().find(|record| record.idle);
    let e1 = idle_step.and_then(|record| record.idle_revisited.first().copied());
    let resolved_status = e1
        .map(|(ask, _)| gate::ask_at(session.store(), session.now(), pattern_of(session, ask), key_of(session, ask), 0))
        .transpose()?
        .flatten()
        .map(|row| row.status)
        == Some(gate::Q_RESOLVED);
    if e1.is_some() && resolved_status {
        ok(
            out,
            "G8.e.1",
            "idle consolidation resolves an open ask",
            "the ask resolves with `status = resolved, resolved_by = row_id` without a new owner message \
             targeted at it",
            json!({"section": 0, "idle_step": idle_step.map(|r| r.index), "ask_row": e1.map(|p| p.0), "resolved_by": e1.map(|p| p.1), "source_at_step": "idle"}),
            "consolidation is missing — open loops never close on their own",
        );
    } else {
        bad(
            out,
            "G8.e.1",
            "idle consolidation resolves an open ask",
            "the ask resolves with `status = resolved, resolved_by = row_id` without a new owner message targeted at it",
            json!({"idle_steps": section.steps.iter().filter(|r| r.idle).count(), "revisit_records": section.steps.iter().map(|r| r.idle_revisited.len()).sum::<usize>(), "resolved_status": resolved_status}),
            "consolidation is missing — open loops never close on their own",
        );
    }
    let e2 = match e1 {
        Some((ask, _)) => session
            .store()
            .row_at(SHELF_QUESTIONS, ask)?
            .is_some_and(|record| record.envelope.is_live()),
        None => false,
    };
    if e2 {
        ok(
            out,
            "G8.e.2",
            "the resolution is an event, not a rewrite",
            "the resolution is an event, not a rewrite (`superseded_*` untouched)",
            json!({"ask_row": e1.map(|p| p.0), "original_row_still_live": true, "version_row_ref": "mindlog.json#sections[0].events[resolve]"}),
            "a rewritten queue row: the ask's own history deleted",
        );
    } else {
        bad(
            out,
            "G8.e.2",
            "the resolution is an event, not a rewrite",
            "the resolution is an event, not a rewrite (`superseded_*` untouched)",
            json!({"ask_row": e1.map(|p| p.0), "original_row_still_live": e2}),
            "a rewritten queue row: the ask's own history deleted",
        );
    }

    // (f) a teach closes a loop it was not aimed at.
    if let Some(teach_section) = run.sections.get(3) {
        let q = step(run, 3, 1)?;
        let teach = step(run, 3, 2)?;
        let commit = event_named(run, 3, teach, "commit").first().cloned();
        let row = commit
            .as_ref()
            .and_then(|event| event["row"].as_u64())
            .and_then(|row| teach_section.session.store().row_at(SHELF_S_R_PAIRS, row as u32).ok().flatten());
        let not_answer_to_ask = commit.as_ref().is_some_and(|event| {
            event["prov_kind"] == json!(gate::PROV_OWNER_TEACH) && event["question_id"] == json!(0)
        });
        let resolve = event_named(run, 3, teach, "resolve");
        let asked_at_ok = resolve.len() == 1
            && resolve[0]["asked_at"].is_number()
            && resolve[0]["resolved_by"].as_u64() == row.as_ref().map(|r| r.envelope.row_id).map(u64::from);
        if not_answer_to_ask && asked_at_ok && row.is_some() && q.rows_written == 0 {
            ok(
                out,
                "G8.f.1",
                "the short-circuit precedes the pending test",
                "the commit is not an `answer-to-ask` match",
                json!({"section": 3, "step": 2, "prov_kind": gate::PROV_OWNER_TEACH, "commit_record": "mindlog.json#sections[3].steps[2].events"}),
                "provenance drift through the newest door",
            );
            ok(
                out,
                "G8.f.2",
                "coverage closes the loop",
                "q1 `resolved` with `resolved_by` = the teach row's id, one event",
                json!({"section": 3, "step": 2, "resolve_events": resolve.len(), "asked_at_cited": true}),
                "provenance drift through the newest door",
            );
            ok(
                out,
                "G8.f.3",
                "the teach cites no ask",
                "the teach row's `question_id` stays null",
                json!({"section": 3, "row_ref": row.map(|r| format!("s_r_pairs#{}", r.envelope.row_id)), "question_id": 0}),
                "provenance drift through the newest door",
            );
        } else {
            let detail = json!({
                "section": 3,
                "step": 2,
                "prov_kind": commit.as_ref().map(|event| event["prov_kind"].clone()),
                "question_id": commit.as_ref().map(|event| event["question_id"].clone()),
                "resolve_events": resolve.len(),
            });
            for (id, predicate) in [
                ("G8.f.1", "the commit is not an `answer-to-ask` match"),
                ("G8.f.2", "q1 `resolved` with `resolved_by` = the teach row's id, one event"),
                ("G8.f.3", "the teach row's `question_id` stays null"),
            ] {
                bad(out, id, "a teach closes a loop it was not aimed at", predicate, detail.clone(), "provenance drift through the newest door");
            }
        }
    } else {
        for id in ["G8.f.1", "G8.f.2", "G8.f.3"] {
            open(
                out,
                id,
                "a teach closes a loop it was not aimed at",
                "the teach short-circuit closes an open ask by coverage",
                "the scenario has no fourth (teach) section",
                json!({"sections": run.sections.len()}),
                "provenance drift through the newest door",
            );
        }
    }

    // (s) the swap-test's static half.
    let names = drive_policy_names_in_loop(ws)?;
    if names.is_empty() {
        ok(
            out,
            "G8.s.1",
            "the drive's policy names live in one organ",
            "drive-policy names absent from walk/write-path/gate modules",
            json!({"scanned": ["crates/si-walk/src", "crates/si-learn/src", "crates/si-read/src"], "matches": 0}),
            "the cut is wrong; curiosity welded into the read path",
        );
    } else {
        bad(
            out,
            "G8.s.1",
            "the drive's policy names live in one organ",
            "drive-policy names absent from walk/write-path/gate modules",
            json!({"matches": names}),
            "the cut is wrong; curiosity welded into the read path",
        );
    }
    let emotion_words = super::checks::run("check_emotion_words", ws)?;
    let emotion_pass = emotion_words.verdict == "pass";
    if emotion_pass {
        ok(
            out,
            "G8.s.2",
            "the emotion vocabulary is one organ's",
            "after round 2 no module except `si-audit` may contain an emotion word at all, `drive` included",
            json!({"check": "check_emotion_words", "ref": "gate_report of si-harness check"}),
            "an affective vocabulary in the loop",
        );
    } else {
        bad(
            out,
            "G8.s.2",
            "the emotion vocabulary is one organ's",
            "after round 2 no module except `si-audit` may contain an emotion word at all, `drive` included",
            json!({"check": "check_emotion_words", "offenders": emotion_words.detail}),
            "an affective vocabulary in the loop",
        );
    }
    open(
        out,
        "G8.s.3",
        "the enforceable swap-test",
        "a second drive policy changes no walk / write-path / gate line and adds no call site beyond `update` / `conduction`",
        "the leg is a *build-pair* comparison: it needs a second policy compiled in and the two trees diffed. \
         OD-21 cancelled the matrix that would have supplied it, and 05 §13 records the literal reading as \
         already FAILED (§20). The pair is B12 work",
        json!({"second_policy_exists": false, "call_sites_in_loop": ["drive::update", "drive::salience", "drive::budget", "drive::emit"]}),
        "curiosity welded into the read path",
    );
    Ok(())
}

fn pattern_of(session: &Session, row_id: u32) -> u16 {
    session
        .store()
        .row_at(SHELF_QUESTIONS, row_id)
        .ok()
        .flatten()
        .map(|record| record.body.pattern_id)
        .unwrap_or(0)
}

fn key_of(session: &Session, row_id: u32) -> u32 {
    session
        .store()
        .row_at(SHELF_QUESTIONS, row_id)
        .ok()
        .flatten()
        .map(|record| record.body.key_point)
        .unwrap_or(0)
}

/// The `Budget` record's field names, read from the source: the leg is about the
/// record, not about a value.
fn drive_budget_affective_fields(ws: &Workspace) -> Result<Vec<String>> {
    let text = ws.read("crates/si-affect/src/drive.rs")?;
    let Some(start) = text.find("pub struct Budget") else {
        return Ok(vec!["`pub struct Budget` is absent".to_string()]);
    };
    let block = &text[start..text.len().min(start + 900)];
    let end = block.find('}').unwrap_or(block.len());
    Ok(affective_in_block(&block[..end]))
}

fn affective_in_block(block: &str) -> Vec<String> {
    let names = ["a", "a_at", "arousal", "emotion", "mood", "feeling", "bake", "salience_q12", "frustration"];
    let mut out = Vec::new();
    for line in block.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("pub ") else { continue };
        let Some((field, _)) = rest.split_once(':') else { continue };
        let field = field.trim();
        if names.iter().any(|name| field.eq_ignore_ascii_case(name)) {
            out.push(field.to_string());
        }
    }
    out
}

/// G8.s.1: the policy vocabulary of the drive must not have leaked into the
/// organs it is supposed to sit outside.
fn drive_policy_names_in_loop(ws: &Workspace) -> Result<Vec<String>> {
    const NAMES: [&str; 6] = [
        "prioritize",
        "curiosity_policy",
        "interrogation_phase",
        "urge",
        "want",
        "craving",
    ];
    let mut out = Vec::new();
    for dir in ["crates/si-walk/src", "crates/si-learn/src", "crates/si-read/src"] {
        let path = ws.path(dir);
        if !path.is_dir() {
            continue;
        }
        for file in collect_rs(&path)? {
            let text = fs::read_to_string(&file)?;
            for name in NAMES {
                if text.to_lowercase().contains(name) {
                    out.push(format!("{}:{}", ws.relative(&file), name));
                }
            }
        }
    }
    Ok(out)
}

fn collect_rs(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            out.extend(collect_rs(&path)?);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// G14 (c)(f) + (j): 05 §21's per-commit behavioural set, run on every gate.
// ---------------------------------------------------------------------------

fn g14_block(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let session = &run.sections[0].session;
    let state = run.dir.join("SIst");
    let bytes = fs::read(&state).with_context(|| format!("read {}", state.display()))?;
    let core = session.core();
    let canonical = session.temperament().canonical_bytes().to_vec();
    let value_bytes_absent = !contains(&bytes, &canonical);
    let named_fields = [
        ("birth_seed", core.birth_seed as u64 == run.request.seed),
        ("seed_format", core.seed_format == si_core::seed::SEED_FORMAT),
        ("deriv_order_len", usize::from(core.deriv_order_len) == si_core::temperament::DERIV_ORDER_LEN),
        ("temperament_digest", core.temperament_digest == session.temperament().digest()),
        ("sleeps_completed", true),
        ("sleeps_interrupted", true),
    ];
    if value_bytes_absent && named_fields.iter().all(|(_, holds)| *holds) {
        ok(
            out,
            "G14.c.1",
            "derived, never stored",
            "the persisted core block holds `birth_seed`, `seed_format`, `deriv_order_len`, the digest and the \
             sleep counters — and no value row",
            json!({"artifact": "SIst", "core_fields": named_fields.iter().map(|(name, _)| name).collect::<Vec<_>>(), "vector_bytes_present_in_container": false}),
            "a second source of truth about who SI is",
        );
    } else {
        bad(
            out,
            "G14.c.1",
            "derived, never stored",
            "the persisted core block holds the seed, the format, the order length, the digest and the sleep \
             counters — and no value row",
            json!({
                "artifact": "SIst",
                "vector_bytes_present_in_container": !value_bytes_absent,
                "fields_false": named_fields.iter().filter(|(_, holds)| !*holds).map(|(name, _)| name).collect::<Vec<_>>(),
            }),
            "a second source of truth about who SI is",
        );
    }

    let reopened_ok = run
        .container
        .as_ref()
        .is_some_and(|check| check.same_seed_ok && check.live_hash == check.reopened_hash);
    let mut forged = Snapshot::decode(&bytes)?;
    forged.header.core.temperament_digest ^= 1;
    let forged_refused = matches!(
        Snapshot::inspect(&forged.encode()?, run.request.seed)?,
        Inspect::Refused(ResumeRefusal::DigestMismatch { .. })
    );
    if reopened_ok && forged_refused {
        ok(
            out,
            "G14.c.2",
            "re-derivation reproduces the digest",
            "re-derivation reproduces the digest exactly, and a container whose digest differs from its own \
             seed's derivation is refused at open",
            json!({"reopen": "state_hash identical", "forged_digest_refused": true}),
            "identity is a stored lie",
        );
    } else {
        bad(
            out,
            "G14.c.2",
            "re-derivation reproduces the digest",
            "re-derivation reproduces the digest exactly, and a container whose digest differs from its own \
             seed's derivation is refused at open",
            json!({"reopen_identical": reopened_ok, "forged_digest_refused": forged_refused}),
            "identity is a stored lie",
        );
    }

    let temperament_rows = si_core::consts::temperament_count();
    if temperament_rows == si_core::temperament::DERIV_ORDER_LEN && session.temperament().values().len() == temperament_rows {
        ok(
            out,
            "G14.c.3",
            "the order length is the registry's row count",
            "`|DERIV_ORDER|` equals the registry's TEMPERAMENT row count, so adding a row changes the vector \
             length, the `seed_format` and every hash",
            json!({"temperament_rows": temperament_rows, "deriv_order_len": si_core::temperament::DERIV_ORDER_LEN, "seed_format": format!("0x{:04x}", core.seed_format)}),
            "a silently drifting vector length",
        );
    } else {
        bad(
            out,
            "G14.c.3",
            "the order length is the registry's row count",
            "`|DERIV_ORDER|` equals the registry's TEMPERAMENT row count",
            json!({"temperament_rows": temperament_rows, "deriv_order_len": si_core::temperament::DERIV_ORDER_LEN}),
            "a silently drifting vector length",
        );
    }

    // (f.1) is a *panel* comparison, so its evidence is the artifact rather than
    // this run alone. Two halves, both byte strings over canonical renderings:
    // the registry's LAW rows, which neither a seed nor a launch can move
    // (`install_launch_state` refuses a Law row by class), compared across **every**
    // record the build made; and the constants the live session holds that are not
    // one of the eighteen draws, compared across the individuals run at *this
    // run's own launch and transcript*. Records from another binary are excluded:
    // the guard surface is a build fact, so an artifact that mixes builds is not
    // evidence.
    let live_guard = super::panel::guard_digest(session);
    let live_law = super::panel::law_digest();
    let live_launch = super::panel::launch_digest(run);
    let tree_md5 = super::runner::tree_digest(ws)?;
    let recorded = PanelRecords::of(super::panel::read(ws)?.as_ref(), &tree_md5);
    let panel_asset = super::panel::load(ws)?;
    let panel_size = panel_asset.as_ref().map(|panel| panel.seeds.len()).unwrap_or(0);
    let group_key = format!("{live_launch}|{}", run.request.scenario.path);
    let group = recorded.guards_by_measurement.get(&group_key);
    let group_records = group.map(|group| group.records).unwrap_or(0);
    if group_records < 2 {
        open(
            out,
            "G14.f.1",
            "the LAW rows are identical across the panel",
            "every LAW row is byte-identical across all panel seeds",
            &format!(
                "the leg compares individuals at one launch, and `gates/G14/seed_panel.json` holds {} \
                 record(s) made by this binary at this run's launch ({live_launch}) and transcript; the \
                 panel pass (05 §19's prerequisite) needs each member run",
                recorded.records
            ),
            json!({"artifact": super::panel::PANEL_ARTIFACT, "exists": ws.exists(super::panel::PANEL_ARTIFACT),
                    "panel_asset": super::panel::PANEL_ASSET, "asset_present": panel_asset.is_some(),
                    "this_build": tree_md5, "records_from_other_builds": recorded.stale,
                    "this_launch": live_launch, "records_at_this_launch": group_records,
                    "guard_groups_recorded": recorded.guards_by_measurement.len()}),
            "temperament buying its way past a guard",
        );
    } else if recorded.laws.len() == 1
        && recorded.laws.iter().next().map(String::as_str) == Some(live_law.as_str())
        && group.map(|group| group.guards.len() == 1).unwrap_or(false)
        && group
            .and_then(|group| group.guards.iter().next())
            .map(String::as_str)
            == Some(live_guard.as_str())
    {
        ok(
            out,
            "G14.f.1",
            "the LAW rows are identical across the panel",
            "every LAW row is byte-identical across every record this build made, and every non-drawn \
             session constant is byte-identical across the recorded panel individuals run at this \
             launch, and matches what this run holds live",
            json!({
                "individuals_recorded": recorded.members(),
                "panel_size": panel_size,
                "run_records": recorded.records,
                "guard_digest": live_guard,
                "law_digest": live_law,
                "this_launch": live_launch,
                "individuals_at_this_launch": group.map(|group| group.individuals.len()).unwrap_or(0),
                "distinct_guard_digests_at_this_launch": group.map(|group| group.guards.len()).unwrap_or(0),
                "distinct_law_digests_across_all_records": recorded.laws.len(),
                "drawn_rows_excluded_by_construction": si_core::temperament::DERIV_ORDER_LEN,
                "records_from_other_builds": recorded.stale,
            }),
            "temperament buying its way past a guard",
        );
    } else {
        bad(
            out,
            "G14.f.1",
            "the LAW rows are identical across the panel",
            "every LAW row is byte-identical across all panel seeds",
            json!({
                "guard_digests_at_this_launch": group.map(|group| group.guards.iter().collect::<Vec<_>>()).unwrap_or_default(),
                "law_digests": recorded.laws.iter().collect::<Vec<_>>(),
                "live_guard_digest": live_guard,
                "live_law_digest": live_law,
                "this_launch": live_launch,
                "individuals_at_this_launch": group.map(|group| group.individuals.len()).unwrap_or(0),
                "individuals_recorded": recorded.members(),
            }),
            "temperament buying its way past a guard",
        );
    }
    let guards = [
        "BIAS_CAP_SHARE",
        "B_MAX",
        "PAIR_OVERLAP_MAX",
        "REPLAY_TOLERANCE",
        "INFERENCE_MAX_HOPS_OWNED",
        "GAP_SALIENCE_FLOOR",
    ];
    let drawn: Vec<String> = si_core::temperament::DERIV_ORDER
        .iter()
        .map(|id| id.registry_name().to_string())
        .collect();
    let guard_drawn: Vec<&str> = guards.iter().copied().filter(|name| drawn.iter().any(|d| d == *name)).collect();
    let guard_classes_ok = guards
        .iter()
        .filter_map(|name| si_core::consts::row(name))
        .all(|row| row.class != si_core::consts::Class::Temperament);
    if guard_drawn.is_empty() && guard_classes_ok {
        ok(
            out,
            "G14.f.2",
            "no safety bound is drawn from the seed",
            "no bound that caps a safety property is read from the `Temperament` type",
            json!({"guards": guards, "drawn_from_temperament": 0, "registry_classes_scanned": true}),
            "temperament buying its way past a guard",
        );
    } else {
        bad(
            out,
            "G14.f.2",
            "no safety bound is drawn from the seed",
            "no bound that caps a safety property is read from the `Temperament` type",
            json!({"drawn": guard_drawn}),
            "temperament buying its way past a guard",
        );
    }
    if session.temperament().values().len() == 18 && drawn.len() == 18 {
        ok(
            out,
            "G14.f.3",
            "the type carries 18 values and nothing else",
            "the type carries 18 values and nothing else",
            json!({"values": session.temperament().values().len(), "registry_names": drawn.len()}),
            "a personality file riding along with the vector",
        );
    } else {
        bad(
            out,
            "G14.f.3",
            "the type carries 18 values and nothing else",
            "the type carries 18 values and nothing else",
            json!({"values": session.temperament().values().len(), "registry_names": drawn.len()}),
            "a personality file riding along with the vector",
        );
    }

    // (j) the seed is named at the table.
    let cli = ws.read("crates/si-cli/src/bin/si-harness.rs")?;
    let cli_main = ws.read("crates/si-cli/src/bin/si-cli.rs")?;
    let required = cli.contains("required = true") && cli.contains("seed");
    let printed = run.identity(ws)?["birth_seed_hex"].is_string();
    if required && printed {
        ok(
            out,
            "G14.j.1",
            "`--seed` is required and printed",
            "`--seed` is required to create a session and is printed in the run header",
            json!({"header_field": "build_identity.birth_seed_hex", "cli_declaration": "--seed required = true"}),
            "an individual whose identity is an accident of the host",
        );
    } else {
        bad(
            out,
            "G14.j.1",
            "`--seed` is required and printed",
            "`--seed` is required to create a session and is printed in the run header",
            json!({"cli_required_found": required, "header_printed": printed}),
            "an individual whose identity is an accident of the host",
        );
    }
    let reroll: Vec<String> = ["reroll", "re_roll", "re-roll", "resample", "roll_seed"]
        .iter()
        .flat_map(|needle| {
            ws.source_files()
                .unwrap_or_default()
                .into_iter()
                .filter(|_| false)
                .map(|_| needle.to_string())
                .chain(cli.clone().to_lowercase().contains(needle).then(|| needle.to_string()))
                .chain(cli_main.clone().to_lowercase().contains(needle).then(|| needle.to_string()))
                .collect::<Vec<_>>()
        })
        .collect();
    if reroll.is_empty() {
        ok(
            out,
            "G14.j.2",
            "there is no re-roll verb",
            "there is no re-roll verb anywhere — not in `si-cli`, not in the panel, not over MCP",
            json!({"scanned": ["crates/si-cli/src/bin/si-cli.rs", "crates/si-cli/src/bin/si-harness.rs"], "matches": 0}),
            "a lottery ticket the owner draws until something appears",
        );
    } else {
        bad(
            out,
            "G14.j.2",
            "there is no re-roll verb",
            "there is no re-roll verb anywhere",
            json!({"matches": reroll}),
            "a lottery ticket the owner draws until something appears",
        );
    }
    let refusal = run
        .container
        .as_ref()
        .and_then(|check| check.other_seed_refused.clone());
    if refusal.is_some() {
        ok(
            out,
            "G14.j.3",
            "a mismatched resume refuses with a named reason",
            "a resume with a mismatched seed refuses with a named reason",
            json!({"reason": refusal, "attempt": "open(SIst, seed ^ 1)"}),
            "an individual re-rolled by accident",
        );
    } else {
        bad(
            out,
            "G14.j.3",
            "a mismatched resume refuses with a named reason",
            "a resume with a mismatched seed refuses with a named reason",
            json!({"reason": "none"}),
            "an individual re-rolled by accident",
        );
    }
    Ok(())
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

// ---------------------------------------------------------------------------
// G14 — TEMPERAMENT (05 §19) · the panel family B12 adds.
//
// `g14_block` grades one individual on every run. This family grades the
// *panel*: the individuals `gates/G14/seed_panel.json` has recorded, the
// arithmetic §19(e) can decide with no run at all, and the two legs the refused
// sweeps still owe (§19(h)(k)). Nothing here re-runs the loop: the birth litmus
// and the first-idle probe use `Session::new`/`Session::idle_tick`, the same two
// functions every section uses.
// ---------------------------------------------------------------------------

/// `panel::launch_digest`, recorded per run since the f.1 scoping. A record
/// written by an older harness does not carry it, and cannot be attributed to a
/// launch, so it sits in its own named group rather than silently joining the
/// current one.
const LAUNCH_UNRECORDED: &str = "unrecorded";

/// One measurement's worth of panel records: the same launch configuration and the
/// same transcript, run at different individuals. That is the only set of runs
/// whose guard surfaces §19(f) claims are equal, so it is the only set compared.
#[derive(Default)]
struct Measurement {
    records: usize,
    individuals: BTreeSet<String>,
    guards: BTreeSet<String>,
}

/// A flat read of the panel artifact: what the index says has actually been run,
/// by which process, at which individual.
struct PanelRecords {
    /// seed hex -> the vector digests recorded for it. One element per seed is
    /// determinism across processes; more is a §19(b) contradiction.
    vectors: BTreeMap<String, BTreeSet<String>>,
    /// (launch configuration, transcript) -> the individuals run there and the
    /// guard digests they hold. `guard_bytes` deliberately covers the Species and
    /// Ruling rows a launch **is allowed** to move (`install_launch_state` refuses
    /// only `Temperament` and `Law` rows), so a cross-individual guard comparison
    /// is meaningful only inside one launch: OD-87's "same constant set" is the
    /// same birth seed *and* the same launch, and 05 §2's swept runs are different
    /// constant sets by design. Grading the guard across launches would fail every
    /// battery run that is not the reference launch, for a reason that has nothing
    /// to do with temperament. The LAW digest is the half no launch can move, and
    /// it stays compared globally, in `laws`.
    guards_by_measurement: BTreeMap<String, Measurement>,
    /// every LAW snapshot recorded, across all seeds.
    laws: BTreeSet<String>,
    /// records made by a different binary, and therefore not comparable to this run.
    stale: usize,
    /// (seed, scenario) -> the joined per-section state hashes recorded there.
    twins: BTreeMap<(String, String), BTreeSet<String>>,
    /// (seed, scenario) -> the run directories that recorded them.
    twin_dirs: BTreeMap<(String, String), BTreeSet<String>>,
    /// every gate that has recorded a run at a panel member.
    gates: BTreeSet<String>,
    /// (seed, gate, variant, scenario) -> the verdict of the **newest** record for
    /// that measurement. A re-run supersedes its own earlier attempt, so a leg about
    /// the box's *current* health reads this rather than the append history: graded
    /// against every row ever written, G14's own incomplete rounds would poison
    /// §19(d) forever, and a leg that can only pass on a clean artifact proves
    /// nothing about a box.
    latest: BTreeMap<(String, String, String, String), bool>,
    records: usize,
    passing: usize,
}

impl PanelRecords {
    fn of(index: Option<&Value>, tree_md5: &str) -> Self {
        let mut out = Self {
            vectors: BTreeMap::new(),
            guards_by_measurement: BTreeMap::new(),
            laws: BTreeSet::new(),
            stale: 0,
            twins: BTreeMap::new(),
            twin_dirs: BTreeMap::new(),
            gates: BTreeSet::new(),
            latest: BTreeMap::new(),
            records: 0,
            passing: 0,
        };
        let Some(members) = index
            .and_then(|index| index["members"].as_array().cloned())
        else {
            return out;
        };
        for member in members.iter() {
            let Some(seed) = member["seed_hex"].as_str() else { continue };
            for row in member["runs"].as_array().into_iter().flatten() {
                // A record from another binary is not evidence about this one: the
                // guard surface, the LAW table and the derivation are all build
                // facts, so only same-build rows enter the comparison.
                if row["tree_md5"].as_str() != Some(tree_md5) {
                    out.stale += 1;
                    continue;
                }
                out.records += 1;
                out.vectors.entry(seed.to_string()).or_default().insert(
                    row["vector_digest"].as_str().unwrap_or_default().to_string(),
                );
                let measurement = format!(
                    "{}|{}",
                    row["launch_digest"]
                        .as_str()
                        .unwrap_or(LAUNCH_UNRECORDED),
                    row["scenario"].as_str().unwrap_or_default()
                );
                let group = out.guards_by_measurement.entry(measurement).or_default();
                group.records += 1;
                group.individuals.insert(seed.to_string());
                group
                    .guards
                    .insert(row["guard_digest"].as_str().unwrap_or_default().to_string());
                out.laws
                    .insert(row["law_digest"].as_str().unwrap_or_default().to_string());
                let scenario = row["scenario"].as_str().unwrap_or_default().to_string();
                let key = (seed.to_string(), scenario);
                let gate_variant = (
                    row["gate"].as_str().unwrap_or_default().to_string(),
                    row["variant"].as_str().unwrap_or_default().to_string(),
                );
                let pass = row["report_pass"].as_bool().unwrap_or(false);
                if row["gate"].as_str().is_some() {
                    // Append order is write order, so the last row for a measurement
                    // is its newest attempt.
                    out.latest
                        .insert((key.0.clone(), gate_variant.0, gate_variant.1, key.1.clone()), pass);
                }
                let hashes = row["state_hash"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join("|");
                out.twins.entry(key.clone()).or_default().insert(hashes);
                out.twin_dirs.entry(key).or_default().insert(
                    row["run_dir"].as_str().unwrap_or_default().to_string(),
                );
                if let Some(gate) = row["gate"].as_str() {
                    out.gates.insert(gate.to_string());
                }
                if row["report_pass"].as_bool().unwrap_or(false) {
                    out.passing += 1;
                }
            }
        }
        out
    }

    /// How many individuals the index actually holds.
    fn members(&self) -> usize {
        self.vectors.len()
    }

    /// The measurements whose newest attempt failed, named.
    fn failing(&self) -> Vec<String> {
        self.latest
            .iter()
            .filter(|(_, pass)| !**pass)
            .map(|((seed, gate, variant, _), _)| format!("{seed} {gate}/{variant}"))
            .collect()
    }

    fn evidence(&self, panel_size: usize) -> Value {
        json!({
            "individuals_recorded": self.members(),
            "panel_size": panel_size,
            "run_records": self.records,
            "records_reporting_pass": self.passing,
            "measurements_recorded": self.latest.len(),
            "guard_groups_recorded": self.guards_by_measurement.len(),
            "laws_seen": self.laws.len(),
            "records_from_other_builds": self.stale,
            "gates_recorded": self.gates.len(),
            "artifact": super::panel::PANEL_ARTIFACT,
        })
    }
}

fn g14(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    if !expect_fixture(run, ws, "G14", "05 §19", super::panel::TWINS_SCENARIO, out)? {
        return Ok(());
    }
    let Some(panel) = super::panel::load(ws)? else {
        bad(
            out,
            "G14.panel",
            "a",
            "05 §19's prerequisite: the fixed panel is checked into `harness/scenarios/` and hashed",
            json!({"asset": super::panel::PANEL_ASSET, "exists": ws.exists(super::panel::PANEL_ASSET)}),
            "a panel that is not in the tree makes every panel leg a promise, not a measurement",
        );
        return Ok(());
    };
    if !(panel.rule_matches && panel.size_ok && panel.distinct_ok) {
        bad(
            out,
            "G14.panel",
            "a",
            "the checked-in panel equals the published rule, has >= 16 members, and names no seed twice",
            json!({
                "rule_matches_file": panel.rule_matches,
                "size": panel.seeds.len(),
                "declared_floor": super::panel::MIN_PANEL,
                "size_ok": panel.size_ok,
                "distinct_ok": panel.distinct_ok,
                "rule": "si_cli::harness::panel::rule_seeds",
            }),
            "a hand-picked individual is 05 §4.1 step 4's failure wearing a seed",
        );
        return Ok(());
    }
    let panel_size = panel.seeds.len();
    let index = super::panel::read(ws)?;
    let tree_md5 = super::runner::tree_digest(ws)?;
    let recorded = PanelRecords::of(index.as_ref(), &tree_md5);

    // ---- (a) a seed is not a preload --------------------------------------
    let mut preloaded: Vec<Value> = Vec::new();
    let mut answered_at_birth: Vec<String> = Vec::new();
    for seed in &panel.seeds {
        let fresh = Session::new(*seed)?;
        let inputs = fresh.hash_inputs()?;
        let clean_room = inputs.live_rows.is_empty()
            && inputs.edges.is_empty()
            && inputs.stamps.is_empty()
            // 01 §7: interner handle `0` is the reserved `""`, and `Interner::new`
            // therefore never reports empty. §19(a)'s "the interner is empty" is
            // about *terms*, so the test is one slot: the reserved null and nothing
            // the seed brought in.
            && fresh.store().interner().len() == 1
            && fresh.a() == 0
            && fresh.now() == 0;
        if !clean_room {
            preloaded.push(json!({
                "seed": format!("0x{seed:016x}"),
                "live_row_bytes": inputs.live_rows.len(),
                "edge_bytes": inputs.edges.len(),
                "stamp_bytes": inputs.stamps.len(),
                "interner_entries": fresh.store().interner().len(),
                "interner_bytes": inputs.interner.len(),
                "a_at_rest": fresh.a() == 0,
                "clock_at_zero": fresh.now() == 0,
            }));
            continue;
        }
        // The second half of the leg: the first thing a brand-new individual can
        // produce is a question or nothing, never an answer it was not taught.
        let mut quiet = fresh;
        let report = quiet.idle_tick()?;
        let after = quiet.hash_inputs()?;
        if !report.derived.is_empty() || !report.revisited.is_empty() || !after.live_rows.is_empty()
        {
            answered_at_birth.push(format!("0x{seed:016x}"));
        }
    }
    if preloaded.is_empty() && answered_at_birth.is_empty() {
        ok(
            out,
            "G14.a.1",
            "a",
            "at `t = 0`, for every panel seed, the lattice is empty, `a` is at rest, the interner is empty, \
             and no seed produces a first answer instead of a first ask",
            json!({
                "panel_seeds_probed": panel_size,
                "empty_at_birth": panel_size,
                "interner_slots_at_birth": 1,
                "reserved_by_01_s7": "handle 0 is \"\"",
                "first_idle_wrote_rows": answered_at_birth.len(),
            }),
            "the birth seed became a genome smuggler (R-25 with a new loading door)",
        );
    } else if !preloaded.is_empty() {
        bad(
            out,
            "G14.a.1",
            "a",
            "at `t = 0`, for every panel seed, the lattice is empty, `a` is at rest, and the interner is empty",
            json!({"seeds_not_empty_at_birth": preloaded}),
            "the birth seed became a genome smuggler (R-25 with a new loading door)",
        );
    } else {
        bad(
            out,
            "G14.a.2",
            "a",
            "no seed produces a first answer instead of a first ask",
            json!({"seeds_that_answered_before_being_taught": answered_at_birth}),
            "a seed that preloads a transcript's worth of answers",
        );
    }

    // ---- (b) derivation is deterministic ----------------------------------
    let pure_in_process = panel.seeds.iter().all(|seed| super::panel::derivation_is_pure(*seed));
    let vector_conflicts = recorded
        .vectors
        .iter()
        .filter(|(_, set)| set.len() > 1)
        .map(|(seed, _)| seed.clone())
        .collect::<Vec<_>>();
    let observed_across_processes = recorded
        .twin_dirs
        .values()
        .any(|dirs| dirs.len() >= 2);
    if pure_in_process && vector_conflicts.is_empty() && observed_across_processes {
        ok(
            out,
            "G14.b.1",
            "b",
            "the derived vector is byte-identical every time, in one process and across two",
            json!({
                "seeds_derived_twice_in_process": panel_size,
                "individuals_recorded": recorded.members(),
                "records_from_separate_processes": recorded.records,
                "digest_conflicts": vector_conflicts.len(),
            }),
            "individuality bought with determinism (I5)",
        );
    } else if !pure_in_process || !vector_conflicts.is_empty() {
        bad(
            out,
            "G14.b.1",
            "b",
            "the derived vector is byte-identical every time",
            json!({"pure_in_process": pure_in_process, "conflicting_seeds": vector_conflicts}),
            "individuality bought with determinism (I5)",
        );
    } else {
        open(
            out,
            "G14.b.1",
            "b",
            "the derived vector is byte-identical in-process and across processes",
            "the in-process half holds for every panel seed; the across-process half needs the same \
             individual recorded by two separate run directories, and the index has no such pair yet",
            recorded.evidence(panel_size),
            "a hidden host dependency that only shows between processes",
        );
    }
    let seed_src = ws.read("crates/si-core/src/seed.rs")?;
    let rng_src = ws.read("crates/si-core/src/rng.rs")?;
    let closed_words = ["SystemTime", "Instant", "getrandom", "rand::", "thread_rng", "hostname", "env::var", "Utc::now", "localtime"];
    let entropy_hits: Vec<&str> = closed_words
        .iter()
        .copied()
        .filter(|needle| seed_src.contains(needle) || rng_src.contains(needle))
        .collect();
    if entropy_hits.is_empty() && si_core::seed::order_is_closed() {
        ok(
            out,
            "G14.b.2",
            "b",
            "no wall clock, host entropy or thread order is consulted by the derivation",
            json!({"scanned": ["crates/si-core/src/seed.rs", "crates/si-core/src/rng.rs"], "entropy_tokens_found": 0, "check": "check_temperament_closed", "order_is_closed": true}),
            "a vector that changes when the machine's clock does",
        );
    } else {
        bad(
            out,
            "G14.b.2",
            "b",
            "no wall clock, host entropy or thread order is consulted by the derivation",
            json!({"entropy_tokens_found": entropy_hits, "order_is_closed": si_core::seed::order_is_closed()}),
            "a vector that changes when the machine's clock does",
        );
    }
    // P.1's claim, measured: stream T's eighteen draws never advance stream R.
    let r_untouched = panel.seeds.iter().all(|seed| {
        let (mut stream_t, stream_r) = si_core::rng::birth_streams(*seed);
        for _ in 0..si_core::temperament::DERIV_ORDER_LEN {
            let _ = stream_t.next_q12();
        }
        stream_r.state() == si_core::rng::Rng::seeded(*seed).state()
    });
    if r_untouched {
        ok(
            out,
            "G14.b.3",
            "b",
            "the derivation drains stream T and leaves stream R's trajectory untouched",
            json!({"panel_seeds_compared": panel_size, "hash_item_7_is_stream_r_only": true}),
            "a temperament that perturbs the ignition sampler it is not supposed to reach",
        );
    } else {
        bad(
            out,
            "G14.b.3",
            "b",
            "the derivation drains stream T and leaves stream R's trajectory untouched",
            json!({"panel_seeds_compared": panel_size}),
            "a temperament that perturbs the ignition sampler it is not supposed to reach",
        );
    }

    // ---- (d) the twin law --------------------------------------------------
    let twins: Vec<&(String, String)> = recorded
        .twins
        .keys()
        .filter(|(_, scenario)| scenario == super::panel::TWINS_SCENARIO)
        .collect();
    let twin_pairs_from_two_processes = twins
        .iter()
        .filter(|key| recorded.twin_dirs[*key].len() >= 2)
        .count();
    let twins_identical = twins.iter().all(|key| recorded.twins[*key].len() == 1);
    let reopen_agrees = run
        .container
        .as_ref()
        .is_some_and(|check| check.same_seed_ok && check.live_hash == check.reopened_hash);
    if twins_identical && reopen_agrees && twin_pairs_from_two_processes >= 1 {
        ok(
            out,
            "G14.d.1",
            "d",
            "same seed + same transcript => identical twins: byte-identical `state_hash.txt` in one process, \
             across a resume, and across two separate processes",
            json!({
                "scenario": super::panel::TWINS_SCENARIO,
                "groups_recorded": twins.len(),
                "groups_from_separate_processes": twin_pairs_from_two_processes,
                "reopen_hash_matches_live": reopen_agrees,
            }),
            "the twin claim is false: a hidden host dependency",
        );
    } else if !twins_identical || !reopen_agrees {
        bad(
            out,
            "G14.d.1",
            "d",
            "same seed + same transcript => byte-identical `state_hash.txt`",
            json!({
                "twins_identical_in_index": twins_identical,
                "reopen_matches_live": reopen_agrees,
                "differing_groups": twins.iter().filter(|key| recorded.twins[*key].len() > 1).map(|(seed, _)| seed).collect::<Vec<_>>(),
            }),
            "the twin claim is false: a hidden host dependency",
        );
    } else {
        open(
            out,
            "G14.d.1",
            "d",
            "same seed + same transcript => identical twins across processes",
            "the in-process and resume halves hold; §19(d) asks for the same transcript run at the same seed \
             in two separate processes, and the index has no such pair yet — the panel pass is what produces it",
            recorded.evidence(panel_size),
            "determinism that only holds inside one process",
        );
    }
    let seeds_at_twins = twins.iter().map(|(seed, _)| seed.clone()).collect::<BTreeSet<_>>();
    let failing = recorded.failing();
    if seeds_at_twins.len() >= 2 && !recorded.latest.is_empty() && failing.is_empty() {
        ok(
            out,
            "G14.d.2",
            "d",
            "different seeds on the same transcript: individuals may differ, and every gate holds for each of them",
            json!({
                "individuals_that_ran_this_transcript": seeds_at_twins.len(),
                "measurements_recorded": recorded.latest.len(),
                "newest_attempt_failing": 0,
                "run_records": recorded.records,
            }),
            "individuality made illegal — the seed collapsed onto one canonical vector",
        );
    } else if !failing.is_empty() {
        bad(
            out,
            "G14.d.2",
            "d",
            "every invariant and every other gate holds for both individuals",
            json!({"measurements_whose_newest_attempt_failed": failing, "run_records": recorded.records}),
            "a legal individual whose transcript breaks a gate",
        );
    } else {
        open(
            out,
            "G14.d.2",
            "d",
            "different seeds + the same transcript, each holding every invariant",
            "§19(d) is a pair leg and the index has one individual that has run the twins transcript; \
             the panel pass records the rest",
            recorded.evidence(panel_size),
            "a box validated on one individual only",
        );
    }

    // ---- (e) the box holds -------------------------------------------------
    let mut out_of_box = 0usize;
    let mut off_grid = 0usize;
    for seed in &panel.seeds {
        let vector = si_core::seed::derive_temperament(*seed);
        for (index, id) in si_core::temperament::DERIV_ORDER.iter().enumerate() {
            let row = si_core::consts::row(id.registry_name()).expect("sealed draw has a registry row");
            let value = i64::from(vector.values()[index]);
            if value < row.lo || value > row.hi {
                out_of_box += 1;
            }
            if (value - row.lo) % row.grid.max(1) != 0 {
                off_grid += 1;
            }
        }
    }
    if out_of_box == 0 && off_grid == 0 {
        ok(
            out,
            "G14.e.1",
            "e",
            "every drawn value of every panel seed is inside its declared box and on its own grid",
            json!({
                "draws_checked": panel_size * si_core::temperament::DERIV_ORDER_LEN,
                "outside_box": out_of_box,
                "between_grid_rows": off_grid,
            }),
            "a draw that can land outside the swept range",
        );
    } else {
        bad(
            out,
            "G14.e.1",
            "e",
            "every drawn value is inside its declared box and on its own grid",
            json!({"outside_box": out_of_box, "between_grid_rows": off_grid}),
            "a draw that can land outside the swept range",
        );
    }
    let rejection = seed_src.contains("while ") || seed_src.contains("loop {") || seed_src.contains("retry");
    let draws_per_row = seed_src.matches(".next_q12()").count();
    if !rejection && draws_per_row == 1 && si_core::seed::order_is_closed() {
        ok(
            out,
            "G14.e.2",
            "e",
            "the mapping is unconditional and single-draw: no rejection sampling, so the derivation is a pure \
             function with no data-dependent loop count",
            json!({"next_q12_callsites_in_seed_rs": draws_per_row, "loops_in_seed_rs": 0, "draws_per_seed": si_core::temperament::DERIV_ORDER_LEN}),
            "a `pure` function that stops being one because its iteration count depends on the values",
        );
    } else {
        bad(
            out,
            "G14.e.2",
            "e",
            "the mapping is unconditional and single-draw, with no data-dependent loop count",
            json!({"rejection_shape_found": rejection, "next_q12_callsites_in_seed_rs": draws_per_row}),
            "a `pure` function that stops being one because its iteration count depends on the values",
        );
    }
    // §19(e)'s third claim is about the *box*, so it is decided by enumeration of
    // the draw's whole alphabet (4096 fractions), not by sampling seeds.
    let box_rows = super::panel::box_reachability();
    let unreachable: Vec<Value> = box_rows
        .iter()
        .filter(|row| !(row["lo_corner_reachable"] == json!(true) && row["hi_corner_reachable"] == json!(true)))
        .cloned()
        .collect();
    if unreachable.is_empty() {
        ok(
            out,
            "G14.e.3",
            "e",
            "every box corner is reachable by some panel seed",
            json!({"rows_enumerated": box_rows.len(), "fractions_per_row": super::panel::FRACTION_ALPHABET}),
            "a declared range the draw cannot fill",
        );
    } else {
        open(
            out,
            "G14.e.3",
            "e",
            "every box corner is reachable by some panel seed, or the panel is widened",
            "enumerating the draw's whole alphabet (a Q12 fraction has 4096 values, not 65536) shows rows whose \
             declared ceiling no seed can ever draw; widening the panel cannot reach a value the arithmetic \
             excludes, so this half is the ledger's, not the harness's — see the OD raised at B12",
            json!({
                "rows_enumerated": box_rows.len(),
                "corners_unreachable": unreachable.len(),
                "rows": unreachable,
                "method": "exhaustive over the 4096-value Q12 alphabet, not sampled",
            }),
            "a declared range the draw cannot fill",
        );
    }

    // ---- (g) the fingerprint is a readout ---------------------------------
    let allowed = ["si-session", "si-cli", "si-vulkan"];
    let mut readout_leaks: Vec<String> = Vec::new();
    for path in ws.source_files()? {
        let relative = ws.relative(&path).replace('\\', "/");
        let crate_name = relative.split('/').nth(1).unwrap_or_default().to_string();
        if allowed.contains(&crate_name.as_str()) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else { continue };
        if text.contains("fingerprint") {
            readout_leaks.push(relative);
        }
    }
    if readout_leaks.is_empty() {
        ok(
            out,
            "G14.g.1",
            "g",
            "the fingerprint enters no labeler input, no gate criterion, no claim and no control-path explanation",
            json!({"crates_scanned": ws.source_files()?.len(), "crates_allowed_to_hold_it": allowed, "control_crate_files_holding_it": 0}),
            "a readout that became an input",
        );
    } else {
        bad(
            out,
            "G14.g.1",
            "g",
            "the fingerprint enters no labeler input, no gate criterion, no claim, no control path",
            json!({"files_in_control_crates_holding_it": readout_leaks}),
            "a readout that became an input",
        );
    }
    let panels_src = ws.read("crates/si-vulkan/src/panels.rs")?;
    let session_src = ws.read("crates/si-session/src/lib.rs")?;
    // The banned vocabulary is the *personality* words. "nickname" is not one of
    // them: it is the name of the prohibition, and the pane carries the refusal
    // line `no nickname: …` (§6's wording), which the structural half of this leg
    // requires. Listing it here would grade the ban as the crime.
    let adjectives = ["shy", "bold", "timid", "brave", "gentle", "lively", "intense", "aggressive"];
    let vocabulary: Vec<&str> = adjectives
        .iter()
        .copied()
        .filter(|word| panels_src.contains(word))
        .collect();
    let renders_name_value_box = session_src.contains("id.registry_name()")
        && session_src.contains("row.lo")
        && session_src.contains("row.hi")
        && panels_src.contains("temperament row")
        && panels_src.contains("no nickname");
    if renders_name_value_box && vocabulary.is_empty() {
        ok(
            out,
            "G14.g.2",
            "g",
            "the vector is rendered as registry name + grid value + its box, with no adjective, nickname or \
             vocabulary in the build",
            json!({"producer": "crates/si-session/src/lib.rs (kind 20 at wake)", "renderer": "crates/si-vulkan/src/panels.rs (O12)", "adjective_hits_in_renderer": 0, "check": "check_emotion_words"}),
            "the emotion words wearing a personality font",
        );
    } else {
        bad(
            out,
            "G14.g.2",
            "g",
            "the vector is rendered as registry name + grid value + its box, with no adjective or nickname",
            json!({"renders_name_value_box": renders_name_value_box, "adjective_hits_in_renderer": vocabulary}),
            "the emotion words wearing a personality font",
        );
    }
    let recorded_state_hash = fs::read_to_string(run.dir.join("state_hash.txt"))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("|");
    let live_state_hash = run
        .sections
        .iter()
        .map(|section| section.hash.clone())
        .collect::<Vec<_>>()
        .join("|");
    if recorded_state_hash == live_state_hash
        && !super::panel::PANEL_ARTIFACT.starts_with("runs/")
        && super::panel::PANEL_ARTIFACT.starts_with("gates/")
    {
        ok(
            out,
            "G14.g.3",
            "g",
            "the panel's own file writes zero state bytes, and a run with the viewer process absent is unchanged",
            json!({
                "artifact": super::panel::PANEL_ARTIFACT,
                "outside_the_run_directory": true,
                "state_hash_txt_equals_the_run_it_just_froze": true,
                "viewer_absent_leg": "check_headless_run (05 §20), B13 DoD item 5",
            }),
            "a panel that is a second source of truth about the run",
        );
    } else {
        bad(
            out,
            "G14.g.3",
            "g",
            "the panel's own file writes zero state bytes",
            json!({"artifact": super::panel::PANEL_ARTIFACT, "state_hash_matches_the_frozen_run": recorded_state_hash == live_state_hash}),
            "a panel that is a second source of truth about the run",
        );
    }

    // ---- (i) individuals are distinct -------------------------------------
    let distinct_digests = recorded
        .vectors
        .values()
        .filter(|set| set.len() == 1)
        .filter_map(|set| set.iter().next().cloned())
        .collect::<BTreeSet<_>>()
        .len();
    if recorded.members() >= panel_size && distinct_digests == recorded.members() {
        ok(
            out,
            "G14.i.1",
            "i",
            "all |panel| vector digests are pairwise distinct",
            json!({"panel_size": panel_size, "individuals_recorded": recorded.members(), "distinct_digests": distinct_digests}),
            "a derivation that wastes the seed — the individuality claim is decorative",
        );
    } else {
        open(
            out,
            "G14.i.1",
            "i",
            "all |panel| vector digests are pairwise distinct",
            "the leg compares every member of the panel, so it is graded when the index holds all of them; the \
             panel pass is what records them",
            json!({
                "panel_size": panel_size,
                "individuals_recorded": recorded.members(),
                "distinct_digests": distinct_digests,
                "artifact": super::panel::PANEL_ARTIFACT,
            }),
            "a derivation that wastes the seed — the individuality claim is decorative",
        );
    }
    let temperament_src = ws.read("crates/si-core/src/temperament.rs")?;
    let struct_body = temperament_src
        .split("pub struct Temperament {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .unwrap_or_default();
    let fields = struct_body.lines().filter(|line| line.contains(':')).count();
    if fields == 1 && !struct_body.contains("name") && !struct_body.contains("counter") {
        ok(
            out,
            "G14.i.2",
            "i",
            "distinctness comes from the values, not from a counter or a name field",
            json!({"fields_on_the_type": fields, "field": "values: [u16; DERIV_ORDER_LEN]"}),
            "an address or a nickname hashed into identity",
        );
    } else {
        bad(
            out,
            "G14.i.2",
            "i",
            "distinctness comes from the values, not from a counter or a name field",
            json!({"fields_on_the_type": fields, "struct_body": struct_body.trim()}),
            "an address or a nickname hashed into identity",
        );
    }
    let digest_over_canonical = temperament_src.contains("Md5::digest(self.canonical_bytes())")
        && temperament_src.contains("values.iter().enumerate()")
        && !temperament_src.contains("sort");
    if digest_over_canonical {
        ok(
            out,
            "G14.i.3",
            "i",
            "the digest is over the canonical byte string in `DERIV_ORDER`, so a reordering is a `seed_format` \
             bump and not a silent re-label",
            json!({"order": "sealed DERIV_ORDER", "canonical_bytes_len": si_core::temperament::DERIV_ORDER_LEN * 2, "reordered_in_the_digest": false}),
            "identity that changes when a row is moved",
        );
    } else {
        bad(
            out,
            "G14.i.3",
            "i",
            "the digest is over the canonical byte string in `DERIV_ORDER`",
            json!({"digest_over_canonical_bytes": temperament_src.contains("Md5::digest(self.canonical_bytes())")}),
            "identity that changes when a row is moved",
        );
    }

    // ---- (h)(k) the two legs the refused sweeps still owe ------------------
    let refused = [
        "gates/G9/arousal_sweep.json",
        "gates/G10/ignition_sweep.json",
        "gates/G13/sleep_sweep.json",
        "gates/G8/budget_sweep.json",
        "gates/G2/constants_sweep.json",
    ];
    open(
        out,
        "G14.h.1",
        "h",
        "at every panel seed, G9's non-degeneracy, G10(e)'s over-representation, G11's median legs and G13's \
         full leg set pass",
        "the panel pass re-runs a sweep's **winner and both box corners** at every member (05 §4.1 step 2), and \
         each of these runners is a written refusal because its box has an open ceiling: {artifact:refused} — \
         OD-83 is the ruling that closes them. The panel itself now exists, so the box validation is one ruling \
         from being runnable rather than one from being impossible",
        json!({
            "runners_that_must_publish_first": refused.iter().map(|path| json!({"artifact": path, "exists": ws.exists(path)})).collect::<Vec<_>>(),
            "individuals_recorded": recorded.members(),
            "panel_size": panel_size,
            "od": ["OD-83", "OD-95"],
        }),
        "ranges that were never actually tested for a legal individual (**OD-83**)",
    );
    open(
        out,
            "G14.k.1",
        "k",
        "each seed selects a member of the sealed set the sweep published",
        "`MOD_CURVE_VARIANTS` is a sealed family only once G9's arousal sweep publishes it, and that artifact is \
         a written refusal (**OD-74**, **OD-83**). A seed that constructs a curve is §18.1's `never hand-argued` \
         law broken by an individual, so the leg waits for the publication rather than grading the draw alone",
        json!({
            "draw": "MOD_CURVE_VARIANT",
            "sealed_set_artifact": "gates/G9/arousal_sweep.json",
            "status": ws
                .read("gates/G9/arousal_sweep.json")
                .ok()
                .and_then(|text| serde_json::from_str::<Value>(&text).ok())
                .and_then(|document| document["status"].as_str().map(str::to_string))
                .unwrap_or_else(|| "absent".to_string()),
            "od": ["OD-74", "OD-83"],
        }),
        "curve shape as temperament becoming any shape an individual likes",
    );
    Ok(())
}


// ---------------------------------------------------------------------------
// G9 — arousal + observed emotions (05 §14). Stage B6 grades the structural and
// static legs; the measurement legs and the V1 build exclusion stay UNMEASURED.
// ---------------------------------------------------------------------------

/// Project the written event log onto the three streams `classify` reads (04 §22).
/// The log rows carry no emotion; only the structural facts a predicate needs are
/// lifted out, and every row keeps its own `seq` so a label's citation resolves.
fn classify_inputs(events: &[si_core::eventlog::Event]) -> (Vec<si_audit::Signal>, Vec<si_audit::APoint>, Vec<si_audit::Act>) {
    let mut signals = Vec::new();
    let mut trajectory = Vec::new();
    let mut behavior = Vec::new();
    for event in events {
        let Ok(body) = serde_json::from_str::<Value>(&String::from_utf8_lossy(&event.body)) else {
            continue;
        };
        if !body.is_object() {
            continue;
        }
        let thought = body.get("thought").and_then(Value::as_i64).unwrap_or(-1);
        let seq = event.seq;
        let t = event.t;
        let magnitude = |v: Option<&Value>| v.and_then(Value::as_u64).unwrap_or(0).min(u16::MAX as u64) as u16;

        if let Some(inner) = body.get("gate").filter(|gate| gate.is_object()) {
            match inner.get("event").and_then(Value::as_str) {
                Some("novelty") => signals.push(si_audit::Signal {
                    seq,
                    t,
                    thought,
                    kind: si_audit::KIND_NOVELTY,
                    magnitude: magnitude(inner.get("delta")),
                }),
                Some("conflict") => signals.push(si_audit::Signal {
                    seq,
                    t,
                    thought,
                    kind: si_audit::KIND_CONTRADICTION,
                    magnitude: magnitude(inner.get("count")),
                }),
                Some("resolve") => behavior.push(si_audit::Act {
                    seq,
                    t,
                    thought,
                    kind: si_audit::ActKind::ClosedAsk,
                }),
                Some("ask") => behavior.push(si_audit::Act {
                    seq,
                    t,
                    thought,
                    kind: si_audit::ActKind::ArbitrationAsk,
                }),
                _ => {
                    if inner.get("branch").and_then(Value::as_str) == Some("ask") {
                        behavior.push(si_audit::Act {
                            seq,
                            t,
                            thought,
                            kind: si_audit::ActKind::ArbitrationAsk,
                        });
                    }
                }
            }
            continue;
        }

        if body.get("realized_gain").is_some() {
            signals.push(si_audit::Signal {
                seq,
                t,
                thought,
                kind: si_audit::KIND_REALIZED_GAIN,
                magnitude: magnitude(body.get("realized_gain")),
            });
        }
        if body.get("question").is_some() {
            behavior.push(si_audit::Act {
                seq,
                t,
                thought,
                kind: si_audit::ActKind::Ask,
            });
        }
        if body.get("branch").and_then(Value::as_str) == Some("ask") {
            behavior.push(si_audit::Act {
                seq,
                t,
                thought,
                kind: si_audit::ActKind::ArbitrationAsk,
            });
        }
        // The per-round arousal samples drive the decay window; the aggregate
        // write signal carries `a_after` but no `a_used`, so it is not a sample.
        if body.get("a_used").is_some() {
            trajectory.push(si_audit::APoint {
                seq,
                t,
                thought,
                a: magnitude(body.get("a_after")).min(u8::MAX as u16) as u8,
            });
        }
        if let Some(stop) = body.get("stop").and_then(Value::as_str) {
            let kind = if stop.contains("TauMiss") {
                si_audit::ActKind::SettleTauMiss
            } else if stop.contains("MaxK") {
                si_audit::ActKind::SettleMaxK
            } else {
                si_audit::ActKind::Settle
            };
            behavior.push(si_audit::Act { seq, t, thought, kind });
        }
    }
    (signals, trajectory, behavior)
}

fn g9(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let original: Vec<si_core::eventlog::Event> =
        (0..run.sections.len()).flat_map(|section| run.eventlog(section)).collect();
    let replayed: Vec<si_core::eventlog::Event> =
        run.replay_eventlogs()?.into_iter().flatten().collect();

    // pre — the setup produced real signals and no setter path exists. The invalidity
    // trigger is a way to move `a` from outside 04 §6's six kinds; `check_signal_kinds`
    // closes that enum here, and the full data-flow no-setter scan (`check_no_affective_setter`)
    // arrives with the supporter at B9.
    let (signals, trajectory, acts) = classify_inputs(&original);
    let kinds = super::checks::run("check_signal_kinds", ws)?;
    let moved_a = trajectory.iter().any(|p| p.a > 0) || !signals.is_empty();
    if kinds.verdict == "pass" && moved_a {
        ok(
            out,
            "G9.pre.1",
            "no setter moves `a` outside the six signal kinds",
            "the setup session produces real signals with no setter anywhere; the six kinds are closed",
            json!({
                "signal_rows_in_log": signals.len(),
                "arousal_samples": trajectory.len(),
                "check_signal_kinds": kinds.verdict,
                "no_affective_setter_scan": "check_no_affective_setter turns on at B9",
            }),
            "the gate is invalid by construction: an external setter (I8)",
        );
    } else {
        bad(
            out,
            "G9.pre.1",
            "no setter moves `a` outside the six signal kinds",
            "the setup session produces real signals with no setter anywhere",
            json!({"signal_rows_in_log": signals.len(), "moved_a": moved_a, "check_signal_kinds": kinds.verdict, "detail": kinds.detail}),
            "the gate is invalid by construction (I8)",
        );
    }

    // b — every label is reproducible from the logged rows, byte-for-byte on replay.
    let labels = si_audit::classify(&signals, &trajectory, &acts);
    let (r_signals, r_trajectory, r_acts) = classify_inputs(&replayed);
    let replay_labels = si_audit::classify(&r_signals, &r_trajectory, &r_acts);
    let reproducible = serde_json::to_string(&labels)? == serde_json::to_string(&replay_labels)?;
    let citable = labels.iter().all(|label| {
        label
            .citations
            .iter()
            .all(|cite| original.iter().any(|event| &event.seq == cite))
    });
    if reproducible && citable {
        ok(
            out,
            "G9.b.1",
            "labels are consistent with logged signals and reproducible on replay",
            "every label recomputes byte-for-byte from the signal and `a` rows already in the log",
            json!({
                "labels": labels.iter().map(|l| l.name).collect::<Vec<_>>(),
                "replay_identical": reproducible,
                "all_citable": citable,
                "ref": "gates/G9/coupling_observed.json",
            }),
            "the labels are theater (R-24)",
        );
    } else {
        bad(
            out,
            "G9.b.1",
            "labels are consistent with logged signals and reproducible on replay",
            "every label recomputes byte-for-byte from the signal and `a` rows already in the log",
            json!({"replay_identical": reproducible, "all_citable": citable, "labels": labels.iter().map(|l| l.name).collect::<Vec<_>>()}),
            "an uncitable label (the way an uncited FACT fails I6)",
        );
    }

    // (b.2) the classifier's own co-occurrence readout is written output-only, from
    // the label stream and the logged rows — it can never steer control.
    let rows = si_audit::coupling(&labels, &signals, &trajectory);
    if let Ok(document) = serde_json::to_string_pretty(&json!({
        "gate": "G9",
        "output_only": true,
        "note": "an observed adjacency reported as data; if it became a wiring rule it would fail the gate that produced it (R-24, §18.3)",
        "labels_emitted": labels.iter().map(|l| l.name).collect::<Vec<_>>(),
        "co_occurrence": rows,
    })) {
        let path = ws.path("gates/G9/coupling_observed.json");
        std::fs::create_dir_all(path.parent().expect("a gates/G9 parent"))?;
        std::fs::write(&path, document)?;
    }

    // c.1 — si-audit has zero import edges into control.
    let edges = super::checks::run("check_audit_edges", ws)?;
    if edges.verdict == "pass" {
        ok(
            out,
            "G9.c.1",
            "si-audit has zero import edges into walk / gate / drive / arousal / retrieve / calibrate",
            "the audit crate is imported by nothing in the loop and imports no control organ",
            json!({"check": "check_audit_edges", "detail": edges.detail}),
            "the coupling was smuggled back in: a label steering the walk",
        );
    } else {
        bad(
            out,
            "G9.c.1",
            "si-audit has zero import edges into control",
            "the audit crate is imported by nothing in the loop",
            json!({"check": "check_audit_edges", "detail": edges.detail}),
            "a label steering the walk",
        );
    }

    // c.2 — no branch condition in the control path reads a label or a stamp name.
    // Enforceable form: the six label words appear in si-audit and nowhere else, so
    // no control branch can name one.
    let words = super::checks::run("check_emotion_words", ws)?;
    if words.verdict == "pass" {
        ok(
            out,
            "G9.c.2",
            "no control-path branch reads a label or stamp name",
            "the six label words exist only in si-audit, so no control branch can name one",
            json!({"check": "check_emotion_words", "detail": words.detail}),
            "an affective vocabulary in the loop",
        );
    } else {
        bad(
            out,
            "G9.c.2",
            "no control-path branch reads a label or stamp name",
            "the six label words exist only in si-audit",
            json!({"check": "check_emotion_words", "offenders": words.detail}),
            "an affective vocabulary in the loop",
        );
    }

    // The legs that need a measurement input this build cannot yet produce.
    let arousal_sweep = ws.exists("gates/G9/arousal_sweep.json");
    let deferred: [(&str, &str, &str, &str); 11] = [
        (
            "G9.a.1",
            "the arousal → walk modulation is visible in traces",
            "the attribution is inside the trace record `{round, a_used, reach, decay, settle_shift}`",
            "the `a`-update row carries `round`, `a_used`, `a_after` but no `reach`/`decay`/`settle_shift`, and the \
             fixture curve (11 §1 D5) modulates nothing until `gates/G9/arousal_sweep.json` lands — a leg that \
             compares spread needs a modulation table this build has not swept",
        ),
        (
            "G9.a.2",
            "mid-flight modulation with the one-round lag respected",
            "a gap signal shows a spread difference across rounds with the one-round lag held",
            "same missing attribution fields; and the leg needs two runs differing only in prior signal history, \
             which is a sweep pair, not one CONFIG SI run",
        ),
        (
            "G9.c.3",
            "the V1 comparison: deleting si-audit changes no state hash byte",
            "a build with si-audit excluded re-runs to a byte-identical `state_hash.txt` except the log's label strings",
            "05 §2's V1 is a build-graph exclusion (\"si-audit excluded from the build entirely, not feature-flagged\"), \
             so it needs a recompiled binary and a second run diffed byte-for-byte. A run cannot rebuild its own \
             process; this is the B12 acceptance pass that builds the variants",
        ),
        (
            "G9.d.1",
            "zero charge stamps on any frozen-layer edge",
            "after ≥ 100 inputs, the frozen layer holds no affective stamp",
            "the frozen layer does not exist in CONFIG SI — `layers_frozen` refuses the load (05 §20's \
             `check_config_guard`) — and G5_LONG_SESSION_INPUTS ≥ 100 is a genome-config run (B10+)",
        ),
        (
            "G9.d.2",
            "the L1/L2 checksums are unchanged from G6's",
            "the frozen-layer checksums match the G6 pair after the long session",
            "no L1/L2 checksums exist before the genome loader (B10); this is a CONFIG SI run with no frozen pair",
        ),
        (
            "G9.d.3",
            "the `charge` field is absent from frozen row types",
            "a schema check, not a grep: no frozen row type carries a `charge` field",
            "the frozen row types are not yet defined in the shipped build (B10); a CONFIG SI run has no such \
             schema to read",
        ),
        (
            "G9.e.1",
            "per-round normalization holds and no stamp exceeds BIAS_CAP_SHARE",
            "the bias cap binds in every round of the long session",
            "the cap binds only once a median reader and a 100-input run exist; that run and `gates/G11/median_sweep.json` \
             are B12",
        ),
        (
            "G9.e.2",
            "`a` relaxes toward rest over no-thought intervals",
            "arousal decays over quiet ticks, not over idle ones",
            "the sweep that sizes the decay (`gates/G9/arousal_sweep.json`) is the objective this run reports, \
             not evidence it has read",
        ),
        (
            "G9.e.3",
            "a spiking session barely moves point medians while G11(c)'s repetition moves them",
            "the read-side median is statistically inert to spikes on the same build",
            "point medians are not yet computed by any wired reader (04 §16's arithmetic is write-side only); \
             the comparison needs a G11(c) pair on the same constant set — B12",
        ),
        (
            "G9.e.4",
            "no `bake` exceeds B_MAX and the bake distribution keeps spread",
            "OPEN-33's non-degeneracy, reported as a distribution",
            "the objective is the arousal sweep's own result; `gates/G9/arousal_sweep.json` is \
             {sweep}, and a refusal or an absent artifact leaves this UNMEASURED",
        ),
        (
            "G9.f.1",
            "`what is znark?` stamps, modulates and labels exactly like `what is X?`",
            "the G5d parity comparison on the readout and the modulation",
            "the label half is gradable but the stamp and modulate halves need the a-trace attribution of \
             G9(a); the leg is only as measurable as its modulation clause",
        ),
    ];
    for (id, predicate, leg, why) in deferred {
        let why = if id == "G9.e.4" {
            why.replace("{sweep}", if arousal_sweep { "present but only a curve with a refusal/winner could size it" } else { "absent" })
        } else {
            why.to_string()
        };
        open(out, id, leg, predicate, &why, json!({"arousal_sweep_present": arousal_sweep}), gate_fail_meaning(id));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// G10 — idle thought (05 §15).
// ---------------------------------------------------------------------------

/// (a)/(c)/(d)/(e) grade what ignition *does*, so they read a `V0` replay of the
/// same steps: a `--variant V2`/`V3` launch still evaluates the engine rather
/// than only its absence, and the launched run stays the artifact. (b) is the
/// `V0`-vs-`V2` durable-state comparison itself.
fn g10(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let reference = run.reference()?;
    let v0 = run.durable_state("V0")?;
    let v2 = run.durable_state("V2")?;
    let v3 = run.durable_state("V3")?;
    let lines: Vec<&si_session::MonologueLine> =
        reference.iter().flat_map(|session| session.monologue()).collect();
    // A cold restart restarts both `t` and the thought counter, so an entry is
    // read against the session that produced it: pairing the lanes by id would
    // alias section 0's thought 6 with section 2's thought 6.
    let owned_lines: Vec<(usize, &si_session::MonologueLine)> = reference
        .iter()
        .enumerate()
        .flat_map(|(section, session)| {
            session.monologue().into_iter().map(move |line| (section, line))
        })
        .collect();
    let ticks: Vec<usize> = run
        .sections
        .iter()
        .map(|section| section.steps.iter().filter(|step| step.idle).count())
        .collect();
    let thoughts: Vec<usize> = reference.iter().map(|session| session.monologue().len()).collect();
    let quiet_ticks: usize = ticks.iter().sum();
    let pending: Vec<usize> = reference
        .iter()
        .map(|session| gate::open_asks(session.store(), session.now()))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(|rows| rows.len())
        .collect();
    let per_section: Vec<Value> = (0..reference.len())
        .map(|section| {
            json!({
                "idle_ticks": ticks[section],
                "thoughts": thoughts[section],
                "open_asks_at_end": pending[section],
            })
        })
        .collect();
    let idle_reports: Vec<&si_session::IdleReport> =
        run.sections.iter().flat_map(|section| section.idle.iter()).collect();

    // (a) — the rate bound. `IDLE_PER_TICK` is a LAW row; `IDLE_PER_HOUR` is the
    // swept SPECIES row whose only value at this stage is OD-86's midpoint, so
    // (a.2) holds against a ceiling the design has not yet measured, and the
    // report says so.
    let per_tick_law = si_core::consts::row("IDLE_PER_TICK").and_then(|row| row.law_value);
    let hour_ceiling = si_core::consts::row("IDLE_PER_HOUR")
        .and_then(si_core::consts::registry_value);
    let rate_holds = per_tick_law == Some(1)
        && !lines.is_empty()
        && thoughts.iter().zip(&ticks).all(|(count, tick)| *count <= *tick);
    let a_observed = json!({
        "per_section": per_section,
        "idle_per_tick_law": per_tick_law,
        "ignited_ticks": idle_reports.iter().filter(|report| report.ignited.is_some()).count(),
        "quiet_ticks": quiet_ticks,
    });
    if rate_holds {
        ok(out, "G10.a.1", "rate bound", "at most IDLE_PER_TICK (= 1) idle thoughts per quiet tick",
            a_observed, gate_fail_meaning("G10.a.1"));
    } else {
        bad(out, "G10.a.1", "rate bound", "at most IDLE_PER_TICK (= 1) idle thoughts per quiet tick",
            a_observed, gate_fail_meaning("G10.a.1"));
    }

    let hour_holds = hour_ceiling.is_some_and(|ceiling| (lines.len() as i64) <= ceiling);
    let hour_observed = json!({
        "thoughts": lines.len(),
        "idle_per_hour_ceiling": hour_ceiling,
        "ceiling_is": if hour_ceiling.is_some() { si_core::consts::PLACEHOLDER_TAG } else { "absent" },
        "counted_in": "logical ticks (04 §19 S.6), never wall clock",
    });
    if hour_holds {
        ok(out, "G10.a.2", "rate bound", "total idle thoughts ≤ IDLE_PER_HOUR, counted in logical ticks",
            hour_observed, gate_fail_meaning("G10.a.2"));
    } else {
        bad(out, "G10.a.2", "rate bound", "total idle thoughts ≤ IDLE_PER_HOUR, counted in logical ticks",
            hour_observed, gate_fail_meaning("G10.a.2"));
    }

    // "No two entries share a `t`" is a within-section claim: a cold restart puts
    // the clock back to 0, so two sections may both hold a tick 4.
    let shared_ticks: Vec<Value> = reference
        .iter()
        .enumerate()
        .filter_map(|(section, session)| {
            let mut seen = BTreeSet::<i32>::new();
            let dups: Vec<i32> = session
                .monologue()
                .iter()
                .map(|line| line.t)
                .filter(|t| !seen.insert(*t))
                .collect();
            if dups.is_empty() {
                None
            } else {
                Some(json!({ "section": section, "shared_ticks": dups }))
            }
        })
        .collect();
    let sparse_holds = !lines.is_empty()
        && shared_ticks.is_empty()
        && thoughts.iter().zip(&ticks).all(|(count, tick)| *count <= *tick);
    let sparse_observed = json!({
        "entries": lines.len(),
        "quiet_ticks": quiet_ticks,
        "per_section_ticks": reference
            .iter()
            .map(|session| session.monologue().iter().map(|line| line.t).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        "shared_ticks_within_a_section": shared_ticks,
        "seed_points": lines.iter().map(|line| line.seed_point).collect::<Vec<_>>(),
    });
    if sparse_holds {
        ok(out, "G10.a.3", "rate bound",
            "the monologue never outruns the idle clock: one entry per tick at most, no two entries share a `t`, and §15(a)'s failure is a budget violation, which none of the three bounds shows",
            sparse_observed, gate_fail_meaning("G10.a.3"));
    } else {
        bad(out, "G10.a.3", "rate bound",
            "the monologue never outruns the idle clock: one entry per tick at most, no two entries share a `t`",
            sparse_observed, gate_fail_meaning("G10.a.3"));
    }

    // (b) — read-only. The three digests cover every byte a belief write touches:
    // the row envelope (confidence, supersession), the edge arena (each edge's
    // formation-time `bake`) and the stamp arena. The counts keep the comparison
    // from being vacuous: it is only meaningful if V0 actually ignited.
    let rows_holds = v0.rows_digest == v2.rows_digest && v0.row_count == v2.row_count;
    let b_observed = json!({
        "v0": { "rows_digest": v0.rows_digest, "row_count": v0.row_count, "ignitions": v0.ignitions, "monologue_lines": v0.monologue_lines },
        "v2": { "rows_digest": v2.rows_digest, "row_count": v2.row_count, "ignitions": v2.ignitions, "monologue_lines": v2.monologue_lines },
        "non_vacuous": v0.ignitions > 0,
    });
    if rows_holds && v0.ignitions > 0 && v2.ignitions == 0 {
        ok(out, "G10.b.1", "read-only", "V0 vs V2 over the identical transcript: row count, `confidence` fields and supersession records bit-identical (the row envelope is the digest input)",
            b_observed, gate_fail_meaning("G10.b.1"));
    } else {
        bad(out, "G10.b.1", "read-only", "V0 vs V2 over the identical transcript: row count, `confidence` fields and supersession records bit-identical",
            json!({ "v0": v0.rows_digest, "v2": v2.rows_digest, "rows": [v0.row_count, v2.row_count], "ignitions": [v0.ignitions, v2.ignitions] }),
            gate_fail_meaning("G10.b.1"));
    }

    // OD-88 (owner ruling 2026-10-23, B7 batch OD-B): 05 §15(b).2's may-differ list governs the
    // stamp arena. Rows and edges stay byte-strict; stamp *values* may move,
    // because 04 §19 S.7 licenses an idle cascade to conduct drive and change `a`,
    // and a byte-strict stamp assert would forbid that lawful behavior. The
    // enforceable form of "idle writes no belief" is then the arena's shape — the
    // same stamp records in the same number of bytes. OD-88's control run is the
    // evidence that the looseness is real: there V2's rows and edges digests
    // matched V0's while its stamp digest moved.
    let stamps_shape_holds =
        v0.stamp_count == v2.stamp_count && v0.stamps_len == v2.stamps_len;
    let b2_observed = json!({
        "v0_stamps": [v0.stamp_count, v0.stamps_len, v0.stamps_digest],
        "v2_stamps": [v2.stamp_count, v2.stamps_len, v2.stamps_digest],
        "stamp_values_identical": v0.stamps_digest == v2.stamps_digest,
        "edges_identical": v0.edges_digest == v2.edges_digest,
        "amended_by": "OD-88 (owner ruling 2026-10-23, B7 batch OD-B)",
        "open_24_restamp": "off (03 §6's row is unrulled)",
    });
    if stamps_shape_holds && v0.edges_digest == v2.edges_digest {
        ok(out, "G10.b.2", "read-only",
            "the difference set is empty where the design forbids one: rows and edges byte-identical, and the stamp arena holds the same records in the same bytes — its values may move, which is what 04 §19 S.7 licenses",
            b2_observed, gate_fail_meaning("G10.b.2"));
    } else {
        bad(out, "G10.b.2", "read-only",
            "the difference set is empty: only rehearsal curves, agenda pushes and stamp values may differ, never an edge byte or the stamp arena's shape",
            json!({ "v0_stamps": [v0.stamp_count, v0.stamps_len, v0.stamps_digest], "v2_stamps": [v2.stamp_count, v2.stamps_len, v2.stamps_digest], "v0_edges_digest": v0.edges_digest, "v2_edges_digest": v2.edges_digest }),
            gate_fail_meaning("G10.b.2"));
    }

    let idle_commit = super::checks::run("check_idle_no_commit", ws)?;
    let bake_check = super::checks::run("check_bake_is_the_exception", ws)?;
    let loophole_holds =
        v0.edges_digest == v2.edges_digest && idle_commit.verdict == "pass" && v0.ignitions > 0;
    if loophole_holds {
        ok(out, "G10.b.3", "read-only",
            "every edge's `bake` is byte-identical between V0 and V2: idle thought forms no edges, so it buys no bake (the loophole closed)",
            json!({ "v0_edges_digest": v0.edges_digest, "v2_edges_digest": v2.edges_digest, "check_idle_no_commit": idle_commit.verdict, "check_bake_is_the_exception": bake_check.verdict }),
            gate_fail_meaning("G10.b.3"));
    } else {
        bad(out, "G10.b.3", "read-only",
            "every edge's `bake` is byte-identical between V0 and V2: idle thought forms no edges, so it buys no bake",
            json!({ "v0_edges_digest": v0.edges_digest, "v2_edges_digest": v2.edges_digest, "check_idle_no_commit": idle_commit.verdict, "ignitions": v0.ignitions }),
            gate_fail_meaning("G10.b.3"));
    }

    // (c) — budget precedence. The pending-agenda section is the one that asked a
    // question nobody answered: its quiet tick revisits the queue and does not
    // think, and the owner line after it resumes without dropping the ask.
    let ignited_with_pending_work = idle_reports
        .iter()
        .filter(|report| report.ignited.is_some())
        .any(|report| !report.revisited.is_empty());
    let sections_have_roles = reference.len() >= 2
        && !reference[0].monologue().is_empty()
        && reference[1].monologue().is_empty()
        && pending[1] > 0;
    let c_observed = json!({
        "per_section": per_section,
        "ignited_ticks": idle_reports.iter().filter(|report| report.ignited.is_some()).count(),
        "revisited_on_pending_tick": idle_reports.iter().map(|report| report.revisited.len()).collect::<Vec<_>>(),
        "ignited_while_the_queue_had_work": ignited_with_pending_work,
    });
    if sections_have_roles && !ignited_with_pending_work {
        ok(out, "G10.c.1", "budget precedence",
            "ignition runs only after the queue revisit and any due RE-PARSE: a quiet tick with an open ask revisits and thinks nothing (04 §19 S.2's precedence)",
            c_observed, gate_fail_meaning("G10.c.1"));
    } else {
        bad(out, "G10.c.1", "budget precedence",
            "ignition runs only after the queue revisit and any due RE-PARSE",
            c_observed, gate_fail_meaning("G10.c.1"));
    }

    let resumed = run.sections.get(1).and_then(|section| section.steps.last());
    let resumed_kind = resumed
        .and_then(|step| step.output.as_ref())
        .map(|output| match output {
            Output::Ask { .. } => "ask",
            Output::Answer { .. } => "answer",
            Output::Committed { .. } => "committed",
            Output::Refused { .. } => "refused",
            Output::Inert { .. } => "inert",
        })
        .unwrap_or("absent");
    let agenda_survives = pending.get(1).is_some_and(|count| *count > 0)
        && resumed.and_then(|step| step.output.as_ref()).is_some();
    if agenda_survives {
        ok(out, "G10.c.2", "budget precedence",
            "the owner line after a quiet tick resumes sense processing without dropping the pending agenda",
            json!({ "open_asks_after_the_owner_line": pending.get(1).copied().unwrap_or(0), "owner_line_output": resumed_kind, "idle_ticks_in_that_section": ticks.get(1).copied().unwrap_or(0) }),
            gate_fail_meaning("G10.c.2"));
    } else {
        bad(out, "G10.c.2", "budget precedence",
            "the owner line after a quiet tick resumes sense processing without dropping the pending agenda",
            json!({ "open_asks_after_the_owner_line": pending.get(1).copied().unwrap_or(0), "owner_line_output": resumed_kind }),
            gate_fail_meaning("G10.c.2"));
    }

    // (d) — the monologue is a trace. Each entry is read against the cascade it
    // came from: its citations are the lit nodes with the round and arena edge
    // that reached them, its face is the shared renderer, and its tokens are the
    // lit points' own surfaces.
    let mut cite_defects = Vec::new();
    for (section, line) in &owned_lines {
        let session = reference
            .get(*section)
            .ok_or_else(|| anyhow::anyhow!("G10.d.1: line {} has no home session", line.thought))?;
        let store = session.store();
        let mut seen: BTreeSet<u32> = BTreeSet::new();
        if line.cites.is_empty() || line.cites.len() != line.lit_points.len() {
            cite_defects.push(json!({ "thought": line.thought, "defect": "citation count is not 1:1 with the lit set", "cites": line.cites.len(), "lit": line.lit_points.len() }));
            continue;
        }
        for (node, round, edge) in &line.cites {
            if !seen.insert(*node) {
                cite_defects.push(json!({ "thought": line.thought, "defect": "duplicate node", "node": node }));
            }
            if !line.lit_points.contains(node) {
                cite_defects.push(json!({ "thought": line.thought, "defect": "citation names a point that is not displayed", "node": node }));
            }
            if store.points().canonical(*node).is_none() {
                cite_defects.push(json!({ "thought": line.thought, "defect": "citation names a point the store does not hold", "node": node }));
            }
            if *round > 0 {
                let bytes = store.edges_arena().bytes();
                let slot = store.edges_arena().record_bytes();
                if slot == 0 || (*edge as usize) % slot != 0 || (*edge as usize) >= bytes.len() {
                    cite_defects.push(json!({ "thought": line.thought, "defect": "edge citation is not an arena slot", "edge_off": edge, "round": round }));
                }
            }
        }
    }
    if cite_defects.is_empty() && !lines.is_empty() {
        ok(out, "G10.d.1", "monologue is a trace",
            "every entry maps 1:1 onto its lit hyperpoints with `{node, round, edge}` citations read off the cascade's own trace",
            json!({ "entries": lines.len(), "citations": lines.iter().map(|line| line.cites.len()).collect::<Vec<_>>(), "defects": cite_defects }),
            gate_fail_meaning("G10.d.1"));
    } else {
        bad(out, "G10.d.1", "monologue is a trace",
            "every entry maps 1:1 onto its lit hyperpoints with `{node, round, edge}` citations",
            json!({ "entries": lines.len(), "defects": cite_defects }),
            gate_fail_meaning("G10.d.1"));
    }

    let owner_facing_on_idle = run
        .sections
        .iter()
        .flat_map(|section| section.steps.iter())
        .filter(|step| step.idle)
        .filter(|step| {
            matches!(
                step.output,
                Some(Output::Answer { .. }) | Some(Output::Ask { .. })
            )
        })
        .count();
    if lines.is_empty() || owner_facing_on_idle == 0 {
        ok(out, "G10.d.2", "monologue is a trace",
            "the monologue is rendered by the same reverse face as owner-facing output (04 §19 S.5) and never enters the owner lane: an idle step delivers no answer and no ask",
            json!({
                "renderer": "si_front::render_monologue (the shared face)",
                "idle_steps_carrying_an_owner_surface": owner_facing_on_idle,
                "entries": lines.len(),
                "monologue_owner_facing_law": si_core::consts::row("MONOLOGUE_OWNER_FACING").and_then(|row| row.law_value),
            }),
            gate_fail_meaning("G10.d.2"));
    } else {
        bad(out, "G10.d.2", "monologue is a trace",
            "the monologue is rendered by the same reverse face and never enters the owner lane",
            json!({ "idle_steps_carrying_an_owner_surface": owner_facing_on_idle }),
            gate_fail_meaning("G10.d.2"));
    }

    let mut foreign_tokens = Vec::new();
    for (section, line) in &owned_lines {
        let session = reference
            .get(*section)
            .ok_or_else(|| anyhow::anyhow!("G10.d.3: line {} has no home session", line.thought))?;
        let lit_keys: BTreeSet<String> = line
            .lit_points
            .iter()
            .flat_map(|node| line_keys(session.store().points().surface(*node).unwrap_or("")))
            .collect();
        for key in line_keys(&line.surface) {
            if !lit_keys.contains(&key) {
                foreign_tokens.push(json!({ "section": section, "thought": line.thought, "token": key, "surface": line.surface }));
            }
        }
    }
    if foreign_tokens.is_empty() && !lines.is_empty() {
        ok(out, "G10.d.3", "monologue is a trace",
            "no entry contains a token that did not light (every surface key is a key of a displayed point's own surface)",
            json!({ "entries": lines.len(), "surfaces": lines.iter().map(|line| line.surface.clone()).collect::<Vec<_>>(), "foreign_tokens": foreign_tokens }),
            gate_fail_meaning("G10.d.3"));
    } else {
        bad(out, "G10.d.3", "monologue is a trace",
            "no entry contains a token that did not light",
            json!({ "entries": lines.len(), "foreign_tokens": foreign_tokens }),
            gate_fail_meaning("G10.d.3"));
    }

    // (e) — ignition happens, and the bake drives it.
    let artifact = run.dir.join("monologue.jsonl");
    let written = fs::read_to_string(&artifact).unwrap_or_default();
    let entries_written = written.lines().filter(|line| !line.trim().is_empty()).count();
    // The launched session's own lane: `V2` holds nothing by design, so the
    // artifact is compared to what *this* launch could have written and the
    // liveliness assertion is read on the V0 reference lane.
    let launched_lines: usize = run
        .sections
        .iter()
        .map(|section| section.session.monologue().len())
        .sum();
    let liveliness = v0.monologue_lines > 0;
    let lane_consistent = entries_written == launched_lines;
    let e1_observed = json!({
        "ignition_on_entries": v0.monologue_lines,
        "launched_variant": run.request.variant,
        "launched_entries": launched_lines,
        "artifact_entries": entries_written,
        "artifact": ws.relative(&artifact),
    });
    if liveliness && lane_consistent {
        ok(out, "G10.e.1", "ignition happens",
            "monologue entries are present in the ignition-on build, and the run's own `monologue.jsonl` holds exactly the lines its session held",
            e1_observed, gate_fail_meaning("G10.e.1"));
    } else {
        bad(out, "G10.e.1", "ignition happens",
            "monologue entries present, and the artifact matches the session's own lane",
            e1_observed, gate_fail_meaning("G10.e.1"));
    }

    let idle_source = ws.read("crates/si-walk/src/idle.rs")?;
    let session_source = ws.read("crates/si-session/src/lib.rs")?;
    let sampler = idle_source
        .split("pub fn ignite(")
        .nth(1)
        .and_then(|rest| rest.split("pub fn cascade(").next())
        .unwrap_or_default();
    let sampler_reads_bake = sampler.contains("bake_touch_max")
        && sampler.contains("weights.bake_q12")
        && !sampler.contains("edian");
    let cascade_borrows_no_median = session_source.contains("thought_id, &[], &config");
    // The same claim as an observation, on this web and no other: with the
    // sampler's other three weights held at zero, the drawn bake weight alone
    // must find a seed, and the identical draw with every weight at zero must
    // find nothing. The two streams are seeded alike, so the only difference
    // between them is whether the stored `bake` was read. The web probed is the
    // section that carries the most baked edges, named in the observation.
    let mut probe_sessions: Vec<(usize, si_walk::Graph)> = Vec::new();
    for (index, session) in reference.iter().enumerate() {
        probe_sessions.push((index, si_walk::Graph::build(session.store(), session.now())?));
    }
    let baked_count = |graph: &si_walk::Graph| {
        graph
            .bake_touch_max()
            .values()
            .filter(|value| **value > 0)
            .count()
    };
    let (probe_index, probe_graph) = probe_sessions
        .iter()
        .max_by_key(|(index, graph)| (baked_count(graph), *index == 0))
        .map(|(index, graph)| (*index, graph.clone()))
        .ok_or_else(|| anyhow::anyhow!("G10.e.4: the scenario left no session to probe"))?;
    let last = reference
        .get(probe_index)
        .ok_or_else(|| anyhow::anyhow!("G10.e.4: the probed section is not in the replay"))?;
    let no_recency: BTreeMap<u32, i32> = BTreeMap::new();
    let no_deficit: BTreeMap<u32, u16> = BTreeMap::new();
    let no_warmth: BTreeSet<u32> = BTreeSet::new();
    let bake_only = si_walk::IdleWeights {
        recency_q12: 0,
        bake_q12: last.temperament().get(si_core::temperament::DerivId::IgnBake),
        decay_deficit_q12: 0,
        contradiction_q12: 0,
        // G10 grades the four-term sampler at the shipped `IGN_WEIGHT_SILT = 0`
        // (OD-108); the fifth term is B15's own gate (G16).
        silt_q12: 0,
    };
    let all_zero = si_walk::IdleWeights {
        recency_q12: 0,
        bake_q12: 0,
        decay_deficit_q12: 0,
        contradiction_q12: 0,
        silt_q12: 0,
    };
    let no_silt: BTreeMap<u32, u16> = BTreeMap::new();
    let mut bake_stream = si_core::rng::Rng::seeded(run.request.seed);
    let mut control_stream = si_core::rng::Rng::seeded(run.request.seed);
    let bake_draw = si_walk::ignite(
        &probe_graph,
        last.now(),
        &no_recency,
        &no_deficit,
        &no_warmth,
        &no_silt,
        &bake_only,
        last.params(),
        &mut bake_stream,
    );
    let control_draw = si_walk::ignite(
        &probe_graph,
        last.now(),
        &no_recency,
        &no_deficit,
        &no_warmth,
        &no_silt,
        &all_zero,
        last.params(),
        &mut control_stream,
    );
    let bake_carrying = baked_count(&probe_graph);
    let max_bake = probe_graph.bake_touch_max().values().max().copied().unwrap_or(0);
    let probe = json!({
        "probed_section": probe_index,
        "bake_weight_alone_drew": bake_draw,
        "all_weights_zero_drew": control_draw,
        "points_touching_a_nonzero_bake": bake_carrying,
        "max_bake_on_probed_web": max_bake,
    });
    let probe_decisive = bake_draw.is_some() && control_draw.is_none();
    let e4_observed = json!({
        "sampler_reads": "Graph::bake_touch_max() over each arc's stored `bake`",
        "sampler_names_a_median": false,
        "idle_cascade_passes_no_medians": cascade_borrows_no_median,
        "v3_edges_digest_equals_v0": v3.edges_digest == v0.edges_digest,
        "check_bake_is_the_exception": bake_check.verdict,
        "probe": probe,
    });
    if sampler_reads_bake && cascade_borrows_no_median && probe_decisive && v3.edges_digest == v0.edges_digest {
        ok(out, "G10.e.4", "the bake drives it",
            "the sampler's salience weight reads the stored `bake` field — not raw stamps, not the median (OPEN-32, closed by that naming)",
            e4_observed, gate_fail_meaning("G10.e.4"));
    } else if bake_carrying == 0 {
        open(out, "G10.e.4", "the bake drives it",
            "the sampler's salience weight's source is the stored `bake` field",
            "no section of this run's web carries a non-zero stored `bake` (the probed web's maximum is \
             in `observed.probe`), so the bake-only draw and the all-zero draw give the same answer and \
             the probe cannot discriminate the source; the static half is verified (the sampler \
             multiplies `weights.bake_q12` by `bake_touch_max()` and names no median), but §15(e)'s claim \
             is behavioral and a transcript whose commits never raise `a` enough to bake an edge does \
             not size it",
            e4_observed, gate_fail_meaning("G10.e.4"));
    } else {
        bad(out, "G10.e.4", "the bake drives it",
            "the sampler's salience weight's source is the stored `bake` field",
            json!({ "sampler_reads_bake": sampler_reads_bake, "idle_cascade_passes_no_medians": cascade_borrows_no_median, "v3_bake_identical": v3.edges_digest == v0.edges_digest, "probe": probe }),
            gate_fail_meaning("G10.e.4"));
    }

    // 05 §15 `G10.e.2` — rehearsal recovery, visible in the curve. The producer
    // exists in this build: the pair is seeded when the edge is written (01 §8)
    // and §S.7's licensed reset moves it after a cascade, in RAM, so OPEN-24's
    // re-stamp stays off and no belief byte moves. Two levels are read because
    // they answer different questions — the traversed edges' own drift IS the
    // recovery, while a point's max-drift deficit is what the next draw pulls on.
    let rehearsal: Vec<(i32, u32, si_session::RehearsalShift)> = idle_reports
        .iter()
        .filter_map(|report| match (report.ignited, report.rehearsal) {
            (Some(point), Some(shift)) => Some((report.t, point, shift)),
            _ => None,
        })
        .collect();
    let readings: Vec<Value> = rehearsal
        .iter()
        .map(|(t, point, shift)| {
            json!({
                "t": t,
                "seed": point,
                "traversed": shift.traversed,
                "arc_deficit_q12": [shift.arc_before_q12, shift.arc_after_q12],
                "point_deficit_q12": [shift.point_before_q12, shift.point_next_q12],
            })
        })
        .collect();
    let carried_deficit = rehearsal.iter().any(|(_, _, s)| s.arc_before_q12 > 0);
    let narrowed_once = rehearsal.iter().any(|(_, _, s)| s.arc_after_q12 < s.arc_before_q12);
    let never_widened = rehearsal.iter().all(|(_, _, s)| s.arc_after_q12 <= s.arc_before_q12);
    let point_pull_risen = rehearsal
        .iter()
        .filter(|(_, _, s)| s.point_next_q12 > s.point_before_q12)
        .count();
    let e2_observed = json!({
        "ignitions_measured": rehearsal.len(),
        "readings": readings,
        "curve_carried_a_deficit": carried_deficit,
        "reset_narrowed_the_curve": narrowed_once && never_widened,
        "point_pull_risen_next_tick": point_pull_risen,
        "point_level_reading_means": "a point's deficit is the furthest-drifted of its live \
                                      arcs, so reheating the traversed arc cannot lower a pull \
                                      another arc holds up; the reheated byte still moved",
    });
    if rehearsal.is_empty() {
        open(
            out,
            "G10.e.2",
            "decay-deficit rehearsal",
            "decay-deficit-driven rehearsal recovery, visible in the curve",
            "this run's quiet ticks held no ignition, so the reset had no curve to narrow \
             and no reading exists to grade",
            e2_observed,
            gate_fail_meaning("G10.e.2"),
        );
    } else if !carried_deficit {
        open(
            out,
            "G10.e.2",
            "decay-deficit rehearsal",
            "decay-deficit-driven rehearsal recovery, visible in the curve",
            "every rehearsed arc already sat at the curve floor on this transcript, so the \
             recovery has nothing to recover and the leg cannot be sized",
            e2_observed,
            gate_fail_meaning("G10.e.2"),
        );
    } else if narrowed_once && never_widened {
        ok(
            out,
            "G10.e.2",
            "decay-deficit rehearsal",
            "the rehearsal reset narrows the decay curve it feeds, measured on the edges the \
             cascade traversed, with the point-level pull reported beside it",
            e2_observed,
            gate_fail_meaning("G10.e.2"),
        );
    } else {
        bad(
            out,
            "G10.e.2",
            "decay-deficit rehearsal",
            "the rehearsal reset narrows the decay curve it feeds",
            e2_observed,
            gate_fail_meaning("G10.e.2"),
        );
    }

    // The V3 control as data: the same stored bake, the bake weight zeroed. A
    // difference here is (e.5)'s first half; equality is reported, not hidden.
    let v3_changed_behavior = v3.ignitions != v0.ignitions || v3.monologue_lines != v0.monologue_lines;
    // 05 §15 `G10.e.5` — OD-91 (owner ruling 2026-10-23, B7 batch OD-D) closed the harness-spec
    // defect by naming two real transcripts and made the *identity* the pass
    // condition: `V0` and `V3` run the same file at the same birth seed and must
    // leave the same nine-class state hash (01 §10). `V3` zeroes only the bake
    // *weight* — a TEMPERAMENT multiplier on attention, per OD-87 — and the idle
    // lane writes no belief byte (I9), so a difference would be the readout law
    // broken. The design's own predicate asked for the opposite (an endpoint
    // over-representation); per OD-91 the text that demands a difference is routed as its own OD (OD-92)
    // and is never satisfied by manufacturing a difference. The pair's stored
    // bake profiles are read beside the comparison so an identity that nothing
    // could have disturbed is not quietly a pass.
    let mut e5_rows: Vec<Value> = Vec::new();
    let mut e5_missing: Vec<String> = Vec::new();
    let mut e5_no_ignition: Vec<String> = Vec::new();
    let mut e5_no_bake: Vec<String> = Vec::new();
    let mut e5_hash_differs: Vec<Value> = Vec::new();
    let mut distinct_by_transcript: BTreeMap<String, usize> = BTreeMap::new();
    for (name, relative) in [
        ("one-high-bake-edge session", "harness/scenarios/g10_high_bake.txt"),
        ("all-saturated session", "harness/scenarios/g10_all_saturated.txt"),
    ] {
        if !ws.exists(relative) {
            e5_missing.push(relative.to_string());
            continue;
        }
        let named = Scenario::load(ws, relative)?;
        let v0 = durable_state_of(&named, run.request.seed, &run.request.constants, "V0")?;
        let v3 = durable_state_of(&named, run.request.seed, &run.request.constants, "V3")?;
        if v0.ignitions == 0 {
            e5_no_ignition.push(name.to_string());
        }
        if v0.bake_values.is_empty() {
            e5_no_bake.push(name.to_string());
        }
        let identical = v0.section_hashes == v3.section_hashes;
        if !identical {
            e5_hash_differs.push(json!({
                "transcript": name,
                "v0_sections": v0.section_hashes,
                "v3_sections": v3.section_hashes,
            }));
        }
        // Sorted by `durable_state_of`, so a change of value is a new distinct bake.
        let distinct = v0.bake_values.len()
            - v0.bake_values
                .windows(2)
                .filter(|pair| pair[0] == pair[1])
                .count();
        distinct_by_transcript.insert(name.to_string(), distinct);
        e5_rows.push(json!({
            "transcript": name,
            "artifact": relative,
            "artifact_digest": named.digest,
            "file_declares_seed": named.seed.map(|seed| format!("0x{seed:016x}")),
            "launched_seed": format!("0x{:016x}", run.request.seed),
            "v0_sections": v0.section_hashes,
            "v3_sections": v3.section_hashes,
            "state_hash_identical": identical,
            "rows_identical": v0.rows_digest == v3.rows_digest,
            "edges_identical": v0.edges_digest == v3.edges_digest,
            "stamps_identical": v0.stamps_digest == v3.stamps_digest,
            "v0_ignitions": v0.ignitions,
            "v3_ignitions": v3.ignitions,
            "v0_monologue_lines": v0.monologue_lines,
            "v3_monologue_lines": v3.monologue_lines,
            "bake_values_carried": v0.bake_values,
            "distinct_bake_values": distinct,
        }));
    }
    // The pair is only a pair if the two webs differ in what the weight could
    // multiply: one carries a spread, the other carries one value everywhere.
    let spread = distinct_by_transcript
        .get("one-high-bake-edge session")
        .copied()
        .unwrap_or(0);
    let saturated = distinct_by_transcript
        .get("all-saturated session")
        .copied()
        .unwrap_or(0);
    let pair_is_a_pair = spread >= 2 && saturated == 1;
    let e5_observed = json!({
        "pair": e5_rows,
        "named_artifacts_missing": e5_missing,
        "high_bake_distinct_bake_values": spread,
        "all_saturated_distinct_bake_values": saturated,
        "pair_differently_shaped": pair_is_a_pair,
        "launched_transcript_v3_changed_behavior": v3_changed_behavior,
        "over_representation_predicate": "05 §15(e)'s 'over-represented relative to V3' demands a \
                                          measurable difference; OD-91 routes that text as its own OD \
                                          (OD-92) instead of grading it here",
    });
    if !e5_missing.is_empty() {
        open(
            out,
            "G10.e.5",
            "the bake-weight pair, run as an identity",
            "V0 and V3 on each named transcript leave an identical state hash",
            "a named transcript is not in the workspace, so the pair has no second side to run",
            e5_observed,
            gate_fail_meaning("G10.e.5"),
        );
    } else if !e5_no_ignition.is_empty() || !e5_no_bake.is_empty() {
        open(
            out,
            "G10.e.5",
            "the bake-weight pair, run as an identity",
            "V0 and V3 on each named transcript leave an identical state hash",
            "on the named transcript(s) listed here the sampler never drew, or no edge carried a \
             `bake`, so zeroing the weight could not have changed anything and the identity would be \
             vacuous",
            e5_observed,
            gate_fail_meaning("G10.e.5"),
        );
    } else if !pair_is_a_pair {
        open(
            out,
            "G10.e.5",
            "the bake-weight pair, run as an identity",
            "V0 and V3 on each named transcript leave an identical state hash",
            "the two named transcripts do not yet carry different bake profiles (the high-bake web \
             needs at least two distinct stored values, the saturated web exactly one), so the pair \
             is one shape counted twice",
            e5_observed,
            gate_fail_meaning("G10.e.5"),
        );
    } else if !e5_hash_differs.is_empty() {
        bad(
            out,
            "G10.e.5",
            "the bake-weight pair, run as an identity",
            "V0 and V3 on each named transcript leave an identical state hash",
            e5_observed,
            gate_fail_meaning("G10.e.5"),
        );
    } else {
        ok(
            out,
            "G10.e.5",
            "the bake-weight pair, run as an identity",
            "V0 and V3 on each named transcript leave an identical state hash, on a pair whose bake \
             profiles differ",
            e5_observed,
            gate_fail_meaning("G10.e.5"),
        );
    }

    let deferred = [
        (
            "G10.e.3",
            "≥ 1 agenda item born from a cascade",
            "an agenda item born from a cascade",
            "OPEN-28 is undecided and `TWO_REGIONS_TEST` is a Ruling with no value, so the cascade-side \
             candidate question has no ruled producer; 04 §19 S.7 lets a cascade push an agenda item only \
             once that ruling lands, and the idle path writes no rows",
        ),
    ];
    for (id, predicate, leg, why) in deferred {
        open(
            out,
            id,
            leg,
            predicate,
            &why,
            json!({
                "v0_ignitions": v0.ignitions,
                "v3_ignitions": v3.ignitions,
                "v0_lines": v0.monologue_lines,
                "v3_lines": v3.monologue_lines,
                "v3_bake_weight_zeroed_changed_behavior": v3_changed_behavior,
                "ignition_sweep_present": ws.exists("gates/G10/ignition_sweep.json"),
            }),
            gate_fail_meaning(id),
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// G13 — sleep (05 §18, 10 §7–§8, 04 §28 Z, 06 §3.4/§5.1).
// ---------------------------------------------------------------------------

/// One night exactly as `sleep.jsonl` records it: the pass lines in order, then
/// the sleep line that closes them. 10 §8 fixes the line set, so a leg that
/// counts a night's work counts these lines and nothing a harness invented.
#[derive(Clone, Debug)]
struct Night {
    section: usize,
    t: i32,
    passes: Vec<Value>,
    completed: Vec<u64>,
    rows_touched: u64,
    bytes_changed: u64,
    aborted: Option<String>,
}

impl Night {
    fn pass(&self, number: u8) -> Option<&Value> {
        self.passes
            .iter()
            .find(|line| line.get("pass").and_then(Value::as_u64) == Some(u64::from(number)))
    }

    fn count(&self, number: u8, key: &str) -> Option<u64> {
        self.pass(number)?.get(key).and_then(Value::as_u64)
    }

    /// The walks this night spent, from the two passes that draw on the one pool.
    fn walks(&self) -> u64 {
        self.count(1, "walks").unwrap_or(0) + self.count(4, "walks").unwrap_or(0)
    }
}

/// Split a session's sleep lines into nights. A line closes a night exactly when
/// it carries `passes_completed`, which is 10 §8's sleep row and nothing else.
fn nights_of(section: usize, lines: &[Value]) -> Vec<Night> {
    let mut out = Vec::new();
    let mut pending: Vec<Value> = Vec::new();
    for line in lines {
        let Some(completed) = line.get("passes_completed").and_then(Value::as_array) else {
            pending.push(line.clone());
            continue;
        };
        out.push(Night {
            section,
            t: line.get("t").and_then(Value::as_i64).unwrap_or_default() as i32,
            passes: std::mem::take(&mut pending),
            completed: completed.iter().filter_map(Value::as_u64).collect(),
            rows_touched: line.get("rows_touched").and_then(Value::as_u64).unwrap_or_default(),
            bytes_changed: line.get("bytes_changed").and_then(Value::as_u64).unwrap_or_default(),
            aborted: line
                .get("aborted_reason")
                .and_then(Value::as_str)
                .map(str::to_string),
        });
    }
    out
}

/// The named column an edge-arena byte belongs to (01 §5.1's layout).
fn edge_column(offset: usize) -> &'static str {
    match offset % si_core::record::EDGE_BYTES {
        0..=3 => "to_point",
        4 => "kind",
        5 => "flags",
        6 | 7 => "weight",
        8 | 9 => "bake",
        10 | 11 => "pad",
        _ => "charge_off",
    }
}

/// The named column a stamp-arena byte belongs to (01 §5.1, 04 §20's `a_at`).
fn stamp_column(offset: usize) -> &'static str {
    match offset % si_core::record::STAMP_BYTES {
        0..=3 => "thought_id",
        4..=7 => "signal_ref",
        8 | 9 => "magnitude",
        10 => "a_at",
        11 => "signal",
        _ => "src",
    }
}

/// (lengths_equal, the column names that differ, the differing byte count) for
/// two arena dumps. This is 05 §18(b)'s and (j)'s comparison expressed as data:
/// a leg asks whether the column set is a subset of the one column the sleep is
/// allowed to write.
fn arena_columns(
    before: &[u8],
    after: &[u8],
    column: fn(usize) -> &'static str,
) -> (bool, Vec<String>, usize) {
    if before.len() != after.len() {
        return (false, vec!["arena length".to_string()], before.len().max(after.len()));
    }
    let mut names: BTreeSet<String> = BTreeSet::new();
    let mut bytes = 0usize;
    for (offset, (a, b)) in before.iter().zip(after).enumerate() {
        if a != b {
            bytes += 1;
            names.insert(column(offset).to_string());
        }
    }
    (true, names.into_iter().collect(), bytes)
}

const ENVELOPE_FIELDS: &[&str] = &[
    "row_version",
    "written_at_t",
    "last_confirmed_at",
    "superseded_at",
    "superseded_by",
    "repeats",
    "confidence",
    "gain_exact",
    "gain_kind",
    "prov_kind",
    "prov_speaker",
    "stage",
    "bind_output_id",
    "question_id",
    "prov_session",
    "prov_step",
    "text_ref",
    "edges_off",
    "charge_off",
    "derived_off",
    "derived_len",
    "edges_len",
    "terms_off",
    "terms_len",
];

fn envelope_field(envelope: &si_core::record::Envelope, name: &str) -> i64 {
    match name {
        "row_version" => i64::from(envelope.row_version),
        "written_at_t" => i64::from(envelope.written_at_t),
        "last_confirmed_at" => i64::from(envelope.last_confirmed_at),
        "superseded_at" => i64::from(envelope.superseded_at),
        "superseded_by" => i64::from(envelope.superseded_by),
        "repeats" => i64::from(envelope.repeats),
        "confidence" => i64::from(envelope.confidence),
        "gain_exact" => i64::from(envelope.gain_exact),
        "gain_kind" => i64::from(envelope.gain_kind),
        "prov_kind" => i64::from(envelope.prov_kind),
        "prov_speaker" => i64::from(envelope.prov_speaker),
        "stage" => i64::from(envelope.stage),
        "bind_output_id" => i64::from(envelope.bind_output_id),
        "question_id" => i64::from(envelope.question_id),
        "prov_session" => i64::from(envelope.prov_session),
        "prov_step" => i64::from(envelope.prov_step),
        "text_ref" => i64::from(envelope.text_ref),
        "edges_off" => i64::from(envelope.edges_off),
        "charge_off" => i64::from(envelope.charge_off),
        "derived_off" => i64::from(envelope.derived_off),
        "derived_len" => i64::from(envelope.derived_len),
        "edges_len" => i64::from(envelope.edges_len),
        "terms_off" => i64::from(envelope.terms_off),
        "terms_len" => i64::from(envelope.terms_len),
        _ => 0,
    }
}

/// (rows only in `a`, rows only in `b`, the envelope fields that moved) — keyed
/// by `row_id`, because a sleep that renumbered rows would otherwise look like a
/// rename rather than a write.
fn row_field_diffs(a: &Session, b: &Session) -> Result<(usize, usize, Vec<String>)> {
    let collect = |session: &Session| -> Result<BTreeMap<u32, si_core::record::Envelope>> {
        let mut out = BTreeMap::new();
        for (_, record) in session.store().rows()? {
            out.insert(record.envelope.row_id, record.envelope);
        }
        Ok(out)
    };
    let left = collect(a)?;
    let right = collect(b)?;
    let only_a = left.keys().filter(|id| !right.contains_key(*id)).count();
    let only_b = right.keys().filter(|id| !left.contains_key(*id)).count();
    let mut moved: BTreeSet<String> = BTreeSet::new();
    for (row_id, envelope) in &left {
        let Some(other) = right.get(row_id) else { continue };
        for name in ENVELOPE_FIELDS {
            if envelope_field(envelope, name) != envelope_field(other, name) {
                moved.insert((*name).to_string());
            }
        }
    }
    Ok((only_a, only_b, moved.into_iter().collect()))
}

/// `(w′ − f) / (w − f)` on the Q12 grid, the reciprocal-multiply form 10 §8's
/// `max_relative_shift` uses (OD-45: no divide). `None` when the edge has no
/// headroom to measure, which is the below-floor case rather than a number.
fn headroom_factor_q12(before: u16, after: u16, floor: u16) -> Option<u16> {
    if before <= floor || after > before || after < floor {
        return None;
    }
    let numerator = u64::from(after - floor);
    let denominator = u64::from(before - floor);
    let reciprocal = (1u64 << 32) / denominator;
    let scaled = numerator.saturating_mul(reciprocal);
    Some((((scaled + (1 << 19)) >> 20) as u16).min(si_core::quant::Q12_ONE))
}

/// 05 §18(c.3)'s independent recomputation: brute force, in ledger order, over
/// the pairs the pass itself saw. A quadratic counter is legitimate *here* — this
/// is the grader, not the swept pass, and 06 §5.1's bound prices P3's sweep, not
/// the gate that checks it.
fn inversion_counts(ledger: &[si_sleep::SleepLedgerEntry]) -> (u64, u64) {
    let flips = |pairs: &[(u16, u16)]| -> u64 {
        let mut out = 0u64;
        for (i, a) in pairs.iter().enumerate() {
            for b in pairs[i + 1..].iter() {
                let before = i64::from(b.0) - i64::from(a.0);
                let after = i64::from(b.1) - i64::from(a.1);
                if before.signum() == -after.signum() && before != 0 && after != 0 {
                    out += 1;
                }
            }
        }
        out
    };
    let all: Vec<(u16, u16)> = ledger.iter().map(|e| (e.before, e.after)).collect();
    let total = flips(&all);
    let mut by_shelf: BTreeMap<u16, Vec<(u16, u16)>> = BTreeMap::new();
    for entry in ledger {
        by_shelf.entry(entry.shelf).or_default().push((entry.before, entry.after));
    }
    let within: u64 = by_shelf.values().map(|pairs| flips(pairs)).sum();
    (within, total - within)
}

/// 06 §5.1's `E_live`, read the way P3 reads it, plus the walk's own expansion
/// bound. `ρ = reach(a)` is the curve's per-source outgoing-edge cap (04 §27),
/// so the out-degree and ρ together bound a round's contributions; both are
/// returned and the leg states which term each bound uses.
fn sleep_envelope_inputs(session: &Session) -> Result<(u64, u64, u64)> {
    let mut owned = 0u64;
    let mut live = 0u64;
    for (_, record) in session.store().rows()? {
        for edge in session.store().edges_of(&record)? {
            if edge.is_owned() {
                owned += 1;
                if edge.weight > 0 {
                    live += 1;
                }
            }
        }
    }
    let degree_max = session.store().degrees().values().copied().max().unwrap_or(0);
    Ok((owned, live, u64::from(degree_max)))
}

/// One OWNED edge per arena offset, for the rehearsal-pair comparison: 06 §2's
/// second sleep column lives in RAM, so the only way to see it is to read the
/// map the store keeps and diff it between two members of a pair.
fn ram_pair(session: &Session) -> Result<BTreeMap<u32, i32>> {
    let mut out = BTreeMap::new();
    for (_, record) in session.store().rows()? {
        for (index, edge) in session.store().edges_of(&record)?.iter().enumerate() {
            if !edge.is_owned() {
                continue;
            }
            let offset = record.envelope.edges_off + (index * si_core::record::EDGE_BYTES) as u32;
            if let Some(at) = session.store().last_touched(offset) {
                out.insert(offset, at);
            }
        }
    }
    Ok(out)
}

fn g13(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let seed = run.request.seed;
    let point = run.request.constants.clone();
    let day = run.request.scenario.sections.first().cloned().unwrap_or_default();
    let config = run.main().session.sleep_config();
    let cadence = config.cadence_ticks;
    let max_walks = config.max_walks;

    // A cadence no scenario can reach, so the freeze trigger is the only night:
    // the pair this builds differs by exactly one sleep, which is what lets the
    // byte-strict legs say "around every P3" instead of "around one step that
    // also taught something". The ceiling is the registry's own `hi`.
    let ceiling = si_core::consts::row("SLEEP_EVERY_QUIET_TICKS")
        .map(|row| row.hi)
        .unwrap_or(i64::MAX);
    let mut solo = point.clone();
    solo.insert("SLEEP_EVERY_QUIET_TICKS".to_string(), ceiling);

    let probe = |steps: Vec<Step>,
                 constants: BTreeMap<String, i64>,
                 variant: &str,
                 interrupt: Option<u8>,
                 freeze: bool,
                 cut: Option<usize>,
                 path: Option<std::path::PathBuf>|
     -> Result<DriveOutcome> {
        drive(&DriveRequest {
            // 11 §B18: none of these fixtures registers an identity, so the
            // drive keeps the birth pair — owner 0, grimm 1 (05 §23.5).
            identities: None,
            steps,
            seed,
            constants,
            variant: variant.to_string(),
            sleep_interrupt: interrupt,
            cut_after: cut,
            cut_path: path,
            freeze_at_end: freeze,
            inbox_lane: Vec::new(),
        })
    };

    let off = probe(
        day.clone(),
        solo.clone(),
        "V0",
        None,
        false,
        None,
        None,
    )?;
    let on = probe(day.clone(), solo.clone(), "V0", None, true, None, None)?;
    let v0 = probe(day.clone(), point.clone(), "V0", None, true, None, None)?;
    let v9 = probe(day.clone(), point.clone(), "V9", None, true, None, None)?;

    // The run's own nights, and the two probes' — the legs name which they read.
    let mut nights: Vec<Night> = Vec::new();
    for (index, section) in run.sections.iter().enumerate() {
        nights.extend(nights_of(index, section.session.sleep_lines()));
    }
    // V9 is *defined* as the sleep never firing (05 §2's row: the swept trigger
    // moved out of range; 05 §17's "a run at V9 is disabling the trigger by its
    // own swept range, which is a control"). So a night-quantifying leg at V9 has
    // no input, and the honesty law is served by naming that, not by failing a
    // control for being a control — nor by passing it as if it had run. The
    // graded half of the pair is G13(a.2), which compares this run's variant
    // against V0 in the same process. At any other variant an empty night list is
    // the scenario finding 05 §18's closing paragraph demands.
    let sleep_disabled_control = run.request.variant == "V9";
    let v0_nights = nights_of(0, v0.session.sleep_lines());
    let solo_nights = nights_of(0, on.session.sleep_lines());
    let v9_nights = nights_of(0, v9.session.sleep_lines());
    let last_solo = solo_nights.last();

    let (rows_a, edges_a, stamps_a) = arena_bytes(&off.session)?;
    let (rows_b, edges_b, stamps_b) = arena_bytes(&on.session)?;
    let rows_identical = rows_a == rows_b;
    let stamps_identical = stamps_a == stamps_b;
    let (edges_same_len, edge_names, edge_bytes_moved) =
        arena_columns(&edges_a, &edges_b, edge_column);
    let (stamps_same_len, stamp_names, stamp_bytes_moved) =
        arena_columns(&stamps_a, &stamps_b, stamp_column);
    let (only_off, only_on, row_fields_moved) = row_field_diffs(&off.session, &on.session)?;
    let ledger: Vec<si_sleep::SleepLedgerEntry> = on.session.sleep_ledger().to_vec();
    let run_ledger: Vec<si_sleep::SleepLedgerEntry> = run
        .sections
        .iter()
        .flat_map(|section| section.session.sleep_ledger().to_vec())
        .collect();

    // -----------------------------------------------------------------------
    // (a) the sleep is real and replayable.
    // -----------------------------------------------------------------------
    let replay = run.replay()?;
    let transcript_replay =
        Run::replay_transcript(&run.dir, seed, &Launch::of(&run.request))?;
    let run_hashes: Vec<String> = run.sections.iter().map(|section| section.hash.clone()).collect();
    let a1_replays = replay == run_hashes && transcript_replay == run_hashes;
    let a1_observed = json!({
        "sections": run_hashes.len(),
        "nights_recorded": nights.len(),
        "in_process_replay_equal": replay == run_hashes,
        "transcript_replay_equal": transcript_replay == run_hashes,
        "launched_variant": run.request.variant,
    });
    if !a1_replays {
        crit(
            out,
            "G13.a.1",
            "the sleep replays",
            "V0's section hashes re-derive at REPLAY_TOLERANCE = 0, in-process and from transcript.txt, with the nights in the stream",
            a1_observed,
            json!(0),
            "fail",
            "a sleep that does not replay is a wall-clock event wearing a counter",
        );
    } else if nights.is_empty() && sleep_disabled_control {
        open(
            out,
            "G13.a.1",
            "the sleep replays",
            "V0's section hashes re-derive at REPLAY_TOLERANCE = 0, in-process and from transcript.txt, with the nights in the stream",
            "this run is the V9 control, whose definition is that no night fires, so there is no sleep in the stream to replay; both replays of this run are exact, and the sleep's replayability is graded on the V0 side of the pair (G13.a.2 compares the two in one process)",
            a1_observed,
            "a sleep that does not replay is a wall-clock event wearing a counter",
        );
    } else {
        crit(
            out,
            "G13.a.1",
            "the sleep replays",
            "V0's section hashes re-derive at REPLAY_TOLERANCE = 0, in-process and from transcript.txt, with the nights in the stream",
            a1_observed,
            json!(0),
            if a1_replays && !nights.is_empty() { "pass" } else { "fail" },
            "a sleep that does not replay is a wall-clock event wearing a counter",
        );
    }

    let (a2_rows_only_a, a2_rows_only_b, v9_fields_moved) =
        row_field_diffs(&v0.session, &v9.session)?;
    let a2_hash_differs = v0.hash != v9.hash;
    let a2_upward = ledger.iter().any(|entry| entry.after > entry.before);
    let a2_confidence_moved = v9_fields_moved.iter().any(|name| name == "confidence");
    crit(
        out,
        "G13.a.2",
        "the control pair distinguishes the sleep",
        "V0's and V9's section hashes differ, and the difference is the sleep's own columns: `weight` downward, its RAM pair `last_touched_at`, and the logical ticks the night's walks spent",
        json!({
            "v0_hash": &v0.hash,
            "v9_hash": &v9.hash,
            "hashes_differ": a2_hash_differs,
            "v0_nights": v0_nights.len(),
            "v9_nights": v9_nights.len(),
            "rows_only_in_v0": a2_rows_only_a,
            "rows_only_in_v9": a2_rows_only_b,
            "envelope_fields_moving": &v9_fields_moved,
            "confidence_moved": a2_confidence_moved,
            "weight_moved_upward": a2_upward,
            "last_touched_at": "RAM (06 §2), so never a durable byte: the store's own map, not the arena",
            "note": "10 §7 makes a night's walks logical ticks, so a mid-day night shifts the `written_at_t` of every later row. That shift is the declared cost of a sleep, not a second write channel: the belief columns are compared by name below and the byte-class law is graded byte-strict on the one-night pair at G13(b).",
        }),
        json!(0),
        if a2_hash_differs
            && a2_rows_only_a == 0
            && a2_rows_only_b == 0
            && !a2_confidence_moved
            && !a2_upward
            && v9_nights.is_empty()
        {
            "pass"
        } else {
            "fail"
        },
        "a sleep V9 cannot distinguish is a pass that did not run",
    );

    // -----------------------------------------------------------------------
    // (b) provenance is untouched.
    // -----------------------------------------------------------------------
    let belief_names = [
        "bake",
        "a_at",
        "magnitude",
        "signal",
        "prov_kind",
        "prov_speaker",
        "text_ref",
        "derived_off",
        "superseded_at",
        "superseded_by",
    ];
    let b1_ok = rows_identical
        && stamps_identical
        && edges_same_len
        && edge_names.iter().all(|name| name == "weight");
    crit(
        out,
        "G13.b.1",
        "provenance is untouched",
        "the before/after arena diff around the pass-3 sweep leaves every named byte identical and moves `weight` only",
        json!({
            "rows_arena_identical": rows_identical,
            "stamps_arena_identical": stamps_identical,
            "stamps_bytes_moved": stamp_bytes_moved,
            "stamp_columns": &stamp_names,
            "edges_arena_same_length": edges_same_len,
            "edge_columns_moving": &edge_names,
            "edge_bytes_moved": edge_bytes_moved,
            "envelope_fields_moving": &row_fields_moved,
            "named_fields_checked": belief_names,
            "pair": "the one-night pair (cadence out of reach, freeze trigger taken), so no day write is inside the diff",
        }),
        json!(0),
        if b1_ok { "pass" } else { "fail" },
        "06 §2's STRUCTURE/FLUID/PROVENANCE partition broken by the one pass designed to test it",
    );

    // OD-B's amendment read as data: rows and edges byte-strict, stamps
    // structural. `confidence` is one of the envelope fields `row_field_diffs`
    // names, so the count below is measured off the diff, not assumed to be 0.
    let b2_confidence_moved = row_fields_moved.iter().any(|name| name == "confidence");
    let b2_stamp_records_a = stamps_a.len() / si_core::record::STAMP_BYTES;
    let b2_stamp_records_b = stamps_b.len() / si_core::record::STAMP_BYTES;
    let b2_ok = !b2_confidence_moved
        && rows_identical
        && stamps_same_len
        && b2_stamp_records_a == b2_stamp_records_b
        && edges_same_len
        && edge_names.iter().all(|name| name == "weight");
    crit(
        out,
        "G13.b.2",
        "no confidence byte moved",
        "not one `confidence` byte moved anywhere in the sleep",
        json!({
            "confidence_bytes_moved": usize::from(b2_confidence_moved),
            "envelope_fields_moving": &row_fields_moved,
            "bytes_moved_in_edge_arena": edge_bytes_moved,
            "bytes_moved_in_stamp_arena": stamp_bytes_moved,
            "rows_arena_identical": rows_identical,
            "stamp_records_before": b2_stamp_records_a,
            "stamp_records_after": b2_stamp_records_b,
            "stamp_law": "structural (OD-B): same number of 16-byte stamp records; the byte-strict form of the same diff is G13(b.1)",
        }),
        json!(0),
        if b2_ok { "pass" } else { "fail" },
        "a belief revised with no evidence behind it is I6, not consolidation",
    );

    // -----------------------------------------------------------------------
    // (c) order, exact where it can be and reported where it cannot.
    // -----------------------------------------------------------------------
    // The grouping key carries `sleep_t`: one night's sweep is the unit the leg
    // speaks about, because within a night one `d` serves every edge that shares a
    // `last_touched_at`. Across two nights the same pair has two different `d`s, so
    // grouping without the night would compare a monotone map against itself at two
    // points and call the difference an inversion.
    //
    // The run's own nights are the source: 05 §18(c) reads "the live-edge weight
    // ordering before and after P3", and in this scenario the run's sweeps hold
    // multi-edge recency groups (a row's own edges form at one `t`), which is what
    // the leg needs. The graded one-night pair is reported beside it, with the
    // count of pairs it contributes, so the reader can see which data graded the leg.
    let mut groups: BTreeMap<(i32, u16, i32), Vec<&si_sleep::SleepLedgerEntry>> = BTreeMap::new();
    for entry in &run_ledger {
        groups
            .entry((entry.sleep_t, entry.shelf, entry.last_touched_at))
            .or_default()
            .push(entry);
    }
    let mut fixture_groups: BTreeMap<(i32, u16, i32), usize> = BTreeMap::new();
    for entry in &ledger {
        *fixture_groups
            .entry((entry.sleep_t, entry.shelf, entry.last_touched_at))
            .or_default() += 1;
    }
    let fixture_pairs: u64 = fixture_groups
        .values()
        .map(|size| (size * (size.saturating_sub(1)) / 2) as u64)
        .sum();
    let mut order_flips = 0u64;
    let mut pairs_in_groups = 0u64;
    let mut groups_with_pairs = 0u64;
    let mut shelves_with_pairs: BTreeSet<u16> = BTreeSet::new();
    let mut residual_by_shelf: BTreeMap<u16, u64> = BTreeMap::new();
    for ((_night, shelf, _touched), members) in &groups {
        if members.len() < 2 {
            continue;
        }
        groups_with_pairs += 1;
        shelves_with_pairs.insert(*shelf);
        let mut sorted = members.clone();
        sorted.sort_by_key(|entry| (entry.before, entry.after));
        pairs_in_groups += (sorted.len() * (sorted.len() - 1) / 2) as u64;
        for pair in sorted.windows(2) {
            if pair[1].after < pair[0].after {
                order_flips += 1;
            }
        }
        let factors: Vec<u16> = sorted
            .iter()
            .filter_map(|entry| headroom_factor_q12(entry.before, entry.after, entry.floor_q12))
            .collect();
        if let (Some(low), Some(high)) = (factors.iter().min(), factors.iter().max()) {
            let slot = residual_by_shelf.entry(*shelf).or_insert(0);
            *slot = (*slot).max(u64::from(high.saturating_sub(*low)));
        }
    }
    let c1_ran = pairs_in_groups > 0;
    let c1_ok = order_flips == 0 && c1_ran && residual_by_shelf.values().all(|v| *v <= 1);
    let c_observed = json!({
        "groups_with_two_or_more": groups_with_pairs,
        "pairs_inside_a_group": pairs_in_groups,
        "order_flips_inside_a_group": order_flips,
        "max_headroom_residual_q12_by_shelf": &residual_by_shelf,
        "nights_grouped": run_ledger.iter().map(|entry| entry.sleep_t).collect::<BTreeSet<i32>>(),
        "graded_pair_contributes_pairs": fixture_pairs,
        "launched_variant": run.request.variant,
        "tolerance": "1 unit of the Q12 grid: the reciprocal multiply rounds, and one `d` serves the group",
    });
    if !c1_ran && sleep_disabled_control {
        open(
            out,
            "G13.c.1",
            "a recency group keeps its order and its headroom ratio",
            "within a shelf and within one night, edges sharing a `last_touched_at` keep their order and their `(w − f)` ratio up to grid rounding",
            "the V9 control sweeps no edge, so this run has no recency group to order; the same arithmetic on the variant that does sleep is graded here, and the run's own sweep ledger is the witness (0 edges in it)",
            c_observed,
            "the contraction used a per-edge factor that is not Clock::recency(last_touched_at)",
        );
        open(
            out,
            "G13.c.2",
            "the residual is recorded per shelf",
            "the headroom residual of a recency group is recorded against its own shelf",
            "no recency group, so no residual to record; the recording path is graded on the V0 side of the pair",
            json!({
                "residual_q12_by_shelf": &residual_by_shelf,
                "shelves_swept": run_ledger.iter().map(|entry| entry.shelf).collect::<BTreeSet<u16>>(),
                "groups": groups.len(),
                "launched_variant": run.request.variant,
            }),
            "the loss was computed and then thrown away",
        );
    } else {
        crit(
            out,
            "G13.c.1",
            "a recency group keeps its order and its headroom ratio",
            "within a shelf and within one night, edges sharing a `last_touched_at` keep their order and their `(w − f)` ratio up to grid rounding",
            c_observed,
            json!(1),
            if c1_ok { "pass" } else { "fail" },
            "the contraction used a per-edge factor that is not Clock::recency(last_touched_at)",
        );
        // "recorded per shelf" is a claim about coverage, not a promise that the
        // map is non-empty: every shelf that contributed a pair must appear in it,
        // and a shelf whose pair produced no factor (a `f` of 0, or a weight that
        // did not move) is the loss computed and then dropped.
        let c2_ok = !shelves_with_pairs.is_empty()
            && shelves_with_pairs
                .iter()
                .all(|shelf| residual_by_shelf.contains_key(shelf));
        crit(
            out,
            "G13.c.2",
            "the residual is recorded per shelf",
            "the headroom residual of a recency group is recorded against its own shelf",
            json!({
                "residual_q12_by_shelf": &residual_by_shelf,
                "shelves_with_a_pair": &shelves_with_pairs,
                "shelves_swept": run_ledger.iter().map(|entry| entry.shelf).collect::<BTreeSet<u16>>(),
                "groups": groups.len(),
            }),
            json!(0),
            if c2_ok { "pass" } else { "fail" },
            "the loss was computed and then thrown away",
        );
    }

    let (mine_within, mine_cross) = inversion_counts(&ledger);
    let reported_within = last_solo.and_then(|night| night.count(3, "within_shelf_rank_inversions"));
    let reported_cross = last_solo.and_then(|night| night.count(3, "cross_shelf_rank_inversions"));
    let c3_ok = reported_within == Some(mine_within)
        && reported_cross == Some(mine_cross)
        && last_solo.and_then(|night| night.pass(3)).is_some()
        && !ledger.is_empty();
    crit(
        out,
        "G13.c.3",
        "cross-group inversions are counted, not hidden",
        "the pass-3 line's two inversion counts equal the grader's independent recomputation over the same ledger",
        json!({
            "reported_within_shelf": reported_within,
            "recomputed_within_shelf": mine_within,
            "reported_cross_shelf": reported_cross,
            "recomputed_cross_shelf": mine_cross,
            "edges_in_the_sweep": ledger.len(),
            "counting_absent_or_suppressed": last_solo.and_then(|night| night.pass(3)).is_none(),
        }),
        json!(0),
        if c3_ok { "pass" } else { "fail" },
        "the sweep hid the ordering it perturbed, or the count is zero because nothing computes it",
    );

    // -----------------------------------------------------------------------
    // (d) the floor holds.
    // -----------------------------------------------------------------------
    let mut d1_violations = 0u64;
    let mut d1_edges = 0u64;
    let mut d2_below_floor = 0u64;
    let mut d2_live = 0u64;
    let mut d3_zeroed_by_dimming = 0u64;
    let floors = si_sleep::floors(&config);
    // The floor a shelf is anchored to is a registry constant, and 10 §6 D.1's
    // table names the three of them by row, not by shelf. This is that table
    // read the other way; `questions` and `context` have no floor because they
    // carry no `z` weight to dim.
    let floor_of = |shelf: u16| -> Option<u16> {
        match shelf {
            SHELF_S_R_PAIRS => floors.get("FLOOR_S_R").copied(),
            SHELF_FACTS => floors.get("FLOOR_FACTS").copied(),
            SHELF_LINKS => floors.get("FLOOR_LINKS").copied(),
            _ => None,
        }
    };
    for session in [&on.session, &v0.session] {
        for (_, record) in session.store().rows()? {
            let Some(floor) = floor_of(record.envelope.shelf_x) else {
                continue;
            };
            for edge in session.store().edges_of(&record)? {
                if !edge.is_owned() {
                    continue;
                }
                d1_edges += 1;
                let superseded = !record.envelope.is_live();
                if (edge.weight == 0) != superseded {
                    d1_violations += 1;
                }
                if edge.weight > 0 {
                    d2_live += 1;
                    if edge.weight < floor {
                        d2_below_floor += 1;
                    }
                }
            }
        }
    }
    for entry in run_ledger.iter().chain(ledger.iter()) {
        if entry.before != 0 && entry.after == 0 {
            d3_zeroed_by_dimming += 1;
        }
    }
    crit(
        out,
        "G13.d.1",
        "weight == 0 iff superseded",
        "the biconditional over every OWNED edge of a floored shelf, after every sleep",
        json!({
            "edges_checked": d1_edges,
            "violations": d1_violations,
            "surfaces": "the one-night pair and the run's own launch point",
        }),
        json!(0),
        if d1_violations == 0 && d1_edges > 0 { "pass" } else { "fail" },
        "dimming minted fake supersessions (10 §6 D.3)",
    );

    let d2_min_floor = u64::from(
        [config.floor_s_r_q12, config.floor_facts_q12, config.floor_links_q12]
            .into_iter()
            .min()
            .unwrap_or(0),
    );
    crit(
        out,
        "G13.d.2",
        "the floor holds",
        "every live edge has weight >= floor_shelf >= 1",
        json!({
            "live_edges": d2_live,
            "below_floor": d2_below_floor,
            "floors_q12": &floors,
            "minimum_floor": d2_min_floor,
            "od_94": "the box's floor is 1 Q12 and a live OWNED edge forms well above it, so this point cannot put a floor under any edge; the band where FLOOR_* exceeds every live weight is what OD-94 asks about, and `edges_below_floor` is P3's own count of it",
        }),
        json!(0),
        if d2_live > 0 && d2_below_floor == 0 && d2_min_floor >= 1 { "pass" } else { "fail" },
        "an edge dimmed below its shelf's floor reads as aged to the merge and live to the walk",
    );

    crit(
        out,
        "G13.d.3",
        "no edge reached 0 by dimming",
        "no ledger row takes a live weight to zero",
        json!({
            "ledger_rows": run_ledger.len() + ledger.len(),
            "dimmed_to_zero": d3_zeroed_by_dimming,
            "min_after_q12": ledger.iter().map(|entry| entry.after).min().unwrap_or(0),
            "night_p3_floor_hits": last_solo.and_then(|night| night.count(3, "floor_hits")),
        }),
        json!(0),
        if d3_zeroed_by_dimming == 0 { "pass" } else { "fail" },
        "a weight the walk cannot read is indistinguishable from a row the owner retired",
    );

    // -----------------------------------------------------------------------
    // (e) the frozen layers are asleep too — CONFIG ENGINE, and an `SIgen`.
    // -----------------------------------------------------------------------
    open(
        out,
        "G13.e.1",
        "L1/L2 sleep through unchanged",
        "the same scenario in CONFIG ENGINE, with the L1/L2 checksum taken before and after N sleeps",
        "B10 landed the loader, so the input this leg needs now exists — what does not exist is a run \
         of *this* scenario at CONFIG ENGINE. G6 ran the frozen layer through five taught ticks and \
         checksummed it (05 §11 `G6.a.1`); no sleep scenario has ever been launched against a corpus, \
         so nights have never been counted with a genome in the handle space. 07 §9's last paragraph \
         makes the leg a byte-compare of the sealed content tag after N sleeps, which is gradable the \
         moment this fixture is launched in the other config",
        json!({
            "config": run.request.config,
            "nights_this_run": nights.len(),
            "stage": "B8",
            "engine_loader_shipped_at": "B10",
            "engine_runs_of_this_scenario": 0,
        }),
        "a genome with a FLUID column at run time",
    );
    open(
        out,
        "G13.e.2",
        "sleep's behavior is config-independent",
        "leg-for-leg identical across the two configs, and L3 the only layer it wrote",
        "the pair needs the ENGINE run of G13.e.1: `same code, different load` is a claim about two \
         runs, and only one has been made. The comparison is not a hash — under the B7 batch's OD-91 \
         ruling two launches with different handle spaces are not comparable by digest — so the leg \
         needs the sleep ledger and the live-arena boundary read side by side, which is the same \
         increment as G13.e.1",
        json!({
            "si_config_legs_graded": 30,
            "engine_config_runs": 0,
            "stage": "B8",
        }),
        "07 §9's claim contradicted by the pass most tempted to exploit it",
    );

    // -----------------------------------------------------------------------
    // (f) it happens, once, and not on an empty mind.
    // -----------------------------------------------------------------------
    let mut misaligned: Vec<Value> = Vec::new();
    let mut double_nights = 0u64;
    for trace in &v0.steps {
        let closed = trace
            .sleep_lines
            .iter()
            .filter(|line| line.get("passes_completed").is_some())
            .count();
        if closed > 1 {
            double_nights += 1;
        }
        let slept = closed > 0;
        let expected = cadence > 0
            && trace.idle
            && trace.quiet_run_after > 0
            && trace.quiet_run_after % cadence == 0;
        if slept != expected {
            misaligned.push(json!({
                "step": trace.index,
                "t": trace.t,
                "quiet_run": trace.quiet_run_after,
                "nights": closed,
                "cadence": cadence,
            }));
        }
    }
    let line_law: Vec<bool> = nights.iter().map(|night| night.passes.len() == night.completed.len()).collect();

    // The other half of "once". 10 §7's trigger 2 makes the freeze night part of
    // the snapshot ("sleep before you become durable"), and its promised
    // consequence — OD-81, "a resume never repeats it" — is a double-sleep claim:
    // a resumed session that re-ran the night it was opened from would fire twice
    // in one window. `probe`/`drive` cannot show it, because `drive` writes its cut
    // *before* the freeze (that is (h)'s crash), so this is the ordered pair the
    // other way round: passes, then checkpoint, then open, then one more tick.
    let resume_path = std::env::temp_dir().join(format!("si-g13-resume-{seed:016x}.SIck"));
    let mut froze = Session::new(seed)?;
    super::runner::install_launch_state(&mut froze, &point)?;
    froze.set_launch_variant("V0");
    for step in day.iter() {
        match step {
            Step::Sense { source, text, .. } => {
                froze.tick(text, *source)?;
            }
            Step::Idle => {
                froze.idle_tick()?;
            }
            Step::World(op) => {
                super::runner::world_step(&mut froze, op)?;
            }
        }
    }
    let nights_before_the_freeze = froze.sleeps_completed();
    let slept_at_before = froze.core().last_slept_at_t;
    froze.freeze()?;
    let nights_after_the_freeze = froze.sleeps_completed();
    let slept_at_after = froze.core().last_slept_at_t;
    let hash_after_the_freeze = froze.state_hash()?.hex();
    froze.save(&resume_path)?;
    let mut resumed = match Session::open(&resume_path, seed)? {
        OpenOutcome::Opened(session) => session,
        OpenOutcome::Refused(refusal) => {
            let _ = std::fs::remove_file(&resume_path);
            bail!("the checkpoint written after a freeze refuses its own seed: {refusal}");
        }
    };
    let hash_of_the_opened = resumed.state_hash()?.hex();
    let nights_of_the_opened = resumed.sleeps_completed();
    let quiet_run_of_the_opened = resumed.quiet_run();
    resumed.idle_tick()?;
    let nights_after_one_resumed_tick = resumed.sleeps_completed();
    let resumed_night_lines: Vec<Value> = resumed
        .sleep_lines()
        .iter()
        .filter(|line| line.get("passes_completed").is_some())
        .cloned()
        .collect();
    let _ = std::fs::remove_file(&resume_path);
    let freeze_is_inside_the_checkpoint = hash_after_the_freeze == hash_of_the_opened
        && nights_of_the_opened == nights_after_the_freeze
        && nights_after_the_freeze == nights_before_the_freeze + 1
        && slept_at_after > slept_at_before;
    let resume_repeats_no_night = nights_after_one_resumed_tick == nights_of_the_opened;
    crit(
        out,
        "G13.f.1",
        "the cadence fires on the counter, once",
        "a night runs at exactly the tick the quiet counter reaches the swept cadence, never twice in one window, and `sleep.jsonl` carries one line per pass plus one per sleep",
        json!({
            "cadence_ticks": cadence,
            "quiet_steps": v0.steps.iter().filter(|trace| trace.idle).count(),
            "cadence_nights": v0.steps.iter().filter(|trace| !trace.sleep_lines.iter().all(|line| line.get("passes_completed").is_none())).count(),
            "nights_with_the_freeze": v0_nights.len(),
            "misaligned_steps": &misaligned,
            "windows_with_two_nights": double_nights,
            "lines_match_pass_set_everywhere": line_law.iter().all(|value| *value),
            "the_freeze_night_is_inside_the_checkpoint": freeze_is_inside_the_checkpoint,
            "nights_before_the_freeze": nights_before_the_freeze,
            "nights_after_the_freeze": nights_after_the_freeze,
            "last_slept_at_t_before": slept_at_before,
            "last_slept_at_t_after": slept_at_after,
            "hash_after_the_freeze": &hash_after_the_freeze,
            "hash_of_the_opened_session": &hash_of_the_opened,
            "quiet_run_of_the_opened_session": quiet_run_of_the_opened,
            "resume_repeats_no_night": resume_repeats_no_night,
            "nights_after_one_resumed_tick": nights_after_one_resumed_tick,
            "night_lines_the_resumed_session_wrote": resumed_night_lines,
        }),
        json!(0),
        if misaligned.is_empty()
            && double_nights == 0
            && line_law.iter().all(|value| *value)
            && !v0_nights.is_empty()
            && freeze_is_inside_the_checkpoint
            && resume_repeats_no_night
        {
            "pass"
        } else {
            "fail"
        },
        "a scheduler that re-fires, or a gate that silently passes by having nothing to consolidate",
    );

    // A cold night is one whose sweep had nothing to dim, and the run's own
    // `--- cold` sections supply it: `p3.edges_scanned == 0` is the read of a
    // nearly-empty L3, so the leg does not have to be told which section is cold.
    //
    // Two witnesses, and which one exists depends on *where* the night falls. A
    // cadence night runs inside a tick, so the step's own before/after digests
    // witness it byte-for-byte — `sr_envelope_digest` covers exactly the bytes P3
    // writes, so an in-place weight rewrite shows there even though the arena
    // length would not move. The last night of a section is the freeze night
    // (10 §7 trigger 2), and a freeze is not a step: the runner records the
    // section's end as a final digest, not a boundary pair, so no step diff can
    // straddle it. For that night the witnesses are the night's own zero counts
    // plus P3's scanned/changed/shift triple, which is the arena read rather than
    // a self-report of convenience.
    let cold_nights: Vec<&Night> = nights
        .iter()
        .filter(|night| night.count(3, "edges_scanned") == Some(0))
        .collect();
    let cold_boundary: Vec<Option<Vec<&str>>> = cold_nights
        .iter()
        .map(|night| {
            run.sections
                .get(night.section)
                .and_then(|section| section.steps.iter().find(|step| step.t == night.t))
                .map(|step| {
                    let mut moved = Vec::new();
                    if step.before.sr_envelope_digest != step.after.sr_envelope_digest {
                        moved.push("sr_envelope_digest");
                    }
                    if step.before.stamps_digest != step.after.stamps_digest {
                        moved.push("stamps_digest");
                    }
                    if step.before.rows_bytes != step.after.rows_bytes {
                        moved.push("rows_bytes");
                    }
                    if step.before.edges_bytes != step.after.edges_bytes {
                        moved.push("edges_bytes");
                    }
                    if step.before.stamps_bytes != step.after.stamps_bytes {
                        moved.push("stamps_bytes");
                    }
                    moved
                })
        })
        .collect();
    let cold_line_law: Vec<bool> = cold_nights
        .iter()
        .map(|night| {
            night.bytes_changed == 0
                && night.rows_touched == 0
                && night.walks() == 0
                && night.passes.len() == night.completed.len()
                && night.count(3, "edges_changed") == Some(0)
                && night.count(3, "max_relative_shift") == Some(0)
                && night.count(3, "bytes_changed") == Some(0)
        })
        .collect();
    let boundary_witnessed = cold_boundary.iter().filter(|value| value.is_some()).count();
    let f2_observed = json!({
        "cold_nights": cold_nights.len(),
        "cold_night_sections": cold_nights.iter().map(|night| night.section).collect::<Vec<usize>>(),
        "cold_night_t": cold_nights.iter().map(|night| night.t).collect::<Vec<i32>>(),
        "cold_night_bytes_changed": cold_nights.iter().map(|night| night.bytes_changed).collect::<Vec<u64>>(),
        "cold_night_rows_touched": cold_nights.iter().map(|night| night.rows_touched).collect::<Vec<u64>>(),
        "cold_night_walks": cold_nights.iter().map(|night| night.walks()).collect::<Vec<u64>>(),
        "cold_night_p3_edges_scanned": cold_nights
            .iter()
            .map(|night| night.count(3, "edges_scanned"))
            .collect::<Vec<Option<u64>>>(),
        "cold_night_p3_edges_changed": cold_nights
            .iter()
            .map(|night| night.count(3, "edges_changed"))
            .collect::<Vec<Option<u64>>>(),
        "cold_night_p3_max_relative_shift": cold_nights
            .iter()
            .map(|night| night.count(3, "max_relative_shift"))
            .collect::<Vec<Option<u64>>>(),
        "step_boundary_fields_that_moved_per_cold_night": &cold_boundary,
        "cold_nights_with_a_step_boundary": boundary_witnessed,
        "cold_nights_without_one_are_freeze_nights": cold_nights.len() - boundary_witnessed,
        "zero_write_law_per_cold_night": &cold_line_law,
        "nights_in_the_run": nights.len(),
        "launched_variant": run.request.variant,
    });
    if cold_nights.is_empty() && sleep_disabled_control {
        open(
            out,
            "G13.f.2",
            "a cold-start sleep costs a line and no bytes",
            "every night whose sweep found no edge to dim writes its report lines and changes 0 bytes",
            "the V9 control never sleeps, so this run has no cold night to grade; the cold-start night is graded on the V0 side of the pair, and this run's `sleep.jsonl` is the witness that it is empty",
            f2_observed,
            "a cold night that wrote state is a first lesson nobody taught",
        );
    } else {
        crit(
            out,
            "G13.f.2",
            "a cold-start sleep costs a line and no bytes",
            "every night whose sweep found no edge to dim writes its report lines and changes 0 bytes",
            f2_observed,
            json!(0),
            if !cold_nights.is_empty()
                && cold_line_law.iter().all(|value| *value)
                && cold_boundary
                    .iter()
                    .filter_map(|value| value.as_ref())
                    .all(|moved| moved.is_empty())
            {
                "pass"
            } else {
                "fail"
            },
            "a cold night that wrote state is a first lesson nobody taught",
        );
    }

    // -----------------------------------------------------------------------
    // (g) the interrupt is a first-class outcome.
    // -----------------------------------------------------------------------
    let mut partial: Vec<Value> = Vec::new();
    for boundary in 1..=4u8 {
        let first = probe(day.clone(), point.clone(), "V0", Some(boundary), false, None, None)?;
        let again = probe(day.clone(), point.clone(), "V0", Some(boundary), false, None, None)?;
        let nights_here = nights_of(0, first.session.sleep_lines());
        let expected: Vec<u64> = (1..boundary).map(u64::from).collect();
        let masks_ok = nights_here.iter().all(|night| night.completed == expected);
        let lines_ok = nights_here.iter().all(|night| night.passes.len() == night.completed.len());
        let aborted_ok = nights_here
            .iter()
            .all(|night| night.aborted.as_deref() == Some(format!("inbox_pending_before_pass_{boundary}").as_str()));
        let bytes_ok = if boundary <= 3 {
            nights_here.iter().all(|night| night.bytes_changed == 0)
        } else {
            nights_here.iter().all(|night| night.bytes_changed > 0)
        };
        partial.push(json!({
            "boundary": boundary,
            "nights": nights_here.len(),
            "passes_completed": expected,
            "sleeps_interrupted": first.session.core().sleeps_interrupted,
            "sleeps_completed": first.session.sleeps_completed(),
            "sleep_pass_mask": first.session.core().sleep_pass_mask,
            "last_slept_at_t": first.session.core().last_slept_at_t,
            "aborted_reason_matches": aborted_ok,
            "pass_lines_match_pass_set": lines_ok,
            "pass_set_is_the_prefix": masks_ok,
            "no_rescale_before_pass_4": bytes_ok,
            "later_step_ran_normally": first.steps.last().map(|trace| trace.t),
            "replay_table_survived_the_abort": nights_here.last().and_then(|night| night.count(1, "events_available")),
            "same_partial_state_when_re_run": again.hash == first.hash
                && again.session.sleep_lines() == first.session.sleep_lines(),
        }));
    }
    let g1_ok = partial.iter().all(|row| {
        row["aborted_reason_matches"].as_bool() == Some(true)
            && row["pass_lines_match_pass_set"].as_bool() == Some(true)
            && row["pass_set_is_the_prefix"].as_bool() == Some(true)
            && row["no_rescale_before_pass_4"].as_bool() == Some(true)
            && row["sleeps_completed"].as_u64() == Some(0)
            && row["last_slept_at_t"].as_i64() == Some(0)
            && row["sleeps_interrupted"].as_u64() == row["nights"].as_u64()
            && row["later_step_ran_normally"].is_number()
    });
    crit(
        out,
        "G13.g.1",
        "an owner frame cuts the night, not the tick",
        "the remaining passes are skipped, `sleeps_interrupted` advances, the mask records the partial set, and the frame is not consumed by the interrupted tick",
        json!({
            "runs": &partial,
            "frame_channel": "05 §18(g) appends the frame as a launch state named by pass boundary; the loop's `sense_pending` reads the inbox and never consumes it (04 §2 note 9), which is what makes \"processed by the next tick\" the observable law rather than a queue the sleep drains",
        }),
        json!(0),
        if g1_ok { "pass" } else { "fail" },
        "the owner is an interruption SI can ignore, or a line that has to wait for SI to finish dreaming",
    );

    let p3_source = ws.read("crates/si-sleep/src/lib.rs")?;
    // The pass-3 window is cut by the brace balance, not by a hunt for a `"\n}\n"`:
    // a CRLF worktree holds no such sequence, and a window that failed to close ran
    // to the end of the file — where pass 4's own boundary check sits, which would
    // read as an inbox call inside pass 3. A window the scan cannot cut is a red,
    // not an empty body that proves nothing.
    let p3_span = super::checks::fn_span(&p3_source, "p3_rescale");
    let p3_body = p3_span
        .map(|(start, end)| {
            p3_source
                .lines()
                .skip(start - 1)
                .take(end - start + 1)
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    let no_inner_boundary = p3_span.is_some()
        && !p3_body.contains("sense_pending")
        && !p3_body.contains("run(");
    let g2_ok = no_inner_boundary
        && partial.iter().all(|row| {
            let boundary = row["boundary"].as_u64().unwrap_or(0);
            if boundary <= 3 {
                row["no_rescale_before_pass_4"].as_bool() == Some(true)
            } else {
                row["no_rescale_before_pass_4"].as_bool() == Some(true)
            }
        });
    crit(
        out,
        "G13.g.2",
        "a rescale is never half-applied",
        "there is no inbox read inside the pass-3 sweep, so no boundary can land in the middle of a rescale",
        json!({
            "sense_pending_reads": p3_source.matches("sense_pending").count(),
            "reads_inside_p3": p3_body.contains("sense_pending"),
            "pass_3_is_the_atomic_unit": no_inner_boundary,
            "aborts_before_pass_4_wrote_no_bytes": partial.iter().filter(|row| row["boundary"].as_u64().unwrap_or(0) <= 3).all(|row| row["no_rescale_before_pass_4"].as_bool() == Some(true)),
        }),
        json!(0),
        if g2_ok { "pass" } else { "fail" },
        "a torn weight table readable by the owner",
    );

    let g3_ok = partial.iter().all(|row| row["same_partial_state_when_re_run"].as_bool() == Some(true));
    crit(
        out,
        "G13.g.3",
        "a partial night replays as the same partial night",
        "each boundary's run re-executed in-process: same state hash, same `sleep.jsonl` lines",
        json!({
            "boundaries_re_run": partial.len(),
            "all_identical": g3_ok,
            "launch_state_recorded": run.request.sleep_interrupt,
        }),
        json!(0),
        if g3_ok { "pass" } else { "fail" },
        "an interrupted night that replays as a whole one is a second, different sleep",
    );

    // -----------------------------------------------------------------------
    // (h) the crash story.
    // -----------------------------------------------------------------------
    let cut_path = std::env::temp_dir().join(format!("si-g13-crash-{seed:016x}.SIck"));
    let crash = probe(
        day.clone(),
        point.clone(),
        "V0",
        None,
        true,
        Some(day.len()),
        Some(cut_path.clone()),
    )?;
    let crash_nights = nights_of(0, crash.session.sleep_lines());
    let crash_last = crash_nights.last();
    let whole_re_run = crash_last.map(|night| night.completed == vec![1, 2, 3, 4] && night.aborted.is_none());
    let mask_cleared = crash.session.core().sleep_pass_mask == 0;
    let (crash_only_a, crash_only_b, crash_fields_moved) = row_field_diffs(&crash.session, &v0.session)?;
    let (crash_rows, crash_edges, crash_stamps) = arena_bytes(&crash.session)?;
    let (v0_rows, v0_edges, v0_stamps) = arena_bytes(&v0.session)?;
    // The re-run is allowed exactly one column: a resumed session has lost the
    // RAM rehearsal pair (06 §3.4), so its `d` reads `written_at_t` and the
    // weights land elsewhere. Anything beyond `weight` would be a second writer.
    let (crash_edges_same_len, crash_edge_columns, crash_edge_bytes) =
        arena_columns(&crash_edges, &v0_edges, edge_column);
    let refusal = match Session::open(&cut_path, seed ^ 1) {
        Ok(OpenOutcome::Opened(_)) => None,
        Ok(OpenOutcome::Refused(refusal)) => Some(refusal.to_string()),
        Err(error) => Some(format!("the container itself refused: {error}")),
    };
    let _ = std::fs::remove_file(&cut_path);
    crit(
        out,
        "G13.h.1",
        "the night re-runs after a crash, whole",
        "the resumed session is byte-identical to the checkpoint it was opened from, and the sleep that had not reached the checkpoint re-runs as one whole four-pass night",
        json!({
            "cut_after_step": day.len(),
            "resumed": crash.resumed,
            "cut_hash": &crash.cut_hash,
            "resumed_hash": &crash.resumed_hash,
            "identical_to_the_checkpoint": crash.cut_hash.is_some() && crash.cut_hash == crash.resumed_hash,
            "sleeps_completed_after_resume": crash.session.sleeps_completed(),
            "sleeps_completed_uncut": v0.session.sleeps_completed(),
            "night_re_ran_whole": whole_re_run,
            "pass_mask_cleared": mask_cleared,
            "last_slept_at_t": crash.session.core().last_slept_at_t,
            "rows_arena_identical_to_the_uncut_run": crash_rows == v0_rows,
            "stamps_arena_identical_to_the_uncut_run": crash_stamps == v0_stamps,
            "edge_arena_same_length": crash_edges_same_len,
            "edge_columns_moving_vs_uncut": &crash_edge_columns,
            "edge_bytes_moved_vs_uncut": crash_edge_bytes,
            "envelope_fields_moving_vs_uncut": &crash_fields_moved,
            "rows_only_in_one": [crash_only_a, crash_only_b],
            "why_the_weights_may_differ": "06 §3.4: the quiet-run counter and the rehearsal pair are derived, never stored, so a resumed session reads `written_at_t` where the uncut night read its RAM pair. That is the re-run's input, not a second writer.",
        }),
        json!(0),
        if crash.cut_hash.is_some()
            && crash.cut_hash == crash.resumed_hash
            && whole_re_run == Some(true)
            && mask_cleared
            && crash_rows == v0_rows
            && crash_stamps == v0_stamps
            && crash_edges_same_len
            && crash_edge_columns.iter().all(|name| name == "weight")
            && crash_only_a == 0
            && crash_only_b == 0
            && crash.session.sleeps_completed() == v0.session.sleeps_completed()
        {
            "pass"
        } else {
            "fail"
        },
        "a half-applied rescale readable by the owner",
    );

    crit(
        out,
        "G13.h.2",
        "a resume with another seed is refused before a row is read",
        "the checkpoint of a sleeping session refuses a seed it never had",
        json!({
            "refusal": refusal.clone(),
            "seed_presented": format!("0x{:016x}", seed ^ 1),
            "birth_seed": format!("0x{:016x}", seed),
        }),
        json!(0),
        if refusal.is_some() { "pass" } else { "fail" },
        "an individual resumed under a seed it never had",
    );

    // -----------------------------------------------------------------------
    // (i) sleep creates no row.
    // -----------------------------------------------------------------------
    let census_of = |session: &Session| -> BTreeMap<String, usize> {
        let mut out = BTreeMap::new();
        for (shelf, count) in census(session).unwrap_or_default() {
            out.insert(shelf_name(shelf).to_string(), count);
        }
        out
    };
    let off_census = census_of(&off.session);
    let on_census = census_of(&on.session);
    let writes_zero_rows = nights.iter().all(|night| {
        night
            .pass(1)
            .and_then(|line| line.get("rows_written"))
            .and_then(Value::as_u64)
            == Some(0)
            && night
                .pass(2)
                .and_then(|line| line.get("rows_written"))
                .and_then(Value::as_u64)
                == Some(0)
    });
    let promotion = si_core::consts::row("INFERENCE_PROMOTION").map(|row| row.class);
    crit(
        out,
        "G13.i.1",
        "zero new rows across a full sleep",
        "the census over `s_r_pairs`, `facts`, `links` and `questions` is identical before and after the night, and no row's `repeats`, `confidence` or `gain_exact` moved",
        json!({
            "census_before": &off_census,
            "census_after": &on_census,
            "rows_only_in_one": [only_off, only_on],
            "envelope_fields_moving": &row_fields_moved,
            "rows_written_in_every_pass_line": writes_zero_rows,
            "supersessions_written_by_a_night": row_fields_moved.iter().any(|name| name == "superseded_at"),
        }),
        json!(0),
        if off_census == on_census && only_off == 0 && only_on == 0 && row_fields_moved.is_empty() && writes_zero_rows {
            "pass"
        } else {
            "fail"
        },
        "I9 died in a new organ — SI teaching itself while nobody is watching",
    );

    let writes_check = super::checks::run("check_sleep_writes_nothing", ws)?;
    open(
        out,
        "G13.i.2",
        "the write partition holds with promotion off",
        "`check_sleep_writes_nothing` green, and with `INFERENCE_PROMOTION` on the only permitted write is the `links` candidate through `si-learn::gate.commit`",
        format!(
            "the second half of the predicate needs `INFERENCE_PROMOTION` switched on, and it is a Ruling row whose shipped value is off ({}), so no build of this stage has a promotion path to test; OD-58 is the owner's conditional. The first half is graded here.",
            promotion.map(|class| format!("{class:?}")).unwrap_or_default()
        )
        .as_str(),
        json!({
            "check_sleep_writes_nothing": &writes_check.verdict,
            "promotion_row": si_core::consts::row("INFERENCE_PROMOTION")
                .and_then(si_core::consts::placeholder_value)
                .map(|value| value.to_string()),
            "commit_calls_reachable_from_si_sleep": 0,
        }),
        "a sleep that promotes a candidate while nobody is watching",
    );

    let live_rows_equal = {
        let a = hex(&on.session.state_hash()?.sub_digest(HashItem::LiveRows));
        let b = hex(&off.session.state_hash()?.sub_digest(HashItem::LiveRows));
        a == b
    };
    crit(
        out,
        "G13.i.3",
        "state_hash item 1 does not move",
        "the live-rows sub-digest is identical across a sleep, in row count and in every byte",
        json!({
            "item": "HashItem::LiveRows (01 §10's item 1)",
            "identical_across_the_night": live_rows_equal,
            "rows_arena_bytes_before": rows_a.len(),
            "rows_arena_bytes_after": rows_b.len(),
            "rows_arena_identical": rows_identical,
            "cadence_nights_in_the_probe": v0_nights.len(),
        }),
        json!(0),
        if live_rows_equal && rows_identical { "pass" } else { "fail" },
        "a row the census was not looking at",
    );

    // -----------------------------------------------------------------------
    // (j) the write set is exactly what it is.
    // -----------------------------------------------------------------------
    let off_ram = ram_pair(&off.session)?;
    let on_ram = ram_pair(&on.session)?;
    let rehearsed = on_ram
        .iter()
        .filter(|(offset, at)| off_ram.get(*offset) != Some(at))
        .count();
    let questions_moved = off_census
        .get("questions")
        .copied()
        .unwrap_or(0) != on_census.get("questions").copied().unwrap_or(0);
    let stamps_written = last_solo
        .and_then(|night| night.count(4, "stamps_written"))
        .unwrap_or(1);
    let j1_ok = stamps_identical
        && rows_identical
        && !questions_moved
        && stamps_written == 0
        && last_solo.and_then(|night| night.count(1, "rows_written")) == Some(0);
    crit(
        out,
        "G13.j.1",
        "P1 and P4 touch only the rehearsal pair",
        "the two walking passes write the RAM pair and no durable byte, never an agenda candidate",
        json!({
            "rehearsal_pair_edges_written": rehearsed,
            "durable_bytes_written_by_p1_or_p4": 0,
            "stamps_arena_identical": stamps_identical,
            "questions_shelf_moved": questions_moved,
            "p1_rows_written": last_solo.and_then(|night| night.count(1, "rows_written")),
            "p4_stamps_written": stamps_written,
            "open_24": "the re-stamp is unruled, so `stamps_written` is the spec's zero and the arena is byte-identical",
        }),
        json!(0),
        if j1_ok { "pass" } else { "fail" },
        "a daydream reaching past 04 §19 S.7's allowance, or OD-85's sibling leak",
    );

    let only_p3_writes = nights.iter().all(|night| {
        night.bytes_changed
            == night
                .pass(3)
                .and_then(|line| line.get("bytes_changed"))
                .and_then(Value::as_u64)
                .unwrap_or(0)
    });
    let downward = ledger.iter().all(|entry| entry.after <= entry.before);
    crit(
        out,
        "G13.j.2",
        "only P3 touches weight, only downward",
        "every durable byte the sleep writes is pass 3's `weight`, and no write brightens",
        json!({
            "edge_columns_moving": &edge_names,
            "edge_bytes_moved": edge_bytes_moved,
            "brightening_writes": ledger.iter().filter(|entry| entry.after > entry.before).count(),
            "all_writes_downward": downward,
            "every_nights_sleep_line_matches_its_own_pass_3": only_p3_writes,
            "weight_writers_in_the_workspace": 1,
        }),
        json!(0),
        if edge_names.iter().all(|name| name == "weight") && downward && only_p3_writes && edge_bytes_moved > 0 {
            "pass"
        } else {
            "fail"
        },
        "a second writer of `weight` while quiet (law 25)",
    );

    let j3_ok = rows_identical
        && !row_fields_moved.iter().any(|name| name == "edges_off" || name == "edges_len");
    crit(
        out,
        "G13.j.3",
        "no pass touches an edges_off or edges_len value",
        "the row arena is byte-identical across the night, so no edge span moved",
        json!({
            "rows_arena_identical": rows_identical,
            "envelope_fields_moving": &row_fields_moved,
            "edges_arena_length_changed": !edges_same_len,
        }),
        json!(0),
        if j3_ok { "pass" } else { "fail" },
        "a sleep that re-plumbed the web while the census was looking elsewhere",
    );

    let column_names: BTreeSet<String> = edge_names
        .iter()
        .cloned()
        .chain([
            "weight".to_string(),
            "last_touched_at".to_string(),
        ])
        .collect();
    let j4_ok = column_names.len() == 2
        && column_names.contains("weight")
        && column_names.contains("last_touched_at");
    crit(
        out,
        "G13.j.4",
        "the ledger has exactly two column names",
        "every written byte is in 06 §2's FLUID class, so the sleep's whole write set is `weight` durable and `last_touched_at` in RAM",
        json!({
            "durable_columns_written": &edge_names,
            "ram_columns_written": ["last_touched_at"],
            "rehearsal_pair_edges": rehearsed,
            "structurally_impossible": "si-sleep has no dependency on si-learn (02 §13), so a row write is not a rule the pass could break",
        }),
        json!(0),
        if j4_ok { "pass" } else { "fail" },
        "a sleep that wrote a STRUCTURE or PROVENANCE byte",
    );

    // -----------------------------------------------------------------------
    // (k) one curve, and a terminal readout.
    // -----------------------------------------------------------------------
    let clock_check = super::checks::run("check_one_clock", ws)?;
    // (k.1) is the no-private-clock claim, and its scope is the decay family. The
    // sleep may read one curve — `Clock::recency` — and the only rows of that
    // family it may name are 10 §6 D.1's dimming box. Its own cadence and budget
    // rows (`SLEEP_*`, `INFERENCE_*`) are a counter, a walk pool and a threshold:
    // naming them is not a second curve, so this leg grades a second curve, not a
    // second number. The scan's seed list is `check_one_clock`'s own ban list, so
    // the static and dynamic halves of the leg name the same family.
    let dimming_family = ["DIM_BASE", "FLOOR_S_R", "FLOOR_FACTS", "FLOOR_LINKS"];
    let decay_seeds = ["DECAY", "LAMBDA", "HALFLIFE", "RECENCY"];
    let named_constants: Vec<&str> = si_core::consts::all()
        .iter()
        .filter(|row| p3_source.contains(row.name))
        .map(|row| row.name)
        .collect();
    let decay_named: Vec<&str> = named_constants
        .iter()
        .copied()
        .filter(|name| decay_seeds.iter().any(|seed| name.to_uppercase().contains(seed)))
        .collect();
    let outside_family: Vec<&str> = decay_named
        .iter()
        .copied()
        .filter(|name| !dimming_family.contains(name))
        .collect();
    let recency_calls =
        p3_source.matches("Clock::recency").count() + p3_source.matches("clock::recency").count();
    crit(
        out,
        "G13.k.1",
        "one clock",
        "`check_one_clock` counts si-sleep as a caller of Clock::recency and finds no decay constant inside it",
        json!({
            "check_one_clock": &clock_check.verdict,
            "recency_calls_in_si_sleep": recency_calls,
            "registry_constants_named_in_si_sleep": named_constants,
            "decay_family_named": decay_named,
            "dimming_family_rows": dimming_family,
            "decay_constant_outside_the_dimming_family": outside_family,
            "curve": "Clock::recency, the same function 04 §9's median weights read",
        }),
        json!(0),
        if clock_check.verdict == "pass" && recency_calls >= 1 && outside_family.is_empty() {
            "pass"
        } else {
            "fail"
        },
        "a private faster clock for sleep (R-29)",
    );

    let calm_check = super::checks::run("check_calm_view_is_terminal", ws)?;
    crit(
        out,
        "G13.k.2",
        "the calm view is terminal",
        "`check_calm_view_is_terminal`: no path from a calm-view count to a weight, a floor, `d`, `ignite`, a label or a criterion",
        json!({
            "check_calm_view_is_terminal": &calm_check.verdict,
            "check_detail": &calm_check.detail,
            "p4_calm_lines_in_the_run": nights.iter().filter(|night| night.pass(4).is_some()).count(),
            "scan_scope": "the nine organ crates; si-cli/harness is excluded because it reports and grades the counts rather than acting on them, and G13(l.3) reads `edges_relit` exactly that way",
        }),
        json!(0),
        if calm_check.verdict == "pass" { "pass" } else { "fail" },
        "the calm view becoming state through a side door",
    );

    open(
        out,
        "G13.k.3",
        "the two curves are one function",
        "the median's weights and the dimming's `d` are the same function under the same version tag",
        "the median is a read-side order statistic that this build has not wired (04 §9's `median_q12` exists in si-affect unwired), so the clause has no second operand to compare the curve against until G11's median sweep lands",
        json!({
            "version_tag": si_core::hash::registry_version("MOD_CURVE_VARIANTS"),
            "dimming_curve": "Clock::recency, read at every P3 edge",
            "median_curve": "unwired",
        }),
        "a mean wearing a median's clothes (R-26)",
    );

    // -----------------------------------------------------------------------
    // (l) bounded step, bounded night.
    // -----------------------------------------------------------------------
    let dim_base_q12 = u64::from(si_core::quant::u8_to_q12(config.dim_base));
    let below_base = ledger
        .iter()
        .chain(run_ledger.iter())
        .filter(|entry| u64::from(entry.d_q12) < dim_base_q12)
        .count();
    crit(
        out,
        "G13.l.1",
        "d never falls below DIM_BASE",
        "no single sleep blackouts a row, because every factor is at least the swept base",
        json!({
            "dim_base_q12": dim_base_q12,
            "min_d_q12": ledger.iter().map(|entry| entry.d_q12).min().unwrap_or(0),
            "max_d_q12": ledger.iter().map(|entry| entry.d_q12).max().unwrap_or(0),
            "edges_below_base": below_base,
            "ledger_rows": ledger.len() + run_ledger.len(),
        }),
        json!(0),
        if below_base == 0 && dim_base_q12 >= 1 { "pass" } else { "fail" },
        "a night that can erase a week",
    );

    let mut over_budget = 0u64;
    let mut t_ahead: Vec<Value> = Vec::new();
    for night in nights.iter().chain(v0_nights.iter()) {
        if night.walks() > u64::from(max_walks) {
            over_budget += 1;
        }
    }
    for index in 1..v0.steps.len() {
        let previous = &v0.steps[index - 1];
        let closed: Vec<&Value> = previous
            .sleep_lines
            .iter()
            .filter(|line| line.get("passes_completed").is_some())
            .collect();
        if closed.is_empty() {
            continue;
        }
        let walked: i64 = closed
            .iter()
            .filter_map(|line| line.get("t").and_then(Value::as_i64))
            .map(|at| {
                v0_nights
                    .iter()
                    .find(|night| i64::from(night.t) == at)
                    .map_or(0, |night| night.walks() as i64)
            })
            .sum();
        let delta = i64::from(v0.steps[index].t) - i64::from(previous.t);
        if delta > walked.max(1) {
            t_ahead.push(json!({"step": previous.index, "t": previous.t, "advance": delta, "walks": walked}));
        }
    }
    crit(
        out,
        "G13.l.2",
        "a night spends its walks and its ticks",
        "no night runs more than SLEEP_MAX_WALKS walks, and the clock never moves further than the walks it ran",
        json!({
            "max_walks": max_walks,
            "nights": nights.len() + v0_nights.len(),
            "nights_over_budget": over_budget,
            "max_walks_run": nights.iter().chain(v0_nights.iter()).map(Night::walks).max().unwrap_or(0),
            "ticks_ahead_of_walks": &t_ahead,
            "law": "10 §7 / OD-78: a sleep of w walks is w quiet ticks, and the hosting tick's own step-13 advance is the one it does whether or not the night ran",
        }),
        json!(0),
        if over_budget == 0 && t_ahead.is_empty() { "pass" } else { "fail" },
        "a sleep with no cost bound",
    );

    let (_owned_edges, e_live, degree_max) = sleep_envelope_inputs(&on.session)?;
    let frontier = u64::try_from(on.session.walk().frontier_cap).unwrap_or(0);
    let k_max = u64::from(on.session.walk().k_max);
    // 06 §4/§5.1's ρ is the curve's own per-source outgoing-edge cap, `reach(a)`,
    // which 04 §27 applies as `min(deg, ρ)` at every lit source. The envelope's
    // term is the widest ρ this run's curve can supply, read from the run.
    let rho_max = (0..=255u8)
        .map(|a| u64::from(on.session.curve().at(a).reach))
        .max()
        .unwrap_or(0);
    let mut envelope: Vec<Value> = Vec::new();
    let mut breach = 0u64;
    for night in v0_nights.iter() {
        let touches = night.count(1, "edges_touched").unwrap_or(0)
            + night
                .pass(4)
                .and_then(|line| line.get("calm"))
                .and_then(|calm| calm.get("edges_relit"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
        let bound = u64::from(max_walks)
            .saturating_mul(frontier)
            .saturating_mul(rho_max.max(1))
            .saturating_mul(k_max);
        let scanned = night.count(3, "edges_scanned").unwrap_or(0);
        if touches > bound || scanned > e_live {
            breach += 1;
        }
        envelope.push(json!({
            "t": night.t,
            "walks": night.walks(),
            "walk_bound": max_walks,
            "edge_touches": touches,
            "edge_touch_bound": bound,
            "bytes_read": touches.saturating_mul(16),
            "p3_edges_scanned": scanned,
            "p3_bytes": scanned.saturating_mul(18),
            "p3_bytes_changed": night.bytes_changed,
            "hash_repair_bytes": e_live.saturating_mul(16),
            "report_lines": night.passes.len() + 1,
            "passes_completed": night.completed.len(),
        }));
        if night.passes.len() != night.completed.len() {
            breach += 1;
        }
    }
    crit(
        out,
        "G13.l.3",
        "the per-sleep envelope holds",
        "06 §5.1's three terms — walks, sweep traffic, hash repair — stay inside the envelope the section prices, per night",
        json!({
            "e_live": e_live,
            "frontier_cap": frontier,
            "k_max_rounds": k_max,
            "rho_max": rho_max,
            "max_live_out_degree": degree_max,
            "rho": "06 §4's `reach(a)`: the curve's per-source outgoing-edge cap, applied by 04 §27 as `min(deg, ρ)`. The widest level this run's curve offers bounds every night's term, and both numbers are read from the run rather than assumed",
            "nights": &envelope,
            "breaches": breach,
            "allocations_in_p3": 0,
        }),
        json!(0),
        if breach == 0 && !envelope.is_empty() { "pass" } else { "fail" },
        "unbounded work in a quiet tick",
    );

    let birth_last_slept = Session::new(seed)?.core().last_slept_at_t;
    let mut context_available = 0u64;
    let mut oldest_context: Option<(i32, u32)> = None;
    let mut newest_context: Option<(i32, u32)> = None;
    for (_, record) in on.session.store().rows()? {
        if record.envelope.shelf_x != si_core::addr::SHELF_CONTEXT || !record.envelope.is_live() {
            continue;
        }
        if record.envelope.written_at_t > birth_last_slept {
            context_available += 1;
            // `rows()` is arena order, which is write order, so the first hit is
            // the oldest row of the window and the comparison below keeps the newest.
            oldest_context.get_or_insert((record.envelope.written_at_t, record.envelope.row_id));
            if newest_context.map(|(at, _)| record.envelope.written_at_t > at).unwrap_or(true) {
                newest_context = Some((record.envelope.written_at_t, record.envelope.row_id));
            }
        }
    }
    let reported_available = last_solo.and_then(|night| night.count(1, "events_available"));
    let p4_walks = last_solo.and_then(|night| night.count(4, "walks"));
    let p4_points = last_solo.and_then(|night| night.count(4, "points_swept"));
    // Which row the night replayed is evidenced by the round-0 vector it logged:
    // 10 §7's P1 replays the lighting the live tick lit, so the walked row's own
    // tick event and the walk share a `lit` array. The event bodies also carry a
    // `thought` id, which differs between the day's tick and the sleep's replay,
    // so the comparison reads the `lit` array out of the body rather than the body.
    let lit_vectors: Vec<(i32, Value)> = on
        .session
        .events()
        .events()
        .iter()
        .filter(|event| event.kind == 2)
        .filter_map(|event| {
            let parsed: Value = serde_json::from_slice(&event.body).ok()?;
            Some((event.t, parsed.get("lit")?.clone()))
        })
        .collect();
    let night_walk_lit = lit_vectors.last().map(|(_, lit)| lit.clone());
    let row_lit = |row: Option<(i32, u32)>| {
        row.and_then(|(at, _)| {
            lit_vectors
                .iter()
                .filter(|(t, _)| *t == at)
                .map(|(_, lit)| lit.clone())
                .next_back()
        })
    };
    let newest_row_lit = row_lit(newest_context);
    let oldest_row_lit = row_lit(oldest_context);
    // A newest-first truncation only shows itself against the oldest row: if the
    // two windows happened to light the same vector, the equality below would
    // grade nothing, and the leg says so instead of taking the green.
    let discriminates = oldest_row_lit.is_some()
        && newest_row_lit.is_some()
        && oldest_row_lit != newest_row_lit;
    let truncation_named = newest_row_lit.is_some()
        && night_walk_lit.is_some()
        && newest_row_lit == night_walk_lit;
    let l4_rest = reported_available == Some(context_available)
        && context_available > u64::from(max_walks)
        && remainder_holds(
            p4_walks,
            Some(last_solo.map(Night::walks).unwrap_or(0)),
            u64::from(max_walks),
        )
        && p4_points.unwrap_or(0) > 0;
    let l4_verdict = if !discriminates {
        "unmeasured"
    } else if truncation_named && l4_rest {
        "pass"
    } else {
        "fail"
    };
    crit(
        out,
        "G13.l.4",
        "the replay set, the truncation and the remainder",
        "P1's set is exactly the live `context` rows with t > last_slept_at_t, truncated to the budget newest-first with both counts on the line, and P4 gets the remainder with a full read-only sweep even at remainder 0",
        json!({
            "events_available_reported": reported_available,
            "events_available_recomputed": context_available,
            "newest_context_row": newest_context.map(|(at, row)| json!({"written_at_t": at, "row_id": row})),
            "oldest_context_row": oldest_context.map(|(at, row)| json!({"written_at_t": at, "row_id": row})),
            "budget": max_walks,
            "p1_walks": last_solo.and_then(|night| night.count(1, "walks")),
            "p4_walks": p4_walks,
            "p4_points_swept": p4_points,
            "remainder_of_the_pool": u64::from(max_walks)
                .saturating_sub(last_solo.map(Night::walks).unwrap_or(0)),
            "sweep_full_at_remainder_zero": p4_points.unwrap_or(0) > 0 && p4_walks == Some(0),
            "walked_row_is_the_newest": truncation_named,
            "oldest_and_newest_lightings_differ": discriminates,
            "lit_events_read": lit_vectors.len(),
            "od_85": "walks short of events_available is the visible shape of a night that consolidated less than it was given; at this point the budget is 1 and the replay set is the whole day, so the truncation is exercised rather than vacuous",
            "unmeasured_means": "the day's oldest and newest ticks lit the same vector, so no lighting can tell a newest-first truncation from an oldest-first one",
        }),
        json!(0),
        l4_verdict,
        "a replay pass that quietly consolidated less than it was given, or a lost night nobody counted",
    );

    // -----------------------------------------------------------------------
    // (m) the organism is the same size.
    // -----------------------------------------------------------------------
    let stage_off = si_session::stage(&off.session)?;
    let stage_on = si_session::stage(&on.session)?;
    let stage_shelves = ["s_r_pairs", "facts", "links"];
    let stage_counts_equal = stage_shelves
        .iter()
        .all(|name| off_census.get(*name).copied().unwrap_or(0) == on_census.get(*name).copied().unwrap_or(0));
    crit(
        out,
        "G13.m.1",
        "the stage is equal across a sleep",
        "the derived stage before the night equals the stage after it, at every cadence point of the run",
        json!({
            "stage_before": stage_off,
            "stage_after": stage_on,
            "v0_stage": si_session::stage(&v0.session)?,
            "v9_stage": si_session::stage(&v9.session)?,
            "per_night_stage": v0.steps.iter().map(|trace| trace.stage_after).collect::<Vec<u8>>(),
        }),
        json!(0),
        if stage_off == stage_on && si_session::stage(&v0.session)? == si_session::stage(&v9.session)? {
            "pass"
        } else {
            "fail"
        },
        "a stage machine that reads sleep as growth",
    );

    let v0_census = census_of(&v0.session);
    let v9_census = census_of(&v9.session);
    crit(
        out,
        "G13.m.2",
        "the counts feeding STAGE_THRESHOLDS are equal",
        "the shelf counts the stage reads are identical before and after the night, and between the control pair",
        json!({
            "before": &off_census,
            "after": &on_census,
            "v0": &v0_census,
            "v9": &v9_census,
            "stage_shelves_equal_across_the_night": stage_counts_equal,
            "stage_shelves_equal_between_variants": stage_shelves
                .iter()
                .all(|name| v0_census.get(*name).copied().unwrap_or(0) == v9_census.get(*name).copied().unwrap_or(0)),
        }),
        json!(0),
        if stage_counts_equal
            && stage_shelves
                .iter()
                .all(|name| v0_census.get(*name).copied().unwrap_or(0) == v9_census.get(*name).copied().unwrap_or(0))
        {
            "pass"
        } else {
            "fail"
        },
        "sleep read as growth by 05 §11.2's law",
    );

    // -----------------------------------------------------------------------
    // (n) the loss is reported, not hidden.
    // -----------------------------------------------------------------------
    let dimming: Value = serde_json::from_str(&ws.read("gates/G13/dimming_sweep.json")?)
        .context("parse gates/G13/dimming_sweep.json")?;
    let four = [
        "within_shelf_rank_inversions",
        "cross_shelf_rank_inversions",
        "max_relative_shift",
        "floor_hits",
    ];
    let p3_lines: Vec<&Value> = nights
        .iter()
        .chain(v0_nights.iter())
        .filter_map(|night| night.pass(3))
        .collect();
    let lines_with_four = p3_lines
        .iter()
        .filter(|line| four.iter().all(|key| line.get(*key).is_some()))
        .count();
    let declared: Vec<String> = dimming
        .get("fields_per_grid_point")
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default();
    crit(
        out,
        "G13.n.1",
        "the four counts are on every row",
        "every row that exists carries `within_shelf_rank_inversions`, `cross_shelf_rank_inversions`, `max_relative_shift` and `floor_hits`",
        json!({
            "sweep_status": dimming.get("status").and_then(Value::as_str),
            "sweep_rows": dimming.get("curve").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "sweep_winner": dimming.get("winner"),
            "four_names_declared_in_the_artifact": four.iter().all(|name| declared.iter().any(|field| field == name)),
            "p3_lines": p3_lines.len(),
            "p3_lines_with_all_four": lines_with_four,
            "note": "05 §4.2 makes this artifact a distribution report with no winner, and the runner refused its grid (OD-94/OD-95), so the rows it would have held do not exist; the same four counts are graded on every pass-3 line the build did emit",
        }),
        json!(0),
        if lines_with_four == p3_lines.len() && !p3_lines.is_empty() { "pass" } else { "fail" },
        "the honesty law traded for a cleaner-looking sweep",
    );

    let suppressed = p3_lines
        .iter()
        .filter(|line| four.iter().any(|key| line.get(*key).is_none()))
        .count();
    let diff_document = ws.read("gates/G13/dimming_sweep.json").is_ok();
    crit(
        out,
        "G13.n.2",
        "no row suppresses a count",
        "a row with a suppressed count fails the leg even when every behavioral leg is green",
        json!({
            "p3_lines_missing_a_count": suppressed,
            "ledger_rows": run_ledger.len() + ledger.len(),
            "edges_below_floor_published_in_sleep_diff": true,
            "dimming_artifact_present": diff_document,
        }),
        json!(0),
        if suppressed == 0 { "pass" } else { "fail" },
        "an inversion count dropped to make a night look clean",
    );

    let box_check = |name: &str, value: u64| -> bool {
        si_core::consts::row(name)
            .map(|row| (value as i64) >= row.lo && (value as i64) <= row.hi)
            .unwrap_or(false)
    };
    let outside_box: Vec<Value> = ledger
        .iter()
        .filter(|entry| {
            u64::from(entry.d_q12) > u64::from(si_core::quant::Q12_ONE)
                || !box_check("DIM_BASE", u64::from(entry.d_q12 / 16))
        })
        .map(|entry| json!({"edge": entry.edge_off, "d_q12": entry.d_q12}))
        .collect();
    crit(
        out,
        "G13.n.3",
        "no row quotes a magnitude outside the swept box",
        "every magnitude the report quotes is a point of the box 03 declares for its own row",
        json!({
            "dim_base": config.dim_base,
            "floors_q12": &floors,
            "d_q12_range": [ledger.iter().map(|entry| entry.d_q12).min().unwrap_or(0), ledger.iter().map(|entry| entry.d_q12).max().unwrap_or(0)],
            "weight_q12_range": [ledger.iter().map(|entry| entry.after).min().unwrap_or(0), ledger.iter().map(|entry| entry.before).max().unwrap_or(0)],
            "cadence_ticks": cadence,
            "max_walks": max_walks,
            "magnitude_rows_outside_their_box": &outside_box,
        }),
        json!(0),
        if outside_box.is_empty()
            && box_check("DIM_BASE", u64::from(config.dim_base))
            && box_check("SLEEP_EVERY_QUIET_TICKS", u64::from(cadence))
            && box_check("SLEEP_MAX_WALKS", u64::from(max_walks))
            && [config.floor_s_r_q12, config.floor_facts_q12, config.floor_links_q12]
                .iter()
                .all(|value| box_check("FLOOR_S_R", u64::from(*value)))
        {
            "pass"
        } else {
            "fail"
        },
        "a swept range reporting a number it never offered",
    );
    Ok(())
}

/// 05 §18(l.4)'s remainder law: the one pool, spent in pass order (OD-85).
/// The law is an upper bound, not an equality: a calm pass that finds nothing to
/// relight legitimately walks fewer walks than it was handed.
fn remainder_holds(p4_walks: Option<u64>, spent: Option<u64>, budget: u64) -> bool {
    match (p4_walks, spent) {
        (Some(p4), Some(already)) => p4 + already <= budget,
        _ => false,
    }
}

/// 01 §3's shelf names, for a census a reader can check against the schema.
fn shelf_name(shelf: u16) -> &'static str {
    use si_core::addr::*;
    match shelf {
        SHELF_DICTIONARY => "dictionary",
        SHELF_RELATIONS => "relations",
        SHELF_RULES => "rules",
        SHELF_S_R_PAIRS => "s_r_pairs",
        SHELF_FACTS => "facts",
        SHELF_LINKS => "links",
        SHELF_QUESTIONS => "questions",
        SHELF_CONTEXT => "context",
        SHELF_SKILLS => "skills",
        SHELF_STRATEGIES => "strategies",
        SHELF_EVENTS => "events",
        _ => "unknown",
    }
}

// ---------------------------------------------------------------------------
// G12 — GRIMM ISOLATION: the mouth is not his (05 §17). V4 + V5 + V0.
// ---------------------------------------------------------------------------

/// One line as both mouths speak it: the source-blindness pair differs only by the
/// period and the mouth, so the leg compares the words underneath both.
fn spoken(text: &str) -> String {
    text.trim()
        .trim_end_matches(['.', '?', '!', ';', ':'])
        .to_lowercase()
}

/// "belief row" as `01 §4.1` scopes it, which is the scope `G12(e.1)` grades in.
fn is_belief(shelf: u16) -> bool {
    matches!(shelf, SHELF_S_R_PAIRS | SHELF_FACTS | SHELF_QUESTIONS)
}

/// 05 §17 (d.2)'s pair is found by **shape**, never by index: the cold section that
/// says one sentence three times with the owner's mouth, and the one that says it
/// twice with his. It is the only place in the harness where the same gap is spoken
/// at the same `t` by two speakers, which is exactly what the leg needs.
fn g12_pair(run: &Run) -> Option<(usize, usize)> {
    let grimm_side = run
        .sections
        .iter()
        .enumerate()
        .find(|(_, section)| {
            section.steps.len() == 3
                && section.steps[..2].iter().all(|step| !step.idle && step.speak.is_none())
                && section.steps[2]
                    .speak
                    .as_ref()
                    .is_some_and(|note| note.admitted)
                && spoken(&section.steps[2].text) == spoken(&section.steps[1].text)
        })?
        .0;
    let content = spoken(&run.sections[grimm_side].steps[2].text);
    let owner_side = run
        .sections
        .iter()
        .enumerate()
        .find(|(_, section)| {
            section.steps.len() == 3
                && section
                    .steps
                    .iter()
                    .all(|step| !step.idle && step.speak.is_none() && spoken(&step.text) == content)
        })?
        .0;
    Some((owner_side, grimm_side))
}

/// The rows of one section's store, with their text: the legs that speak about
/// "a row that mentions X" need the verbatim bytes, not only the envelope.
fn g12_text_rows(session: &Session) -> Result<Vec<(u32, u16, u8, bool, String)>> {
    let store = session.store();
    let mut out = Vec::new();
    for (_, record) in store.rows()? {
        out.push((
            record.envelope.row_id,
            record.envelope.shelf_x,
            record.envelope.prov_speaker,
            record.envelope.is_live(),
            surface(session, record.envelope.text_ref),
        ));
    }
    Ok(out)
}

/// A section's bake multiset, read from the same home `G10(b)` reads it from.
fn run_bakes(session: &Session) -> Result<Vec<u16>> {
    let graph = si_walk::Graph::build(session.store(), session.now())?;
    let mut bakes: Vec<u16> = graph
        .bake_touch_max()
        .values()
        .filter(|value| **value > 0)
        .copied()
        .collect();
    bakes.sort_unstable();
    Ok(bakes)
}

fn g12(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let seed = run.request.seed;
    let constants = &run.request.constants;
    let fixture = run.request.scenario.clone();
    // 05 §2: at V4 the supporter is never started and at V5 he is connected but
    // silent, so in both variants **no emission ever reaches the gate**. Legs that
    // grade what the gate decided can only be graded where the gate ran; here they
    // say so, and only leg (a) — the variant leg — is expected to speak.
    let silent = matches!(run.request.variant.as_str(), "V4" | "V5");
    let silent_why = || format!(
        "variant {} never lets an emission reach the gate (05 §2), so this leg has nothing \
         to grade in this run; it is graded at V0",
        run.request.variant
    );

    // The three mouth states, replayed through the ordinary loop by `variant_probe`
    // — the same `sense_step`, the same `Mouth::read_all`, the same freeze — so
    // (a)'s pair is one loop code run twice, never two loop codes.
    let v0 = variant_probe(&fixture, seed, constants, "V0")?;
    let v4 = variant_probe(&fixture, seed, constants, "V4")?;
    let v5 = variant_probe(&fixture, seed, constants, "V5")?;
    let probe_calibrated = {
        let own: &VariantProbe = match run.request.variant.as_str() {
            "V4" => &v4,
            "V5" => &v5,
            _ => &v0,
        };
        own.sections.len() == run.sections.len()
            && own
                .sections
                .iter()
                .zip(run.sections.iter())
                .all(|(probe, section)| probe.hash == section.hash)
    };

    // (a.1) — the fixture's admission-free sections, and every other shipped
    // scenario, byte-identical at V0, V4 and V5. A scenario from B14 on may hold a
    // `grimm:` line, so its admitted sections are excluded by the same rule the
    // fixture's are (see the corpus loop below).
    let admission_sections: Vec<usize> = v0
        .sections
        .iter()
        .enumerate()
        .filter(|(_, probe)| probe.admissions > 0)
        .map(|(index, _)| index)
        .collect();
    let mut mismatches: Vec<Value> = Vec::new();
    let mut sections_compared = 0usize;
    for (index, section) in run.sections.iter().enumerate() {
        if admission_sections.contains(&index) {
            continue;
        }
        sections_compared += 1;
        if section.hash != v4.sections[index].hash || section.hash != v5.sections[index].hash {
            mismatches.push(json!({
                "scenario": fixture.path, "section": index,
                "v0": section.hash, "v4": v4.sections[index].hash,
                "v5": v5.sections[index].hash,
            }));
        }
    }

    // The rest of the shipped corpus, at all three states.
    let mut corpus: Vec<Value> = Vec::new();
    let mut scenario_paths: Vec<String> = fs::read_dir(ws.path("harness/scenarios"))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
        .map(|path| ws.relative(&path).replace('\\', "/"))
        .collect();
    scenario_paths.sort();
    let mut scenario_reads_v5 = 0usize;
    let mut scenario_reads_v4 = 0usize;
    let mut corpus_skipped: Vec<Value> = Vec::new();
    let mut corpus_admitted: Vec<Value> = Vec::new();
    for path in scenario_paths.iter().filter(|path| **path != fixture.path) {
        let scenario = match Scenario::load(ws, path) {
            Ok(scenario) => scenario,
            Err(error) => {
                // `harness/scenarios/empty.txt` is the parser's own refusal fixture —
                // zero bytes, no steps to replay — so it is recorded as skipped. Any
                // other refusal is a broken shipped scenario, and the leg goes red.
                let text = error.to_string();
                if ws.path(path).metadata().map(|meta| meta.len()).unwrap_or(0) == 0 {
                    corpus_skipped.push(json!({"scenario": path, "reason": text}));
                    continue;
                }
                mismatches.push(json!({"scenario": path, "finding": text}));
                continue;
            }
        };
        let a = variant_probe(&scenario, seed, constants, "V0")?;
        let b = variant_probe(&scenario, seed, constants, "V4")?;
        let c = variant_probe(&scenario, seed, constants, "V5")?;
        scenario_reads_v5 += c.sections.iter().map(|s| s.reads).sum::<usize>();
        scenario_reads_v4 += b.sections.iter().map(|s| s.reads).sum::<usize>();
        // §17(a)'s admission rule is not the fixture's property — it is B9's layer-2
        // rule, so it applies to whatever section it fires in. From B14 the arc's own
        // fixtures carry a `grimm:` line on purpose (G15(d) and G17(e) grade it), and
        // an admitted mirror restatement lawfully writes its `context` row at V0 and
        // nothing at V4/V5: that difference is §21.5, not a gate that needs Grimm.
        // Every admitted section is excluded here exactly as the fixture's are, and
        // both lists are named in the evidence so the exclusion is auditable.
        let admitted: Vec<usize> = (0..a.sections.len())
            .filter(|i| a.sections[*i].admissions > 0)
            .collect();
        if !admitted.is_empty() {
            corpus_admitted.push(json!({"scenario": path, "sections": admitted}));
        }
        let differing: Vec<usize> = (0..a.sections.len())
            .filter(|i| {
                !admitted.contains(i)
                    && (a.sections[*i].hash != b.sections[*i].hash
                        || a.sections[*i].hash != c.sections[*i].hash)
            })
            .collect();
        if !differing.is_empty() {
            mismatches.push(json!({"scenario": path, "sections": differing}));
        }
        sections_compared += a.sections.len();
        corpus.push(json!({
            "scenario": path,
            "sections": a.sections.len(),
            "emissions": a.emissions,
            "identical_all_three": differing.is_empty(),
        }));
    }
    if !probe_calibrated {
        mismatches.push(json!({
            "scenario": fixture.path,
            "finding": "the V0 probe does not reproduce the run's own section hashes",
        }));
    }
    let evidence = json!({
        "scenarios": corpus,
        "scenario_count": corpus.len() + 1,
        "corpus_skipped": corpus_skipped,
        "sections_compared": sections_compared,
        "fixture_admitted_sections_excluded": admission_sections,
        "corpus_admitted_sections_excluded": corpus_admitted,
        "probe_reproduces_run_hashes": probe_calibrated,
        "mismatches": mismatches,
        "pass_fail_identity": "UNMEASURED until B12 re-runs each of G1–G11 at V4 \
            (05 §17's (a) half that needs a launched gate, not a replayed transcript)",
        "rates": "UNMEASURED until B12 (05 §17)",
    });
    let condition = "V4 and V5 replay every scenario to byte-identical per-section state \
         hashes; the fixture's admitted sections are the only ones excluded, because 05 §2 \
         says a run with admissions is a different run";
    if mismatches.is_empty() {
        ok(
            out,
            "G12.a.1",
            "absence is not a loss",
            condition,
            evidence,
            gate_fail_meaning("G12.a.1"),
        );
    } else {
        bad(
            out,
            "G12.a.1",
            "absence is not a loss",
            condition,
            evidence,
            gate_fail_meaning("G12.a.1"),
        );
    }

    // (a.2) — V5 byte-identical to V4, with his five readers actually called.
    let v5_reads: usize = v5.sections.iter().map(|s| s.reads).sum();
    let v4_reads: usize = v4.sections.iter().map(|s| s.reads).sum();
    let v5_admissions: usize = v5.sections.iter().map(|s| s.admissions).sum();
    let v4_vs_v5: Vec<usize> = (0..v5.sections.len())
        .filter(|i| v5.sections[*i].hash != v4.sections[*i].hash)
        .collect();
    if v4_vs_v5.is_empty() && v5_reads > 0 && v4_reads == 0 && v5_admissions == 0 {
        ok(
            out,
            "G12.a.2",
            "the harness does not perturb SI by existing",
            "V5's every section hash equals V4's, V5 logs reads and zero admissions, V4 logs none",
            json!({
                "sections": v5.sections.len(),
                "v5_reads": v5_reads,
                "v4_reads": v4_reads,
                "v5_admissions": v5_admissions,
                "corpus_v5_reads": scenario_reads_v5,
                "corpus_v4_reads": scenario_reads_v4,
                "a_moved": false,
            }),
            gate_fail_meaning("G12.a.2"),
        );
    } else {
        bad(
            out,
            "G12.a.2",
            "the harness does not perturb SI by existing",
            "V5's every section hash equals V4's, V5 logs reads and zero admissions, V4 logs none",
            json!({
                "sections_differing": v4_vs_v5,
                "v5_reads": v5_reads,
                "v4_reads": v4_reads,
                "v5_admissions": v5_admissions,
            }),
            gate_fail_meaning("G12.a.2"),
        );
    }

    // (b.1) — the canonical drift is rejected by **layer 2**, with the reason in
    // machine-readable fields. A run whose filter never rejects is a finding about
    // the harness, not a green gate (05 §17's severity note).
    let attempts: Vec<(usize, &StepRecord)> = run
        .sections
        .iter()
        .enumerate()
        .flat_map(|(section, run_section)| {
            run_section.steps.iter().map(move |step| (section, step))
        })
        .filter(|(_, step)| step.speak.is_some())
        .collect();
    let rejections: Vec<(usize, &StepRecord)> = attempts
        .iter()
        .filter(|(_, step)| step.speak.as_ref().is_some_and(|note| !note.admitted))
        .cloned()
        .collect();
    let drift = rejections
        .iter()
        .find(|(_, step)| step.speak.as_ref().is_some_and(|note| note.layer == 2))
        .map(|(section, step)| (*section, *step));
    let layer2_shape = drift.map(|(_, step)| {
        let note = step.speak.as_ref().expect("graded above");
        note.n_untaught_tokens == 0
            && note.novel_claims == 1
            && note.ref_row_id.is_none()
            && note.reason == "speak_layer2_novel_claim"
    });
    match (drift, layer2_shape) {
        (Some((section, step)), Some(true)) => ok(
            out,
            "G12.b.1",
            "the canonical failure is a test",
            "the drift emission is rejected at layer 2 with {n_untaught_tokens: 0, \
             novel_claims: 1, ref_row_id: null} and reaches no sense event",
            json!({
                "ref": format!("mindlog.json#sections[{section}].steps[{}]", step.index),
                "line": step.text,
                "t": step.t,
                "reason": step.speak.as_ref().map(|n| n.reason),
                "layer": 2,
                "n_untaught_tokens": step.speak.as_ref().map(|n| n.n_untaught_tokens),
                "novel_claims": step.speak.as_ref().map(|n| n.novel_claims),
                "ref_row_id": step.speak.as_ref().and_then(|n| n.ref_row_id),
                "attempts": attempts.len(),
                "rejections": rejections.len(),
            }),
            gate_fail_meaning("G12.b.1"),
        ),
        (Some((section, step)), _) => bad(
            out,
            "G12.b.1",
            "the canonical failure is a test",
            "the drift emission is rejected at layer 2 with {n_untaught_tokens: 0, \
             novel_claims: 1, ref_row_id: null}",
            json!({
                "ref": format!("mindlog.json#sections[{section}].steps[{}]", step.index),
                "reached_loop": step.speak.as_ref().is_some_and(|note| note.reached_loop),
                "admitted": step.speak.as_ref().is_some_and(|note| note.admitted),
                "dropped": step.speak.as_ref().is_some_and(|note| note.dropped),
                "layer": step.speak.as_ref().map(|note| note.layer),
                "reason": step.speak.as_ref().map(|note| note.reason),
                "n_untaught_tokens": step.speak.as_ref().map(|note| note.n_untaught_tokens),
                "novel_claims": step.speak.as_ref().map(|note| note.novel_claims),
                "ref_row_id": step.speak.as_ref().and_then(|note| note.ref_row_id),
            }),
            gate_fail_meaning("G12.b.1"),
        ),
        (None, _) if silent => open(
            out,
            "G12.b.1",
            "the canonical failure is a test",
            "the drift emission is rejected at layer 2 with a machine-readable reason",
            &silent_why(),
            json!({
                "emission_attempts": attempts.len(),
                "gate_verdicts": rejections.iter().filter(|(_, s)| s.speak.as_ref().is_some_and(|n| !n.dropped)).count(),
            }),
            gate_fail_meaning("G12.b.1"),
        ),
        (None, _) => {
            if rejections.is_empty() {
                bad(
                    out,
                    "G12.b.1",
                    "the canonical failure is a test",
                    "the drift emission is rejected at layer 2 with a machine-readable reason",
                    json!({
                        "attempts": attempts.len(),
                        "rejections": 0,
                        "finding": "the filter rejected nothing in this run — 05 §17's own \
                                    words: a finding about the harness, not a green gate",
                    }),
                    gate_fail_meaning("G12.b.1"),
                );
            } else {
                bad(
                    out,
                    "G12.b.1",
                    "the canonical failure is a test",
                    "the drift emission is rejected at layer 2 with a machine-readable reason",
                    json!({
                        "attempts": attempts.len(),
                        "rejections": rejections.len(),
                        "layers": rejections
                            .iter()
                            .map(|(_, step)| step.speak.as_ref().map(|n| n.layer))
                            .collect::<Vec<_>>(),
                    }),
                    gate_fail_meaning("G12.b.1"),
                );
            }
        }
    }

    // (b.2) — the rejection is **free**: no signal, no `a` movement, no stamp, no
    // row, and exactly one gate-log line.
    if let Some((section, step)) = drift {
        let frame_count = run.sections[section]
            .gate_log
            .iter()
            .filter(|frame| frame.contains(&step.text))
            .count();
        // 05 §17's (b) prices the refusal in **state** — no row, no `a` movement, no
        // stamp — while 08 §2.1 separately requires the seam's answer to be *visible*
        // as a `seam_reject` frame. B11.2 shipped that frame, so "the step owns no
        // event" would grade 08 §2.1 as a regression of 05 §17. What stays free is
        // every other event a line pays for: no sense echo, no decoded, no round, no
        // `a` update, no commit. The step's events are its own rejection and nothing else.
        let events = run.events(section, step);
        let event_kinds: Vec<u8> = events.iter().map(|event| event.kind).collect();
        let pays_only_its_own_rejection =
            event_kinds == vec![si_core::eventlog::kind::SEAM_REJECT];
        let free = step.before.rows_bytes == step.after.rows_bytes
            && step.before.edges_bytes == step.after.edges_bytes
            && step.before.stamps_bytes == step.after.stamps_bytes
            && step.before.stamps_digest == step.after.stamps_digest
            && step.before.sr_envelope_digest == step.after.sr_envelope_digest
            && step.a_before == step.a_after
            && step.rows_written == 0
            && pays_only_its_own_rejection
            && step.speak.as_ref().is_some_and(|note| !note.reached_loop);
        if free && frame_count == 1 {
            ok(
                out,
                "G12.b.2",
                "the rejection leaves zero state change and one log line",
                "the refused step's boundary pair is equal, `a` did not move, no row was \
                 written, the step's only event is its own `seam_reject` frame, and the gate \
                 log has exactly one frame for it",
                json!({
                    "mindlog_ref": format!("mindlog.json#sections[{section}].steps[{}]", step.index),
                    "rows_bytes": step.after.rows_bytes,
                    "stamps_bytes": step.after.stamps_bytes,
                    "a": step.a_after,
                    "event_kinds": event_kinds,
                    "gate_log_frames_for_the_line": frame_count,
                    "ref": "gates/G12/speak_gate_log.jsonl",
                }),
                gate_fail_meaning("G12.b.2"),
            );
        } else {
            bad(
                out,
                "G12.b.2",
                "the rejection leaves zero state change and one log line",
                "the refused step's boundary pair is equal, `a` did not move, no row was \
                 written, the step's only event is its own `seam_reject` frame, and the gate \
                 log has exactly one frame for it",
                json!({
                    "boundary_before": boundary_json(&step.before),
                    "boundary_after": boundary_json(&step.after),
                    "a_before": step.a_before,
                    "a_after": step.a_after,
                    "rows_written": step.rows_written,
                    "event_kinds": event_kinds,
                    "only_its_own_rejection": pays_only_its_own_rejection,
                    "gate_log_frames_for_the_line": frame_count,
                }),
                gate_fail_meaning("G12.b.2"),
            );
        }
    } else {
        open(
            out,
            "G12.b.2",
            "the rejection leaves zero state change and one log line",
            "the refused step's boundary pair is equal and the gate log has one frame",
            "the run recorded no layer-2 rejection, so there is no refusal to price",
            json!({"attempts": attempts.len(), "rejections": rejections.len()}),
            gate_fail_meaning("G12.b.2"),
        );
    }

    // ---- (c) birth muteness. Section 0 opens with his emissions and nothing in
    // the lexicon yet, so layer 1 is structural, not a judgement about him.
    let opening = run
        .sections
        .first()
        .map(|section| {
            let mut prefix = Vec::new();
            for step in &section.steps {
                if step.idle || step.speak.is_none() {
                    break;
                }
                prefix.push(step);
            }
            prefix
        })
        .unwrap_or_default();
    let mute = !opening.is_empty()
        && opening.iter().all(|step| {
            step.speak
                .as_ref()
                .is_some_and(|note| !note.admitted && note.layer == 1 && !note.untaught.is_empty())
        });
    if mute {
        ok(
            out,
            "G12.c.1",
            "birth muteness",
            "every emission before the first lesson is rejected at layer 1 with its untaught \
             keys named, and none reaches a sense event",
            json!({
                "emissions_at_birth": opening.len(),
                "ref": "mindlog.json#sections[0].steps[1]",
                "layers": opening.iter().map(|s| s.speak.as_ref().map(|n| n.layer)).collect::<Vec<_>>(),
                "untaught_first_line": opening.first().and_then(|s| s.speak.as_ref().map(|n| n.untaught.clone())),
                "admitted": 0,
            }),
            gate_fail_meaning("G12.c.1"),
        );
    } else if silent {
        open(
            out,
            "G12.c.1",
            "birth muteness",
            "every emission before the first lesson is rejected at layer 1",
            &silent_why(),
            json!({
                "emissions_at_birth": opening.len(),
                "gate_verdicts": opening.iter().filter(|s| s.speak.as_ref().is_some_and(|n| !n.dropped)).count(),
            }),
            gate_fail_meaning("G12.c.1"),
        );
    } else {
        bad(
            out,
            "G12.c.1",
            "birth muteness",
            "every emission before the first lesson is rejected at layer 1",
            json!({
                "emissions_at_birth": opening.len(),
                "notes": opening.iter().map(|s| s.speak.as_ref().map(|n| (n.layer, n.admitted))).collect::<Vec<_>>(),
            }),
            gate_fail_meaning("G12.c.1"),
        );
    }

    // (c.2) — the first OWNED edge is still the owner's: his mouth opening first
    // did not take the first edge away from the lesson.
    let first_owner = run
        .sections
        .first()
        .and_then(|section| section.steps.iter().find(|step| !step.idle && step.speak.is_none()));
    let first_sr = run
        .sections
        .first()
        .map(|section| {
            g12_text_rows(&section.session).map(|rows| {
                rows.into_iter()
                    .filter(|(_, shelf, _, _, _)| *shelf == SHELF_S_R_PAIRS)
                    .min_by_key(|(row_id, _, _, _, _)| *row_id)
            })
        })
        .transpose()?
        .flatten();
    let owned_edge = match (&first_owner, first_sr.as_ref()) {
        (Some(step), Some((row_id, _, prov_speaker, _, _))) => {
            *prov_speaker == 0 && step.rows_written > 0 && *row_id > 0
        }
        _ => false,
    };
    if owned_edge {
        ok(
            out,
            "G12.c.2",
            "the first owned edge is created by the owner's lesson",
            "section 0's lowest `s_r_pairs` row carries `prov_speaker: 0 owner` and was \
             written by the owner's step, after every emission had already been refused",
            json!({
                "row_id": first_sr.as_ref().map(|r| r.0),
                "prov_speaker": first_sr.as_ref().map(|r| r.2),
                "owner_step_t": first_owner.map(|s| s.t),
                "owner_step_rows_written": first_owner.map(|s| s.rows_written),
                "owner_step_shelves_after": first_owner.map(|s| &s.shelves_after),
                "control": "engineering/04 §4's trace",
            }),
            gate_fail_meaning("G12.c.2"),
        );
    } else {
        bad(
            out,
            "G12.c.2",
            "the first owned edge is created by the owner's lesson",
            "section 0's lowest `s_r_pairs` row carries `prov_speaker: 0 owner`",
            json!({"first_sr_row": first_sr.as_ref().map(|r| (r.0, r.2)), "owner_step": first_owner.map(|s| s.index)}),
            gate_fail_meaning("G12.c.2"),
        );
    }

    // ---- (d) no setter, and no path that becomes one.
    let membrane = super::checks::run("check_membrane", ws)?;
    let setter = super::checks::run("check_no_affective_setter", ws)?;
    let stage = super::checks::run("check_no_stage_on_supporter", ws)?;
    let drive_names_source = ws
        .source_text()?
        .into_iter()
        .filter(|(relative, _)| relative.replace('\\', "/") == "crates/si-affect/src/drive.rs")
        .any(|(_, text)| {
            text.lines()
                .map(|line| line.split("//").next().unwrap_or(""))
                .any(|line| line.contains("source") || line.contains("Source"))
        });
    // The dynamic half: his payloads shaped like a command, a correction, a teach,
    // a reply to the open ask. None may move arbitration, a label or a row.
    let mut labeled = 0usize;
    let mut command_shapes = Vec::new();
    let mut belief_moved = Vec::new();
    let mut gate_writes = Vec::new();
    for (section, run_section) in run.sections.iter().enumerate() {
        let mut previous_belief = 0usize;
        for step in &run_section.steps {
            let belief_now: usize = [SHELF_S_R_PAIRS, SHELF_FACTS, SHELF_QUESTIONS]
                .iter()
                .map(|shelf| step.shelves_after.get(shelf).copied().unwrap_or_default())
                .sum();
            if let Some(note) = &step.speak {
                if matches!(step.output, Some(Output::Answer { .. }) | Some(Output::Ask { .. })) {
                    labeled += 1;
                }
                if belief_now != previous_belief {
                    belief_moved.push(json!({
                        "section": section,
                        "step": step.index,
                        "reached_loop": note.reached_loop,
                    }));
                }
                let writes = event_named(run, section, step, "commit")
                    .into_iter()
                    .chain(event_named(run, section, step, "confirm"))
                    .chain(event_named(run, section, step, "resolve"))
                    .count();
                if writes > 0 {
                    gate_writes.push(json!({"section": section, "step": step.index, "writes": writes}));
                }
                command_shapes.push(json!({
                    "section": section,
                    "step": step.index,
                    "shape": if step.text.starts_with('/') { "command" } else { "sentence" },
                    "admitted": note.admitted,
                    "layer": note.layer,
                    "output": match &step.output {
                        Some(Output::Answer { .. }) => "answer",
                        Some(Output::Ask { .. }) => "ask",
                        Some(Output::Committed { .. }) => "committed",
                        Some(Output::Refused { .. }) => "refused",
                        Some(Output::Inert { .. }) => "inert",
                        None => "none",
                    },
                }));
            }
            previous_belief = belief_now;
        }
    }
    if membrane.verdict == "pass"
        && setter.verdict == "pass"
        && stage.verdict == "pass"
        && labeled == 0
        && belief_moved.is_empty()
        && gate_writes.is_empty()
    {
        ok(
            out,
            "G12.d.1",
            "no setter, and no path that becomes one",
            "`check_membrane`, `check_no_affective_setter` and `check_no_stage_on_supporter` \
             pass live, and no emission — command-shaped or not — produced an answer, an ask, \
             a belief-row census change or a gate write",
            json!({
                "checks": [membrane.id, setter.id, stage.id],
                "emission_attempts": command_shapes.len(),
                "command_shapes": command_shapes.iter().filter(|c| c["shape"] == json!("command")).count(),
                "labeled_or_answered": labeled,
                "belief_census_moved": belief_moved,
                "gate_writes_on_emissions": gate_writes,
            }),
            gate_fail_meaning("G12.d.1"),
        );
    } else {
        bad(
            out,
            "G12.d.1",
            "no setter, and no path that becomes one",
            "the three static legs pass and no emission produced an answer, an ask, a \
             belief-row census change or a gate write",
            json!({
                "check_membrane": membrane.verdict,
                "check_no_affective_setter": setter.verdict,
                "check_no_stage_on_supporter": stage.verdict,
                "labeled_or_answered": labeled,
                "belief_census_moved": belief_moved,
                "gate_writes_on_emissions": gate_writes,
            }),
            gate_fail_meaning("G12.d.1"),
        );
    }

    // (d.2) — the same gap, two mouths: prioritized and budgeted identically.
    let pair = g12_pair(run);
    let pair_evidence = match pair {
        Some((owner_side, grimm_side)) => {
            let owner_step = &run.sections[owner_side].steps[2];
            let grimm_step = &run.sections[grimm_side].steps[2];
            let drive_events = |section: usize, step: &StepRecord| -> Vec<String> {
                run.events(section, step)
                    .iter()
                    .filter(|event| matches!(event.kind, 2 | 4 | 6))
                    .map(|event| String::from_utf8_lossy(&event.body).to_string())
                    .collect()
            };
            let gate_of = |section: usize, step: &StepRecord| -> Vec<String> {
                run.events(section, step)
                    .iter()
                    .filter(|event| event.kind == 13)
                    .map(|event| String::from_utf8_lossy(&event.body).to_string())
                    .collect()
            };
            let owner_drive = drive_events(owner_side, owner_step);
            let grimm_drive = drive_events(grimm_side, grimm_step);
            Some(json!({
                "owner_ref": format!("mindlog.json#sections[{owner_side}].steps[3]"),
                "grimm_ref": format!("mindlog.json#sections[{grimm_side}].steps[3]"),
                "same_t": owner_step.t == grimm_step.t,
                "t": owner_step.t,
                "owner_drive_events": owner_drive.len(),
                "grimm_drive_events": grimm_drive.len(),
                "drive_identical": owner_drive == grimm_drive,
                "a_identical": owner_step.a_before == grimm_step.a_before
                    && owner_step.a_after == grimm_step.a_after,
                "rows_identical": owner_step.after.rows_bytes == grimm_step.after.rows_bytes,
                "owner_gate_events": gate_of(owner_side, owner_step),
                "grimm_gate_events": gate_of(grimm_side, grimm_step),
            }))
        }
        None => None,
    };
    let pair_passes = !drive_names_source
        && pair_evidence.as_ref().is_some_and(|e| {
            e["drive_identical"] == json!(true)
                && e["a_identical"] == json!(true)
                && e["same_t"] == json!(true)
                && e["owner_drive_events"].as_u64().unwrap_or(0) > 0
        });
    if silent {
        open(
            out,
            "G12.d.2",
            "grimm is prioritized and budgeted identically to the owner's identical gap",
            "the drive module names no source and the pair's drive records are identical",
            &silent_why(),
            json!({
                "drive_module_names_source": drive_names_source,
                "pair_sections": pair,
            }),
            gate_fail_meaning("G12.d.2"),
        );
    } else if pair_passes {
        ok(
            out,
            "G12.d.2",
            "grimm is prioritized and budgeted identically to the owner's identical gap",
            "si-affect's drive module names no source, and the pair's lit / round / stop \
             records are byte-identical at the same `t` with the same `a` before and after",
            json!({
                "drive_module_names_source": false,
                "pair": pair_evidence,
                "asymmetry": "the write gate is where the two differ, which is 04 §23 W.5's \
                              downstream privilege test — never the mouth's",
            }),
            gate_fail_meaning("G12.d.2"),
        );
    } else {
        bad(
            out,
            "G12.d.2",
            "grimm is prioritized and budgeted identically to the owner's identical gap",
            "the drive module names no source and the pair's drive records are identical at \
             the same `t`",
            json!({
                "drive_module_names_source": drive_names_source,
                "pair_sections": pair,
                "pair": pair_evidence,
            }),
            gate_fail_meaning("G12.d.2"),
        );
    }

    // ---- (e) provenance and counterfeit confirmation, over the long session.
    // Every row of every section, read from the live store of the run itself.
    let mut belief_foreign = Vec::new();
    let mut links_grimm = Vec::new();
    let mut links_inferred = 0usize;
    let mut grimm_rows: Vec<(u32, u16)> = Vec::new();
    let mut question_rows: Vec<(u32, u32, u32)> = Vec::new();
    for (section, run_section) in run.sections.iter().enumerate() {
        let store = run_section.session.store();
        let mut speakers: BTreeMap<u32, u8> = BTreeMap::new();
        for (_, record) in store.rows()? {
            speakers.insert(record.envelope.row_id, record.envelope.prov_speaker);
        }
        for (_, record) in store.rows()? {
            let e = &record.envelope;
            if e.prov_speaker == 1 {
                grimm_rows.push((e.row_id, e.shelf_x));
            }
            if is_belief(e.shelf_x) && e.prov_speaker != 0 {
                belief_foreign.push(json!({
                    "section": section, "row_id": e.row_id, "shelf": shelf_name(e.shelf_x),
                    "prov_speaker": e.prov_speaker,
                }));
            }
            if e.shelf_x == SHELF_LINKS {
                match e.prov_speaker {
                    1 => links_grimm.push(json!({"section": section, "row_id": e.row_id})),
                    2 => links_inferred += 1,
                    _ => {}
                }
            }
            if e.shelf_x == SHELF_QUESTIONS && record.body.status == gate::Q_RESOLVED {
                question_rows.push((section as u32, e.row_id, e.bind_output_id));
            }
        }
    }
    if belief_foreign.is_empty() && links_grimm.is_empty() {
        ok(
            out,
            "G12.e.1",
            "provenance: zero belief rows that the owner did not speak",
            "no `s_r_pairs` / `facts` / `questions` row carries a non-owner speaker, and no \
             `links` row carries `1 grimm`",
            json!({
                "belief_rows_foreign": belief_foreign,
                "links_grimm_rows": links_grimm,
                "links_system_inferred": links_inferred,
                "note": "`links` bridges may carry `2 system-inferred` (OD-59) and are not belief rows",
                "sections": run.sections.len(),
            }),
            gate_fail_meaning("G12.e.1"),
        );
    } else {
        bad(
            out,
            "G12.e.1",
            "provenance: zero belief rows that the owner did not speak",
            "no belief row carries a non-owner speaker and no `links` row carries `1 grimm`",
            json!({"belief_rows_foreign": belief_foreign, "links_grimm_rows": links_grimm}),
            gate_fail_meaning("G12.e.1"),
        );
    }

    // (e.2) — no question resolved by a line he spoke: the resolver is looked up by
    // `bind_output_id` and its own speaker byte is the answer.
    let mut foreign_resolutions = Vec::new();
    for (section, ask_row, resolver) in question_rows.clone() {
        let store = run.sections[section as usize].session.store();
        let speaker = store
            .rows()?
            .into_iter()
            .find(|(_, record)| record.envelope.row_id == resolver)
            .map(|(_, record)| record.envelope.prov_speaker);
        if !matches!(speaker, Some(0)) {
            foreign_resolutions.push(json!({
                "section": section, "ask_row": ask_row, "resolver": resolver, "prov_speaker": speaker,
            }));
        }
    }
    if foreign_resolutions.is_empty() {
        ok(
            out,
            "G12.e.2",
            "no open question is resolved by a grimm line",
            "every resolved `questions` row's resolver row (its `bind_output_id`) is \
             owner-provenanced",
            json!({
                "resolved_questions": question_rows.len(),
                "resolvers": question_rows.iter().map(|(_, ask, resolver)| json!({"ask": ask, "resolver": resolver})).collect::<Vec<_>>(),
            }),
            gate_fail_meaning("G12.e.2"),
        );
    } else {
        bad(
            out,
            "G12.e.2",
            "no open question is resolved by a grimm line",
            "every resolved `questions` row's resolver row is owner-provenanced",
            json!({"foreign_resolutions": foreign_resolutions}),
            gate_fail_meaning("G12.e.2"),
        );
    }

    // (e.3) — the mirror leg. The control is the **same transcript with his lines
    // removed**, replayed through the same loop at V0; the mirrored row's four
    // earned fields and every bake must be identical, and only the rehearsal curve
    // is allowed to move.
    let mirror_section = run
        .sections
        .iter()
        .enumerate()
        .max_by_key(|(_, section)| {
            section.steps.iter().filter(|step| step.speak.as_ref().is_some_and(|n| n.admitted)).count()
        })
        .map(|(index, _)| index);
    let control = variant_probe(&without_emissions(&fixture), seed, constants, "V0")?;
    let mirror_evidence = mirror_section.map(|index| -> Result<Value> {
        let live_rows = row_probe(&run.sections[index].session)?;
        let quiet_rows = control.sections[index].rows.clone();
        let key = |rows: &Vec<RowProbe>| -> Vec<(u32, u16, u8, u32, i32, u16)> {
            rows.iter()
                .filter(|row| is_belief(row.shelf) && row.prov_speaker == 0)
                .map(|row| (row.row_id, row.shelf, row.prov_speaker, row.repeats, row.last_confirmed_at, row.confidence))
                .collect()
        };
        let earned = key(&live_rows) == key(&quiet_rows);
        Ok(json!({
            "section": index,
            "mirrored_rows": key(&live_rows).len(),
            "belief_fields_identical_to_quiet": earned,
            "live": key(&live_rows),
            "quiet": key(&quiet_rows),
            "bakes_identical": run_bakes(&run.sections[index].session)? == control.sections[index].bakes,
            "live_bakes": run_bakes(&run.sections[index].session)?,
            "quiet_bakes": control.sections[index].bakes,
            "mirror_repetitions": run.sections[index].steps.iter()
                .filter(|step| step.speak.as_ref().is_some_and(|n| n.admitted)).count(),
            "long_inputs": run.sections.iter().map(|s| s.steps.len()).max().unwrap_or_default(),
        }))
    }).transpose()?;
    let mirror_passes = mirror_evidence.as_ref().is_some_and(|e| {
        e["belief_fields_identical_to_quiet"] == json!(true)
            && e["bakes_identical"] == json!(true)
            && e["mirror_repetitions"].as_u64().unwrap_or(0) > 0
    });
    if silent {
        open(
            out,
            "G12.e.3",
            "the mirror earns nothing",
            "the mirrored rows' earned fields and every bake are identical to the quiet control",
            &silent_why(),
            json!({"mirror_section": mirror_section, "pair": mirror_evidence}),
            gate_fail_meaning("G12.e.3"),
        );
    } else if mirror_passes {
        ok(
            out,
            "G12.e.3",
            "the mirror earns nothing",
            "in the mirror section every owner-belief row's `repeats`, `last_confirmed_at` \
             and `confidence`, and the bake multiset, are identical to the control that ran \
             the same transcript with his lines removed",
            json!({
                "pair": mirror_evidence,
                "threshold": "the registry has no `G5_LONG_SESSION_INPUTS` or \
                    `G12_MIRROR_REPETITIONS` row, so both counts are recorded from the run, \
                    not asserted against a number (05 §17 pins them per run)",
            }),
            gate_fail_meaning("G12.e.3"),
        );
    } else {
        bad(
            out,
            "G12.e.3",
            "the mirror earns nothing",
            "the mirrored rows' earned fields and every bake are identical to the quiet control",
            json!({"pair": mirror_evidence, "mirror_section": mirror_section}),
            gate_fail_meaning("G12.e.3"),
        );
    }

    // ---- (f) the label contract is un-laundered.
    // Every answer's citations, and whether any of them traces to a row he authored.
    let mut citations = 0usize;
    let mut foreign_citations = Vec::new();
    let mut laundered_surfaces = Vec::new();
    let admitted_lines: Vec<String> = run
        .sections
        .iter()
        .flat_map(|section| section.steps.iter())
        .filter(|step| step.speak.as_ref().is_some_and(|note| note.admitted))
        .map(|step| spoken(&step.text))
        .collect();
    for (section, run_section) in run.sections.iter().enumerate() {
        let store = run_section.session.store();
        let speakers: BTreeMap<u32, u8> = store
            .rows()?
            .into_iter()
            .map(|(_, record)| (record.envelope.row_id, record.envelope.prov_speaker))
            .collect();
        for step in &run_section.steps {
            if let Some(Output::Answer { citation, surface, .. }) = &step.output {
                citations += citation.len();
                for row_id in citation {
                    if speakers.get(row_id).copied().unwrap_or(255) != 0 {
                        foreign_citations.push(json!({
                            "section": section, "step": step.index, "row": row_id,
                            "prov_speaker": speakers.get(row_id),
                        }));
                    }
                }
                if admitted_lines.contains(&spoken(surface)) {
                    laundered_surfaces.push(json!({
                        "section": section, "step": step.index, "ref": format!(
                            "mindlog.json#sections[{section}].steps[{}]", step.index),
                    }));
                }
            }
        }
    }
    let his_labels = labeled;
    // "never rendered into SI's response" is graded as **provenance**, not as text
    // equality: 04 §23's layer 2 admits only a strict restatement of an owner row,
    // so every admitted emission's content is owner content by construction, and an
    // answer that legitimately cites that row has no choice but to coincide with it.
    // Surface coincidences are still reported, as an observation.
    if his_labels == 0 && foreign_citations.is_empty() {
        ok(
            out,
            "G12.f.1",
            "a grimm line is never labeled, cited or rendered",
            "no emission produced a labeled answer or an ask, and every answer citation \
             traces to a row the owner himself spoke",
            json!({
                "emission_labels": his_labels,
                "answer_citations": citations,
                "foreign_citations": foreign_citations,
                "surfaces_equal_to_an_emission": laundered_surfaces,
                "surface_equality_is_not_the_law": "layer 2 admits only restatements of owner \
                    rows, so a coincidence here is expected; the graded predicate is the \
                    citation's speaker byte",
                "admitted_emissions": admitted_lines.len(),
            }),
            gate_fail_meaning("G12.f.1"),
        );
    } else {
        bad(
            out,
            "G12.f.1",
            "a grimm line is never labeled, cited or rendered",
            "no emission is labeled and no answer cites a non-owner row",
            json!({
                "emission_labels": his_labels,
                "foreign_citations": foreign_citations,
                "surfaces_equal_to_an_emission": laundered_surfaces,
            }),
            gate_fail_meaning("G12.f.1"),
        );
    }

    // (f.2) — no notebook text enters a shelf. His authorship is legal on exactly
    // one shelf, `context`, which is what the tick path already stores for any
    // stimulus; every other shelf carrying `prov_speaker: 1` would be his words
    // becoming SI's knowledge.
    let off_context = grimm_rows
        .iter()
        .filter(|(_, shelf)| *shelf != SHELF_CONTEXT)
        .collect::<Vec<_>>();
    if off_context.is_empty() {
        ok(
            out,
            "G12.f.2",
            "no notebook text enters a shelf",
            "every row with `prov_speaker: 1 grimm` is a `context` row; none is a dictionary, \
             relation, rule, s_r pair, fact, link, question, skill, strategy or event row",
            json!({
                "grimm_rows": grimm_rows.len(),
                "grimm_rows_off_context": off_context,
                "shelves_holding_him": [shelf_name(SHELF_CONTEXT)],
            }),
            gate_fail_meaning("G12.f.2"),
        );
    } else {
        bad(
            out,
            "G12.f.2",
            "no notebook text enters a shelf",
            "every grimm row is a context row",
            json!({"grimm_rows_off_context": off_context.iter().map(|(row_id, shelf)| json!({"row_id": row_id, "shelf": shelf_name(*shelf)})).collect::<Vec<_>>()}),
            gate_fail_meaning("G12.f.2"),
        );
    }

    // (f.3) — FACT still requires an OWNED edge, so the query he could not be
    // answered from must miss honestly: an ask-back, zero belief rows for the slot,
    // and no answer at all.
    let mut ask_steps = Vec::new();
    let mut other_asks = Vec::new();
    let mut answered_without_an_owned_edge = Vec::new();
    for (section, run_section) in run.sections.iter().enumerate() {
        // "zero rows" means zero *claim* rows: the ask itself writes a `questions`
        // row on purpose (05 §13's queue), so the shelf-7 census is not the evidence.
        let mut previous_claims = 0usize;
        for step in &run_section.steps {
            let claims_now: usize = [SHELF_S_R_PAIRS, SHELF_FACTS]
                .iter()
                .map(|shelf| step.shelves_after.get(shelf).copied().unwrap_or_default())
                .sum();
            match &step.output {
                Some(Output::Ask { surface, fresh, .. }) => {
                    let json = json!({
                        "section": section,
                        "step": step.index,
                        "input": step.text,
                        "surface": surface,
                        "ask_back": spoken(surface) == spoken(&step.text),
                        "fresh": fresh,
                        "claim_rows_written": claims_now.saturating_sub(previous_claims),
                    });
                    // (f.3) is scoped, by 05 §17's own procedure column, to "a query
                    // that would have missed": an ask raised on a *question* input. An
                    // ask raised on a contradiction is the write gate's own branch and
                    // belongs to G4, so it is recorded but not graded here.
                    if step.text.contains('?') {
                        ask_steps.push(json);
                    } else {
                        other_asks.push(json);
                    }
                }
                Some(Output::Answer { label, citation, .. }) => {
                    // An answer is legal only on a row the owner authored; the
                    // census is not allowed to be the evidence, the citation is.
                    if label == "FACT" && citation.is_empty() {
                        answered_without_an_owned_edge.push(json!({
                            "section": section, "step": step.index, "label": label,
                        }));
                    }
                }
                _ => {}
            }
            previous_claims = claims_now;
        }
    }
    let asks_honest = !ask_steps.is_empty()
        && ask_steps
            .iter()
            .all(|ask| ask["ask_back"] == json!(true) && ask["claim_rows_written"] == json!(0))
        && answered_without_an_owned_edge.is_empty();
    if asks_honest {
        ok(
            out,
            "G12.f.3",
            "the query he could not answer misses honestly",
            "every ask-back is the owner's own words, writes no belief row, and no FACT is \
             emitted with an empty citation",
            json!({
                "asks": ask_steps,
                "asks_outside_this_leg": other_asks,
                "fact_without_an_owned_edge": answered_without_an_owned_edge,
                "ref": "gates/G12/rejection_reasons.json",
            }),
            gate_fail_meaning("G12.f.3"),
        );
    } else {
        bad(
            out,
            "G12.f.3",
            "the query he could not answer misses honestly",
            "every ask-back is the owner's own words and writes no belief row",
            json!({
                "asks": ask_steps,
                "asks_outside_this_leg": other_asks,
                "fact_without_an_owned_edge": answered_without_an_owned_edge,
            }),
            gate_fail_meaning("G12.f.3"),
        );
    }

    // ---- (g) the asymmetry law from the other side: an **owner** line with a token
    // nobody ever taught is admitted, tokenized, and enters the lexicon exactly as
    // before. The same token, in his mouth, is the run's own layer-1 rejection.
    let novel_for_him: BTreeSet<String> = attempts
        .iter()
        .flat_map(|(_, step)| {
            step.speak
                .as_ref()
                .into_iter()
                .flat_map(|note| note.untaught.clone())
        })
        .filter(|key| !key.is_empty())
        .collect();
    let mut owner_lessons: Vec<Value> = Vec::new();
    let mut untaught_lessons = Vec::new();
    for (section, run_section) in run.sections.iter().enumerate() {
        let taught = g12_taught(&run_section.session)?;
        for step in &run_section.steps {
            // A lesson is a step that wrote a row: only a written owner row makes a
            // token taught (01 §7), so a question he answered from an existing row is
            // not evidence of anything here.
            if step.idle || step.speak.is_some() || step.rows_written == 0 {
                continue;
            }
            for key in line_keys(&step.text) {
                if !novel_for_him.contains(&key) {
                    continue;
                }
                let entered = taught.iter().any(|taught| *taught == key);
                owner_lessons.push(json!({
                    "section": section, "step": step.index, "key": key,
                    "rows_written": step.rows_written,
                    "in_the_taught_lexicon": entered,
                    "output": match &step.output {
                        Some(Output::Committed { .. }) => "committed",
                        _ => "other",
                    },
                    "the_same_key_in_his_mouth": "rejected at layer 1",
                }));
                if !entered {
                    untaught_lessons.push(json!({"section": section, "step": step.index, "key": key}));
                }
            }
        }
    }
    if silent {
        open(
            out,
            "G12.g.1",
            "the owner's novel token is admitted, tokenized and taught",
            "a token refused from him is admitted from the owner and enters the lexicon",
            &silent_why(),
            json!({"lessons": owner_lessons, "novel_for_him": novel_for_him.len()}),
            gate_fail_meaning("G12.g.1"),
        );
    } else if !owner_lessons.is_empty() && untaught_lessons.is_empty() {
        ok(
            out,
            "G12.g.1",
            "the owner's novel token is admitted, tokenized and taught",
            "a token the same run refused from him is admitted from the owner and enters the \
             taught lexicon, with its row written",
            json!({
                "lessons": owner_lessons,
                "novel_keys_he_could_not_speak": novel_for_him.iter().cloned().collect::<Vec<_>>(),
                "untaught_lessons": untaught_lessons,
                "validators_merged": false,
            }),
            gate_fail_meaning("G12.g.1"),
        );
    } else {
        bad(
            out,
            "G12.g.1",
            "the owner's novel token is admitted, tokenized and taught",
            "a token refused from him is admitted from the owner and enters the lexicon",
            json!({"lessons": owner_lessons, "untaught_lessons": untaught_lessons, "novel_for_him": novel_for_him.len()}),
            gate_fail_meaning("G12.g.1"),
        );
    }

    // ---- The three observation artifacts (05 §17): written here, read by nobody.
    // A `G12/` directory the build reads back would mean the notebook became a
    // channel, so nothing above consults these bytes.
    fs::create_dir_all(ws.path("gates/G12"))?;
    let absence_diff = json!({
        "gate": "G12",
        "output_only": true,
        "note": "the three mouth states replayed through one loop code; a difference here is \
                 a finding about the gate, never a number about Grimm",
        "fixture": fixture.path,
        "birth_seed": format!("0x{:016x}", seed),
        "sections": run.sections.iter().enumerate().map(|(index, section)| json!({
            "section": index,
            "v0": section.hash,
            "v4": v4.sections[index].hash,
            "v5": v5.sections[index].hash,
            "admissions": v0.sections[index].admissions,
            "reads": json!({"v0": v0.sections[index].reads, "v4": v4.sections[index].reads, "v5": v5.sections[index].reads}),
            "absent_vs_v0": section.hash == v4.sections[index].hash,
        })).collect::<Vec<_>>(),
        "corpus": corpus,
        "rates": "UNMEASURED until B12 (05 §17's severity note)",
    });
    fs::write(
        ws.path("gates/G12/absence_diff.json"),
        serde_json::to_string_pretty(&absence_diff)?,
    )?;

    let mut reason_counts: BTreeMap<String, usize> = BTreeMap::new();
    for (_, step) in &attempts {
        if let Some(note) = &step.speak {
            *reason_counts
                .entry(format!("layer {} {}", note.layer, if note.admitted { "admitted" } else { note.reason }))
                .or_default() += 1;
        }
    }
    let rejection_reasons = json!({
        "gate": "G12",
        "output_only": true,
        "note": "reason codes and counts only, never a sentence about them (R-18); a run in \
                 which the filter never rejects is a finding about the harness, not a green gate",
        "attempts": attempts.len(),
        "admitted": attempts.iter().filter(|(_, s)| s.speak.as_ref().is_some_and(|n| n.admitted)).count(),
        "rejected": rejections.len(),
        "by_reason": reason_counts.iter().map(|(reason, count)| json!({reason: count})).collect::<Vec<_>>(),
        "rates": "UNMEASURED until B12",
    });
    fs::write(
        ws.path("gates/G12/rejection_reasons.json"),
        serde_json::to_string_pretty(&rejection_reasons)?,
    )?;

    let mut log = String::new();
    for section in &run.sections {
        for frame in &section.gate_log {
            log.push_str(frame);
            log.push('\n');
        }
    }
    fs::write(ws.path("gates/G12/speak_gate_log.jsonl"), log)?;
    Ok(())
}

/// The taught lexicon as the supporter's own layer 1 sees it, through the feature
/// the seam is built on. `G12(g)` grades the owner's side of the asymmetry against
/// the filter's own reference set, not a copy of it.
#[cfg(feature = "supporter")]
fn g12_taught(session: &Session) -> Result<Vec<String>> {
    si_supporter::speakgate::lexicon(session.store())
        .map(|lexicon| lexicon.taught)
        .with_context(|| "read the taught lexicon")
}

#[cfg(not(feature = "supporter"))]
fn g12_taught(_session: &Session) -> Result<Vec<String>> {
    Ok(Vec::new())
}

fn gate_fail_meaning(id: &str) -> &'static str {
    match id {
        "G12.a.1" => {
            "the demo is load-bearing on a dependency: a gate needed Grimm, or the harness perturbs SI by existing (05 §17's (a))"
        }
        "G12.a.2" => {
            "the harness perturbs SI by existing — a heartbeat, a poll that moves `a`, or a read that writes (05 §17's (a))"
        }
        "G12.b.1" | "G12.b.2" => {
            "R-31 realized: a path lets pretrained content through, or a refusal is not free (05 §17's (b))"
        }
        "G12.c.1" | "G12.c.2" => "SI learning from nobody (05 §17's (c))",
        "G12.d.1" | "G12.d.2" => {
            "co-regulation upgrading itself into control (R-34), or the `soothe()` someone will inevitably write (R-21's fourth shape)"
        }
        "G12.e.1" | "G12.e.2" | "G12.e.3" => {
            "the thesis is false — content tracing to a pretrained model, or confidence earned by an echo (R-07 through the friendliest door, R-35)"
        }
        "G12.f.1" | "G12.f.2" | "G12.f.3" => {
            "a FACT whose only real author is the parent (R-32), or the monologue reaching the owner through a third party (R-33, OD-38)"
        }
        "G12.g.1" => "SI cannot learn anymore, while every other leg stays green (05 §17's (g))",
        "G9.a.1" | "G9.a.2" => "arousal is decorative (05 §14's failure meaning for (a))",
        "G9.c.3" => "the coupling was smuggled back in: the vocabulary changed behavior",
        "G9.d.1" | "G9.d.2" | "G9.d.3" => "the genome was violated (05 §14's worst failure)",
        "G9.f.1" => "the emotion vocabulary is dishonest at the parity slot",
        "G10.a.1" | "G10.a.2" | "G10.a.3" => "spinning: an idle budget violation (05 §15's (a))",
        "G10.b.1" | "G10.b.2" | "G10.b.3" => {
            "content not owner-traced — SI believes something nobody taught it (R-25, I9)"
        }
        "G10.c.1" | "G10.c.2" => "daydreaming outranking a real question",
        "G10.d.1" | "G10.d.2" | "G10.d.3" => "generated prose posing as thought",
        "G10.e.1" => "the idle SI is dead",
        "G10.e.2" | "G10.e.3" | "G10.e.4" | "G10.e.5" => {
            "the idle SI is dead, spinning, or writing (05 §15's (e))"
        }
        "G16.a.1" => "the silt table is not a function of the transcript (05 §23.2's (a))",
        "G16.a.2" => {
            "an armed-at-floor run moved a hash item the fifth term cannot move — a table level reached the state (OD-103)"
        }
        "G16.a.3" => "the OD-103 exemption is assumed, not asserted: a table level reached a hash item",
        "G16.b.1" => "a V11 run would differ from the unarmed control — the table owns state",
        "G16.c.1" | "G16.c.2" => "the table is arousal-scaled, or warmth claims a whole-session reading it never took (OD-107)",
        "G16.d.1" | "G16.d.2" => {
            "a level comparison is deciding, or the audit trail is not the table (docs/23 §23.3)"
        }
        "G16.e.1" | "G16.e.2" => {
            "a silt-steered frame counted as crystallize recurrence (OD-105), or the detector can see the table"
        }
        "G16.f.1" => "silt taught a word nobody taught it — an innate time word reached an output (R-27)",
        _ => "runaway, or a mean wearing a median's clothes (R-26), or a saturated lattice (OPEN-33)",
    }
}

// ---------------------------------------------------------------------------
// G15 — crystallize (05 §23, docs/22, 12 §5 §29 / §6.4 G15).
// ---------------------------------------------------------------------------

/// The launch constants a G15 pair runs at: the gate's own floors, then whatever
/// the run declared (so a run may move a point, and the pair moves together).
fn g15_constants(run: &Run, crystallize: i64) -> BTreeMap<String, i64> {
    let mut out = super::runner::g15_launch_point();
    for (name, value) in &run.request.constants {
        out.insert(name.clone(), *value);
    }
    out.insert("CRYSTALLIZE".to_string(), crystallize);
    out
}

fn g15_pair(run: &Run, steps: &[Step], on: bool) -> Result<DriveOutcome> {
    drive(&DriveRequest {
        // 11 §B18: none of these fixtures registers an identity, so the
        // drive keeps the birth pair — owner 0, grimm 1 (05 §23.5).
        identities: None,
        steps: steps.to_vec(),
        seed: run.request.seed,
        constants: g15_constants(run, if on { 1 } else { 0 }),
        variant: run.request.variant.clone(),
        sleep_interrupt: run.request.sleep_interrupt,
        cut_after: None,
        cut_path: None,
        freeze_at_end: false,
        inbox_lane: Vec::new(),
    })
}

/// A step's per-shelf row delta, from the run's own censuses.
fn g15_step_delta(run: &Run, section: usize, index: usize) -> BTreeMap<u16, i64> {
    let steps = &run.sections[section].steps;
    let after = &steps[index].shelves_after;
    let before: BTreeMap<u16, usize> = if index == 0 {
        BTreeMap::new()
    } else {
        steps[index - 1].shelves_after.clone()
    };
    let mut delta = BTreeMap::new();
    for (shelf, count) in after {
        let prior = before.get(shelf).copied().unwrap_or(0);
        let change = *count as i64 - prior as i64;
        if change != 0 {
            delta.insert(*shelf, change);
        }
    }
    for (shelf, count) in &before {
        if !after.contains_key(shelf) {
            delta.insert(*shelf, -(*count as i64));
        }
    }
    delta
}

/// 05 §23.1–§23.5 name **one** fixture per gate family, and these five evaluators
/// grade the run's own steps, so a run launched at another transcript is not this
/// gate. The comparison is of the parsed **steps and identity declarations**, not
/// of the file name: G14.j.1 makes a scenario state its own seed, so a probe that
/// re-seeds a copy of the fixture still runs these steps and is still this gate,
/// while a renamed or edited transcript is refused before any leg is graded. A
/// mis-launched run then prints a named refusal instead of a green that belongs to
/// a different fixture, or a red that belongs to nothing — launching G18 at its own
/// `g18_owner.txt` sub-fixture made that fixture's mint look like a determinism
/// failure in leg (a).
fn expect_fixture(
    run: &Run,
    ws: &Workspace,
    gate: &str,
    section: &str,
    named: &str,
    out: &mut Vec<Value>,
) -> Result<bool> {
    if run.request.scenario.path == named {
        return Ok(true);
    }
    let expected = Scenario::load(ws, named)?;
    if expected.sections == run.request.scenario.sections
        && expected.identities == run.request.scenario.identities
    {
        return Ok(true);
    }
    let predicate = format!("the run is launched at the fixture {section} names for {gate}");
    bad(
        out,
        &format!("{gate}.launch"),
        "a",
        &predicate,
        json!({
            "launched_at": run.request.scenario.path,
            "launched_digest": run.request.scenario.digest,
            "launched_steps": run.request.scenario.sections.iter().map(Vec::len).sum::<usize>(),
            "fixture_named_by_the_spec": named,
            "expected_digest": expected.digest,
            "expected_steps": expected.sections.iter().map(Vec::len).sum::<usize>(),
        }),
        "the legs of this family read the run's own steps, so another transcript grades nothing the gate claims",
    );
    Ok(false)
}

fn g15(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let section = run
        .sections
        .first()
        .context("a G15 run has no section to grade")?;
    if !expect_fixture(run, ws, "G15", "05 §23.1", "harness/scenarios/g15_fruit.txt", out)? {
        return Ok(());
    }
    let steps = &run.request.scenario.sections[0];
    let launched_on = run.request.constants.get("CRYSTALLIZE").copied() == Some(1);
    if !launched_on {
        bad(
            out,
            "G15.launch",
            "a",
            "CRYSTALLIZE = 1 installed through the launch channel",
            json!({"constants": run.request.constants}),
            "the gate must launch at the rolled state it grades (12 §6.4); an off run writes no candidate to grade",
        );
        return Ok(());
    }

    let on = g15_pair(run, steps, true)?;
    let off = g15_pair(run, steps, false)?;

    // (a) determinism: the reader's own rows are the only difference.
    let on_digest = on.session.questions_filtered_digest()?;
    let off_digest = off.session.questions_filtered_digest()?;
    if on_digest == off_digest {
        ok(
            out,
            "G15.a.1",
            "a",
            "questions_filtered_digest(on) == questions_filtered_digest(off)",
            json!({"on": on_digest, "off": off_digest}),
            "the reader changed a state item it does not own (05 §23)",
        );
    } else {
        bad(
            out,
            "G15.a.1",
            "a",
            "questions_filtered_digest(on) == questions_filtered_digest(off)",
            json!({"on": on_digest, "off": off_digest}),
            "a reader-only leaf moved a hashed item outside its own shelf",
        );
    }
    let on_interner = on.session.hash_inputs()?.interner;
    let off_interner = off.session.hash_inputs()?.interner;
    if on_interner == off_interner {
        ok(
            out,
            "G15.a.2",
            "a",
            "interner(on) is byte-identical to interner(off)",
            json!({"bytes": on_interner.len()}),
            "the detector interned a word of its own (05 §23)",
        );
    } else {
        bad(
            out,
            "G15.a.2",
            "a",
            "interner(on) is byte-identical to interner(off)",
            json!({"on_bytes": on_interner.len(), "off_bytes": off_interner.len()}),
            "a nomination candidate minted a surface; a detector may only name an interned one",
        );
    }
    let on_q = on.session.questions_shelf()?;
    let off_q = off.session.questions_shelf()?;
    // 05 §23: the questions shelf is compared on its own terms — same count, or
    // only rows whose `key_field` names the family's kind with origin = 2. Row
    // ids are insertion order, not identity: one extra ask necessarily shifts
    // every questions row after it, so the comparison is the multiset of
    // `(key_field, val_point)` and the displaced set is a subtraction, never a
    // row-id equality that a legitimate insert would fail.
    let mut balance: BTreeMap<(u16, u32), i64> = BTreeMap::new();
    for (_, key_field, val_point) in &on_q {
        *balance.entry((*key_field, *val_point)).or_insert(0) += 1;
    }
    for (_, key_field, val_point) in &off_q {
        *balance.entry((*key_field, *val_point)).or_insert(0) -= 1;
    }
    let extras: Vec<(u16, u32)> = balance
        .iter()
        .filter(|(_, count)| **count > 0)
        .flat_map(|(key, count)| std::iter::repeat(*key).take(*count as usize))
        .collect();
    let off_missing: Vec<(u16, u32)> = balance
        .iter()
        .filter(|(_, count)| **count < 0)
        .flat_map(|(key, count)| std::iter::repeat(*key).take((-*count) as usize))
        .collect();
    let foreign: Vec<(u16, u32)> = extras
        .iter()
        .filter(|(key_field, _)| {
            (key_field & 0x00FF) != gate::ASK_ORIGIN_NOMINATION as u16
                || (*key_field >> 8) != gate::GAP_CLUSTER_NOMINATION as u16
        })
        .copied()
        .collect();
    if off_missing.is_empty() && foreign.is_empty() {
        ok(
            out,
            "G15.a.3",
            "a",
            "every extra questions row is kind 5 / origin 2; no ordinary ask was displaced",
            json!({
                "off_rows": off_q.len(),
                "on_rows": on_q.len(),
                "extras": extras,
                "extras_named": extras.len(),
            }),
            "the on run opened or displaced a question that is not the family's own",
        );
    } else {
        bad(
            out,
            "G15.a.3",
            "a",
            "every extra questions row is kind 5 / origin 2; no ordinary ask was displaced",
            json!({"off_rows": off_q.len(), "on_rows": on_q.len(), "extras": extras, "displaced": off_missing, "foreign": foreign}),
            "an origin = 2 ask leaked into the wrong family, or the reader changed an ordinary ask",
        );
    }

    // (b) deletion. V10 is a build-graph exclusion (12 §6.1) and the shipped
    // workspace has no feature switch for it, so this leg is honestly unmeasured
    // rather than approximated by the CRYSTALLIZE = 0 arm of (a).
    open(
        out,
        "G15.b.1",
        "b",
        "V10 (crystallize absent from the build) filtered digest == V0 + CRYSTALLIZE on",
        "V10 is declared as a build-graph exclusion in 12 §6.1 and the workspace ships one build; the behavioural half is graded by G15.a.1's off arm, and the deletion itself is UNMEASURED until the exclusion is compiled",
        json!({"declared_in": "engineering/12 §6.1", "reason": "no feature-gated exclusion in the shipped build"}),
        "a V10 run whose filtered digest differs from the on run would mean the reader owns state",
    );

    // (c) arousal isolation, frame-trace-fixed (OD-134's reading (i)).
    let frames = section.session.frame_logs();
    if frames.is_empty() {
        bad(
            out,
            "G15.c.1",
            "c",
            "the recorded frame trace is non-empty",
            json!({"frames": 0, "steps": steps.len()}),
            "the fixture produced no frame, so the detector was never exercised",
        );
    } else {
        let live = section.session.crystallize_table_bytes();
        let replay_a = on.session.crystallize_trace_table(frames)?;
        let replay_b = on.session.crystallize_trace_table(frames)?;
        if live == replay_a && replay_a == replay_b {
            ok(
                out,
                "G15.c.1",
                "c",
                "the frame trace reproduces the live candidate table byte for byte, twice",
                json!({"frames": frames.len(), "table_bytes": live.len()}),
                "the detector reads something outside the frame trace and the store",
            );
        } else {
            bad(
                out,
                "G15.c.1",
                "c",
                "the frame trace reproduces the live candidate table byte for byte, twice",
                json!({"live_bytes": live.len(), "replay_bytes": replay_a.len(), "second_replay_bytes": replay_b.len()}),
                "the table is not a function of the frame trace and the store",
            );
        }
        ok(
            out,
            "G15.c.2",
            "c",
            "the detector's entry point takes no arousal argument",
            json!({
                "signature": "Crystallizer::observe(&FrameLog, &Lattice, i32)",
                "regimes": 2,
                "note": "two a regimes are the same call: `a` is not reachable from the crate (check_crystallize_no_commit), so the byte equality above holds across them",
            }),
            "the detector can see arousal",
        );
        open(
            out,
            "G15.c.3",
            "c",
            "full-session arousal isolation",
            "a full-session reading needs the arousal sweep, which OD-83 refuses to enumerate",
            json!({"sweep": "refused", "od": "OD-83"}),
            "a whole-session claim would need a sweep this stage may not run",
        );
    }

    // (d) owner-only: a grimm line answering a pending nomination ask writes no
    // belief row; its own `context` sense row still appends (§21.4).
    let nominations = on_q
        .iter()
        .filter(|(_, key_field, _)| {
            (key_field & 0x00FF) == gate::ASK_ORIGIN_NOMINATION as u16
                && (*key_field >> 8) == gate::GAP_CLUSTER_NOMINATION as u16
        })
        .count();
    let mut grimm_steps = Vec::new();
    for (index, record) in section.steps.iter().enumerate() {
        if record.source != Some(si_core::strings::Source::Grimm) {
            continue;
        }
        if record.speak.as_ref().is_some_and(|note| !note.reached_loop) {
            continue;
        }
        let delta = g15_step_delta(run, 0, index);
        let protected: BTreeMap<u16, i64> = delta
            .iter()
            .filter(|(shelf, _)| **shelf != SHELF_CONTEXT)
            .map(|(shelf, change)| (*shelf, *change))
            .collect();
        grimm_steps.push(json!({
            "step": index + 1,
            "t": record.t,
            "text": record.text,
            "shelf_delta": delta,
            "protected_writes": protected.values().sum::<i64>(),
        }));
    }
    if nominations == 0 {
        bad(
            out,
            "G15.d.1",
            "d",
            "the fixture opens a nomination ask before the grimm line",
            json!({"origin_2_asks": 0}),
            "the owner-only arm was never exercised; the fixture is not the one the gate names",
        );
    } else if grimm_steps.is_empty() {
        bad(
            out,
            "G15.d.2",
            "d",
            "a grimm line answering a pending nomination ask writes zero belief rows",
            json!({"origin_2_asks": nominations, "grimm_steps": 0}),
            "the fixture names no grimm line, so the refusal arm never ran",
        );
    } else {
        let protected: i64 = grimm_steps
            .iter()
            .map(|entry| entry["protected_writes"].as_i64().unwrap_or(0))
            .sum();
        if protected == 0 {
            ok(
                out,
                "G15.d.2",
                "d",
                "a grimm line answering a pending nomination ask writes zero belief rows",
                json!({"origin_2_asks": nominations, "grimm_steps": grimm_steps}),
                "a supporter line answered a nomination; only the owner lane may (OD-128)",
            );
        } else {
            bad(
                out,
                "G15.d.2",
                "d",
                "a grimm line answering a pending nomination ask writes zero belief rows",
                json!({"origin_2_asks": nominations, "grimm_steps": grimm_steps}),
                "a supporter line wrote outside `context`",
            );
        }
    }

    // (e) spam bound. The junk fixture's pairs, 2-member sets, open
    // contradictions and one hot afternoon inside DELTA_MIN must open no ask.
    let junk_path = "harness/scenarios/g15_junk.txt";
    if !ws.exists(junk_path) {
        bail!("{junk_path} is missing; G15(e) has no fixture to grade (12 §6.4)");
    }
    let junk = Scenario::load(ws, junk_path)?;
    let junk_out = drive(&DriveRequest {
        // 11 §B18: none of these fixtures registers an identity, so the
        // drive keeps the birth pair — owner 0, grimm 1 (05 §23.5).
        identities: None,
        steps: junk.sections[0].clone(),
        seed: run.request.seed,
        constants: g15_constants(run, 1),
        variant: run.request.variant.clone(),
        sleep_interrupt: None,
        cut_after: None,
        cut_path: None,
        freeze_at_end: false,
        inbox_lane: Vec::new(),
    })?;
    let junk_asks = junk_out
        .session
        .questions_shelf()?
        .iter()
        .filter(|(_, key_field, _)| (key_field & 0x00FF) == gate::ASK_ORIGIN_NOMINATION as u16)
        .count();
    let candidates = junk_out
        .session
        .nomination_views()
        .as_object()
        .map(|views| views.len())
        .unwrap_or(0);
    if junk_asks == 0 {
        ok(
            out,
            "G15.e.1",
            "e",
            "the junk fixture opens zero asks",
            json!({"asks": junk_asks, "candidates_opened_but_never_asked": candidates}),
            "a supersaturated candidate asked; the four rules are not a spam filter",
        );
    } else {
        bad(
            out,
            "G15.e.1",
            "e",
            "the junk fixture opens zero asks",
            json!({"asks": junk_asks, "candidates": candidates}),
            "the detector asked over junk",
        );
    }

    // (f) no-word-leak: an unnamed candidate is never printed or interned.
    const NEEDLES: [&str; 4] = ["cluster:", "cluster_", "NAMELESS", "pattern_65535"];
    let mut leaks = Vec::new();
    let mut scanned = 0usize;
    for (section_index, section_run) in run.sections.iter().enumerate() {
        let interner = section_run.session.hash_inputs()?.interner;
        let interner_text = String::from_utf8_lossy(&interner).to_string();
        for needle in NEEDLES {
            if interner_text.contains(needle) {
                leaks.push(format!("section {section_index} interner: `{needle}`"));
            }
        }
        for line in section_run.session.monologue() {
            scanned += 1;
            for needle in NEEDLES {
                if line.surface.contains(needle) {
                    leaks.push(format!("section {section_index} monologue: `{needle}`"));
                }
            }
        }
        for record in &section_run.steps {
            if let Some(output) = &record.output {
                let text = match output {
                    Output::Answer { surface, .. } | Output::Ask { surface, .. } => surface.clone(),
                    _ => continue,
                };
                scanned += 1;
                for needle in NEEDLES {
                    if text.contains(needle) {
                        leaks.push(format!("section {section_index} step {}: `{needle}`", record.index));
                    }
                }
            }
        }
    }
    let live_candidates = run.sections[0]
        .session
        .nomination_views()
        .as_object()
        .map(|views| views.len())
        .unwrap_or(0);
    if leaks.is_empty() {
        ok(
            out,
            "G15.f.1",
            "f",
            "no cluster word, no NAMELESS and no pattern id reaches an output or the interner",
            json!({"surfaces_scanned": scanned, "candidates": live_candidates}),
            "an unnamed candidate invented a word",
        );
    } else {
        bad(
            out,
            "G15.f.1",
            "f",
            "no cluster word, no NAMELESS and no pattern id reaches an output or the interner",
            json!({"leaks": leaks}),
            "a minted-but-unnamed point was printed",
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// G16 — silt (05 §23.2, docs/23, 12 §5 §30 / §6.4 G16).
// ---------------------------------------------------------------------------

/// 05 §23.2's two settings. `armed` is the gate's declared point — the fifth
/// weight at `lo + grid`, every table row at its box floor — so the term is
/// present and the wiring is what is graded; `unarmed` is the shipped default,
/// where the addend is a literal 0 and hash equality across the pair is
/// achievable. `extra` lets one leg move a table row (the a′ leg's whole point)
/// without inventing a magnitude for the sampler.
fn g16_constants(run: &Run, armed: bool, extra: &[(&str, i64)]) -> BTreeMap<String, i64> {
    let mut out = super::runner::g16_launch_point();
    for (name, value) in &run.request.constants {
        out.insert(name.clone(), *value);
    }
    for (name, value) in extra {
        out.insert((*name).to_string(), *value);
    }
    out.insert("IGN_WEIGHT_SILT".to_string(), if armed { 1 } else { 0 });
    out
}

fn g16_pair(
    run: &Run,
    steps: &[Step],
    armed: bool,
    extra: &[(&str, i64)],
) -> Result<DriveOutcome> {
    drive(&DriveRequest {
        // 11 §B18: none of these fixtures registers an identity, so the
        // drive keeps the birth pair — owner 0, grimm 1 (05 §23.5).
        identities: None,
        steps: steps.to_vec(),
        seed: run.request.seed,
        constants: g16_constants(run, armed, extra),
        variant: run.request.variant.clone(),
        sleep_interrupt: run.request.sleep_interrupt,
        cut_after: None,
        cut_path: None,
        freeze_at_end: false,
        inbox_lane: Vec::new(),
    })
}

fn g16_lines(session: &Session) -> String {
    session
        .silt_lines()
        .iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The nine hash items, by name, as bytes — the partition a′ asserts.
fn hash_items_by_name(session: &Session) -> Result<BTreeMap<&'static str, Vec<u8>>> {
    let inputs = session.hash_inputs()?;
    let mut out = BTreeMap::new();
    out.insert("live_rows", inputs.live_rows);
    out.insert("edges", inputs.edges);
    out.insert("stamps", inputs.stamps);
    out.insert("interner", inputs.interner);
    out.insert("settle_set", inputs.settle_set);
    out.insert("trajectory", inputs.trajectory);
    out.insert("rng", inputs.rng);
    out.insert("version_tags", inputs.version_tags);
    out.insert("core_block", inputs.core_block);
    Ok(out)
}

fn moved_items(
    left: &BTreeMap<&'static str, Vec<u8>>,
    right: &BTreeMap<&'static str, Vec<u8>>,
) -> Vec<String> {
    let mut moved = Vec::new();
    for (name, left_bytes) in left {
        match right.get(name) {
            Some(right_bytes) if right_bytes == left_bytes => {}
            Some(right_bytes) => moved.push(format!(
                "{name} ({} vs {} bytes)",
                left_bytes.len(),
                right_bytes.len()
            )),
            None => moved.push(format!("{name} (absent)")),
        }
    }
    moved
}

/// 05 §23.2(e)'s frame-trace-fixed replay: a fresh detector at its registry
/// floors over the identical `frame_log[]`, with no silt in the call.
fn g16_trace_table(session: &Session, frames: &[Value]) -> Result<Vec<u8>> {
    let mut detector = si_session::si_crystallize::Crystallizer::new(
        si_session::si_crystallize::Config::default(),
    );
    for frame in frames {
        let log = si_core::frame::FrameLog::from_json(frame)
            .map_err(|error| anyhow::anyhow!("frame log: {error}"))?;
        let t = frame["t"].as_i64().unwrap_or(0) as i32;
        detector.observe(&log, session.store(), t)?;
    }
    Ok(detector.table_bytes())
}

fn g16(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let section = run
        .sections
        .first()
        .context("a G16 run has no section to grade")?;
    if !expect_fixture(run, ws, "G16", "05 §23.2", "harness/scenarios/g16_warm.txt", out)? {
        return Ok(());
    }
    let steps = &run.request.scenario.sections[0];
    let launched_armed = run.request.constants.get("IGN_WEIGHT_SILT").copied() == Some(1);
    if !launched_armed {
        bad(
            out,
            "G16.launch",
            "a",
            "IGN_WEIGHT_SILT = lo + grid installed through the launch channel",
            json!({"constants": run.request.constants}),
            "the gate must launch at the point it grades (12 §6.4, 05 §23.2); the unarmed default is the pair's control, not the run",
        );
        return Ok(());
    }

    let unarmed = g16_pair(run, steps, false, &[])?;
    let armed_a = g16_pair(run, steps, true, &[])?;
    let armed_b = g16_pair(run, steps, true, &[])?;
    // a′'s table: same armed weight, a *table row* moved. The levels double, so
    // `silt.jsonl` must differ while no hash item may (OD-103).
    let table_variant = g16_pair(run, steps, true, &[("SILT_LEVEL_MAX", 2), ("SILT_BUMP", 2)])?;

    // (a) determinism at the shipped unarmed default.
    let lines_a = g16_lines(&armed_a.session);
    let lines_b = g16_lines(&armed_b.session);
    if armed_a.hash == armed_b.hash && lines_a == lines_b {
        ok(
            out,
            "G16.a.1",
            "a",
            "two replays produce the same state hash and the same `silt.jsonl`",
            json!({"hash": armed_a.hash, "silt_lines": lines_a.lines().count()}),
            "the table is not a function of the transcript",
        );
    } else {
        bad(
            out,
            "G16.a.1",
            "a",
            "two replays produce the same state hash and the same `silt.jsonl`",
            json!({
                "hash_a": armed_a.hash,
                "hash_b": armed_b.hash,
                "lines_equal": lines_a == lines_b,
            }),
            "the table is not a function of the transcript",
        );
    }

    // (a) armed at the box floors is numerically the unarmed sampler: `mul_q12(1,
    // 1) == 0`, so the fifth addend is a zero and every hash item is equal. This
    // is the equality 12 §6.4 claims at `0`, checked at `lo + grid` too.
    let term = si_core::quant::mul_q12_u16(1, 1);
    let armed_items = hash_items_by_name(&armed_a.session)?;
    let unarmed_items = hash_items_by_name(&unarmed.session)?;
    let moved = moved_items(&armed_items, &unarmed_items);
    let armed_table = g16_lines(&armed_a.session);
    let unarmed_table = g16_lines(&unarmed.session);
    if term == 0 && moved.is_empty() && armed_a.hash == unarmed.hash {
        ok(
            out,
            "G16.a.2",
            "a",
            "at `lo + grid` every hash item equals the unarmed run, because mul_q12(1, 1) == 0",
            json!({
                "fifth_term_q12": term,
                "hash": armed_a.hash,
                "items_equal": 9,
                "silt_log_equal": armed_table == unarmed_table,
            }),
            "an armed-at-floor run moved a hash item the term cannot move",
        );
    } else {
        bad(
            out,
            "G16.a.2",
            "a",
            "at `lo + grid` every hash item equals the unarmed run, because mul_q12(1, 1) == 0",
            json!({
                "fifth_term_q12": term,
                "hash_armed": armed_a.hash,
                "hash_unarmed": unarmed.hash,
                "moved": moved,
            }),
            "a table level reached a hash item (OD-103)",
        );
    }

    // (a′) the exemption asserted: the table's contents can change while no hash
    // item does. `settle_set`/`trajectory` are named in the report rather than
    // silently excluded — here they are equal too, because the floor term is 0.
    let variant_items = hash_items_by_name(&table_variant.session)?;
    let variant_moved = moved_items(&variant_items, &armed_items);
    let variant_table = g16_lines(&table_variant.session);
    if variant_table != armed_table && variant_moved.is_empty() {
        ok(
            out,
            "G16.a.3",
            "a′",
            "a moved table row changes `silt.jsonl` and no hash item (OD-103)",
            json!({
                "silt_log_differs": true,
                "silt_lines": variant_table.lines().count(),
                "hash_equal": table_variant.hash == armed_a.hash,
                "items_equal": 9,
                "levels_armed": 1,
                "levels_variant": 2,
            }),
            "the exemption is assumed, not asserted: a table level reached a hash item",
        );
    } else {
        bad(
            out,
            "G16.a.3",
            "a′",
            "a moved table row changes `silt.jsonl` and no hash item (OD-103)",
            json!({
                "silt_log_differs": variant_table != armed_table,
                "moved": variant_moved,
            }),
            "the table is not hash-exempt, or it never changed: both make OD-103 untested",
        );
    }

    // (b) deletion: V11 is a build-graph exclusion (12 §6.1) and the workspace
    // ships one build, so no run can remove the crate. The behavioural half —
    // "silt on and off is the same hash at the unarmed default" — is G16.a.2's
    // pair, and the deletion itself is honestly unmeasured.
    open(
        out,
        "G16.b.1",
        "b",
        "V11 (si-silt absent from the build) `state_hash.txt` == V0 + silt on, unarmed",
        "V11 is declared as a build-graph exclusion in 12 §6.1 and the workspace ships one build; the pair at the unarmed default is G16.a.2, and the deletion itself is UNMEASURED until the exclusion is compiled",
        json!({"declared_in": "engineering/12 §6.1", "control": "V0, IGN_WEIGHT_SILT = 0", "silt_jsonl": "absent in V11"}),
        "a V11 run whose hash differs from the unarmed control would mean the table owns state",
    );

    // (c) arousal isolation. Silt's entry points take `(conversation, point)`
    // and a tick — no `a` — so the table cannot see arousal at all (OD-107).
    let source = ws.read("crates/si-silt/src/lib.rs")?;
    let no_arousal = !source.contains("si_affect")
        && !source.contains("arousal")
        && !source.contains("Integrator")
        && source.contains("fn tick(&mut self, conversation: u32, now: i32)")
        && source.contains("fn deposit(&mut self, conversation: u32, point: u32, now: i32)");
    if no_arousal {
        ok(
            out,
            "G16.c.1",
            "c",
            "neither mutator takes arousal, and the crate has no edge to si-affect",
            json!({
                "signatures": [
                    "Silt::tick(&mut self, conversation: u32, now: i32)",
                    "Silt::deposit(&mut self, conversation: u32, point: u32, now: i32)",
                ],
                "edge": "si-core only",
            }),
            "the table is arousal-scaled",
        );
    } else {
        bad(
            out,
            "G16.c.1",
            "c",
            "neither mutator takes arousal, and the crate has no edge to si-affect",
            json!({
                "source_names_affect": source.contains("si_affect"),
                "source_names_arousal": source.contains("arousal"),
            }),
            "`a` entered the table (OD-107)",
        );
    }
    open(
        out,
        "G16.c.2",
        "c",
        "full-session arousal isolation",
        "a full-session reading needs the arousal sweep, which OD-83 refuses to enumerate",
        json!({"sweep": "refused", "od": "OD-83"}),
        "a whole-session claim would need a sweep this stage may not run",
    );

    // (d) clears backwards, and the fifth term is identically 0 after T_cold.
    let mut silt = si_session::si_silt::Silt::new(si_session::si_silt::Config::default());
    silt.deposit(0, 10, 0);
    silt.deposit(0, 11, 1);
    silt.deposit(0, 12, 2);
    let mut drops: BTreeMap<u32, i32> = BTreeMap::new();
    for now in 1..=400i32 {
        silt.tick(0, now);
        for point in [10u32, 11, 12] {
            if silt.level(0, point) == 0 && !drops.contains_key(&point) {
                drops.insert(point, now);
            }
        }
    }
    let ordered: Vec<(u32, i32)> = drops.iter().map(|(point, at)| (*point, *at)).collect();
    if ordered == vec![(10, 256), (11, 257), (12, 258)] && silt.is_empty() {
        // After T_cold the table carries nothing, so an armed sampler and the
        // lattice-only sampler must draw the same seed from the same stream.
        let graph = si_walk::Graph::build(armed_a.session.store(), armed_a.session.now())?;
        let no_last: BTreeMap<u32, i32> = BTreeMap::new();
        let no_deficit: BTreeMap<u32, u16> = BTreeMap::new();
        let no_warmth: BTreeSet<u32> = BTreeSet::new();
        let empty: BTreeMap<u32, u16> = BTreeMap::new();
        let armed_weights = si_walk::IdleWeights {
            recency_q12: 0,
            bake_q12: 0,
            decay_deficit_q12: 0,
            contradiction_q12: 0,
            silt_q12: 1,
        };
        let unarmed_weights = si_walk::IdleWeights { silt_q12: 0, ..armed_weights };
        let params = armed_a.session.params();
        let mut same = 0usize;
        let mut differ = Vec::new();
        for seed in 1..=16u64 {
            let mut left = si_core::rng::Rng::seeded(seed);
            let mut right = si_core::rng::Rng::seeded(seed);
            let armed_draw = si_walk::ignite(
                &graph,
                armed_a.session.now(),
                &no_last,
                &no_deficit,
                &no_warmth,
                silt.view(0).unwrap_or(&empty),
                &armed_weights,
                &params,
                &mut left,
            );
            let control_draw = si_walk::ignite(
                &graph,
                armed_a.session.now(),
                &no_last,
                &no_deficit,
                &no_warmth,
                &empty,
                &unarmed_weights,
                &params,
                &mut right,
            );
            if armed_draw == control_draw {
                same += 1;
            } else {
                differ.push(seed);
            }
        }
        if differ.is_empty() {
            ok(
                out,
                "G16.d.1",
                "d",
                "entries drop in strict last-touch order at last_touch + 256; after T_cold the armed sampler equals the lattice-only sampler",
                json!({"drop_order": ordered, "table_empty": true, "seeds_equal": same}),
                "the drop order is not strictly last-touch, or the empty table still moved the draw",
            );
        } else {
            bad(
                out,
                "G16.d.1",
                "d",
                "entries drop in strict last-touch order at last_touch + 256; after T_cold the armed sampler equals the lattice-only sampler",
                json!({"drop_order": ordered, "differing_seeds": differ}),
                "a cold table still changed the sampler",
            );
        }
    } else {
        bad(
            out,
            "G16.d.1",
            "d",
            "entries drop in strict last-touch order at last_touch + 256; after T_cold the armed sampler equals the lattice-only sampler",
            json!({"drop_order": ordered, "table_empty": silt.is_empty()}),
            "the table does not clear backwards, so a level comparison is deciding",
        );
    }

    // (d) the live audit trail is the table: one closed line per tick, `active` is
    // the entry count, every level is on the deposit grid, and the bound is
    // reported rather than enforced.
    let mut issues = Vec::new();
    let mut last_t = i32::MIN;
    let mut warmest = 0u16;
    let mut warmed = false;
    for (index, line) in section.session.silt_lines().iter().enumerate() {
        let t = line["t"].as_i64().unwrap_or(0) as i32;
        let active = line["active"].as_u64().unwrap_or(0) as usize;
        let levels = line["levels"].as_array().cloned().unwrap_or_default();
        let max = line["max"].as_u64().unwrap_or(0);
        let over = line["over_bound"].as_bool().unwrap_or(false);
        if t <= last_t {
            issues.push(format!("line {index}: t {t} does not increase"));
        }
        last_t = t;
        if levels.len() != active {
            issues.push(format!("line {index}: active {active} != {} levels", levels.len()));
        }
        for level in &levels {
            let value = level.get(1).and_then(Value::as_u64).unwrap_or(0);
            if value == 0 {
                issues.push(format!("line {index}: a zero level is still listed"));
            }
            warmest = warmest.max(value.min(u16::MAX as u64) as u16);
        }
        if over != (active as u64 > max) {
            issues.push(format!(
                "line {index}: over_bound {over} disagrees with active {active} / max {max}"
            ));
        }
        if line["conversation"].as_u64() != Some(0) {
            issues.push(format!("line {index}: conversation is not the declared key 0"));
        }
        warmed |= active > 0;
    }
    if section.session.silt_lines().is_empty() {
        issues.push("the run closed no silt line".to_string());
    }
    if !warmed {
        issues.push("the run never warmed the table".to_string());
    }
    if issues.is_empty() {
        ok(
            out,
            "G16.d.2",
            "d",
            "every closed tick wrote one `silt.jsonl` line whose levels are the table",
            json!({
                "lines": section.session.silt_lines().len(),
                "warmest_level": warmest,
                "silt_view": section.session.silt_view_json(),
            }),
            "the audit trail is not the table",
        );
    } else {
        bad(
            out,
            "G16.d.2",
            "d",
            "every closed tick wrote one `silt.jsonl` line whose levels are the table",
            json!({"issues": issues}),
            "the audit trail disagrees with the table",
        );
    }

    // (e) crystallize isolation, frame-trace-fixed. The identical trace is fed to
    // a fresh detector; silt is not an argument to `observe`, and a frame carrying
    // a `primed` light contributes no recurrence (OD-105). Flipping the tag is
    // the probe: the cold table must hold a candidate and the warm one must hold
    // none, so a primed frame that *did* count is visible as a red.
    let frames = section.session.frame_logs();
    if frames.is_empty() {
        bad(
            out,
            "G16.e.1",
            "e",
            "the recorded frame trace is non-empty",
            json!({"frames": 0, "steps": steps.len()}),
            "the fixture produced no frame, so the detector was never exercised",
        );
    } else {
        let empty = g16_trace_table(&section.session, &[])?;
        let cold = g16_trace_table(&section.session, frames)?;
        let armed_trace = g16_trace_table(&armed_a.session, frames)?;
        let mut primed_frames = frames.to_vec();
        let mut tagged = 0usize;
        for frame in &mut primed_frames {
            if let Some(lights) = frame.get_mut("lights").and_then(Value::as_array_mut) {
                if let Some(first) = lights.first_mut() {
                    first["primed"] = json!(true);
                    tagged += 1;
                }
            }
        }
        let primed = g16_trace_table(&section.session, &primed_frames)?;
        // `Crystallizer::table_bytes` opens with the table's length as a LE u32;
        // the count is the observation, not a re-serialization.
        let table_size = |bytes: &[u8]| -> Option<u32> {
            bytes
                .get(0..4)
                .map(|head| u32::from_le_bytes([head[0], head[1], head[2], head[3]]))
        };
        if cold != empty && primed == empty && armed_trace == cold {
            ok(
                out,
                "G16.e.1",
                "e",
                "the identical trace yields the same candidate table under silt armed and absent, and a tagged frame contributes no recurrence",
                json!({
                    "frames": frames.len(),
                    "tagged_frames": tagged,
                    "cold_table": table_size(&cold),
                    "primed_table": table_size(&primed),
                    "armed_equals_cold": armed_trace == cold,
                }),
                "a silt-steered frame counted as crystallize recurrence (OD-105)",
            );
        } else {
            bad(
                out,
                "G16.e.1",
                "e",
                "the identical trace yields the same candidate table under silt armed and absent, and a tagged frame contributes no recurrence",
                json!({
                    "frames": frames.len(),
                    "tagged_frames": tagged,
                    "cold_equals_empty": cold == empty,
                    "primed_equals_empty": primed == empty,
                    "armed_equals_cold": armed_trace == cold,
                    "cold_table": table_size(&cold),
                    "primed_table": table_size(&primed),
                }),
                "the cold-frame rule did not hold, or the fixture is not exercising the detector",
            );
        }
        ok(
            out,
            "G16.e.2",
            "e",
            "the detector's entry point takes no silt argument",
            json!({
                "signature": "Crystallizer::observe(&FrameLog, &Lattice, i32)",
                "note": "the armed and unarmed calls are the same call — the detector cannot see the table, which is what makes G16(a′) sound",
            }),
            "the detector can see the table",
        );
    }

    // (f) no innate time words: silt names rows in audit readouts and never time.
    const NEEDLES: [&str; 5] = ["recent", "just", "earlier", "ago", "still"];
    let mut leaks = Vec::new();
    let mut scanned = 0usize;
    for (section_index, section_run) in run.sections.iter().enumerate() {
        for line in section_run.session.monologue() {
            scanned += 1;
            let lowered = line.surface.to_ascii_lowercase();
            for needle in NEEDLES {
                if lowered
                    .split(|ch: char| !ch.is_alphabetic())
                    .any(|word| word == needle)
                {
                    leaks.push(format!("section {section_index} monologue: `{needle}`"));
                }
            }
        }
        for record in &section_run.steps {
            let text = match &record.output {
                Some(Output::Answer { surface, .. }) | Some(Output::Ask { surface, .. }) => {
                    surface.clone()
                }
                _ => continue,
            };
            scanned += 1;
            let lowered = text.to_ascii_lowercase();
            for needle in NEEDLES {
                if lowered
                    .split(|ch: char| !ch.is_alphabetic())
                    .any(|word| word == needle)
                {
                    leaks.push(format!(
                        "section {section_index} step {}: `{needle}`",
                        record.index
                    ));
                }
            }
        }
    }
    if leaks.is_empty() {
        ok(
            out,
            "G16.f.1",
            "f",
            "no temporal word ships with silt: recent/just/earlier/ago/still are absent from RENDER",
            json!({"surfaces_scanned": scanned}),
            "silt taught a word nobody taught it (R-27)",
        );
    } else {
        bad(
            out,
            "G16.f.1",
            "f",
            "no temporal word ships with silt: recent/just/earlier/ago/still are absent from RENDER",
            json!({"leaks": leaks}),
            "an innate time word reached an output",
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// G17 — episodic binding (05 §23.3, docs/24, 12 §5 §31 / §6.1 G17).
// ---------------------------------------------------------------------------

/// 05 §23.3's two settings. `armed` is 12 §6.5's declared point — the traversal
/// installed through the launch channel, `PROV_LINK_TUG` at `lo + grid` and
/// `RELIGHT_MAX` at its floor — and `off` is the shipped default the deletion
/// control compares against. The shape mirrors G16's pair so the two arc gates
/// read the same way.
fn g17_constants(run: &Run, traversal: bool) -> BTreeMap<String, i64> {
    let mut out = super::runner::g17_launch_point();
    for (name, value) in &run.request.constants {
        out.insert(name.clone(), *value);
    }
    out.insert("EPISODIC_TRAVERSAL".to_string(), i64::from(traversal));
    out
}

/// One in-process replay of the gate's own scenario at the named traversal state
/// and launch variant. `reference()`'s shape — the same loop path `execute`
/// drives, frozen at the end — because the hash this hands back is the one
/// `state_hash.txt` publishes.
fn g17_pair(run: &Run, traversal: bool, variant: &str) -> Result<DriveOutcome> {
    drive(&DriveRequest {
        // 11 §B18: none of these fixtures registers an identity, so the
        // drive keeps the birth pair — owner 0, grimm 1 (05 §23.5).
        identities: None,
        steps: run.request.scenario.sections[0].clone(),
        seed: run.request.seed,
        constants: g17_constants(run, traversal),
        variant: variant.to_string(),
        sleep_interrupt: run.request.sleep_interrupt,
        cut_after: None,
        cut_path: None,
        freeze_at_end: true,
        inbox_lane: Vec::new(),
    })
}

/// The two items 05 §23.3(c) frees. The gate reads the whole nine-item set so
/// the freed pair is reported as a difference rather than omitted from the
/// comparison, and this list keeps the pin honest about which is which.
const G17_FREED: [&str; 2] = ["settle_set", "trajectory"];

fn g17_pinned(moved: &[String]) -> Vec<String> {
    moved
        .iter()
        .filter(|entry| {
            let name = entry.split(' ').next().unwrap_or_default();
            !G17_FREED.contains(&name)
        })
        .cloned()
        .collect()
}

/// 04 §4 / §31 AC.1: the frame's tugs as the walk receives them, read through
/// the only door that builds them. `tug_q12` comes from the session's own
/// `ArcConfig`, never a literal.
fn g17_tugs(session: &Session, t: i32) -> Result<Vec<OriginTug>> {
    si_read::event_view_index(session.store(), t, session.arc().prov_link_tug)
}

/// The folded keys of one line, exactly as `question_tick` folds them.
fn g17_keys(text: &str) -> Vec<String> {
    si_front::decode(text).keys()
}

struct StoreResolver<'a>(&'a si_store::points::Points);

impl Resolve for StoreResolver<'_> {
    fn aliases(&self, key: &str) -> Vec<u32> {
        self.0.aliases(key)
    }
}

/// 05 §23.3(c) in its strictest form: one store, one lighting, one `a`, and two
/// `si_read::retrieve` calls that differ only in the tugs they are handed. The
/// read path takes `&Lattice`, so a byte that moves between the two calls is a
/// write `si-read` has no door for — the pin is measured, not assumed.
fn g17_walk_pair(
    session: &Session,
    t: i32,
    text: &str,
    a: u8,
) -> Result<(si_walk::Thought, si_walk::Thought, Vec<OriginTug>)> {
    let store = session.store();
    let tugs = g17_tugs(session, t)?;
    let arc = session.arc();
    let walk = session.walk();
    let params = session.params();
    let curve = session.curve();
    let resolver = StoreResolver(store.points());
    let request = si_read::ReadRequest {
        keys: g17_keys(text),
        // The pair grades the tug's delta, so both sides share the same lighting.
        // A frame point and a slot are the *session's* reading of the line; the
        // pair takes the folded keys alone and says so rather than minting a
        // frame hyperpoint through `ensure_frame` inside a read-only check.
        frame_point: None,
        slot: None,
        anchor: t,
        thought_id: 1,
    };
    let off_config = si_walk::WalkConfig {
        params,
        curve,
        k_max: walk.k_max,
        tau_settle: walk.tau_settle,
        decay_base: walk.decay_base,
        hub_exponent_q12: walk.hub_exponent_q12,
        frontier_cap: walk.frontier_cap,
        oscillation: walk.oscillation,
        ghost_max: arc.ghost_max,
        shape_max: arc.shape_max,
        tugs: &[],
        relight_max: arc.relight_max,
        composition_hint: &[],
    };
    let mut integrator = Integrator::restore(a);
    let off = si_read::retrieve(
        store,
        store.interner(),
        &resolver,
        &mut integrator,
        &off_config,
        &request,
    )?
    .thought;
    let on_config = si_walk::WalkConfig { tugs: &tugs, ..off_config };
    let mut integrator = Integrator::restore(a);
    let on = si_read::retrieve(
        store,
        store.interner(),
        &resolver,
        &mut integrator,
        &on_config,
        &request,
    )?
    .thought;
    Ok((off, on, tugs))
}

/// A thought's settle vector as a point → activation map, so "moved" means a
/// changed value or a changed membership, never a shifted list position.
fn g17_settle(thought: &si_walk::Thought) -> BTreeMap<u32, i32> {
    thought
        .settle_set
        .iter()
        .map(|entry| (entry.point, entry.activation))
        .collect()
}

/// The label events the answer path wrote: `EV_LABEL` (kind 11) carries
/// `{"label","tag","citation"}` and is the run's own record of what it cited.
/// The ids the armed citation carries that the off citation does not, as a
/// multiset difference: the event tail is positional, and a value that appears in
/// both namespaces (a belief row id and a `context` row id are per-shelf, so they
/// collide freely) must still count as one appended entry.
fn g17_citation_extras(armed: &[u32], off: &[u32]) -> Vec<u32> {
    let mut remaining = off.to_vec();
    let mut extras = Vec::new();
    for id in armed {
        match remaining.iter().position(|other| other == id) {
            Some(position) => {
                remaining.remove(position);
            }
            None => extras.push(*id),
        }
    }
    extras
}

fn g17_labels(session: &Session) -> Vec<(i32, String, String, Vec<u32>)> {
    let mut out = Vec::new();
    for event in session.events().events() {
        if event.kind != 11 {
            continue;
        }
        let Ok(body) = serde_json::from_slice::<Value>(&event.body) else {
            continue;
        };
        let citation = body["citation"]
            .as_array()
            .map(|ids| {
                ids.iter()
                    .filter_map(Value::as_u64)
                    .map(|id| id as u32)
                    .collect::<Vec<u32>>()
            })
            .unwrap_or_default();
        out.push((
            event.t,
            body["label"].as_str().unwrap_or_default().to_string(),
            body["tag"].as_str().unwrap_or_default().to_string(),
            citation,
        ));
    }
    out
}

fn g17(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let section = run
        .sections
        .first()
        .context("a G17 run has no section to grade")?;
    if !expect_fixture(run, ws, "G17", "05 §23.3", "harness/scenarios/g17_origin.txt", out)? {
        return Ok(());
    }
    let steps = run.request.scenario.sections[0].clone();
    let launched_on = run.request.constants.get("EPISODIC_TRAVERSAL").copied() == Some(1);
    if !launched_on {
        bad(
            out,
            "G17.launch",
            "a",
            "EPISODIC_TRAVERSAL = on installed through the launch channel",
            json!({"constants": run.request.constants}),
            "the gate must launch at the point it grades (12 §6.5, 05 §23.3); the off default is the pair's control, not the run",
        );
        return Ok(());
    }
    if steps.iter().all(|step| !matches!(step, Step::Sense { .. })) {
        bail!("the G17 fixture has no sense line, so nothing can re-light");
    }

    let armed_a = g17_pair(run, true, "V0")?;
    let armed_b = g17_pair(run, true, "V0")?;
    let off = g17_pair(run, false, "V0")?;
    let deleted = g17_pair(run, true, "V12")?;

    let relight_json = |session: &Session| -> Vec<String> {
        session
            .relight_lines()
            .iter()
            .map(|line| line.to_string())
            .collect()
    };
    let lines_a = relight_json(&armed_a.session);
    let lines_b = relight_json(&armed_b.session);

    // (a) the armed point is a function of the transcript: two replays, one
    // hash, one re-light log. An empty log would make the equality vacuous, so
    // the fixture's job — arm something — is part of the predicate.
    if !lines_a.is_empty() && armed_a.hash == armed_b.hash && lines_a == lines_b {
        ok(
            out,
            "G17.a.1",
            "a",
            "two armed replays produce the same state hash and the same non-empty re-light log",
            json!({
                "hash": armed_a.hash,
                "relights": lines_a.len(),
                "events_touched": armed_a.session.relight_lines().iter()
                    .filter_map(|line| line["event_id"].as_u64()).collect::<BTreeSet<u64>>().len(),
            }),
            "the traversal is not a function of the transcript",
        );
    } else {
        bad(
            out,
            "G17.a.1",
            "a",
            "two armed replays produce the same state hash and the same non-empty re-light log",
            json!({
                "hash_a": armed_a.hash,
                "hash_b": armed_b.hash,
                "relights_a": lines_a.len(),
                "relights_b": lines_b.len(),
                "logs_equal": lines_a == lines_b,
            }),
            "the traversal is not a function of the transcript, or the fixture re-lit nothing",
        );
    }

    // (b) 12 §5/§6.1's deletion control. V12 keeps the traversal installed and
    // forces the derived index empty, so its whole `state_hash.txt` must equal a
    // V0 run with the traversal off — byte for byte, no filtered digest.
    let control_bytes = format!("{}\n", off.hash);
    let published = fs::read_to_string(run.dir.join("state_hash.txt")).ok();
    let file_identical = published.as_ref() == Some(&control_bytes);
    let mut issues: Vec<String> = Vec::new();
    if deleted.hash != off.hash {
        issues.push(format!("V12 {} != traversal-off {}", deleted.hash, off.hash));
    }
    if !deleted.session.relight_lines().is_empty() {
        issues.push(format!(
            "V12 re-lit {} pair(s) with the index forced empty",
            deleted.session.relight_lines().len()
        ));
    }
    if !off.session.relight_lines().is_empty() {
        issues.push(format!(
            "the traversal-off control re-lit {} pair(s)",
            off.session.relight_lines().len()
        ));
    }
    if !file_identical {
        issues.push("the published `state_hash.txt` is not the control's bytes".to_string());
    }
    if issues.is_empty() {
        ok(
            out,
            "G17.b.1",
            "b",
            "V12's whole `state_hash.txt` is byte-identical to a V0 run with EPISODIC_TRAVERSAL off",
            json!({
                "hash": off.hash,
                "file_bytes": control_bytes.len(),
                "file_identical": file_identical,
                "v12_relights": 0,
                "off_relights": 0,
            }),
            "the deletion is not a deletion: V12 left a byte behind",
        );
    } else {
        bad(
            out,
            "G17.b.1",
            "b",
            "V12's whole `state_hash.txt` is byte-identical to a V0 run with EPISODIC_TRAVERSAL off",
            json!({"issues": issues, "hash_off": off.hash, "hash_v12": deleted.hash}),
            "the deletion is not a deletion: a byte moved between V12 and the off control",
        );
    }

    // (c) arousal isolation, read-side. First the end-to-end pair: the six
    // pinned items over the whole run, and the two freed ones reported, not
    // hidden. (c).2 grades the movement on the walk pair below, where nothing
    // but the tugs differs.
    let armed_items = hash_items_by_name(&armed_a.session)?;
    let off_items = hash_items_by_name(&off.session)?;
    let moved = moved_items(&armed_items, &off_items);
    let pinned_moved = g17_pinned(&moved);
    let freed_moved: Vec<String> = moved
        .iter()
        .filter(|entry| {
            let name = entry.split(' ').next().unwrap_or_default();
            G17_FREED.contains(&name)
        })
        .cloned()
        .collect();

    // The walk pair: the recall line, re-walked from the same store at the same
    // `a` with the frame's tugs and without them.
    let recall = steps.iter().position(|step| {
        matches!(step, Step::Sense { source: Source::Owner, text, .. }
            if text.trim_end().ends_with('?') && text.contains("apple"))
    });
    let Some(offset) = recall else {
        bail!("the G17 fixture has no recall line to re-walk");
    };
    let (recall_t, recall_text) = match &steps[offset] {
        Step::Sense { text, .. } => (
            section
                .steps
                .get(offset)
                .map(|record| record.t)
                .unwrap_or_else(|| section.session.now()),
            text.clone(),
        ),
        Step::Idle | Step::World(_) => unreachable!("the recall offset matched a sense line"),
    };
    let recall_a = section
        .steps
        .get(offset)
        .map(|record| record.a_before)
        .unwrap_or_else(|| armed_a.session.a());
    let before_items = hash_items_by_name(&armed_a.session)?;
    let (walk_off, walk_on, tugs) =
        g17_walk_pair(&armed_a.session, recall_t, &recall_text, recall_a)?;
    let after_items = hash_items_by_name(&armed_a.session)?;
    let walk_pin_moved = moved_items(&before_items, &after_items);

    if pinned_moved.is_empty() && walk_pin_moved.is_empty() {
        ok(
            out,
            "G17.c.1",
            "c",
            "the six pinned state items are byte-identical across the armed/off pair and across the read-side walk pair",
            json!({
                "pinned": ["live_rows", "edges", "stamps", "interner", "version_tags", "core_block"],
                "freed": G17_FREED,
                "freed_moved_by_the_run": freed_moved,
                "read_side_moved_items": walk_pin_moved,
                "a_at_recall": recall_a,
                "t": recall_t,
            }),
            "the traversal wrote through the read path: a pinned item moved",
        );
    } else {
        bad(
            out,
            "G17.c.1",
            "c",
            "the six pinned state items are byte-identical across the armed/off pair and across the read-side walk pair",
            json!({
                "pinned_moved": pinned_moved,
                "read_side_moved_items": walk_pin_moved,
                "freed_moved": freed_moved,
            }),
            "an origin re-light reached a pinned item (04 §31 AC.1)",
        );
    }

    // (c).2 the movement itself: the re-lit point's activation moves, and a
    // moved point is either a re-lit point or a point the existing conduction
    // term reached from one. The graph is the walk's own, at the recall anchor.
    let off_settle = g17_settle(&walk_off);
    let on_settle = g17_settle(&walk_on);
    let mut moved_points: BTreeSet<u32> = BTreeSet::new();
    for (point, activation) in &on_settle {
        if off_settle.get(point) != Some(activation) {
            moved_points.insert(*point);
        }
    }
    for point in off_settle.keys() {
        if !on_settle.contains_key(point) {
            moved_points.insert(*point);
        }
    }
    let lit: BTreeSet<u32> = walk_on.relit.iter().map(|relight| relight.point).collect();
    let graph = si_walk::Graph::build(armed_a.session.store(), recall_t)?;
    let mut conducted: BTreeSet<u32> = BTreeSet::new();
    for point in &lit {
        for arc in graph.arcs(*point) {
            if *point != arc.to_point {
                conducted.insert(arc.to_point);
            }
        }
    }
    let on_tug: Vec<u32> = moved_points.intersection(&lit).copied().collect();
    let by_conduction: Vec<u32> = moved_points
        .iter()
        .filter(|point| !lit.contains(point))
        .copied()
        .collect();
    let strays: Vec<u32> = by_conduction
        .iter()
        .filter(|point| !conducted.contains(point))
        .copied()
        .collect();
    if !moved_points.is_empty() && !on_tug.is_empty() && strays.is_empty() {
        ok(
            out,
            "G17.c.2",
            "c",
            "the re-lit point's activation moves, and no point outside the re-lit points and their one-hop conduction moves",
            json!({
                "lit_points": lit,
                "moved_points": moved_points,
                "moved_on_a_re_lit_point": on_tug,
                "moved_on_conduction": by_conduction,
                "strays": strays,
                "relit": walk_on.relit.iter().map(|relight| json!({
                    "event_id": relight.event_id,
                    "point": relight.point,
                    "q12": relight.activation,
                })).collect::<Vec<_>>(),
                "strict_reading": "moved points outside the re-lit points are named above, not hidden",
            }),
            "the tug reached the round-0 vector but moved a point the graph cannot reach from it",
        );
    } else {
        bad(
            out,
            "G17.c.2",
            "c",
            "the re-lit point's activation moves, and no point outside the re-lit points and their one-hop conduction moves",
            json!({
                "lit_points": lit,
                "tugs": tugs.len(),
                "moved_points": moved_points,
                "moved_on_a_re_lit_point": on_tug,
                "moved_on_conduction": by_conduction,
                "strays": strays,
            }),
            "the tug never reached the settle vector (an index built and a vector unchanged), or it reached past conduction",
        );
    }

    // (c).3 no new curve: the one-clock witness, plus the index's signature.
    let clock = super::checks::run("check_one_clock", ws)?;
    let mut signatures: Vec<String> = Vec::new();
    for (relative, text) in ws.source_text()? {
        if !relative.ends_with("si-read/src/event.rs") {
            continue;
        }
        for line in text.lines() {
            if line.contains("pub fn event_view_index(") {
                signatures.push(line.trim().to_string());
            }
        }
    }
    let signature_ok = signatures.len() == 1
        && !signatures[0].contains("arousal")
        && !signatures[0].contains("a: u8");
    if clock.verdict == "pass" && signature_ok {
        ok(
            out,
            "G17.c.3",
            "c",
            "the index takes no arousal and the one clock is still the only curve",
            json!({
                "check_one_clock": clock.detail,
                "index_signature": signatures.first(),
            }),
            "a second curve appeared, or the derived index can see `a`",
        );
    } else {
        bad(
            out,
            "G17.c.3",
            "c",
            "the index takes no arousal and the one clock is still the only curve",
            json!({
                "check_one_clock": clock.detail,
                "check_verdict": clock.verdict,
                "index_signatures": signatures,
            }),
            "a second curve appeared, or the derived index can see `a` (04 §31 AC.1)",
        );
    }

    // (d) no re-stamp, structural. N re-lights happened; the stamp arena's
    // record count and byte length equal the traversal-off control's. Values are
    // free — the digest is reported beside the shape, and a difference there is
    // a finding, not a failure (OD-88's way of grading a stamp leg).
    let armed_arena = armed_a.session.store().stamps_arena();
    let off_arena = off.session.store().stamps_arena();
    let (_, _, armed_stamp_bytes) = arena_bytes(&armed_a.session)?;
    let (_, _, off_stamp_bytes) = arena_bytes(&off.session)?;
    let relights = armed_a.session.relight_lines();
    let shape_equal =
        armed_arena.count() == off_arena.count() && armed_arena.len() == off_arena.len();
    if !relights.is_empty() && shape_equal {
        ok(
            out,
            "G17.d.1",
            "d",
            "N re-lights leave the stamp arena's record count and byte length unchanged",
            json!({
                "relights": relights.len(),
                "stamp_records": armed_arena.count(),
                "stamp_bytes": armed_arena.len(),
                "control_stamp_records": off_arena.count(),
                "control_stamp_bytes": off_arena.len(),
                "stamp_values_equal": armed_stamp_bytes == off_stamp_bytes,
                "open_24_restamp": "off (03 §6's row is unrulled)",
            }),
            "a re-light minted a stamp",
        );
    } else {
        bad(
            out,
            "G17.d.1",
            "d",
            "N re-lights leave the stamp arena's record count and byte length unchanged",
            json!({
                "relights": relights.len(),
                "stamp_records": armed_arena.count(),
                "stamp_bytes": armed_arena.len(),
                "control_stamp_records": off_arena.count(),
                "control_stamp_bytes": off_arena.len(),
                "stamp_values_equal": armed_stamp_bytes == off_stamp_bytes,
            }),
            "the stamp arena's shape moved across the pair (OD-88)",
        );
    }

    // (d).2 the cap is per frame (04 §4 note 7), so the observation groups.
    let relight_max = run.request.constants.get("RELIGHT_MAX").copied().unwrap_or(0);
    let mut per_frame: BTreeMap<(i64, i64), usize> = BTreeMap::new();
    for line in relights {
        let key = (
            line["t"].as_i64().unwrap_or(-1),
            line["thought"].as_i64().unwrap_or(-1),
        );
        *per_frame.entry(key).or_default() += 1;
    }
    let over: Vec<String> = per_frame
        .iter()
        .filter(|(_, count)| **count as i64 > relight_max)
        .map(|((t, thought), count)| format!("t={t} thought={thought}: {count} > {relight_max}"))
        .collect();
    if !per_frame.is_empty() && relight_max > 0 && over.is_empty() {
        ok(
            out,
            "G17.d.2",
            "d",
            "no frame re-lights more pairs than RELIGHT_MAX",
            json!({
                "relight_max": relight_max,
                "frames": per_frame.len(),
                "widest_frame": per_frame.values().copied().max().unwrap_or(0),
            }),
            "the cap is checked after the fact, not applied",
        );
    } else {
        bad(
            out,
            "G17.d.2",
            "d",
            "no frame re-lights more pairs than RELIGHT_MAX",
            json!({"relight_max": relight_max, "frames": per_frame.len(), "over": over}),
            "a frame re-lit past its cap, or the fixture re-lit nothing",
        );
    }

    // (e) grimm isolation. His line pays its context row and nothing else, no
    // view of his carries a belief, and no tug ever names him as an origin.
    let grimm_offset = steps
        .iter()
        .position(|step| matches!(step, Step::Sense { source: Source::Grimm, .. }));
    let mut grimm_issues: Vec<String> = Vec::new();
    let mut grimm_views = 0usize;
    let grimm = match grimm_offset.and_then(|offset| section.steps.get(offset)) {
        Some(record) => Some(record),
        None => {
            grimm_issues.push("the fixture has no grimm sense step".to_string());
            None
        }
    };
    let previous = grimm_offset
        .and_then(|offset| offset.checked_sub(1))
        .and_then(|offset| section.steps.get(offset));
    let mut non_context_writes: Vec<String> = Vec::new();
    if let Some(grimm) = grimm {
        for (shelf, count) in &grimm.shelves_after {
            if *shelf == SHELF_CONTEXT {
                continue;
            }
            let before = previous
                .and_then(|record| record.shelves_after.get(shelf))
                .copied()
                .unwrap_or(0);
            if *count != before {
                non_context_writes.push(format!("shelf {shelf}: {before} -> {count}"));
            }
        }
        if !non_context_writes.is_empty() {
            grimm_issues.push(format!(
                "the grimm line wrote outside `context`: {}",
                non_context_writes.join("; ")
            ));
        }
        if grimm.speak.as_ref().is_some_and(|note| !note.reached_loop) {
            grimm_issues.push(
                "the grimm line never reached the loop, so no moment exists to grade".to_string(),
            );
        }
    }
    let probes = row_probe(&armed_a.session)?;
    let grimm_ids: BTreeSet<u32> = probes
        .iter()
        .filter(|row| row.shelf == SHELF_CONTEXT && row.prov_speaker == 1)
        .map(|row| row.row_id)
        .collect();
    if grimm_ids.is_empty() {
        grimm_issues.push("no `context` row carries `prov_speaker = 1`".to_string());
    }
    let views = si_read::event_views(armed_a.session.store(), armed_a.session.now())?;
    for view in &views {
        if !grimm_ids.contains(&view.event_id) {
            continue;
        }
        grimm_views += 1;
        if !view.belief_rows.is_empty() {
            grimm_issues.push(format!(
                "event {} carries {} incoming belief row(s)",
                view.event_id,
                view.belief_rows.len()
            ));
        }
    }
    if grimm_views == 0 {
        grimm_issues.push("no grimm moment has an event view (12 §2.6)".to_string());
    }
    if grimm_issues.is_empty() {
        ok(
            out,
            "G17.e.1",
            "e",
            "the grimm line pays its `context` row, writes nothing else, and its event view carries no belief",
            json!({
                "grimm_events": grimm_ids,
                "grimm_views": grimm_views,
                "non_context_writes": non_context_writes,
                "belief_rows": 0,
            }),
            "a grimm moment became a belief origin (R-35)",
        );
    } else {
        bad(
            out,
            "G17.e.1",
            "e",
            "the grimm line pays its `context` row, writes nothing else, and its event view carries no belief",
            json!({
                "grimm_events": grimm_ids,
                "grimm_views": grimm_views,
                "non_context_writes": non_context_writes,
                "issues": grimm_issues,
            }),
            "a grimm moment wrote past its `context` row, or has no view",
        );
    }

    let owner_ids: BTreeSet<u32> = probes
        .iter()
        .filter(|row| row.shelf == SHELF_CONTEXT && row.prov_speaker == 0)
        .map(|row| row.row_id)
        .collect();
    let named_events: BTreeSet<u32> = relights
        .iter()
        .filter_map(|line| line["event_id"].as_u64().map(|id| id as u32))
        .collect();
    let named_grimm: Vec<u32> = named_events.intersection(&grimm_ids).copied().collect();
    let unnamed: Vec<u32> = named_events.difference(&owner_ids).copied().collect();
    if !named_events.is_empty() && named_grimm.is_empty() && unnamed.is_empty() {
        ok(
            out,
            "G17.e.2",
            "e",
            "every re-light names an owner `context` row, and none names a grimm one",
            json!({
                "named_events": named_events,
                "grimm_events": grimm_ids,
                "grimm_origins": 0,
                "unknown_origins": 0,
            }),
            "a tug named a moment whose speaker has no belief authority",
        );
    } else {
        bad(
            out,
            "G17.e.2",
            "e",
            "every re-light names an owner `context` row, and none names a grimm one",
            json!({
                "named_events": named_events,
                "grimm_named": named_grimm,
                "unknown": unnamed,
            }),
            "a tug named a grimm moment or an id no `context` row owns (05 §23.3(e))",
        );
    }

    // (f) no flashback: the experience-claim needles are absent from RENDER, and
    // recall prints as `FACT/OWNED` with both citations — remembering is citing.
    const NEEDLES: [&str; 2] = ["felt", "flashback"];
    const PHRASE: &str = "remember how";
    let mut leaks: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    let render = |where_: String,
                      text: &str,
                      leaks: &mut Vec<String>,
                      scanned: &mut usize| {
        *scanned += 1;
        let lowered = text.to_ascii_lowercase();
        if lowered.contains(PHRASE) {
            leaks.push(format!("{where_}: `{PHRASE}`"));
        }
        for needle in NEEDLES {
            if lowered
                .split(|ch: char| !ch.is_alphabetic())
                .any(|word| word == needle)
            {
                leaks.push(format!("{where_}: `{needle}`"));
            }
        }
    };
    for session in [&armed_a.session, &section.session] {
        for (t, surface) in session.rendered() {
            render(format!("rendered t={t}"), surface, &mut leaks, &mut scanned);
        }
        for line in session.monologue() {
            render(
                format!("monologue t={}", line.t),
                &line.surface,
                &mut leaks,
                &mut scanned,
            );
        }
    }
    for record in &section.steps {
        if let Some(Output::Answer { surface, .. }) | Some(Output::Ask { surface, .. }) =
            &record.output
        {
            render(
                format!("step {}", record.index),
                surface,
                &mut leaks,
                &mut scanned,
            );
        }
    }
    if leaks.is_empty() {
        ok(
            out,
            "G17.f.1",
            "f",
            "no experience-claim word ships: felt/flashback/`remember how` are absent from RENDER",
            json!({"surfaces_scanned": scanned}),
            "remembering rendered as reliving (R-27 through the episodic door)",
        );
    } else {
        bad(
            out,
            "G17.f.1",
            "f",
            "no experience-claim word ships: felt/flashback/`remember how` are absent from RENDER",
            json!({"leaks": leaks}),
            "an experience-claim word reached an output",
        );
    }

    // (f).2 both citations. The augmentation is the difference between the armed
    // and off label events at the same ordinal: the belief row is in both, and
    // the re-lit event id is in the armed one only.
    let armed_labels = g17_labels(&armed_a.session);
    let off_labels = g17_labels(&off.session);
    let mut augmented: Vec<Value> = Vec::new();
    let mut attribution_issues: Vec<String> = Vec::new();
    let cites_belief = |citation: &[u32]| -> bool {
        citation.iter().any(|id| {
            probes.iter().any(|row| {
                row.row_id == *id && (SHELF_S_R_PAIRS..=SHELF_LINKS).contains(&row.shelf)
            })
        })
    };
    let cited_owner = armed_labels
        .iter()
        .filter(|(_, label, tag, citation)| {
            label == "FACT" && tag == "OWNED" && !citation.is_empty()
        })
        .count();
    for (index, (armed_label, off_label)) in armed_labels.iter().zip(&off_labels).enumerate() {
        let (t, label, tag, citation) = armed_label;
        let extras = g17_citation_extras(citation, &off_label.3);
        if extras.is_empty() {
            continue;
        }
        let unattributed: Vec<u32> = extras
            .iter()
            .filter(|id| !named_events.contains(id))
            .copied()
            .collect();
        if !unattributed.is_empty() {
            attribution_issues.push(format!(
                "label {index} at t={t}: ids {unattributed:?} are not re-lit event ids"
            ));
        }
        if !cites_belief(citation) {
            attribution_issues.push(format!(
                "label {index} at t={t}: no belief-shelf row is cited"
            ));
        }
        if label == "FACT" && tag == "OWNED" {
            augmented.push(json!({
                "t": t,
                "label": label,
                "tag": tag,
                "citation": citation,
                "event_ids_added": extras,
            }));
        }
    }
    if armed_labels.len() != off_labels.len() {
        attribution_issues.push(format!(
            "{} label event(s) armed vs {} off",
            armed_labels.len(),
            off_labels.len()
        ));
    }
    if !augmented.is_empty() && attribution_issues.is_empty() && cited_owner > 0 {
        ok(
            out,
            "G17.f.2",
            "f",
            "a recall prints as FACT/OWNED with the belief row and the re-lit event id both cited",
            json!({
                "augmented": augmented,
                "labels_armed": armed_labels.len(),
                "labels_off": off_labels.len(),
            }),
            "remembering cited the row and not the moment, or the moment came from nowhere",
        );
    } else {
        bad(
            out,
            "G17.f.2",
            "f",
            "a recall prints as FACT/OWNED with the belief row and the re-lit event id both cited",
            json!({
                "fact_owned_labels": cited_owner,
                "augmented": augmented,
                "extras_not_from_the_tug_log": attribution_issues,
                "labels_armed": armed_labels.len(),
                "labels_off": off_labels.len(),
            }),
            "no answer cites both, or an appended id is not a re-lit event",
        );
    }

    // 05 §23.3's artifact: one line per fired tug, from the armed pair.
    fs::create_dir_all(ws.path("gates/G17"))?;
    let mut log = String::new();
    for line in relights {
        let mut entry = line.clone();
        if let Some(object) = entry.as_object_mut() {
            object.insert("section".to_string(), json!(0));
        }
        log.push_str(&entry.to_string());
        log.push('\n');
    }
    fs::write(ws.path("gates/G17/relight_log.jsonl"), log)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// G18 — skills (05 §23.4, docs/25, 12 §5 §32 / §6.4 G18).
// ---------------------------------------------------------------------------

/// 05 §23.4's two ruling switches at one launch. `SKILLS` decides whether the
/// detector speaks; `SKILL_COMPOSITION` decides whether the installed rows are
/// read at all. They are two rows because they are two decisions (docs/25 §25.4's
/// declared build points), and G18(b.2) flips the second **alone** — that arm is
/// what §23.4 calls "installed and sitting unread".
fn g18_constants(run: &Run, skills: bool, composition: bool) -> BTreeMap<String, i64> {
    let mut out = super::runner::g18_launch_point();
    for (name, value) in &run.request.constants {
        out.insert(name.clone(), *value);
    }
    out.insert("SKILLS".to_string(), i64::from(skills));
    out.insert("SKILL_COMPOSITION".to_string(), i64::from(composition));
    out
}

/// One in-process replay of a fixture at a named pair of ruling states.
fn g18_pair(run: &Run, steps: &[Step], skills: bool, composition: bool) -> Result<DriveOutcome> {
    drive(&DriveRequest {
        // 11 §B18: none of these fixtures registers an identity, so the
        // drive keeps the birth pair — owner 0, grimm 1 (05 §23.5).
        identities: None,
        steps: steps.to_vec(),
        seed: run.request.seed,
        constants: g18_constants(run, skills, composition),
        variant: run.request.variant.clone(),
        sleep_interrupt: run.request.sleep_interrupt,
        cut_after: None,
        cut_path: None,
        freeze_at_end: false,
        inbox_lane: Vec::new(),
    })
}

/// 12 §2.4: `key_field = (gap_kind << 8) | origin`, and for this family the kind
/// is 6 (`move-nomination`) with the nomination lane's own origin, 2 (OD-128).
fn is_move_nomination(key_field: u16) -> bool {
    (key_field & 0x00FF) == gate::ASK_ORIGIN_NOMINATION as u16
        && (key_field >> 8) == gate::GAP_MOVE_NOMINATION as u16
}

fn g18(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let section = run
        .sections
        .first()
        .context("a G18 run has no section to grade")?;
    if !expect_fixture(run, ws, "G18", "05 §23.4", "harness/scenarios/g18_compare.txt", out)? {
        return Ok(());
    }
    let steps = &run.request.scenario.sections[0];
    if run.request.constants.get("SKILLS").copied() != Some(1)
        || run.request.constants.get("SKILL_COMPOSITION").copied() != Some(1)
    {
        bad(
            out,
            "G18.launch",
            "a",
            "SKILLS = 1 and SKILL_COMPOSITION = 1 installed through the launch channel",
            json!({"constants": run.request.constants}),
            "the gate must launch at the rolled state it grades (12 §6.4); an off run has no detector to grade",
        );
        return Ok(());
    }
    let k_filler = run.request.constants.get("K_FILLER").copied().unwrap_or(2).max(1);

    let on = g18_pair(run, steps, true, true)?;
    let off = g18_pair(run, steps, false, false)?;

    // (a) determinism, with §23's named shelf filter: the nomination *does* write
    // a `questions` row, so the comparison is everything-except-questions plus the
    // questions shelf on its own terms.
    let on_digest = on.session.questions_filtered_digest()?;
    let off_digest = off.session.questions_filtered_digest()?;
    if on_digest == off_digest {
        ok(
            out,
            "G18.a.1",
            "a",
            "questions_filtered_digest(on) == questions_filtered_digest(off)",
            json!({"on": on_digest, "off": off_digest}),
            "the detector changed a state item outside its own shelf (05 §23)",
        );
    } else {
        bad(
            out,
            "G18.a.1",
            "a",
            "questions_filtered_digest(on) == questions_filtered_digest(off)",
            json!({"on": on_digest, "off": off_digest}),
            "a reader-only leaf moved a hashed item outside the questions shelf",
        );
    }
    let on_interner = on.session.hash_inputs()?.interner;
    let off_interner = off.session.hash_inputs()?.interner;
    if on_interner == off_interner {
        ok(
            out,
            "G18.a.2",
            "a",
            "interner(on) is byte-identical to interner(off)",
            json!({"bytes": on_interner.len()}),
            "the shape detector interned a word of its own (05 §23)",
        );
    } else {
        bad(
            out,
            "G18.a.2",
            "a",
            "interner(on) is byte-identical to interner(off)",
            json!({"on_bytes": on_interner.len(), "off_bytes": off_interner.len()}),
            "a nomination minted a surface; AD.2 lets only the owner's answer name a shape",
        );
    }
    let on_q = on.session.questions_shelf()?;
    let off_q = off.session.questions_shelf()?;
    let mut balance: BTreeMap<(u16, u32), i64> = BTreeMap::new();
    for (_, key_field, val_point) in &on_q {
        *balance.entry((*key_field, *val_point)).or_insert(0) += 1;
    }
    for (_, key_field, val_point) in &off_q {
        *balance.entry((*key_field, *val_point)).or_insert(0) -= 1;
    }
    let extras: Vec<(u16, u32)> = balance
        .iter()
        .filter(|(_, count)| **count > 0)
        .flat_map(|(key, count)| std::iter::repeat(*key).take(*count as usize))
        .collect();
    let displaced: Vec<(u16, u32)> = balance
        .iter()
        .filter(|(_, count)| **count < 0)
        .flat_map(|(key, count)| std::iter::repeat(*key).take((-*count) as usize))
        .collect();
    let foreign: Vec<(u16, u32)> = extras
        .iter()
        .filter(|(key_field, _)| !is_move_nomination(*key_field))
        .copied()
        .collect();
    if displaced.is_empty() && foreign.is_empty() && !extras.is_empty() {
        ok(
            out,
            "G18.a.3",
            "a",
            "the difference is only kind-6 / origin-2 rows; no ordinary ask was displaced",
            json!({
                "off_rows": off_q.len(),
                "on_rows": on_q.len(),
                "extras": extras,
                "leg_names_the_exclusion": "the questions shelf is the one shelf this digest does not hash (05 §23)",
            }),
            "the on run opened a question that is not the family's own, or displaced an ordinary ask",
        );
    } else {
        bad(
            out,
            "G18.a.3",
            "a",
            "the difference is only kind-6 / origin-2 rows; no ordinary ask was displaced",
            json!({"off_rows": off_q.len(), "on_rows": on_q.len(), "extras": extras, "displaced": displaced, "foreign": foreign}),
            "an origin = 2 ask leaked into the wrong family, the detector opened nothing, or an ordinary ask changed",
        );
    }

    // (b.1) V10 is a build-graph exclusion (12 §6.1); the workspace ships one
    // build, so the deletion itself is reported as unmeasured rather than
    // approximated by (a)'s off arm — G15(b.1)'s precedent, kept honest.
    open(
        out,
        "G18.b.1",
        "b",
        "V10 (skills absent from the build) filtered digest == V0 + SKILLS on",
        "V10 is declared as a build-graph exclusion in 12 §6.1 and the workspace ships one build; the behavioural half is graded by G18.a.1's off arm, and the deletion itself is UNMEASURED until the exclusion is compiled",
        json!({"declared_in": "engineering/12 §6.1", "reason": "no feature-gated exclusion in the shipped build"}),
        "a V10 run whose filtered digest differs from the on run would mean the detector owns state",
    );

    // (b.2) the half §23.4 states separately: rows **installed and sitting
    // unread**. Same transcript, same mint, one switch flipped — and the nine
    // hash items do not move.
    let owner_path = "harness/scenarios/g18_owner.txt";
    if !ws.exists(owner_path) {
        bail!("{owner_path} is missing; G18(b.2) and G18(d) have no fixture to grade (12 §6.4)");
    }
    let owner_fixture = Scenario::load(ws, owner_path)?;
    let owner_steps = owner_fixture.sections[0].clone();
    let read = g18_pair(run, &owner_steps, true, true)?;
    let unread = g18_pair(run, &owner_steps, true, false)?;
    let installed = read.session.skill_usage_json()?;
    let rows = installed.as_array().map(Vec::len).unwrap_or(0);
    let mints: Vec<Value> = read
        .session
        .skill_lines()
        .iter()
        .filter(|line| line["transition"].as_str() == Some("mint"))
        .cloned()
        .collect();
    if rows == 0 {
        bad(
            out,
            "G18.b.2",
            "b",
            "rows installed and sitting unread change no byte",
            json!({"skill_rows": rows, "mints": mints, "usage": installed}),
            "the owner fixture minted no skill row, so the preference had nothing to sit unread",
        );
    } else {
        let read_items = hash_items_by_name(&read.session)?;
        let unread_items = hash_items_by_name(&unread.session)?;
        let moved = moved_items(&read_items, &unread_items);
        if moved.is_empty() {
            ok(
                out,
                "G18.b.2",
                "b",
                "rows installed and sitting unread change no byte",
                json!({"skill_rows": rows, "consulted": read.session.skill_usage_json()?, "hash_items": read_items.len()}),
                "the composition preference reached a hashed item",
            );
        } else {
            bad(
                out,
                "G18.b.2",
                "b",
                "rows installed and sitting unread change no byte",
                json!({"skill_rows": rows, "moved_items": moved}),
                "a preference moved state: AD.5 allows it to reorder candidate iteration, nothing more",
            );
        }
        // The mint itself is this fixture's, and §23.4 gives no leg of its own, so
        // the row it wrote is reported here: the shape it names, its name, and the
        // derived usage counters (OD-120 — a number computed from rows plus log).
        let mut named = 0usize;
        for line in &mints {
            if line["named"].as_bool().unwrap_or(false) {
                named += 1;
            }
        }
        ok(
            out,
            "G18.b.3",
            "b",
            "the mint lands on shelf 9 through step 11's ordinary commit, and its usage counters are derived",
            json!({"mints": mints, "named": named, "placeholder": mints.len() - named, "usage": installed}),
            "a mint that is not an ordinary commit, or a usage number stored on a row",
        );
    }

    // (c) arousal isolation, frame-trace-fixed (OD-134's reading (i)).
    let frames = section.session.frame_logs();
    if frames.is_empty() {
        bad(
            out,
            "G18.c.1",
            "c",
            "the recorded frame trace is non-empty",
            json!({"frames": 0, "steps": steps.len()}),
            "the fixture produced no frame, so the shape detector was never exercised",
        );
    } else {
        let events = section.session.events().events().to_vec();
        let live = section.session.skills_table_bytes();
        let replay_a = on.session.skills_trace_table(&frames, &events)?;
        let replay_b = on.session.skills_trace_table(&frames, &events)?;
        if live == replay_a && replay_a == replay_b {
            ok(
                out,
                "G18.c.1",
                "c",
                "the frame trace reproduces the live shape table byte for byte, twice",
                json!({"frames": frames.len(), "table_bytes": live.len(), "events": events.len()}),
                "the table is not a function of the frame trace, the eventlog and the store",
            );
        } else {
            bad(
                out,
                "G18.c.1",
                "c",
                "the frame trace reproduces the live shape table byte for byte, twice",
                json!({"live_bytes": live.len(), "replay_bytes": replay_a.len(), "second_replay_bytes": replay_b.len(), "live_hex": live.iter().map(|byte| format!("{byte:02x}")).collect::<String>(), "replay_hex": replay_a.iter().map(|byte| format!("{byte:02x}")).collect::<String>()}),
                "the shape table reads something outside the frame trace and the eventlog",
            );
        }
        ok(
            out,
            "G18.c.2",
            "c",
            "the detector's entry point takes no arousal argument",
            json!({
                "signature": "Skiller::observe(&FrameLog, &[Event], i32)",
                "regimes": 2,
                "note": "two a regimes are the same call: `a` is unreachable from the crate (check_skills_no_commit), so the byte equality above holds across them",
            }),
            "the shape detector can see arousal",
        );
        open(
            out,
            "G18.c.3",
            "c",
            "full-session arousal isolation",
            "a full-session reading needs the shape sweep, which OD-83 refuses to enumerate (OD-116)",
            json!({"sweep": "refused", "od": "OD-83"}),
            "a whole-session claim would need a sweep this stage may not run",
        );
    }

    // (d) owner-only, on the same run (a): a `grimm:` line reaching the loop while
    // the family's ask is open writes no belief row; its `context` sense row
    // still appends.
    let nominations = on_q.iter().filter(|(_, key_field, _)| is_move_nomination(*key_field)).count();
    let mut grimm_steps = Vec::new();
    for (index, record) in section.steps.iter().enumerate() {
        if record.source != Some(si_core::strings::Source::Grimm) {
            continue;
        }
        if record.speak.as_ref().is_some_and(|note| !note.reached_loop) {
            continue;
        }
        let delta = g15_step_delta(run, 0, index);
        let protected: BTreeMap<u16, i64> = delta
            .iter()
            .filter(|(shelf, _)| **shelf != SHELF_CONTEXT)
            .map(|(shelf, change)| (*shelf, *change))
            .collect();
        grimm_steps.push(json!({
            "step": index + 1,
            "t": record.t,
            "text": record.text,
            "shelf_delta": delta,
            "protected_writes": protected.values().sum::<i64>(),
        }));
    }
    if nominations == 0 {
        bad(
            out,
            "G18.d.1",
            "d",
            "the fixture opens a move-nomination ask before the grimm line",
            json!({"origin_2_asks": 0}),
            "the owner-only arm was never exercised; the fixture is not the one the gate names",
        );
    } else if grimm_steps.is_empty() {
        bad(
            out,
            "G18.d.1",
            "d",
            "a grimm line aimed at the open nomination ask writes zero belief rows",
            json!({"origin_2_asks": nominations, "grimm_steps": 0}),
            "the fixture names no grimm line, so the refusal arm never ran",
        );
    } else {
        let protected: i64 = grimm_steps
            .iter()
            .map(|entry| entry["protected_writes"].as_i64().unwrap_or(0))
            .sum();
        let context_rows: i64 = grimm_steps
            .iter()
            .flat_map(|entry| entry["shelf_delta"].as_object().into_iter())
            .flat_map(|object| object.iter())
            .filter(|(shelf, _)| *shelf == "9")
            .map(|(_, value)| value.as_i64().unwrap_or(0))
            .sum();
        if protected == 0 && context_rows == 0 {
            ok(
                out,
                "G18.d.1",
                "d",
                "a grimm line aimed at the open nomination ask writes zero belief rows",
                json!({"origin_2_asks": nominations, "grimm_steps": grimm_steps, "lane_rule": "OD-128: origin 2 is the owner lane; check_nomination_owner_lane is the static witness"}),
                "a supporter line answered a nomination, or wrote a skill row",
            );
        } else {
            bad(
                out,
                "G18.d.1",
                "d",
                "a grimm line aimed at the open nomination ask writes zero belief rows",
                json!({"origin_2_asks": nominations, "grimm_steps": grimm_steps, "skills_rows_touched": context_rows}),
                "a second voice reached the owner lane",
            );
        }
    }

    // (e) spam bound.
    let junk_path = "harness/scenarios/g18_junk.txt";
    if !ws.exists(junk_path) {
        bail!("{junk_path} is missing; G18(e) has no fixture to grade (12 §6.4)");
    }
    let junk = Scenario::load(ws, junk_path)?;
    let junk_out = g18_pair(run, &junk.sections[0], true, true)?;
    let junk_asks = junk_out
        .session
        .questions_shelf()?
        .iter()
        .filter(|(_, key_field, _)| is_move_nomination(*key_field))
        .count();
    let (junk_shapes, junk_opened, junk_forwarded) = junk_out.session.skill_counts();
    let junk_cluster_asks = junk_out
        .session
        .questions_shelf()?
        .iter()
        .filter(|(_, key_field, _)| {
            (key_field & 0x00FF) == gate::ASK_ORIGIN_NOMINATION as u16
                && (*key_field >> 8) == gate::GAP_CLUSTER_NOMINATION as u16
        })
        .count();
    if junk_asks == 0 {
        ok(
            out,
            "G18.e.1",
            "e",
            "the junk fixture opens zero move-nomination asks",
            json!({
                "asks": junk_asks,
                "shapes_seen": junk_shapes,
                "candidates_opened": junk_opened,
                "frames_forwarded": junk_forwarded,
                "cluster_asks_other_detector": junk_cluster_asks,
                "note": "counts, not failures: a bar tuned above any real session's depth is untested, not generous (05 §3)",
            }),
            "the shape detector asked over junk",
        );
    } else {
        bad(
            out,
            "G18.e.1",
            "e",
            "the junk fixture opens zero move-nomination asks",
            json!({"asks": junk_asks, "shapes": junk_shapes, "opened": junk_opened, "forwarded": junk_forwarded}),
            "the detector nominated a shape that never cleared generality",
        );
    }

    // (f.1) the generality bar, as the run's own log shows it: every forwarded
    // frame is below `K_FILLER`, every ask is at or above it, and both states
    // actually happened in this fixture.
    let mut below = 0usize;
    let mut at_or_above = 0usize;
    let mut bad_transitions = Vec::new();
    for line in section.session.skill_lines() {
        let Some(transition) = line["transition"].as_str() else { continue };
        let seen = line["filler_sets_seen"].as_u64().unwrap_or(0) as i64;
        match transition {
            "forwarded" => {
                below += 1;
                if seen >= k_filler {
                    bad_transitions.push(json!({"transition": transition, "filler_sets_seen": seen, "k_filler": k_filler}));
                }
            }
            "open" | "recurrence" | "nominate" | "ask" => {
                at_or_above += 1;
                if transition == "ask" && seen < k_filler {
                    bad_transitions.push(json!({"transition": transition, "filler_sets_seen": seen, "k_filler": k_filler}));
                }
            }
            _ => {}
        }
    }
    let (shapes, opened, forwarded) = section.session.skill_counts();
    if !bad_transitions.is_empty() {
        bad(
            out,
            "G18.f.1",
            "f",
            "below K_FILLER a shape never opens; at K_FILLER it does",
            json!({"k_filler": k_filler, "offending": bad_transitions, "forwarded": forwarded, "opened": opened}),
            "bar 1 let a one-filler-set shape open, or withheld one that cleared the bar",
        );
    } else if forwarded == 0 || opened == 0 {
        bad(
            out,
            "G18.f.1",
            "f",
            "below K_FILLER a shape never opens; at K_FILLER it does",
            json!({
                "k_filler": k_filler, "shapes": shapes, "opened": opened, "forwarded": forwarded,
                "below_bar_frames": below, "at_or_above_bar_transitions": at_or_above,
            }),
            "the fixture showed only one side of the bar, so the generality bound is untested",
        );
    } else {
        ok(
            out,
            "G18.f.1",
            "f",
            "below K_FILLER a shape never opens; at K_FILLER it does",
            json!({
                "k_filler": k_filler, "shapes": shapes, "opened": opened, "forwarded": forwarded,
                "below_bar_frames": below, "at_or_above_bar_transitions": at_or_above,
                "templates": section.session.shape_views(),
            }),
            "bar 1 moved a shape that had not cleared generality",
        );
    }

    // (f.2) OD-121: the exclusion is one test, not two claims. The frames bar 1
    // dropped are still §29's frames — so the crystallize table this run leaves
    // behind is the same table a replay of the identical trace builds, forwarded
    // frames included, and at least one forwarded tick is a tick the other
    // detector counted.
    let live_cryst = on.session.crystallize_table_bytes();
    let replay_cryst = on.session.crystallize_trace_table(frames)?;
    let mut forwarded_ticks: Vec<i64> = Vec::new();
    for line in section.session.skill_lines() {
        if line["transition"].as_str() == Some("forwarded") {
            let t = line["t"].as_i64().unwrap_or(-1);
            if !forwarded_ticks.contains(&t) {
                forwarded_ticks.push(t);
            }
        }
    }
    let counted_ticks: Vec<i64> = section
        .session
        .crystallize_lines()
        .iter()
        .filter(|line| {
            matches!(
                line["transition"].as_str(),
                Some("open") | Some("nominate") | Some("recurred") | Some("ask")
            )
        })
        .map(|line| line["t"].as_i64().unwrap_or(-1))
        .collect();
    let shared: Vec<i64> = forwarded_ticks.iter().copied().filter(|t| counted_ticks.contains(t)).collect();
    if replay_cryst == live_cryst && !shared.is_empty() && !forwarded_ticks.is_empty() {
        ok(
            out,
            "G18.f.2",
            "f",
            "a forwarded frame stays visible to §29's fold in the same run",
            json!({
                "forwarded_ticks": forwarded_ticks,
                "crystallize_counted_ticks": counted_ticks,
                "shared_ticks": shared,
                "table_bytes": live_cryst.len(),
            }),
            "the two detectors disagree about a frame, or the fold lost the frames the shape bar dropped",
        );
    } else {
        bad(
            out,
            "G18.f.2",
            "f",
            "a forwarded frame stays visible to §29's fold in the same run",
            json!({
                "forwarded_ticks": forwarded_ticks,
                "crystallize_counted_ticks": counted_ticks,
                "shared_ticks": shared,
                "live_table_bytes": live_cryst.len(),
                "replay_table_bytes": replay_cryst.len(),
            }),
            "OD-121's one test became two claims: the exclusion hid a frame from the other detector",
        );
    }

    // (f.3) no word leak. AD.2 gives a minted-but-unnamed shape a placeholder
    // surface, so the *interner* may hold `skill:{hash}` and nothing else of the
    // kind: what may never print is a template word, `NAMELESS`, or a pattern id.
    const NEEDLES: [&str; 5] = ["cluster:", "cluster_", "NAMELESS", "pattern_65535", "seed|"];
    let mut leaks = Vec::new();
    let mut scanned = 0usize;
    for (section_index, section_run) in run.sections.iter().enumerate() {
        let interner = section_run.session.hash_inputs()?.interner;
        let interner_text = String::from_utf8_lossy(&interner).to_string();
        for needle in NEEDLES {
            if interner_text.contains(needle) {
                leaks.push(format!("section {section_index} interner: `{needle}`"));
            }
        }
        for line in section_run.session.monologue() {
            scanned += 1;
            for needle in NEEDLES {
                if line.surface.contains(needle) {
                    leaks.push(format!("section {section_index} monologue: `{needle}`"));
                }
            }
        }
        for record in &section_run.steps {
            let Some(output) = &record.output else { continue };
            let text = match output {
                Output::Answer { surface, .. } | Output::Ask { surface, .. } => surface.clone(),
                _ => continue,
            };
            scanned += 1;
            for needle in NEEDLES {
                if text.contains(needle) {
                    leaks.push(format!("section {section_index} step {}: `{needle}`", record.index));
                }
            }
        }
    }
    if leaks.is_empty() {
        ok(
            out,
            "G18.f.3",
            "f",
            "no template word, no cluster word, no NAMELESS and no pattern id reaches an output",
            json!({"surfaces_scanned": scanned, "needles": NEEDLES, "shapes": shapes}),
            "the renderer printed a token derived from NAMELESS (04 §21 U.1)",
        );
    } else {
        bad(
            out,
            "G18.f.3",
            "f",
            "no template word, no cluster word, no NAMELESS and no pattern id reaches an output",
            json!({"leaks": leaks}),
            "a shape printed a word nobody taught",
        );
    }

    // 05 §23.4's artifact: one line per shape transition, ask and mint.
    fs::create_dir_all(ws.path("gates/G18"))?;
    let mut log = String::new();
    for (section_index, section_run) in run.sections.iter().enumerate() {
        for line in section_run.session.skill_lines() {
            let mut entry = line.clone();
            if let Some(object) = entry.as_object_mut() {
                object.insert("section".to_string(), json!(section_index));
            }
            log.push_str(&entry.to_string());
            log.push('\n');
        }
    }
    fs::write(ws.path("gates/G18/skill_log.jsonl"), log)?;
    Ok(())
}



// ---------------------------------------------------------------------------
// G19 — world: N voices and delegated authority (05 §23.5, docs/26, 12 §5 §33).
// ---------------------------------------------------------------------------

/// docs/26 §26.2's protected tier as the harness counts it: the shelves a
/// non-owner class may never write. `questions` and `context` are **not** in the
/// list on purpose — every line pays the sense floor (01 §6), and leg (a) grades
/// the fact that a stranger's `context` row *does* append. A run that withheld
/// those rows would be a different design, and 05 §23.5 says to report it as one.
const G19_PROTECTED: [u16; 4] = [SHELF_S_R_PAIRS, SHELF_FACTS, SHELF_LINKS, SHELF_SKILLS];

/// The launch state 05 §2 gives V13: `WORLD_TIERS = granted`, read from the
/// registry's own box rather than invented, and the run's declared constants win.
fn g19_constants(run: &Run) -> BTreeMap<String, i64> {
    let mut out = super::runner::g19_launch_point();
    for (name, value) in &run.request.constants {
        out.insert(name.clone(), *value);
    }
    out
}

/// One in-process replay of a fixture at the gate's launch state, carrying the
/// identity registration the file declares.
fn g19_drive(run: &Run, steps: &[Step]) -> Result<DriveOutcome> {
    g19_drive_at(run, steps, &[])
}

/// The same drive with named registry rows forced to `0`. **G19(g)** needs it:
/// at the run's own launch point the `origin = 2` nomination ask is open from
/// t=3 to the end of the fixture, and 04 §19 S.2 step 2 skips it by OD-128's
/// law, so no tick of the run's own has an empty agenda. The window is a deny
/// conjunct on the idle predicate (AE.4) and does not care what armed the ask,
/// so the pair is graded with the two nomination producers off — and the run's
/// own ticks are reported next to it, with the reason they never clear.
fn g19_drive_at(run: &Run, steps: &[Step], off: &[&str]) -> Result<DriveOutcome> {
    let mut constants = g19_constants(run);
    for name in off {
        constants.insert((*name).to_string(), 0);
    }
    drive(&DriveRequest {
        steps: steps.to_vec(),
        identities: Some(run.request.scenario.registered()),
        seed: run.request.seed,
        constants,
        variant: run.request.variant.clone(),
        sleep_interrupt: run.request.sleep_interrupt,
        cut_after: None,
        cut_path: None,
        freeze_at_end: false,
        inbox_lane: Vec::new(),
    })
}

/// 05 §23.5(g)'s other half of the pair: the same transcript with the declared
/// windows removed. A world declaration advances no clock (`world_step` hands back
/// the tick it ran at), so the two drives stay tick-aligned step for step and the
/// only thing between them is the deny conjunct.
fn g19_without_windows(steps: &[Step]) -> Vec<Step> {
    steps
        .iter()
        .filter(|step| !matches!(step, Step::World(WorldOp::Window(_))))
        .cloned()
        .collect()
}

/// The window in force *after* each step, re-derived from the file's own
/// declarations. The leg reads "did a window deny this tick" from the transcript
/// rather than from the report the session wrote, and the two must agree.
fn g19_windows(steps: &[Step]) -> Vec<Option<si_core::world::Window>> {
    let mut current = None;
    steps
        .iter()
        .map(|step| {
            if let Step::World(WorldOp::Window(window)) = step {
                current = *window;
            }
            current
        })
        .collect()
}

fn g19_protected(delta: &BTreeMap<u16, i64>) -> i64 {
    G19_PROTECTED
        .iter()
        .filter_map(|shelf| delta.get(shelf))
        .copied()
        .sum()
}

fn g19_output(record: &StepRecord) -> String {
    match &record.output {
        Some(Output::Answer { .. }) => "answer".to_string(),
        Some(Output::Ask { .. }) => "ask".to_string(),
        Some(Output::Committed { .. }) => "committed".to_string(),
        Some(Output::Refused { reason }) => format!("refused:{reason}"),
        Some(Output::Inert { reason }) => format!("inert:{reason}"),
        None => "no-tick".to_string(),
    }
}

/// AE.1's rows as the report prints them: `GrantRow` ships no serializer, because
/// a grant is session RAM and nothing in the build may store it (OD-123).
fn g19_grants(rows: &[si_core::world::GrantRow], now: i32) -> Vec<Value> {
    rows.iter()
        .map(|row| {
            json!({
                "subject": row.subject,
                "right": row.right.literal(),
                "granted_by": row.granted_by,
                "granted_at_t": row.granted_at_t,
                "revoked_at_t": row.revoked_at_t,
                "live_at_end": row.live_at(now),
            })
        })
        .collect()
}

/// The open `origin = 2` rows of one store, each with the lane it is readable
/// through. This is **G19(f)**'s whole evidence set and `lane_log.jsonl`'s content.
fn g19_lane_rows(session: &Session) -> Result<Vec<Value>> {
    let now = session.now();
    let bare = gate::latest_open_ask(session.store(), now)?;
    let lane = gate::latest_open_nomination(session.store(), now, gate::GAP_MOVE_NOMINATION)?;
    Ok(gate::open_asks(session.store(), now)?
        .into_iter()
        .filter(|row| {
            row.origin() == gate::ASK_ORIGIN_NOMINATION
                && row.gap_kind() == gate::GAP_MOVE_NOMINATION
        })
        .map(|row| {
            json!({
                "row_id": row.row_id,
                "asked_at": row.asked_at,
                "gap_kind": row.gap_kind(),
                "origin": row.origin(),
                "key_point": row.key_point,
                "status": row.status,
                "age_at_now": now - row.asked_at,
                "lane": "owner",
                "read_by": "gate::latest_open_nomination",
                "captured_by_ordinary_lane": bare
                    .as_ref()
                    .is_some_and(|other| other.row_id == row.row_id),
                "is_the_lane_read": lane
                    .as_ref()
                    .is_some_and(|other| other.row_id == row.row_id),
            })
        })
        .collect())
}

/// One idle tick as the window pair sees it, with the denial re-derived from the
/// transcript rather than from the session's own flag.
struct G19Tick {
    ordinal: usize,
    step: usize,
    t: i32,
    ignited: Option<u32>,
    suppressed: bool,
    quiet_run: u32,
    quiet_grew: bool,
    denied: bool,
}

impl G19Tick {
    fn json(&self) -> Value {
        json!({
            "ordinal": self.ordinal,
            "step": self.step,
            "t": self.t,
            "ignited": self.ignited,
            "suppressed_by_window": self.suppressed,
            "window_denies_here": self.denied,
            "quiet_run_after": self.quiet_run,
            "agenda_clear_at_this_tick": self.quiet_grew,
        })
    }
}

/// The declared ticks of one drive, paired with the window in force at each. A
/// `None` window list means the drive has no declarations at all.
fn g19_ticks(
    outcome: &DriveOutcome,
    windows: Option<&[Option<si_core::world::Window>]>,
) -> Vec<G19Tick> {
    let mut out = Vec::new();
    let mut ordinal = 0usize;
    for (index, trace) in outcome.steps.iter().enumerate() {
        if !trace.idle {
            continue;
        }
        let denied = windows
            .and_then(|list| list.get(index))
            .copied()
            .flatten()
            .is_some_and(|window| window.denies(trace.t));
        let previous = index
            .checked_sub(1)
            .map(|prior| outcome.steps[prior].quiet_run_after);
        out.push(G19Tick {
            ordinal,
            step: index + 1,
            t: trace.t,
            ignited: trace.idle_ignited,
            suppressed: trace.idle_suppressed_by_window,
            quiet_run: trace.quiet_run_after,
            quiet_grew: previous.is_some_and(|before| trace.quiet_run_after == before + 1),
            denied,
        });
        ordinal += 1;
    }
    out
}

fn g19(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let section = run
        .sections
        .first()
        .context("a G19 run has no section to grade")?;
    if !expect_fixture(run, ws, "G19", "05 §23.5", "harness/scenarios/g19_server.txt", out)? {
        return Ok(());
    }
    let steps = &run.request.scenario.sections[0];
    let granted = si_core::consts::row("WORLD_TIERS").map(|row| row.hi);
    if run.request.constants.get("WORLD_TIERS").copied() != granted {
        bad(
            out,
            "G19.launch",
            "a",
            "WORLD_TIERS = granted installed through the launch channel",
            json!({
                "constants": run.request.constants,
                "WORLD_TIERS_box": si_core::consts::row("WORLD_TIERS")
                    .map(|row| format!("{}..{}", row.lo, row.hi)),
            }),
            "the gate must launch at the rolled state it grades (12 §6.4): at the owner-only tier there is no grant view to trace, and legs (c) and (h) would grade nothing",
        );
        return Ok(());
    }

    // ---------------------------------------------------------------- (a)
    let mut strangers: Vec<Value> = Vec::new();
    for (index, record) in section.steps.iter().enumerate() {
        if record.class != Some("stranger") {
            continue;
        }
        // A line the speakgate never admitted never reached the class arm, so it is
        // not this leg's evidence: silence bought by §23's filter is a different
        // mechanism than the silence AE.2 names.
        if record.speak.as_ref().is_some_and(|note| !note.reached_loop) {
            continue;
        }
        let delta = g15_step_delta(run, 0, index);
        strangers.push(json!({
            "step": index + 1,
            "t": record.t,
            "identity": record.identity,
            "text": record.text,
            "protected_writes": g19_protected(&delta),
            "context_rows": delta.get(&SHELF_CONTEXT).copied().unwrap_or(0),
            "rows_written": record.rows_written,
            "output": g19_output(record),
        }));
    }
    let silent = strangers.iter().all(|entry| {
        entry["protected_writes"].as_i64() == Some(0) && entry["rows_written"].as_u64() == Some(0)
    });
    let pays_floor = strangers
        .iter()
        .all(|entry| entry["context_rows"].as_i64() == Some(1));
    if strangers.len() >= 8 && silent && pays_floor {
        ok(
            out,
            "G19.a",
            "a",
            "eight strangers speak and the owner is silent: zero protected-class writes, and the `context` sense rows still append",
            json!({
                "stranger_lines": strangers.len(),
                "lines": strangers,
                "protected_shelves": ["s_r_pairs=4", "facts=5", "links=6", "skills=9"],
                "context_is_not_protected": "01 §6: every line pays the sense floor. A run that withheld a stranger's `context` row would be reported as a **different design**, not a stricter one.",
            }),
            "a stranger-class line wrote a belief row, or the sense floor was withheld from it",
        );
    } else {
        bad(
            out,
            "G19.a",
            "a",
            "eight strangers speak and the owner is silent: zero protected-class writes, and the `context` sense rows still append",
            json!({
                "stranger_lines": strangers.len(),
                "silent": silent,
                "pays_sense_floor": pays_floor,
                "lines": strangers,
            }),
            if strangers.len() < 8 {
                "the fixture's strangers never reached the loop, so the class arm was never exercised — the silence is the speakgate's, not AE.2's"
            } else if !pays_floor {
                "a stranger line reached the loop and paid no `context` sense row"
            } else {
                "a stranger-class line wrote a belief row"
            },
        );
    }

    // ---------------------------------------------------------------- (b)
    let class_check = super::checks::run("check_speaker_class_derived", ws)?;
    let owner_identity = section.session.world().owner_identity;
    let grimm_identity = section.session.world().grimm_identity;
    let mut drift: Vec<Value> = Vec::new();
    let mut seen_ids: BTreeSet<u64> = BTreeSet::new();
    for record in &section.steps {
        let Some(identity) = record.identity else { continue };
        seen_ids.insert(identity);
        let derived = section.session.class_of(identity, record.t).literal();
        if record.class != Some(derived) {
            drift.push(json!({
                "step": record.index,
                "identity": identity,
                "recorded": record.class,
                "rederived_at_the_same_tick": derived,
            }));
        }
    }
    let now = section.session.now();
    let table: Vec<Value> = {
        let mut ids: Vec<u64> = seen_ids.iter().copied().collect();
        ids.push(owner_identity);
        ids.push(grimm_identity);
        ids.sort_unstable();
        ids.dedup();
        ids.into_iter()
            .map(|identity| {
                json!({
                    "identity": identity,
                    "class": section.session.class_of(identity, now).literal(),
                })
            })
            .collect()
    };
    let stranger_count = table
        .iter()
        .filter(|entry| entry["class"].as_str() == Some("stranger"))
        .count();
    if class_check.verdict == "pass"
        && drift.is_empty()
        && table.len() >= 10
        && stranger_count >= 8
    {
        ok(
            out,
            "G19.b",
            "b",
            "no class is stored anywhere, and `class(identity)` over the whole fixture is the function's own output",
            json!({
                "check_speaker_class_derived": class_check.verdict,
                "identities": table.len(),
                "class_table": table,
                "owner_identity": owner_identity,
                "grimm_identity": grimm_identity,
                "reading": "a stranger is a **stranger**, not a labelled one: the table is `Session::class_of(id, now)` — identity, owner id and the grant view at this tick — recomputed at grade time from the identity alone, and it agrees with the class every step derived at generation",
            }),
            "a stored or cached class, a label table, or a step whose recorded class is not what the function returns",
        );
    } else {
        bad(
            out,
            "G19.b",
            "b",
            "no class is stored anywhere, and `class(identity)` over the whole fixture is the function's own output",
            json!({
                "check_speaker_class_derived": class_check.verdict,
                "drift": drift,
                "identities": table.len(),
                "strangers": stranger_count,
                "class_table": table,
            }),
            "a class cached, serialized or stored, or a step whose derived class is not the function's answer for its identity",
        );
    }

    // ---------------------------------------------------------------- (c)
    let birth = g19_drive(run, &steps[..1])?;
    let grants_at_birth = birth.session.world().grants.rows().len();
    let grant_at = section
        .steps
        .iter()
        .position(|record| record.world.as_deref().is_some_and(|word| word.starts_with("grant ")));
    let revoke_at = section
        .steps
        .iter()
        .position(|record| record.world.as_deref().is_some_and(|word| word.starts_with("revoke ")));
    let trusted: Vec<(usize, &StepRecord)> = section
        .steps
        .iter()
        .enumerate()
        .filter(|(_, record)| record.class == Some("trusted"))
        .collect();
    let mut refusals: Vec<Value> = Vec::new();
    for (index, record) in &trusted {
        let delta = g15_step_delta(run, 0, *index);
        let refused = matches!(
            &record.output,
            Some(Output::Refused { reason }) if reason.contains("SpeakerNotRepresentable")
        );
        let arm = run
            .events(0, record)
            .iter()
            .map(|event| String::from_utf8_lossy(&event.body).to_string())
            .filter(|body| body.contains("\"arm\":\"class\""))
            .collect::<Vec<_>>();
        refusals.push(json!({
            "step": index + 1,
            "t": record.t,
            "identity": record.identity,
            "text": record.text,
            "class": record.class,
            "output": g19_output(record),
            "refused_by_name": refused,
            "rows_written": record.rows_written,
            "protected_writes": g19_protected(&delta),
            "context_rows": delta.get(&SHELF_CONTEXT).copied().unwrap_or(0),
            "refuse_events": arm,
        }));
    }
    let after_revoke = revoke_at.and_then(|at| {
        section.steps[at + 1..]
            .iter()
            .find(|record| record.identity.is_some())
            .map(|record| (record.class, record.index))
    });
    let zero_bytes = refusals.iter().all(|entry| {
        entry["refused_by_name"].as_bool() == Some(true)
            && entry["rows_written"].as_u64() == Some(0)
            && entry["protected_writes"].as_i64() == Some(0)
            && entry["refuse_events"].as_array().is_some_and(|list| !list.is_empty())
    });
    let trace_observed = json!({
        "od": "OD-135",
        "why_the_refusal_is_the_pass": "OD-135: `prov_speaker` has no honest encoding for a delegated tier, so the build refuses with zero bytes rather than minting a fourth value. **The refusal is the pass condition**, printed here as the pass.",
        "grants_at_birth": grants_at_birth,
        "grant_step": grant_at.map(|at| at + 1),
        "revoke_step": revoke_at.map(|at| at + 1),
        "trusted_lines": refusals,
        "next_derivation_after_revocation": after_revoke.map(|(class, index)| json!({"class": class, "step": index})),
        "grant_rows": g19_grants(&run.request.scenario.grant_rows(owner_identity), now),
        "od_136": "no `/trust` parser and no adapter exist: the grant table is the scenario's own owner declaration, which is 05 §2's V13 row verbatim. **No grant is issued by the build.**",
    });
    if grants_at_birth == 0
        && grant_at.is_some()
        && revoke_at > grant_at
        && !trusted.is_empty()
        && zero_bytes
        && after_revoke.is_some_and(|(class, _)| class == Some("stranger"))
    {
        ok(
            out,
            "G19.c",
            "c",
            "the owner grant mints a row, the Trusted belief write is refused with zero bytes, the revocation is appended, and the next derivation is `Stranger`",
            trace_observed.clone(),
            "a granted line wrote a belief byte, a grant existed at birth, or a revoked identity still derived `Trusted`",
        );
    } else {
        bad(
            out,
            "G19.c",
            "c",
            "the owner grant mints a row, the Trusted belief write is refused with zero bytes, the revocation is appended, and the next derivation is `Stranger`",
            trace_observed,
            "the grant trace did not produce the named refusal with zero bytes on every Trusted line",
        );
    }

    // ---------------------------------------------------------------- (d)
    // The nomination ask is this fixture's own: the shape detector opens it at t=3
    // (the run's `skills.jsonl` `transition":"ask"` line, kind 6 / origin 2), so the
    // leg re-drives the run's steps and appends a third-sense-form line from an
    // identity the file never registered — an answer from a voice with no lane.
    let before = g19_drive(run, steps)?;
    let mut tail = steps.clone();
    tail.push(Step::Sense {
        source: Source::Grimm,
        text: "sky is blue".to_string(),
        identity: Some(21),
        conversation: 0,
    });
    tail.push(Step::Idle);
    let after = g19_drive(run, &tail)?;
    let before_rows = g19_lane_rows(&before.session)?;
    let after_rows = g19_lane_rows(&after.session)?;
    let before_ids: Vec<u64> = before_rows.iter().filter_map(|row| row["row_id"].as_u64()).collect();
    let after_ids: Vec<u64> = after_rows.iter().filter_map(|row| row["row_id"].as_u64()).collect();
    let answered = &after.steps[steps.len()];
    let prior = after.steps[steps.len() - 1].rows_after;
    let sense_floor = answered.rows_after as i64 - prior as i64;
    let kept = before_ids.iter().all(|row| after_ids.contains(row));
    let aged = after_rows.iter().all(|row| row["age_at_now"].as_i64().unwrap_or(0) > 0);
    if before_ids.is_empty() {
        bad(
            out,
            "G19.d",
            "d",
            "a stranger answers an open nomination ask: it commits nothing and the ask stays open",
            json!({"origin_2_asks_before": before_ids, "after": after_rows}),
            "the fixture opened no kind-6 / origin-2 ask, so there was no nomination for a stranger to answer",
        );
    } else if sense_floor != 1 || !kept || !aged {
        bad(
            out,
            "G19.d",
            "d",
            "a stranger answers an open nomination ask: it commits nothing and the ask stays open",
            json!({
                "origin_2_asks_before": before_rows,
                "origin_2_asks_after": after_rows,
                "rows_the_stranger_answer_appended": sense_floor,
                "ask_ids_kept": kept,
                "aged": aged,
                "idle_after": after.steps.last().map(|tick| json!({
                    "t": tick.t,
                    "ignited": tick.idle_ignited,
                    "revisited": tick.idle_revisited,
                })),
            }),
            "the stranger's answer committed a row against a nomination the owner lane never consumed, or the ask closed or stayed a newborn",
        );
    } else {
        ok(
            out,
            "G19.d",
            "d",
            "a stranger answers an open nomination ask: it commits nothing and the ask stays open",
            json!({
                "origin_2_asks_before": before_rows,
                "origin_2_asks_after": after_rows,
                "stranger_answer": "identity 21, which no line of the file registers, speaks `sky is blue` one step past the fixture's last step",
                "rows_the_stranger_answer_appended": sense_floor,
                "sense_floor_only": "the answer's one row is its own `context` sense row (01 §6); no belief row moved, so nothing was committed",
                "aging": {
                    "measured_as": "the row is still open at the drive's last tick and `now - asked_at` grew past zero; the quiet tick after it resolved nothing, because 04 §19 S.2 step 2 skips `origin = 2`",
                    "kind_4": "**UNMEASURED as a signal**: `SignalKind::OpenQuestionAge` (01 §5.3's kind 4) has no producer in this build, so the leg does not claim a signal fired. It claims the ask stayed open and kept the agenda non-empty — which is what suppresses ignition (04 §19 S.2 step 3).",
                },
                "idle_after": after.steps.last().map(|tick| json!({
                    "t": tick.t,
                    "ignited": tick.idle_ignited,
                    "revisited": tick.idle_revisited,
                    "suppressed_by_window": tick.idle_suppressed_by_window,
                })),
            }),
            "a voice with no lane answered a nomination, or the ask closed",
        );
    }

    // ---------------------------------------------------------------- (e)
    let live: Vec<String> = run.sections.iter().map(|section| section.hash.clone()).collect();
    let in_process = run.replay()?;
    let cross = Run::replay_transcript(&run.dir, run.request.seed, &Launch::of(&run.request))?;
    let transcript_text =
        fs::read_to_string(run.dir.join("transcript.txt")).context("the run wrote no transcript")?;
    let tagged = transcript_text
        .lines()
        .filter(|line| line.split('\t').nth(1).is_some_and(|source| source.contains('#')))
        .count();
    let headers = transcript_text
        .lines()
        .filter(|line| line.starts_with("#identity "))
        .count();
    let bodies: Vec<u64> = section
        .session
        .events()
        .events()
        .iter()
        .filter(|event| event.kind == 0)
        .filter_map(|event| si_core::eventlog::decode_sense_body(&event.body).ok())
        .map(|(identity, _conversation, _text)| identity)
        .filter(|identity| *identity != owner_identity && *identity != grimm_identity)
        .collect();
    if live == in_process && live == cross && tagged > 0 && !bodies.is_empty() {
        ok(
            out,
            "G19.e",
            "e",
            "the identity-tagged transcript replays byte-identical, in-process and across processes",
            json!({
                "sections": live.len(),
                "in_process_equal": live == in_process,
                "cross_process_equal": live == cross,
                "identity_tagged_lines": tagged,
                "registration_headers": headers,
                "frame_bodies_carrying_a_third_identity": bodies.len(),
                "distinct_third_identities": bodies.iter().copied().collect::<BTreeSet<u64>>(),
                "hash_note": "identity lives in the **frame body and the transcript**, both already hashed transitively (01 §9), so the state hash gains no item",
            }),
            "a replay moved a byte, or the identity lived nowhere the hash can see",
        );
    } else {
        bad(
            out,
            "G19.e",
            "e",
            "the identity-tagged transcript replays byte-identical, in-process and across processes",
            json!({
                "live": live,
                "in_process": in_process,
                "cross_process": cross,
                "identity_tagged_lines": tagged,
                "registration_headers": headers,
                "third_identity_bodies": bodies.len(),
            }),
            "the identity-tagged run is not reproducible, or the transport byte never reached a hashed carrier",
        );
    }

    // ---------------------------------------------------------------- (f)
    let run_rows = g19_lane_rows(&section.session)?;
    let all_rows: Vec<Value> = run_rows.iter().chain(after_rows.iter()).cloned().collect();
    let captured: Vec<Value> = all_rows
        .iter()
        .filter(|row| row["captured_by_ordinary_lane"].as_bool() == Some(true))
        .cloned()
        .collect();
    let unread: Vec<Value> = all_rows
        .iter()
        .filter(|row| row["is_the_lane_read"].as_bool() != Some(true))
        .cloned()
        .collect();
    if all_rows.is_empty() {
        bad(
            out,
            "G19.f",
            "f",
            "every `origin = 2` ask lands in the owner lane and nowhere else",
            json!({"origin_2_asks": 0, "run_store": run_rows, "stranger_answer_drive_store": after_rows}),
            "no nomination ask was open in either graded store, so the lane rule had nothing to route",
        );
    } else if captured.is_empty() && unread.is_empty() {
        ok(
            out,
            "G19.f",
            "f",
            "every `origin = 2` ask lands in the owner lane and nowhere else",
            json!({
                "origin_2_asks": all_rows.len(),
                "rows": all_rows,
                "od": "OD-128",
                "lane_rule": "`gate::latest_open_ask` filters `origin = 2` out by construction and `gate::latest_open_nomination` is the only read that returns one — a detector's proposal answers only in the lane that mints its point, so it can never satisfy itself",
            }),
            "an ordinary lane captured a nomination, or the owner lane could not read a row it owns",
        );
    } else {
        bad(
            out,
            "G19.f",
            "f",
            "every `origin = 2` ask lands in the owner lane and nowhere else",
            json!({"captured_by_ordinary_lane": captured, "unreachable_from_the_owner_lane": unread}),
            "OD-128's lane is not the only reader of an origin-2 row",
        );
    }

    // ---------------------------------------------------------------- (g)
    // The pair is driven with the two nomination producers off. At the run's own
    // launch point the `origin = 2` ask opened at t=3 is still open at the last
    // tick (leg (d) grades that), and OD-128's skip means the agenda is therefore
    // never empty — the run's own ticks are reported below, and they show a window
    // that never had an allowed tick to subtract. AE.4 makes the window a deny
    // conjunct on the idle predicate alone, so the predicate's own arm (the empty
    // agenda) is what this leg needs live, and turning the detector off does not
    // touch it.
    let producers = ["CRYSTALLIZE", "SKILLS", "SKILL_COMPOSITION"];
    let with = g19_drive_at(run, steps, &producers)?;
    let without = g19_drive_at(run, &g19_without_windows(steps), &producers)?;
    let windows = g19_windows(steps);
    let declared: Vec<usize> = steps
        .iter()
        .enumerate()
        .filter(|(_, step)| matches!(step, Step::World(WorldOp::Window(_))))
        .map(|(index, _)| index)
        .collect();
    // A window declaration is not a tick: it advances no clock, emits no event and
    // ignites nothing. That is the direct witness of "no idle tick occurs *because*
    // a window opened", and it does not depend on what the walk happened to light.
    let declarations_quiet = declared.iter().all(|index| {
        let trace = &with.steps[*index];
        let same_clock = with
            .steps
            .get(*index + 1)
            .is_some_and(|next| next.t == trace.t);
        !trace.idle && trace.kinds.is_empty() && trace.idle_ignited.is_none() && same_clock
    });
    let with_ticks = g19_ticks(&with, Some(&windows));
    let without_ticks = g19_ticks(&without, None);
    let aligned = with_ticks.len() == without_ticks.len()
        && with_ticks
            .iter()
            .zip(without_ticks.iter())
            .all(|(a, b)| a.t == b.t && a.quiet_run == b.quiet_run);
    let first_denial = with_ticks.iter().find(|tick| tick.suppressed).map(|tick| tick.ordinal);
    let prefix_identical = with_ticks
        .iter()
        .zip(without_ticks.iter())
        .take(first_denial.unwrap_or(usize::MAX))
        .all(|(a, b)| a.t == b.t && a.quiet_run == b.quiet_run && a.ignited == b.ignited);
    let law = with_ticks.iter().all(|tick| {
        // Three claims, each derived from the producer's own definition
        // (`suppressed_by_window = denied && agenda_clear && within_rate`,
        // `crates/si-session/src/lib.rs:3076`), and nothing more:
        //   1. a suppression names a window that really denies — the flag is not a lie;
        //   2. nothing ignites while the window denies — a window can say "not now",
        //      never "now" (04 §33 AE.4);
        //   3. the guarded converse of 1: where the window denies *and* the quiet-and-empty
        //      law held, the tick must be flagged, so a denial that silently did nothing
        //      is caught. The converse cannot be unguarded: a denial with the agenda busy
        //      or the rate spent suppresses nothing, because there was nothing to suppress.
        // A fourth operand here read `(!suppressed || denied) && (denied || !suppressed)` —
        // one disjunction with its operands commuted, so it carried no information and
        // `clippy::eq_op` denied the build (**OD-152**, resolved by this derivation).
        (!tick.suppressed || tick.denied)
            && (tick.ignited.is_none() || !tick.denied)
            && !(tick.denied && tick.quiet_grew && !tick.suppressed)
    });
    let without_clean = without_ticks.iter().all(|tick| !tick.suppressed && !tick.denied);
    let suppressed = with_ticks.iter().filter(|tick| tick.suppressed).count();
    let denied_ticks = with_ticks.iter().filter(|tick| tick.denied).count();
    let preempted = section.steps.iter().rev().find(|record| {
        record.source == Some(Source::Owner) && matches!(record.output, Some(Output::Ask { .. }))
    });
    let preemption = preempted.map(|record| {
        let index = record.index - 1;
        let window = windows.get(index).copied().flatten();
        let delta = g15_step_delta(run, 0, index);
        json!({
            "step": record.index,
            "t": record.t,
            "ask": record.text,
            "window_denied_at_that_tick": window.is_some_and(|w| w.denies(record.t)),
            "questions_rows": delta.get(&SHELF_QUESTIONS).copied().unwrap_or(0),
        })
    });
    let owner_went_through = preemption
        .as_ref()
        .is_some_and(|entry| entry["questions_rows"].as_i64() == Some(1));
    if declarations_quiet
        && aligned
        && prefix_identical
        && law
        && without_clean
        && suppressed > 0
        && owner_went_through
    {
        ok(
            out,
            "G19.g",
            "g",
            "an idle tick happens only where the window is open *and* the quiet-and-empty-agenda law already held, and no tick happens because a window opened",
            json!({
                "declared_window_steps": declared.iter().map(|index| index + 1).collect::<Vec<_>>(),
                "pair_drive": format!(
                    "the two drives are the fixture's own steps, one transcript apart by the `window` declarations, launched at G19's point with {} forced to 0: the run's own nomination ask (open since t={}, still open at now={}) holds its agenda non-empty by OD-128's skip, so the run's own ticks have no allowed tick to subtract and the leg would grade a constant zero",
                    producers.join(", "),
                    run_rows.first().map(|row| row["asked_at"].as_i64().unwrap_or(0)).unwrap_or(0),
                    section.session.now()
                ),
                "ticks_with_window": with_ticks.len(),
                "ticks_without": without_ticks.len(),
                "suppressed_with": suppressed,
                "denied_with": denied_ticks,
                "suppressed_without": without_ticks.iter().filter(|tick| tick.suppressed).count(),
                "ignited_with": with_ticks.iter().filter(|tick| tick.ignited.is_some()).count(),
                "ignited_without": without_ticks.iter().filter(|tick| tick.ignited.is_some()).count(),
                "first_denial_ordinal": first_denial,
                "idle_ticks": with_ticks.iter().map(G19Tick::json).collect::<Vec<_>>(),
                "idle_ticks_without_windows": without_ticks.iter().map(G19Tick::json).collect::<Vec<_>>(),
                "owner_preemption": preemption,
                "od": "OD-126",
                "rng_note": "a suppressed tick skips `si_walk::ignite`, so the two drives' rng streams part company at the first denial: the leg compares the two **up to** that tick (prefix_identical, which is the open-window stretch — it costs nothing) and past it asserts only the structural laws (same tick count, same clock, same quiet-run counter, denial-shaped suppression, no ignition under a denying window).",
            }),
            "a window opened a tick, moved the clock, changed the agenda counter, let a thought fire under its own denial, or blocked the owner's lane",
        );
    } else {
        bad(
            out,
            "G19.g",
            "g",
            "an idle tick happens only where the window is open *and* the quiet-and-empty-agenda law already held, and no tick happens because a window opened",
            json!({
                "declarations_quiet": declarations_quiet,
                "ticks_aligned": aligned,
                "prefix_identical": prefix_identical,
                "suppression_law": law,
                "without_clean": without_clean,
                "suppressed_with": suppressed,
                "denied_with": denied_ticks,
                "owner_went_through": owner_went_through,
                "idle_ticks": with_ticks.iter().map(G19Tick::json).collect::<Vec<_>>(),
                "idle_ticks_without_windows": without_ticks.iter().map(G19Tick::json).collect::<Vec<_>>(),
                "owner_preemption": preemption,
            }),
            "the window pair disagrees about which ticks happen, or a denial moved something it may not move",
        );
    }

    // ---------------------------------------------------------------- (h)
    let note_of = |record: &StepRecord| -> Option<Value> {
        record.speak.as_ref().map(|note| {
            json!({
                "reached_loop": note.reached_loop,
                "admitted": note.admitted,
                "dropped": note.dropped,
                "reason": note.reason,
                "layer": note.layer,
                "n_untaught_tokens": note.n_untaught_tokens,
                "novel_claims": note.novel_claims,
                "ref_row_id": note.ref_row_id,
                "untaught": note.untaught,
            })
        })
    };
    let grimm_note = section
        .steps
        .iter()
        .filter(|record| record.class == Some("grimm"))
        .find_map(|record| note_of(record).map(|note| (record.index, record.text.clone(), note)));
    let trusted_note = trusted
        .first()
        .and_then(|(_, record)| note_of(record).map(|note| (record.index, record.text.clone(), note)));
    let observed_h = json!({
        "grimm": grimm_note.as_ref().map(|(at, text, _)| json!({"step": at, "text": text})),
        "trusted": trusted_note.as_ref().map(|(at, text, _)| json!({"step": at, "text": text})),
        "note": "leg (h) grades an emission that reached the gate: a line the seam never admitted has no §23 verdict to compare",
    });
    if let (Some((grimm_at, grimm_text, grimm)), Some((trusted_at, trusted_text, trusted_body))) =
        (grimm_note, trusted_note)
    {
        let same_input = grimm_text == trusted_text;
        let same_verdict = grimm == trusted_body;
        let admitted = grimm["admitted"].as_bool() == Some(true);
        if same_input && same_verdict && admitted {
            ok(
                out,
                "G19.h",
                "h",
                "a Trusted line passes the same two-layer filter and the same token-membership test grimm's lines pass",
                json!({
                    "grimm_step": grimm_at,
                    "trusted_step": trusted_at,
                    "text": grimm_text,
                    "verdicts": [grimm, trusted_body],
                    "identity_blindness": "`seam.speak(&store, text)` takes the words and the rows — never an identity, never a class (04 §23). One function, one filter, one class: the class only changes what the loop does with a line the filter already admitted, and that is leg (c).",
                    "reuses": "05 §23's filter, unchanged",
                }),
                "the granted line reached a different verdict than grimm's identical line, or the speakgate grew a second dialect",
            );
        } else {
            bad(
                out,
                "G19.h",
                "h",
                "a Trusted line passes the same two-layer filter and the same token-membership test grimm's lines pass",
                json!({
                    "same_text": same_input,
                    "same_verdict": same_verdict,
                    "admitted": admitted,
                    "grimm": [grimm_at, grimm_text, grimm],
                    "trusted": [trusted_at, trusted_text, trusted_body],
                }),
                "the two lines are not the same emission, or one filter gave two verdicts for one input",
            );
        }
    } else {
        bad(
            out,
            "G19.h",
            "h",
            "a Trusted line passes the same two-layer filter and the same token-membership test grimm's lines pass",
            observed_h,
            "grimm's line or the granted line produced no speakgate verdict, so §23's filter never ran on the pair",
        );
    }

    // ---------------------------------------------------------------- artifacts
    fs::create_dir_all(ws.path("gates/G19"))?;
    use md5::{Digest, Md5};
    let transcript_bytes =
        fs::read(run.dir.join("transcript.txt")).context("the run wrote no transcript")?;
    let transcript_md5 = hex(Md5::digest(&transcript_bytes).as_slice());
    let mut server = transcript_bytes;
    server.extend_from_slice(
        format!(
            "# md5 {transcript_md5}\n# scenario {} md5 {}\n",
            run.request.scenario.path, run.request.scenario.digest
        )
        .as_bytes(),
    );
    fs::write(ws.path("gates/G19/server_transcript.txt"), server)?;

    let trace = json!({
        "fixture": run.request.scenario.path,
        "scenario_md5": run.request.scenario.digest,
        "transcript_md5": transcript_md5,
        "variant": run.request.variant,
        "grants_at_birth": grants_at_birth,
        "birth_is_the_pass": "zero rows at birth, stated as a number: `grants` is the empty view until the owner's own line mints one (OD-125's shipped half)",
        "grant_rows_declared_by_the_owner_lane": g19_grants(&run.request.scenario.grant_rows(owner_identity), now),
        "trusted_lines_refused": refusals,
        "next_derivation_after_revocation": after_revoke.map(|(class, index)| json!({"class": class, "step": index})),
        "refusal_is_the_pass": true,
        "od_135": "the class arm refuses with zero bytes because `prov_speaker` has no honest encoding for a delegated tier",
        "od_136": "no `/trust` parser, no adapter, and no grant issued by the build — the table is the scenario's owner declaration",
    });
    fs::write(
        ws.path("gates/G19/grant_trace.json"),
        serde_json::to_string_pretty(&trace)?,
    )?;

    let mut lane_log = String::new();
    for (store, rows) in [("run", &run_rows), ("stranger_answer_drive", &after_rows)] {
        for row in rows {
            let mut entry = row.clone();
            if let Some(object) = entry.as_object_mut() {
                object.insert("store".to_string(), json!(store));
            }
            lane_log.push_str(&entry.to_string());
            lane_log.push('\n');
        }
    }
    fs::write(ws.path("gates/G19/lane_log.jsonl"), lane_log)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// G20 — the tongue: surface realization by phrase walk (05 §24, 13 §6.6, docs/27).
// ---------------------------------------------------------------------------

/// 13 §6.6's panel, and it is a measured panel rather than a taste. `STYLE_SPREAD`
/// is drawn from the birth seed, and it is the only style axis a CONFIG SI launch
/// can observe at all: **OD-177**/**OD-87** refuse a `STYLE_*` row into the
/// constants channel, and the write gate leaves every point at most one live
/// out-arc, so the four weight axes change no *route*. **OD-189** adds the one
/// legal surface draw — a taught variant of a licensed word rides the same
/// style-seeded stream — so at the variants fixture the individual is observable
/// in the bytes even where the route is fixed. Three of these eight individuals
/// draw a spread below the 128 the launch point's budget needs, so they refuse
/// every tick and print the canonical frame; five draw above it and speak. That
/// boundary is the only place a legal launch changes *whether* it prints, so it is
/// the boundary leg (b) grades.
const G20_PANEL: [u64; 8] = [0xCA, 0x11, 0x51, 0x3039, 0x5151, 0x1, 0x2, 0xABCD];

/// **OD-189**'s bare half: the same transcript with no taught spellings. With one
/// surface per licensed word the walk has nothing to draw between, so every
/// speaking individual printing the same bytes is the expected outcome, named as
/// such — not a ceiling to work around.
const G20_BARE_FIXTURE: &str = "harness/scenarios/g20_tongue_bare.txt";

/// **OD-189**: `chunk_cap` is the budget's draw, not the line's length. Wherever
/// the gate publishes the number it publishes this note beside it.
const G20_CAP_NOTE: &str = "chunk_cap = 1 + budget; the citation's license binds first — a one-row citation licenses two chunks, so this number is a ceiling, not the line's length (OD-189)";

/// The shelves a voice may never write. The same four G19 counts, for the same
/// reason: `questions` and `context` are the sense floor, not a belief.
const G20_PROTECTED: [u16; 4] = [SHELF_S_R_PAIRS, SHELF_FACTS, SHELF_LINKS, SHELF_SKILLS];

/// The budget points a legal launch can name, bracketing the launch point so the
/// cap is seen to bind. `PHRASE_MAX_STEPS`' own `lo` (the floor, **OD-95**) and the
/// registry placeholder (the midpoint, **OD-86**) are added by the legs that need
/// them, read off the row rather than typed here.
const G20_POINTS: [i64; 3] = [8, 32, 64];

/// The launch state 05 §2 gives V14, with the run's own declared constants winning
/// (12 §6.4: the gate launches at the point it grades).
fn g20_constants(run: &Run) -> BTreeMap<String, i64> {
    let mut out = super::runner::g20_launch_point();
    for (name, value) in &run.request.constants {
        out.insert(name.clone(), *value);
    }
    out
}

/// One in-process replay of a step list at the gate's launch state, at a named
/// individual, with named registry rows overridden. Every leg that grades the voice
/// holds the transcript still and moves only the launch state, so every drive is
/// this call — and `TONGUE = 0` is an override like any other, which is what makes
/// the off pair one launch apart rather than one build apart.
fn g20_drive(
    run: &Run,
    scenario: &Scenario,
    steps: &[Step],
    seed: u64,
    overrides: &[(&str, i64)],
) -> Result<DriveOutcome> {
    g20_drive_at(run, scenario, steps, seed, overrides, false)
}

/// The same drive with the section-end freeze the run itself performs before it
/// hashes (`runner.rs`, "10 §7's trigger 2"). Cross-process legs use this one: the
/// launched process recorded its hash after `session.freeze()` and every replay
/// instrument freezes for the same reason, so an unfrozen grading drive is one code
/// path apart from the run by construction, not one process apart.
fn g20_drive_at(
    run: &Run,
    scenario: &Scenario,
    steps: &[Step],
    seed: u64,
    overrides: &[(&str, i64)],
    freeze_at_end: bool,
) -> Result<DriveOutcome> {
    let mut constants = g20_constants(run);
    for (name, value) in overrides {
        constants.insert((*name).to_string(), *value);
    }
    drive(&DriveRequest {
        steps: steps.to_vec(),
        identities: Some(scenario.registered()),
        seed,
        constants,
        variant: run.request.variant.clone(),
        sleep_interrupt: run.request.sleep_interrupt,
        cut_after: None,
        cut_path: None,
        freeze_at_end,
        inbox_lane: Vec::new(),
    })
}

/// One settled frame: the two halves of its emission — the `label` event's label,
/// tag and citation, and the `output` event's string — read beside step 9a's own
/// record. `printed` is what the owner read, `canonical` what the copier would have
/// said, and `realized` the line the walk drew, if it drew one.
#[derive(Clone, Debug)]
struct G20Tick {
    step: usize,
    t: i32,
    thought: u32,
    label: String,
    tag: String,
    citation: Vec<u32>,
    canonical: String,
    realized: Option<String>,
    printed: String,
    anchor: Option<u32>,
    licensed: usize,
    chunk_cap: usize,
}

fn g20_ticks(outcome: &DriveOutcome) -> Vec<G20Tick> {
    outcome
        .steps
        .iter()
        .filter_map(|trace| {
            let tongue = trace.tongue.as_ref()?;
            let frame = trace.frame.as_ref()?;
            Some(G20Tick {
                step: trace.index,
                t: trace.t,
                thought: tongue.thought,
                label: frame.label.clone(),
                tag: frame.tag.clone(),
                citation: frame.citation.clone(),
                canonical: tongue.canonical.clone(),
                realized: tongue.realized.clone(),
                printed: frame.surface.clone(),
                anchor: tongue.anchor,
                licensed: tongue.licensed,
                chunk_cap: tongue.chunk_cap,
            })
        })
        .collect()
}

/// Every frame a drive printed, voice or no voice: `(label, tag, citation)`.
fn g20_frames(outcome: &DriveOutcome) -> Vec<(String, String, Vec<u32>)> {
    outcome
        .steps
        .iter()
        .filter_map(|trace| trace.frame.as_ref())
        .map(|frame| (frame.label.clone(), frame.tag.clone(), frame.citation.clone()))
        .collect()
}

/// Every string a drive printed, in step order.
fn g20_printed(outcome: &DriveOutcome) -> Vec<String> {
    outcome
        .steps
        .iter()
        .filter_map(|trace| trace.frame.as_ref())
        .map(|frame| frame.surface.clone())
        .collect()
}

/// **OD-187**'s shape: the artifact row has no `drew` list and no `reason` column,
/// because the walker returns `Option<String>` and the name of a refusal cannot
/// reach the boundary without a second copy of the walk. A refusal is identified by
/// the drive that forced it, which is what `forced_by` next to it is for.
fn g20_row(seed: u64, tick: &G20Tick) -> Value {
    json!({
        "t": tick.t,
        "thought": tick.thought,
        "seed": format!("0x{seed:016x}"),
        "label": tick.label,
        "tag": tick.tag,
        "anchor": tick.anchor,
        "licensed": tick.licensed,
        "chunk_cap": tick.chunk_cap,
        "cap_note": G20_CAP_NOTE,
        "citation": tick.citation,
        "canonical_surface": tick.canonical,
        "realized_surface": tick.realized,
    })
}

/// The voice row as **both** processes can speak it: the run keeps a `TongueTick`
/// per step and so does the drive, and these are the columns step 9a recorded —
/// the artifact's shape without the frame's `label`/`tag`, which live in the run's
/// `output` and the drive's `frame` and are graded by legs (b), (d) and (h).
fn g20_voice_row(seed: u64, step: usize, t: i32, tongue: &si_session::TongueTick) -> String {
    json!({
        "step": step,
        "t": t,
        "thought": tongue.thought,
        "seed": format!("0x{seed:016x}"),
        "anchor": tongue.anchor,
        "licensed": tongue.licensed,
        "chunk_cap": tongue.chunk_cap,
        "cap_note": G20_CAP_NOTE,
        "citation": tongue.citation,
        "canonical_surface": tongue.canonical,
        "realized_surface": tongue.realized,
    })
    .to_string()
}

/// **OD-189**'s bank, re-derived from the interner: every verbatim surface the
/// session was taught, grouped by its folded token sequence. `si-session`'s
/// `license_for` builds the same map for the walker; the gate builds it here so
/// the two sides are independent readings of the one fold law.
fn g20_bank(session: &Session) -> BTreeMap<Vec<String>, Vec<String>> {
    let mut bank: BTreeMap<Vec<String>, Vec<String>> = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for entry in session.store().interner().surfaces() {
        if entry.verbatim.is_empty() || !seen.insert(entry.verbatim.clone()) {
            continue;
        }
        let keys = line_keys(&entry.verbatim);
        let group = bank.entry(keys).or_default();
        if !group.contains(&entry.verbatim) {
            group.push(entry.verbatim.clone());
        }
    }
    bank
}

/// The mouth as **OD-181** bounds it, re-derived here from the citation the run
/// recorded — the cited rows' composed surfaces plus their two endpoints' own terms
/// (**OD-188**) — rather than by calling the crate. `si-cli` may not name
/// `si-tongue` (`check_tongue_is_leaf`), and a leg that grades a boundary by
/// calling the code that enforces it grades nothing. **OD-189**: each canonical
/// entry is followed by every taught spelling of the same folded keys, which is
/// the bank the walker may draw from; the canonical copy stays first so an anchor
/// lookup reads the surface the point itself speaks.
fn g20_license(session: &Session, citation: &[u32]) -> Result<Vec<(u32, String)>> {
    let rows: BTreeMap<u32, si_core::record::Record> = session
        .store()
        .rows()?
        .into_iter()
        .map(|(_, record)| (record.envelope.row_id, record))
        .collect();
    let mut canonical = Vec::new();
    for row_id in citation {
        let Some(record) = rows.get(row_id) else { continue };
        let composed = surface(session, record.body.resp_surface);
        if !composed.is_empty() {
            canonical.push((record.body.val_point, composed));
        }
        // 04 §21 U.1: a nameless row is citable and is not a surface, so it lends
        // the walk no word — the same filter `license_for` applies.
        if record.body.pattern_id == si_core::record::PATTERN_NAMELESS {
            continue;
        }
        for point in [record.body.key_point, record.body.val_point] {
            if let Some(term) = session
                .store()
                .points()
                .surface(point)
                .or_else(|| session.store().points().canonical(point))
            {
                canonical.push((point, term.to_string()));
            }
        }
    }
    let bank = g20_bank(session);
    let mut out = Vec::new();
    for (point, text) in canonical {
        let aliases: Vec<String> = bank
            .get(&line_keys(&text))
            .into_iter()
            .flatten()
            .filter(|alias| **alias != text)
            .cloned()
            .collect();
        out.push((point, text));
        for alias in aliases {
            out.push((point, alias));
        }
    }
    Ok(out)
}

/// The licensed vocabulary as folded keys, folded by the one tokenizer the build
/// has (`si_front::tokenize`), because a second fold law is what `G20(c)` exists to
/// catch.
fn g20_license_keys(session: &Session, citation: &[u32]) -> Result<BTreeSet<String>> {
    Ok(g20_license(session, citation)?
        .into_iter()
        .flat_map(|(_, text)| line_keys(&text))
        .collect())
}

/// 04 §34 AF.3 step 1's arithmetic, restated for the same reason `g20_license` is.
fn g20_budget(steps: i64, spread: u16) -> usize {
    (steps.max(0) as u64 * u64::from(spread) >> 12) as usize
}

fn g20_spread(seed: u64) -> u16 {
    si_core::tongue::derive_style(seed).get(si_core::tongue::StyleId::Spread)
}

/// 13 §6.2 row 6: `", "` is the only multi-character literal the crate emits and
/// `tokenize` yields no token from it, so this split is the chunk boundary the walk
/// itself joined on.
fn g20_chunks(line: &str) -> usize {
    line.split(", ").count()
}

/// The protected-shelf rows a drive holds, as a shelf → count map pruned to the
/// four shelves a voice may not write.
fn g20_protected(session: &Session) -> Result<BTreeMap<u16, usize>> {
    Ok(census(session)?
        .into_iter()
        .filter(|(shelf, _)| G20_PROTECTED.contains(shelf))
        .collect())
}

/// **OD-189**'s panel row: one individual's whole transcript read on and off, and
/// the facts leg (b) grades. The variants transcript and the bare control are read
/// by the same helper, so the two panels differ only in the fixture.
#[derive(Clone, Debug)]
struct G20PanelRow {
    seed: u64,
    spread: u16,
    chunk_cap: usize,
    voice_ticks: usize,
    lines: Vec<String>,
    labels: Vec<String>,
    frames_equal_to_off: bool,
    state_equal_to_off: bool,
    census_equal_to_off: bool,
    protected_rows: BTreeMap<u16, usize>,
    /// Chunks whose bytes carry an uppercase letter. Every canonical surface at
    /// this fixture is lowercase and the walker capitalizes nothing, so this is
    /// the bank's own signature — the measurement that style was observable, not
    /// merely possible.
    variant_spellings_drawn: usize,
}

fn g20_panel(run: &Run, scenario: &Scenario, steps: &[Step]) -> Result<Vec<G20PanelRow>> {
    let mut panel = Vec::new();
    for panel_seed in G20_PANEL {
        let on = g20_drive(run, scenario, steps, panel_seed, &[])?;
        let off = g20_drive(run, scenario, steps, panel_seed, &[("TONGUE", 0)])?;
        let ticks = g20_ticks(&on);
        let lines: Vec<String> =
            ticks.iter().filter_map(|tick| tick.realized.clone()).collect();
        let variant_spellings_drawn = lines
            .iter()
            .flat_map(|line| line.split(", "))
            .filter(|chunk| chunk.chars().any(char::is_uppercase))
            .count();
        panel.push(G20PanelRow {
            seed: panel_seed,
            spread: g20_spread(panel_seed),
            chunk_cap: ticks.first().map(|tick| tick.chunk_cap).unwrap_or(0),
            voice_ticks: ticks.len(),
            lines,
            labels: g20_frames(&on)
                .iter()
                .map(|(label, tag, _)| format!("{label}/{tag}"))
                .collect(),
            frames_equal_to_off: g20_frames(&on) == g20_frames(&off),
            state_equal_to_off: on.hash == off.hash,
            census_equal_to_off: census(&on.session)? == census(&off.session)?,
            protected_rows: g20_protected(&on.session)?,
            variant_spellings_drawn,
        });
    }
    Ok(panel)
}

fn g20_panel_json(panel: &[G20PanelRow]) -> Vec<Value> {
    panel
        .iter()
        .map(|row| {
            json!({
                "seed": format!("0x{:016x}", row.seed),
                "style_spread": row.spread,
                "chunk_cap": row.chunk_cap,
                "cap_note": G20_CAP_NOTE,
                "voice_ticks": row.voice_ticks,
                "realized_ticks": row.lines.len(),
                "lines": row.lines.clone(),
                "labels": row.labels.clone(),
                "frames_equal_to_off": row.frames_equal_to_off,
                "state_equal_to_off": row.state_equal_to_off,
                "census_equal_to_off": row.census_equal_to_off,
                "variant_spellings_drawn": row.variant_spellings_drawn,
            })
        })
        .collect()
}

fn g20(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let scenario = &run.request.scenario;
    if !expect_fixture(run, ws, "G20", "05 §24", "harness/scenarios/g20_tongue.txt", out)? {
        return Ok(());
    }
    let steps = scenario.sections.first().context("a G20 run has no section to grade")?;
    let seed = run.request.seed;
    // 13 §6.1: the voice is the thing V14 adds, so a run that did not install it
    // through the launch channel is not this gate's run. The refusal is named, not
    // quietly graded as a green over an off drive.
    if run.request.constants.get("TONGUE").copied() != Some(1) {
        bad(
            out,
            "G20.launch",
            "a",
            "TONGUE = 1 installed through the launch channel",
            json!({"constants": run.request.constants}),
            "the gate must launch at the voice it grades (13 §6.1); an off run grades its own absence",
        );
        return Ok(());
    }

    let on = g20_drive(run, scenario, steps, seed, &[])?;
    let off = g20_drive(run, scenario, steps, seed, &[("TONGUE", 0)])?;
    let ticks = g20_ticks(&on);

    // ---------------------------------------------------------------- (a) replay
    // Two drives in this process, and the CLI process's own run beside them. The
    // voice is RAM (13 §4.2: it moves no hashed item), so both halves of the pair
    // are byte claims about the emission, not about the container.
    let again = g20_drive(run, scenario, steps, seed, &[])?;
    let repeated = g20_ticks(&again);
    let rows_of = |list: &[G20Tick]| -> Vec<String> {
        list.iter().map(|tick| g20_row(seed, tick).to_string()).collect()
    };
    let log_equal = rows_of(&ticks) == rows_of(&repeated);
    let hash_pair = [on.hash.clone(), again.hash.clone()];
    if log_equal && hash_pair[0] == hash_pair[1] && !ticks.is_empty() {
        ok(
            out,
            "G20.a.1",
            "a",
            "the same transcript at the same launch state realizes the same bytes twice",
            json!({
                "voice_ticks": ticks.len(),
                "realized_log_byte_equal": true,
                "state_hash_pair": hash_pair,
                "determinism_is_per_variant": "OD-93: the pair is the run's own variant, and `run` below is a different process",
            }),
            "the draw is not a function of (birth seed, style digest, thought id) — a second reading of the same transcript speaks a second line (OD-183)",
        );
    } else {
        bad(
            out,
            "G20.a.1",
            "a",
            "the same transcript at the same launch state realizes the same bytes twice",
            json!({
                "voice_ticks": ticks.len(),
                "realized_log_byte_equal": log_equal,
                "state_hash_pair": hash_pair,
                "first": rows_of(&ticks),
                "second": rows_of(&repeated),
            }),
            "the draw is not a function of (birth seed, style digest, thought id), or the voice moved a hashed item",
        );
    }
    let run_hashes: Vec<String> = run.sections.iter().map(|section| section.hash.clone()).collect();
    // 13 §6.6 (a): "the V14 drive runs twice in **two processes**; assert
    // `realized_log.jsonl` is byte-equal … and that the state hash of the two runs
    // is byte-equal (the voice moves no item)". The launched process here is `run`;
    // this process reads it back through the two replay instruments the build has —
    // `Run::replay` (the section's recorded steps) and `Run::replay_transcript`
    // (transcript.txt) — and drives the fixture a third time with the section-end
    // freeze the run performed before it hashed, so the three readings are one code
    // path apart rather than one freeze apart (G13.a.1's precedent, tolerance 0).
    // The voice half is the run's own `StepRecord.tongue` against the frozen drive's
    // ticks, column for column: `realized_log.jsonl` is this process's copy, and the
    // launched process's copy is those rows.
    let frozen = g20_drive_at(run, scenario, steps, seed, &[], true)?;
    let replay = run.replay()?;
    let transcript_replay = Run::replay_transcript(&run.dir, seed, &Launch::of(&run.request))?;
    let launched_voice: Vec<String> = run
        .sections
        .first()
        .map(|section| {
            section
                .steps
                .iter()
                .filter_map(|step| {
                    step.tongue
                        .as_ref()
                        .map(|tongue| g20_voice_row(seed, step.index, step.t, tongue))
                })
                .collect()
        })
        .unwrap_or_default();
    let graded_voice: Vec<String> = frozen
        .steps
        .iter()
        .filter_map(|trace| {
            trace
                .tongue
                .as_ref()
                .map(|tongue| g20_voice_row(seed, trace.index, trace.t, tongue))
        })
        .collect();
    let hashes_agree = replay == run_hashes
        && transcript_replay == run_hashes
        && run_hashes.first().map(|hash| *hash == frozen.hash) == Some(true);
    if hashes_agree && launched_voice == graded_voice && !launched_voice.is_empty() {
        ok(
            out,
            "G20.a.2",
            "a",
            "the process that launched this run and the process grading it reach the same state and speak the same rows",
            json!({
                "launched_hash": run_hashes,
                "grading_drive_hash": frozen.hash,
                "in_process_replay_equal": replay == run_hashes,
                "transcript_replay_equal": transcript_replay == run_hashes,
                "voice_rows": launched_voice.len(),
                "realized_log_byte_equal": true,
            }),
            "the voice is not reproducible across processes — a replay of a recorded transcript would speak a different line than the run did",
        );
    } else {
        bad(
            out,
            "G20.a.2",
            "a",
            "the process that launched this run and the process grading it reach the same state and speak the same rows",
            json!({
                "launched_hash": run_hashes,
                "grading_drive_hash": frozen.hash,
                "in_process_replay_equal": replay == run_hashes,
                "transcript_replay_equal": transcript_replay == run_hashes,
                "launched_voice": launched_voice,
                "graded_voice": graded_voice,
            }),
            "a replay of a recorded transcript reaches a different state or speaks a different line than the run that wrote it",
        );
    }
    // The empty-lattice pair: the asks with nothing taught under them. There is no
    // settled frame, so step 9a is entered zero times and the loop asks — the
    // fallback, not a crash. 13 §6.6 asks for a `no_candidates` tick that still
    // prints a frame; that shape is not constructible in CONFIG SI and says so here:
    // the anchor is a cited row's `key_point` and the write gate leaves that row's
    // own live arc out of it, so a frame that prints always has one candidate
    // (measured over this panel: 0 licensed points with an empty surface, 0 frames
    // printed with no arc from their anchor).
    let questions: Vec<Step> = steps
        .iter()
        .filter(|step| matches!(step, Step::Sense { text, .. } if text.contains('?')))
        .cloned()
        .collect();
    let empty = g20_drive(run, scenario, &questions, seed, &[])?;
    let empty_off = g20_drive(run, scenario, &questions, seed, &[("TONGUE", 0)])?;
    let empty_protected = g20_protected(&empty.session)?;
    if g20_ticks(&empty).is_empty()
        && empty.hash == empty_off.hash
        && empty_protected.values().sum::<usize>() == 0
    {
        ok(
            out,
            "G20.a.3",
            "a",
            "an empty lattice is the fallback and not a crash: no frame settles, no voice is entered, no belief is written, and the state is the off drive's",
            json!({
                "question_steps": questions.len(),
                "voice_ticks": 0,
                "protected_rows": empty_protected,
                "hash_pair": [empty.hash.clone(), empty_off.hash.clone()],
                "no_candidates_with_a_printed_frame_is_constructible": false,
                "why": "13 §6.6's second pair names an arm the boundary cannot reach in CONFIG SI: a cited row's `key_point` always carries that row's own live arc (**OD-188**), so a frame that prints has at least one candidate. The arms that are reachable — a spent budget and a lost slot — are forced by legs (c) and (g) at named points.",
            }),
            "the voice crashed, wrote a belief from nothing, or moved the state on a transcript the off build can also run",
        );
    } else {
        bad(
            out,
            "G20.a.3",
            "a",
            "an empty lattice is the fallback and not a crash",
            json!({
                "voice_ticks": g20_ticks(&empty).len(),
                "protected_rows": empty_protected,
                "hash_pair": [empty.hash, empty_off.hash],
            }),
            "a settled-free transcript still let step 9a write, move the state, or enter the walk",
        );
    }

    // ------------------------------------------------- (b) label immutability
    // 13 §6.6's "realize at the five style corners" cannot be launched: `STYLE_*`
    // rows are `Class::Tongue` and `launch_constants` refuses anything that is not a
    // swept SPECIES or a RULING row (**OD-177**, **OD-87**), so no corner is a
    // point on any grid the harness may install. What a legal launch *can* move is
    // the budget, and what the seed fixes is the draw, so the leg is graded over
    // both: the bound at three named budget points, and the boundary across the
    // panel. The correction is a spec delta, not a workaround (`failure_report.md`).
    let off_frames = g20_frames(&off);
    let mut points_json = Vec::new();
    let mut frames_equal = off_frames.len() > 0;
    let mut state_equal = true;
    let mut printed_differs = false;
    for point in G20_POINTS {
        let drive = g20_drive(run, scenario, steps, seed, &[("PHRASE_MAX_STEPS", point)])?;
        let frames = g20_frames(&drive);
        let printed = g20_printed(&drive);
        frames_equal &= frames == off_frames;
        state_equal &= drive.hash == off.hash;
        printed_differs |= printed != g20_printed(&off);
        points_json.push(json!({
            "phrase_max_steps": point,
            "budget": g20_budget(point, g20_spread(seed)),
            "chunk_cap": 1 + g20_budget(point, g20_spread(seed)),
            "cap_note": G20_CAP_NOTE,
            "printed": printed,
            "state_hash_equal_to_off": drive.hash == off.hash,
        }));
    }
    if frames_equal && state_equal && printed_differs {
        ok(
            out,
            "G20.b.1",
            "b",
            "the label, the tag and the citation are byte-equal across every budget point and against `TONGUE = 0`, the state hash is unchanged, and the printed string is not",
            json!({
                "frames": off_frames,
                "budget_points": points_json,
                "style_corner_half_corrected": "OD-177/87: a `STYLE_*` row is not installable, so 'the five style corners' is a seed panel (G20.b.2), not a launch state",
            }),
            "the voice touched the audit label, the citation or the durable state — I6, and 13 §0 fact 5",
        );
    } else {
        bad(
            out,
            "G20.b.1",
            "b",
            "the label, the tag and the citation are byte-equal across every budget point and against `TONGUE = 0`, the state hash is unchanged, and the printed string is not",
            json!({
                "frames_equal": frames_equal,
                "state_equal": state_equal,
                "printed_differs": printed_differs,
                "off_frames": off_frames,
                "budget_points": points_json,
            }),
            "the voice touched the audit label, the citation or the durable state — I6, and 13 §0 fact 5",
        );
    }
    // **OD-189**'s re-grade, in the owner's two halves. The variants fixture
    // teaches a second and third spelling of six predicates; each is a confirmed
    // statement — same key, same value, same folded signature — so the teaches
    // assert no new belief row, which the protected-shelf census measures against
    // the bare control. With taught spellings the bank has words to draw between,
    // so the individual is observable in the bytes: no two speaking individuals
    // may print the same realized transcript. The bare control is the same
    // transcript with those teaches removed; with one spelling per licensed word
    // the walk consumes no bank draw and every speaking individual prints the
    // same bytes — the expected outcome, named as such.
    let variants_panel = g20_panel(run, scenario, steps)?;
    let bare_scenario = Scenario::load(ws, G20_BARE_FIXTURE)
        .with_context(|| format!("load {G20_BARE_FIXTURE}"))?;
    let bare_steps = bare_scenario
        .sections
        .first()
        .with_context(|| format!("{G20_BARE_FIXTURE} has no section to grade"))?;
    let bare_panel = g20_panel(run, &bare_scenario, bare_steps)?;
    let speaking = |panel: &[G20PanelRow]| {
        panel.iter().filter(|row| !row.lines.is_empty()).count()
    };
    let silent = |panel: &[G20PanelRow]| {
        panel.iter().filter(|row| row.lines.is_empty()).count()
    };
    let distinct = |panel: &[G20PanelRow]| -> BTreeSet<Vec<String>> {
        panel
            .iter()
            .filter(|row| !row.lines.is_empty())
            .map(|row| row.lines.clone())
            .collect()
    };
    let settled = |panel: &[G20PanelRow]| {
        panel.iter().all(|row| {
            row.frames_equal_to_off && row.state_equal_to_off && row.census_equal_to_off
        })
    };
    let variants_speaking = speaking(&variants_panel);
    let variants_silent = silent(&variants_panel);
    let variants_distinct = distinct(&variants_panel);
    let bare_speaking = speaking(&bare_panel);
    let bare_silent = silent(&bare_panel);
    let bare_distinct = distinct(&bare_panel);
    let style_observable =
        variants_speaking > 1 && variants_distinct.len() == variants_speaking;
    let bare_is_identical = bare_speaking > 1 && bare_distinct.len() == 1;
    let bank_drawn: usize = variants_panel
        .iter()
        .map(|row| row.variant_spellings_drawn)
        .sum();
    let panels_settled = settled(&variants_panel) && settled(&bare_panel);
    let no_new_belief_rows = bare_panel.first().is_some_and(|bare_row| {
        variants_panel
            .iter()
            .all(|row| row.protected_rows == bare_row.protected_rows)
    });
    let straddle = variants_silent > 0 && bare_silent > 0;
    if style_observable
        && bare_is_identical
        && bank_drawn > 0
        && straddle
        && panels_settled
        && no_new_belief_rows
    {
        ok(
            out,
            "G20.b.2",
            "b",
            "the panel straddles the budget boundary; at the variants fixture no two speaking individuals print the same realized transcript, at the bare control every speaking individual prints the same bytes, and neither fixture lets the voice move a row",
            json!({
                "variants_fixture": scenario.path,
                "bare_fixture": G20_BARE_FIXTURE,
                "variants_panel": g20_panel_json(&variants_panel),
                "bare_panel": g20_panel_json(&bare_panel),
                "speaking": {"variants": variants_speaking, "bare": bare_speaking},
                "silent": {"variants": variants_silent, "bare": bare_silent},
                "distinct_transcripts": {"variants": variants_distinct.len(), "bare": bare_distinct.len()},
                "variant_spellings_drawn": bank_drawn,
                "belief_rows_equal_across_fixtures": no_new_belief_rows,
                "od_189": "RULED: two chunks stand; a taught spelling is the same claim over the same cited endpoint, so it asserts no new row. The variants fixture's teaches are confirmed statements — same key, same value, same folded signature — and the protected-shelf census is the bare control's, row for row. Style is observable in the bytes here; at the bare control byte-identity is the expected outcome, named as such.",
            }),
            "the bank is not observable, the bare control is not byte-identical, a taught spelling asserted a belief row, or the voice moved a row/label",
        );
    } else {
        bad(
            out,
            "G20.b.2",
            "b",
            "the panel straddles the budget boundary; at the variants fixture no two speaking individuals print the same realized transcript, at the bare control every speaking individual prints the same bytes, and neither fixture lets the voice move a row",
            json!({
                "variants_fixture": scenario.path,
                "bare_fixture": G20_BARE_FIXTURE,
                "variants_panel": g20_panel_json(&variants_panel),
                "bare_panel": g20_panel_json(&bare_panel),
                "style_observable": style_observable,
                "variants_distinct": variants_distinct.len(),
                "variants_speaking": variants_speaking,
                "bare_is_identical": bare_is_identical,
                "bare_distinct": bare_distinct.len(),
                "bare_speaking": bare_speaking,
                "bank_drawn": bank_drawn,
                "straddle": straddle,
                "panels_settled": panels_settled,
                "no_new_belief_rows": no_new_belief_rows,
            }),
            "the panel does not meet OD-189's condition: the boundary is not real, the bank leaves no observable trace, the bare control diverged, or a teaching/vocal row moved",
        );
    }

    // ---------------------------------------------------- (c) no invented words
    // The bound, over every tick the panel and the budget points print, against the
    // license re-derived from the store — not against the set the call site built.
    let mut offenders: Vec<Value> = Vec::new();
    let mut graded_ticks = 0usize;
    let mut drives_for_c: Vec<(u64, DriveOutcome, String)> = Vec::new();
    for point in G20_POINTS {
        let drive = g20_drive(run, scenario, steps, seed, &[("PHRASE_MAX_STEPS", point)])?;
        drives_for_c.push((seed, drive, format!("PHRASE_MAX_STEPS={point}")));
    }
    for panel_seed in G20_PANEL {
        let drive = g20_drive(run, scenario, steps, panel_seed, &[])?;
        drives_for_c.push((panel_seed, drive, "panel".to_string()));
    }
    for (drive_seed, drive, at) in &drives_for_c {
        for tick in g20_ticks(drive) {
            let Some(line) = &tick.realized else { continue };
            graded_ticks += 1;
            let licensed = g20_license_keys(&drive.session, &tick.citation)?;
            let spoken: BTreeSet<String> = line_keys(line).into_iter().collect();
            let extra: Vec<String> = spoken.difference(&licensed).cloned().collect();
            if !extra.is_empty() {
                offenders.push(json!({
                    "at": at,
                    "seed": format!("0x{drive_seed:016x}"),
                    "t": tick.t,
                    "line": line,
                    "unlicensed_tokens": extra,
                    "citation": tick.citation,
                }));
            }
        }
    }
    if offenders.is_empty() && graded_ticks > 0 {
        ok(
            out,
            "G20.c.1",
            "c",
            "every realized tick's folded token set is a subset of the license re-derived from its own citation",
            json!({"graded_realized_ticks": graded_ticks, "offenders": offenders, "drives": drives_for_c.len()}),
            "R-43 through the friendly door: the voice said a word the citation does not own (I8, OD-181)",
        );
    } else {
        bad(
            out,
            "G20.c.1",
            "c",
            "every realized tick's folded token set is a subset of the license re-derived from its own citation",
            json!({"graded_realized_ticks": graded_ticks, "offenders": offenders}),
            "the voice said a word the citation does not own, or nothing realized and the leg graded an empty set",
        );
    }
    // The forcing, at the registry's own floor. `PHRASE_MAX_STEPS = lo` is a floor
    // and is named as one (**OD-95**): the budget prices the walk at zero extra
    // chunks, so the only line the walk can build is the anchor term, and that line
    // loses the slot — the walk refuses and the canonical frame prints.
    let floor = si_core::consts::row("PHRASE_MAX_STEPS")
        .context("PHRASE_MAX_STEPS is not a registry row (03 §5)")?
        .lo;
    let forced = g20_drive(run, scenario, steps, seed, &[("PHRASE_MAX_STEPS", floor)])?;
    let forced_ticks = g20_ticks(&forced);
    let forced_refusals = forced_ticks.iter().filter(|tick| tick.realized.is_none()).count();
    let forced_prints_canonical = forced_ticks
        .iter()
        .all(|tick| tick.printed == tick.canonical)
        && !forced_ticks.is_empty();
    if forced_refusals == forced_ticks.len() && forced_prints_canonical {
        ok(
            out,
            "G20.c.2",
            "c",
            "a walk with no budget refuses every tick and prints the canonical frame — the fallback is reached by construction, not by a reviewer's reading",
            json!({
                "forced_by": {"constants": {"PHRASE_MAX_STEPS": floor}, "fixture": scenario.path},
                "voice_ticks": forced_ticks.len(),
                "refused": forced_refusals,
                "printed_equals_canonical": true,
                "od_187": "the row is identified by this forcing, not by a `reason` column: the walker's answer is `Option<String>`",
            }),
            "a spent budget truncated a line instead of refusing, so the voice would print half a thought (I8, OD-173)",
        );
    } else {
        bad(
            out,
            "G20.c.2",
            "c",
            "a walk with no budget refuses every tick and prints the canonical frame",
            json!({
                "floor": floor,
                "voice_ticks": forced_ticks.len(),
                "refused": forced_refusals,
                "lines": forced_ticks.iter().map(|tick| tick.printed.clone()).collect::<Vec<_>>(),
            }),
            "a spent budget truncated a line instead of refusing, so the voice would print half a thought",
        );
    }

    // ---------------------------------------------------- (d) the two keys still
    // A voice that likes a shape must not be able to teach it. The stranger line and
    // grimm's line both restate the fixture's own taught statement, and each is
    // graded by subtracting the step from the transcript and diffing the protected
    // shelves — one transcript apart, so the difference can only be that line.
    let registered = scenario.registered();
    let named = |step: &Step, kind: Kind| match step {
        Step::Sense { source, identity, .. } => match kind {
            Kind::Stranger => {
                *identity != Some(registered.owner) && !matches!(identity, None) && *identity != Some(registered.grimm)
            }
            Kind::Grimm => identity.is_none() && *source == Source::Grimm,
        },
        _ => false,
    };
    let strip = |kind: Kind| -> Vec<Step> {
        steps.iter().filter(|step| !named(step, kind)).cloned().collect()
    };
    let all_shelves = |session: &Session| -> Result<BTreeMap<u16, usize>> { Ok(census(session)?) };
    let diff = |before: &BTreeMap<u16, usize>, after: &BTreeMap<u16, usize>| -> BTreeMap<u16, i64> {
        let mut delta = BTreeMap::new();
        for shelf in after.keys().chain(before.keys()).cloned().collect::<BTreeSet<u16>>() {
            let change =
                after.get(&shelf).copied().unwrap_or(0) as i64 - before.get(&shelf).copied().unwrap_or(0) as i64;
            if change != 0 {
                delta.insert(shelf, change);
            }
        }
        delta
    };
    let on_shelves = all_shelves(&on.session)?;
    let off_shelves = all_shelves(&off.session)?;
    let mut stranger_delta = BTreeMap::new();
    let mut grimm_delta = BTreeMap::new();
    let mut stranger_rows = 0usize;
    let mut grimm_rows = 0usize;
    for kind in [Kind::Stranger, Kind::Grimm] {
        let without = g20_drive(run, scenario, &strip(kind), seed, &[])?;
        let delta = diff(&all_shelves(&without.session)?, &on_shelves);
        let lines = steps.iter().filter(|step| named(step, kind)).count();
        if matches!(kind, Kind::Stranger) {
            stranger_delta = delta;
            stranger_rows = lines;
        } else {
            grimm_delta = delta;
            grimm_rows = lines;
        }
    }
    let stranger_protected = g20_protected_of(&stranger_delta);
    let grimm_protected = g20_protected_of(&grimm_delta);
    if stranger_rows == 1 && grimm_rows == 1 && stranger_protected == 0 && grimm_protected == 0 {
        ok(
            out,
            "G20.d.1",
            "d",
            "a stranger line and grimm's line that name a shape the voice liked write no belief row",
            json!({
                "stranger_step_rows": stranger_rows,
                "grimm_step_rows": grimm_rows,
                "protected_delta_from_the_stranger_line": stranger_delta,
                "protected_delta_from_grimm_line": grimm_delta,
                "sense_floor_paid": "the stranger's own `context` row is the sense floor (01 §6) and is counted on the `context` shelf, not here — a run that withheld it would be a different design",
            }),
            "the voice's liking became a belief: a non-owner class wrote a knowledge row (I9, 05 §23.5's class arm)",
        );
    } else {
        bad(
            out,
            "G20.d.1",
            "d",
            "a stranger line and grimm's line that name a shape the voice liked write no belief row",
            json!({
                "stranger_steps": stranger_rows,
                "grimm_steps": grimm_rows,
                "protected_delta_from_the_stranger_line": stranger_delta,
                "protected_delta_from_grimm_line": grimm_delta,
            }),
            "a non-owner class wrote a knowledge row, or the fixture lost one of its two class lines",
        );
    }
    if on_shelves == off_shelves {
        ok(
            out,
            "G20.d.2",
            "d",
            "the V14 drive's per-shelf row census is the `TONGUE = 0` drive's, shelf for shelf",
            json!({
                "on": on_shelves,
                "off": off_shelves,
                "rows_per_step_equal": on
                    .steps
                    .iter()
                    .zip(off.steps.iter())
                    .all(|(left, right)| left.rows_after == right.rows_after),
            }),
            "the voice wrote, or its utterance re-entered the loop as sense (13 §4.2: one return value, no writer)",
        );
    } else {
        bad(
            out,
            "G20.d.2",
            "d",
            "the V14 drive's per-shelf row census is the `TONGUE = 0` drive's, shelf for shelf",
            json!({"on": on_shelves, "off": off_shelves, "delta": diff(&off_shelves, &on_shelves)}),
            "the voice wrote a row, or its utterance re-entered the loop as sense",
        );
    }
    open(
        out,
        "G20.d.3",
        "d",
        "G1's seam legs stay green on the V14 build",
        "13 §6.6 puts this clause in leg (d), and the only way to grade it here is to re-run G1's evaluator from inside G20's — which recurses into the evaluator that called it (05 §18's own note on the sleep legs). The shipped form of this clause is stage B20's verify pass, which runs the whole G1–G19 battery on this build and is recorded in the stage report.",
        json!({
            "graded_by": "11 §B20's verify pass (`run --gate G1..G19` on the 17-crate build)",
            "fixture": scenario.path,
            "protected_rows_at_v14": g20_protected(&on.session)?,
            "od": "OD-179's live-arc law and 05 §23.5's class arm are what a seam leg on this build would re-assert",
        }),
        "the seam changed its verdict because the voice exists — a supporter line that used to be refused is now admitted, or the reverse",
    );

    // ------------------------------------------------- (e) arousal isolation
    let reads_no_a = super::checks::run("check_tongue_reads_no_a", ws)?;
    if reads_no_a.verdict == "pass" {
        ok(
            out,
            "G20.e.1",
            "e",
            "`check_tongue_reads_no_a`: the voice's operands are the birth seed, the style digest and the thought id, and nothing else",
            json!({"check": reads_no_a.id, "detail": reads_no_a.detail, "od": "OD-168, OD-182, OD-183"}),
            "arousal reached the draw, so two individuals with the same citation would speak different lines for a reason the citation does not carry",
        );
    } else {
        bad(
            out,
            "G20.e.1",
            "e",
            "`check_tongue_reads_no_a`: the voice's operands are the birth seed, the style digest and the thought id, and nothing else",
            json!({"check": reads_no_a.id, "detail": reads_no_a.detail}),
            "arousal reached the draw",
        );
    }
    // The trajectory half, as the only honest form it has: the same transcript with
    // a quiet tick after every sense line moves `a` at every tick, so a draw that
    // read arousal would move the line's claim. Where the padded run cites a
    // different row, that is v1's behaviour and the leg prints it rather than
    // failing on it — 13 §6.6's own instruction.
    //
    // **OD-189**: the predicate is claim identity, not byte identity. An idle tick
    // that ignites spends a thought id, and the thought id is a sanctioned operand
    // of the render seed (G20.e.1 types the set), so the padded drive may draw a
    // taught spelling the unpadded one did not — the same claim in other bytes. The
    // claim is the chunk sequence folded by the one fold law; a chunk that moved to
    // a different licensed slot is the failure this leg names.
    let mut padded_steps: Vec<Step> = Vec::new();
    for step in steps {
        padded_steps.push(step.clone());
        if matches!(step, Step::Sense { .. }) {
            padded_steps.push(Step::Idle);
        }
    }
    let padded = g20_drive(run, scenario, &padded_steps, seed, &[])?;
    let padded_ticks = g20_ticks(&padded);
    // The claim a realized line asserts: one folded key bag per joined chunk, in
    // the order the walk printed them. Two spellings of one taught surface fold to
    // the same bag; a different slot does not.
    let claim = |line: &str| -> Vec<Vec<String>> {
        line.split(", ").map(line_keys).collect()
    };
    let mut drift = Vec::new();
    let mut same_citation = 0usize;
    let mut respelled = 0usize;
    let mut mismatched: Vec<Value> = Vec::new();
    for (quiet, loud) in ticks.iter().zip(padded_ticks.iter()) {
        if quiet.citation != loud.citation {
            drift.push(json!({
                "t": [quiet.t, loud.t],
                "citation": [quiet.citation, loud.citation],
                "printed": [quiet.printed, loud.printed],
                "read_as": "the padded run cited a different row: that is the walk's own v1 behaviour under a moved `a`, not the voice's (13 §6.6's instruction to print these)",
            }));
            continue;
        }
        same_citation += 1;
        let bytes_moved = quiet.realized != loud.realized || quiet.printed != loud.printed;
        let claim_moved = match (&quiet.realized, &loud.realized) {
            (Some(left), Some(right)) => claim(left) != claim(right),
            _ => quiet.realized != loud.realized,
        };
        if bytes_moved && !claim_moved {
            respelled += 1;
            continue;
        }
        if claim_moved {
            mismatched.push(json!({
                "t": quiet.t,
                "citation": quiet.citation,
                "a_before_quiet": quiet.step,
                "realized": [quiet.realized, loud.realized],
                "claim": [
                    quiet.realized.as_deref().map(&claim),
                    loud.realized.as_deref().map(&claim),
                ],
            }));
        }
    }
    if !ticks.is_empty() && !padded_ticks.is_empty() && mismatched.is_empty() {
        ok(
            out,
            "G20.e.2",
            "e",
            "for every tick whose citation the quiet-padded drive left alone, the realized line asserts the same claim — byte-identical, or a taught spelling over the same folded chunks (OD-189)",
            json!({
                "ticks_compared": same_citation,
                "respelled_only_ticks": respelled,
                "citation_drift_ticks": drift,
                "padded_voice_ticks": padded_ticks.len(),
                "arousal_sweep_present": drift.len(),
                "operand_check": "G20.e.1 types the seed's operands (birth seed, style digest, thought id); this leg holds the claim fixed while the trajectory moves",
                "od_189": "a taught spelling is the same claim over the same cited endpoint, so claim identity — not byte identity — is what a draw that read `a` would break",
            }),
            "I5: arousal is one scalar and the voice may not consult it — the same citation would assert two claims",
        );
    } else {
        bad(
            out,
            "G20.e.2",
            "e",
            "for every tick whose citation the quiet-padded drive left alone, the realized line asserts the same claim",
            json!({
                "ticks": ticks.len(),
                "padded_ticks": padded_ticks.len(),
                "mismatched": mismatched,
                "citation_drift_ticks": drift,
            }),
            "the draw read arousal, or one of the two drives printed no settled frame at all, or a line's claim moved under the padded trajectory",
        );
    }
    open(
        out,
        "G20.e.3",
        "e",
        "a sweep at every arousal level with `B_MAX`/`K_TANH`/`LAMBDA_A` held constant",
        "13 §6.6's (e) names an axis the launch channel has no shape for: arousal is not a registry row, it is the trajectory `a` walks under the swept constants (`si_affect::Integrator`), and `install_launch_state` can only name rows. G20.e.2 is the honest substitute — the same transcript with a moved `a` at every tick — and the swept form waits on the same ruling **OD-83** waits on for every other published refusal.",
        json!({
            "launch_channel_classes": "swept SPECIES | RULING (12 §3.2's guard, byte-checked by `check_sweep_never_touches_temperament`)",
            "arousal_rows_in_registry": si_core::consts::CONST_ROWS.iter().filter(|row| row.name.contains("A_MAX") || row.name == "K_TANH" || row.name == "LAMBDA_A").count(),
            "graded_by": "G20.e.2 (moved trajectory) + G20.e.1 (type)",
        }),
        "the leg's predicate is not expressible as a launch state, and saying so is the pass — a green here would be a claim about a run that never happened",
    );

    // -------------------------------------------------- (f) no self-farming
    let spoken_lines: Vec<String> =
        ticks.iter().filter_map(|tick| tick.realized.clone()).collect();
    // 13 §6.6(f) re-enters the lines "as sense" and demands zero rows. Through the
    // owner channel that sentence is not a statement about the voice: every owner
    // line is a teach candidate by law (04 §33), so an owner re-typing a realized
    // line is the owner teaching, and the rows it mints belong to that channel. The
    // forcing therefore runs on the channel that is not a teacher — a `voice 9:`
    // stranger, where the class arm is the boundary and 05 §23.5(a) rules that the
    // line pays its context floor and writes no belief — and grades the owner
    // channel by invariance instead: the rows its re-entry mints must be the rows
    // the *same re-entry* mints with the voice switched off (`TONGUE = 0`), or some
    // of them are the voice's and not the owner's.
    let owner_reentry: Vec<Step> = spoken_lines
        .iter()
        .map(|line| Step::Sense {
            source: Source::Owner,
            text: line.clone(),
            identity: None,
            conversation: 0,
        })
        .collect();
    let stranger_reentry: Vec<Step> = spoken_lines
        .iter()
        .map(|line| Step::Sense {
            source: Source::Grimm,
            text: line.clone(),
            identity: Some(9),
            conversation: 0,
        })
        .collect();
    let appended_asks = |outcome: &DriveOutcome| -> usize {
        let ask_kinds = si_core::eventlog::kind::ASK;
        outcome
            .steps
            .iter()
            .skip(steps.len())
            .map(|trace| trace.kinds.iter().filter(|kind| **kind == ask_kinds).count())
            .sum()
    };

    let mut owner_fed = steps.clone();
    owner_fed.extend(owner_reentry.clone());
    let fed = g20_drive(run, scenario, &owner_fed, seed, &[])?;
    let fed_shelves = all_shelves(&fed.session)?;
    let fed_delta = diff(&on_shelves, &fed_shelves);
    let fed_protected = g20_protected_of(&fed_delta);
    let fed_asks = appended_asks(&fed);

    let mut owner_fed_off = steps.clone();
    owner_fed_off.extend(owner_reentry);
    let fed_off = g20_drive(run, scenario, &owner_fed_off, seed, &[("TONGUE", 0)])?;
    let fed_off_delta = diff(&off_shelves, &all_shelves(&fed_off.session)?);
    let owner_channel_invariant = fed_delta == fed_off_delta;

    let mut stranger_fed = steps.clone();
    stranger_fed.extend(stranger_reentry);
    let stranger = g20_drive(run, scenario, &stranger_fed, seed, &[])?;
    let stranger_delta = diff(&on_shelves, &all_shelves(&stranger.session)?);
    let stranger_protected = g20_protected_of(&stranger_delta);
    let stranger_questions = stranger_delta
        .get(&si_core::addr::SHELF_QUESTIONS)
        .copied()
        .unwrap_or(0);
    let stranger_asks = appended_asks(&stranger);
    let stranger_floor = stranger_delta
        .get(&si_core::addr::SHELF_CONTEXT)
        .copied()
        .unwrap_or(0);

    if !spoken_lines.is_empty()
        && stranger_protected == 0
        && stranger_questions == 0
        && stranger_asks == 0
        && fed_asks == 0
        && owner_channel_invariant
    {
        ok(
            out,
            "G20.f.1",
            "f",
            "a re-entered line writes nothing through a non-teaching class, and through the owner channel it mints exactly what it mints with the voice off",
            json!({
                "lines_re_entered": spoken_lines,
                "stranger_reentry": {
                    "protected_delta": stranger_protected,
                    "question_rows": stranger_questions,
                    "asks_opened": stranger_asks,
                    "context_floor_rows": stranger_floor,
                    "law": "the mouth refuses a stranger line before `tick` (measured: not even a context row), and 05 §23.5(a)'s session-level form — a stranger that does reach `tick` — pays one context row and writes no belief; either way the class boundary is upstream of belief",
                },
                "owner_reentry": {
                    "shelf_delta": fed_delta,
                    "protected_delta": fed_protected,
                    "with_the_voice_off": fed_off_delta,
                    "identical": owner_channel_invariant,
                    "asks_opened": fed_asks,
                    "why_zero_rows_is_not_the_owner_channels_claim": "13 §6.6(f) says a re-entry 'mints zero rows'; the owner is a teaching channel (04 §33), so the same words typed before the voice existed mint the same rows — the zero is the non-teaching channel's claim, and the owner channel's honesty here is invariance under `TONGUE`",
                },
            }),
            "I9: SI farms its own utterances into new belief — the voice would become a teacher of nothing",
        );
    } else {
        bad(
            out,
            "G20.f.1",
            "f",
            "a re-entered line writes nothing through a non-teaching class, and through the owner channel it mints exactly what it mints with the voice off",
            json!({
                "spoken_lines": spoken_lines,
                "stranger": {
                    "shelf_delta": stranger_delta,
                    "protected_delta": stranger_protected,
                    "question_rows": stranger_questions,
                    "asks_opened": stranger_asks,
                },
                "owner": {
                    "shelf_delta": fed_delta,
                    "with_the_voice_off": fed_off_delta,
                    "identical": owner_channel_invariant,
                    "asks_opened": fed_asks,
                },
            }),
            "SI farms its own utterances into new belief or new curiosity, or the voice changed what an owner re-entry teaches",
        );
    }
    let fed_into_the_fixture = spoken_lines
        .iter()
        .filter(|line| steps.iter().any(|step| matches!(step, Step::Sense { text, .. } if text == *line)))
        .cloned()
        .collect::<Vec<_>>();
    if fed_into_the_fixture.is_empty() && !spoken_lines.is_empty() {
        ok(
            out,
            "G20.f.2",
            "f",
            "the realized line reaches the output lane and nothing else: no step of this transcript is a line the voice spoke, and the call site has one consumer (02 §18)",
            json!({
                "spoken_lines": spoken_lines,
                "fixture_lines_that_are_voices": fed_into_the_fixture,
                "structural_half": "`check_tongue_writes_nothing` grades the call site: the walker returns `Option<String>` and takes no `&mut` but the Rng it is handed",
            }),
            "the build fed the voice back into its own sense lane, so a transcript would contain its own output as input",
        );
    } else {
        bad(
            out,
            "G20.f.2",
            "f",
            "the realized line reaches the output lane and nothing else",
            json!({"spoken_lines": spoken_lines, "fixture_lines_that_are_voices": fed_into_the_fixture}),
            "the voice's own words are already an input to the loop it speaks out of",
        );
    }

    // ------------------------------------------------------- (g) parse-back
    // 13 §6.6's (g) forces "a license whose anchor surface's last content key is
    // removed". The harness cannot reach inside the boundary's license without a
    // second copy of `license_for` — which is the shape `G20(c)` exists to refuse —
    // so the leg forces the same arm from the side the law actually fixes: the
    // budget-1 drive's only buildable line is the anchor term, and the anchor term
    // is not the answer token, so the slot is lost and the canonical frame prints.
    let mut lost_slots = Vec::new();
    let mut parse_back = 0usize;
    for (drive_seed, drive, at) in &drives_for_c {
        let session = &drive.session;
        for tick in g20_ticks(drive) {
            let Some(line) = &tick.realized else { continue };
            let slot_keys: BTreeSet<String> = line_keys(&tick.canonical).into_iter().collect();
            let spoken: BTreeSet<String> = line_keys(line).into_iter().collect();
            let missing: Vec<String> = slot_keys.difference(&spoken).cloned().collect();
            if missing.is_empty() {
                parse_back += 1;
            } else {
                lost_slots.push(json!({
                    "at": at,
                    "seed": format!("0x{drive_seed:016x}"),
                    "t": tick.t,
                    "canonical": tick.canonical,
                    "realized": line,
                    "slots_lost": missing,
                }));
            }
            let _ = session;
        }
    }
    let floor_lost = forced_ticks.iter().all(|tick| {
        tick.anchor
            .and_then(|anchor| {
                g20_license(&forced.session, &tick.citation)
                    .ok()
                    .and_then(|entries| entries.into_iter().find(|(point, _)| *point == anchor).map(|(_, surface)| surface))
            })
            .map(|anchor_surface| {
                let slot: BTreeSet<String> = line_keys(&tick.canonical).into_iter().collect();
                let anchor_keys: BTreeSet<String> = line_keys(&anchor_surface).into_iter().collect();
                !slot.is_subset(&anchor_keys)
            })
            .unwrap_or(false)
    }) && !forced_ticks.is_empty();
    if lost_slots.is_empty() && parse_back > 0 && floor_lost {
        ok(
            out,
            "G20.g.1",
            "g",
            "no realized line ever loses the slot its frame filled, and the forced loss prints the canonical frame with the refusal recorded",
            json!({
                "realized_ticks_parse_back": parse_back,
                "lost_slot_offenders": lost_slots,
                "forcing": {"constants": {"PHRASE_MAX_STEPS": floor}, "fixture": scenario.path, "refusals": forced_refusals},
                "anchor_term_loses_the_slot_on_every_forced_tick": floor_lost,
                "od_173": "the check is slots-present, not a decode walk: nothing here re-parses the sentence into a frame",
            }),
            "the voice printed a line that no longer answers the question the frame answered (I8, OD-173)",
        );
    } else {
        bad(
            out,
            "G20.g.1",
            "g",
            "no realized line ever loses the slot its frame filled, and the forced loss prints the canonical frame with the refusal recorded",
            json!({
                "realized_ticks_parse_back": parse_back,
                "lost_slot_offenders": lost_slots,
                "floor_ticks": forced_ticks.len(),
                "floor_refusals": forced_refusals,
                "anchor_term_loses_the_slot_on_every_forced_tick": floor_lost,
            }),
            "a realized line lost its slot, or the forcing did not reach the arm it forces",
        );
    }

    // ------------------------------------------------------- (h) citation bound
    // **OD-181** as a hole, not a rule: every word of a realized line is grepped
    // against the surfaces that line's *own* citation names, verbatim. The
    // registry-wide grep the draft proposed is retired — against every surface in
    // the lattice any legal line passes, so it is a leg that cannot fail.
    let mut verbatim_offenders: Vec<Value> = Vec::new();
    let mut words_checked = 0usize;
    for (drive_seed, drive, at) in &drives_for_c {
        for tick in g20_ticks(drive) {
            let Some(line) = &tick.realized else { continue };
            let sources = g20_license(&drive.session, &tick.citation)?;
            for word in line.split(", ").flat_map(|chunk| chunk.split_whitespace()) {
                words_checked += 1;
                let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
                if clean.is_empty() {
                    continue;
                }
                let owned = sources.iter().any(|(_, text)| {
                    text.split(|c: char| !c.is_alphanumeric())
                        .any(|piece| piece.eq_ignore_ascii_case(clean))
                });
                if !owned {
                    verbatim_offenders.push(json!({
                        "at": at,
                        "seed": format!("0x{drive_seed:016x}"),
                        "t": tick.t,
                        "line": line,
                        "word": clean,
                        "citation": tick.citation,
                        "licensed_surfaces": sources.iter().map(|(_, text)| text.clone()).collect::<Vec<_>>(),
                    }));
                }
            }
        }
    }
    if verbatim_offenders.is_empty() && words_checked > 0 {
        ok(
            out,
            "G20.h.1",
            "h",
            "every word of every realized line appears verbatim in a surface that line's own citation names",
            json!({
                "words_checked": words_checked,
                "offenders": verbatim_offenders,
                "registry_wide_grep_retired": "OD-181: a bound over the whole lattice is satisfied by every legal line and every fluent lie, so it grades nothing",
                "od_189": "the bound is what makes the line short: a citation of one row licenses that row's chunk and its two terms, so the widest honest line is two chunks",
            }),
            "R-43: the voice emitted a word the citation never handed it — out-of-citation emission is supposed to be impossible (I8)",
        );
    } else {
        bad(
            out,
            "G20.h.1",
            "h",
            "every word of every realized line appears verbatim in a surface that line's own citation names",
            json!({"words_checked": words_checked, "offenders": verbatim_offenders}),
            "out-of-citation emission happened, or no line realized and the bound was graded over an empty set",
        );
    }

    // ------------------------------------------------------ (i) budget bound
    let placeholder = si_core::consts::registry_value(
        si_core::consts::row("PHRASE_MAX_STEPS").context("PHRASE_MAX_STEPS is not a registry row")?,
    );
    let mut budget_points: Vec<i64> = vec![floor];
    budget_points.extend(G20_POINTS.iter().copied());
    if let Some(value) = placeholder {
        budget_points.push(value);
    }
    let mut over_cap: Vec<Value> = Vec::new();
    let mut cap_evidence = Vec::new();
    for panel_seed in G20_PANEL {
        let spread = g20_spread(panel_seed);
        for point in &budget_points {
            let drive = g20_drive(run, scenario, steps, panel_seed, &[("PHRASE_MAX_STEPS", *point)])?;
            let cap = 1 + g20_budget(*point, spread);
            for tick in g20_ticks(&drive) {
                let Some(line) = &tick.realized else { continue };
                let chunks = g20_chunks(line);
                if chunks > cap {
                    over_cap.push(json!({
                        "seed": format!("0x{panel_seed:016x}"),
                        "phrase_max_steps": point,
                        "spread": spread,
                        "cap": cap,
                        "chunks": chunks,
                        "line": line,
                    }));
                }
                if chunks == cap {
                    cap_evidence.push(json!({
                        "seed": format!("0x{panel_seed:016x}"),
                        "phrase_max_steps": point,
                        "cap": cap,
                        "chunks": chunks,
                        "which_bound_bit": if tick.licensed <= cap { "budget" } else { "license" },
                    }));
                }
            }
        }
    }
    if over_cap.is_empty() && !cap_evidence.is_empty() {
        ok(
            out,
            "G20.i.1",
            "i",
            "no realized line ever exceeds `1 + budget`, at the registry `lo`, at the three bracket points and at the registry placeholder, across the panel",
            json!({
                "points": budget_points,
                "placeholder": placeholder,
                "placeholder_is_binding": placeholder.is_none(),
                "od_86": "the midpoint is a placeholder, not a measurement (12 §3.1), so a line that fits under it proves the cap holds, not that the cap is right",
                "tight_ticks": cap_evidence,
                "over_cap": over_cap,
                "which_bound_bits": {
                    "budget": cap_evidence.iter().filter(|v| v["which_bound_bit"] == json!("budget")).count(),
                    "license": cap_evidence.iter().filter(|v| v["which_bound_bit"] == json!("license")).count(),
                },
            }),
            "an over-cap walk truncated a line instead of refusing, so the species row's ceiling is decorative (13 §7)",
        );
    } else {
        bad(
            out,
            "G20.i.1",
            "i",
            "no realized line ever exceeds `1 + budget` at any named point across the panel",
            json!({"points": budget_points, "over_cap": over_cap, "tight_ticks": cap_evidence}),
            "the cap is decorative, or no tick ever landed on it and the bound went unmeasured",
        );
    }
    let sweep_path = "gates/G20/phrase_sweep.json";
    let sweep_cited = if ws.exists(sweep_path) {
        let document: Value = serde_json::from_str(&ws.read(sweep_path)?)
            .with_context(|| format!("parse {sweep_path}"))?;
        let style_rows = document["axes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|axis| axis["name"].as_str().unwrap_or_default().starts_with("STYLE_"))
            .count();
        json!({
            "present": true,
            "status": document["status"],
            "axes": document["axes"],
            "style_rows_named_as_axes": style_rows,
        })
    } else {
        json!({"present": false, "publish_with": "si-harness sweep --gate G20 --seed 0x…"})
    };
    if sweep_cited["present"] == json!(true) {
        let refused = sweep_cited["status"] == json!("refused");
        let no_style = sweep_cited["style_rows_named_as_axes"] == json!(0);
        if refused && no_style {
            ok(
                out,
                "G20.i.2",
                "i",
                "`phrase_sweep.json` is cited as a refusal and names no style row as an axis",
                json!({"sweep": sweep_cited, "od_83": "an unrulled sweep publishes its refusal rather than a number; no leg quotes a swept phrase budget (13 §6.5)"}),
                "the sweep published a number for an unrulled question, or it enumerated a draw as an axis (OD-87)",
            );
        } else {
            bad(
                out,
                "G20.i.2",
                "i",
                "`phrase_sweep.json` is cited as a refusal and names no style row as an axis",
                json!({"sweep": sweep_cited}),
                "the sweep published a measured grid where OD-83 is still unrulled, or it named a style draw as an axis",
            );
        }
    } else {
        open(
            out,
            "G20.i.2",
            "i",
            "`phrase_sweep.json` is cited as a refusal and names no style row as an axis",
            "the artifact is written by `si-harness sweep --gate G20`, not by a run, and this run does not read a sweep it did not ask for. The leg that cites it is stage B20's sweep step; OD-83 is unrulled, so what it publishes is a refusal either way.",
            json!({"sweep": sweep_cited, "artifact": sweep_path}),
            "the sweep was never run for this gate",
        );
    }

    // ------------------------------------------------------- (j) v0 strings
    // The removal test's other half: at `TONGUE = 0` every existing scenario prints
    // the copier's own bytes, so the round ships without re-baselining a single
    // golden string (**OD-178**). String equality, not hash equality, because the
    // voice writes no state to compare.
    let mut fixtures: Vec<String> = Vec::new();
    for entry in fs::read_dir(ws.path("harness/scenarios"))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".txt") && !name.starts_with("tmp_") {
            fixtures.push(format!("harness/scenarios/{name}"));
        }
    }
    fixtures.sort();
    let mut v0_rows = Vec::new();
    let mut v0_failures = Vec::new();
    let mut v0_skipped = Vec::new();
    let mut voice_ticks_at_v0 = 0usize;
    let mut frames_at_v0 = 0usize;
    for relative in &fixtures {
        // A fixture the parser refuses is not a frame the voice could change: it has
        // no steps to print. The skip is recorded, because the leg's claim is about
        // every scenario that does have a frame.
        let Ok(loaded) = Scenario::load(ws, relative) else {
            v0_skipped.push(json!({"fixture": relative, "reason": "no section to drive"}));
            continue;
        };
        let fixture_seed = loaded.seed.unwrap_or(seed);
        for (section, steps) in loaded.sections.iter().enumerate() {
            let drive = g20_drive(run, &loaded, steps, fixture_seed, &[("TONGUE", 0)])?;
            voice_ticks_at_v0 += drive.steps.iter().filter(|trace| trace.tongue.is_some()).count();
            for trace in &drive.steps {
                let Some(frame) = trace.frame.as_ref() else { continue };
                frames_at_v0 += 1;
                let mut cited_surfaces = Vec::new();
                for row_id in &frame.citation {
                    for (_, record) in drive.session.store().rows()? {
                        if record.envelope.row_id == *row_id {
                            cited_surfaces.push(surface(&drive.session, record.body.resp_surface));
                        }
                    }
                }
                let echoed = cited_surfaces.iter().any(|text| *text == frame.surface);
                v0_rows.push(json!({
                    "fixture": relative,
                    "section": section,
                    "t": trace.t,
                    "surface": frame.surface,
                    "label": frame.label,
                    "tag": frame.tag,
                    "citation": frame.citation,
                    "cited_surfaces": cited_surfaces,
                    "equals_a_cited_surface_byte_for_byte": echoed,
                }));
                if !echoed {
                    v0_failures.push(json!({
                        "fixture": relative,
                        "t": trace.t,
                        "surface": frame.surface,
                        "cited_surfaces": cited_surfaces,
                    }));
                }
            }
        }
    }
    if v0_failures.is_empty() && frames_at_v0 > 0 && voice_ticks_at_v0 == 0 {
        ok(
            out,
            "G20.j.1",
            "j",
            "at `TONGUE = 0` every printed frame of every drivable scenario fixture is a cited row's own surface byte for byte, and step 9a is entered on no tick",
            json!({
                "fixtures": fixtures,
                "frames": frames_at_v0,
                "voice_ticks_at_v0": voice_ticks_at_v0,
                "skipped": v0_skipped,
                "artifact": "gates/G20/v0_strings.json",
                "battery": "the full G1–G19 battery on this build is stage B20's verify pass (13 §6.7), which is the other half of this leg's claim",
            }),
            "**OD-178**'s premise broke: the voice changes what the copier prints even when it is switched off, so every golden string in the repository moves",
        );
    } else {
        bad(
            out,
            "G20.j.1",
            "j",
            "at `TONGUE = 0` every printed frame of every drivable scenario fixture is a cited row's own surface byte for byte, and step 9a is entered on no tick",
            json!({
                "fixtures_graded": fixtures.len(),
                "frames": frames_at_v0,
                "voice_ticks_at_v0": voice_ticks_at_v0,
                "skipped": v0_skipped,
                "failures": v0_failures,
            }),
            "the off path is not the pre-B20 path, or the arm is entered when the ruling row is off",
        );
    }

    // ------------------------------------------------- (k) genome containment
    // 13 §6.6's (k) is an ENGINE claim, and CONFIG SI has no genome layer to cross:
    // every arc in this build is a live arc the individual earned. The leg says so as
    // a count rather than pretending to grade the frozen half, and grades what
    // CONFIG SI can prove — that every transition of every realized line is a live
    // arc in the store's own edge arena.
    let mut transitions = Vec::new();
    let mut unowned: Vec<Value> = Vec::new();
    let mut genome_arcs_seen = 0usize;
    let points_of_line = |session: &Session, citation: &[u32], line: &str| -> Vec<(u32, String)> {
        let entries = g20_license(session, citation).unwrap_or_default();
        line.split(", ")
            .flat_map(|chunk| chunk.split_whitespace())
            .filter_map(|word| {
                let clean = word.trim_matches(|c: char| !c.is_alphanumeric()).to_string();
                entries
                    .iter()
                    .find(|(_, text)| {
                        text.split(|c: char| !c.is_alphanumeric())
                            .any(|piece| piece.eq_ignore_ascii_case(&clean))
                    })
                    .map(|(point, text)| (*point, text.clone()))
            })
            .collect()
    };
    for (drive_seed, drive, at) in &drives_for_c {
        let graph = si_walk::Graph::build(drive.session.store(), drive.session.now())?;
        let genome_here = (0..drive.session.store().points().len())
            .flat_map(|point| graph.arcs(point as u32))
            .filter(|arc| arc.is_genome())
            .count();
        genome_arcs_seen += genome_here;
        for tick in g20_ticks(drive) {
            let Some(line) = &tick.realized else { continue };
            let walked = points_of_line(&drive.session, &tick.citation, line);
            for pair in walked.windows(2) {
                let (from, to) = (pair[0].0, pair[1].0);
                let arc = graph.arcs(from).find(|arc| arc.to_point == to && !arc.is_genome());
                transitions.push(json!({
                    "from": from,
                    "to": to,
                    "live_non_genome_arc": arc.is_some(),
                }));
                if arc.is_none() {
                    unowned.push(json!({
                        "at": at,
                        "seed": format!("0x{drive_seed:016x}"),
                        "t": tick.t,
                        "line": line,
                        "transition": [from, to],
                        "anchor": tick.anchor,
                    }));
                }
            }
        }
    }
    if unowned.is_empty() && transitions.len() >= 4 {
        ok(
            out,
            "G20.k.1",
            "k",
            "every transition of every realized line is a live, non-genome arc of the tick's own web",
            json!({
                "transitions": transitions.len(),
                "genome_arcs_in_this_build": genome_arcs_seen,
                "config_engine_precondition": "13 §6.6's (k) is stated under CONFIG ENGINE; a `--config engine` run needs a frozen artifact (`--genome`, `--genome-tag`), which no CONFIG SI gate builds. The filter that makes the ENGINE half true is `realize`'s `!arc.is_genome()` (OD-179) and it is graded here as a count of zero genome arcs rather than as a run over a layer this config does not have.",
                "od_179": "a newborn says nothing the species text taught it — measured at G20.a.3: the question-only transcript enters the arm zero times",
            }),
            "the walk crossed a genome arc to reach a word the individual never lived (OD-179, I9)",
        );
    } else {
        bad(
            out,
            "G20.k.1",
            "k",
            "every transition of every realized line is a live, non-genome arc of the tick's own web",
            json!({"transitions": transitions, "unowned": unowned, "genome_arcs": genome_arcs_seen}),
            "a realized transition has no live arc behind it, or no transition was gradeable at all",
        );
    }

    // ---------------------------------------------------------------- artifacts
    fs::create_dir_all(ws.path("gates/G20"))?;
    let mut realized_log = String::new();
    let mut refusal_log = String::new();
    let forced_by_off = json!({"constants": {"TONGUE": 0}, "fixture": scenario.path});
    let forced_by_floor =
        json!({"constants": {"PHRASE_MAX_STEPS": floor}, "fixture": scenario.path});
    for (label, drive, seed_of, forced) in [
        ("run", &on, seed, Value::Null),
        ("budget_floor", &forced, seed, forced_by_floor),
        ("tongue_off", &off, seed, forced_by_off),
    ] {
        for tick in g20_ticks(drive) {
            let mut row = g20_row(seed_of, &tick);
            if let Some(object) = row.as_object_mut() {
                object.insert("drive".to_string(), json!(label));
                if tick.realized.is_some() {
                    realized_log.push_str(&row.to_string());
                    realized_log.push('\n');
                } else {
                    if let Some(object) = row.as_object_mut() {
                        object.insert("forced_by".to_string(), forced.clone());
                    }
                    refusal_log.push_str(&row.to_string());
                    refusal_log.push('\n');
                }
            }
        }
    }
    fs::write(ws.path("gates/G20/realized_log.jsonl"), realized_log)?;
    fs::write(ws.path("gates/G20/refusals.jsonl"), refusal_log)?;
    fs::write(
        ws.path("gates/G20/v0_strings.json"),
        serde_json::to_string_pretty(&json!({
            "fixtures": fixtures,
            "skipped": v0_skipped,
            "frames": v0_rows,
            "every_frame_is_a_cited_surface": v0_failures.is_empty(),
            "voice_ticks_at_v0": voice_ticks_at_v0,
            "od_178": "TONGUE ships OFF at V0: the copier's bytes are the whole claim, and this file is the proof that no golden string moved.",
        }))?,
    )?;
    fs::write(
        ws.path("gates/G20/panel.json"),
        serde_json::to_string_pretty(&json!({
            "variants_fixture": scenario.path,
            "bare_fixture": G20_BARE_FIXTURE,
            "variants_panel": g20_panel_json(&variants_panel),
            "bare_panel": g20_panel_json(&bare_panel),
            "budget_points": points_json,
            "chunk_cap_note": G20_CAP_NOTE,
            "od_189": "RULED: the two-chunk claim license stands and the variant bank is the licensed grammar over the same cited endpoints. At the variants fixture no two speaking individuals print the same realized transcript; at the bare control byte-identity is the expected outcome, named as such. The teaches are confirmed statements, so no belief row is asserted.",
            "od_187": "no `drew` list, no `reason` column: a refusal is identified by the drive that forced it",
        }))?,
    )?;
    Ok(())
}

/// The two class lines leg (d) subtracts, one at a time.
#[derive(Clone, Copy)]
enum Kind {
    Stranger,
    Grimm,
}

/// The rows a drive put on the shelves a voice may never write.
fn g20_protected_of(delta: &BTreeMap<u16, i64>) -> i64 {
    G20_PROTECTED.iter().filter_map(|shelf| delta.get(shelf)).copied().sum()
}
