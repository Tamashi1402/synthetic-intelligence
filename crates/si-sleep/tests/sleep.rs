//! B8 DoD item 1 (11:375–386): the six sleep laws stated as assertions on
//! [00 §2](engineering)'s real types. Every number here is a scenario constant,
//! not a registry value; no gate verdict depends on one (11 §1 D5).

use std::collections::BTreeSet;

use si_core::addr::{SHELF_CONTEXT, SHELF_FACTS, SHELF_LINKS, SHELF_QUESTIONS, SHELF_S_R_PAIRS};
use si_core::quant::{Q12_ONE, u8_to_q12};
use si_core::record::{Body, Edge, Envelope, Stamp, EDGE_BYTES, STAMP_BYTES, SUPERSEDED_LIVE};
use si_sleep::SleepRig;
use si_store::store::{EdgeDraft, Lattice};
use si_walk::{Curve, CurvePoint, Integrator, LagUnit, Lit, OscillationRule, Params};

const A: u32 = 1;
const B: u32 = 2;
const C: u32 = 3;
const D: u32 = 4;

/// The three shelves a floor constant exists for.
const FLOORED: [u16; 3] = [SHELF_S_R_PAIRS, SHELF_FACTS, SHELF_LINKS];

fn stamp() -> Stamp {
    Stamp { thought_id: 7, signal_ref: 9, magnitude: 4096, a_at: 128, signal: 1, src: 0 }
}

fn add_row(lattice: &mut Lattice, shelf: u16, source: u32, at_t: i32, edges: &[(u32, u16)]) -> u32 {
    let drafts: Vec<EdgeDraft> = edges
        .iter()
        .map(|(to, weight)| {
            let mut draft = EdgeDraft::owned(*to, *weight, 0);
            draft.stamps.push(stamp());
            draft
        })
        .collect();
    let mut body = Body::default();
    body.pattern_id = 1;
    body.key_point = source;
    lattice
        .append_row(Envelope::live(shelf, at_t, 1, at_t), body, drafts, &[], &[])
        .unwrap()
}

/// Two floored rows with an edge each, one `context` row to replay, one
/// `questions` row that has no floor at all, and one `links` edge already under
/// its floor — OD-94's shape. The `context` row is edgeless: 01 §6 makes it the
/// control group, raw surface only, no edges and no charge.
fn lattice() -> Lattice {
    let mut lattice = Lattice::new();
    add_row(&mut lattice, SHELF_S_R_PAIRS, A, 2, &[(B, 4096), (C, 2048)]);
    add_row(&mut lattice, SHELF_FACTS, B, 3, &[(D, 3000)]);
    add_row(&mut lattice, SHELF_LINKS, C, 4, &[(D, 2000), (B, 40)]);
    add_row(&mut lattice, SHELF_QUESTIONS, D, 4, &[(A, 4096)]);
    add_row(&mut lattice, SHELF_CONTEXT, A, 5, &[]);
    lattice
}

/// `(edge offset, shelf, row live?, weight)` in arena order — the map every
/// write-partition assert below reads the arena through.
fn edge_table(lattice: &Lattice) -> Vec<(u32, u16, bool, u16)> {
    let mut out = Vec::new();
    for (_, record) in lattice.rows().unwrap() {
        let live = record.envelope.is_live();
        for (index, edge) in lattice.edges_of(&record).unwrap().iter().enumerate() {
            out.push((
                record.envelope.edges_off + (index * EDGE_BYTES) as u32,
                record.envelope.shelf_x,
                live,
                edge.weight,
            ));
        }
    }
    out
}

/// The registry's floor for a shelf, read the way the pass reads it: three
/// constants, and no floor at all for `questions` and `context`.
fn floor_for(cfg: &si_sleep::SleepConfig, shelf: u16) -> Option<u16> {
    match shelf {
        SHELF_S_R_PAIRS => Some(cfg.floor_s_r_q12),
        SHELF_FACTS => Some(cfg.floor_facts_q12),
        SHELF_LINKS => Some(cfg.floor_links_q12),
        _ => None,
    }
}

fn params() -> Params {
    Params {
        lambda_a: 128,
        gain: [0; 6],
        beta: 4096,
        gamma: 4096,
        ask_cap_per_input: 1,
        priority: [4096, 4096, 4096],
        gap_salience_floor: 0,
        dedupe_window: 0,
        bias_cap_share: 4096,
        b_max: 4096,
        k_tanh: 1,
        median_min_n: 1,
        median_shape: 0,
        lag_rounds: 1,
        lag_unit: LagUnit::PerRound,
    }
}

fn rig() -> SleepRig {
    SleepRig {
        params: params(),
        curve: Curve::fixture(CurvePoint { reach: 4, decay_q12: 4096, settle_shift_q12: 0 }),
        k_max: 4,
        tau_settle: 2048,
        decay_base: 128,
        hub_exponent_q12: 4096,
        frontier_cap: 16,
        oscillation: OscillationRule::MaxKOnly,
        ghost_max: 0,
        shape_max: 0,
    }
}

fn config() -> si_sleep::SleepConfig {
    si_sleep::SleepConfig {
        cadence_ticks: 4,
        max_walks: 2,
        dim_base: 64,
        floor_s_r_q12: 128,
        floor_facts_q12: 64,
        floor_links_q12: 64,
        stable_n: 3,
    }
}

fn request<'a>(
    store: &'a mut Lattice,
    integrator: &'a mut Integrator,
    sense_pending: &'a dyn Fn(u8) -> bool,
) -> si_sleep::SleepRequest<'a> {
    let context_row = store
        .rows()
        .unwrap()
        .into_iter()
        .find(|(_, record)| record.envelope.shelf_x == SHELF_CONTEXT)
        .map(|(_, record)| record.envelope.row_id)
        .unwrap();
    si_sleep::SleepRequest {
        store,
        integrator,
        rig: rig(),
        config: config(),
        now: 10,
        last_slept_at_t: 0,
        thought_id_start: 100,
        replay: vec![si_sleep::ReplayEvent {
            row_id: context_row,
            written_at_t: 5,
            lights: vec![Lit { point: A, activation: 4096 }],
        }],
        sense_pending,
    }
}

fn night(store: &mut Lattice, sense_pending: &dyn Fn(u8) -> bool) -> si_sleep::SleepReport {
    let mut integrator = Integrator::restore(0);
    si_sleep::run(&mut request(store, &mut integrator, sense_pending)).unwrap()
}

// --------------------------------------------------------- no row is reachable

/// No `commit`, no `append_row`, no `age` is reachable from a sleep: the row
/// arena, the stamp arena and the row census do not move by one byte, and the
/// only arena that does is the edge arena. `check_sleep_writes_nothing` is the
/// static half; this is the dynamic half on a real night.
#[test]
fn a_whole_night_writes_no_row_and_no_stamp() {
    let mut store = lattice();
    let rows_before = store.rows_arena().bytes().to_vec();
    let stamps_before = store.stamps_arena().bytes().to_vec();
    let census_before: Vec<(u16, u32, bool)> = store
        .rows()
        .unwrap()
        .iter()
        .map(|(_, record)| (record.envelope.shelf_x, record.envelope.row_id, record.envelope.is_live()))
        .collect();

    let report = night(&mut store, &|_| false);

    assert_eq!(store.rows_arena().bytes(), &rows_before[..], "a sleep wrote a row byte");
    assert_eq!(store.stamps_arena().bytes(), &stamps_before[..], "a sleep wrote a stamp");
    let census_after: Vec<(u16, u32, bool)> = store
        .rows()
        .unwrap()
        .iter()
        .map(|(_, record)| (record.envelope.shelf_x, record.envelope.row_id, record.envelope.is_live()))
        .collect();
    assert_eq!(census_after, census_before, "a sleep created or retired a row (G13(i))");
    assert_eq!(report.rows_touched, 0);
    assert_eq!(report.p1.as_ref().unwrap().rows_written, 0);
    assert_eq!(report.p2.as_ref().unwrap().rows_written, 0);
    assert_eq!(report.p4.as_ref().unwrap().stamps_written, 0, "OPEN-24 is unruled, so this is the spec's zero");
    assert_eq!(report.passes_completed, vec![1, 2, 3, 4]);
    assert_eq!(report.aborted_reason, None);
}

/// The edge arena moves only inside bytes 6..8 of a record — the `weight` column.
#[test]
fn the_night_moves_only_the_weight_column() {
    let mut store = lattice();
    let before = store.edges_arena().bytes().to_vec();
    night(&mut store, &|_| false);
    let after = store.edges_arena().bytes().to_vec();
    assert_eq!(before.len(), after.len(), "the sleep grew the edge arena");
    let mut moved = 0usize;
    for (offset, (previous, later)) in before.iter().zip(&after).enumerate() {
        if previous != later {
            assert!(
                (6..8).contains(&(offset % EDGE_BYTES)),
                "byte {offset} moved outside the `weight` column of its record"
            );
            moved += 1;
        }
    }
    assert!(moved > 0, "the night wrote nothing, so this proves nothing");
}

// ------------------------------------------------- P3 is the only weight writer

/// P3's own before/after table never brightens and never zeroes a live edge, and
/// it is the whole write set rather than a sample of it.
#[test]
fn p3_never_brightens_and_never_zeros_a_live_edge() {
    let mut store = lattice();
    let all_before = edge_table(&store);
    let report = night(&mut store, &|_| false);
    let all_after = edge_table(&store);

    // The sweep's scope: live OWNED edges on the three floored shelves, in
    // `(x, y)` order — for this fixture the same order as the arena.
    let scanned: Vec<(u32, u16, bool, u16)> = all_before
        .iter()
        .filter(|(_, shelf, live, _)| FLOORED.contains(shelf) && *live)
        .copied()
        .collect();
    assert_eq!(report.ledger.len(), scanned.len(), "the ledger is not the scan set");
    assert_eq!(report.p3.as_ref().unwrap().edges_scanned as usize, scanned.len());

    for (entry, (offset, shelf, _, previous)) in report.ledger.iter().zip(&scanned) {
        assert_eq!(entry.edge_off, *offset, "the ledger is not in scan order");
        assert_eq!(entry.shelf, *shelf);
        assert_eq!(entry.before, *previous);
        assert!(entry.after <= entry.before, "P3 brightened edge {offset}");
        assert!(entry.after > 0, "P3 zeroed the live edge {offset} (10 §6 D.3)");
    }

    let written: BTreeSet<u32> =
        report.ledger.iter().filter(|e| e.before != e.after).map(|e| e.edge_off).collect();
    for (offset, _, _, previous) in &all_before {
        let later = all_after.iter().find(|(at, ..)| at == offset).unwrap().3;
        if later != *previous {
            assert!(written.contains(offset), "edge {offset} moved with no ledger row");
        }
    }
    assert_eq!(report.bytes_changed, u32::try_from(written.len()).unwrap() * 2, "06 §5.1: two bytes per dimmed edge");
}

/// The store itself refuses the two outcomes, and the one column it will write:
/// `dim_edge_weight` is the whole mutation surface and it is 2 bytes wide.
#[test]
fn the_store_refuses_a_brightening_a_zeroing_and_a_wider_write() {
    let mut store = lattice();
    let table = edge_table(&store);
    let victim = store
        .edges_arena()
        .bytes()
        .to_vec()
        .chunks(EDGE_BYTES)
        .enumerate()
        .filter(|(_, bytes)| bytes.len() == EDGE_BYTES)
        .map(|(index, bytes)| (index, Edge::decode(bytes).unwrap().weight))
        .find(|(_, weight)| *weight > 1)
        .map(|(index, _)| table[index].0)
        .unwrap();
    let edge =
        Edge::decode(&store.edges_arena().bytes()[victim as usize..victim as usize + EDGE_BYTES]).unwrap();

    assert!(store.dim_edge_weight(victim, edge.weight + 1).is_err(), "the store accepted a brightening");
    assert!(store.dim_edge_weight(victim, 0).is_err(), "the store accepted a zeroing of a live edge");
    assert!(!store.dim_edge_weight(victim, edge.weight).unwrap(), "an equal weight wrote bytes");
    assert!(store.dim_edge_weight(victim, edge.weight - 1).unwrap(), "a lawful dimming wrote nothing");

    let after =
        Edge::decode(&store.edges_arena().bytes()[victim as usize..victim as usize + EDGE_BYTES]).unwrap();
    assert_eq!(after.weight, edge.weight - 1);
    assert_eq!(after.to_point, edge.to_point);
    assert_eq!(after.kind, edge.kind);
    assert_eq!(after.flags, edge.flags);
    assert_eq!(after.bake, edge.bake);
    assert_eq!(after.charge_off, edge.charge_off);
}

/// Only P3 writes `weight`: P1, P2 and P4 leave the edge arena exactly as they
/// found it.
#[test]
fn the_other_three_passes_write_no_weight() {
    let mut store = lattice();
    let edges_before = store.edges_arena().bytes().to_vec();
    let mut integrator = Integrator::restore(0);
    let mut trail = Vec::new();
    {
        let mut request = request(&mut store, &mut integrator, &|_| false);
        si_sleep::p1_replay(&mut request, 10, 2, &mut trail).unwrap();
        si_sleep::p2_integration_scan(&mut request).unwrap();
    }
    assert_eq!(store.edges_arena().bytes(), &edges_before[..], "P1 or P2 wrote an edge byte");
    let mut request = request(&mut store, &mut integrator, &|_| false);
    si_sleep::p4_calm_reprocessing(&mut request, 10, 2, &mut trail).unwrap();
    assert_eq!(store.edges_arena().bytes(), &edges_before[..], "P4 wrote an edge byte");
    assert!(!trail.is_empty(), "P1 and P4 were supposed to walk");
}

// ----------------------------------------------------- the floors are constants

/// The floors are the three registry constants and nothing else: a shelf with no
/// floor — `questions`, `context` — is never touched, and no record has a floor
/// column to read.
#[test]
fn floors_are_constants_not_columns() {
    let cfg = config();
    let floors = si_sleep::floors(&cfg);
    let names: Vec<&str> = floors.keys().copied().collect();
    assert_eq!(names, vec!["FLOOR_FACTS", "FLOOR_LINKS", "FLOOR_S_R"], "a fourth floor appeared, or one of the three is gone");
    assert_eq!(floors["FLOOR_S_R"], cfg.floor_s_r_q12);
    assert_eq!(floors["FLOOR_FACTS"], cfg.floor_facts_q12);
    assert_eq!(floors["FLOOR_LINKS"], cfg.floor_links_q12);
    assert_eq!(si_sleep::floors(&si_sleep::SleepConfig::from_registry()).len(), 3);
    for shelf in [SHELF_QUESTIONS, SHELF_CONTEXT] {
        assert!(floor_for(&cfg, shelf).is_none(), "shelf {shelf} grew a floor constant");
    }

    // A record has 16 bytes and they are all accounted for by 01 §5.1's columns,
    // so there is no room for a per-edge floor.
    let encoded = Edge { to_point: B, kind: 0, flags: 0, weight: 4096, bake: 0, charge_off: 0 }.encode();
    assert_eq!(encoded.len(), EDGE_BYTES);
    assert_eq!(&encoded[6..8], &4096u16.to_le_bytes(), "weight is the column the floor anchors to");
    assert_eq!(&encoded[8..10], &[0, 0], "bake is not a floor");
    assert_eq!(&encoded[10..12], &[0, 0], "the pad is not a floor");

    let mut store = lattice();
    let before = edge_table(&store);
    let before_bytes = store.edges_arena().bytes().to_vec();
    let report = night(&mut store, &|_| false);
    let after_bytes = store.edges_arena().bytes().to_vec();

    for entry in &report.ledger {
        assert!(FLOORED.contains(&entry.shelf), "P3 dimmed shelf {}, which has no floor constant", entry.shelf);
        assert_eq!(
            Some(entry.floor_q12),
            floor_for(&cfg, entry.shelf),
            "the ledger's floor is not the swept constant for shelf {}",
            entry.shelf
        );
    }
    // The un-floored shelves keep every byte of their edge records.
    for (offset, shelf, _, _) in &before {
        if FLOORED.contains(shelf) {
            continue;
        }
        let window = (*offset as usize)..(*offset as usize + EDGE_BYTES);
        assert_eq!(
            &before_bytes[window.clone()],
            &after_bytes[window],
            "shelf {shelf} was written by a pass whose only floor constants name shelves 4, 5 and 6"
        );
    }
}

/// OD-94's shape, asserted rather than assumed: an edge already under its floor
/// stays exactly where it is and is counted; every other edge ends at or above
/// its floor, and `d` never leaves `[DIM_BASE, 1]`.
#[test]
fn an_edge_below_the_floor_stays_and_is_counted() {
    let cfg = config();
    let mut store = lattice();
    let report = night(&mut store, &|_| false);
    let below: Vec<_> = report.ledger.iter().filter(|entry| entry.below_floor).collect();
    assert!(!below.is_empty(), "the fixture has an edge under its floor and P3 did not see it");
    for entry in &below {
        assert_eq!(entry.before, entry.after, "P3 lifted an edge below the floor, which is a brightening");
        assert!(entry.before < entry.floor_q12);
    }
    assert_eq!(report.p3.as_ref().unwrap().edges_below_floor, below.len() as u32);
    for entry in &report.ledger {
        if !entry.below_floor {
            assert!(entry.after >= entry.floor_q12, "P3 left an edge under its shelf's floor");
        }
        assert!(entry.d_q12 >= u8_to_q12(cfg.dim_base), "d fell below DIM_BASE: one sleep would blackout a row");
        assert!(entry.d_q12 <= Q12_ONE, "d exceeded 1.0 and would brighten");
    }
    let p3 = report.p3.unwrap();
    assert!(p3.floor_hits <= p3.edges_scanned);
}

// -------------------------------------------------------- the calm view is read

/// P4's counts have no consumer. The store-visible half: a calm pass writes zero
/// bytes, so no weight downstream of it can depend on a count. The static half is
/// `check_calm_view_is_terminal`.
#[test]
fn the_calm_pass_writes_nothing_and_its_counts_agree_with_its_line() {
    let mut store = lattice();
    let rows_before = store.rows_arena().bytes().to_vec();
    let edges_before = store.edges_arena().bytes().to_vec();
    let stamps_before = store.stamps_arena().bytes().to_vec();
    let mut integrator = Integrator::restore(0);
    let mut trail = Vec::new();
    let (report, walked) = {
        let mut request = request(&mut store, &mut integrator, &|_| false);
        si_sleep::p4_calm_reprocessing(&mut request, 10, 2, &mut trail).unwrap()
    };
    assert_eq!(walked, report.walks);
    assert_eq!(store.rows_arena().bytes(), &rows_before[..]);
    assert_eq!(store.edges_arena().bytes(), &edges_before[..], "the calm pass wrote a weight");
    assert_eq!(store.stamps_arena().bytes(), &stamps_before[..]);
    // The line and the view are the same readout, twice: neither is a second
    // number anyone could act on differently.
    assert_eq!(report.calm.points_swept, report.points_swept);
    assert_eq!(report.calm.walks, report.walks);
    assert_eq!(report.calm.rows_relit, report.rows_relit);
    assert_eq!(report.calm.rows_read_calm_not_settleable, report.rows_read_calm_not_settleable);
    assert_eq!(report.calm.stamps_written, report.stamps_written);
    assert_eq!(report.stamps_written, 0);
    assert!(report.calm.points_swept > 0, "the sweep is a read and happened to no points");

    // OD-85: the sweep is full even at remainder 0, because a sweep is a read and
    // only a walk is a spend.
    let mut second = lattice();
    let mut second_integrator = Integrator::restore(0);
    let (zero_remainder, _) = {
        let mut request = request(&mut second, &mut second_integrator, &|_| false);
        si_sleep::p4_calm_reprocessing(&mut request, 10, 0, &mut Vec::new()).unwrap()
    };
    assert_eq!(zero_remainder.walks, 0, "P4 walked past a remainder of 0");
    assert_eq!(zero_remainder.points_swept, report.points_swept, "the sweep was skipped at remainder 0");
}

// ------------------------------------------------------------- a walk is a tick

/// Each sleep walk advances `t` by exactly one and is an ordinary thought: its
/// own `thought_id`, its own round list, and the whole night's clock cost is its
/// walk count (OD-78).
#[test]
fn each_walk_is_a_tick_and_an_ordinary_thought() {
    let mut store = lattice();
    let report = night(&mut store, &|_| false);
    let p1 = report.p1.as_ref().unwrap();
    let p4 = report.p4.as_ref().unwrap();
    assert_eq!(report.walks, p1.walks + p4.walks);
    assert_eq!(
        report.final_now,
        report.t + i32::try_from(report.walks).unwrap(),
        "the night's clock is its walks (OD-78)"
    );
    assert!(!report.trail.is_empty());
    for (index, walk) in report.trail.iter().enumerate() {
        assert_eq!(walk.t, report.t + i32::try_from(index).unwrap(), "walk {index} did not take the next tick");
        assert_eq!(walk.thought_id, 100 + u32::try_from(index).unwrap());
        assert!(!walk.thought.rounds.is_empty(), "walk {index} produced no rounds");
        assert!(walk.seed_point != 0, "walk {index} has no round-0 lighting");
        assert!([1, 4].contains(&walk.pass), "pass {} produced a thought but is not a walker", walk.pass);
    }
    // OD-85: one pool, spent in pass order. P1 is served first and P4 gets what
    // the pool left, so P4's walks can never exceed the remainder P1 was given.
    let budget = config().max_walks;
    assert!(p1.walks <= budget, "P1 walked past the whole pool");
    assert!(p4.walks <= budget.saturating_sub(p1.walks), "P4 spent past the remainder P1 left (OD-85)");
    assert!(p1.walks + p4.walks <= budget, "the night overspent the walk pool");
    assert!(p1.walks > 0, "the pool was spent on P4 while P1 was still owed");
    // 10 §8: the ledger's own clock stays inside the night it reports.
    assert_eq!(report.ledger.iter().filter(|e| e.sleep_t > report.final_now).count(), 0);
}

/// The replay set is the `context` rows with `t > last_slept_at_t`, truncated to
/// the budget newest-first, and both counts are on the line so the truncation is
/// visible rather than inferable (**G13(l).4**).
#[test]
fn the_replay_set_is_truncated_newest_first_and_reports_both_counts() {
    let mut store = Lattice::new();
    let first = add_row(&mut store, SHELF_CONTEXT, A, 1, &[]);
    let second = add_row(&mut store, SHELF_CONTEXT, B, 2, &[]);
    let third = add_row(&mut store, SHELF_CONTEXT, C, 3, &[]);
    let _ = first;
    let _ = second;
    let mut integrator = Integrator::restore(0);
    let mut request = request(&mut store, &mut integrator, &|_| false);
    request.last_slept_at_t = 0;
    request.config.max_walks = 1;
    // Only the newest row has a recorded lighting, so a newest-first truncation
    // walks one event and an oldest-first one would walk none.
    request.replay = vec![si_sleep::ReplayEvent {
        row_id: third,
        written_at_t: 3,
        lights: vec![Lit { point: C, activation: 4096 }],
    }];
    let mut trail = Vec::new();
    let (report, walked) = si_sleep::p1_replay(&mut request, 10, 1, &mut trail).unwrap();
    assert_eq!(report.events_available, 3, "the count must be taken before the truncation");
    assert_eq!(report.walks, 1);
    assert_eq!(walked, 1);
    assert_eq!(trail[0].seed_point, C, "the walk was not the newest row's");
    assert_eq!(report.rows_written, 0);
    assert_eq!(report.pass, 1);
}

// ------------------------------------------------------------- the abort is ok

/// An abort lands at a pass boundary, returns `Ok`, completes the passes before
/// it and records the partial set in the mask. Boundary 5 is the read after P4's
/// report line: the night is complete there, not partial (10 §8's pass-set law).
#[test]
fn an_abort_at_each_boundary_is_ok_and_leaves_the_mask() {
    for boundary in 1..=5u8 {
        let mut store = lattice();
        let report = night(&mut store, &|pass| pass == boundary);
        let expected: Vec<u8> = if boundary == 5 { vec![1, 2, 3, 4] } else { (1..boundary).collect() };
        assert_eq!(report.passes_completed, expected, "boundary {boundary} ran the wrong pass set");
        let expected_mask = expected.iter().fold(0u8, |mask, pass| mask | (1 << (pass - 1)));
        assert_eq!(report.pass_mask(), expected_mask, "boundary {boundary} wrote the wrong mask");
        if boundary == 5 {
            assert_eq!(report.aborted_reason, None, "a complete night reported itself as partial");
        } else {
            assert_eq!(
                report.aborted_reason.as_deref(),
                Some(format!("inbox_pending_before_pass_{boundary}").as_str())
            );
        }
        // A rescale is never half-applied: there is no boundary inside P3, so
        // interrupting before it leaves the whole table unwritten and
        // interrupting after it leaves the whole table written.
        if boundary <= 3 {
            assert!(report.p3.is_none(), "boundary {boundary} ran P3 anyway");
            assert!(report.ledger.is_empty(), "boundary {boundary} wrote weights with no P3 report");
        } else {
            assert!(report.p3.is_some(), "boundary {boundary} skipped P3");
            assert_eq!(report.ledger.len(), report.p3.as_ref().unwrap().edges_scanned as usize);
        }
        assert_eq!(report.rows_touched, 0);
        // `sleeps_interrupted` is the caller's, and it advances on every abort but
        // boundary 5's, which is a completed sleep.
        assert_eq!(report.aborted_reason.is_some(), boundary < 5);
    }
    // `due` is a counter and a checkpoint, never a wall clock (02 §13).
    let cfg = config();
    assert_eq!(si_sleep::due(&cfg, 0, false, false), si_sleep::Due::No);
    assert_eq!(si_sleep::due(&cfg, 4, false, false), si_sleep::Due::Yes);
    assert_eq!(si_sleep::due(&cfg, 8, false, false), si_sleep::Due::Yes, "it fires on every multiple, once");
    assert_eq!(si_sleep::due(&cfg, 3, false, false), si_sleep::Due::No);
    assert_eq!(si_sleep::due(&cfg, 4, false, true), si_sleep::Due::No, "a pending frame outranks the cadence");
    assert_eq!(si_sleep::due(&cfg, 3, true, false), si_sleep::Due::Yes, "the freeze trigger");
    assert_eq!(si_sleep::due(&cfg, 3, true, true), si_sleep::Due::No);
    // 03 §5.1 gives this row the range "≥ 2", so the build has no defined meaning
    // for a cadence of 0 or 1 and no assert here invents one. The two triggers the
    // spec does name are the counter and the freeze, both tested above.
    assert!(cfg.cadence_ticks >= 2, "the scenario's cadence is inside the swept box");
}

/// The biconditional 10 §6 D.3 states: `weight == 0` only for a superseded row.
/// Dimming cannot mint a fake supersession, and a superseded row is not scanned.
#[test]
fn dimming_does_not_mint_supersessions() {
    let mut store = lattice();
    let facts_row = store
        .rows()
        .unwrap()
        .into_iter()
        .find(|(_, record)| record.envelope.shelf_x == SHELF_FACTS)
        .map(|(_, record)| record.envelope.row_id)
        .unwrap();
    store.age(SHELF_FACTS, facts_row, 9, 1).unwrap();
    let report = night(&mut store, &|_| false);

    for (_, record) in store.rows().unwrap() {
        let live = record.envelope.superseded_at == SUPERSEDED_LIVE;
        for edge in store.edges_of(&record).unwrap() {
            if !edge.is_owned() {
                continue;
            }
            assert_eq!(
                edge.weight == 0,
                !live,
                "row {} on shelf {} breaks 10 §6 D.3's biconditional",
                record.envelope.row_id,
                record.envelope.shelf_x
            );
        }
    }
    for entry in &report.ledger {
        assert_ne!(entry.after, 0, "P3 zeroed edge {}", entry.edge_off);
        assert_ne!(entry.shelf, SHELF_FACTS, "P3 scanned the superseded row");
    }
    assert_eq!(report.bytes_changed, report.p3.unwrap().edges_changed * 2);
}

/// P2 counts the composite tally and writes nothing, because `INFERENCE_PROMOTION`
/// is off and OD-58 is unruled.
#[test]
fn p2_counts_candidates_and_writes_nothing() {
    let mut store = lattice();
    let rows_before = store.rows_arena().bytes().to_vec();
    let edges_before = store.edges_arena().bytes().to_vec();
    let mut integrator = Integrator::restore(0);
    let report = {
        let mut request = request(&mut store, &mut integrator, &|_| false);
        si_sleep::p2_integration_scan(&mut request).unwrap()
    };
    assert_eq!(report.pass, 2);
    assert_eq!(report.rows_written, 0);
    assert_eq!(store.rows_arena().bytes(), &rows_before[..]);
    assert_eq!(store.edges_arena().bytes(), &edges_before[..]);
    // Nothing has been rehearsed, so the composite tally is empty and stable is
    // empty with it — the pass reports what it counted, not what a sweep wishes.
    assert!(report.stable_ge_N <= report.candidates);
}

/// 06 §3.4's byte claim, checked against the block the sleep actually writes:
/// 36 bytes, independent of rows, edges and sleeps, and `si-sleep` moves no byte
/// of it itself — the caller does, once per sleep.
#[test]
fn the_sleep_record_is_the_size_the_budget_prices() {
    assert_eq!(si_core::hash::CORE_BLOCK_BYTES, 36, "06 §3.4's core block");
    assert_eq!(STAMP_BYTES, 16);
    assert_eq!(EDGE_BYTES, 16);
    // A birth block reads as "never slept", which is what makes the first night's
    // replay set the whole of L3 rather than an empty window.
    let born = si_core::hash::CoreBlock::at_birth(7);
    assert_eq!(born.sleep_pass_mask, 0);
    assert_eq!(born.sleeps_completed, 0);
    assert_eq!(born.last_slept_at_t, 0);
    // The block is 36 bytes whoever slept and however many rows the store holds:
    // the budget's claim is that the sleep's own state does not grow.
    let slept = si_core::hash::CoreBlock {
        sleeps_completed: 9,
        sleeps_interrupted: 2,
        last_slept_at_t: 41,
        sleep_pass_mask: 15,
        ..born
    };
    assert_eq!(born.encode().len(), si_core::hash::CORE_BLOCK_BYTES);
    assert_eq!(slept.encode().len(), si_core::hash::CORE_BLOCK_BYTES);
    assert_eq!(si_core::hash::CoreBlock::decode(&slept.encode()).unwrap(), slept);
}
