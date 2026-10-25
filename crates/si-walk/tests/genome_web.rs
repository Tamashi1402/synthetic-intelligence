//! 07 §9 W.2–W.4: the merged web, over the pinned offline fixture. Every count
//! here is a fixture's count and is never quoted as a corpus property; what is
//! asserted is the merge's *shape* — cursor order, the two address namespaces,
//! the record shape that makes a stamped genome edge unrepresentable, and the
//! degree seed as a sum.
//!
//! The loader body is the only way to hold a real `Frozen`, so this file is the
//! reason `si-store`'s `engine` feature is a dev-dependency of this crate and
//! nothing more.

mod common;

use std::path::{Path, PathBuf};
use std::rc::Rc;

use si_affect::Integrator;
use si_core::genome::GENOME_KIND_FIRST;
use si_store::build::Builder;
use si_store::layers_frozen::{load, pin, Config, Frozen};
use si_store::store::Lattice;
use si_walk::{walk, Arc, Graph, Lit, LIGHT_QUANTUM};

use common::{add_row, config, flat_curve, params};

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/genome/fixture")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("si-b10-walk-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The fixture as a loaded layer: built once per test into its own scratch, so a
/// failure names the file it opened.
fn layer(name: &str) -> (Rc<Frozen>, PathBuf) {
    let dir = scratch(name);
    let built = Builder::new(&fixture_dir().join("manifest.json"), Some(&dir)).run().unwrap();
    let pinned = pin(built.artifact.clone(), built.tag);
    let frozen = load(Config::Engine, Some(&pinned)).unwrap();
    (Rc::new(frozen), dir)
}

/// The fixture's widest sealed row, as (point, genome points) — the point the
/// two-cursor run is worth testing on.
fn seeded_point(frozen: &Frozen) -> (u32, usize) {
    (0..frozen.genome_points())
        .map(|point| (point, frozen.row(point).len()))
        .filter(|(_, width)| *width >= 2)
        .max_by_key(|(_, width)| *width)
        .expect("the fixture seals a point with more than one outgoing relation")
}

/// W.3: one point, two cursors, live first. The order is not a courtesy — it is
/// 04 §5 step 6's ascending-offset tie-break resolving in OWNED's favor, and the
/// reason §16.2's weighting law needs no branch on a source class.
#[test]
fn the_live_list_is_walked_before_the_sealed_row() {
    let (frozen, dir) = layer("cursor");
    let (point, width) = seeded_point(&frozen);
    let mut lattice = Lattice::new();
    lattice.attach_frozen(frozen.clone()).unwrap();
    // An OWNED edge taught *from* a genome point: the one shape where both
    // cursors answer for the same handle.
    add_row(&mut lattice, point, &[(point + 1, 2048, 0, 128)]);

    let graph = Graph::build(&lattice, 0).unwrap();
    let arcs: Vec<Arc> = graph.arcs(point).collect();
    assert_eq!(arcs.len(), 1 + width, "W.3 is one iterator over both webs");
    assert!(!arcs[0].is_genome(), "the live cursor is first");
    assert_eq!(arcs[0].to_point, point + 1);

    let row = frozen.row(point);
    for (index, arc) in arcs[1..].iter().enumerate() {
        assert!(arc.is_genome(), "the sealed row follows in its own order");
        assert_eq!(arc.to_point, row[index].to_point);
        assert_eq!(arc.kind, row[index].edge_kind);
        assert_eq!(arc.weight, row[index].weight_q12());
        // W.2's second namespace: a genome arc carries the CSR index, not an
        // arena byte offset, and the two spaces overlap numerically.
        assert_eq!(arc.edge_off, frozen.row_start(point) + index as u32);
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// 07 §9's "two record shapes, and that is the enforcement": a genome arc is
/// built with no `bake`, no stamp and no charge list to read, so R-23's
/// "unrepresentable, not merely disallowed" is a field fact of this iterator.
#[test]
fn a_genome_arc_carries_no_bake_and_no_stamp() {
    let (frozen, dir) = layer("shape");
    let (point, _) = seeded_point(&frozen);
    let mut lattice = Lattice::new();
    lattice.attach_frozen(frozen.clone()).unwrap();
    let graph = Graph::build(&lattice, 0).unwrap();

    let arcs: Vec<Arc> = graph.arcs(point).collect();
    assert!(!arcs.is_empty());
    for arc in &arcs {
        assert!(arc.is_genome());
        assert!(arc.kind >= GENOME_KIND_FIRST, "01 §5.2: slots 0..=6 are OWNED-only");
        assert_eq!(arc.bake, 0, "a genome record has no bake column");
        assert_eq!(arc.a_at, 0, "a genome record has no stamp to read a level from");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// W.4: `deg` is a sum read at the point of use, never a stored number. The
/// hub-damp table would otherwise treat a corpus hub as a one-edge point, and a
/// damp that silently equals 1.0 is a damp that does nothing.
#[test]
fn the_degree_is_seeded_from_both_webs() {
    let (frozen, dir) = layer("degree");
    let (point, width) = seeded_point(&frozen);
    let mut lattice = Lattice::new();
    lattice.attach_frozen(frozen.clone()).unwrap();
    add_row(&mut lattice, point, &[(point + 1, 4096, 0, 128), (point + 2, 4096, 0, 128)]);

    let graph = Graph::build(&lattice, 0).unwrap();
    assert_eq!(graph.out_degree(point), 2 + width as u32);
    assert_eq!(graph.out_degree(point), lattice.out_degree(point) + frozen.degree_seed(point));
    assert!(
        graph.points().any(|lit| lit == point),
        "W.2: the genome owns the low handles, so the merged point set starts at 0"
    );
    assert_eq!(graph.points().min(), Some(0), "genome-first allocation order");
    let _ = std::fs::remove_dir_all(&dir);
}

/// G9(d) / G6(c): the affective term is read from a live edge's charge list by
/// arena offset. A CSR index aliases some live offset, so the walk must not read
/// bias for a genome arc — and the proof is that the genome half of the spread
/// does not move when the live web is given a bake that does move it.
#[test]
fn a_genome_arc_never_conducts() {
    let (frozen, dir) = layer("conduct");
    let (point, _) = seeded_point(&frozen);

    let run = |bake: u16| {
        let mut lattice = Lattice::new();
        lattice.attach_frozen(frozen.clone()).unwrap();
        add_row(&mut lattice, point, &[(point + 1, 4096, bake, 200)]);
        let graph = Graph::build(&lattice, 0).unwrap();
        let params = params();
        let curve = flat_curve(16, 4096);
        let mut integrator = Integrator::at_birth();
        walk(
            &graph,
            &[Lit { point, activation: LIGHT_QUANTUM }],
            &mut integrator,
            1,
            &[],
            &[],
            &config(&params, &curve, 1, 1),
        )
        .unwrap()
    };

    let cold = run(0);
    let hot = run(4096);
    // One round, because a round's own activations are the seed: from round 2 on
    // a genome arc's *source* may be hotter simply because an OWNED edge was
    // biased, and that is the walk behaving correctly, not a corpus edge earning
    // a reading. Round 0 separates the two claims.
    let amounts = |thought: &si_walk::Thought, genome: bool| -> Vec<i32> {
        thought
            .rounds
            .iter()
            .flat_map(|round| round.contributions.iter())
            .filter(|contribution| contribution.genome == genome)
            .map(|contribution| contribution.amount)
            .collect()
    };

    // The bake did something, or the comparison below proves nothing.
    assert!(
        hot.rounds.iter().any(|round| round.bias_total_q12 > 0),
        "the live edge conducted no bias at all, so this read is vacuous"
    );
    assert_ne!(
        amounts(&cold, false),
        amounts(&hot, false),
        "the OWNED half of the same walk did not respond to its own stamp"
    );
    assert!(
        !amounts(&hot, true).is_empty(),
        "the walk never reached the sealed row, so it merged nothing"
    );
    assert_eq!(
        amounts(&cold, true),
        amounts(&hot, true),
        "a genome edge earned an affective reading: R-23 is a schema fact, not a hope"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// CONFIG SI's twin of every claim above: with no layer attached the index is
/// the shipped one — no genome arc, no degree seed, and the walk's arithmetic
/// unchanged. This is the half that keeps 00 §6's "same code, different load"
/// honest at the crate boundary.
#[test]
fn with_no_layer_attached_the_index_is_the_one_si_ships() {
    let (frozen, dir) = layer("si-twin");
    let (point, width) = seeded_point(&frozen);
    let mut lattice = Lattice::new();
    add_row(&mut lattice, point, &[(point + 1, 4096, 0, 128)]);

    let graph = Graph::build(&lattice, 0).unwrap();
    let arcs: Vec<Arc> = graph.arcs(point).collect();
    assert_eq!(arcs.len(), 1, "no cursor opens on a genome that was never loaded");
    assert_eq!(graph.out_degree(point), 1);
    assert_eq!(frozen.degree_seed(point), width as u32, "the layer itself is unread here");
    assert_eq!(lattice.point_base(), 0, "W.2's base is a CONFIG ENGINE fact");
    let _ = std::fs::remove_dir_all(&dir);
}

/// W.2's handle space: a live arrival allocates above the genome, and a genome
/// handle is never a live point table entry — the two index spaces are one point
/// set, not one table.
#[test]
fn a_live_arrival_allocates_above_the_sealed_points() {
    let (frozen, dir) = layer("alloc");
    let genome_points = frozen.genome_points();
    let mut lattice = Lattice::new();
    lattice.attach_frozen(frozen.clone()).unwrap();
    assert_eq!(lattice.point_base(), genome_points);

    let first = lattice.points_mut().ensure_term("water", "water");
    let second = lattice.points_mut().ensure_term("fire", "fire");
    assert_eq!(first, genome_points, "live points allocate from P_gen upward");
    assert_eq!(second, genome_points + 1);
    assert!(lattice.points().get(genome_points - 1).is_none(), "a genome handle is not a live row");
    assert_eq!(lattice.points().get(first).unwrap().surface.as_deref(), Some("water"));

    let again = lattice.points_mut().ensure_term("water", "water");
    assert_eq!(again, first, "identity is dedup, not similarity (01 §1)");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A second layer on one lattice is a fork in the point space, and 07 §13's tag
/// refusal is the only other thing that could catch it.
#[test]
fn one_lattice_holds_one_frozen_layer() {
    let (frozen, dir) = layer("double");
    let mut lattice = Lattice::new();
    lattice.attach_frozen(frozen.clone()).unwrap();
    let err = lattice.attach_frozen(frozen).unwrap_err().to_string();
    assert!(err.contains("already attached"), "the second attach said: {err}");
    let _ = std::fs::remove_dir_all(&dir);
}
