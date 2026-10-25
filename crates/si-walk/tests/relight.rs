//! 04 §4 note 7 / §31 AC.2: step 5b's tugs light after the token lights, in
//! `(event_id, point)` order, capped at `RELIGHT_MAX`, and write nothing.
//!
//! Every number here is a fixture value, not a registry reading.

mod common;

use common::{chain, config, flat_curve, params, A, B, C};
use si_affect::Integrator;
use si_core::episodic::OriginTug;
use si_walk::{walk, Graph, Lit, LIGHT_QUANTUM};

fn lit_a() -> Vec<Lit> {
    vec![Lit { point: A, activation: LIGHT_QUANTUM }]
}

#[test]
fn tugs_light_in_event_then_point_order_and_the_cap_is_a_prefix() {
    let lattice = chain();
    let graph = Graph::build(&lattice, i32::MAX).unwrap();
    let params = params();
    let curve = flat_curve(1, 4096);
    let tugs = vec![
        OriginTug { event_id: 7, points: vec![A, B], weight_q12: 64 },
        OriginTug { event_id: 3, points: vec![C], weight_q12: 32 },
    ];
    let mut integrator = Integrator::at_birth();
    let mut settings = config(&params, &curve, 2, 1);
    settings.tugs = &tugs;
    settings.relight_max = 8;
    let thought = walk(&graph, &lit_a(), &mut integrator, 1, &[], &[], &settings).unwrap();
    let pairs: Vec<(u32, u32, i32)> = thought
        .relit
        .iter()
        .map(|relight| (relight.event_id, relight.point, relight.activation))
        .collect();
    assert_eq!(pairs, vec![(3, C, 32), (7, A, 64), (7, B, 64)]);

    settings.relight_max = 1;
    let mut integrator = Integrator::at_birth();
    let capped = walk(&graph, &lit_a(), &mut integrator, 1, &[], &[], &settings).unwrap();
    assert_eq!(capped.relit.len(), 1);
    assert_eq!((capped.relit[0].event_id, capped.relit[0].point), (3, C));
}

#[test]
fn the_re_lit_point_is_a_round_zero_light_and_carries_the_primed_tag() {
    let lattice = chain();
    let graph = Graph::build(&lattice, i32::MAX).unwrap();
    let params = params();
    let curve = flat_curve(1, 4096);
    let tugs = vec![OriginTug { event_id: 3, points: vec![C], weight_q12: 32 }];
    let mut settings = config(&params, &curve, 1, 1);
    settings.tugs = &tugs;
    settings.relight_max = 8;
    let mut integrator = Integrator::at_birth();
    let thought = walk(&graph, &lit_a(), &mut integrator, 1, &[], &[], &settings).unwrap();
    let seed = thought
        .frame_log
        .lights
        .iter()
        .find(|light| light.round == 0 && light.point == C)
        .expect("the re-lit point is a round-0 light");
    assert!(seed.primed);
    let token = thought
        .frame_log
        .lights
        .iter()
        .find(|light| light.round == 0 && light.point == A)
        .expect("the token light is there");
    assert!(!token.primed);
}

#[test]
fn off_is_the_shipped_state_and_changes_no_activation() {
    let lattice = chain();
    let graph = Graph::build(&lattice, i32::MAX).unwrap();
    let params = params();
    let curve = flat_curve(1, 4096);
    let tugs = vec![OriginTug { event_id: 3, points: vec![C], weight_q12: 4096 }];
    let mut settings = config(&params, &curve, 2, 1);
    settings.tugs = &tugs;
    settings.relight_max = 0;
    let mut left = Integrator::at_birth();
    let off = walk(&graph, &lit_a(), &mut left, 1, &[], &[], &settings).unwrap();

    // AC.1: `EPISODIC_TRAVERSAL = off` hands the walk the empty vector, and the
    // arithmetic is bit-identical to the pre-B16 call.
    let empty: Vec<OriginTug> = Vec::new();
    let mut settings = config(&params, &curve, 2, 1);
    settings.tugs = &empty;
    settings.relight_max = 4096;
    let mut right = Integrator::at_birth();
    let baseline = walk(&graph, &lit_a(), &mut right, 1, &[], &[], &settings).unwrap();
    assert!(off.relit.is_empty());
    assert_eq!(off, baseline);
}

#[test]
fn an_empty_token_lighting_stays_nothing_lit_even_with_a_tug() {
    let lattice = chain();
    let graph = Graph::build(&lattice, i32::MAX).unwrap();
    let params = params();
    let curve = flat_curve(1, 4096);
    let tugs = vec![OriginTug { event_id: 3, points: vec![C], weight_q12: 4096 }];
    let mut settings = config(&params, &curve, 2, 1);
    settings.tugs = &tugs;
    settings.relight_max = 8;
    let mut integrator = Integrator::at_birth();
    let thought = walk(&graph, &[], &mut integrator, 1, &[], &[], &settings).unwrap();
    assert_eq!(thought.stop_reason, si_walk::StopReason::NothingLit);
    assert!(thought.relit.is_empty());
}

#[test]
fn a_zero_weight_tug_lights_nothing() {
    let lattice = chain();
    let graph = Graph::build(&lattice, i32::MAX).unwrap();
    let params = params();
    let curve = flat_curve(1, 4096);
    let tugs = vec![OriginTug { event_id: 3, points: vec![C], weight_q12: 0 }];
    let mut settings = config(&params, &curve, 2, 1);
    settings.tugs = &tugs;
    settings.relight_max = 8;
    let mut integrator = Integrator::at_birth();
    let thought = walk(&graph, &lit_a(), &mut integrator, 1, &[], &[], &settings).unwrap();
    assert!(thought.relit.is_empty());
}
