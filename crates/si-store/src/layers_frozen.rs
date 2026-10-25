//! 00 §6 / 07 §9: the frozen-layer entry, its two guards and the loaded layer.
//! CONFIG SI refuses before a path string is formed; a build without the
//! `engine` feature refuses before a handle exists. This file performs no I/O at
//! all, and the loader body that does is `frozen_engine`, compiled only behind
//! the feature (W.1).
//!
//! What this file does hold is the *shape* of a loaded genome — the sealed
//! header, its batch table, the canonical file — and the W.2–W.4 reads a wake
//! makes of it. Those are pure accessors over bytes someone else opened, which is
//! why the config guard can be tested without a disk (05 §20).

use std::fmt;
use std::path::PathBuf;

use si_core::genome::{CsrElement, GenomeHeader};

use crate::genome_file::{GenomeFile, Provenance};
use crate::interner::fold_key;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Config {
    #[default]
    Si,
    Engine,
}

impl Config {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "si" => Some(Config::Si),
            "engine" => Some(Config::Engine),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Config::Si => "si",
            Config::Engine => "engine",
        }
    }
}

/// The one artifact a CONFIG ENGINE open may name, plus its seal tag. CONFIG SI
/// never reaches a method on it: `load` refuses first, and the CLI does not
/// construct one at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrozenManifest {
    pub artifact: PathBuf,
    pub version_tag: u64,
}

impl FrozenManifest {
    pub fn pinned(artifact: impl Into<PathBuf>, version_tag: u64) -> Self {
        Self { artifact: artifact.into(), version_tag }
    }

    /// 00 §6.3: the path recorded as expected-absent. A file at this path during
    /// a CONFIG SI run is itself a G5c failure, not a convenience.
    pub fn expected_absent(&self) -> PathBuf {
        self.artifact.clone()
    }
}

/// 00 §6.3 / 07 §13: the one way a caller names a frozen artifact without writing
/// this module's manifest type at the call site. The config guard counts
/// occurrences of that type name, so a caller that needs a pin uses this.
pub fn pin(artifact: impl Into<PathBuf>, version_tag: u64) -> FrozenManifest {
    FrozenManifest::pinned(artifact, version_tag)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigRefusal {
    /// 00 §6.1: the runtime guard. A designed branch, not an error.
    SiHasNoFrozenLayer,
    /// 00 §6.2: the build guard. The loader body is absent from this binary.
    EngineFeatureAbsent,
    /// 07 §1 / §13: CONFIG ENGINE asked for its frozen layer with nothing pinned.
    /// The manifest lock is the only source list, so there is no default to fall
    /// back to and no directory to go looking in.
    NothingPinned,
}

impl fmt::Display for ConfigRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigRefusal::SiHasNoFrozenLayer => f.write_str(
                "CONFIG SI has no frozen layer: L1/L2 load only in CONFIG ENGINE (I2, 00 §6)",
            ),
            ConfigRefusal::EngineFeatureAbsent => f.write_str(
                "this build carries no frozen loader body; the refusal precedes any handle (00 §6.2)",
            ),
            ConfigRefusal::NothingPinned => f.write_str(
                "CONFIG ENGINE was asked with no pinned artifact: the manifest lock is the only \
                 source list, so there is nothing to open and no directory to search (07 §1, §13)",
            ),
        }
    }
}

impl std::error::Error for ConfigRefusal {}

/// What a `load` attempt can end in. The three branches are three different laws,
/// and a caller must not be able to confuse them: `Refused` is 00 §6's config
/// guard, taken before a handle exists; `TagMismatch` is 07 §13's statement that
/// this artifact is not the genome this session was born into; `Artifact` is A10
/// and the codec's own refusals over bytes that were read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadError {
    Refused(ConfigRefusal),
    TagMismatch { pinned: u64, sealed: u64 },
    Artifact(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Refused(refusal) => refusal.fmt(f),
            LoadError::TagMismatch { pinned, sealed } => write!(
                f,
                "the artifact seals tag {sealed:016x} but the lock pins {pinned:016x}; a resume                  under a foreign genome is not a continuation (07 §13)",
            ),
            LoadError::Artifact(detail) => {
                write!(f, "the frozen artifact is not a file this build reads: {detail}")
            }
        }
    }
}

impl std::error::Error for LoadError {}

/// The loaded frozen layer: the sealed header, the batch table and the canonical
/// file, exactly as the artifact's bytes said them. The merged web a wake walks is
/// not materialized here — that copy is what 07 §9's load budget forbids.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frozen {
    pub manifest: FrozenManifest,
    pub header: GenomeHeader,
    pub provenance: Vec<Provenance>,
    pub file: GenomeFile,
}

impl Frozen {
    /// W.1's answer, and §13's: the content tag of the bytes now in memory.
    pub fn version_tag(&self) -> u64 {
        self.header.version_tag
    }

    /// 01 §10 item 8: does the genome tag this session stored still name the
    /// artifact now loaded? A false answer is a resume into somebody else's
    /// corpus, and 04 §25.5's seed refusal is the same law one row over.
    pub fn resumes_with(&self, genome_t0_tag: u64) -> bool {
        self.header.version_tag == genome_t0_tag
    }

    /// W.2: genome points own the handles `0…points-1`, so a live session
    /// allocates from here upward. Allocation order is a function of the tag,
    /// which is what makes a wake replayable.
    pub fn genome_points(&self) -> u32 {
        self.header.points
    }

    /// W.3's second cursor: the point's sealed adjacency row, in sealed order.
    /// The live `z` list is walked before it, and that order is the whole of
    /// §16.2's weighting law — the spread reads neither the row's source nor
    /// its kind byte.
    pub fn row(&self, point: u32) -> &[CsrElement] {
        self.file.row(point)
    }

    /// W.3's address space for a genome edge: the index of its row's first
    /// element in the sealed CSR run, which is what a merged arc carries where a
    /// live arc carries an arena byte offset (07 §11 counts edges as indices).
    pub fn row_start(&self, point: u32) -> u32 {
        self.file.row_start(point)
    }

    /// W.4: the genome half of a point's degree, which seeds the hub-damp table
    /// beside the live counts.
    pub fn degree_seed(&self, point: u32) -> u32 {
        self.file.row(point).len() as u32
    }

    /// A7 read back at the load: the widest sealed row, against the header's
    /// declared `DEGREE_TABLE_RANGE`.
    pub fn max_degree(&self) -> u32 {
        self.file.max_degree()
    }

    /// §6's citation table: what a slot means in the source's own words.
    pub fn kind_name(&self, edge_kind: u8) -> Option<&str> {
        self.file.kind_name(edge_kind)
    }

    /// §5's retrieval space: the surfaces a point speaks, lemma first.
    pub fn surface_of(&self, point: usize) -> anyhow::Result<&str> {
        self.file.surface_of(point)
    }

    /// The points a word reaches, through the same fold the live interner uses.
    pub fn points_for(&self, surface: &str) -> Vec<u32> {
        self.file.points_for_folded(fold_key(surface).as_bytes())
    }

    /// The same read for a caller who already holds a folded key — the round-0
    /// lighting's case, where folding twice would be a second fold law in a
    /// second crate. Sealed order, which is ascending handle order.
    pub fn points_for_folded(&self, folded: &str) -> Vec<u32> {
        self.file.points_for_folded(folded.as_bytes())
    }
}

/// `Frozen::load(config, manifest)` of 02 §2. The SI guard is the first
/// statement, so no path — not even a relative one — is formed for it. `None` is
/// 07 §1's refusal: CONFIG ENGINE with nothing pinned has no source list, and the
/// loader does not go looking for one (00 §6.3 records the absence).
pub fn load(config: Config, manifest: Option<&FrozenManifest>) -> Result<Frozen, LoadError> {
    if config == Config::Si {
        return Err(LoadError::Refused(ConfigRefusal::SiHasNoFrozenLayer));
    }
    load_engine(manifest)
}

#[cfg(feature = "engine")]
fn load_engine(manifest: Option<&FrozenManifest>) -> Result<Frozen, LoadError> {
    let Some(pinned) = manifest else {
        return Err(LoadError::Refused(ConfigRefusal::NothingPinned));
    };
    let (header, provenance, file) =
        crate::frozen_engine::open(&pinned.artifact, pinned.version_tag)?;
    Ok(Frozen { manifest: pinned.clone(), header, provenance, file })
}

#[cfg(not(feature = "engine"))]
fn load_engine(_manifest: Option<&FrozenManifest>) -> Result<Frozen, LoadError> {
    Err(LoadError::Refused(ConfigRefusal::EngineFeatureAbsent))
}
