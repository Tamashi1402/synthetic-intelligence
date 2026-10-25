use si_affect::arousal::Integrator;
use si_affect::params::{LagUnit, Params};
use si_affect::signal::{Signal, SignalKind};
use si_affect::{bake_q12, update};

fn params() -> Params {
    Params {
        lambda_a: 128,
        gain: [1024, 2048, 0, 0, 0, 0],
        beta: 4096,
        gamma: 4096,
        ask_cap_per_input: 2,
        priority: [4096, 4096, 4096],
        gap_salience_floor: 0,
        dedupe_window: 16,
        bias_cap_share: 2048,
        b_max: 32768,
        k_tanh: 32768,
        median_min_n: 1,
        median_shape: 0,
        lag_rounds: 33,
        lag_unit: LagUnit::PerRound,
    }
}

#[test]
fn the_hand_computed_vector_matches_to_the_integer() {
    let params = params();
    let mut integrator = Integrator::at_birth();
    let mut trajectory = Vec::new();
    let first = update(
        &mut integrator,
        &[Signal::new(SignalKind::NoveltySpike, 4096), Signal::new(SignalKind::RealizedGain, 2048)],
        7,
        0,
        &params,
        &mut trajectory,
    );
    assert_eq!(first.a, 128);
    assert_eq!(first.applies_at, 40);
    let quiet = update(&mut integrator, &[], 8, 0, &params, &mut trajectory);
    assert_eq!(quiet.a, 64, "an empty boundary is decay, never reset");
    let third = update(
        &mut integrator,
        &[Signal::new(SignalKind::RealizedGain, 4096)],
        9,
        0,
        &params,
        &mut trajectory,
    );
    assert_eq!(third.a, 160);
    assert_eq!(trajectory, vec![(7, 64), (7, 128), (8, 64), (9, 160)]);
}

#[test]
fn events_apply_in_emission_order() {
    let params = params();
    let mut forward = Integrator::at_birth();
    let mut forward_trace = Vec::new();
    update(
        &mut forward,
        &[Signal::new(SignalKind::NoveltySpike, 4096), Signal::new(SignalKind::RealizedGain, 4096)],
        1,
        0,
        &params,
        &mut forward_trace,
    );
    let mut reverse = Integrator::at_birth();
    let mut reverse_trace = Vec::new();
    update(
        &mut reverse,
        &[Signal::new(SignalKind::RealizedGain, 4096), Signal::new(SignalKind::NoveltySpike, 4096)],
        1,
        0,
        &params,
        &mut reverse_trace,
    );
    assert_eq!(forward_trace, vec![(1, 64), (1, 192)]);
    assert_eq!(reverse_trace, vec![(1, 128), (1, 192)]);
    assert_eq!(forward_trace.last(), reverse_trace.last());
}

#[test]
fn the_lag_records_the_round_applied() {
    let mut changed = params();
    changed.lag_rounds = 3;
    let mut integrator = Integrator::at_birth();
    let lagged = update(
        &mut integrator,
        &[Signal::new(SignalKind::Contradiction, 4096)],
        11,
        0,
        &changed,
        &mut Vec::new(),
    );
    assert_eq!(lagged.applies_at, 14);
}

#[test]
fn arousal_stays_on_the_grid_and_saturates() {
    let params = params();
    let mut previous = 0u8;
    let mut last = 0u8;
    for magnitude in (0..=u16::MAX).step_by(4096) {
        let mut integrator = Integrator::at_birth();
        let lagged = update(
            &mut integrator,
            &[Signal::new(SignalKind::NoveltySpike, magnitude)],
            1,
            0,
            &params,
            &mut Vec::new(),
        );
        assert!(lagged.a >= previous, "a larger signal never lowers a");
        previous = lagged.a;
        last = lagged.a;
    }
    assert_eq!(last, 255, "the biggest gain fills the grid");

    // Contradiction has gain 0 here, so restore(255) would only decay; the
    // saturating add is exercised with a kind that carries a gain.
    let mut saturated = Integrator::restore(255);
    let lagged = update(
        &mut saturated,
        &[Signal::new(SignalKind::NoveltySpike, u16::MAX)],
        1,
        0,
        &params,
        &mut Vec::new(),
    );
    assert_eq!(lagged.a, 255, "the clamp is the grid's top, not a wrap");
}

#[test]
fn per_token_decays_before_each_event() {
    let mut per_token = params();
    per_token.lag_unit = LagUnit::PerToken;
    per_token.gain[0] = 2048;
    let mut integrator = Integrator::restore(128);
    let mut trajectory = Vec::new();
    update(
        &mut integrator,
        &[Signal::new(SignalKind::NoveltySpike, 4096), Signal::new(SignalKind::NoveltySpike, 4096)],
        1,
        0,
        &per_token,
        &mut trajectory,
    );
    assert_eq!(trajectory, vec![(1, 192), (1, 224)]);
}

#[test]
fn bake_is_the_one_curve_into_the_write_side() {
    let params = params();
    assert_eq!(bake_q12(0, params.b_max, params.k_tanh), 0, "zero affect bakes zero");
    let mut previous = 0u16;
    for a in 0..=255u8 {
        let value = bake_q12(a, params.b_max, params.k_tanh);
        assert!(value >= previous, "bake is monotone in a_formation");
        assert!(value <= params.b_max, "B_max is the hard ceiling");
        previous = value;
    }
    assert_eq!(
        bake_q12(255, params.b_max, params.k_tanh),
        params.b_max,
        "the top of the grid saturates at B_max, never above it"
    );
}
