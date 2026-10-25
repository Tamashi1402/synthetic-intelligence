//! 04 §25 step 2's W.0→W.4 end to end, over the pinned offline fixture: an ENGINE
//! wake owns the genome's handle space before its first thought, the sealed tag is
//! recorded and refused, and the corpus is reachable by the mind's own tokenizer.
//!
//! Every count here is a fixture count. Nothing in this file is a corpus
//! measurement, and no state hash it prints is a claim about a shipped genome.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use si_core::seed;
use si_core::strings::Source;
use si_session::{OpenOutcome, Session};
use si_store::build::Builder;
use si_store::layers_frozen::{load, pin, Config, Frozen};
use si_store::snapshot::{ResumeRefusal, Snapshot};

const SEED: u64 = 0x51;

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/genome/fixture")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("si-b10-session-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn layer(name: &str) -> (Rc<Frozen>, PathBuf) {
    let dir = scratch(name);
    let built = Builder::new(&fixture_dir().join("manifest.json"), Some(&dir)).run().unwrap();
    let pinned = pin(built.artifact.clone(), built.tag);
    let frozen = load(Config::Engine, Some(&pinned)).unwrap();
    (Rc::new(frozen), dir)
}

fn write(path: &Path, bytes: &[u8]) -> PathBuf {
    std::fs::write(path, bytes).unwrap();
    path.to_path_buf()
}

/// A fixture word that lights a corpus point whose sealed row reaches a second
/// corpus point the word does not itself light: the smallest input that can tell
/// the two-cursor run apart from a lighting-only settle.
fn walkable_word(frozen: &Frozen) -> (String, u32, u32) {
    for point in 0..frozen.genome_points() {
        let spoken = frozen.surface_of(point as usize).unwrap().to_string();
        let lit = frozen.points_for(&spoken);
        for element in frozen.row(point) {
            if element.to_point != point && !lit.contains(&element.to_point) {
                return (spoken, point, element.to_point);
            }
        }
    }
    panic!("the fixture seals no walkable word");
}

/// W.0 still runs first, and W.2 is the whole of the difference: the same
/// derive-and-compare birth, on a lattice whose low handles belong to the corpus.
#[test]
fn an_engine_wake_allocates_the_genome_first() {
    let (frozen, dir) = layer("wake");
    let session = Session::wake_engine(SEED, frozen.clone()).unwrap();

    assert_eq!(session.seed(), SEED);
    assert_eq!(
        session.core().temperament_digest,
        seed::temperament_digest(SEED),
        "W.0 is not a load: the vector is re-derived in ENGINE exactly as in SI"
    );
    assert_eq!(session.store().point_base(), frozen.genome_points(), "W.2's base");
    assert_eq!(
        session.store().frozen().map(|layer| layer.version_tag()),
        Some(frozen.version_tag())
    );

    // CONFIG SI's own wake is untouched by the same code path: base 0, no layer,
    // and the same individual — the genome is a load, not a seed.
    let plain = Session::new(SEED).unwrap();
    assert_eq!(plain.store().point_base(), 0);
    assert!(plain.store().frozen().is_none());
    assert_eq!(plain.fingerprint(), session.fingerprint());
    let _ = std::fs::remove_dir_all(&dir);
}

/// 07 §13: the tag is compared before a single row is decoded. Resuming under a
/// foreign corpus would let one mind's rows claim another's history, which is the
/// same branch 04 §25 step 5 draws for a seed.
#[test]
fn a_resume_compares_the_genome_it_was_born_on() {
    let (frozen, dir) = layer("resume");
    let path = dir.join("SIst");
    let mut session = Session::wake_engine(SEED, frozen.clone()).unwrap();
    session.tick("water", Source::Owner).unwrap();
    session.save(&path).unwrap();
    let before = session.state_hash().unwrap();

    // The tag the snapshot carries is the tag the layer seals, written by the
    // capture rather than remembered by a caller.
    let header = Snapshot::read_header(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(header.geometry.genome_t0_tag, frozen.version_tag(), "01 §10 item 8");

    match Session::open_engine(&path, SEED, Some(frozen.clone())).unwrap() {
        OpenOutcome::Opened(resumed) => {
            assert_eq!(resumed.store().point_base(), frozen.genome_points());
            assert_eq!(resumed.state_hash().unwrap().digest(), before.digest());
        }
        OpenOutcome::Refused(refusal) => panic!("a same-genome resume refused: {refusal}"),
    }

    // A session that recorded a different tag is refused from the header, before
    // the payload decodes. A forged header is the cheapest way to hold both halves
    // of that claim without a second corpus.
    let mut forged = Snapshot::decode(&std::fs::read(&path).unwrap()).unwrap();
    let sealed = frozen.version_tag();
    forged.header.geometry.genome_t0_tag = sealed ^ 0x01;
    let forged_path = write(&dir.join("forged"), &forged.encode().unwrap());
    match Session::open_engine(&forged_path, SEED, Some(frozen.clone())).unwrap() {
        OpenOutcome::Refused(ResumeRefusal::GenomeTag { stored, pinned }) => {
            assert_eq!((stored, pinned), (sealed ^ 0x01, sealed));
        }
        OpenOutcome::Refused(refusal) => {
            panic!("a foreign genome was refused for the wrong reason: {refusal}")
        }
        OpenOutcome::Opened(_) => panic!("a foreign genome was resumed anyway"),
    }

    // CONFIG SI never asks: with no layer there is nothing to compare, and the
    // shipped resume path is unchanged.
    assert!(matches!(Session::open(&path, SEED).unwrap(), OpenOutcome::Opened(_)));
    let _ = std::fs::remove_dir_all(&dir);
}

/// G6.a.1's engine half at the smallest scale the build can reach: the mind
/// thinks, and the sealed bytes it reads are the bytes it started with. The layer
/// is opened read-only and never written, so this is a byte comparison and not a
/// policy hope.
#[test]
fn thinking_never_mutates_the_frozen_layer() {
    let (frozen, dir) = layer("integrity");
    let before = frozen.file.encode();
    let mut session = Session::wake_engine(SEED, frozen.clone()).unwrap();
    for line in ["water", "is water wet", "fire", "what is water", "water"] {
        session.tick(line, Source::Owner).unwrap();
    }
    assert_eq!(frozen.file.encode(), before, "L1/L2 moved under a live mind");
    assert_eq!(
        frozen.version_tag(),
        si_core::genome::GenomeHeader::tag_u64(&frozen.header.seal_md5).unwrap(),
        "the loaded layer still answers to its own seal"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// 04 §4's alias law is the corpus's way in: a token whose folded key the sealed
/// surface index speaks lights the genome's point as another member of the same
/// set, and the walk then reads that point's sealed row. No second reader, no
/// second walk, no branch on a source class.
#[test]
fn a_word_the_corpus_speaks_lights_a_genome_point() {
    let (frozen, dir) = layer("light");
    let (spoken, source, reached) = walkable_word(&frozen);

    let mut engine = Session::wake_engine(SEED, frozen.clone()).unwrap();
    engine.tick(&format!("what is {spoken}"), Source::Owner).unwrap();
    let settle = si_session::last_settle(&engine);
    assert!(settle.contains(&source), "{spoken} lights no corpus point: {settle:?}");
    assert!(
        settle.contains(&reached),
        "the sealed row of {spoken} was never walked: {settle:?}"
    );
    assert!(
        reached < engine.store().point_base(),
        "W.2: the reached handle is not a genome's"
    );

    // Non-vacuity: `reached` is not one of the points the word itself speaks, so
    // nothing but the two-cursor run of 07 §9 W.3 could have settled on it.
    assert!(!frozen.points_for(&spoken).contains(&reached));

    // The same input in CONFIG SI walks a web with no corpus in it, so it settles
    // on a different set of handles.
    let mut plain = Session::new(SEED).unwrap();
    plain.tick(&format!("what is {spoken}"), Source::Owner).unwrap();
    assert_eq!(plain.store().point_base(), 0);
    assert_ne!(
        si_session::last_settle(&plain), settle,
        "the layer is attached but inert: the two minds settled identically"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// W.1's first half again, from the session side: an ENGINE wake with a layer the
/// lock does not pin is a refusal the CLI cannot walk past, and the resume that
/// follows it is the same individual's.
#[test]
fn the_same_seed_wakes_a_different_mind_under_a_different_tag() {
    let (frozen, dir) = layer("twins");
    let path = dir.join("SIst");
    let mut session = Session::wake_engine(SEED, frozen.clone()).unwrap();
    session.tick("fire", Source::Owner).unwrap();
    session.save(&path).unwrap();

    let mut other = Session::new(SEED).unwrap();
    other.tick("fire", Source::Owner).unwrap();
    assert_ne!(
        other.state_hash().unwrap().digest(),
        session.state_hash().unwrap().digest(),
        "an ENGINE state hashes exactly like the SI one: the genome reached nothing"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
