//! 00 §5 / 01 §9 / 01 §13: the `SIst` state snapshot and the `SIck` checkpoint
//! — one container shape, a header that carries the geometry and the core block,
//! and the profile payload. A resume compares the header (seed, geometry,
//! digest) **before** a single arena byte is decoded.

use std::collections::BTreeMap;
use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use si_core::consts::CONSTS_VERSION;
use si_core::container::{Container, Format, CONTAINER_VERSION};
use si_core::encode::{Reader, Writer};
use si_core::hash::{CoreBlock, BLOCK_VERSION, CORE_BLOCK_BYTES, HASH_FORMAT};
use si_core::record::{BODY_BYTES, EDGE_BYTES, ENVELOPE_BYTES, RECORD_BYTES, STAMP_BYTES};
use si_core::rng::{RngState, NO_SPARE};
use si_core::seed::SEED_FORMAT;
use si_core::shelves::SHELVES;

use crate::arena::Arena;
use crate::interner::Interner;
use crate::points::Points;
use crate::store::{Lattice, HANDLE_BYTES};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Geometry {
    pub record_bytes: u32,
    pub envelope_bytes: u32,
    pub body_bytes: u32,
    pub edge_bytes: u32,
    pub stamp_bytes: u32,
    pub handle_bytes: u32,
    pub core_block_bytes: u32,
    pub hash_format: u16,
    pub consts_version: u16,
    /// The base this session was born on. Compared against the pinned manifest
    /// at open (07 §13), never against the code's own geometry.
    pub genome_t0_tag: u64,
    pub shelf_row_schema: BTreeMap<String, u16>,
    pub grids: BTreeMap<String, String>,
}

impl Geometry {
    pub fn current() -> Self {
        let shelf_row_schema = SHELVES
            .iter()
            .map(|shelf| (shelf.shelf_id.to_string(), shelf.row_schema_version))
            .collect::<BTreeMap<_, _>>();
        let grids = [
            ("activation", "q12"),
            ("weight", "q12"),
            ("bake", "q12"),
            ("arousal", "u8-255"),
            ("confidence", "hundredths"),
            ("genome_weight", "u8-255"),
        ]
        .into_iter()
        .map(|(name, tag)| (name.to_string(), tag.to_string()))
        .collect();
        Self {
            record_bytes: RECORD_BYTES as u32,
            envelope_bytes: ENVELOPE_BYTES as u32,
            body_bytes: BODY_BYTES as u32,
            edge_bytes: EDGE_BYTES as u32,
            stamp_bytes: STAMP_BYTES as u32,
            handle_bytes: HANDLE_BYTES as u32,
            core_block_bytes: CORE_BLOCK_BYTES as u32,
            hash_format: HASH_FORMAT,
            consts_version: CONSTS_VERSION,
            genome_t0_tag: 0,
            shelf_row_schema,
            grids,
        }
    }

    pub fn first_mismatch(&self, current: &Self) -> Option<&'static str> {
        if self.record_bytes != current.record_bytes {
            return Some("record_bytes");
        }
        if self.envelope_bytes != current.envelope_bytes {
            return Some("envelope_bytes");
        }
        if self.body_bytes != current.body_bytes {
            return Some("body_bytes");
        }
        if self.edge_bytes != current.edge_bytes {
            return Some("edge_bytes");
        }
        if self.stamp_bytes != current.stamp_bytes {
            return Some("stamp_bytes");
        }
        if self.handle_bytes != current.handle_bytes {
            return Some("handle_bytes");
        }
        if self.core_block_bytes != current.core_block_bytes {
            return Some("core_block_bytes");
        }
        if self.hash_format != current.hash_format {
            return Some("hash_format");
        }
        if self.consts_version != current.consts_version {
            return Some("consts_version");
        }
        if self.shelf_row_schema != current.shelf_row_schema {
            return Some("shelf_row_schema");
        }
        if self.grids != current.grids {
            return Some("grids");
        }
        None
    }
}

/// A typed refusal, because a resume that continues another individual is a
/// branch the design requires, not an `anyhow` string (02 §1's error rule).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResumeRefusal {
    ContainerVersion { stored: u32 },
    SeedMismatch { stored: u64, requested: u64 },
    SeedFormat { stored: u16 },
    Geometry { field: &'static str },
    DigestMismatch { stored: u64 },
    /// 07 §13 / 04 §25 step 2's W.1: the genome this session was born on is not
    /// the pinned artifact now loaded, so the rows would claim a history thought
    /// over a different corpus.
    GenomeTag { stored: u64, pinned: u64 },
}

impl fmt::Display for ResumeRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContainerVersion { stored } => {
                write!(f, "snapshot container version {stored} is not {CONTAINER_VERSION}")
            }
            Self::SeedMismatch { stored, requested } => write!(
                f,
                "resume seed {requested:#018x} is not the individual's stored seed {stored:#018x}"
            ),
            Self::SeedFormat { stored } => {
                write!(f, "snapshot seed_format {stored:#06x} is not {SEED_FORMAT:#06x}")
            }
            Self::Geometry { field } => write!(f, "snapshot geometry mismatches on `{field}`"),
            Self::DigestMismatch { stored } => write!(
                f,
                "core block digest {stored:#018x} does not re-derive from its own seed (G14(c))"
            ),
            Self::GenomeTag { stored, pinned } => write!(
                f,
                "the session was born on genome {stored:016x} and the pinned artifact seals \
                 {pinned:016x}; resuming under a foreign corpus is not a continuation (07 §13)"
            ),
        }
    }
}

impl std::error::Error for ResumeRefusal {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotHeader {
    pub container_version: u32,
    pub geometry: Geometry,
    pub core: CoreBlock,
}

impl SnapshotHeader {
    /// CONFIG SI's refusal: there is no pinned artifact, so W.1's second half has
    /// nothing to compare.
    pub fn first_refusal(&self, requested_seed: u64) -> Option<ResumeRefusal> {
        self.first_refusal_pinned(requested_seed, None)
    }

    /// The refusal order: framing, the individual, the geometry, then the
    /// digest. All of it is decided before the payload is read.
    ///
    /// `pinned_genome_tag` is `None` in CONFIG SI, which has no frozen layer to
    /// compare against; `Some(tag)` is W.1's second half — the manifest's own pin
    /// is already checked at the load, so what is left to check here is that the
    /// *session* was born on that same genome.
    pub fn first_refusal_pinned(
        &self,
        requested_seed: u64,
        pinned_genome_tag: Option<u64>,
    ) -> Option<ResumeRefusal> {
        if self.container_version != CONTAINER_VERSION {
            return Some(ResumeRefusal::ContainerVersion { stored: self.container_version });
        }
        if self.core.birth_seed != requested_seed {
            return Some(ResumeRefusal::SeedMismatch {
                stored: self.core.birth_seed,
                requested: requested_seed,
            });
        }
        if self.core.seed_format != SEED_FORMAT {
            return Some(ResumeRefusal::SeedFormat { stored: self.core.seed_format });
        }
        if let Some(field) = self.geometry.first_mismatch(&Geometry::current()) {
            return Some(ResumeRefusal::Geometry { field });
        }
        if let Some(pinned) = pinned_genome_tag {
            if self.geometry.genome_t0_tag != pinned {
                return Some(ResumeRefusal::GenomeTag {
                    stored: self.geometry.genome_t0_tag,
                    pinned,
                });
            }
        }
        if !self.core.digest_matches_seed() {
            return Some(ResumeRefusal::DigestMismatch { stored: self.core.temperament_digest });
        }
        None
    }
}

#[derive(Clone, Debug)]
pub enum Inspect {
    Accepted(SnapshotHeader),
    Refused(ResumeRefusal),
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub header: SnapshotHeader,
    pub rows: Arena,
    pub edges: Arena,
    pub stamps: Arena,
    pub handles: Arena,
    pub interner: Interner,
    pub points: Points,
    pub settle: Vec<(u32, u16)>,
    pub trajectory: Vec<(u16, u8)>,
    /// The current `a` on the 1/255 grid. The trajectory is the hashed artifact;
    /// this scalar is the integrator's live value a resume must not lose.
    pub a: u8,
    pub rng: RngState,
    pub advances: Vec<(i32, u32)>,
}

impl Snapshot {
    #[allow(clippy::too_many_arguments)]
    pub fn capture(
        core: CoreBlock,
        store: &Lattice,
        settle: &[(u32, u16)],
        trajectory: &[(u16, u8)],
        rng: RngState,
        advances: &[(i32, u32)],
        a: u8,
    ) -> Self {
        Self {
            header: SnapshotHeader {
                container_version: CONTAINER_VERSION,
                // W.2/W.1: the tag is what the lattice's attached layer seals, so
                // a CONFIG SI capture writes the 0 it always wrote and an ENGINE
                // one cannot forget to record its corpus.
                geometry: Geometry {
                    genome_t0_tag: store.frozen().map_or(0, |layer| layer.version_tag()),
                    ..Geometry::current()
                },
                core,
            },
            rows: store.rows_arena().clone(),
            edges: store.edges_arena().clone(),
            stamps: store.stamps_arena().clone(),
            handles: store.handles_arena().clone(),
            interner: store.interner().clone(),
            points: store.points().clone(),
            settle: settle.to_vec(),
            trajectory: trajectory.to_vec(),
            a,
            rng,
            advances: advances.to_vec(),
        }
    }

    pub fn store(&self) -> Result<Lattice> {
        Lattice::from_parts(
            self.rows.clone(),
            self.edges.clone(),
            self.stamps.clone(),
            self.handles.clone(),
            self.interner.clone(),
            self.points.clone(),
        )
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        self.encode_as(Format::StateSnapshot)
    }

    pub fn encode_as(&self, format: Format) -> Result<Vec<u8>> {
        if !matches!(format, Format::StateSnapshot | Format::Checkpoint) {
            bail!("a session snapshot is SIst or SIck, not {}", format.name());
        }
        let mut header = Map::new();
        header.insert("geometry".to_string(), serde_json::to_value(&self.header.geometry)?);
        header.insert("core_block".to_string(), core_to_value(&self.header.core));
        let container =
            Container::new(format, Value::Object(header), self.encode_payload());
        container.encode()
    }

    pub fn read_header(bytes: &[u8]) -> Result<SnapshotHeader> {
        let container = Container::decode(bytes)?;
        if !matches!(container.format, Format::StateSnapshot | Format::Checkpoint) {
            bail!("container is {}, not a session snapshot", container.format.name());
        }
        let object = container
            .header
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("snapshot header is not an object"))?;
        let geometry_value = object
            .get("geometry")
            .ok_or_else(|| anyhow::anyhow!("snapshot header has no `geometry`"))?;
        let geometry: Geometry = serde_json::from_value(geometry_value.clone())
            .context("snapshot header geometry is not readable")?;
        let core = core_from_value(
            object
                .get("core_block")
                .ok_or_else(|| anyhow::anyhow!("snapshot header has no `core_block`"))?,
        )?;
        Ok(SnapshotHeader { container_version: container.version, geometry, core })
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        Self::decode_with(bytes, 0)
    }

    /// W.2's base is not a stored byte — it is what the pinned artifact says it
    /// is — so a CONFIG ENGINE resume passes the genome's own point count here
    /// after the header's tag has already been refused or accepted.
    pub fn decode_with(bytes: &[u8], point_base: u32) -> Result<Self> {
        let header = Self::read_header(bytes)?;
        let container = Container::decode(bytes)?;
        let (rows, edges, stamps, handles, interner, points, a, settle, trajectory, rng, advances) =
            decode_payload(&container.payload, point_base)?;
        Ok(Self {
            header,
            rows,
            edges,
            stamps,
            handles,
            interner,
            points,
            settle,
            trajectory,
            a,
            rng,
            advances,
        })
    }

    pub fn inspect(bytes: &[u8], requested_seed: u64) -> Result<Inspect> {
        Self::inspect_pinned(bytes, requested_seed, None)
    }

    pub fn inspect_pinned(
        bytes: &[u8],
        requested_seed: u64,
        pinned_genome_tag: Option<u64>,
    ) -> Result<Inspect> {
        let header = Self::read_header(bytes)?;
        Ok(match header.first_refusal_pinned(requested_seed, pinned_genome_tag) {
            Some(refusal) => Inspect::Refused(refusal),
            None => Inspect::Accepted(header),
        })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        write_atomic(path, &self.encode()?)
    }

    fn encode_payload(&self) -> Vec<u8> {
        let mut w = Writer::new();
        put_bytes(&mut w, self.rows.bytes());
        put_bytes(&mut w, self.edges.bytes());
        put_bytes(&mut w, self.stamps.bytes());
        put_bytes(&mut w, self.handles.bytes());
        w.u32(self.interner.len() as u32);
        for entry in self.interner.surfaces() {
            w.str(&entry.verbatim);
        }
        put_bytes(&mut w, &self.points.canonical_bytes());
        w.u8(self.a);
        w.u32(self.settle.len() as u32);
        for (point, activation) in &self.settle {
            w.u32(*point);
            w.u16(*activation);
            w.u16(0);
        }
        w.u32(self.trajectory.len() as u32);
        for (round, arousal) in &self.trajectory {
            w.u16(*round);
            w.u8(*arousal);
            w.u8(0);
        }
        w.u32(self.advances.len() as u32);
        for (t, draws) in &self.advances {
            w.i32(*t);
            w.u32(*draws);
        }
        w.bytes(&self.rng.canonical_bytes());
        w.finish()
    }
}

fn put_bytes(w: &mut Writer, bytes: &[u8]) {
    w.u32(bytes.len() as u32);
    w.bytes(bytes);
}

type PayloadParts = (
    Arena,
    Arena,
    Arena,
    Arena,
    Interner,
    Points,
    u8,
    Vec<(u32, u16)>,
    Vec<(u16, u8)>,
    RngState,
    Vec<(i32, u32)>,
);

/// `point_base` is W.2's allocation base, never a stored byte: CONFIG SI reads 0
/// and a CONFIG ENGINE resume reads the pinned artifact's own point count.
fn decode_payload(payload: &[u8], point_base: u32) -> Result<PayloadParts> {
    let mut r = Reader::new(payload);
    let rows = take_arena(&mut r, RECORD_BYTES)?;
    let edges = take_arena(&mut r, EDGE_BYTES)?;
    let stamps = take_arena(&mut r, STAMP_BYTES)?;
    let handles = take_arena(&mut r, HANDLE_BYTES)?;

    let count = r.u32()? as usize;
    if count == 0 || count > payload.len() {
        bail!("interner count {count} is not plausible");
    }
    let mut surfaces = Vec::new();
    for _ in 0..count {
        surfaces.push(r.str()?.to_string());
    }
    let interner = Interner::from_surfaces(surfaces)?;

    let len = r.u32()? as usize;
    if len > r.remaining() {
        bail!("point table length {len} runs past the payload");
    }
    let points = Points::decode_with(r.bytes(len)?, point_base)?;
    let a = r.u8()?;

    let count = r.u32()? as usize;
    if count > payload.len() {
        bail!("settle-set count {count} is not plausible");
    }
    let mut settle = Vec::new();
    for _ in 0..count {
        let point = r.u32()?;
        let activation = r.u16()?;
        if r.u16()? != 0 {
            bail!("settle-set pad is not zero");
        }
        settle.push((point, activation));
    }

    let count = r.u32()? as usize;
    if count > payload.len() {
        bail!("trajectory count {count} is not plausible");
    }
    let mut trajectory = Vec::new();
    for _ in 0..count {
        let round = r.u16()?;
        let arousal = r.u8()?;
        if r.u8()? != 0 {
            bail!("trajectory pad is not zero");
        }
        trajectory.push((round, arousal));
    }

    let count = r.u32()? as usize;
    if count > payload.len() {
        bail!("advance count {count} is not plausible");
    }
    let mut advances = Vec::new();
    for _ in 0..count {
        advances.push((r.i32()?, r.u32()?));
    }

    let state = u64::from_le_bytes(r.bytes(8)?.try_into().expect("8 bytes"));
    let inc = u64::from_le_bytes(r.bytes(8)?.try_into().expect("8 bytes"));
    let spare = u32::from_le_bytes(r.bytes(4)?.try_into().expect("4 bytes"));
    if spare != NO_SPARE {
        // v1 never draws a spare; an unknown sentinel means the writer and this
        // reader disagree about the RNG state format.
        bail!("rng spare is {spare}, not the v1 no-spare sentinel");
    }
    if !r.is_empty() {
        bail!("snapshot payload has {} trailing bytes", r.remaining());
    }
    Ok((
        rows,
        edges,
        stamps,
        handles,
        interner,
        points,
        a,
        settle,
        trajectory,
        RngState { state, inc, spare },
        advances,
    ))
}

fn take_arena(r: &mut Reader<'_>, record_bytes: usize) -> Result<Arena> {
    let len = r.u32()? as usize;
    if len > r.remaining() {
        bail!("arena length {len} runs past the payload");
    }
    Arena::from_bytes(record_bytes, r.bytes(len)?.to_vec())
}

fn core_to_value(core: &CoreBlock) -> Value {
    json!({
        "birth_seed": core.birth_seed,
        "seed_format": core.seed_format,
        "deriv_order_len": core.deriv_order_len,
        "temperament_digest": core.temperament_digest,
        "sleeps_completed": core.sleeps_completed,
        "sleeps_interrupted": core.sleeps_interrupted,
        "last_slept_at_t": core.last_slept_at_t,
        "sleep_pass_mask": core.sleep_pass_mask,
        "block_version": BLOCK_VERSION,
    })
}

fn core_from_value(value: &Value) -> Result<CoreBlock> {
    let object = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("core_block is not an object"))?;
    macro_rules! field {
        ($name:literal, $method:ident) => {
            object
                .get($name)
                .and_then(Value::$method)
                .ok_or_else(|| anyhow::anyhow!("core_block.{} is missing or the wrong type", $name))?
        };
    }
    let block_version = field!("block_version", as_u64) as u8;
    if object.get("block_version").and_then(Value::as_u64) != Some(BLOCK_VERSION as u64) {
        bail!("core_block.block_version {block_version:#04x} is not {BLOCK_VERSION:#04x}");
    }
    let core = CoreBlock {
        birth_seed: field!("birth_seed", as_u64),
        seed_format: field!("seed_format", as_u64) as u16,
        deriv_order_len: field!("deriv_order_len", as_u64) as u8,
        temperament_digest: field!("temperament_digest", as_u64),
        sleeps_completed: field!("sleeps_completed", as_u64) as u32,
        sleeps_interrupted: field!("sleeps_interrupted", as_u64) as u32,
        last_slept_at_t: field!("last_slept_at_t", as_i64) as i32,
        sleep_pass_mask: field!("sleep_pass_mask", as_u64) as u8,
    };
    // Re-encode and decode: this is where the pads, the version and the sealed
    // lengths are checked, exactly as a binary reader would check them.
    let decoded = CoreBlock::decode(&core.encode())?;
    if decoded != core {
        bail!("core_block fields do not round-trip through the 36-byte block");
    }
    Ok(core)
}

/// 00 §5: `name.tmp` → `sync_all` → `rename`. A staged write that is dropped
/// never touches the target, which is the property the atomic-save test reads.
pub struct StagedWrite {
    target: PathBuf,
    tmp: PathBuf,
    committed: bool,
}

impl StagedWrite {
    pub fn stage(path: &Path, bytes: &[u8]) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        let tmp = PathBuf::from(format!("{}.tmp", path.display()));
        let mut file = File::create(&tmp).with_context(|| format!("create {}", tmp.display()))?;
        file.write_all(bytes).with_context(|| format!("write {}", tmp.display()))?;
        file.sync_all().with_context(|| format!("sync {}", tmp.display()))?;
        Ok(Self { target: path.to_path_buf(), tmp, committed: false })
    }

    pub fn commit(mut self) -> Result<()> {
        fs::rename(&self.tmp, &self.target).with_context(|| {
            format!("rename {} to {}", self.tmp.display(), self.target.display())
        })?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for StagedWrite {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_file(&self.tmp);
        }
    }
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    StagedWrite::stage(path, bytes)?.commit()
}
