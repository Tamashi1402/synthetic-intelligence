//! 06 §1: the append-only byte arenas. A record arena holds fixed-width
//! records; every `*_off` column is a `u32` byte offset, never an index.

use anyhow::{bail, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arena {
    record_bytes: usize,
    bytes: Vec<u8>,
}

impl Arena {
    pub fn new(record_bytes: usize) -> Self {
        assert!(record_bytes > 0 && record_bytes % 4 == 0, "arena record length must be 4-aligned");
        Self { record_bytes, bytes: Vec::new() }
    }

    pub fn from_bytes(record_bytes: usize, bytes: Vec<u8>) -> Result<Self> {
        if record_bytes == 0 || record_bytes % 4 != 0 {
            bail!("arena record length {record_bytes} is not 4-aligned");
        }
        if bytes.len() % record_bytes != 0 {
            bail!("arena length {} is not a multiple of {record_bytes}", bytes.len());
        }
        Ok(Self { record_bytes, bytes })
    }

    pub fn record_bytes(&self) -> usize {
        self.record_bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn count(&self) -> usize {
        self.bytes.len() / self.record_bytes
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// The only append primitive. A wrong length or a non-4-aligned offset is a
    /// build bug, not a runtime state (02 §1).
    pub fn append(&mut self, record: &[u8]) -> Result<u32> {
        if record.len() != self.record_bytes {
            bail!("record is {} bytes, arena holds {}", record.len(), self.record_bytes);
        }
        let offset = self.bytes.len();
        if offset > u32::MAX as usize {
            bail!("arena offset {offset} exceeds the u32 address space");
        }
        self.bytes.extend_from_slice(record);
        Ok(offset as u32)
    }

    /// FLUID columns are rewritten in place at their existing address (06 §2's
    /// closed mutable set); the record length and offset cannot change.
    pub fn write(&mut self, offset: u32, record: &[u8]) -> Result<()> {
        let start = self.offset_index(offset)?;
        if record.len() != self.record_bytes {
            bail!("record is {} bytes, arena holds {}", record.len(), self.record_bytes);
        }
        self.bytes[start..start + self.record_bytes].copy_from_slice(record);
        Ok(())
    }

    pub fn read(&self, offset: u32) -> Result<&[u8]> {
        let start = self.offset_index(offset)?;
        Ok(&self.bytes[start..start + self.record_bytes])
    }

    pub fn read_list(&self, offset: u32, count: usize) -> Result<&[u8]> {
        if count == 0 {
            if offset != 0 {
                bail!("empty list at offset {offset}");
            }
            return Ok(&[]);
        }
        let start = self.offset_index(offset)?;
        let end = start
            .checked_add(count * self.record_bytes)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| anyhow::anyhow!("list at {offset}+{count} runs past the arena"))?;
        Ok(&self.bytes[start..end])
    }

    pub fn records(&self) -> impl Iterator<Item = &[u8]> {
        let width = self.record_bytes;
        self.bytes.chunks_exact(width)
    }

    fn offset_index(&self, offset: u32) -> Result<usize> {
        let offset = offset as usize;
        if offset % self.record_bytes != 0 {
            bail!("arena offset {offset} is not aligned to {}", self.record_bytes);
        }
        if offset + self.record_bytes > self.bytes.len() {
            bail!("arena offset {offset} is past the {} byte arena", self.bytes.len());
        }
        Ok(offset)
    }
}
