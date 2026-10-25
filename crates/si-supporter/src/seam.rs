//! 02 §8 / docs/21 §21.2: the seam. Read-only tools plus exactly one emission,
//! and the emission is [`speakgate`]'s. This is the **only** place a
//! `source: grimm` line may be born, so it is also the only place a rejected
//! emission can be seen to have never happened.
//!
//! The transport is a frame per line on stdin and a frame per line on stdout —
//! 00 §7's process C reaching process A. Nothing here opens a lattice handle for
//! writing: the store arrives borrowed and is read, and the artifacts arrive as
//! bytes from the run directory.
//!
//! docs/21 §21.3's honesty boundary is that the filter is **stdlib and
//! auditable**, so this module formats its own frames with `format!` and takes no
//! serializer: one dependency fewer between the mouth and the lattice.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use si_store::store::Lattice;

use crate::speakgate::{self, Verdict};

/// The three states a G12 run launches at (05 §2's variant table). They are
/// **launch states of the seam**, not code paths inside SI: V4 never gets a
/// reader, V5 gets readers and no mouth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeamState {
    /// V4 — process C never started. Nothing arrives, so nothing can be admitted.
    Closed,
    /// V5 — connected but silent: the tools are called, zero emissions arrive.
    Silent,
    /// V0 — the shipped seam: emissions are attempted and gated.
    Open,
}

impl SeamState {
    pub fn from_variant(variant: &str) -> Self {
        match variant {
            "V4" => SeamState::Closed,
            "V5" => SeamState::Silent,
            _ => SeamState::Open,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SeamState::Closed => "closed",
            SeamState::Silent => "connected-silent",
            SeamState::Open => "open",
        }
    }

    /// Does an emission attempt reach the gate at all? A closed or silent seam
    /// drops the line **before** the gate, which is why both leave SI's state
    /// byte-identical to a run in which he was never installed.
    pub fn admits(self) -> bool {
        matches!(self, SeamState::Open)
    }
}

/// What the seam did with one emission attempt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Admission {
    /// The gate passed it; the caller may now feed the line to the ordinary tick
    /// path as `source: grimm`. This is the only route to a grimm sense event.
    Admitted(Verdict),
    /// The gate rejected it. Zero state bytes, one gate-log line.
    Rejected(Verdict),
    /// The seam was closed or silent, so the attempt never arrived. SI cannot
    /// tell this from a run with no supporter in the process, which is the point.
    Dropped,
}

/// One attempt, in the machine-readable shape 05 §17 `G12(b).1` names. The
/// reason codes and the counts are the contract; the line is logged verbatim
/// because the rephrasing loop (§21.2) needs to see what he said.
#[derive(Clone, Debug)]
pub struct Attempt {
    pub t: i32,
    pub line: String,
    pub verdict: Option<Verdict>,
    /// A closed or silent seam dropped the attempt before the gate ran.
    pub dropped: bool,
}

impl Attempt {
    /// The gate log's one line for this attempt, in the shape 05 §17 `G12(b).1`
    /// names: reason codes and counts, never a sentence about them.
    pub fn frame(&self) -> String {
        let source = si_core::SOURCE_LITERALS[1];
        match &self.verdict {
            Some(verdict) => format!(
                "{{\"t\":{},\"source\":\"{source}\",\"line\":\"{}\",\"admitted\":{},\
                 \"reason\":\"{}\",\"layer\":{},\"n_untaught_tokens\":{},\"novel_claims\":{},\
                 \"ref_row_id\":{},\"untaught\":[{}]}}",
                self.t,
                escape(&self.line),
                verdict.admitted,
                verdict.reason,
                verdict.layer,
                verdict.n_untaught_tokens,
                verdict.novel_claims,
                match verdict.ref_row_id {
                    Some(row_id) => row_id.to_string(),
                    None => "null".to_string(),
                },
                verdict
                    .untaught
                    .iter()
                    .map(|key| format!("\"{}\"", escape(key)))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            None => format!(
                "{{\"t\":{},\"source\":\"{source}\",\"line\":\"{}\",\"admitted\":false,\
                 \"reason\":\"seam_closed\",\"layer\":0,\"n_untaught_tokens\":0,\
                 \"novel_claims\":0,\"ref_row_id\":null,\"untaught\":[]}}",
                self.t,
                escape(&self.line)
            ),
        }
    }
}

/// docs/21 §21.7's append-only gate log: one line per attempt, admitted or
/// rejected. It is an **observation** — nothing in the build reads it back.
#[derive(Default)]
pub struct GateLog {
    path: Option<PathBuf>,
    pub attempts: Vec<Attempt>,
}

impl GateLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn at(path: impl AsRef<Path>) -> Self {
        Self { path: Some(path.as_ref().to_path_buf()), attempts: Vec::new() }
    }

    pub fn record(&mut self, attempt: Attempt) {
        self.attempts.push(attempt);
    }

    pub fn flush(&self) -> Result<()> {
        let Some(path) = &self.path else { return Ok(()) };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        let mut text = String::new();
        for attempt in &self.attempts {
            text.push_str(&attempt.frame());
            text.push('\n');
        }
        std::fs::write(path, text).with_context(|| format!("write {}", path.display()))
    }

    /// The log's lines in order, without writing them anywhere. `05 §17`'s
    /// observation artifact is exported from these; nothing reads them back.
    pub fn frames(&self) -> Vec<String> {
        self.attempts.iter().map(Attempt::frame).collect()
    }
}

/// The seam, with its log. One per run: it is the process-C side of the wire.
pub struct Seam {
    pub state: SeamState,
    pub log: GateLog,
    /// OD-40(iii): every read is logged, so a tool that quietly became a channel
    /// is visible in the same artifact as the emissions.
    pub reads: Vec<String>,
    artifacts: Option<PathBuf>,
}

impl Seam {
    pub fn new(state: SeamState, log: GateLog) -> Self {
        Self { state, log, reads: Vec::new(), artifacts: None }
    }

    pub fn closed() -> Self {
        Self::new(SeamState::Closed, GateLog::new())
    }

    /// The harness's constructor: one seam per section, launched at the run's
    /// variant, writing its gate log into the run directory.
    pub fn for_variant(variant: &str, log_path: impl AsRef<Path>) -> Self {
        Self::new(
            SeamState::from_variant(variant),
            GateLog::at(log_path),
        )
    }

    /// The gate log's lines, in attempt order.
    pub fn frames(&self) -> Vec<String> {
        self.log.frames()
    }

    pub fn admissions(&self) -> usize {
        self.log
            .attempts
            .iter()
            .filter(|attempt| attempt.verdict.as_ref().is_some_and(|verdict| verdict.admitted))
            .count()
    }

    pub fn with_artifacts(mut self, dir: impl AsRef<Path>) -> Self {
        self.artifacts = Some(dir.as_ref().to_path_buf());
        self
    }

    /// 04 §23: attempt one emission. The store is borrowed read-only, and a
    /// rejection returns before the caller can hand the line to `tick`, so no
    /// sense event, no signal, no `a` movement, no stamp and no row exists.
    pub fn speak(&mut self, store: &Lattice, line: &str) -> Result<Admission> {
        if !self.state.admits() {
            self.log.record(Attempt {
                t: store.now(),
                line: line.to_string(),
                verdict: None,
                dropped: true,
            });
            return Ok(Admission::Dropped);
        }
        let verdict = speakgate::check(store, line)?;
        self.log.record(Attempt {
            t: store.now(),
            line: line.to_string(),
            verdict: Some(verdict.clone()),
            dropped: false,
        });
        Ok(if verdict.admitted { Admission::Admitted(verdict) } else { Admission::Rejected(verdict) })
    }

    /// docs/21 §21.2's five readers. Each returns bytes SI already published for
    /// the owner, logs that it was read, and writes nothing anywhere.
    pub fn read(&mut self, store: &Lattice, tool: &str) -> Result<String> {
        let body = match tool {
            "read_trace" | "read_state" | "read_monologue" => {
                let name = if tool == "read_monologue" { "monologue.jsonl" } else { "mindlog.json" };
                let text = self.artifact(name)?;
                format!("{{\"artifact\":\"{name}\",\"bytes\":{}}}", text.len())
            }
            "read_vocab" => {
                let lexicon = speakgate::lexicon(store)?;
                format!(
                    "{{\"taught\":[{}],\"n_taught\":{}}}",
                    lexicon
                        .taught
                        .iter()
                        .map(|key| format!("\"{}\"", escape(key)))
                        .collect::<Vec<_>>()
                        .join(","),
                    lexicon.taught.len()
                )
            }
            "read_rows" => {
                let lexicon = speakgate::lexicon(store)?;
                format!(
                    "{{\"claims\":[{}],\"n_live\":{}}}",
                    lexicon
                        .claims
                        .iter()
                        .map(|claim| format!(
                            "{{\"row_id\":{},\"keys\":[{}]}}",
                            claim.row_id,
                            claim
                                .keys
                                .iter()
                                .map(|key| format!("\"{}\"", escape(key)))
                                .collect::<Vec<_>>()
                                .join(",")
                        ))
                        .collect::<Vec<_>>()
                        .join(","),
                    lexicon.claims.len()
                )
            }
            other => anyhow::bail!("{other} is not one of the five read-only tools"),
        };
        self.reads.push(format!(
            "{{\"t\":{},\"tool\":\"{tool}\",\"bytes\":{}}}",
            store.now(),
            body.len()
        ));
        Ok(body)
    }

    fn artifact(&self, name: &str) -> Result<String> {
        let Some(dir) = self.artifacts.as_ref() else {
            // A seam launched without a run directory has no lane to read. The
            // cross-variant probe runs sections in memory, so this is the normal
            // shape of a read there: bytes 0, and still no write anywhere.
            return Ok(String::new());
        };
        let path = dir.join(name);
        match std::fs::read_to_string(&path) {
            Ok(text) => Ok(text),
            // An artifact that does not exist yet reads as empty, not as an error:
            // a tool that cannot find a lane has no lane to leak.
            Err(_) => Ok(String::new()),
        }
    }

    /// 00 §7's frame transport: one request per line on the reader, one frame per
    /// line on the writer. The harness drives [`Seam::speak`] and [`Seam::read`]
    /// directly, which is why this loop holds no state of its own.
    pub fn serve<R: BufRead, W: Write>(
        &mut self,
        store: &Lattice,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<usize> {
        let mut admissions = 0usize;
        let mut frame = String::new();
        loop {
            frame.clear();
            let read = reader.read_line(&mut frame).context("read one seam frame from stdin")?;
            if read == 0 {
                break;
            }
            let request = frame.trim_end();
            if request.is_empty() {
                continue;
            }
            let response = if let Some(line) = request.strip_prefix("speak ") {
                match self.speak(store, line)? {
                    Admission::Admitted(verdict) => {
                        admissions += 1;
                        format!(
                            "{{\"admitted\":true,\"ref_row_id\":{}}}",
                            match verdict.ref_row_id {
                                Some(row_id) => row_id.to_string(),
                                None => "null".to_string(),
                            }
                        )
                    }
                    Admission::Rejected(verdict) => format!(
                        "{{\"admitted\":false,\"reason\":\"{}\",\"n_untaught_tokens\":{},\
                         \"novel_claims\":{},\"ref_row_id\":{}}}",
                        verdict.reason,
                        verdict.n_untaught_tokens,
                        verdict.novel_claims,
                        match verdict.ref_row_id {
                            Some(row_id) => row_id.to_string(),
                            None => "null".to_string(),
                        }
                    ),
                    Admission::Dropped => "{\"admitted\":false,\"reason\":\"seam_closed\"}".into(),
                }
            } else {
                self.read(store, request)?
            };
            writeln!(writer, "{response}").context("write one seam frame to stdout")?;
        }
        Ok(admissions)
    }

    pub fn flush(&self) -> Result<()> {
        self.log.flush()
    }
}

/// The frames are JSON and the lines are owner bytes, so the two escapes needed
/// are quote and backslash. Everything else travels verbatim.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
    out
}
