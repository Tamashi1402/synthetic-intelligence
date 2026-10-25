//! 04 §5 steps 6–9: spread order, the per-source reach cap, hub damp, the NOW
//! filter, and the two determinism claims (same lattice twice; insertion order
//! may move citations but never behaviour).

mod common;

use common::{add_row, chain, config, flat_curve, params, A, B, C};
use si_affect::Integrator;
use si_walk::{walk, Graph, Lit, StopReason, LIGHT_QUANTUM};

fn two_edge_lattice() -> si_store::store::Lattice {
    let mut lattice = si_store::store::Lattice::new();
    add_row(&mut lattice, A, &[(C, 4096, 0, 128), (B, 2048, 0, 128)]);
    lattice
}

#[test]
fn three_rounds_of_the_chain_match_the_hand_computation() {
    let lattice = chain();
    let graph = Graph::build(&lattice, 0).unwrap();
    let params = params();
    let curve = flat_curve(8, 4096);
    let mut integrator = Integrator::at_birth();
    let thought = walk(
        &graph,
        &[Lit { point: A, activation: LIGHT_QUANTUM }],
        &mut integrator,
        1,
        &[],
        &[],
        &config(&params, &curve, 3, 1),
    )
    .unwrap();

    let round = |index: usize| -> Vec<(u32, i32)> {
        thought.rounds[index]
            .activations
            .iter()
            .map(|entry| (entry.point, entry.activation))
            .collect()
    };
    assert_eq!(round(0), vec![(A, 2048), (B, 2048)]);
    assert_eq!(round(1), vec![(A, 1024), (B, 2048), (C, 1024)]);
    assert_eq!(round(2), vec![(A, 512), (B, 1536), (C, 1536)]);
    assert_eq!(thought.stop_reason, StopReason::MaxK);
    assert_eq!(thought.rounds.len(), 3);
    assert!(!thought.oscillated);

    // base_total is the frontier's plain Σ act·W·damp, before any bias.
    let base: Vec<u16> = thought.rounds.iter().map(|round| round.base_total_q12).collect();
    assert_eq!(base, vec![4096, 4096, 3072]);
    // No bake and no median: every bias is zero, so the affective share is 0.
    assert!(thought.rounds.iter().all(|round| round.bias_total_q12 == 0));

    let first: Vec<(u32, u16)> = thought
        .settle_set
        .iter()
        .map(|entry| (entry.point, entry.first_lit))
        .collect();
    assert_eq!(first, vec![(A, 0), (B, 1), (C, 2)]);
    // The settle-set is stored by point id; the display order is the other one.
    let shown: Vec<u32> = thought.display(2).iter().map(|entry| entry.point).collect();
    assert_eq!(shown, vec![B, C], "equal activations break by first-lit round");
    assert_eq!(thought.a_trajectory, vec![(1, 0), (2, 0), (3, 0)]);
}

#[test]
fn one_source_spreads_in_offset_order_and_the_reach_cap_keeps_the_heaviest() {
    let lattice = two_edge_lattice();
    let graph = Graph::build(&lattice, 0).unwrap();
    let params = params();

    let wide = flat_curve(8, 4096);
    let thought = walk(
        &graph,
        &[Lit { point: A, activation: LIGHT_QUANTUM }],
        &mut Integrator::at_birth(),
        1,
        &[],
        &[],
        &config(&params, &wide, 1, 1),
    )
    .unwrap();
    let first = &thought.rounds[0];
    let route: Vec<(u32, i32)> =
        first.contributions.iter().map(|entry| (entry.to_point, entry.amount)).collect();
    // Ascending arena offset is the insertion order inside the row: C first.
    assert_eq!(route, vec![(C, 2048), (B, 1024)]);
    // deg(A) = 2, so hub damp halves each contribution (04 §5 step 7).
    assert_eq!(first.base_total_q12, 3072);
    assert_eq!(
        first.activations.iter().map(|entry| (entry.point, entry.activation)).collect::<Vec<_>>(),
        vec![(A, 2048), (B, 512), (C, 1024)]
    );

    let narrow = flat_curve(1, 4096);
    let capped = walk(
        &graph,
        &[Lit { point: A, activation: LIGHT_QUANTUM }],
        &mut Integrator::at_birth(),
        1,
        &[],
        &[],
        &config(&params, &narrow, 1, 1),
    )
    .unwrap();
    let capped_route: Vec<u32> =
        capped.rounds[0].contributions.iter().map(|entry| entry.to_point).collect();
    assert_eq!(capped_route, vec![C], "reach 1 keeps the highest weight_eff edge");
}

#[test]
fn inserting_the_same_rows_in_another_order_keeps_the_behaviour() {
    let first = chain();
    let mut second = si_store::store::Lattice::new();
    add_row(&mut second, B, &[(C, 4096, 0, 128)]);
    add_row(&mut second, A, &[(B, 4096, 0, 128)]);

    let params = params();
    let curve = flat_curve(8, 4096);
    let run = |lattice: &si_store::store::Lattice| {
        walk(
            &Graph::build(lattice, 0).unwrap(),
            &[Lit { point: A, activation: LIGHT_QUANTUM }],
            &mut Integrator::at_birth(),
            1,
            &[],
            &[],
            &config(&params, &curve, 3, 1),
        )
        .unwrap()
    };
    let left = run(&first);
    let right = run(&second);

    assert_eq!(left.behavioural_trace(), right.behavioural_trace());
    assert_eq!(left.settle_set, right.settle_set);
    // The citations really do move; that is why the behavioural trace is the
    // object the determinism claim is made over.
    assert_ne!(left.rounds[0].contributions[0].edge_off, right.rounds[0].contributions[0].edge_off);
}

#[test]
fn the_same_lattice_walked_twice_is_byte_for_byte_the_same_thought() {
    let lattice = chain();
    let graph = Graph::build(&lattice, 0).unwrap();
    let params = params();
    let curve = flat_curve(8, 4096);
    let run = || {
        walk(
            &graph,
            &[Lit { point: A, activation: LIGHT_QUANTUM }],
            &mut Integrator::at_birth(),
            1,
            &[],
            &[],
            &config(&params, &curve, 3, 1),
        )
        .unwrap()
    };
    assert_eq!(run(), run());
}

#[test]
fn a_superseded_edge_is_unreachable_at_the_present_anchor() {
    let mut lattice = si_store::store::Lattice::new();
    let old = add_row(&mut lattice, A, &[(B, 4096, 0, 128)]);
    let successor = add_row(&mut lattice, C, &[(A, 4096, 0, 128)]);
    lattice.age(4, old, 5, successor).unwrap();

    assert_eq!(lattice.out_degree(A), 0, "aging zeroes the weight and leaves deg");
    let graph = Graph::build(&lattice, 5).unwrap();
    assert!(graph.arcs(A).is_empty());
    assert_eq!(graph.out_degree(A), 0);
    // I4 is literal: the edge is weight 0 forever, so no anchor sees it
    // through the live web. Only a store-provided history anchor can restore it.
    let before = Graph::build(&lattice, 4).unwrap();
    assert!(before.arcs(A).is_empty());

    let params = params();
    let curve = flat_curve(8, 4096);
    let thought = walk(
        &graph,
        &[Lit { point: A, activation: LIGHT_QUANTUM }],
        &mut Integrator::at_birth(),
        1,
        &[],
        &[],
        &config(&params, &curve, 2, 1),
    )
    .unwrap();
    assert_eq!(
        thought.rounds[0].activations.iter().map(|entry| (entry.point, entry.activation)).collect::<Vec<_>>(),
        vec![(A, 2048)],
        "an aged edge contributes nothing"
    );
}

#[test]
fn the_graph_and_the_store_agree_on_live_out_degree() {
    let mut lattice = two_edge_lattice();
    add_row(&mut lattice, C, &[(A, 4096, 0, 128)]);
    let graph = Graph::build(&lattice, lattice.now()).unwrap();
    for point in [A, B, C] {
        assert_eq!(graph.out_degree(point), lattice.out_degree(point), "point {point}");
    }
    assert_eq!(graph.out_degree(A), 2);
}

#[test]
fn the_route_composes_across_two_rounds_with_no_compose_step() {
    let lattice = chain();
    let graph = Graph::build(&lattice, 0).unwrap();
    let params = params();
    let curve = flat_curve(8, 4096);
    let thought = walk(
        &graph,
        &[Lit { point: A, activation: LIGHT_QUANTUM }],
        &mut Integrator::at_birth(),
        1,
        &[],
        &[],
        &config(&params, &curve, 2, 1),
    )
    .unwrap();

    let route: Vec<(u16, u32, u32)> = thought
        .rounds
        .iter()
        .flat_map(|round| {
            round
                .contributions
                .iter()
                .map(move |entry| (round.round, entry.source, entry.to_point))
        })
        .collect();
    assert_eq!(route, vec![(1, A, B), (2, A, B), (2, B, C)]);
    assert!(
        thought.settle_set.iter().any(|entry| entry.point == C),
        "A → B → C arrives although nobody taught A → C"
    );
    // The route is in the trace, so the labeler can count hops without the walk
    // ever forming an edge.
    assert!(route.contains(&(2, B, C)));
}

#[test]
fn the_spread_accumulator_is_single_threaded_integer_arithmetic() {
    for (name, source) in [
        ("walk.rs", include_str!("../src/walk.rs")),
        ("graph.rs", include_str!("../src/graph.rs")),
        ("idle.rs", include_str!("../src/idle.rs")),
    ] {
        for banned in ["std::thread", "std::sync", "rayon", "Atomic", "Mutex", "RwLock", "par_iter"] {
            assert!(!source.contains(banned), "{name} reaches `{banned}`");
        }
    }
}
