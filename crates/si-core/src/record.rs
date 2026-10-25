//! 01 §4–§5: the 80-byte envelope, its 32-byte body, the 16-byte `z` edge and
//! the 16-byte charge stamp. Reserved bytes are written as zero and checked on
//! read: a reserved byte is asserted, never trusted.

use anyhow::{bail, Result};

use crate::addr::{edge_kind_is_owned, EDGE_KIND_COOCCUR_WEAK};
use crate::encode::{Reader, Writer};

pub const ENVELOPE_BYTES: usize = 80;
pub const BODY_BYTES: usize = 32;
pub const RECORD_BYTES: usize = ENVELOPE_BYTES + BODY_BYTES;
pub const EDGE_BYTES: usize = 16;
pub const STAMP_BYTES: usize = 16;

/// 01 §11: `i32::MAX` is "live", so `active_at` is one comparison.
pub const SUPERSEDED_LIVE: i32 = i32::MAX;
/// 01 §11: arousal at ask has no "rest" ambiguity, so it gets its own null.
pub const AROUSAL_NONE: u8 = 255;
/// 01 §11 / 12 §2.5: the body's `pattern_id` sentinel for "nothing names this
/// yet". `0` is a real pattern index, so it cannot serve; an addressable,
/// enumerable, reason-over-able row that the renderer must refuse to print.
pub const PATTERN_NAMELESS: u16 = 0xFFFF;
/// 12 §2.5: the per-shelf row-body kind bytes the arc adds — `links` kind 8 and
/// `skills` kind 7. They are a different enum from 03 §5.2's seven **edge**
/// kinds, which is why `EDGE_KIND_COUNT` does not move.
pub const LINK_KIND_CLUSTER_POINT: u16 = 8;
pub const SKILL_KIND_SHAPE: u16 = 7;
/// The only row schema version v1 licenses.
pub const ROW_VERSION_V1: u16 = 0x0001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Envelope {
    pub row_id: u32,
    pub shelf_x: u16,
    pub row_version: u16,
    pub written_at_t: i32,
    pub last_confirmed_at: i32,
    pub superseded_at: i32,
    pub superseded_by: u32,
    pub repeats: u32,
    pub confidence: u16,
    pub gain_exact: i16,
    pub gain_kind: u8,
    pub prov_kind: u8,
    pub prov_speaker: u8,
    pub stage: u8,
    pub bind_output_id: u32,
    pub question_id: u32,
    pub prov_session: u32,
    pub prov_step: i32,
    pub text_ref: u32,
    pub edges_off: u32,
    pub charge_off: u32,
    pub derived_off: u32,
    pub derived_len: u16,
    pub edges_len: u16,
    pub terms_off: u32,
    pub terms_len: u16,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            row_id: 0,
            shelf_x: 0,
            row_version: ROW_VERSION_V1,
            written_at_t: 0,
            last_confirmed_at: 0,
            superseded_at: SUPERSEDED_LIVE,
            superseded_by: 0,
            repeats: 0,
            confidence: 0,
            gain_exact: 0,
            gain_kind: 0,
            prov_kind: 1,
            prov_speaker: 0,
            stage: 0,
            bind_output_id: 0,
            question_id: 0,
            prov_session: 0,
            prov_step: 0,
            text_ref: 0,
            edges_off: 0,
            charge_off: 0,
            derived_off: 0,
            derived_len: 0,
            edges_len: 0,
            terms_off: 0,
            terms_len: 0,
        }
    }
}

impl Envelope {
    pub fn live(shelf_x: u16, written_at_t: i32, prov_session: u32, prov_step: i32) -> Self {
        Self {
            shelf_x,
            written_at_t,
            prov_session,
            prov_step,
            ..Self::default()
        }
    }

    pub fn is_live(&self) -> bool {
        self.superseded_at == SUPERSEDED_LIVE
    }

    /// `active_at(s)`: `written_at ≤ s < superseded_at ∨ ∞` (01 §4.1).
    pub fn active_at(&self, anchor: i32) -> bool {
        self.written_at_t <= anchor && (self.superseded_at == SUPERSEDED_LIVE || anchor < self.superseded_at)
    }

    pub fn encode(&self) -> [u8; ENVELOPE_BYTES] {
        let mut w = Writer::new();
        w.u32(self.row_id);
        w.u16(self.shelf_x);
        w.u16(self.row_version);
        w.i32(self.written_at_t);
        w.i32(self.last_confirmed_at);
        w.i32(self.superseded_at);
        w.u32(self.superseded_by);
        w.u32(self.repeats);
        w.u16(self.confidence);
        w.u16(self.gain_exact as u16);
        w.u8(self.gain_kind);
        w.u8(self.prov_kind);
        w.u8(self.prov_speaker);
        w.u8(self.stage);
        w.u32(self.bind_output_id);
        w.u32(self.question_id);
        w.u32(self.prov_session);
        w.i32(self.prov_step);
        w.u32(self.text_ref);
        w.u32(self.edges_off);
        w.u32(self.charge_off);
        w.u32(self.derived_off);
        w.u16(self.derived_len);
        w.u16(self.edges_len);
        w.u32(self.terms_off);
        w.u16(self.terms_len);
        w.u16(0);
        debug_assert_eq!(w.len(), ENVELOPE_BYTES);
        w.finish().try_into().unwrap()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != ENVELOPE_BYTES {
            bail!("envelope is {} bytes, want {ENVELOPE_BYTES}", bytes.len());
        }
        let mut r = Reader::new(bytes);
        let envelope = Self {
            row_id: r.u32()?,
            shelf_x: r.u16()?,
            row_version: r.u16()?,
            written_at_t: r.i32()?,
            last_confirmed_at: r.i32()?,
            superseded_at: r.i32()?,
            superseded_by: r.u32()?,
            repeats: r.u32()?,
            confidence: r.u16()?,
            gain_exact: r.u16()? as i16,
            gain_kind: r.u8()?,
            prov_kind: r.u8()?,
            prov_speaker: r.u8()?,
            stage: r.u8()?,
            bind_output_id: r.u32()?,
            question_id: r.u32()?,
            prov_session: r.u32()?,
            prov_step: r.i32()?,
            text_ref: r.u32()?,
            edges_off: r.u32()?,
            charge_off: r.u32()?,
            derived_off: r.u32()?,
            derived_len: r.u16()?,
            edges_len: r.u16()?,
            terms_off: r.u32()?,
            terms_len: r.u16()?,
        };
        if r.u16()? != 0 {
            bail!("envelope reserved u16 is not zero");
        }
        if envelope.row_version != ROW_VERSION_V1 {
            bail!("row_version {:#06x} is not {ROW_VERSION_V1:#06x}", envelope.row_version);
        }
        Ok(envelope)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Body {
    pub pattern_id: u16,
    pub key_field: u16,
    pub key_point: u32,
    pub val_point: u32,
    pub stim_signature: u32,
    pub stim_surface: u32,
    pub resp_surface: u32,
    pub target_slot: u16,
    pub arousal_at_ask: u8,
    pub status: u8,
}

impl Body {
    pub fn encode(&self) -> [u8; BODY_BYTES] {
        let mut w = Writer::new();
        w.u16(self.pattern_id);
        w.u16(self.key_field);
        w.u32(self.key_point);
        w.u32(self.val_point);
        w.u32(self.stim_signature);
        w.u32(self.stim_surface);
        w.u32(self.resp_surface);
        w.u16(self.target_slot);
        w.u8(self.arousal_at_ask);
        w.u8(self.status);
        w.u32(0);
        debug_assert_eq!(w.len(), BODY_BYTES);
        w.finish().try_into().unwrap()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != BODY_BYTES {
            bail!("body is {} bytes, want {BODY_BYTES}", bytes.len());
        }
        let mut r = Reader::new(bytes);
        let body = Self {
            pattern_id: r.u16()?,
            key_field: r.u16()?,
            key_point: r.u32()?,
            val_point: r.u32()?,
            stim_signature: r.u32()?,
            stim_surface: r.u32()?,
            resp_surface: r.u32()?,
            target_slot: r.u16()?,
            arousal_at_ask: r.u8()?,
            status: r.u8()?,
        };
        if r.u32()? != 0 {
            bail!("body reserved u32 is not zero");
        }
        Ok(body)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    pub envelope: Envelope,
    pub body: Body,
}

impl Record {
    pub fn encode(&self) -> [u8; RECORD_BYTES] {
        let mut out = [0u8; RECORD_BYTES];
        out[..ENVELOPE_BYTES].copy_from_slice(&self.envelope.encode());
        out[ENVELOPE_BYTES..].copy_from_slice(&self.body.encode());
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != RECORD_BYTES {
            bail!("record is {} bytes, want {RECORD_BYTES}", bytes.len());
        }
        Ok(Self {
            envelope: Envelope::decode(&bytes[..ENVELOPE_BYTES])?,
            body: Body::decode(&bytes[ENVELOPE_BYTES..])?,
        })
    }
}

/// 01 §5.2 / 04 §21 U.2 / 05 §20 `check_candidate_unanswerable`: the one
/// predicate every cited-row set shares. A `6 cooccur-weak` candidate and a
/// `confidence = 0` row are never answerable, whichever way OD-58 lands; a row
/// that is not active at the read anchor is not a belief at that anchor.
///
/// `primary_kind` is the kind of the live edge that would cite the row (for a
/// `links` row, its own edge); a row with no live edge is passed as
/// `EDGE_KIND_COOCCUR_WEAK` by the reader and is therefore never a claim.
pub fn answerable_meta(confidence: u16, primary_kind: u8, active: bool) -> bool {
    active && confidence > 0 && primary_kind != EDGE_KIND_COOCCUR_WEAK
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Edge {
    pub to_point: u32,
    pub kind: u8,
    pub flags: u8,
    pub weight: u16,
    pub bake: u16,
    pub charge_off: u32,
}

impl Edge {
    pub fn is_owned(&self) -> bool {
        edge_kind_is_owned(self.kind)
    }

    /// A superseded edge is weight 0 and leaves `deg` (06 §4 trap 1).
    pub fn is_live(&self) -> bool {
        self.is_owned() && self.weight > 0
    }

    pub fn encode(&self) -> [u8; EDGE_BYTES] {
        let mut w = Writer::new();
        w.u32(self.to_point);
        w.u8(self.kind);
        w.u8(self.flags);
        w.u16(self.weight);
        w.u16(self.bake);
        w.u16(0);
        w.u32(self.charge_off);
        debug_assert_eq!(w.len(), EDGE_BYTES);
        w.finish().try_into().unwrap()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != EDGE_BYTES {
            bail!("edge is {} bytes, want {EDGE_BYTES}", bytes.len());
        }
        let mut r = Reader::new(bytes);
        let to_point = r.u32()?;
        let kind = r.u8()?;
        let flags = r.u8()?;
        let weight = r.u16()?;
        let bake = r.u16()?;
        if r.u16()? != 0 {
            bail!("edge pad is not zero");
        }
        let edge = Self { to_point, kind, flags, weight, bake, charge_off: r.u32()? };
        if !edge.is_owned() && edge.bake != 0 {
            // A frozen edge has no bake column at all; kind 7+ carrying one is
            // a schema lie rather than a policy violation (01 §5.1, 07 §9).
            bail!("edge kind {} is frozen-genome but carries a bake", edge.kind);
        }
        Ok(edge)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stamp {
    pub thought_id: u32,
    pub signal_ref: u32,
    pub magnitude: u16,
    pub a_at: u8,
    pub signal: u8,
    pub src: u8,
}

impl Stamp {
    pub fn encode(&self) -> [u8; STAMP_BYTES] {
        let mut w = Writer::new();
        w.u32(self.thought_id);
        w.u32(self.signal_ref);
        w.u16(self.magnitude);
        w.u8(self.a_at);
        w.u8(self.signal);
        w.u8(self.src);
        w.u16(0);
        w.u8(0);
        debug_assert_eq!(w.len(), STAMP_BYTES);
        w.finish().try_into().unwrap()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != STAMP_BYTES {
            bail!("stamp is {} bytes, want {STAMP_BYTES}", bytes.len());
        }
        let mut r = Reader::new(bytes);
        let stamp = Self {
            thought_id: r.u32()?,
            signal_ref: r.u32()?,
            magnitude: r.u16()?,
            a_at: r.u8()?,
            signal: r.u8()?,
            src: r.u8()?,
        };
        if r.u16()? != 0 || r.u8()? != 0 {
            bail!("stamp pads are not zero");
        }
        Ok(stamp)
    }
}
