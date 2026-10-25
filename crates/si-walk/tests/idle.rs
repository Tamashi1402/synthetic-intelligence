//! 04 §19 S: the gate, the fixed precedence, the one seeded sampler, and a
//! cascade that is §5 unchanged. The lattice is read, never written.

mod common;

use std::collections::{BTreeMap, BTreeSet};

use common::{add_row, config, flat_curve, params, A, B, C};
use si_affect::{Integrator, Params};
use si_core::rng::Rng;
use si_store::store::Lattice;
use si_walk::{
    cascade, idle_tick, ignite, Agenda, Graph, IdleAction, IdleInputs, IdleWeights, QuestionRef,
    LIGHT_QUANTUM,
};

fn baked_lattice() -> Lattice {
    let mut lattice = Lattice::new();
    add_row(&mut lattice, A, &[(B, 4096, 4096, 128)]);
    add_row(&mut lattice, C, &[(B, 4096, 2048, 128)]);
    lattice
}

fn bake_only_weights() -> IdleWeights {
    IdleWeights {
        recency_q12: 0,
        bake_q12: 4096,
        decay_deficit_q12: 0,
        contradiction_q12: 0,
        silt_q12: 0,
    }
}

/// The quiet-time statistics a session would hand the tick; empty unless a leg
/// needs them.
struct Env {
    last_lit: BTreeMap<u32, i32>,
    decay_deficit: BTreeMap<u32, u16>,
    contradiction_warmth: BTreeSet<u32>,
    silt: BTreeMap<u32, u16>,
}

impl Env {
    fn new() -> Self {
        Self {
            last_lit: BTreeMap::new(),
            decay_deficit: BTreeMap::new(),
            contradiction_warmth: BTreeSet::new(),
            silt: BTreeMap::new(),
        }
    }

    fn inputs<'a>(&'a self, agenda: &'a Agenda, quiet: bool, idles: u32, per: u16) -> IdleInputs<'a> {
        IdleInputs {
            senses_quiet: quiet,
            agenda,
            idles_this_window: idles,
            per_window: per,
            last_lit: &self.last_lit,
            decay_deficit: &self.decay_deficit,
            contradiction_warmth: &self.contradiction_warmth,
            silt: &self.silt,
        }
    }

    fn draw(&self, graph: &Graph, weights: &IdleWeights, params: &Params, rng: &mut Rng) -> Option<u32> {
        ignite(
            graph,
            10,
            &self.last_lit,
            &self.decay_deficit,
            &self.contradiction_warmth,
            &self.silt,
            weights,
            params,
            rng,
        )
    }
}

fn footprint(lattice: &Lattice) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>) {
    (
        lattice.rows_arena().bytes().to_vec(),
        lattice.edges_arena().bytes().to_vec(),
        lattice.stamps_arena().bytes().to_vec(),
        lattice.handles_arena().bytes().to_vec(),
        lattice.interner().canonical_bytes().to_vec(),
    )
}

#[test]
fn a_noisy_tick_is_never_idle_not_even_to_revisit() {
    let lattice = baked_lattice();
    let graph = Graph::build(&lattice, 0).unwrap();
    let env = Env::new();
    let agenda = Agenda {
        open_questions: vec![QuestionRef { question_id: 4, asked_at: 1 }],
        reparse_due: None,
    };
    let params = params();
    let mut rng = Rng::seeded(7);
    let inputs = env.inputs(&agenda, false, 0, 4);
    assert_eq!(
        idle_tick(&graph, 10, &inputs, &bake_only_weights(), &params, &mut rng),
        IdleAction::Nothing
    );
}

#[test]
fn the_oldest_open_question_is_revisited_before_any_ignition() {
    let lattice = baked_lattice();
    let graph = Graph::build(&lattice, 0).unwrap();
    let env = Env::new();
    let agenda = Agenda {
        open_questions: vec![
            QuestionRef { question_id: 9, asked_at: 5 },
            QuestionRef { question_id: 3, asked_at: 2 },
        ],
        reparse_due: Some(1),
    };
    let params = params();
    let mut rng = Rng::seeded(7);
    let inputs = env.inputs(&agenda, true, 0, 4);
    assert_eq!(
        idle_tick(&graph, 10, &inputs, &bake_only_weights(), &params, &mut rng),
        IdleAction::Revisit { question_id: 3 }
    );
}

#[test]
fn a_due_reparse_outranks_ignition_but_not_a_question() {
    let lattice = baked_lattice();
    let graph = Graph::build(&lattice, 0).unwrap();
    let env = Env::new();
    let agenda = Agenda { open_questions: Vec::new(), reparse_due: Some(12) };
    let params = params();
    let mut rng = Rng::seeded(7);
    let inputs = env.inputs(&agenda, true, 0, 4);
    assert_eq!(
        idle_tick(&graph, 10, &inputs, &bake_only_weights(), &params, &mut rng),
        IdleAction::Reparse { row_id: 12 }
    );
}

#[test]
fn the_window_rate_cap_gates_ignition() {
    let lattice = baked_lattice();
    let graph = Graph::build(&lattice, 0).unwrap();
    let env = Env::new();
    let agenda = Agenda::default();
    let params = params();
    let mut rng = Rng::seeded(7);
    let blocked = env.inputs(&agenda, true, 4, 4);
    assert_eq!(
        idle_tick(&graph, 10, &blocked, &bake_only_weights(), &params, &mut rng),
        IdleAction::Nothing
    );
    let allowed = env.inputs(&agenda, true, 3, 4);
    assert!(matches!(
        idle_tick(&graph, 10, &allowed, &bake_only_weights(), &params, &mut rng),
        IdleAction::Ignition { .. }
    ));
}

#[test]
fn no_candidate_with_any_weight_is_nothing_not_a_zero_activation() {
    let mut lattice = Lattice::new();
    add_row(&mut lattice, A, &[(B, 4096, 0, 128)]);
    let graph = Graph::build(&lattice, 0).unwrap();
    let env = Env::new();
    let agenda = Agenda::default();
    let params = params();
    let mut rng = Rng::seeded(7);
    assert_eq!(env.draw(&graph, &bake_only_weights(), &params, &mut rng), None);
    let inputs = env.inputs(&agenda, true, 0, 4);
    assert_eq!(
        idle_tick(&graph, 10, &inputs, &bake_only_weights(), &params, &mut rng),
        IdleAction::Nothing
    );
}

#[test]
fn the_sampler_reads_the_stored_bake_and_is_a_function_of_the_seed() {
    let lattice = baked_lattice();
    let graph = Graph::build(&lattice, 0).unwrap();
    let bake = graph.bake_touch_max();
    assert_eq!(bake.get(&A).copied(), Some(4096));
    assert_eq!(bake.get(&B).copied(), Some(4096));
    assert_eq!(bake.get(&C).copied(), Some(2048));

    let env = Env::new();
    let weights = bake_only_weights();
    let params = params();
    let pick = |seed: u64| {
        let mut rng = Rng::seeded(seed);
        env.draw(&graph, &weights, &params, &mut rng)
    };
    assert_eq!(pick(11), pick(11), "a replay draws the same candidate");

    let picks: BTreeSet<Option<u32>> = (1..=32).map(pick).collect();
    assert!(picks.len() > 1, "the seed actually steers the sampler: {picks:?}");
    assert!(picks.iter().flatten().all(|point| [A, B, C].contains(point)));
}

#[test]
fn an_idle_tick_writes_nothing_to_the_lattice() {
    let lattice = baked_lattice();
    let before = footprint(&lattice);
    let graph = Graph::build(&lattice, 0).unwrap();
    let env = Env::new();
    let params = params();
    let curve = flat_curve(8, 4096);
    let mut rng = Rng::seeded(3);
    let point = env
        .draw(&graph, &bake_only_weights(), &params, &mut rng)
        .expect("a baked candidate exists");
    let thought =
        cascade(&graph, point, &mut Integrator::at_birth(), 1, &[], &config(&params, &curve, 3, 1))
            .unwrap();
    assert!(!thought.settle_set.is_empty());
    assert_eq!(footprint(&lattice), before, "ignite + cascade leave every arena untouched");
}

#[test]
fn the_silt_term_is_inert_at_weight_zero_and_steers_when_armed() {
    let lattice = baked_lattice();
    let graph = Graph::build(&lattice, 0).unwrap();
    let params = params();
    let mut env = Env::new();
    env.silt.insert(C, 4096);
    let unarmed = bake_only_weights();
    let armed = IdleWeights { silt_q12: 4096, ..bake_only_weights() };
    let mut shifted = 0;
    for seed in 1..=32u64 {
        let mut a = Rng::seeded(seed);
        let mut b = Rng::seeded(seed);
        let with_silt = env.draw(&graph, &unarmed, &params, &mut a);
        env.silt.clear();
        let without = env.draw(&graph, &unarmed, &params, &mut b);
        env.silt.insert(C, 4096);
        assert_eq!(with_silt, without, "IGN_WEIGHT_SILT = 0 must leave the draw untouched");
        let mut c = Rng::seeded(seed);
        let biased = env.draw(&graph, &armed, &params, &mut c);
        if biased != with_silt {
            shifted += 1;
        }
    }
    assert!(shifted > 0, "an armed fifth term must be able to move the pick");
}

#[test]
fn the_sampler_has_exactly_one_seeded_stream_and_no_host_entropy() {    let source = include_str!("../src/idle.rs");
    for banned in ["std::time", "SystemTime", "thread_rng", "rand::", "getrandom", "fastrand"] {
        assert!(!source.contains(banned), "the idle sampler reaches `{banned}`");
    }
    assert!(source.contains("rng.next_q12()"), "the one draw is a Q12 fraction from R");
    assert!(!source.contains("next_u64("), "ignition draws one value, not a stream");
}

#[test]
fn the_cascade_is_the_walk_with_the_quantum_lighting() {
    let lattice = baked_lattice();
    let graph = Graph::build(&lattice, 0).unwrap();
    let params = params();
    let curve = flat_curve(8, 4096);
    let config = config(&params, &curve, 3, 1);
    let from_cascade = cascade(&graph, A, &mut Integrator::at_birth(), 1, &[], &config).unwrap();
    let from_walk = si_walk::walk(
        &graph,
        &[si_walk::Lit { point: A, activation: LIGHT_QUANTUM }],
        &mut Integrator::at_birth(),
        1,
        &[],
        &[],
        &config,
    )
    .unwrap();
    assert_eq!(from_cascade, from_walk);
}
