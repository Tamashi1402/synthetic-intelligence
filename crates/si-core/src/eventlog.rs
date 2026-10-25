//! 01 §9: the framed stream record and the event log. A stream frame is one
//! `SIro` record `{seq u64, t i32, source u8, kind u8, body}`; the loop never
//! blocks on a reader (OD-48).

use anyhow::{bail, Result};

/// 08 §2.1: the output lane's frame-kind table, named once.
///
/// It lives in `si-core` because that is the only crate both writers of the lane
/// may share: the loop reaches it through `si-session`, and process B may import
/// nothing else ([00 §2]'s one edge, [02 §10]'s "may never import `si-learn`,
/// `si-read`, `si-walk`"). A viewer that re-typed the numbers would be a second
/// table, and 08 §11's whole argument for kinds 19 and 20 is that a panel and a
/// text file cannot be allowed to disagree.
///
/// `kind` is presentation-only: [01 §10]'s nine hashed item classes contain no
/// event log, so emitting a frame and renaming one move zero hashed bytes —
/// which is what makes this module an addition rather than a re-baseline.
pub mod kind {
    pub const SENSE_ECHO: u8 = 0;
    pub const DECODED: u8 = 1;
    pub const LIT_ROUND0: u8 = 2;
    pub const ROUND: u8 = 3;
    pub const SETTLE: u8 = 4;
    pub const SIGNAL_EVENT: u8 = 5;
    pub const A_UPDATE: u8 = 6;
    pub const MEDIAN_USED: u8 = 7;
    pub const GAP: u8 = 8;
    pub const DOOR_CALL: u8 = 9;
    pub const ASK: u8 = 10;
    pub const LABEL: u8 = 11;
    pub const OUTPUT: u8 = 12;
    pub const COMMIT: u8 = 13;
    pub const SUPERSEDE: u8 = 14;
    pub const STAGE: u8 = 15;
    pub const TICK_END: u8 = 16;
    pub const SEAM_REJECT: u8 = 17;
    /// Reserved, never emitted in v1: the harness-only counterfactual probe
    /// (08 §9's forbidden-button row, **OD-67**). A parser must refuse it, not
    /// draw a panel for it — that is B11 DoD 1's third assert.
    pub const PROBE_RESULT: u8 = 18;
    pub const SLEEP_PASS: u8 = 19;
    pub const TEMPERAMENT_READOUT: u8 = 20;
    /// **OD-143**: 08 §2.1's table is sealed at 0…20 and has no slot for the
    /// idle-ignition record, which B7 shipped and 12 §2.5's five-term weights are
    /// the only witness of. §2.1's own note refuses the alternative — no "this
    /// was a dream" flag on kinds 3/4/6/13 — so the record takes the first slot
    /// above the sealed table rather than displacing `seam_reject`, which the
    /// table does name and CHAT renders.
    pub const IGNITION: u8 = 21;

    /// The highest kind 08 §2.1 names. `ignition` sits above it and says so.
    pub const LAST_SEALED: u8 = 20;

    /// `(kind, name)` in table order, reserved row included: what a parser and a
    /// check both read, so "handles kinds 0–20" is one list rather than two.
    pub const SEALED: [(u8, &'static str); 21] = [
        (SENSE_ECHO, "sense_echo"),
        (DECODED, "decoded"),
        (LIT_ROUND0, "lit_round0"),
        (ROUND, "round"),
        (SETTLE, "settle"),
        (SIGNAL_EVENT, "signal_event"),
        (A_UPDATE, "a_update"),
        (MEDIAN_USED, "median_used"),
        (GAP, "gap"),
        (DOOR_CALL, "door_call"),
        (ASK, "ask"),
        (LABEL, "label"),
        (OUTPUT, "output"),
        (COMMIT, "commit"),
        (SUPERSEDE, "supersede"),
        (STAGE, "stage"),
        (TICK_END, "tick_end"),
        (SEAM_REJECT, "seam_reject"),
        (PROBE_RESULT, "probe_result (reserved, never emitted)"),
        (SLEEP_PASS, "sleep_pass"),
        (TEMPERAMENT_READOUT, "temperament_readout"),
    ];

    pub fn name(kind: u8) -> Option<&'static str> {
        if kind == IGNITION {
            return Some("ignition (OD-143)");
        }
        SEALED.iter().find(|(id, _)| *id == kind).map(|(_, name)| *name)
    }

    /// 08 §2.1's one refusal: kind 18 has a name and no producer in v1.
    pub fn is_reserved(kind: u8) -> bool {
        kind == PROBE_RESULT
    }
}

pub const EVENT_HEADER_BYTES: usize = 14;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub seq: u64,
    pub t: i32,
    pub source: u8,
    pub kind: u8,
    pub body: Vec<u8>,
}

impl Event {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(EVENT_HEADER_BYTES + self.body.len());
        out.extend_from_slice(&self.seq.to_le_bytes());
        out.extend_from_slice(&self.t.to_le_bytes());
        out.push(self.source);
        out.push(self.kind);
        out.extend_from_slice(&self.body);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < EVENT_HEADER_BYTES {
            bail!("event frame shorter than its header");
        }
        Ok(Self {
            seq: u64::from_le_bytes(bytes[0..8].try_into().expect("8 bytes")),
            t: i32::from_le_bytes(bytes[8..12].try_into().expect("4 bytes")),
            source: bytes[12],
            kind: bytes[13],
            body: bytes[EVENT_HEADER_BYTES..].to_vec(),
        })
    }
}

/// 01 §9 / 12 §2.1: the arc's one transport byte-pair. From B18 a **sense** frame's
/// body is `speaker_identity u64le | conversation u32le | text`; every other kind
/// keeps its bare body, which is why the prefix is a codec and not a header field —
/// the 14-byte record header is [01 §9]'s and does not move.
pub const SENSE_TRANSPORT_BYTES: usize = 12;

/// The identity a frame's writer has no other name for: the literal's own index
/// (`owner` 0, `grimm` 1, [01 §3.2]'s two voices). It is the identity no B18 body
/// may carry, because `u64::MAX` is what a *pre-B18* body's first eight bytes
/// decode to on average; the discriminator is the container's version, and this
/// value is reserved so a guess can never look like a real tag.
pub const IDENTITY_UNTAGGED: u64 = u64::MAX;

pub fn encode_sense_body(
    text: &[u8],
    speaker_identity: u64,
    conversation: u32,
) -> Result<Vec<u8>> {
    if speaker_identity == IDENTITY_UNTAGGED {
        bail!("a sense body cannot carry the untagged identity");
    }
    if text.is_empty() {
        bail!("a sense body is its transport prefix and a word; an empty line is not sense");
    }
    let mut out = Vec::with_capacity(SENSE_TRANSPORT_BYTES + text.len());
    out.extend_from_slice(&speaker_identity.to_le_bytes());
    out.extend_from_slice(&conversation.to_le_bytes());
    out.extend_from_slice(text);
    Ok(out)
}

/// The reader's half: the identity, the interned thread id and the text.
///
/// The container's version is what tells a B18 body from a pre-B18 one; this
/// guard is the weaker, structural half. It refuses a body that cannot carry the
/// pair **and a word** — a sense line is never empty ([05 §1]'s grammar bails on
/// one), so a body of exactly the prefix's length is not a sentence and would
/// otherwise be read as an identity and a thread made of its own letters.
pub fn decode_sense_body(body: &[u8]) -> Result<(u64, u32, Vec<u8>)> {
    if body.len() <= SENSE_TRANSPORT_BYTES {
        bail!(
            "a B18 sense body is no longer than its {SENSE_TRANSPORT_BYTES}-byte transport \
             prefix: an inbox written before the version bump"
        );
    }
    Ok((
        u64::from_le_bytes(body[0..8].try_into().expect("8 bytes")),
        u32::from_le_bytes(body[8..12].try_into().expect("4 bytes")),
        body[SENSE_TRANSPORT_BYTES..].to_vec(),
    ))
}

#[derive(Clone, Debug, Default)]
pub struct EventLog {
    events: Vec<Event>,
}

impl EventLog {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn append(&mut self, t: i32, source: u8, kind: u8, body: Vec<u8>) -> u64 {
        let seq = self.events.len() as u64;
        self.events.push(Event { seq, t, source, kind, body });
        seq
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// 01 §10 item 6 reads the event-ordered byte stream; the log owns no time.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for event in &self.events {
            out.extend_from_slice(&event.encode());
        }
        out
    }
}
