//! 07 §2–§13 / engineering/11 §B10: the converter over the pinned offline
//! fixture. Every count here is a fixture's count and is never quoted as a corpus
//! measurement; what is asserted is structure — order, seals, refuses and the
//! A-asserts — not volume.
//!
//! The fixture lives at `data/genome/fixture/`, its three sources are pinned by
//! sha256 in its manifest, and its tier table is authored, not computed (07 §8).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use si_core::sha256::sha256_hex;
use si_store::layers_frozen::{load, Config, ConfigRefusal, LoadError};
#[cfg(feature = "engine")]
use si_store::layers_frozen::pin;
use si_store::opens;

/// The open register is one process-wide log, and cargo runs these tests on parallel
/// threads: a test that turns tracing on records **every** load happening in the
/// process, not only its own. So every test that opens a frozen layer takes this lock,
/// not only the two that flip `SI_TRACE_OPENS` — measured: with three loaders unlocked,
/// `the_loader_body_records_its_one_frozen_open` saw its neighbour's artifact in its own
/// trace and read `2` where the law says `1` (1 failed / 23 passed in 1 of 15 runs).
static TRACE_LOCK: Mutex<()> = Mutex::new(());

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/genome/fixture")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("si-b10-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 07 §M.1, cross-checked against a second implementation: the pins in the
/// manifest were computed with `hashlib.sha256`, so a match here is a match
/// between this crate's pure-integer SHA-256 and an independent one.
#[test]
fn the_pinned_digests_agree_with_an_independent_implementation() {
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fixture_dir().join("manifest.json")).unwrap())
            .unwrap();
    for source in manifest["sources"].as_array().unwrap() {
        let name = source["path"].as_str().unwrap();
        let bytes = std::fs::read(fixture_dir().join(name)).unwrap();
        assert_eq!(
            sha256_hex(&bytes),
            source["sha256"].as_str().unwrap(),
            "the pin of {name} is what a second implementation computes for it"
        );
    }
}

/// 00 §6.1: the runtime guard. The refusal is the first statement of `load`, so
/// no path — not even a relative one — is formed, and the open register holds
/// nothing that the loader could have named.
#[test]
fn config_si_refuses_before_a_frozen_path_is_reachable() {
    let _serialise = TRACE_LOCK.lock();
    let dir = scratch("si-refusal");
    let probe = dir.join("never-named");
    let probe_text = probe.display().to_string();
    let previous = std::env::var_os("SI_TRACE_OPENS");
    std::env::set_var("SI_TRACE_OPENS", "1");
    let outcome = load(Config::Si, None);
    let trace = opens::trace();
    match previous {
        Some(value) => std::env::set_var("SI_TRACE_OPENS", value),
        None => std::env::remove_var("SI_TRACE_OPENS"),
    }
    assert_eq!(
        outcome.unwrap_err(),
        LoadError::Refused(ConfigRefusal::SiHasNoFrozenLayer)
    );
    assert!(opens::frozen_opens(&probe_text).is_empty());
    assert!(
        !trace
            .iter()
            .any(|line| line.split_once(' ').map(|(_, path)| path) == Some(probe_text.as_str())),
        "a refused CONFIG SI run named the path it was never allowed to form"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// 00 §6.2: the build guard, read on a binary that carries no loader body.
#[cfg(not(feature = "engine"))]
#[test]
fn a_build_without_the_feature_refuses_before_a_handle_exists() {
    assert_eq!(
        load(Config::Engine, None).unwrap_err(),
        LoadError::Refused(ConfigRefusal::EngineFeatureAbsent)
    );
}

/// Everything the pipeline tests need, from one build of the fixture.
#[cfg(feature = "engine")]
mod pipeline {
    use super::*;
    use si_core::genome::{
        hex_bytes, DictRow, PointRow, DICT_BYTES, REL_BYTES, RULE_BYTES,
        GENOME_KIND_FIRST,
    };
    use si_store::build::{a11_violations, Built, Manifest, Builder};
    use si_store::genome_file::{check_seal, GenomeFile};

    struct Run {
        built: Built,
        dir: PathBuf,
    }

    /// One fixture build into a caller-named scratch directory.
    fn build(name: &str) -> Run {
        let dir = scratch(name);
        let manifest = fixture_dir().join("manifest.json");
        let built = Builder::new(&manifest, Some(&dir)).run().unwrap();
        assert_eq!(built.directory.parent().unwrap(), dir.as_path());
        Run { built, dir }
    }

    /// The artifact as it sits on disk: the bytes a reader of the seal gets.
    fn shipped(run: &Run) -> Vec<u8> {
        std::fs::read(&run.built.artifact).unwrap()
    }

    #[test]
    fn a_fixture_build_seals_and_passes_every_structural_assert() {
        let run = build("green");
        assert_eq!(
            run.built.asserts,
            vec!["A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9", "A10", "A11"],
            "07 §12's Group A is the whole list the build may claim"
        );
        assert!(run.built.points > 0 && run.built.edges > 0 && run.built.rows > 0);
        assert_eq!(run.built.batches, 3);
        for path in &run.built.files {
            assert!(path.exists(), "{} was published and then vanished", path.display());
        }
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    #[test]
    fn the_canonical_order_is_the_verbatim_record_bytes() {
        let run = build("sorted");
        let file = &run.built.file;
        // §4: compared as an unsigned lexicographic byte string — never by lemma,
        // never by a locale collation.
        for pair in file.dict.windows(2) {
            assert!(pair[0].encode() <= pair[1].encode(), "the DICT run is not byte-sorted");
        }
        for pair in file.rel.windows(2) {
            assert!(pair[0].encode() <= pair[1].encode(), "the REL run is not byte-sorted");
        }
        for pair in file.rule.windows(2) {
            assert!(pair[0].encode() <= pair[1].encode(), "the RULE run is not byte-sorted");
        }
        // The hashed row file is its records and nothing else: no framing, no
        // section lengths, so a run cannot hide a second encoding inside itself.
        assert_eq!(
            file.rows_bytes().len(),
            file.dict.len() * DICT_BYTES + file.rel.len() * REL_BYTES + file.rule.len() * RULE_BYTES
        );
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    #[test]
    fn the_same_manifest_reproduces_the_same_tag_byte_for_byte() {
        let first = build("idem-1");
        let second = build("idem-2");
        assert_eq!(first.built.tag, second.built.tag, "07 §3's idempotence law");
        assert_eq!(first.built.seal.seal_md5, second.built.seal.seal_md5);
        assert_eq!(shipped(&first), shipped(&second), "the published artifact is the same file");
        let _ = std::fs::remove_dir_all(&first.dir);
        let _ = std::fs::remove_dir_all(&second.dir);
    }

    #[test]
    fn a_byte_changed_after_sealing_fails_a10() {
        let run = build("mutated");
        let mut bytes = shipped(&run);
        let header_len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        let payload = 12 + header_len;
        // Past the section count and the DICT run's length word, so this is a
        // record byte inside the hashed region: the seal itself is what refuses
        // (07 §13: an edited file is not loaded).
        bytes[payload + 8 + 1] ^= 0x01;
        let err = check_seal(&bytes).unwrap_err().to_string();
        assert!(err.contains("A10"), "the seal check said {err}");
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    #[test]
    fn a_reserved_byte_set_is_refused_by_the_record_decoder() {
        let run = build("reserved");
        let mut canonical = run.built.file.encode();
        // The file opens with its section count and then the DICT run's length,
        // so the reserved tail of the first record is four bytes before 36.
        canonical[4 + 4 + DICT_BYTES - 1] = 9;
        let err = GenomeFile::decode(&canonical).unwrap_err().to_string();
        assert!(err.contains("reserved"), "the decoder said {err}");
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    #[test]
    fn the_walkable_edges_are_genome_slots_and_their_names_are_the_sources_own() {
        let run = build("kinds");
        let file = &run.built.file;
        for row in &file.rel {
            assert!(row.edge_kind >= GENOME_KIND_FIRST, "a corpus row holds an OWNED kind");
            let name = file.kind_name(row.edge_kind).unwrap();
            assert!(
                ["IsA", "PartOf", "CapableOf", "MotivatedByGoal", "has-property", "is-a"]
                    .contains(&name)
            );
        }
        // §6: the table lists names and maps none, so two shapes a human would
        // call one idea hold two slots.
        let slots: Vec<&str> =
            (GENOME_KIND_FIRST..).map_while(|slot| file.kind_name(slot)).collect();
        assert_eq!(slots.len(), file.kind_ids.len());
        assert!(slots.contains(&"IsA") && slots.contains(&"is-a"), "07 §6: no grouping");
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    #[test]
    fn an_unmatched_endpoint_takes_its_own_row_in_its_own_batch() {
        let run = build("dropped");
        let file = &run.built.file;
        let salt = (0..file.points.len())
            .find(|point| file.surface_of(*point).unwrap() == "salt")
            .expect("07 §5: no silent drop — the endpoint is a point");
        assert_eq!(file.dict[salt].prov_batch, 1, "the edges batch is the second source");
        assert!(file.rel.iter().any(|rel| rel.a_point as usize == salt));
        assert_eq!(run.built.unmatched_points, 1, "only the fixture's one unmatched surface");
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    #[test]
    fn a_weight_tier_is_the_manifest_step_table_and_never_a_computed_number() {
        let run = build("tiers");
        let file = &run.built.file;
        let tier_of = |name: &str| -> u8 {
            let slot = (GENOME_KIND_FIRST..)
                .find(|slot| file.kind_name(*slot) == Some(name))
                .unwrap();
            file.rel.iter().find(|row| row.edge_kind == slot).unwrap().weight
        };
        // The fixture's own authored buckets: 1..33 gives 40, 34..66 gives 82,
        // 67..100 gives 125, and a row with no stated weight takes the top tier.
        assert_eq!(tier_of("PartOf"), 125); // stated weight 92, the top bucket
        assert_eq!(tier_of("MotivatedByGoal"), 40); // stated weight 12, the bottom bucket
        assert_eq!(tier_of("is-a"), 82); // stated weight 45, the middle bucket
        assert_eq!(tier_of("CapableOf"), 125); // stated weight 70, past the middle bucket
        assert_eq!(tier_of("IsA"), 125); // no weight: the top tier
        assert_eq!(tier_of("has-property"), 125); // no weight: the top tier
        // §8: the conversion is a shift, so 125/255 becomes 2000 in Q12.
        assert_eq!(file.rel.iter().find(|row| row.weight == 125).unwrap().weight_q12(), 2000);
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    #[test]
    fn rules_are_sealed_and_citable_but_hold_no_edge() {
        let run = build("rules");
        let file = &run.built.file;
        assert_eq!(file.rule.len(), 2, "the two paradigm tables of the grammar source");
        // A parked rule is citable by its shelf-3 address and composes to depth 1
        // (07 §7); it is not in the walkable web, so a rule row carries no point
        // and no kind slot, only interned tokens.
        for row in &file.rule {
            assert!(!file.triggers_of(row).unwrap().is_empty());
            assert!(!file.results_of(row).unwrap().is_empty());
            assert_eq!(row.prov_batch, 2);
        }
        // The paradigm tables are acyclic by construction, which is what lets A9
        // be a load-time property rather than a repair.
        let first = file.rule[0];
        let second = file.rule[1];
        assert_ne!(first.triggers_off, second.triggers_off);
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    #[test]
    fn the_converter_opens_no_lattice_run_or_gate_path() {
        let _serialise = TRACE_LOCK.lock();
        let previous = std::env::var_os("SI_TRACE_OPENS");
        std::env::set_var("SI_TRACE_OPENS", "1");
        let run = build("handles");
        let trace = opens::trace();
        match previous {
            Some(value) => std::env::set_var("SI_TRACE_OPENS", value),
            None => std::env::remove_var("SI_TRACE_OPENS"),
        }
        assert!(!trace.is_empty(), "the register recorded nothing, so A11 would be vacuous");
        assert!(a11_violations(&trace).is_empty(), "A11: the build opened {}", trace.join(", "));
        // The register holds the manifest, the three pinned copies and the staged
        // writes of M.11 — all under the fixture or under this build's own scratch.
        for line in &trace {
            let path = line.split_once(' ').map(|(_, rest)| rest).unwrap_or(line);
            let lower = path.to_ascii_lowercase();
            assert!(lower.contains("fixture") || lower.contains("si-b10-"), "A11: {path}");
        }
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    #[test]
    fn a_source_that_moved_is_refused_before_a_row_is_parsed() {
        let dir = scratch("moved-source");
        for name in ["senses.tsv", "edges.tsv", "grammar.tsv", "manifest.json"] {
            std::fs::copy(fixture_dir().join(name), dir.join(name)).unwrap();
        }
        let mut bytes = std::fs::read(dir.join("senses.tsv")).unwrap();
        let last = bytes.len() - 2;
        bytes[last] = b'x';
        std::fs::write(dir.join("senses.tsv"), &bytes).unwrap();
        let err = Builder::new(&dir.join("manifest.json"), None)
            .run()
            .unwrap_err()
            .to_string();
        assert!(err.contains("hash mismatch"), "M.1 said {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_manifest_with_an_empty_license_or_a_live_shelf_is_a_refusal() {
        let dir = scratch("bad-manifest");
        for name in ["senses.tsv", "edges.tsv", "grammar.tsv"] {
            std::fs::copy(fixture_dir().join(name), dir.join(name)).unwrap();
        }
        let text = std::fs::read_to_string(fixture_dir().join("manifest.json")).unwrap();
        // An empty license field (07 §10, A5).
        let stripped = text.replace("CC0-1.0 (fixture authored for this build)", " ");
        std::fs::write(dir.join("manifest.json"), stripped).unwrap();
        let err = Manifest::load(&dir.join("manifest.json")).unwrap_err().to_string();
        assert!(err.contains("license"), "M.0 said {err}");
        // A source aimed at a live shelf (E2).
        let live = text.replace("\"target_shelf\": 3", "\"target_shelf\": 4");
        assert_ne!(live, text);
        std::fs::write(dir.join("manifest.json"), live).unwrap();
        let err = Manifest::load(&dir.join("manifest.json")).unwrap_err().to_string();
        assert!(err.contains("frozen shelves"), "M.0 said {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_line_the_adapter_cannot_parse_ends_the_build() {
        let dir = scratch("unparseable");
        for name in ["edges.tsv", "grammar.tsv", "manifest.json"] {
            std::fs::copy(fixture_dir().join(name), dir.join(name)).unwrap();
        }
        // A grammar-source line inside the synset source: legal dialect, wrong
        // adapter, so M.2 stops rather than counting a warning.
        let mut text = std::fs::read_to_string(fixture_dir().join("senses.tsv")).unwrap();
        text.push_str("Q\tn\theat\t1\tIsA\tn\tice\t1\t-\n");
        std::fs::write(dir.join("senses.tsv"), &text).unwrap();
        let manifest = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
        let repinned = manifest.replace(&fixture_pin(), &sha256_hex(text.as_bytes()));
        assert_ne!(repinned, manifest);
        std::fs::write(dir.join("manifest.json"), repinned).unwrap();
        let err = Builder::new(&dir.join("manifest.json"), None)
            .run()
            .unwrap_err()
            .to_string();
        assert!(err.contains("not a record the"), "M.2 said {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn fixture_pin() -> String {
        let text = std::fs::read_to_string(fixture_dir().join("manifest.json")).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        value["sources"][0]["sha256"].as_str().unwrap().to_string()
    }

    #[test]
    fn republishing_identical_bytes_at_the_same_tag_is_a_no_op() {
        let run = build("republish");
        let bytes = shipped(&run);
        let again = Builder::new(&fixture_dir().join("manifest.json"), Some(&run.dir))
            .run()
            .unwrap();
        assert_eq!(again.tag, run.built.tag);
        assert_eq!(std::fs::read(&again.artifact).unwrap(), bytes);
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    #[test]
    fn the_sealed_header_carries_the_tag_and_the_rows_carry_the_source_pin() {
        let run = build("header");
        let (header, provenance, file) = check_seal(&shipped(&run)).unwrap();
        assert_eq!(header.version_tag, run.built.tag);
        // 07 §13: a resume compares the tag it recorded against the artifact's
        // own, so the number a state hash item needs is readable from the header.
        assert_eq!(header.version_tag, si_core::genome::GenomeHeader::tag_u64(&header.seal_md5).unwrap());
        assert_eq!(provenance.len(), 3);
        assert_eq!(file.dict.len(), header.dict_rows as usize);
        // The per-row version tag is the source's pin, not the seal (07 §4).
        let first: DictRow = file.dict[0];
        let pinned = hex_bytes(&provenance[usize::from(first.prov_batch)].sha256).unwrap();
        assert_eq!(first.version_tag, u32::from_le_bytes(pinned[0..4].try_into().unwrap()));
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    #[test]
    fn every_point_row_speaks_the_dictionary_row_at_its_own_address() {
        let run = build("parallel");
        let file = &run.built.file;
        assert_eq!(file.points.len(), file.dict.len());
        for (index, point) in file.points.iter().enumerate() {
            let want = PointRow {
                hyperpoint_id: file.dict[index].hyperpoint_id,
                dict_row: index as u32,
                ..Default::default()
            };
            let got =
                PointRow { hyperpoint_id: point.hyperpoint_id, dict_row: point.dict_row, ..Default::default() };
            assert_eq!(got, want, "07 §11: a point and its dictionary row are one row");
            assert_eq!(file.surface_of(index).unwrap(), file.strings.get(file.dict[index].lemma).unwrap());
        }
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    #[test]
    fn a_form_a_source_lists_is_reachable_from_its_own_folded_key() {
        let run = build("forms");
        let file = &run.built.file;
        let mut listed_variant = 0;
        for (index, row) in file.dict.iter().enumerate() {
            let forms = file.forms_of(row).unwrap();
            if forms.len() > 1 {
                listed_variant += 1;
            }
            for form in forms {
                let folded = si_store::interner::fold_key(form);
                assert!(
                    file.points_for_folded(folded.as_bytes()).contains(&(index as u32)),
                    "A8: {form} folds to a key that reaches nothing"
                );
            }
        }
        assert_ne!(listed_variant, 0, "the fixture lists a form beside the surface it varies");
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    /// 07 §9 W.1–W.4: the loader body over the artifact the same build published.
    /// What it hands back is the sealed file, not a merged copy of it — the row it
    /// answers with is the CSR's own, and the degree it seeds with is its length.
    #[test]
    fn the_loader_body_answers_w1_to_w4_over_the_published_artifact() {
        let _serialise = super::TRACE_LOCK.lock();
        let run = build("loader");
        let pinned = pin(run.built.artifact.clone(), run.built.tag);
        let frozen = load(Config::Engine, Some(&pinned)).unwrap();
        assert_eq!(frozen.version_tag(), run.built.tag, "W.1: the tag the lock pins");
        assert_eq!(frozen.genome_points(), run.built.points, "W.2: the handle space it owns");
        assert_eq!(frozen.header.dict_rows as usize, run.built.file.dict.len());
        assert_eq!(frozen.provenance.len(), run.built.batches as usize, "§10's batch table");
        assert_eq!(frozen.header.edges as usize, frozen.file.csr.len());
        let mut counted = 0u32;
        for point in 0..frozen.genome_points() {
            let row = frozen.row(point);
            assert_eq!(row.len() as u32, frozen.degree_seed(point), "W.3 and W.4 are one read");
            counted += row.len() as u32;
            for element in row {
                assert!((element.to_point) < frozen.genome_points(), "A1 at the load");
            }
        }
        assert_eq!(counted, frozen.header.edges, "every genome edge is in exactly one row");
        assert_eq!(frozen.max_degree(), run.built.max_degree);
        assert!(
            frozen.max_degree() <= frozen.header.degree_table_range,
            "A7 read back at the load, from the sealed header"
        );
        // §6's citation table and §5's retrieval space, through the loaded layer.
        let slots: Vec<&str> =
            (GENOME_KIND_FIRST..).map_while(|slot| frozen.kind_name(slot)).collect();
        let mut sorted = slots.clone();
        sorted.sort();
        assert_eq!(slots, sorted, "§6's table is sealed in sorted order");
        assert!(
            slots.contains(&"IsA") && slots.contains(&"is-a"),
            "§6 lists names and maps none: {slots:?}"
        );
        let water = (0..run.built.file.points.len())
            .find(|point| run.built.file.surface_of(*point).unwrap() == "water")
            .expect("the fixture has a water point");
        assert!(
            frozen.points_for("Water").contains(&(water as u32)),
            "§5's retrieval space: the loaded fold reaches the point it sealed"
        );
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    /// 07 §13: the header's tag is compared to the lock before the layer is used.
    /// A session resuming under a foreign genome is a different mind's history.
    #[test]
    fn a_pinned_tag_that_is_not_the_seal_refuses_the_load() {
        let _serialise = super::TRACE_LOCK.lock();
        let run = build("foreign");
        let pinned = pin(run.built.artifact.clone(), run.built.tag ^ 0x01);
        assert_eq!(
            load(Config::Engine, Some(&pinned)).unwrap_err(),
            LoadError::TagMismatch { pinned: run.built.tag ^ 0x01, sealed: run.built.tag }
        );
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    /// 07 §1: the manifest lock is the only source list. ENGINE with nothing
    /// pinned is a refusal, not a search — and it forms no path, so the open
    /// register stays exactly as full as it was.
    #[test]
    fn an_engine_load_with_nothing_pinned_refuses_without_naming_a_path() {
        let _serialise = super::TRACE_LOCK.lock();
        std::env::set_var("SI_TRACE_OPENS", "1");
        opens::clear();
        let outcome = load(Config::Engine, None);
        let trace = opens::trace();
        assert_eq!(outcome.unwrap_err(), LoadError::Refused(ConfigRefusal::NothingPinned));
        assert!(trace.is_empty(), "a refusal with no pin named a path anyway: {trace:?}");
        std::env::remove_var("SI_TRACE_OPENS");
    }

    /// G5c's positive half, on the same register its negative half reads: the
    /// engine body opens the one artifact its lock pins, and nothing else.
    #[test]
    fn the_loader_body_records_its_one_frozen_open() {
        let _serialise = super::TRACE_LOCK.lock();
        let run = build("opens");
        let pinned = pin(run.built.artifact.clone(), run.built.tag);
        let text = run.built.artifact.display().to_string();
        std::env::set_var("SI_TRACE_OPENS", "1");
        opens::clear();
        let frozen = load(Config::Engine, Some(&pinned)).unwrap();
        let trace = opens::trace();
        std::env::remove_var("SI_TRACE_OPENS");
        assert_eq!(trace.len(), 1, "the load opened {trace:?}");
        assert_eq!(opens::frozen_opens(&text).len(), 1, "one read-only handle, in the open mode");
        assert_eq!(frozen.version_tag(), run.built.tag);
        let _ = std::fs::remove_dir_all(&run.dir);
    }

    /// A file that is not the sealed artifact — absent, short, or edited after
    /// sealing — is a load refusal with a message, never a walk over a partial
    /// genome (§13: an edited file is not loaded).
    #[test]
    fn an_artifact_that_is_not_the_sealed_file_refuses_the_load() {
        let run = build("broken");
        let dir = run.dir.join("absent");
        std::fs::create_dir_all(&dir).unwrap();
        let missing = pin(dir.join("nope"), run.built.tag);
        let err = load(Config::Engine, Some(&missing)).unwrap_err();
        assert!(matches!(err, LoadError::Artifact(_)), "{err}");
        assert!(err.to_string().contains("nope"), "the refusal does not say which file: {err}");

        let mut bytes = shipped(&run);
        let flip = 12 + u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize + 12;
        bytes[flip] ^= 0x40;
        let edited = run.dir.join("edited");
        std::fs::create_dir_all(&edited).unwrap();
        let target = edited.join("edited-artifact");
        std::fs::write(&target, &bytes).unwrap();
        let err = load(Config::Engine, Some(&pin(target, run.built.tag))).unwrap_err();
        assert!(err.to_string().contains("A10"), "the edited file refused as: {err}");
        let _ = std::fs::remove_dir_all(&run.dir);
    }
}

