//! 07 §9 W.3's readout and 05 §11 G6(b.1)'s counting half: a thought that walks a
//! sealed row reports the hops it took there as BASE, beside the OWNED hops of the
//! live web, and the two counts are never merged into one number.
//!
//! The corpus here is the pinned offline fixture. Every count is a fixture count,
//! not a corpus measurement.

use std::path::{Path, PathBuf};

use si_affect::params::LagUnit;
use si_affect::{Integrator, Params};
use si_read::light::Resolve;
use si_read::{retrieve, ReadRequest};
use si_store::build::Builder;
use si_store::interner::fold_key;
use si_store::layers_frozen::{load, pin, Config, Frozen};
use si_store::store::Lattice;
use si_walk::{Curve, CurvePoint, OscillationRule, WalkConfig};

struct Corpus<'a> {
    frozen: &'a Frozen,
}

impl Resolve for Corpus<'_> {
    /// The genome's own index, in the same ascending order the sealed file lists
    /// it. A live point would be merged in by a CONFIG ENGINE session; this test
    /// holds only the corpus side, so the count it reports is the corpus's.
    fn aliases(&self, key: &str) -> Vec<u32> {
        self.frozen.points_for_folded(key)
    }
}

/// CONFIG SI's index before the mind has been taught anything: no member of the
/// alias set, because there is no alias set.
struct Nothing;

impl Resolve for Nothing {
    fn aliases(&self, _key: &str) -> Vec<u32> {
        Vec::new()
    }
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/genome/fixture")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("si-b10-read-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn layer(name: &str) -> (std::rc::Rc<Frozen>, PathBuf) {
    let dir = scratch(name);
    let built = Builder::new(&fixture_dir().join("manifest.json"), Some(&dir)).run().unwrap();
    let pinned = pin(built.artifact.clone(), built.tag);
    (std::rc::Rc::new(load(Config::Engine, Some(&pinned)).unwrap()), dir)
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

fn curve(reach: u16) -> Curve {
    Curve::fixture(CurvePoint { reach, decay_q12: 4096, settle_shift_q12: 0 })
}

fn config<'a>(params: &'a Params, curve: &'a Curve, k_max: u16) -> WalkConfig<'a> {
    WalkConfig {
        params,
        curve,
        k_max,
        tau_settle: 1,
        decay_base: 128,
        hub_exponent_q12: 4096,
        frontier_cap: 64,
        oscillation: OscillationRule::MaxKOnly,
        ghost_max: 0,
        shape_max: 0,
        tugs: &[],
        relight_max: 0,
        composition_hint: &[],
    }
}

/// A fixture word whose sealed row reaches a second point, so the thought has a
/// corpus hop to count.
fn walkable(frozen: &Frozen) -> (String, u32) {
    for point in 0..frozen.genome_points() {
        let surface = frozen.surface_of(point as usize).unwrap().to_string();
        for element in frozen.row(point) {
            if element.to_point != point {
                return (surface, point);
            }
        }
    }
    panic!("the fixture seals no walkable word");
}

#[test]
fn a_corpus_hop_is_counted_as_base_and_never_as_owned() {
    let (frozen, dir) = layer("base-route");
    let mut lattice = Lattice::new();
    lattice.attach_frozen(frozen.clone()).unwrap();
    let (surface, lit) = walkable(&frozen);
    assert!(lit < lattice.point_base(), "W.2: a genome handle is below the live base");

    let resolve = Corpus { frozen: &frozen };
    let params = params();
    let shape = curve(1);
    let config = config(&params, &shape, 2);
    let mut integrator = Integrator::at_birth();
    let read = retrieve(
        &lattice,
        lattice.interner(),
        &resolve,
        &mut integrator,
        &config,
        &ReadRequest {
            keys: vec![fold_key(&surface)],
            frame_point: None,
            slot: None,
            anchor: 1,
            thought_id: 1,
        },
    )
    .unwrap();

    assert!(read.route.base > 0, "the sealed row was walked but counted nothing");
    assert_eq!(read.route.owned, 0, "a corpus hop is not a taught edge");
    // 07 §9's two record shapes, at the one place the route is reported: the
    // corpus has no belief rows, so a BASE-only route has nothing to cite.
    assert!(
        read.cited.is_empty(),
        "a corpus thought cited live rows it never touched: {:?}",
        read.cited
    );
    for round in &read.thought.rounds {
        for contribution in &round.contributions {
            if contribution.genome {
                assert!(contribution.amount > 0);
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn with_no_layer_there_is_no_base_to_count() {
    let lattice = Lattice::new();
    let params = params();
    let shape = curve(1);
    let config = config(&params, &shape, 2);
    let mut integrator = Integrator::at_birth();
    let read = retrieve(
        &lattice,
        lattice.interner(),
        &Nothing,
        &mut integrator,
        &config,
        &ReadRequest {
            keys: vec![fold_key("water")],
            frame_point: None,
            slot: None,
            anchor: 1,
            thought_id: 1,
        },
    )
    .unwrap();
    assert_eq!(read.route.base, 0, "CONFIG SI counted a web it does not have");
    assert_eq!(read.route.owned, 0);
}
