//! 02 §6 / 04 §7.3 / §20 T: retrieval cites what the thought reached, keyed on
//! the decoded slot; a cooccur-weak row or a zero-confidence row is never a
//! source (05 §20 `check_candidate_unanswerable`).

use std::collections::BTreeMap;

use si_affect::params::LagUnit;
use si_affect::{Integrator, Params};
use si_core::addr::SHELF_S_R_PAIRS;
use si_core::patterns::PatternId;
use si_core::record::{Body, Envelope};
use si_store::store::{EdgeDraft, Lattice};
use si_walk::{Curve, CurvePoint, LIGHT_QUANTUM, OscillationRule, WalkConfig};

use si_read::{light, light::Resolve, retrieve, ReadRequest, SlotKey, StopReason};

struct Map(BTreeMap<String, Vec<u32>>);

impl Map {
    fn new(entries: &[(&str, &[u32])]) -> Self {
        Self(
            entries
                .iter()
                .map(|(key, points)| ((*key).to_string(), points.to_vec()))
                .collect(),
        )
    }
}

impl Resolve for Map {
    fn aliases(&self, key: &str) -> Vec<u32> {
        self.0.get(key).cloned().unwrap_or_default()
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

fn config<'a>(params: &'a Params, curve: &'a Curve, k_max: u16) -> WalkConfig<'a> {
    WalkConfig {
        params,
        curve,
        k_max,
        tau_settle: 1,
        decay_base: 128,
        hub_exponent_q12: 4096,
        frontier_cap: 16,
        oscillation: OscillationRule::MaxKOnly,
        ghost_max: 0,
        shape_max: 0,
        // 04 §31 AC.1: a caller with no index passes the shipped off state.
        tugs: &[],
        relight_max: 0,
        composition_hint: &[],
    }
}

fn curve(reach: u16) -> Curve {
    Curve::fixture(CurvePoint { reach, decay_q12: 4096, settle_shift_q12: 0 })
}

fn row(
    lattice: &mut Lattice,
    key_point: u32,
    val_point: u32,
    kind: u8,
    confidence: u16,
    surface: &str,
) -> u32 {
    row_at(lattice, key_point, val_point, kind, confidence, surface, 1)
}

fn row_at(
    lattice: &mut Lattice,
    key_point: u32,
    val_point: u32,
    kind: u8,
    confidence: u16,
    surface: &str,
    written_at: i32,
) -> u32 {
    let resp_surface = lattice.interner_mut().intern(surface);
    let mut body = Body::default();
    body.pattern_id = PatternId::PQuestA.index() as u16;
    body.key_point = key_point;
    body.val_point = val_point;
    body.resp_surface = resp_surface;
    let mut edge = EdgeDraft::owned(val_point, 4096, 0);
    edge.kind = kind;
    let mut envelope = Envelope::live(SHELF_S_R_PAIRS, written_at, 0, 1);
    envelope.confidence = confidence;
    lattice.append_row(envelope, body, vec![edge], &[], &[]).unwrap()
}

fn request(keys: &[&str], slot: Option<u32>, anchor: i32) -> ReadRequest {
    ReadRequest {
        keys: keys.iter().map(|key| (*key).to_string()).collect(),
        frame_point: None,
        slot: slot.map(|point| SlotKey { frame: PatternId::PQuestA, point }),
        anchor,
        thought_id: 1,
    }
}

#[test]
fn lighting_shares_one_quantum_across_an_alias_set() {
    let resolve = Map::new(&[("apple", &[10, 11])]);
    let lighting = light(
        &si_read::LightRequest { keys: vec!["apple".to_string()], frame_point: Some(99) },
        &resolve,
    );
    assert_eq!(lighting.lit.len(), 3);
    assert_eq!(lighting.lit[0].point, 99);
    assert_eq!(lighting.lit[0].activation, LIGHT_QUANTUM);
    assert_eq!(lighting.lit[1].activation, LIGHT_QUANTUM / 2);
    assert_eq!(lighting.lit[2].activation, LIGHT_QUANTUM / 2);
    assert_eq!(lighting.aliases, vec![(0, vec![10, 11])]);
    assert!(lighting.novel.is_empty());
}

#[test]
fn a_token_with_no_hyperpoint_is_a_novelty_event_not_a_light() {
    let resolve = Map::new(&[("apple", &[10])]);
    let lighting = light(
        &si_read::LightRequest { keys: vec!["znark".to_string()], frame_point: None },
        &resolve,
    );
    assert!(lighting.lit.is_empty());
    assert_eq!(lighting.novel, vec![0]);
}

#[test]
fn retrieval_cites_only_the_decoded_slots_row() {
    let mut lattice = Lattice::new();
    let apple = row(&mut lattice, 10, 20, 0, 95, "it is red");
    let _pear = row(&mut lattice, 11, 21, 0, 95, "it is green");
    let resolve = Map::new(&[("apple", &[10]), ("pear", &[11])]);
    let params = params();
    let curve = curve(1);
    let config = config(&params, &curve, 2);
    let mut integrator = Integrator::at_birth();

    let read = retrieve(
        &lattice,
        lattice.interner(),
        &resolve,
        &mut integrator,
        &config,
        &request(&["apple", "pear"], Some(10), 10),
    )
    .unwrap();
    assert_eq!(read.cited.len(), 1);
    assert_eq!(read.cited[0].row_id, apple);
    assert_eq!(read.cited[0].key_point, 10);
    assert_eq!(read.answer, Some(0));
    assert!(read.named.is_none());
}

#[test]
fn a_cooccur_weak_or_zero_confidence_row_is_never_cited() {
    let mut lattice = Lattice::new();
    let owned = row(&mut lattice, 10, 20, 0, 95, "it is red");
    let _weak = row(&mut lattice, 11, 21, 6, 95, "it is maybe");
    let _cold = row(&mut lattice, 12, 22, 0, 0, "it is cold");
    let resolve = Map::new(&[("apple", &[10]), ("pear", &[11]), ("plum", &[12])]);
    let params = params();
    let curve = curve(1);
    let config = config(&params, &curve, 1);
    let mut integrator = Integrator::at_birth();

    let read = retrieve(
        &lattice,
        lattice.interner(),
        &resolve,
        &mut integrator,
        &config,
        &request(&["apple", "pear", "plum"], None, 10),
    )
    .unwrap();
    assert_eq!(read.cited.len(), 1);
    assert_eq!(read.cited[0].row_id, owned);
    assert_eq!(read.cited[0].kind, 0);
}

#[test]
fn a_named_dark_slot_is_returned_only_when_it_was_lit() {
    let lattice = Lattice::new();
    let resolve = Map::new(&[("znark", &[30])]);
    let params = params();
    let curve = curve(1);
    let config = config(&params, &curve, 1);
    let mut integrator = Integrator::at_birth();

    let read = retrieve(
        &lattice,
        lattice.interner(),
        &resolve,
        &mut integrator,
        &config,
        &request(&["znark"], Some(30), 10),
    )
    .unwrap();
    assert!(read.cited.is_empty());
    assert_eq!(read.named, Some(SlotKey { frame: PatternId::PQuestA, point: 30 }));
    assert_eq!(read.thought.stop_reason, StopReason::MaxK);

    let mut integrator = Integrator::at_birth();
    let unmatched = retrieve(
        &lattice,
        lattice.interner(),
        &resolve,
        &mut integrator,
        &config,
        &request(&["znark"], Some(40), 10),
    )
    .unwrap();
    assert!(unmatched.named.is_none());
}

#[test]
fn an_unlit_key_is_a_novelty_and_the_walk_has_nothing_to_settle() {
    let lattice = Lattice::new();
    let resolve = Map::new(&[]);
    let params = params();
    let curve = curve(1);
    let config = config(&params, &curve, 1);
    let mut integrator = Integrator::at_birth();
    let read = retrieve(
        &lattice,
        lattice.interner(),
        &resolve,
        &mut integrator,
        &config,
        &request(&["znark"], None, 10),
    )
    .unwrap();
    assert_eq!(read.novel, vec![0]);
    assert_eq!(read.thought.stop_reason, StopReason::NothingLit);
    assert!(read.cited.is_empty());
    assert!(read.named.is_none());
}

#[test]
fn two_rows_on_one_slot_compose_and_count_two_owned_hops() {
    let mut lattice = Lattice::new();
    let first = row(&mut lattice, 10, 20, 0, 95, "it is red");
    let second = row(&mut lattice, 10, 21, 0, 95, "it is round");
    let resolve = Map::new(&[("apple", &[10])]);
    let params = params();
    let curve = curve(2);
    let config = config(&params, &curve, 1);
    let mut integrator = Integrator::at_birth();
    let read = retrieve(
        &lattice,
        lattice.interner(),
        &resolve,
        &mut integrator,
        &config,
        &request(&["apple"], Some(10), 10),
    )
    .unwrap();
    let mut ids: Vec<u32> = read.cited.iter().map(|row| row.row_id).collect();
    ids.sort_unstable();
    assert_eq!(ids, vec![first, second]);
    assert_eq!(read.route.owned, 2);
    assert_eq!(read.route.hops, 2);
    assert_eq!(read.route.base, 0);
}

#[test]
fn the_same_call_at_an_earlier_anchor_is_the_history_walk() {
    let mut lattice = Lattice::new();
    let written_at_five = row_at(&mut lattice, 10, 20, 0, 95, "it is red", 5);
    let resolve = Map::new(&[("apple", &[10])]);
    let params = params();
    let curve = curve(1);
    let config = config(&params, &curve, 1);

    let mut integrator = Integrator::at_birth();
    let before = retrieve(
        &lattice,
        lattice.interner(),
        &resolve,
        &mut integrator,
        &config,
        &request(&["apple"], Some(10), 4),
    )
    .unwrap();
    assert!(before.cited.is_empty(), "a row is not citable before it was written");

    let mut integrator = Integrator::at_birth();
    let after = retrieve(
        &lattice,
        lattice.interner(),
        &resolve,
        &mut integrator,
        &config,
        &request(&["apple"], Some(10), 10),
    )
    .unwrap();
    assert_eq!(after.cited[0].row_id, written_at_five);
}
