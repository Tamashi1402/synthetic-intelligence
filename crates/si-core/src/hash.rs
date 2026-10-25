//! 01 §10 / 04 §24 X.1: the state hash — nine item classes, in the recorded
//! order, chained as per-class sub-digests (OD-60's lean, 06 §7) and tagged
//! `HASH_FORMAT`. A caller supplies each class's canonical bytes; it cannot
//! supply a median, a calm view or a stored temperament vector, because no such
//! field exists in this API.

use md5::{Digest, Md5};

use crate::consts;
use crate::encode::{Reader, Writer};
use crate::rng::RngState;
use crate::seed;
use crate::temperament::{self, DERIV_ORDER_LEN};

/// 06 §7 / OD-60: the chained construction's format tag. Flipping it changes
/// every digest and therefore invalidates every stored `state_hash.txt` loudly.
pub const HASH_FORMAT: u16 = 0x0001;
pub const HASH_ITEM_COUNT: usize = 9;

/// 01 §13: the session core block, inside hash item 9.
pub const CORE_BLOCK_BYTES: usize = 36;
pub const BLOCK_VERSION: u8 = 0x01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreBlock {
    pub birth_seed: u64,
    pub seed_format: u16,
    pub deriv_order_len: u8,
    pub temperament_digest: u64,
    pub sleeps_completed: u32,
    pub sleeps_interrupted: u32,
    pub last_slept_at_t: i32,
    pub sleep_pass_mask: u8,
}

impl CoreBlock {
    pub fn at_birth(birth_seed: u64) -> Self {
        Self {
            birth_seed,
            seed_format: seed::SEED_FORMAT,
            deriv_order_len: DERIV_ORDER_LEN as u8,
            temperament_digest: seed::temperament_digest(birth_seed),
            sleeps_completed: 0,
            sleeps_interrupted: 0,
            last_slept_at_t: 0,
            sleep_pass_mask: 0,
        }
    }

    /// 01 §13's table, byte for byte. Pads are written as zero and checked on
    /// read; a reserved byte is checked, not trusted.
    pub fn encode(&self) -> [u8; CORE_BLOCK_BYTES] {
        let mut w = Writer::new();
        w.u64(self.birth_seed);
        w.u16(self.seed_format);
        w.u8(self.deriv_order_len);
        w.u8(0);
        w.u64(self.temperament_digest);
        w.u32(self.sleeps_completed);
        w.u32(self.sleeps_interrupted);
        w.i32(self.last_slept_at_t);
        w.u8(self.sleep_pass_mask);
        w.u16(0);
        w.u8(BLOCK_VERSION);
        debug_assert_eq!(w.len(), CORE_BLOCK_BYTES);
        w.finish().try_into().unwrap()
    }

    pub fn decode(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() != CORE_BLOCK_BYTES {
            anyhow::bail!("core block is {} bytes, want {}", bytes.len(), CORE_BLOCK_BYTES);
        }
        let mut r = Reader::new(bytes);
        let birth_seed = r.u64()?;
        let seed_format = r.u16()?;
        let deriv_order_len = r.u8()?;
        let pad = r.u8()?;
        let temperament_digest = r.u64()?;
        let sleeps_completed = r.u32()?;
        let sleeps_interrupted = r.u32()?;
        let last_slept_at_t = r.i32()?;
        let sleep_pass_mask = r.u8()?;
        let pad2 = r.u16()?;
        let block_version = r.u8()?;
        if pad != 0 || pad2 != 0 {
            anyhow::bail!("core block reserved bytes are not zero");
        }
        if block_version != BLOCK_VERSION {
            anyhow::bail!("core block version {block_version:#04x} is not {BLOCK_VERSION:#04x}");
        }
        if seed_format != seed::SEED_FORMAT {
            anyhow::bail!("core block seed_format {seed_format:#06x} is not {:#06x}", seed::SEED_FORMAT);
        }
        if deriv_order_len as usize != DERIV_ORDER_LEN {
            anyhow::bail!("core block deriv_order_len {deriv_order_len} is not {DERIV_ORDER_LEN}");
        }
        Ok(Self {
            birth_seed,
            seed_format,
            deriv_order_len,
            temperament_digest,
            sleeps_completed,
            sleeps_interrupted,
            last_slept_at_t,
            sleep_pass_mask,
        })
    }

    /// G14(c): re-derive from the seed and compare against the recorded digest.
    pub fn digest_matches_seed(&self) -> bool {
        self.temperament_digest == seed::temperament_digest(self.birth_seed)
    }
}

/// 04 X.1: item 8's version tags. The per-row list is in registry reading order
/// and carries each row's `ver` (`0x0001` until a sweep bumps it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionTags {
    pub hash_format: u16,
    pub consts_version: u16,
    pub modulation_curve_variant: u8,
    pub genome_t0_tag: u64,
    pub inference_max_hops_owned: u8,
}

impl VersionTags {
    pub fn at_birth(birth_seed: u64) -> Self {
        let temperament = seed::derive_temperament(birth_seed);
        let hops = consts::row("INFERENCE_MAX_HOPS_OWNED")
            .and_then(consts::registry_value)
            .expect("INFERENCE_MAX_HOPS_OWNED is a swept scalar SPECIES row") as u8;
        Self {
            hash_format: HASH_FORMAT,
            consts_version: consts::CONSTS_VERSION,
            modulation_curve_variant: temperament.get(temperament::DerivId::ModCurveVariant) as u8,
            genome_t0_tag: 0,
            inference_max_hops_owned: hops,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u16(self.hash_format);
        w.u16(self.consts_version);
        w.u8(self.modulation_curve_variant);
        w.u64(self.genome_t0_tag);
        w.u8(self.inference_max_hops_owned);
        w.u16(consts::CONST_ROWS.len() as u16);
        for row in consts::CONST_ROWS {
            w.str(row.name);
            w.u16(registry_version(row.name));
        }
        w.finish()
    }
}

/// Each registry row's own `ver`. A swept winner replaces its row's tag; the
/// table is a single place so a re-sweep cannot silently leave an old tag.
pub fn registry_version(name: &str) -> u16 {
    let _ = name;
    consts::CONSTS_VERSION
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HashItem {
    LiveRows,
    Edges,
    Stamps,
    Interner,
    SettleSet,
    Trajectory,
    Rng,
    VersionTags,
    CoreBlock,
}

impl HashItem {
    pub const ORDER: [HashItem; HASH_ITEM_COUNT] = [
        HashItem::LiveRows,
        HashItem::Edges,
        HashItem::Stamps,
        HashItem::Interner,
        HashItem::SettleSet,
        HashItem::Trajectory,
        HashItem::Rng,
        HashItem::VersionTags,
        HashItem::CoreBlock,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            HashItem::LiveRows => "live_rows",
            HashItem::Edges => "edges",
            HashItem::Stamps => "stamps",
            HashItem::Interner => "interner",
            HashItem::SettleSet => "settle_set",
            HashItem::Trajectory => "trajectory",
            HashItem::Rng => "rng",
            HashItem::VersionTags => "version_tags",
            HashItem::CoreBlock => "core_block",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HashInputs {
    pub live_rows: Vec<u8>,
    pub edges: Vec<u8>,
    pub stamps: Vec<u8>,
    pub interner: Vec<u8>,
    pub settle_set: Vec<u8>,
    pub trajectory: Vec<u8>,
    pub rng: Vec<u8>,
    pub version_tags: Vec<u8>,
    pub core_block: Vec<u8>,
}

impl HashInputs {
    pub fn class(&self, item: HashItem) -> &[u8] {
        match item {
            HashItem::LiveRows => &self.live_rows,
            HashItem::Edges => &self.edges,
            HashItem::Stamps => &self.stamps,
            HashItem::Interner => &self.interner,
            HashItem::SettleSet => &self.settle_set,
            HashItem::Trajectory => &self.trajectory,
            HashItem::Rng => &self.rng,
            HashItem::VersionTags => &self.version_tags,
            HashItem::CoreBlock => &self.core_block,
        }
    }

    /// A freshly created session: no rows, no edges, no stamps, no settle-set,
    /// no trajectory; stream R at its birth state; the core block of
    /// `birth_seed`. This is the empty-state value the B0 test pins.
    pub fn empty_at_birth(birth_seed: u64) -> Self {
        let runtime = seed::runtime_stream(birth_seed).state();
        Self {
            live_rows: Vec::new(),
            edges: Vec::new(),
            stamps: Vec::new(),
            interner: encode_empty_interner(),
            settle_set: encode_settle_set(&[]),
            trajectory: encode_trajectory(&[]),
            rng: encode_rng(&runtime, &[]),
            version_tags: VersionTags::at_birth(birth_seed).encode(),
            core_block: CoreBlock::at_birth(birth_seed).encode().to_vec(),
        }
    }
}

/// Item 4 at birth: the reserved id `0 = ""` is always in the table (01 §7), so
/// the empty interner's canonical bytes are `count = 1` and one empty string —
/// not an empty byte run.
pub fn encode_empty_interner() -> Vec<u8> {
    let mut w = Writer::new();
    w.u32(1);
    w.str("");
    w.finish()
}

/// Item 7: `RngState{state, inc, spare}` then the advance history as
/// `(t, draws)` pairs, event-ordered. The history is evidence, the state is the
/// law; both are hashed so a replay that draws the same count at different steps
/// cannot hide.
pub fn encode_rng(state: &RngState, advances: &[(i32, u32)]) -> Vec<u8> {
    let mut w = Writer::new();
    w.bytes(&state.canonical_bytes());
    w.u32(advances.len() as u32);
    for (t, draws) in advances {
        w.i32(*t);
        w.u32(*draws);
    }
    w.finish()
}

/// Item 5: `{(point, quantized_activation)}`, sorted by point id. Rendering
/// order (activation desc) is deliberately not hashable.
pub fn encode_settle_set(entries: &[(u32, u16)]) -> Vec<u8> {
    let mut sorted = entries.to_vec();
    sorted.sort_unstable_by_key(|(point, _)| *point);
    let mut w = Writer::new();
    w.u32(sorted.len() as u32);
    for (point, activation) in sorted {
        w.u32(point);
        w.u16(activation);
        w.u16(0);
    }
    w.finish()
}

/// Item 6: the per-thought `a` trajectory, one `(round, a)` per signal event in
/// emission order (01 §8). A round number that decreases marks a thought
/// boundary; the encoding invents no thought-id field.
pub fn encode_trajectory(entries: &[(u16, u8)]) -> Vec<u8> {
    let mut w = Writer::new();
    w.u32(entries.len() as u32);
    for (round, a) in entries {
        w.u16(*round);
        w.u8(*a);
        w.u8(0);
    }
    w.finish()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateHash {
    digest: [u8; 16],
    sub: [[u8; 16]; HASH_ITEM_COUNT],
}

impl StateHash {
    /// 04 X.1 / 06 §7: per-class sub-digests, chained in item order.
    pub fn compose(inputs: &HashInputs) -> Self {
        let mut sub = [[0u8; 16]; HASH_ITEM_COUNT];
        for (index, item) in HashItem::ORDER.iter().enumerate() {
            sub[index].copy_from_slice(Md5::digest(inputs.class(*item)).as_slice());
        }
        let mut chain = Md5::new();
        for digest in &sub {
            chain.update(digest);
        }
        let mut digest = [0u8; 16];
        digest.copy_from_slice(chain.finalize().as_slice());
        Self { digest, sub }
    }

    pub fn digest(&self) -> [u8; 16] {
        self.digest
    }

    pub fn sub_digest(&self, item: HashItem) -> [u8; 16] {
        self.sub[HashItem::ORDER.iter().position(|i| *i == item).expect("sealed item")]
    }

    pub fn hex(&self) -> String {
        hex(&self.digest)
    }
}

pub fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// The empty-state value (birth seed `0`, no session activity). Any change to
/// a hashed constant tag, the core block, the birth RNG or the chained
/// construction changes this string, which is the point of pinning it.
///
/// B1 correction: B0 pinned `8e76…` from a fixture that omitted item 4's
/// reserved `0 = ""` entry and items 5–6's list count prefixes — encodings no
/// live session can produce. The construction and `HASH_FORMAT` are unchanged;
/// this value is what `Session::new(0)` and this fixture now both yield.
///
/// B14 re-pin (engineering/12 §3.2): `CONSTS_VERSION` moves `0x0001` →
/// `0x0002` because the arc's twenty-two rows join the registry; item 3 feeds
/// the version into the digest, so the empty state moves with it. Nine items,
/// `HASH_FORMAT` and the construction are unchanged — this is the one hash move
/// the arc is allowed, and a second one would be a schema change.
///
/// B20 re-pin (engineering/13 §3.4): `0x0002` → `0x0003` because the tongue's
/// eight rows join the registry, so item 8's row count and version tag move for
/// **every** run. The three asserting tests (`si-core/tests/kernel.rs`,
/// `si-session/tests/{seed,wake}.rs`) read this constant and re-pin with it, in
/// the same commit as the bump — a bump and its goldens in two commits is a
/// build that is red in the history. `HASH_FORMAT` stays `0x0001` and the voice
/// itself moves no item (13 §0 fact 5): the three tests are the whole ripple.
pub const EMPTY_STATE_MD5_SEED_ZERO: &str = "424cedca347884710a62ca15e28e0399";
