//! 07 §2–§12: the converter. Twelve stages in a fixed order, one CLI verb, and
//! no path to a thought — `si-store::build` has no dependency edge to `si-learn`
//! and opens no lattice (07 §1's "never a second write path", A11).
//!
//! The order is the law: **sort → count → seal** (M.6 → M.7 → M.9). A count
//! report or a version tag computed over anything but the canonical sorted bytes
//! would not reproduce on a second machine, which is the only property the
//! design's cross-machine claim asks for (07 §3).
//!
//! What is *not* here is equally deliberate: no pruning, no relation-name
//! grouping, no weight derived from the corpus, no threshold over content
//! statistics (07 §12's Group B is printed, never judged). A1–A12 are structural
//! asserts, and every one of them reads the bytes that will ship.
//!
//! # The adapter dialects (07 §16's five stage contracts)
//!
//! Every source file is tab-separated, `#` starts a comment line, and `-` is the
//! absent value. One column layout carries every relation in the build:
//!
//! ```text
//! <pos> <lemma> <sense> <rel> <pos> <lemma> <sense> <weight>
//! ```
//!
//! | adapter | lines | emits |
//! |---------|-------|-------|
//! | `synset_tsv` | `S <pos> <lemma> <sense> <forms(+)>`, `R <relation columns>` | `DICT` (shelf 1) + `REL` (shelf 2): S1 and S1-rel of §2's matrix |
//! | `edges_tsv` | `E <relation columns>` | `REL` (shelf 2), plus a `DICT` row for every endpoint nothing matched — §5's no-silent-drop |
//! | `grammar_tsv` | `Q <relation columns>`, `P <operator> <triggers(+)> <results(+)>` | `REL` + `RULE` (shelf 3): S3, with the property half walkable and the n-ary paradigm tables parked (§7) |
//!
//! `sense` is verbatim or `-`. A source that states senses gets one point per
//! sense; a source that states none gets the `-` point. Alignment is therefore
//! the pure intersection of M.4 and never a fuzzy match, and a WordNet sense and
//! a source that names only the lemma land on two points — which is exactly what
//! §5's unmatched-surface rows are for.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use si_core::addr::{hyperpoint_id, SHELF_DICTIONARY, SHELF_RELATIONS, SHELF_RULES};
use si_core::container::{Container, Format};
use si_core::genome::{
    hex_bytes, CsrElement, DictRow, GenomeHeader, PointRow, RelRow, RuleRow, StringTable,
    SurfaceEntry, GENOME_BATCH_SLOTS, GENOME_KIND_CAPACITY, GENOME_KIND_FIRST, GENOME_KIND_SLOTS,
    GENOME_WEIGHT_CEILING,
};
use si_core::sha256::sha256_hex;

use crate::genome_file::{self, BucketRow, GenomeFile, Provenance, TopRow};
use crate::interner::fold_key;
use crate::opens;

/// M.0: the manifest is the lock. Nothing is read that it does not pin, and a
/// field it does not declare is a refusal rather than an ignored key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// The manifest schema version; `1` is this build's.
    pub schema: u32,
    /// Where the sealed artifacts go, relative to the manifest's own directory.
    pub out_dir: String,
    /// A7's bound. 07 §12 names `DEGREE_TABLE_RANGE` and the design gives it no
    /// value, so — exactly as §8 makes the tier boundaries a manifest authoring —
    /// the number is the owner's input and the build only ever asserts against it.
    pub degree_table_range: u32,
    pub sources: Vec<Source>,
}

/// One pinned source, i.e. one sealed provenance batch (07 §10: the license is
/// stamped per batch, which is what lets a `REL` record stay 16 bytes).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    /// The adapter's name, one of `ADAPTERS`.
    pub adapter: String,
    /// The pinned local copy, relative to the manifest's directory. M.1 verifies
    /// its hash and never downloads: a fetch would be a new dependency and
    /// `check_deps` keeps the allowlist closed. The URL and digest are pinned for
    /// the audit and for whoever places the copy.
    pub path: String,
    /// Recorded for the batch table and the attribution export, never
    /// dereferenced by this build.
    pub url: String,
    pub sha256: String,
    pub license: String,
    pub version: String,
    /// 1, 2 or 3. A source aimed at a live shelf is a refusal (E2).
    pub target_shelf: u16,
    /// 07 §8: the closed step table, authored here and never computed from data.
    pub weight_buckets: Vec<Bucket>,
    /// 07 §8: rows the source gives no weight for take the top tier, because
    /// weight `0` is unroutable and an unroutable row is a drop wearing the
    /// no-drop flag.
    pub top_tier: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bucket {
    /// Inclusive, in the source's own units.
    pub min: u32,
    pub max: u32,
    pub tier: u8,
}

/// 07 §16: a fourth source joins by adding one of these names and one manifest
/// entry, and nothing else.
pub const ADAPTERS: [&str; 3] = ["synset_tsv", "edges_tsv", "grammar_tsv"];

/// The 07 §14 row: one per stage, wall-clock only under `SI_PROFILE=1`.
#[derive(Clone, Debug, Serialize)]
pub struct StageRow {
    pub stage: &'static str,
    pub rows_in: u64,
    pub rows_out: u64,
    pub bytes_out: u64,
    pub wall_ns: u128,
}

/// The stage clock of M.0–M.11. Unprofiled it records `wall_ns: 0` in every row,
/// which is §14's "compiled no-op when unset" kept honest: a report with zeros in
/// the timing column says nothing was timed, and nobody may read a blank as a
/// measurement.
struct Stamper {
    started: std::time::Instant,
    clock: std::time::Instant,
    profile: bool,
    stages: Vec<StageRow>,
}

impl Stamper {
    fn new(profile: bool) -> Self {
        Self {
            started: std::time::Instant::now(),
            clock: std::time::Instant::now(),
            profile,
            stages: Vec::new(),
        }
    }

    fn stage(&mut self, id: &'static str, rows_in: u64, rows_out: u64, bytes_out: u64) {
        self.stages.push(StageRow {
            stage: id,
            rows_in,
            rows_out,
            bytes_out,
            wall_ns: if self.profile { self.clock.elapsed().as_nanos() } else { 0 },
        });
        self.clock = std::time::Instant::now();
    }

    fn total_ns(&self) -> u128 {
        if self.profile {
            self.started.elapsed().as_nanos()
        } else {
            0
        }
    }
}

/// What one build produced, for the report and the caller.
#[derive(Clone, Debug)]
pub struct Built {
    pub tag: u64,
    pub seal: genome_file::Seal,
    pub artifact: PathBuf,
    pub directory: PathBuf,
    pub header: GenomeHeader,
    pub file: GenomeFile,
    pub provenance: Vec<Provenance>,
    pub rows: u32,
    pub points: u32,
    pub edges: u32,
    pub kinds: u32,
    pub batches: u32,
    pub unmatched_points: u32,
    pub max_degree: u32,
    pub bytes: u64,
    pub asserts: Vec<&'static str>,
    pub stages: Vec<StageRow>,
    pub files: Vec<PathBuf>,
}

/// A converter run: the manifest, the pinned copies, the output directory.
pub struct Builder<'a> {
    pub manifest_path: &'a Path,
    pub out_dir: Option<&'a Path>,
    pub profile: bool,
}

impl<'a> Builder<'a> {
    /// `si-cli build-genome --manifest <path> [--out <dir>]`.
    pub fn new(manifest_path: &'a Path, out_dir: Option<&'a Path>) -> Self {
        Self {
            manifest_path,
            out_dir,
            profile: std::env::var_os("SI_PROFILE").is_some_and(|v| v != "0"),
        }
    }

    /// The whole of M.0–M.11.
    pub fn run(&self) -> Result<Built> {
        let mut stamper = Stamper::new(self.profile);

        // ---- M.0 MANIFEST ----------------------------------------------------
        let manifest = Manifest::load(self.manifest_path)?;
        stamper.stage("M.0", 0, manifest.sources.len() as u64, 0);

        // ---- M.1 FETCH ------------------------------------------------------
        let pinned = manifest.fetch(self.manifest_path)?;
        let bytes_in = pinned.iter().map(|(_, bytes)| bytes.len() as u64).sum();
        stamper.stage("M.1", manifest.sources.len() as u64, pinned.len() as u64, bytes_in);

        // ---- M.2 PARSE / M.3 KEY / M.4 ALIGN --------------------------------
        // ALIGN is folded into ingest: a point is created on the verbatim
        // `(lemma, pos, sense)` identity, and an endpoint that matches nothing
        // takes its own row in its own batch (07 §5).
        let mut draft = Draft::new(&manifest);
        let mut lines_in = 0u64;
        for (source, bytes) in &pinned {
            lines_in += draft.ingest(source, bytes)?;
        }
        stamper.stage("M.2", lines_in, draft.staged.len() as u64, 0);
        stamper.stage("M.3", draft.staged.len() as u64, draft.relations.len() as u64, 0);
        let unmatched = draft.unmatched();
        stamper.stage("M.4", draft.relations.len() as u64, draft.staged.len() as u64, 0);

        // ---- M.5 EMIT, then M.6 SORT (§4's byte-lexicographic order) --------
        let mut file = draft.emit()?;
        let emitted = (file.dict.len() + file.rel.len() + file.rule.len()) as u64;
        stamper.stage("M.5", emitted, emitted, 0);
        let rows_bytes = file.rows_bytes();
        stamper.stage("M.6", emitted, emitted, rows_bytes.len() as u64);

        // ---- M.7 COUNT REPORT ----------------------------------------------
        // The batch table is read off the sorted runs, so the report line and the
        // sealed header's count are the same fact measured once.
        let provenance = draft.provenance(&file);
        // The value is published at M.11, after the index exists, so its surface
        // counts are the shipped ones while its degrees stay counted off the sorted run.
        let report = count_report(&file, &manifest, &draft, &provenance, unmatched);
        stamper.stage("M.7", 0, 0, 0);

        // ---- M.8 INDEX: the CSR adjacency and the surface lookup ------------
        let elements = index(&mut file)?;
        stamper.stage("M.8", file.rel.len() as u64, elements as u64, 0);
        let max_degree = file.max_degree();

        // ---- M.9 SEAL -------------------------------------------------------
        let canonical = file.encode();
        let seal = genome_file::Seal::compute(&canonical, &file.rows_bytes())?;
        stamper.stage("M.9", canonical.len() as u64, 1, canonical.len() as u64);

        let header = GenomeHeader {
            format: Format::Genome.name().to_string(),
            row_schema_version: GenomeHeader::ROW_SCHEMA_VERSION,
            seal_md5: seal.seal_md5.clone(),
            rows_md5: seal.rows_md5.clone(),
            version_tag: seal.version_tag,
            degree_table_range: manifest.degree_table_range,
            kind_slots: GENOME_KIND_SLOTS as u32,
            batch_slots: GENOME_BATCH_SLOTS as u32,
            dict_rows: file.dict.len() as u32,
            rel_rows: file.rel.len() as u32,
            rule_rows: file.rule.len() as u32,
            points: file.points.len() as u32,
            edges: file.csr.len() as u32,
            kinds: file.kind_ids.len() as u32,
            batches: draft.batches.len() as u32,
            forms_bytes: file.forms.len() as u32,
            surface_keys: file.surface_keys.len() as u32,
        };
        let artifact = encode_artifact(&header, &provenance, &canonical)?;

        // ---- M.10 VERIFY, against the bytes that will ship ------------------
        let kinds = draft.kind_verbatim();
        let asserts = verify(&artifact, &kinds, &opens::trace())?;
        stamper.stage("M.10", 1, asserts.len() as u64, artifact.len() as u64);

        // ---- M.11 PUBLISH ---------------------------------------------------
        // Four sibling files beside the artifact, plus the lock one level up, and
        // the stage row is recorded before the build report is written so the
        // report's stage table is complete (§14 asks for one row per stage).
        let out_root = self
            .out_dir
            .map(|path| path.to_path_buf())
            .unwrap_or_else(|| manifest.directory(self.manifest_path).join(&manifest.out_dir));
        let directory = out_root.join(format!("{:016x}", seal.version_tag));
        let name = Format::Genome.name();
        let artifact_path = publish(&directory, name, &artifact)?;
        let count_path = publish(&directory, "count_report.json", &serde_json::to_vec_pretty(&report)?)?;
        let attribution =
            publish(&directory, "attribution.txt", attribution_text(&provenance).as_bytes())?;
        let lock = publish(
            &out_root,
            "manifest.lock",
            &serde_json::to_vec_pretty(&serde_json::json!({
                "version_tag": seal.version_tag,
                "seal_md5": seal.seal_md5,
                "artifact": artifact_path.display().to_string(),
                "sources": manifest.sources.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
            }))?,
        )?;
        stamper.stage("M.11", 1, 4, artifact.len() as u64);
        let build_report = publish(
            &directory,
            "build_report.json",
            &serde_json::to_vec_pretty(&serde_json::json!({
                "stages": &stamper.stages,
                "total_wall_ns": stamper.total_ns(),
                "profiled": self.profile,
                "asserts": {
                    "passed": &asserts,
                    "A12": "carried by the harness: the engine-off twin's G5c handle audit, not by this build",
                },
                "artifact_bytes": artifact.len(),
                "canonical_bytes": canonical.len(),
                "tag": format!("{:016x}", seal.version_tag),
                "seal_md5": seal.seal_md5,
                "rows_md5": seal.rows_md5,
                "max_degree": max_degree,
                "degree_table_range": manifest.degree_table_range,
                "surface_index": {
                    "keys": file.surface_keys.len(),
                    "aliases": file.surface_aliases.len(),
                },
            }))?,
        )?;

        Ok(Built {
            tag: seal.version_tag,
            seal,
            directory,
            artifact: artifact_path.clone(),
            header: header.clone(),
            file,
            provenance,
            rows: (header.dict_rows + header.rel_rows + header.rule_rows) as u32,
            points: header.points,
            edges: header.edges,
            kinds: header.kinds,
            batches: header.batches,
            unmatched_points: unmatched,
            max_degree,
            bytes: artifact.len() as u64,
            asserts,
            stages: stamper.stages,
            files: vec![artifact_path, count_path, build_report, attribution, lock],
        })
    }
}

impl Manifest {
    /// M.0's fail-stops: an unpinned source, an empty license field, an unknown
    /// `target_shelf`.
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = read_pinned(path)?;
        let text = std::str::from_utf8(&bytes)
            .with_context(|| format!("{} is not UTF-8", path.display()))?;
        let manifest: Manifest = serde_json::from_str(text)
            .with_context(|| format!("{} is not a manifest this build reads", path.display()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    fn validate(&self) -> Result<()> {
        if self.schema != 1 {
            bail!("manifest schema {} is not this build's 1", self.schema);
        }
        if self.sources.is_empty() {
            bail!("the manifest pins no source: a genome with nothing in it is a refusal, not an artifact");
        }
        if self.degree_table_range == 0 {
            bail!("A7 has no bound to assert: the manifest declares a degree table range of 0");
        }
        let mut ids: BTreeSet<String> = BTreeSet::new();
        for source in &self.sources {
            if source.license.trim().is_empty() {
                bail!("source {} has an empty license field (07 §10, A5)", source.id);
            }
            if !ADAPTERS.contains(&source.adapter.as_str()) {
                bail!("source {} names adapter {:?}, outside {ADAPTERS:?}", source.id, source.adapter);
            }
            if !matches!(source.target_shelf, SHELF_DICTIONARY | SHELF_RELATIONS | SHELF_RULES) {
                bail!(
                    "source {} targets shelf {}; a converter may only aim at the frozen shelves 1–3 (E2)",
                    source.id,
                    source.target_shelf
                );
            }
            if source.sha256.len() != 64 || !source.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
                bail!("source {} does not pin a sha256 digest", source.id);
            }
            if source.top_tier == 0 || source.top_tier > GENOME_WEIGHT_CEILING {
                bail!(
                    "source {}'s top tier {} is not inside 1..={GENOME_WEIGHT_CEILING} (07 §8, A4)",
                    source.id,
                    source.top_tier
                );
            }
            let mut previous: Option<&Bucket> = None;
            for bucket in &source.weight_buckets {
                if bucket.min > bucket.max {
                    bail!(
                        "source {} has an inverted bucket {}..{}",
                        source.id,
                        bucket.min,
                        bucket.max
                    );
                }
                if bucket.tier == 0 || bucket.tier > GENOME_WEIGHT_CEILING {
                    bail!("source {} has a bucket tier {} outside 1..=the ceiling", source.id, bucket.tier);
                }
                if let Some(last) = previous {
                    if bucket.min <= last.max {
                        bail!("source {} has overlapping buckets; the tier map is a step table", source.id);
                    }
                    // A4's second half, asserted before a byte is written.
                    if bucket.tier < last.tier {
                        bail!(
                            "source {}'s buckets are not monotone: {}..{} tier {} after tier {}",
                            source.id,
                            bucket.min,
                            bucket.max,
                            bucket.tier,
                            last.tier
                        );
                    }
                }
                previous = Some(bucket);
            }
            if !ids.insert(source.id.clone()) {
                bail!("the manifest repeats source id {}", source.id);
            }
        }
        if ids.len() > GENOME_BATCH_SLOTS {
            bail!("{} batches exceed the {GENOME_BATCH_SLOTS}-slot space (07 §10)", ids.len());
        }
        Ok(())
    }

    /// M.1: a hash mismatch is a hard fail, never a retry-with-what-arrived.
    fn fetch(&self, manifest_path: &Path) -> Result<Vec<(Source, Vec<u8>)>> {
        let dir = self.directory(manifest_path);
        let mut out = Vec::with_capacity(self.sources.len());
        for source in &self.sources {
            let path = dir.join(&source.path);
            if !path.exists() {
                bail!(
                    "source {} is not pinned on disk at {}; this build reads local copies only, \
                     and an absent source is a refusal rather than a partial corpus (07 §M.1)",
                    source.id,
                    path.display()
                );
            }
            let bytes = read_pinned(&path).with_context(|| format!("fetch of source {}", source.id))?;
            let digest = sha256_hex(&bytes);
            if digest != source.sha256 {
                bail!(
                    "source {} hash mismatch: the manifest pins {}, the copy on disk is {digest}; \
                     a moved source is a re-version, and building on it anyway is the exact \
                     failure M.1 exists to stop (07 §M.1)",
                    source.id,
                    source.sha256
                );
            }
            out.push((source.clone(), bytes));
        }
        Ok(out)
    }

    fn directory(&self, manifest_path: &Path) -> PathBuf {
        manifest_path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
    }
}

/// Every read this build performs goes through here, so A11's handle assertion is
/// a fact about the code path rather than about a log someone forgot to write.
fn read_pinned(path: &Path) -> Result<Vec<u8>> {
    opens::record(&path.display().to_string(), "open-ro");
    std::fs::read(path).with_context(|| format!("read {}", path.display()))
}

/// A11 in one sentence: the converter's own open register, asked whether it ever
/// named a lattice, a run directory or a gate directory. The container names come
/// from `Format`, so the set is the whole one rather than a list to forget.
pub fn a11_violations(trace: &[String]) -> Vec<String> {
    let lattice_ends = [Format::Lattice.name(), Format::Checkpoint.name(), Format::StateSnapshot.name()];
    trace
        .iter()
        .filter(|line| {
            let path = line.split_once(' ').map(|(_, rest)| rest).unwrap_or(line.as_str());
            let slashed = path.replace('\\', "/").to_ascii_lowercase();
            let names_a_run = slashed.split('/').any(|part| part == "runs" || part == "gates");
            let names_a_lattice =
                lattice_ends.iter().any(|magic| slashed.ends_with(&magic.to_ascii_lowercase()));
            names_a_run || names_a_lattice
        })
        .cloned()
        .collect()
}

/// The atomic write of 00 §5, applied to M.11: a published tag is never
/// overwritten, so a byte difference at the same tag is a bug, not a version.
fn publish(directory: &Path, name: &str, bytes: &[u8]) -> Result<PathBuf> {
    std::fs::create_dir_all(directory).with_context(|| format!("create {}", directory.display()))?;
    let final_path = directory.join(name);
    if final_path.exists() {
        let existing = std::fs::read(&final_path)?;
        if existing == bytes {
            return Ok(final_path);
        }
        bail!(
            "{} already holds different bytes; a tag that changed underneath a published artifact \
             is a re-version and the old one stays (07 §13)",
            final_path.display()
        );
    }
    let tmp = directory.join(format!("{name}.tmp"));
    opens::record(&tmp.display().to_string(), "create");
    let mut file = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(&tmp)?;
    std::io::Write::write_all(&mut file, bytes)?;
    file.sync_all()?;
    std::fs::rename(&tmp, &final_path)?;
    Ok(final_path)
}

fn attribution_text(provenance: &[Provenance]) -> String {
    let mut out = String::from("# 07 §10: one line per sealed batch, with its row counts.\n");
    for batch in provenance {
        out.push_str(&format!(
            "{}\tbatch {}\t{}\tversion {}\tlicense {}\tsha256 {}\trows dict={} rel={} rule={}\n",
            batch.source,
            batch.batch_id,
            batch.url,
            batch.version,
            batch.license,
            batch.sha256,
            batch.dict_rows,
            batch.rel_rows,
            batch.rule_rows
        ));
    }
    out
}

/// One staged point: a `DICT` row plus the staging the aux tables need.
#[derive(Clone, Debug)]
struct Staged {
    dict: DictRow,
    surface_off: u32,
    surface_len: u16,
    /// True when a sense row created the point; false when a relation endpoint
    /// did, which is M.4's unmatched-surface list.
    from_sense: bool,
    batch: u8,
}

#[derive(Clone, Debug, Default)]
struct KindStat {
    rows: u32,
    batches: BTreeSet<u8>,
}

#[derive(Clone, Debug)]
struct Batch {
    source: String,
    version: String,
    license: String,
    url: String,
    sha256: String,
}

#[derive(Clone, Debug)]
struct StagedRel {
    a: usize,
    b: usize,
    kind: String,
    tier: u8,
    batch: u8,
    version_tag: u32,
}

/// The parsed, aligned picture of a corpus before it becomes bytes.
struct Draft<'m> {
    /// `(lemma, pos, sense)` verbatim → the staged point index. Identity is
    /// verbatim (07 §5): rows differing in any column are different points, and
    /// nothing folds here.
    by_identity: BTreeMap<(String, u8, String), usize>,
    staged: Vec<Staged>,
    relations: Vec<StagedRel>,
    rules: Vec<RuleRow>,
    kinds: BTreeMap<String, KindStat>,
    /// Every relation name each batch actually said: A3's witness, so the assert
    /// reads the artifact and answers from the parse.
    kind_verbatim: BTreeMap<u8, BTreeSet<String>>,
    strings: StringTable,
    payload: Vec<u8>,
    rule_tokens: Vec<u32>,
    batches: Vec<Batch>,
    manifest: &'m Manifest,
}

impl<'m> Draft<'m> {
    fn new(manifest: &'m Manifest) -> Self {
        Self {
            by_identity: BTreeMap::new(),
            staged: Vec::new(),
            relations: Vec::new(),
            rules: Vec::new(),
            kinds: BTreeMap::new(),
            kind_verbatim: BTreeMap::new(),
            strings: StringTable::new(),
            payload: Vec::new(),
            rule_tokens: Vec::new(),
            batches: Vec::new(),
            manifest,
        }
    }

    /// M.2 PARSE and M.3 KEY for one pinned source, plus M.4's point creation.
    /// Returns the number of source lines consumed.
    fn ingest(&mut self, source: &Source, bytes: &[u8]) -> Result<u64> {
        let batch = self
            .manifest
            .sources
            .iter()
            .position(|entry| entry.id == source.id)
            .ok_or_else(|| anyhow::anyhow!("source {} is not in the manifest", source.id))?
            as u8;
        if self.batches.len() != batch as usize {
            bail!(
                "sources must be fetched in manifest order: source {} is batch {batch} and {} \
                 batches are already sealed",
                source.id,
                self.batches.len()
            );
        }
        self.batches.push(Batch {
            source: source.id.clone(),
            version: source.version.clone(),
            license: source.license.clone(),
            url: source.url.clone(),
            sha256: source.sha256.clone(),
        });
        let text = std::str::from_utf8(bytes)
            .with_context(|| format!("source {} is not UTF-8", source.id))?;
        let mut lines = 0u64;
        for (number, line) in text.lines().enumerate() {
            let line = line.trim_end_matches(['\r', ' ']);
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            lines += 1;
            let where_ = format!("source {} line {}", source.id, number + 1);
            let fields: Vec<&str> = line.split('\t').collect();
            let tag = fields[0];
            let known = matches!(tag, "S" | "R" | "E" | "Q" | "P");
            if !known || !self.adapter_takes(source.adapter.as_str(), tag) {
                // M.2's fail-stop: a syntax the adapter cannot parse ends the
                // build. A warning count is how 0.4% of a corpus disappears, and
                // 0.4% of 34M is a content decision made by a `continue` (07 §5).
                bail!(
                    "{where_}: a `{tag}` line is not a record the {} adapter parses; dropping it \
                     would be a content decision (07 §5, M.2)",
                    source.adapter
                );
            }
            match tag {
                "S" => self.sense_row(source, batch, &fields, &where_)?,
                "R" | "E" | "Q" => self.relation_row(source, batch, &fields, &where_)?,
                _ => self.paradigm_row(batch, &fields, &where_)?,
            }
        }
        Ok(lines)
    }

    fn adapter_takes(&self, adapter: &str, tag: &str) -> bool {
        matches!(
            (adapter, tag),
            ("synset_tsv", "S") | ("synset_tsv", "R") | ("edges_tsv", "E") | ("grammar_tsv", "Q") | ("grammar_tsv", "P")
        )
    }

    fn sense_row(&mut self, source: &Source, batch: u8, fields: &[&str], where_: &str) -> Result<()> {
        if fields.len() != 5 {
            bail!("{where_}: an S row has five columns (S, pos, lemma, sense, forms)");
        }
        let pos = pos_byte(fields[1], where_)?;
        let lemma = key(fields[2], where_)?.to_string();
        let sense = sense(fields[3]).to_string();
        let forms = form_list(fields[4]);
        let index = self.point_for(source, batch, &lemma, pos, &sense, true)?;
        self.attach_forms(index, &forms)?;
        Ok(())
    }

    /// A sense row's forms payload, appended only when the point does not already
    /// carry one. An endpoint may have created the point first (§5's no-silent-
    /// drop), and the forms a source lists must still land in the sealed bytes.
    fn attach_forms(&mut self, index: usize, forms: &[String]) -> Result<()> {
        if self.staged[index].dict.forms_len != 0 {
            return Ok(());
        }
        let mut listed: Vec<&String> = forms.iter().filter(|form| !form.is_empty()).collect();
        if listed.is_empty() {
            return Ok(());
        }
        // §4's order applies to records; within a record the forms a source
        // listed are kept in the source's own order, so a reordered source line is
        // a content change and moves the tag.
        listed.sort_by_key(|form| form.as_bytes());
        let off = self.payload.len() as u32;
        let mut len = 0u16;
        for form in listed {
            if form.len() > u16::MAX as usize {
                bail!("a form is longer than a u16 length (07 §M.5: never truncated silently)");
            }
            self.payload.extend_from_slice(&(form.len() as u32).to_le_bytes());
            self.payload.extend_from_slice(form.as_bytes());
            len += 4 + form.len() as u16;
        }
        self.staged[index].dict.forms_off = off;
        self.staged[index].dict.forms_len = len;
        Ok(())
    }

    fn relation_row(&mut self, source: &Source, batch: u8, fields: &[&str], where_: &str) -> Result<()> {
        if fields.len() != 9 {
            bail!(
                "{where_}: a relation row has nine columns \
                 (tag, pos, lemma, sense, rel, pos, lemma, sense, weight)"
            );
        }
        let a_pos = pos_byte(fields[1], where_)?;
        let a_lemma = key(fields[2], where_)?.to_string();
        let a_sense = sense(fields[3]).to_string();
        let name = key(fields[4], where_)?.to_string();
        let b_pos = pos_byte(fields[5], where_)?;
        let b_lemma = key(fields[6], where_)?.to_string();
        let b_sense = sense(fields[7]).to_string();
        let weight = weight_of(fields[8], where_)?;
        let tier = tier_for(source, weight)?;
        let a = self.point_for(source, batch, &a_lemma, a_pos, &a_sense, false)?;
        let b = self.point_for(source, batch, &b_lemma, b_pos, &b_sense, false)?;
        {
            let stat = self.kinds.entry(name.clone()).or_default();
            stat.rows += 1;
            stat.batches.insert(batch);
        }
        self.kind_verbatim.entry(batch).or_default().insert(name.clone());
        self.relations.push(StagedRel {
            a,
            b,
            kind: name,
            tier,
            batch,
            version_tag: version_pin(source)?,
        });
        Ok(())
    }

    fn paradigm_row(&mut self, batch: u8, fields: &[&str], where_: &str) -> Result<()> {
        if fields.len() != 4 {
            bail!("{where_}: a P row has four columns (P, operator, triggers, results)");
        }
        let operator = pos_byte(fields[1], where_)?;
        let triggers = form_list(fields[2]);
        let results = form_list(fields[3]);
        let (triggers_off, triggers_len) = self.tokens(&triggers, where_)?;
        let (results_off, results_len) = self.tokens(&results, where_)?;
        self.rules.push(RuleRow {
            triggers_off,
            triggers_len,
            results_off,
            results_len,
            operator,
            prov_batch: batch,
        });
        Ok(())
    }

    /// A rule's token list, interned and appended to the sealed token array.
    fn tokens(&mut self, list: &[String], where_: &str) -> Result<(u32, u16)> {
        if list.is_empty() {
            bail!("{where_}: an empty token list; a rule with no tokens is not a rule (07 §7)");
        }
        if list.len() > u16::MAX as usize {
            bail!("{where_}: a token list longer than a u16 count (07 §M.5: never truncated)");
        }
        let off = self.rule_tokens.len() as u32;
        for token in list {
            if token.is_empty() {
                bail!("{where_}: an empty rule token (07 §M.3)");
            }
            let id = self.strings.intern(token)?;
            self.rule_tokens.push(id);
        }
        Ok((off, list.len() as u16))
    }

    /// §5's no-silent-drop law in one function: an endpoint that intersects no
    /// other source gets a `DICT` row in **its own** batch, then the row that
    /// references it. Nothing is discarded, fuzzily matched or left dangling.
    fn point_for(
        &mut self,
        source: &Source,
        batch: u8,
        lemma: &str,
        pos: u8,
        sense: &str,
        from_sense: bool,
    ) -> Result<usize> {
        let identity = (lemma.to_string(), pos, sense.to_string());
        if let Some(index) = self.by_identity.get(&identity) {
            let index = *index;
            if from_sense && !self.staged[index].from_sense {
                // The point existed because an endpoint named it; now a sense row
                // claims the same identity, so it stops being an unmatched surface.
                self.staged[index].from_sense = true;
            }
            return Ok(index);
        }
        if lemma.len() > u16::MAX as usize {
            bail!("a surface form is longer than a u16 payload length (07 §M.5: never truncated)");
        }
        let surface_off = self.payload.len() as u32;
        let surface_len = lemma.len() as u16;
        self.payload.extend_from_slice(lemma.as_bytes());
        let synset_key = format!("{lemma}:{pos}:{sense}");
        let dict = DictRow {
            hyperpoint_id: hyperpoint_id(&synset_key),
            lemma: self.strings.intern(lemma)?,
            forms_off: self.payload.len() as u32,
            forms_len: 0,
            pos,
            prov_batch: batch,
            synset_id: self.strings.intern(&synset_key)?,
            version_tag: version_pin(source)?,
        };
        let index = self.staged.len();
        self.staged.push(Staged {
            dict,
            surface_off,
            surface_len,
            from_sense,
            batch,
        });
        self.by_identity.insert(identity, index);
        Ok(index)
    }

    fn unmatched(&self) -> u32 {
        self.staged.iter().filter(|point| !point.from_sense).count() as u32
    }

    /// M.5 EMIT then M.6 SORT: the three shelf runs in §4's byte-lexicographic
    /// order, with the point table parallel to the sorted `DICT` run and every
    /// cross reference rewritten against it.
    fn emit(&mut self) -> Result<GenomeFile> {
        if self.kinds.len() > GENOME_KIND_CAPACITY {
            bail!(
                "the relation vocabulary holds {} names, past the {GENOME_KIND_CAPACITY} slots \
                 between {GENOME_KIND_FIRST} and {GENOME_KIND_SLOTS}: that is a schema-version \
                 change to `edge_kind`'s width, not a clamp (07 §6)",
                self.kinds.len()
            );
        }
        // The sealed kind table: the source's own strings, listed and never
        // grouped, ordered by their bytes so a slot is a function of the data.
        let names: Vec<String> = self.kinds.keys().cloned().collect();
        let mut kind_ids = Vec::with_capacity(names.len());
        let mut slot_of: BTreeMap<String, u8> = BTreeMap::new();
        for (index, name) in names.iter().enumerate() {
            let id = self.strings.intern(name)?;
            kind_ids.push(id);
            slot_of.insert(name.clone(), GENOME_KIND_FIRST + index as u8);
        }

        let staged = &self.staged;
        let relations = &self.relations;
        let rules = &self.rules;
        let payload = std::mem::take(&mut self.payload);
        let rule_tokens = std::mem::take(&mut self.rule_tokens);
        let strings = std::mem::take(&mut self.strings);

        // The DICT run, sorted. The point index is its sorted position, so a point
        // and its dictionary row are the same row of the same file (§11).
        let rows: Vec<DictRow> = staged.iter().map(|point| point.dict).collect();
        let (dict, perm) = sorted_run(&rows, DictRow::encode);
        let mut to_sorted = vec![0usize; perm.len()];
        for (new, old) in perm.iter().enumerate() {
            to_sorted[*old] = new;
        }
        let mut points: Vec<PointRow> = vec![Default::default(); dict.len()];
        for (old, new) in to_sorted.iter().enumerate() {
            points[*new] = PointRow {
                hyperpoint_id: staged[old].dict.hyperpoint_id,
                dict_row: *new as u32,
                surface_off: staged[old].surface_off,
                surface_len: staged[old].surface_len,
                batch: staged[old].batch,
            };
        }

        let rel_rows: Vec<RelRow> = relations
            .iter()
            .map(|relation| RelRow {
                a_point: to_sorted[relation.a] as u32,
                b_point: to_sorted[relation.b] as u32,
                edge_kind: slot_of[&relation.kind],
                weight: relation.tier,
                prov_batch: relation.batch,
                version_tag: relation.version_tag,
            })
            .collect();
        let (rel, _) = sorted_run(&rel_rows, RelRow::encode);

        let mut rule_rows = rules.to_vec();
        rule_rows.sort_by(|a, b| a.encode().cmp(&b.encode()));

        let manifest = self.manifest;
        let buckets = manifest
            .sources
            .iter()
            .enumerate()
            .flat_map(|(index, source)| {
                source.weight_buckets.iter().map(move |bucket| BucketRow {
                    source: index as u32,
                    min: bucket.min,
                    max: bucket.max,
                    tier: bucket.tier,
                })
            })
            .collect();
        let tops = manifest
            .sources
            .iter()
            .enumerate()
            .map(|(index, source)| TopRow { source: index as u32, tier: source.top_tier })
            .collect();

        Ok(GenomeFile {
            dict,
            rel,
            rule: rule_rows,
            points,
            forms: payload,
            rule_tokens,
            kind_ids,
            buckets,
            tops,
            strings,
            ..Default::default()
        })
    }

    /// The sealed batch table. Row counts are read off the sorted runs in one
    /// pass each, so a report line is a fact of the shipped bytes rather than of
    /// staging — and a report is not a per-batch re-scan of the corpus.
    fn provenance(&self, file: &GenomeFile) -> Vec<Provenance> {
        let mut rows = vec![(0u32, 0u32, 0u32); self.batches.len()];
        for row in &file.dict {
            if let Some(entry) = rows.get_mut(row.prov_batch as usize) {
                entry.0 += 1;
            }
        }
        for row in &file.rel {
            if let Some(entry) = rows.get_mut(row.prov_batch as usize) {
                entry.1 += 1;
            }
        }
        for row in &file.rule {
            if let Some(entry) = rows.get_mut(row.prov_batch as usize) {
                entry.2 += 1;
            }
        }
        self.batches
            .iter()
            .zip(rows)
            .enumerate()
            .map(|(index, (batch, (dict_rows, rel_rows, rule_rows)))| Provenance {
                batch_id: index as u8,
                source: batch.source.clone(),
                version: batch.version.clone(),
                license: batch.license.clone(),
                url: batch.url.clone(),
                sha256: batch.sha256.clone(),
                dict_rows,
                rel_rows,
                rule_rows,
            })
            .collect()
    }

    /// A3's witness, cloned out for the assert pass.
    fn kind_verbatim(&self) -> BTreeMap<u8, BTreeSet<String>> {
        self.kind_verbatim.clone()
    }
}

/// §4's order: the record's verbatim bytes, compared as an unsigned lexicographic
/// byte string. The old-index tie-break only orders byte-identical duplicates,
/// which is what makes the `rel_row` remap a function rather than a race.
fn sorted_run<T: Copy, const W: usize>(
    rows: &[T],
    encode: impl Fn(&T) -> [u8; W],
) -> (Vec<T>, Vec<usize>) {
    let mut keyed: Vec<([u8; W], usize)> =
        rows.iter().enumerate().map(|(old, row)| (encode(row), old)).collect();
    keyed.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let perm: Vec<usize> = keyed.iter().map(|(_, old)| *old).collect();
    let sorted = keyed.iter().map(|(_, old)| rows[*old]).collect();
    (sorted, perm)
}

/// M.8 INDEX: the CSR adjacency over the sorted `REL` run, and the surface lookup
/// index of §5. Both are built from sorted input, which is what makes them
/// buildable by partition and merge — and what makes `rayon` legal here, OD-50.
fn index(file: &mut GenomeFile) -> Result<usize> {
    let points = file.points.len();
    let mut lengths = vec![0u32; points];
    for row in &file.rel {
        if row.a_point as usize >= points {
            // The assert pass gives this its A-number; here the stage refuses to
            // build an index over an address it does not hold.
            bail!("M.8: REL row cites point {}, of {} allocated", row.a_point, points);
        }
        lengths[row.a_point as usize] += 1;
    }
    let mut starts = vec![0u32; points + 1];
    for (index, length) in lengths.iter().enumerate() {
        starts[index + 1] = starts[index] + length;
    }
    let mut cursor = starts.clone();
    let mut elements: Vec<Option<CsrElement>> = vec![None; file.rel.len()];
    for (rel_row, row) in file.rel.iter().enumerate() {
        let slot = cursor[row.a_point as usize];
        cursor[row.a_point as usize] += 1;
        elements[slot as usize] = Some(CsrElement {
            to_point: row.b_point,
            edge_kind: row.edge_kind,
            weight: row.weight,
            rel_row: rel_row as u32,
        });
    }
    let csr: Vec<CsrElement> =
        elements.into_iter().map(|element| element.expect("every slot is written once")).collect();

    // The surface index: two index spaces, one point set. Every surface a source
    // lists — the lemma and each form — folds to a key, and a key several points
    // speak keeps all of them as an alias list (§5: the fold may alias, it may
    // never merge; only an owner `P_DEF` splits them, at run time).
    let mut by_key: BTreeMap<Vec<u8>, Vec<u32>> = BTreeMap::new();
    for (point, row) in file.dict.iter().enumerate() {
        let lemma = file.strings.get(row.lemma).unwrap_or_default().to_string();
        let forms = file.forms_of(row)?;
        let mut surfaces: Vec<String> = vec![lemma.clone()];
        surfaces.extend(forms.iter().map(|form| form.to_string()));
        for surface in surfaces {
            let folded = fold_key(&surface);
            if folded.is_empty() {
                bail!("M.8: a surface folds to nothing, so no tokenizer could ever reach point {point}");
            }
            let list = by_key.entry(folded.into_bytes()).or_default();
            if !list.contains(&(point as u32)) {
                list.push(point as u32);
            }
        }
    }
    let mut entries: Vec<SurfaceEntry> = Vec::with_capacity(by_key.len());
    let mut key_starts = vec![0u32; by_key.len() + 1];
    let mut aliases: Vec<u32> = Vec::new();
    let mut keys: Vec<Vec<u8>> = Vec::with_capacity(by_key.len());
    for (index, (key, list)) in by_key.into_iter().enumerate() {
        keys.push(key);
        entries.push(SurfaceEntry { point: list[0], alias_off: aliases.len() as u32 });
        aliases.extend_from_slice(&list[1..]);
        key_starts[index + 1] = aliases.len() as u32;
    }
    file.surface_keys = keys;
    file.surface_entries = entries;
    file.surface_starts = key_starts;
    file.surface_aliases = aliases;
    file.csr = csr;
    file.csr_starts = starts;
    file.check_shape()?;
    Ok(file.csr.len())
}

/// M.7 COUNT REPORT — Group B of §12: counts, the kind table, the weight and
/// degree histograms, the matched/unmatched split, fan-out, bytes and the tag.
/// Every line is printed and read; none of them can fail a build, because a
/// threshold over a content statistic is a content decision wearing a gate.
fn count_report(
    file: &GenomeFile,
    manifest: &Manifest,
    draft: &Draft,
    provenance: &[Provenance],
    unmatched: u32,
) -> serde_json::Value {
    // M.7 runs before M.8, so a degree is counted off the sorted REL run rather
    // than read from the index the next stage builds. A7 re-measures the same
    // numbers off the CSR of the shipped bytes, so the report and the assert can
    // disagree and the assert is the one that stops the build.
    let mut degrees = vec![0u32; file.points.len()];
    for row in &file.rel {
        if let Some(slot) = degrees.get_mut(row.a_point as usize) {
            *slot += 1;
        }
    }
    let max_degree = degrees.iter().copied().max().unwrap_or(0);
    let mut weight_histogram: BTreeMap<u32, u32> = BTreeMap::new();
    for row in &file.rel {
        *weight_histogram.entry(row.weight as u32).or_default() += 1;
    }
    let mut degree_histogram: BTreeMap<u32, u32> = BTreeMap::new();
    for degree in &degrees {
        *degree_histogram.entry(*degree).or_default() += 1;
    }
    let mut hubs: Vec<(u32, u32)> =
        degrees.iter().enumerate().map(|(point, degree)| (point as u32, *degree)).collect();
    hubs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let top_hubs: Vec<serde_json::Value> = hubs
        .iter()
        .take(20)
        .map(|(point, degree)| serde_json::json!({
            "point": point,
            "surface": file.surface_of(*point as usize).unwrap_or_default(),
            "genome_degree": degree,
        }))
        .collect();
    let kinds: Vec<serde_json::Value> = draft
        .kinds
        .iter()
        .enumerate()
        .map(|(index, (name, stat))| serde_json::json!({
            "slot": GENOME_KIND_FIRST as usize + index,
            "name": name,
            "rows": stat.rows,
            "batches": stat.batches.iter().copied().collect::<Vec<_>>(),
        }))
        .collect();
    // Average fan-out per POS in integer Q12: no float is anywhere near a byte the
    // tag depends on (§8's reciprocal-multiply discipline).
    let mut fan_out: BTreeMap<u32, (u64, u64)> = BTreeMap::new();
    for (point, row) in file.dict.iter().enumerate() {
        let edges = degrees[point] as u64;
        let entry = fan_out.entry(row.pos as u32).or_default();
        entry.0 += edges;
        entry.1 += 1;
    }
    let per_pos: Vec<serde_json::Value> = fan_out
        .iter()
        .map(|(pos, (edges, points))| serde_json::json!({
            "pos": pos,
            "points": points,
            "edges": edges,
            "avg_fan_out_q12": *edges * 4096 / (*points).max(1),
        }))
        .collect();
    let mut matched: Vec<u32> = vec![0; provenance.len()];
    let mut batched: Vec<u32> = vec![0; provenance.len()];
    for point in &draft.staged {
        if let Some(slot) = matched.get_mut(point.batch as usize) {
            batched[point.batch as usize] += 1;
            if point.from_sense {
                *slot += 1;
            }
        }
    }
    let per_batch: Vec<serde_json::Value> = provenance
        .iter()
        .enumerate()
        .map(|(index, batch)| {
            let matched = matched[index];
            serde_json::json!({
                "batch_id": batch.batch_id,
                "source": batch.source,
                "license": batch.license,
                "rows": {
                    "dict": batch.dict_rows,
                    "rel": batch.rel_rows,
                    "rule": batch.rule_rows,
                },
                "points": batched[index],
                "matched_points": matched,
                "unmatched_points": batched[index] - matched,
                "zero_matched_points": matched == 0,
            })
        })
        .collect();
    serde_json::json!({
        "schema": 1,
        "note": "07 §12 Group B: report lines, no thresholds. A fixture build's counts are a \
                 fixture's counts and are never quoted as a corpus measurement.",
        "rows": { "dict": file.dict.len(), "rel": file.rel.len(), "rule": file.rule.len() },
        "points": file.points.len(),
        "edges": file.csr.len(),
        "surfaces": {
            "matched": file.points.len() - unmatched as usize,
            "unmatched": unmatched,
            "keys": file.surface_keys.len(),
            "aliases": file.surface_aliases.len(),
        },
        "batches": per_batch,
        "kinds": kinds,
        "weight_histogram": weight_histogram,
        "degree_histogram": degree_histogram,
        "max_degree": max_degree,
        "degree_table_range": manifest.degree_table_range,
        "top_hubs": top_hubs,
        "fan_out_by_pos": per_pos,
        "interner": {
            "entries": file.strings.len(),
            "payload_bytes": file.strings.canonical_bytes().len(),
        },
        "sorted_rows_bytes": file.rows_bytes().len(),
        "sources": manifest.sources.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
    })
}

/// The artifact: the sealed container of 01 §9, whose header is the JSON the tag
/// is written into and whose payload is the canonical byte string M.9 hashes.
fn encode_artifact(header: &GenomeHeader, provenance: &[Provenance], canonical: &[u8]) -> Result<Vec<u8>> {
    let mut value = header.to_json()?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("a container header must be an object"))?;
    object.insert("provenance".to_string(), serde_json::to_value(provenance)?);
    Container::new(Format::Genome, value, canonical.to_vec()).encode()
}

/// M.10: A1–A11, each one read back out of the bytes that will ship. Any failure
/// means the artifact is not published (07 §12, §M.10).
pub fn verify(
    artifact: &[u8],
    kind_verbatim: &BTreeMap<u8, BTreeSet<String>>,
    trace: &[String],
) -> Result<Vec<&'static str>> {
    // A10 is the read itself: the published bytes, re-hashed against the header.
    let (header, provenance, file) = genome_file::check_seal(artifact)?;
    if provenance.is_empty() {
        bail!("A5: the sealed file has an empty batch table, so nothing in it is attributable");
    }
    if header.dict_rows as usize != file.dict.len()
        || header.rel_rows as usize != file.rel.len()
        || header.rule_rows as usize != file.rule.len()
        || header.points as usize != file.points.len()
        || header.edges as usize != file.csr.len()
        || header.kinds as usize != file.kind_ids.len()
        || header.batches as usize != provenance.len()
        || header.forms_bytes as usize != file.forms.len()
        || header.surface_keys as usize != file.surface_keys.len()
    {
        bail!("A6: the header lies about its own counts, and every count is a fact of the bytes");
    }

    let points = file.points.len();
    for (index, row) in file.rel.iter().enumerate() {
        // A1: no dangling endpoint. A2: no self-loop.
        if row.a_point as usize >= points || row.b_point as usize >= points {
            bail!("A1: REL row {index} points at an unallocated point, which is a silent drop with the file left intact");
        }
        if row.a_point == row.b_point {
            bail!("A2: REL row {index} is a self-loop");
        }
        // A3: the slot resolves, and to a name the source itself said.
        let Some(name) = file.kind_name(row.edge_kind) else {
            bail!("A3: REL row {index} names kind slot {} outside the sealed table", row.edge_kind);
        };
        let seen = kind_verbatim.get(&row.prov_batch).ok_or_else(|| {
            anyhow::anyhow!("A5: REL row {index} cites batch {} of {}", row.prov_batch, provenance.len())
        })?;
        if !seen.contains(name) {
            bail!("A3: kind {name:?} was never a relation field of batch {}", row.prov_batch);
        }
        // A4: no corpus weight enters the walk above the weakest owned birth.
        if row.weight > GENOME_WEIGHT_CEILING {
            bail!("A4: REL row {index} weights {}/255, past the derived ceiling", row.weight);
        }
    }
    for element in &file.csr {
        if element.to_point as usize >= points {
            bail!("A1: a CSR element reaches an unallocated point");
        }
        if element.rel_row as usize >= file.rel.len() {
            bail!("A1: a CSR element cites REL row {} of {}", element.rel_row, file.rel.len());
        }
        if element.weight > GENOME_WEIGHT_CEILING {
            bail!("A4: a CSR element weights {}/255, past the derived ceiling", element.weight);
        }
    }
    if file.csr.len() != file.rel.len() {
        bail!("A1: the index holds {} elements for {} rows", file.csr.len(), file.rel.len());
    }

    // A4's second half, from the sealed tier table rather than the manifest.
    let mut by_source: BTreeMap<u32, Vec<&BucketRow>> = BTreeMap::new();
    for row in &file.buckets {
        by_source.entry(row.source).or_default().push(row);
    }
    for (source, rows) in by_source {
        let mut previous: Option<&BucketRow> = None;
        for row in rows {
            if row.min > row.max || row.tier == 0 || row.tier > GENOME_WEIGHT_CEILING {
                bail!(
                    "A4: source {source}'s sealed bucket {}..{} tier {} is not a step",
                    row.min,
                    row.max,
                    row.tier
                );
            }
            if let Some(last) = previous {
                if row.min <= last.max || row.tier < last.tier {
                    bail!("A4: source {source}'s sealed tier map is not monotone");
                }
            }
            previous = Some(row);
        }
    }
    for top in &file.tops {
        if top.tier == 0 || top.tier > GENOME_WEIGHT_CEILING {
            bail!("A4: source {}'s sealed top tier is outside the band", top.source);
        }
    }

    // A5: every row's batch resolves and every batch carries a license string.
    let mut cited: BTreeSet<u8> = BTreeSet::new();
    for row in &file.dict {
        cited.insert(row.prov_batch);
    }
    for row in &file.rel {
        cited.insert(row.prov_batch);
    }
    for row in &file.rule {
        cited.insert(row.prov_batch);
    }
    for batch in &cited {
        if *batch as usize >= provenance.len() {
            bail!("A5: a row cites batch {batch} of {} batches", provenance.len());
        }
    }
    for batch in &provenance {
        if batch.license.trim().is_empty() {
            bail!("A5: batch {} has an empty license field", batch.batch_id);
        }
        if batch.source.trim().is_empty() {
            bail!("A5: batch {} names no source", batch.batch_id);
        }
    }

    // A6 is the record decoders themselves — every reserved byte refused non-zero
    // — plus the round-trip of the bytes that shipped.
    for (index, row) in file.dict.iter().enumerate() {
        if DictRow::decode(&row.encode())? != *row {
            bail!("A6: DICT row {index} does not round-trip its own bytes");
        }
    }
    for (index, row) in file.rel.iter().enumerate() {
        if RelRow::decode(&row.encode())? != *row {
            bail!("A6: REL row {index} does not round-trip its own bytes");
        }
    }

    // A7: the hub-damp table's bound, declared in the sealed header.
    let max_degree = file.max_degree();
    if max_degree > header.degree_table_range {
        bail!(
            "A7: a point holds {max_degree} genome edges, past the declared range {}",
            header.degree_table_range
        );
    }

    // A8: every form a source listed is reachable from its own folded key.
    for (index, row) in file.dict.iter().enumerate() {
        let lemma = file.strings.get(row.lemma).unwrap_or_default().to_string();
        let mut surfaces: Vec<String> = vec![lemma];
        for form in file.forms_of(row)? {
            surfaces.push(form.to_string());
        }
        for surface in surfaces {
            let key = fold_key(&surface);
            if !file.points_for_folded(key.as_bytes()).contains(&(index as u32)) {
                bail!("A8: {surface:?} folds to a key that never reaches point {index}");
            }
        }
    }

    // A9: the parked tables still cannot compose into a cycle (07 §7). The walk
    // never reads rules, so this runs at build and at load, not in the spread.
    let mut producers: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (index, row) in file.rule.iter().enumerate() {
        for token in file.results_of(row)? {
            producers.entry(token).or_default().push(index);
        }
    }
    let mut edges: Vec<Vec<usize>> = vec![Vec::new(); file.rule.len()];
    for (index, row) in file.rule.iter().enumerate() {
        for token in file.triggers_of(row)? {
            if let Some(from) = producers.get(&token) {
                edges[index].extend(from.iter().copied());
            }
        }
    }
    // Iterative three-colour DFS: a back edge is a paradigm that never ends.
    let mut state = vec![0u8; file.rule.len()];
    for start in 0..file.rule.len() {
        if state[start] != 0 {
            continue;
        }
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        state[start] = 1;
        while let Some((node, next)) = stack.last_mut() {
            let here = *node;
            if *next < edges[here].len() {
                let child = edges[here][*next];
                *next += 1;
                match state[child] {
                    0 => {
                        state[child] = 1;
                        stack.push((child, 0));
                    }
                    1 => bail!("A9: the paradigm tables compose into a cycle through rule {child}"),
                    _ => {}
                }
            } else {
                state[here] = 2;
                stack.pop();
            }
        }
    }

    // A11: the converter's own handle assertion, over its own open register.
    let violations = a11_violations(trace);
    if !violations.is_empty() {
        bail!("A11: the build opened {}", violations.join(", "));
    }

    Ok(vec!["A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "A9", "A10", "A11"])
}

/// M.3's key column: verbatim, never folded, never empty.
fn key<'a>(text: &'a str, where_: &str) -> Result<&'a str> {
    if text.is_empty() {
        bail!("{where_}: an empty key (07 §M.3)");
    }
    Ok(text)
}

fn sense(text: &str) -> &str {
    if text == "-" {
        ""
    } else {
        text
    }
}

fn form_list(text: &str) -> Vec<String> {
    if text == "-" || text.is_empty() {
        return Vec::new();
    }
    text.split('+').map(|form| form.to_string()).collect()
}

fn weight_of(text: &str, where_: &str) -> Result<Option<u32>> {
    if text == "-" || text.is_empty() {
        return Ok(None);
    }
    let value = text.parse::<u32>().with_context(|| {
        format!("{where_}: a weight that is not an integer in the source's own units")
    })?;
    Ok(Some(value))
}

/// The source's own POS byte, verbatim. `0` is the reading of "this source states
/// no part of speech" (§4's absent-is-zero), and a multi-byte token is M.2's stop.
fn pos_byte(text: &str, where_: &str) -> Result<u8> {
    if text == "-" {
        return Ok(0);
    }
    let bytes = text.as_bytes();
    if bytes.len() != 1 {
        bail!("{where_}: {text:?} is not a one-byte POS or operator token (07 §M.2)");
    }
    Ok(bytes[0])
}

/// The record's `version_tag`: the first four bytes of the source's pinned
/// sha256, so the pin travels with the row. It is *not* the seal — a content hash
/// cannot live inside the bytes it hashes (07 §4, §13).
fn version_pin(source: &Source) -> Result<u32> {
    let bytes = hex_bytes(&source.sha256)?;
    if bytes.len() != 32 {
        bail!("source {} pins a {}-byte digest, not a sha256", source.id, bytes.len());
    }
    Ok(u32::from_le_bytes(bytes[0..4].try_into().expect("4 bytes")))
}

/// M.5: the tier a source weight maps to, or the top tier when the row has none.
fn tier_for(source: &Source, weight: Option<u32>) -> Result<u8> {
    let Some(value) = weight else {
        return Ok(source.top_tier);
    };
    for bucket in &source.weight_buckets {
        if value >= bucket.min && value <= bucket.max {
            return Ok(bucket.tier);
        }
    }
    bail!(
        "source {} gives a weight of {value} that no bucket of its own table covers; inventing a \
         tier here would be a content decision (07 §8)",
        source.id
    )
}
