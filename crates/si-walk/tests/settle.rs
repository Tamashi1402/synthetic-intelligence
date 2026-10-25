//! 04 §5 steps 3, 10, 11: the four stops, the trim, the τ test, and the lagged
//! pointer. Each leg asserts a value the step table names, not a shape.

mod common;

use common::{chain, config, flat_curve, params, params_with_lag, A, B, C};
use si_affect::Integrator;
use si_store::store::Lattice;
use si_walk::{walk, Graph, Lit, OscillationRule, StopReason, LIGHT_QUANTUM};

fn lit_a() -> [Lit; 1] {
    [Lit { point: A, activation: LIGHT_QUANTUM }]
}

#[test]
fn a_tau_above_the_first_diff_settles_on_round_one() {
    let lattice = chain();
    let graph = Graph::build(&lattice, 0).unwrap();
    let params = params();
    let curve = flat_curve(8, 4096);
    let thought = walk(
        &graph,
        &lit_a(),
        &mut Integrator::at_birth(),
        1,
        &[],
        &[],
        &config(&params, &curve, 8, 65535),
    )
    .unwrap();
    assert_eq!(thought.stop_reason, StopReason::Settled);
    assert_eq!(thought.rounds.len(), 1);
    assert_eq!(thought.rounds[0].diff_q12, 4096);
    assert_eq!(
        thought.settle_set.iter().map(|entry| (entry.point, entry.activation)).collect::<Vec<_>>(),
        vec![(A, 2048), (B, 2048)]
    );
}

#[test]
fn running_out_of_rounds_is_max_k_not_an_answer() {
    let lattice = chain();
    let graph = Graph::build(&lattice, 0).unwrap();
    let params = params();
    let curve = flat_curve(8, 4096);
    let thought = walk(
        &graph,
        &lit_a(),
        &mut Integrator::at_birth(),
        1,
        &[],
        &[],
        &config(&params, &curve, 2, 1),
    )
    .unwrap();
    assert_eq!(thought.stop_reason, StopReason::MaxK);
    assert_eq!(thought.rounds.len(), 2);
    assert_eq!(thought.rounds[1].diff_q12, 2048);
}

#[test]
fn a_zero_retention_curve_leaves_no_candidate_and_stops_on_tau_miss() {
    let lattice = chain();
    let graph = Graph::build(&lattice, 0).unwrap();
    let params = params();
    let curve = flat_curve(8, 0);
    let thought = walk(
        &graph,
        &lit_a(),
        &mut Integrator::at_birth(),
        1,
        &[],
        &[],
        &config(&params, &curve, 4, 1),
    )
    .unwrap();
    assert_eq!(thought.stop_reason, StopReason::TauMiss);
    assert_eq!(thought.rounds.len(), 1);
    assert_eq!(thought.rounds[0].diff_q12, 4096);
    assert!(thought.rounds[0].frontier.is_empty());
    assert!(thought.settle_set.is_empty());
}

#[test]
fn an_empty_lighting_is_nothing_lit_and_not_an_error() {
    let lattice = chain();
    let graph = Graph::build(&lattice, 0).unwrap();
    let params = params();
    let curve = flat_curve(8, 4096);
    let thought = walk(
        &graph,
        &[],
        &mut Integrator::at_birth(),
        1,
        &[],
        &[],
        &config(&params, &curve, 4, 1),
    )
    .unwrap();
    assert_eq!(thought.stop_reason, StopReason::NothingLit);
    assert!(thought.rounds.is_empty());
    assert!(thought.settle_set.is_empty());
    assert!(thought.a_trajectory.is_empty());
}

#[test]
fn the_frontier_trim_keeps_the_loudest_points_and_never_touches_the_settle_set() {
    let lattice = chain();
    let graph = Graph::build(&lattice, 0).unwrap();
    let params = params();
    let curve = flat_curve(8, 4096);
    let mut config = config(&params, &curve, 2, 1);
    config.frontier_cap = 2;
    let thought = walk(&graph, &lit_a(), &mut Integrator::at_birth(), 1, &[], &[], &config).unwrap();

    let kept: Vec<u32> = thought.rounds[1].frontier.iter().map(|entry| entry.point).collect();
    assert_eq!(kept, vec![B, A], "B is loudest; A and C tie on activation, A lit first");
    let settled: Vec<u32> = thought.settle_set.iter().map(|entry| entry.point).collect();
    assert_eq!(settled, vec![A, B, C], "the trim is of propagation, not of the settle-set");
}

#[test]
fn an_activation_tie_drops_by_point_id_not_by_insertion() {
    let lattice = Lattice::new();
    let graph = Graph::build(&lattice, 0).unwrap();
    let params = params();
    let curve = flat_curve(8, 4096);
    let mut config = config(&params, &curve, 1, 1);
    config.frontier_cap = 1;
    let thought = walk(
        &graph,
        &[Lit { point: C, activation: LIGHT_QUANTUM }, Lit { point: A, activation: LIGHT_QUANTUM }],
        &mut Integrator::at_birth(),
        1,
        &[],
        &[],
        &config,
    )
    .unwrap();
    assert_eq!(
        thought.rounds[0].frontier.iter().map(|entry| entry.point).collect::<Vec<_>>(),
        vec![A]
    );
    assert_eq!(thought.settle_set.len(), 2);
}

#[test]
fn the_pointer_conducts_with_the_value_produced_mod_lag_rounds_ago() {
    let lattice = Lattice::new();
    let graph = Graph::build(&lattice, 0).unwrap();
    // A restored `a` replaces the birth 0, so the pointer has a non-zero value
    // to hold back; with no gains, only LAG_UNIT's decay moves it.
    let params = params_with_lag(2);
    let curve = flat_curve(8, 4096);
    let thought = walk(
        &graph,
        &lit_a(),
        &mut Integrator::restore(255),
        1,
        &[],
        &[],
        &config(&params, &curve, 3, 1),
    )
    .unwrap();
    let used: Vec<u8> = thought.rounds.iter().map(|round| round.a_used).collect();
    let after: Vec<u8> = thought.rounds.iter().map(|round| round.a_after).collect();
    // Rounds 1–2 conduct with the pre-thought value; round 3 conducts with the
    // value round 1 produced. A same-round or one-round read would show 128 here.
    assert_eq!(used, vec![255, 255, 128]);
    assert_eq!(after, vec![128, 64, 32]);
    assert_eq!(thought.a_trajectory, vec![(1, 128), (2, 64), (3, 32)]);
}

#[test]
fn the_period_check_still_ends_in_the_uncertainty_readout() {
    let lattice = chain();
    let graph = Graph::build(&lattice, 0).unwrap();
    let params = params();
    let curve = flat_curve(8, 4096);
    let mut config = config(&params, &curve, 2, 1);
    config.oscillation = OscillationRule::PeriodCheck;
    let thought = walk(&graph, &lit_a(), &mut Integrator::at_birth(), 1, &[], &[], &config).unwrap();
    assert_eq!(thought.stop_reason, StopReason::MaxK);
    assert!(!thought.oscillated, "a monotone run is not a 2-cycle");
}

#[test]
fn the_walk_crate_has_no_edge_to_the_write_path() {
    let manifest = include_str!("../Cargo.toml");
    assert!(!manifest.contains("si-learn"));
    assert!(!manifest.contains("si_learn"));
    assert!(!manifest.contains("si-audit"));
    assert!(!manifest.contains("si-sleep"));
}
