use si_affect::drive::{
    budget, conduction, emit, prioritize, EdgeCharge, Gap, PointMedian, DOORS,
};
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
        gap_salience_floor: 2048,
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

const DRIVE_SOURCE: &str = include_str!("../src/drive.rs");

#[test]
fn there_are_exactly_six_doors_and_no_seventh() {
    let mut public_functions: Vec<&str> = DRIVE_SOURCE
        .lines()
        .filter_map(|line| line.strip_prefix("pub fn "))
        .filter_map(|rest| rest.split('(').next())
        .collect();
    public_functions.sort_unstable();
    let mut doors = DOORS.to_vec();
    doors.sort_unstable();
    assert_eq!(public_functions, doors);
}

#[test]
fn budget_reads_no_affect() {
    let params = params();
    let result = budget(&params);
    assert_eq!(result.cap_per_input, params.ask_cap_per_input);
    assert_eq!(result.dedupe_window, params.dedupe_window);
    assert_eq!(result.salience_floor, params.gap_salience_floor);
    let signature = DRIVE_SOURCE
        .lines()
        .find(|line| line.starts_with("pub fn budget"))
        .expect("budget is a door");
    assert!(!signature.contains("a:"), "budget takes no affective level: {signature}");
}

#[test]
fn prioritize_ranks_by_weight_then_slot() {
    let params = params();
    let now = 100;
    let gaps = [
        Gap { slot: 2, salience_q12: 4096, last_pulsed_at: now, usage_q12: 4096 },
        Gap { slot: 1, salience_q12: 4096, last_pulsed_at: now, usage_q12: 4096 },
        Gap { slot: 3, salience_q12: 8192, last_pulsed_at: now, usage_q12: 4096 },
    ];
    let ranked = prioritize(&gaps, now, &params);
    assert_eq!(ranked[0].gap.slot, 3);
    assert_eq!(ranked[1].gap.slot, 1);
    assert_eq!(ranked[2].gap.slot, 2);
}

#[test]
fn emit_respects_the_floor_then_the_cap() {
    let params = params();
    let gaps = [
        Gap { slot: 10, salience_q12: 4096, last_pulsed_at: 0, usage_q12: 4096 },
        Gap { slot: 11, salience_q12: 1024, last_pulsed_at: 0, usage_q12: 4096 },
        Gap { slot: 12, salience_q12: 4096, last_pulsed_at: 0, usage_q12: 4096 },
    ];
    let ranked = prioritize(&gaps, 100, &params);
    let asks = emit(&ranked, &params);
    assert_eq!(asks.len(), 2);
    assert!(asks.iter().all(|ask| ask.slot != 11));
}

#[test]
fn conduction_clips_the_affective_share_once_and_cites_every_edge() {
    let params = params();
    let edges = [
        EdgeCharge { edge_off: 32, to_point: 7, bake: 32768, a_at: 100 },
        EdgeCharge { edge_off: 16, to_point: 7, bake: 32768, a_at: 100 },
        EdgeCharge { edge_off: 48, to_point: 9, bake: 32768, a_at: 100 },
    ];
    let medians = [
        PointMedian { point: 7, median_q12: 32768 },
        PointMedian { point: 9, median_q12: 32768 },
    ];
    let biases = conduction(100, &edges, &medians, 4096, &params);
    assert_eq!(biases.len(), 3);
    assert_eq!(biases.iter().map(|b| b.edge_off).collect::<Vec<_>>(), vec![16, 32, 48]);
    let total: u32 = biases.iter().map(|b| b.bias_q12 as u32).sum();
    assert!(total <= 2050, "the affective share {total} exceeds the cap 2048");
    for bias in &biases {
        assert_eq!(bias.a_now, 100);
        assert_eq!(bias.a_at, 100);
        assert_eq!(bias.median_used, 32768);
        assert!(bias.bias_q12 <= bias.sigma_q12.saturating_add(bias.median_term_q12));
    }
}

#[test]
fn sigma_compares_a_at_and_never_multiplies_it() {
    let params = params();
    let at_match = [EdgeCharge { edge_off: 0, to_point: 1, bake: 32768, a_at: 100 }];
    let at_other = [EdgeCharge { edge_off: 0, to_point: 1, bake: 32768, a_at: 100 }];
    let matched = conduction(100, &at_match, &[], u16::MAX, &params);
    let mismatched = conduction(200, &at_other, &[], u16::MAX, &params);
    assert!(matched[0].sigma_q12 > mismatched[0].sigma_q12);
    assert_eq!(matched[0].a_at, 100);
    assert_eq!(mismatched[0].a_now, 200);
}

#[test]
fn the_signal_enum_has_exactly_six_kinds() {
    assert_eq!(SignalKind::ALL.len(), 6);
    for (index, kind) in SignalKind::ALL.iter().enumerate() {
        assert_eq!(kind.index(), index);
        assert_eq!(SignalKind::from_u8(index as u8), *kind);
    }
}

#[test]
#[should_panic(expected = "not one of the six kinds")]
fn a_seventh_signal_kind_is_a_panic() {
    SignalKind::from_u8(6);
}

#[test]
fn the_crate_has_no_float() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.contains("f32") && !text.contains("f64"), "{} has a float", path.display());
    }
}
