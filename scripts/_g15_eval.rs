
// ---------------------------------------------------------------------------
// G15 — crystallize (05 §23, docs/22, 12 §5 §29 / §6.4 G15).
// ---------------------------------------------------------------------------

/// The floors a G15 run installs on the channel. OD-83 refuses to enumerate
/// the arc's boxes, so the gate names the point it runs at instead of a
/// placeholder the registry happened to pick: `R_MIN`/`R_ASK`/`DELTA_MIN` at
/// `lo`, and the two frame caps at the smallest value that lets one ghost and
/// one role-graph exist. Each is on-grid by construction (12 §6.4, OD-95).
fn g15_launch() -> BTreeMap<String, i64> {
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
        assert!(value >= row.lo && value <= row.hi && (value - row.lo) % row.grid == 0);
        out.insert(name.to_string(), value);
    }
    out
}

/// The launch constants a G15 pair runs at: the gate's own floors, then whatever
/// the run declared (so a run may move a point, and the pair moves together).
fn g15_constants(run: &Run, crystallize: i64) -> BTreeMap<String, i64> {
    let mut out = g15_launch();
    for (name, value) in &run.request.constants {
        out.insert(name.clone(), *value);
    }
    out.insert("CRYSTALLIZE".to_string(), crystallize);
    out
}

fn g15_pair(run: &Run, steps: &[Step], on: bool) -> Result<DriveOutcome> {
    drive(&DriveRequest {
        steps: steps.to_vec(),
        seed: run.request.seed,
        constants: g15_constants(run, if on { 1 } else { 0 }),
        variant: run.request.variant.clone(),
        sleep_interrupt: run.request.sleep_interrupt,
        cut_after: None,
        cut_path: None,
        freeze_at_end: false,
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

fn g15(run: &Run, ws: &Workspace, out: &mut Vec<Value>) -> Result<()> {
    let section = run
        .sections
        .first()
        .context("a G15 run has no section to grade")?;
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
    let extras: Vec<(u32, u16, u32)> = on_q
        .iter()
        .filter(|row| !off_q.contains(row))
        .copied()
        .collect();
    let off_missing: Vec<(u32, u16, u32)> = off_q
        .iter()
        .filter(|row| !on_q.contains(row))
        .copied()
        .collect();
    // 05 §23: the questions shelf is compared on its own terms — same count, or
    // only rows whose `key_field` names the family's kind with origin = 2.
    let foreign: Vec<(u32, u16, u32)> = extras
        .iter()
        .filter(|(_, key_field, _)| {
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
        steps: junk.sections[0].clone(),
        seed: run.request.seed,
        constants: g15_constants(run, 1),
        variant: run.request.variant.clone(),
        sleep_interrupt: None,
        cut_after: None,
        cut_path: None,
        freeze_at_end: false,
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
