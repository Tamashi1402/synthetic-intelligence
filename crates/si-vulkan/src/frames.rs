//! 08 §2: the transport contract, read.
//!
//! A lane is one `SIro` container whose payload is repeated
//! `frame_len u32le | Event{seq u64, t i32, source u8, kind u8, body}`
//! ([01 §9]). The viewer decodes it and never re-encodes it except into
//! `sense_inbox.siro` ([`crate::WRITABLE_FILE`]).
//!
//! Kind validation lives here rather than in each panel, because 08 §2.1's table
//! is one contract with 21 rows and one refusal — kind 18 has a name and no
//! producer in v1, and a parser that *drew* it would be a UI promising a probe
//! the harness keeps (**OD-67**).

use anyhow::{bail, Context, Result};
use si_core::container::{Container, Format};
use si_core::eventlog::{kind, Event, EVENT_HEADER_BYTES};

/// One frame of the owner-facing lane, with its body kept as the string the loop
/// wrote. The body is JSON in every kind, and a panel reads the numbers out of it
/// rather than out of a lattice it has no handle to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub seq: u64,
    pub t: i32,
    pub source: u8,
    pub kind: u8,
    pub body: String,
}

impl Frame {
    /// 08 §2.1's row name for this frame, or `None` for a kind above the table.
    pub fn kind_name(&self) -> Option<&'static str> {
        kind::name(self.kind)
    }

    /// The body as a document. Kinds 19 and 20 forward another artifact's line
    /// verbatim ([08 §11]), so a lane whose body does not parse is a broken
    /// forward, and the panel says so instead of drawing nothing.
    pub fn json(&self) -> Result<serde_json::Value> {
        serde_json::from_str(&self.body)
            .with_context(|| format!("frame {} of kind {} is not JSON", self.seq, self.kind))
    }

    /// `body.field` as a quantized integer, when the panel is allowed to draw it.
    pub fn int(&self, field: &str) -> Option<i64> {
        self.json().ok().and_then(|v| v[field].as_i64())
    }

    /// `body.field` as text. Used for forwarded payloads only — a label, a
    /// surface, a reason code — never for a word the viewer knows on its own
    /// ([`crate::NO_VOCABULARY`]).
    pub fn text(&self, field: &str) -> Option<String> {
        self.json()
            .ok()
            .and_then(|v| v[field].as_str().map(str::to_string))
    }
}

/// A decoded lane: the container's own header plus its frames in order.
#[derive(Clone, Debug)]
pub struct Lane {
    pub header: serde_json::Value,
    pub frames: Vec<Frame>,
    /// Bytes the tail read consumed. §2.2's lag counter counts frames the lane
    /// holds that the panel has not drawn yet, so the reader keeps its cursor.
    pub next_byte: usize,
}

impl Lane {
    /// Decode a whole lane and validate every kind in it. A reserved or unknown
    /// kind is a refusal of the **lane**, not a skipped frame: §2.1's table is a
    /// contract, and a panel that quietly dropped row 18 would be the place a
    /// counterfactual probe starts its life as a picture.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            // 08 §2.2's late-start row has a first case at its limit: a session that has
            // committed nothing has no container yet, and a viewer that refused to open
            // would be a viewer that cannot show an empty mind. Anything *shorter* than a
            // container is still a refusal below.
            return Ok(Self {
                header: serde_json::Value::Null,
                frames: Vec::new(),
                next_byte: 0,
            });
        }
        let container = Container::decode(bytes)?;
        if container.format != Format::StreamRecord {
            bail!(
                "a panel reads {} lanes; this is a {} container",
                Format::StreamRecord.name(),
                container.format.name()
            );
        }
        if container.version != Format::StreamRecord.container_version() {
            bail!(
                "lane version {} is not the version this build writes ({})",
                container.version,
                Format::StreamRecord.container_version()
            );
        }
        let mut frames = Vec::new();
        let mut pos = 0usize;
        while pos + 4 <= container.payload.len() {
            let len = u32::from_le_bytes(container.payload[pos..pos + 4].try_into().expect("4"))
                as usize;
            pos += 4;
            let end = pos
                .checked_add(len)
                .ok_or_else(|| anyhow::anyhow!("a lane frame length overflows its payload"))?;
            if end > container.payload.len() {
                bail!("a lane frame runs past its payload");
            }
            let event = Event::decode(&container.payload[pos..end])?;
            pos = end;
            validate_kind(event.kind)?;
            frames.push(Frame {
                seq: event.seq,
                t: event.t,
                source: event.source,
                kind: event.kind,
                body: String::from_utf8_lossy(&event.body).to_string(),
            });
        }
        Ok(Self { header: container.header, frames, next_byte: pos })
    }

    /// §2.2: a slow viewer tails less often and prints a lag counter. The frames
    /// behind the cursor are still data, never dropped — "the panel drops display
    /// frames, never data".
    pub fn frames_behind(&self, drawn_through: u64) -> usize {
        self.frames.iter().filter(|frame| frame.seq > drawn_through).count()
    }
}

/// 08 §2.1's one refusal, plus the table's own ceiling.
pub fn validate_kind(value: u8) -> Result<()> {
    if kind::is_reserved(value) {
        bail!(
            "kind {value} ({}) is reserved and never emitted in v1 — a frame that carries it \
             did not come from this build's loop (08 §9, OD-67)",
            kind::name(value).unwrap_or("unknown")
        );
    }
    // `ignition` sits above the sealed table with **OD-143** on its name, so the
    // ceiling is that record's slot and not `LAST_SEALED`.
    if value > kind::IGNITION {
        bail!("kind {value} is above 08 §2.1's table and the one OD-143 slot above it");
    }
    Ok(())
}

/// The one inbound shape: a keystroke, minted by the owner's own process.
/// Identity and thread are transport bytes ([01 §9], [12 §2.1]), never a guess
/// read out of the text.
pub struct Outbound {
    pub text: String,
    pub speaker_identity: u64,
    pub conversation: u32,
}

/// `u32le frame_len | Event` — the same framing [`Lane::decode`] reads, kept here
/// so the crate that writes the inbox and the crate that reads a lane agree.
pub fn frame_bytes(event: &Event) -> Vec<u8> {
    let body = event.encode();
    let mut out = Vec::with_capacity(4 + body.len());
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&body);
    out
}

/// Build one sense frame for the inbox: source byte `0` (owner, [00 §7]) and the
/// B18 transport prefix, so a pre-B18 inbox cannot be mistaken for this one.
pub fn sense_frame(step: u64, t: i32, line: &Outbound) -> Result<Event> {
    if line.text.is_empty() {
        bail!("an empty line is not sense (05 §1's grammar)");
    }
    Ok(Event {
        seq: step,
        t,
        source: crate::SOURCE_OWNER,
        kind: kind::SENSE_ECHO,
        body: si_core::eventlog::encode_sense_body(
            line.text.as_bytes(),
            line.speaker_identity,
            line.conversation,
        )?,
    })
}

/// The header [`crate::WRITABLE_FILE`] is written with: the lane's own name and
/// the run it belongs to, so a reader refuses a mismatched container by name.
pub fn inbox_header(run_id: &str) -> serde_json::Value {
    serde_json::json!({ "lane": "sense_inbox", "writer": "viewer", "run_id": run_id })
}

/// The 14-byte record header, re-exported so `tests/laws.rs` can prove the crate
/// frames records with the shared codec rather than its own idea of a header.
pub const FRAME_HEADER_BYTES: usize = EVENT_HEADER_BYTES;
