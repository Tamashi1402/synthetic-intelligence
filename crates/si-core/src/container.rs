//! 00 §5 / 01 §9: the self-describing container
//! `MAGIC(4) | version u32le | header_len u32le | header JSON | payload`.
//! The format is read from the magic, never the extension.

use anyhow::{bail, Result};
use serde_json::Value;

pub const CONTAINER_VERSION: u32 = 1;
/// 01 §9 / 12 §2.1: from B18 the framed `SIro` sense record's body carries
/// `speaker_identity u64le` + `conversation u32le`, so the *stream* container's
/// version is 2 and nothing else's is. A resume into an older-versioned inbox
/// refuses; no envelope byte, no row and no hash item moves.
pub const STREAM_CONTAINER_VERSION: u32 = 2;
/// The spec fixes a 4-byte MAGIC (`MAGIC(4) | version | header_len | header`);
/// the five format names are longer than that, so the magic is the name's first
/// four ASCII bytes and `Format::name()` keeps the full name for headers and logs.
pub const MAGIC_LATTICE: [u8; 4] = *b"SIla";
pub const MAGIC_GENOME: [u8; 4] = *b"SIge";
pub const MAGIC_CHECKPOINT: [u8; 4] = *b"SIck";
pub const MAGIC_STATE: [u8; 4] = *b"SIst";
pub const MAGIC_STREAM: [u8; 4] = *b"SIro";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Lattice,
    Genome,
    Checkpoint,
    StateSnapshot,
    StreamRecord,
}

impl Format {
    pub const fn magic(self) -> [u8; 4] {
        match self {
            Format::Lattice => MAGIC_LATTICE,
            Format::Genome => MAGIC_GENOME,
            Format::Checkpoint => MAGIC_CHECKPOINT,
            Format::StateSnapshot => MAGIC_STATE,
            Format::StreamRecord => MAGIC_STREAM,
        }
    }

    pub fn from_magic(magic: &[u8]) -> Option<Self> {
        match magic {
            m if m == MAGIC_LATTICE => Some(Format::Lattice),
            m if m == MAGIC_GENOME => Some(Format::Genome),
            m if m == MAGIC_CHECKPOINT => Some(Format::Checkpoint),
            m if m == MAGIC_STATE => Some(Format::StateSnapshot),
            m if m == MAGIC_STREAM => Some(Format::StreamRecord),
            _ => None,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Format::Lattice => "SIlat",
            Format::Genome => "SIgen",
            Format::Checkpoint => "SIck",
            Format::StateSnapshot => "SIst",
            Format::StreamRecord => "SIro",
        }
    }

    /// 01 §9: the version a reader of this format writes and the version it
    /// refuses anything else against. Only the stream format is ahead of
    /// `CONTAINER_VERSION`, because only the stream format carries the arc's
    /// transport byte.
    pub const fn container_version(self) -> u32 {
        match self {
            Format::StreamRecord => STREAM_CONTAINER_VERSION,
            _ => CONTAINER_VERSION,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Container {
    pub format: Format,
    pub version: u32,
    pub header: Value,
    pub payload: Vec<u8>,
}

impl Container {
    pub fn new(format: Format, header: Value, payload: Vec<u8>) -> Self {
        Self { format, version: format.container_version(), header, payload }
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        if !self.header.is_object() {
            bail!("container header must be a JSON object");
        }
        let header = serde_json::to_vec(&self.header)?;
        let mut out = Vec::with_capacity(12 + header.len() + self.payload.len());
        out.extend_from_slice(&self.format.magic());
        out.extend_from_slice(&self.version.to_le_bytes());
        out.extend_from_slice(&(header.len() as u32).to_le_bytes());
        out.extend_from_slice(&header);
        out.extend_from_slice(&self.payload);
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 12 {
            bail!("container shorter than its fixed prefix");
        }
        let format = Format::from_magic(&bytes[0..4])
            .ok_or_else(|| anyhow::anyhow!("unknown container magic"))?;
        let version = u32::from_le_bytes(bytes[4..8].try_into().expect("4 bytes"));
        let header_len = u32::from_le_bytes(bytes[8..12].try_into().expect("4 bytes")) as usize;
        let header_end = 12usize
            .checked_add(header_len)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| anyhow::anyhow!("container header length exceeds the file"))?;
        let header: Value = serde_json::from_slice(&bytes[12..header_end])?;
        if !header.is_object() {
            bail!("container header must be a JSON object");
        }
        Ok(Self { format, version, header, payload: bytes[header_end..].to_vec() })
    }
}
