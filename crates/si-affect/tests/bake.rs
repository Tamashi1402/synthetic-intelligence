use si_affect::bake_q12;
use si_affect::drive::{salience, Formation};
use si_affect::params::{LagUnit, Params};
use si_affect::signal::SignalKind;

fn params() -> Params {
    Params {
        lambda_a: 128,
        gain: [4096; 6],
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
fn bake_is_monotone_saturating_and_zero_at_zero() {
    let params = params();
    let mut previous = 0u16;
    for a in 0..=255u8 {
        let bake = bake_q12(a, params.b_max, params.k_tanh);
        assert!(bake >= previous, "bake fell from {previous} to {bake} at a={a}");
        assert!(bake <= params.b_max, "bake {bake} exceeds B_max at a={a}");
        previous = bake;
    }
    assert!(bake_q12(32, params.b_max, params.k_tanh) < bake_q12(96, params.b_max, params.k_tanh));
    assert!(bake_q12(255, params.b_max, params.k_tanh) >= params.b_max - 2);
}

#[test]
fn bake_never_reads_the_signal_and_the_stamp_always_names_it() {
    let params = params();
    let quiet = Formation {
        at_t: 3,
        thought_id: 9,
        signal_ref: 4,
        signal: SignalKind::NoveltySpike,
        magnitude: 1234,
    };
    let loud = Formation {
        at_t: 3,
        thought_id: 9,
        signal_ref: 4,
        signal: SignalKind::Contradiction,
        magnitude: 65535,
    };
    let left = salience(200, &quiet, &params);
    let right = salience(200, &loud, &params);
    assert_eq!(left.bake, right.bake, "the bake may not consult the event's kind or size");
    assert_eq!(left.stamp.signal, 0);
    assert_eq!(right.stamp.signal, 2);
    assert_eq!(left.stamp.magnitude, 1234);
    assert_eq!(right.stamp.magnitude, 65535);
    assert_eq!(left.stamp.a_at, 200);
    assert_eq!(left.stamp.src, 0);
    assert_eq!(left.bake, bake_q12(200, params.b_max, params.k_tanh));
}

#[test]
fn a_calm_formation_cannot_buy_a_bake() {
    let params = params();
    let formation = Formation {
        at_t: 1,
        thought_id: 1,
        signal_ref: 1,
        signal: SignalKind::NoveltySpike,
        magnitude: 4096,
    };
    let result = salience(0, &formation, &params);
    assert_eq!(result.bake, 0, "a_formation = 0 ⇒ bake = 0 is law (OD-56)");
}
