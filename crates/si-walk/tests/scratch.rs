//! 11 §B14 DoD 1 / 12 §2.6, 04 §31 AC.2: the frame scratch is built from this
//! frame's own pulse log at the end of the walk, owned by the `Thought`, and
//! destroyed when the `Thought` is.
//!
//! Every fixture shipped with `ghost_max = 0` and `shape_max = 0`, so the whole
//! branch below — the ghost, the role graph, and the caps' degradation-by-omission
//! — was code no test had ever run. The walk's own legs are the control here: the
//! scratch is a *readout* of the pulse log, so moving the caps may cost scratch
//! and may not cost a single light, round or activation.
//!
//! Every number here is a fixture value, not a registry reading.

mod common;

use common::{add_row, config, flat_curve, params, A, B, C};
use si_affect::Integrator;
use si_core::frame::Role;
use si_store::store::Lattice;
use si_walk::{walk, Graph, Lit, LIGHT_QUANTUM};

/// `A → B → C` and `C → B`: two round-0 seeds — the ghost needs a *set*, and one
/// token lights a frame with nothing to cluster.
fn pair() -> Lattice {
    let mut lattice = Lattice::new();
    add_row(&mut lattice, A, &[(B, 4096, 0, 128)]);
    add_row(&mut lattice, C, &[(B, 4096, 0, 128)]);
    lattice
}

fn scratched(ghost_max: u16, shape_max: u16) -> si_walk::Thought {
    let lattice = pair();
    let graph = Graph::build(&lattice, 0).unwrap();
    let params = params();
    let curve = flat_curve(8, 4096);
    let mut settings = config(&params, &curve, 3, 1);
    settings.ghost_max = ghost_max;
    settings.shape_max = shape_max;
    let mut integrator = Integrator::at_birth();
    walk(
        &graph,
        &[Lit { point: A, activation: LIGHT_QUANTUM }, Lit { point: C, activation: LIGHT_QUANTUM }],
        &mut integrator,
        1,
        &[],
        &[],
        &settings,
    )
    .unwrap()
}

#[test]
fn the_scratch_is_this_frames_pulse_log_read_back_once() {
    let thought = scratched(2, 2);
    let log = &thought.frame_log;

    // The lights are the frame: round 0 carries both seeds, and a later round
    // carries the filler they share.
    let seeds: Vec<u32> = log
        .lights
        .iter()
        .filter(|light| light.role == Role::Seed && light.round == 0)
        .map(|light| light.point)
        .collect();
    assert_eq!(seeds, vec![A, C], "the seed lights are the tokens, in pulse order");
    assert!(log.lights.iter().any(|light| light.round > 0), "the chain conducted");
    let fillers: Vec<u16> = log
        .lights
        .iter()
        .filter(|light| light.role == Role::SharedFiller)
        .map(|light| light.edge_kind)
        .collect();
    assert!(!fillers.is_empty(), "B is the filler both seeds carried");

    // docs/22 §22.2: the ghost is the frame's seed *set*, formed here and now.
    assert_eq!(log.ghosts.len(), 1);
    assert_eq!(log.ghosts[0].members, vec![A, C]);
    assert_eq!(log.ghosts[0].formed_in_frame, 1);
    assert_eq!(log.ghosts[0].frame_seed, u64::from(A));

    // docs/25 §25.5's template: roles present, kinds traversed, filler rounds.
    assert_eq!(log.role_graphs.len(), 1);
    let graph = &log.role_graphs[0];
    assert_eq!(graph.roles[0], Role::Seed.byte(), "the role list is sorted and deduplicated");
    assert!(graph.roles.contains(&Role::SharedFiller.byte()));
    assert!(!graph.roles.contains(&Role::Ghost.byte()), "a ghost is not a role a walk lit");
    assert_eq!(graph.traversed_kinds, vec![0], "kind 0 is ATTRIBUTE, not 'no edge' (AD.2)");
    assert_eq!(graph.filler_sets, fillers.len() as u16);

    // Nothing survives the call to be carried into the next frame: a second walk
    // of the same lattice rebuilds the identical scratch from its own pulse log.
    assert_eq!(scratched(2, 2).frame_log, *log);
}

/// AC.2 / OD-99: an over-bound cap means **less scratch, never a truncated
/// walk**. `ghost_max = 0` and `shape_max = 0` are the shipped registry floors, so
/// this is the state every gate run is in — and the state the two detectors must
/// still be able to read a frame from.
#[test]
fn a_cap_of_zero_builds_no_scratch_and_costs_the_walk_nothing() {
    let capped = scratched(0, 0);
    let armed = scratched(2, 2);
    assert!(capped.frame_log.ghosts.is_empty());
    assert!(capped.frame_log.role_graphs.is_empty());
    assert!(!capped.frame_log.lights.is_empty(), "the lights are not a cap's object");
    assert_eq!(capped.frame_log.lights, armed.frame_log.lights);
    assert_eq!(capped.rounds.len(), armed.rounds.len());
    let activations: Vec<Vec<(u32, i32)>> = capped
        .rounds
        .iter()
        .map(|round| round.activations.iter().map(|a| (a.point, a.activation)).collect())
        .collect();
    let armed_activations: Vec<Vec<(u32, i32)>> = armed
        .rounds
        .iter()
        .map(|round| round.activations.iter().map(|a| (a.point, a.activation)).collect())
        .collect();
    assert_eq!(activations, armed_activations, "the cap truncated the walk");
    assert_eq!(capped.settle_set, armed.settle_set);
    assert_eq!(capped.stop_reason, armed.stop_reason);
}

/// `GHOST_MAX` is a count of ghosts, so `1` is the shipped state and `2` must not
/// invent a second ghost out of the same frame (12 §2.6: the build is one set).
#[test]
fn a_wider_ghost_cap_adds_no_ghost() {
    assert_eq!(scratched(1, 1).frame_log.ghosts.len(), 1);
    assert_eq!(scratched(9, 9).frame_log.ghosts.len(), 1, "a frame has one seed set");
    assert_eq!(scratched(9, 9).frame_log.role_graphs.len(), 1);
}
